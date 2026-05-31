//! OmniFormatter CLI Host
//!
//! A standalone CLI that downloads WASM formatters from the registry,
//! compiles them with Wasmtime, and executes the `format` interface.
//!
//! Security (L-02): Executes third-party formatters in a sandboxed WebAssembly
//! environment (Wasmtime) with no filesystem or network access.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use protocol::config::ConfigIR;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use wasmtime::*;

const REGISTRY_URL: &str = "https://registry.omnifmt.dev";

#[derive(Parser, Debug)]
#[command(
    name = "omnifmt",
    version,
    about = "Universal, blazing fast, WebAssembly-powered code formatter."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Format files in place
    Format {
        /// Files or directories to format
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Path to the local WASM module (overrides registry)
        #[arg(long)]
        plugin: Option<PathBuf>,

        /// Assume this language module (e.g. 'lang-rust')
        #[arg(long)]
        module: Option<String>,
    },
    /// Print formatting result to stdout instead of editing files
    Print {
        /// File to format
        #[arg(required = true)]
        path: PathBuf,

        /// Path to the local WASM module
        #[arg(long)]
        plugin: Option<PathBuf>,
    },
}

#[derive(Deserialize)]
struct RegistryResolveResponse {
    name: String,
    version: String,
    sha256: String,
    download_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Format {
            paths,
            plugin,
            module,
        } => {
            for path in paths {
                if path.is_file() {
                    format_file(&path, plugin.as_deref(), module.as_deref(), true).await?;
                } else if path.is_dir() {
                    for entry in ignore::Walk::new(&path) {
                        let entry = match entry {
                            Ok(e) => e,
                            Err(_) => continue,
                        };
                        if entry.path().is_file() {
                            let _ = format_file(
                                entry.path(),
                                plugin.as_deref(),
                                module.as_deref(),
                                true,
                            )
                            .await;
                        }
                    }
                }
            }
        }
        Commands::Print { path, plugin } => {
            format_file(&path, plugin.as_deref(), None, false).await?;
        }
    }

    Ok(())
}

async fn format_file(
    path: &Path,
    plugin_path: Option<&Path>,
    module_override: Option<&str>,
    in_place: bool,
) -> Result<()> {
    let source = fs::read(path).context("Failed to read source file")?;

    // Determine module name from extension
    let module_name = if let Some(m) = module_override {
        m.to_string()
    } else {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => "lang-rust".to_string(),
            Some("py") => "lang-python".to_string(),
            Some("go") => "lang-go".to_string(),
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => "lang-js".to_string(),
            Some("css") | Some("scss") | Some("less") | Some("html") => "lang-css".to_string(),
            Some(ext) => bail!("No known module for extension: {}", ext),
            None => bail!("File has no extension"),
        }
    };

    // Load or download WASM binary
    let wasm_bytes = if let Some(p) = plugin_path {
        fs::read(p).context("Failed to read local plugin WASM")?
    } else {
        fetch_from_registry(&module_name).await?
    };

    // Prepare configuration
    let config = ConfigIR::default(); // TODO: read local .editorconfig/.omnifmt.json
    let config_json = serde_json::to_string(&config)?;

    // Execute WASM
    let formatted_bytes = execute_wasm_format(&wasm_bytes, &source, &config_json, &module_name)?;

    if in_place {
        if source != formatted_bytes {
            fs::write(path, &formatted_bytes)?;
            println!("Formatted: {}", path.display());
        } else {
            println!("Unchanged: {}", path.display());
        }
    } else {
        use std::io::Write;
        std::io::stdout().write_all(&formatted_bytes)?;
    }

    Ok(())
}

async fn fetch_from_registry(module_name: &str) -> Result<Vec<u8>> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".omnifmt_cache"))
        .join("omnifmt/plugins");
    fs::create_dir_all(&cache_dir)?;

    // 1. Resolve latest version
    let client = reqwest::Client::new();
    let url = format!("{}/resolve/{}", REGISTRY_URL, module_name);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        bail!(
            "Registry failed to resolve {}: HTTP {}",
            module_name,
            res.status()
        );
    }
    let resolve_data: RegistryResolveResponse = res.json().await?;

    let wasm_path = cache_dir.join(format!("{}-{}.wasm", module_name, resolve_data.version));

    // 2. Check cache and verify SHA
    if wasm_path.exists() {
        let bytes = fs::read(&wasm_path)?;
        let hash = sha256_hex(&bytes);
        if hash == resolve_data.sha256 {
            return Ok(bytes);
        }
    }

    // 3. Download from R2
    println!("Downloading {}@{}...", module_name, resolve_data.version);
    let wasm_res = client.get(&resolve_data.download_url).send().await?;
    if !wasm_res.status().is_success() {
        bail!("Failed to download WASM: HTTP {}", wasm_res.status());
    }
    let wasm_bytes = wasm_res.bytes().await?.to_vec();

    // 4. Verify integrity
    let hash = sha256_hex(&wasm_bytes);
    if hash != resolve_data.sha256 {
        bail!(
            "Integrity check failed for {}! Expected {}, got {}",
            module_name,
            resolve_data.sha256,
            hash
        );
    }

    // 5. Save to cache
    fs::write(&wasm_path, &wasm_bytes)?;

    Ok(wasm_bytes)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ── Wasmtime execution ────────────────────────────────────────────────────

fn execute_wasm_format(
    wasm_bytes: &[u8],
    source: &[u8],
    config_json: &str,
    _module_name: &str,
) -> Result<Vec<u8>> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes).context("Failed to compile WASM module")?;

    let mut store = Store::new(&engine, ());
    let instance =
        Instance::new(&mut store, &module, &[]).context("Failed to instantiate WASM module")?;

    // Exported memory and allocator
    let memory = instance
        .get_memory(&mut store, "memory")
        .context("WASM missing exported 'memory'")?;
    let alloc = instance
        .get_typed_func::<u32, u32>(&mut store, "alloc")
        .context("WASM missing exported 'alloc'")?;
    let dealloc = instance
        .get_typed_func::<(u32, u32), ()>(&mut store, "dealloc")
        .context("WASM missing exported 'dealloc'")?;

    // Exported format function
    let format_func = instance
        .get_typed_func::<(u32, u32, u32, u32), u64>(&mut store, "format")
        .context("WASM missing exported 'format' function")?;

    // 1. Allocate and write source buffer
    let source_ptr = alloc.call(&mut store, source.len() as u32)?;
    memory.write(&mut store, source_ptr as usize, source)?;

    // 2. Allocate and write config JSON buffer
    let config_bytes = config_json.as_bytes();
    let config_ptr = alloc.call(&mut store, config_bytes.len() as u32)?;
    memory.write(&mut store, config_ptr as usize, config_bytes)?;

    // 3. Call `format`
    let result_u64 = format_func.call(
        &mut store,
        (
            source_ptr,
            source.len() as u32,
            config_ptr,
            config_bytes.len() as u32,
        ),
    )?;

    // Cleanup input allocations
    dealloc.call(&mut store, (source_ptr, source.len() as u32))?;
    dealloc.call(&mut store, (config_ptr, config_bytes.len() as u32))?;

    // Parse result pointer (u64 = [ptr: u32] | [len: u32])
    let out_ptr = (result_u64 >> 32) as u32;
    let out_len = (result_u64 & 0xFFFFFFFF) as u32;

    if out_ptr == 0 {
        // Parse error or passthrough; returned NULL ptr. Return original source.
        return Ok(source.to_vec());
    }

    // 4. Read result from memory
    let mut out_buffer = vec![0u8; out_len as usize];
    memory.read(&mut store, out_ptr as usize, &mut out_buffer)?;

    // Cleanup output allocation
    dealloc.call(&mut store, (out_ptr, out_len))?;

    Ok(out_buffer)
}
