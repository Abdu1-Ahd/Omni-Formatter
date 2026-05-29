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
pub mod range;
pub mod stitch;
pub mod unicode;
pub mod zones;

use wasm_bindgen::prelude::*;

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
    // Initialise the arena allocator for this request (L-01 mitigation).
    // The arena is dropped at the end of this function, freeing all parse tree
    // nodes in a single deallocation.
    let _arena = arena::RequestArena::new();

    // Parse the incoming request.
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

    // Phase 1 stub: return source unchanged.
    // Language module dispatch, zone detection, comment anchoring, and diff
    // generation are implemented in Phases 3 and 4.
    let response = FormatResponse {
        edits: Vec::new(),        // No edits = source is already "formatted"
        formatter_chain: format!("OmniFormatter core (stub) for {}", request.language_id),
        is_noop: true,
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

        assert!(parsed.get("edits").is_some(), "response must have 'edits' field");
        assert!(parsed.get("is_noop").is_some(), "response must have 'is_noop' field");
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
            parsed["error"]["kind"],
            "file_too_large",
            "must reject files over 10MB"
        );
    }
}
