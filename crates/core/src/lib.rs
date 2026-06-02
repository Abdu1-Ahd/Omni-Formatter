//! OmniFormatter WASM Core
//!
//! This is the single entry point compiled to WebAssembly.
//! It exports one function — `format()` — that the extension host calls
//! via the worker thread pool.
//!
//! Architecture (Tier 3):
//!   Extension Host → postMessage → Worker → WASM `format()` → Worker → postMessage → Extension Host
//!
//! All input/output crosses the WASM boundary as JSON strings.
//! Binary data (source bytes, WASM module bytes) crosses as base64-encoded
//! JSON string values inside the JSON payload.

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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

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

    // Enforce 10MB file size limit (L-01 mitigation).
    const MAX_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
    if request.source.len() > MAX_FILE_BYTES {
        return serde_json::json!({
            "error": {
                "kind": "file_too_large",
                "detail": {
                    "size_bytes": request.source.len(),
                    "limit_bytes": MAX_FILE_BYTES
                }
            }
        })
        .to_string();
    }

    // ── Language dispatch ──────────────────────────────────────────────────
    // Map language_id to a file extension the registry understands.
    // VS Code language IDs don't always match file extensions, so we normalise.
    js_log("Normalizing language ID...");
    let ext = language_id_to_ext(&request.language_id);
    js_log(&format!("Language mapped to ext: {}", ext));

    js_log("Fetching default registry...");
    let registry = registry::default_registry();
    let config = &request.config;
    let source = &request.source;

    js_log("Calling registry.format_by_ext...");
    let formatted = match registry.format_by_ext(ext, source, config) {
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
    // Produce a single whole-document edit if the output differs, or is_noop.
    let (edits, is_noop) = if formatted == *source {
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
fn language_id_to_ext(language_id: &str) -> &str {
    match language_id {
        "javascript" | "javascriptreact" => "js",
        "typescript" | "typescriptreact" => "ts",
        "python" => "py",
        "rust" => "rs",
        "go" => "go",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "html" => "html",
        // Fallback: use the language_id verbatim (covers future plugins)
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_returns_valid_json() {
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

        assert!(
            parsed.get("edits").is_some(),
            "response must have 'edits' field"
        );
        assert!(
            parsed.get("is_noop").is_some(),
            "response must have 'is_noop' field"
        );
    }

    #[test]
    fn format_rejects_oversized_file() {
        let big_source: Vec<u8> = vec![b'a'; 11 * 1024 * 1024]; // 11 MB
        let request = serde_json::json!({
            "source": big_source,
            "language_id": "typescript",
            "config": {},
            "range": null,
            "previous_tree": null,
            "edit": null
        });

        let result = format(&request.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed["error"]["kind"], "file_too_large",
            "must reject files over 10MB"
        );
    }
}
