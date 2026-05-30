//! omnifmt — Command-line interface for OmniFormatter
//!
//! This binary uses the same WASM core and language modules as the VS Code extension.
//! It resolves config files using the same priority chain as the extension host.
//!
//! # Commands
//!
//! - `omnifmt format <files>` — Format files in-place
//! - `omnifmt check <files>` — Check if files are already formatted (exit 1 if not)
//! - `omnifmt config <file>` — Show the resolved ConfigIR for a file
//! - `omnifmt modules list` — List installed language modules
//! - `omnifmt modules install <name>` — Install a community module
//! - `omnifmt modules verify <name>` — Verify a module's SHA-256 integrity
//!
//! # Implementation Status
//!
//! Phase 5 scaffold. CLI argument parsing and help text are complete.
//! WASM invocation requires the Wasmtime host runtime (added in Phase 5 continuation).

use clap::{Parser, Subcommand};

/// OmniFormatter CLI — universal code formatter
#[derive(Parser, Debug)]
#[command(
    name = "omnifmt",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal code formatter — all languages, zero config migration",
    long_about = "OmniFormatter uses the same WASM core as the VS Code extension.\n\
                  It reads your existing config files (.prettierrc, pyproject.toml, etc.) automatically."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Format files in-place. Pass - to read from stdin.
    Format {
        /// Files to format. Use - for stdin.
        #[arg(required = true)]
        files: Vec<String>,

        /// Override the language (uses file extension by default).
        #[arg(long, short)]
        language: Option<String>,

        /// Write output to stdout instead of modifying files.
        #[arg(long, short)]
        stdout: bool,

        /// Do not print formatting results.
        #[arg(long, short)]
        quiet: bool,
    },

    /// Check if files are already formatted. Exit 1 if any file would change.
    Check {
        /// Files to check.
        #[arg(required = true)]
        files: Vec<String>,

        /// List only files that would change (machine-readable output).
        #[arg(long)]
        list_files: bool,
    },

    /// Show the resolved configuration for a file.
    Config {
        /// The file to resolve config for.
        file: String,

        /// Output as pretty-printed JSON.
        #[arg(long, default_value = "true")]
        pretty: bool,
    },

    /// Manage language modules.
    Modules {
        #[command(subcommand)]
        action: ModulesAction,
    },
}

#[derive(Subcommand, Debug)]
enum ModulesAction {
    /// List all installed language modules.
    List,

    /// Install a community module from the registry.
    Install {
        /// Module name (e.g. lang-toml, lang-zig).
        name: String,
    },

    /// Verify the SHA-256 integrity of an installed module.
    Verify {
        /// Module name.
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Format { files, language, stdout, quiet } => {
            cmd_format(files, language, stdout, quiet);
        }
        Command::Check { files, list_files } => {
            cmd_check(files, list_files);
        }
        Command::Config { file, pretty } => {
            cmd_config(file, pretty);
        }
        Command::Modules { action } => {
            cmd_modules(action);
        }
    }
}

fn cmd_format(files: Vec<String>, language: Option<String>, stdout: bool, quiet: bool) {
    // Phase 5 stub: print what would happen.
    let _ = (language, stdout);
    for file in &files {
        if !quiet {
            eprintln!("[omnifmt] Would format: {}", file);
        }
    }
    eprintln!("[omnifmt] Format command: Phase 5 stub. WASM host runtime not yet integrated.");
    std::process::exit(0);
}

fn cmd_check(files: Vec<String>, list_files: bool) {
    // Phase 5 stub.
    for file in &files {
        if list_files {
            println!("{}", file);
        } else {
            eprintln!("[omnifmt] Would check: {}", file);
        }
    }
    eprintln!("[omnifmt] Check command: Phase 5 stub. WASM host runtime not yet integrated.");
    std::process::exit(0);
}

fn cmd_config(file: String, _pretty: bool) {
    // Phase 5 stub: print default ConfigIR.
    let config = protocol::ConfigIR::default();
    match serde_json::to_string_pretty(&config) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serialising config: {}", e);
            std::process::exit(2);
        }
    }
    eprintln!("[omnifmt] Note: showing default ConfigIR. Config file reading in Phase 5.");
    let _ = file;
}

fn cmd_modules(action: ModulesAction) {
    match action {
        ModulesAction::List => {
            println!("Installed modules:");
            println!("  lang-js     0.1.0  javascript, typescript, jsx, tsx");
            println!("  lang-python 0.1.0  python");
            println!("  lang-rust   0.1.0  rust");
            println!("  lang-go     0.1.0  go");
            println!("  lang-css    0.1.0  css, scss, less, html");
        }
        ModulesAction::Install { name } => {
            eprintln!("[omnifmt] Registry not yet live. Cannot install: {}", name);
            eprintln!("[omnifmt] Community module registry launches in Phase 5.");
            std::process::exit(2);
        }
        ModulesAction::Verify { name } => {
            eprintln!("[omnifmt] Verify: {} — registry not yet live.", name);
            std::process::exit(2);
        }
    }
}
