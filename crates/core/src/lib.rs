//! OmniFormatter WASM Core
//!
//! This is the single entry point compiled to WebAssembly.
//! It exports one function — `format()` — that the extension host calls
//! via the worker thread pool.
//!
//! Architecture (Tier 3):
//!   Extension Host → postMessage → Worker → WASM `format()` → postMessage → Extension Host
//!
//! ## Zero-copy source path
//!
//! The extension host sends `source_text: String` (raw UTF-8 JSON string).
//! No byte-array encoding. No base64. WASM receives one JSON parse, done.
//! Unlimited file size — no artificial caps.

pub mod arena;
pub mod comments;
pub mod debug;
pub mod incremental;
pub mod memory;
#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

pub mod range;
pub mod registry;
pub mod stitch;
pub mod unicode;
#[cfg(target_arch = "wasm32")]
pub mod wasm_stdlib;
pub mod zones;

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn js_log(_s: &str) {}

#[wasm_bindgen(start)]
pub fn init_wasm() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_stdlib::init_stubs();
        console_error_panic_hook::set_once();
    }
}

use protocol::{FormatRequest, FormatResponse, TextEdit};

/// The single exported WASM function.
///
/// # Arguments
///
/// * `request_json` — a JSON-serialised `FormatRequest`.
///   The extension host constructs this after converting VS Code UTF-16
///   positions to UTF-8 byte offsets.
///
/// # Returns
///
/// A JSON-serialised `FormatResponse`.
/// On error, returns a JSON-serialised `FormatError` wrapped in an object:
/// `{ "error": <FormatError> }`.
///
/// # Panics (debug builds only)
///
/// In debug builds with the `debug` feature enabled, this function double-formats
/// the source and panics if `format(format(x)) != format(x)` (L-09 mitigation).
#[wasm_bindgen]
pub fn format(request_json: &str) -> String {
    js_log("Entered format()...");

    // Initialise the arena allocator for this request (L-01 mitigation).
    let _arena = arena::RequestArena::new();
    js_log("Arena initialized.");

    // Parse the incoming request.
    js_log("Deserializing request...");
    let request: FormatRequest = match serde_json::from_str(request_json) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::json!({
                "error": {
                    "kind": "internal",
                    "detail": { "message": format!("Failed to deserialise FormatRequest: {}", e) }
                }
            })
            .to_string();
        }
    };
    js_log("Request deserialized.");

    // ── Resolve source bytes — no size cap, no encoding overhead ──────────
    // `source_bytes()` checks `source_text` first (zero-copy string path from
    // the extension host), then falls back to the legacy `source: Vec<u8>` for
    // CLI / test compatibility. File size is unlimited.
    let source = request.source_bytes();

    // ── Language dispatch ──────────────────────────────────────────────────
    js_log("Normalizing language ID...");
    let ext = language_id_to_ext(&request.language_id);
    js_log(&format!("Language mapped to ext: {}", ext));

    js_log("Fetching default registry...");
    let registry = registry::default_registry();
    let config = &request.config;

    js_log("Calling registry.format_by_ext...");
    let formatted = match registry.format_by_ext(ext, &source, config) {
        Ok(bytes) => bytes,
        Err(e) => {
            return serde_json::json!({
                "error": {
                    "kind": "format_failed",
                    "detail": { "message": format!("{}", e) }
                }
            })
            .to_string();
        }
    };

    // ── Diff generation ────────────────────────────────────────────────────
    // Emit a single whole-document replacement edit when the output differs.
    let (edits, is_noop) = if formatted == source {
        (Vec::new(), true)
    } else {
        let new_text = match std::str::from_utf8(&formatted) {
            Ok(s) => s.to_string(),
            Err(_) => {
                return serde_json::json!({
                    "error": {
                        "kind": "internal",
                        "detail": { "message": "formatter produced invalid UTF-8" }
                    }
                })
                .to_string();
            }
        };
        let edit = TextEdit {
            range: protocol::ByteRange {
                start: 0,
                end: source.len(),
            },
            new_text,
        };
        (vec![edit], false)
    };

    let chain = format!("OmniFormatter {} (lang-{})", env!("CARGO_PKG_VERSION"), ext);

    let response = FormatResponse {
        edits,
        formatter_chain: chain,
        is_noop,
    };

    // Serialise the response.
    match serde_json::to_string(&response) {
        Ok(json) => json,
        Err(e) => serde_json::json!({
            "error": {
                "kind": "internal",
                "detail": { "message": format!("Failed to serialise FormatResponse: {}", e) }
            }
        })
        .to_string(),
    }
}

/// Map VS Code language identifiers to the extension keys the registry uses.
///
/// VS Code language IDs don't always match file extensions, so we normalise
/// before looking up the registry. The fallback (`other => other`) means any
/// new language added to `LANGUAGE_MODULE_MAP` in the extension host works
/// automatically without a Rust rebuild.
fn language_id_to_ext(language_id: &str) -> &str {
    match language_id {
        // ── Frontend & Web ───────────────────────────────────────────────
        "javascript" | "javascriptreact" => "js",
        "typescript" | "typescriptreact" => "ts",
        "svelte" => "svelte",
        "vue" => "vue",
        "astro" => "astro",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "html" => "html",
        // ── Systems ──────────────────────────────────────────────────────
        "c" => "c",
        "cpp" | "cuda-cpp" => "cpp",
        "objective-c" => "m",
        "objective-cpp" => "mm",
        "rust" => "rs",
        "go" => "go",
        "zig" => "zig",
        "nim" => "nim",
        "d" => "d",
        // ── JVM & .NET ───────────────────────────────────────────────────
        "java" => "java",
        "kotlin" => "kt",
        "scala" => "scala",
        "groovy" => "groovy",
        "csharp" => "cs",
        "fsharp" => "fs",
        // ── Scripting ────────────────────────────────────────────────────
        "python" => "py",
        "ruby" => "rb",
        "php" => "php",
        "perl" => "pl",
        "r" => "r",
        "julia" => "jl",
        "lua" => "lua",
        // ── Shell ────────────────────────────────────────────────────────
        "shellscript" => "sh",
        "powershell" => "ps1",
        "zsh" => "zsh",
        // ── Mobile ───────────────────────────────────────────────────────
        "swift" => "swift",
        "dart" => "dart",
        // ── Data & Config ────────────────────────────────────────────────
        "json" | "json5" | "jsonc" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "ini" => "ini",
        // ── Query ────────────────────────────────────────────────────────
        "sql" => "sql",
        "graphql" => "graphql",
        // ── DevOps ───────────────────────────────────────────────────────
        "terraform" => "tf",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "nix" => "nix",
        // ── Functional ───────────────────────────────────────────────────
        "haskell" => "hs",
        "elixir" => "ex",
        "erlang" => "erl",
        "ocaml" => "ml",
        "clojure" => "clj",
        "lisp" => "lisp",
        "scheme" => "scm",
        // ── Docs ─────────────────────────────────────────────────────────
        "markdown" => "md",
        "latex" => "tex",
        // ── Blockchain ───────────────────────────────────────────────────
        "solidity" => "sol",
        // ── Game & Automation ────────────────────────────────────────────
        "gdscript" => "gd",
        "ahk" => "ahk",
        // ── Stubs ────────────────────────────────────────────────────────
        "cobol" => "cob",
        "fortran" => "f90",
        "asm" => "asm",
        // ── Templates ────────────────────────────────────────────────────
        "jinja" => "jinja",
        "liquid" => "liquid",
        "ejs" => "ejs",
        "handlebars" => "hbs",
        "twig" => "twig",
        // ── Fallback: pass language_id verbatim ──────────────────────────
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Preferred path: source_text string — zero-copy, unlimited size.
    #[test]
    fn format_source_text_path() {
        let request = serde_json::json!({
            "source_text": "hello",
            "language_id": "typescript",
            "config": {},
            "range": null,
            "previous_tree": null,
            "edit": null
        });
        let result = format(&request.to_string());
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("format() must return valid JSON");
        assert!(parsed.get("edits").is_some(), "must have 'edits'");
        assert!(parsed.get("is_noop").is_some(), "must have 'is_noop'");
    }

    /// Legacy path: source as byte array — kept for CLI / backwards compat.
    #[test]
    fn format_legacy_source_bytes_path() {
        let request = serde_json::json!({
            "source": [104, 101, 108, 108, 111], // b"hello"
            "language_id": "typescript",
            "config": {},
            "range": null,
            "previous_tree": null,
            "edit": null
        });
        let result = format(&request.to_string());
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("format() must return valid JSON");
        assert!(parsed.get("edits").is_some(), "must have 'edits'");
        assert!(parsed.get("is_noop").is_some(), "must have 'is_noop'");
    }

    /// Large file: no size cap. 20 MB of 'a' must format (or pass-through) cleanly.
    #[test]
    fn format_large_file_no_cap() {
        let big_source = "a".repeat(20 * 1024 * 1024); // 20 MB
        let request = serde_json::json!({
            "source_text": big_source,
            "language_id": "typescript",
            "config": {},
            "range": null,
            "previous_tree": null,
            "edit": null
        });
        let result = format(&request.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        // Must NOT return file_too_large
        assert_ne!(
            parsed.get("error").and_then(|e| e.get("kind")),
            Some(&serde_json::Value::String("file_too_large".to_string())),
            "large files must not be rejected"
        );
    }
}
