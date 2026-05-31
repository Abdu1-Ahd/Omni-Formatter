//! JS/TS/JSX/TSX Language Module
//!
//! Implements the OmniFormatterModule interface for JavaScript, TypeScript,
//! JSX, and TSX. Targets Prettier 3.x output parity (Pillar 1, L-08).
//!
//! # Module Interface
//!
//! Every language module exports these five functions:
//!
//! ```text
//! format(source: &[u8], config_json: &str) -> Result<Vec<u8>, FormatError>
//! config_schema() -> &'static str   // JSON Schema for this module's config
//! version() -> &'static str         // semver string
//! language_id() -> &'static str     // VS Code languageId
//! aliases() -> Vec<String>          // file extensions this module handles
//! ```
//!
//! # Prettier 3.x Compat Mode
//!
//! By default, this module operates in "opinionated" mode (Pillar 7, L-12):
//! output matches Prettier 3.x byte-for-byte. The `.omnifmt.json` option
//! `"mode": "advanced"` unlocks the full option surface and voids compat.
//!
//! # Implementation Status
//!
//! Phase 3 scaffold: the module interface is defined and all config types
//! are implemented. The `format()` function is a pass-through stub.
//! Full Prettier-parity formatting logic is implemented in Phase 4 when
//! the Tree-sitter JS/TS grammar is integrated.

pub mod adapter;
pub mod compat;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;
use protocol::{ConfigIR, FormatError};
use serde_json;

/// Format JavaScript, TypeScript, JSX, or TSX source.
///
/// # Arguments
///
/// * `source_bytes` — UTF-8 source to format.
/// * `config_json` — JSON-serialised `ConfigIR`.
///
/// # Returns
///
/// Formatted UTF-8 bytes on success, or a JSON-serialised `FormatError` on failure.
#[wasm_bindgen]
pub fn format_js(source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();

    match format::format(source_bytes, &config) {
        Ok(formatted) => Ok(formatted),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Returns the JSON Schema describing this module's configuration options.
#[wasm_bindgen]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

/// Returns the module's semver version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns the primary VS Code languageId this module handles.
#[wasm_bindgen]
pub fn language_id() -> String {
    "javascript".to_string()
}

/// Returns all VS Code languageIds and file extensions this module handles.
#[wasm_bindgen]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("typescript"),
        JsValue::from_str("javascriptreact"),
        JsValue::from_str("typescriptreact"),
        JsValue::from_str(".js"),
        JsValue::from_str(".mjs"),
        JsValue::from_str(".cjs"),
        JsValue::from_str(".ts"),
        JsValue::from_str(".mts"),
        JsValue::from_str(".cts"),
        JsValue::from_str(".jsx"),
        JsValue::from_str(".tsx"),
    ]
}
