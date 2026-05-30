//! Rust Language Module
//!
//! Implements the OmniFormatterModule interface for Rust (.rs).
//! Targets rustfmt stable output parity (L-08 mitigation).
//!
//! # rustfmt Compat Mode
//!
//! In "opinionated" mode (default), output matches `rustfmt --edition 2021`
//! with the stable config profile byte-for-byte.
//!
//! Key rustfmt rules enforced:
//! - `max_width` → `ConfigIR.print_width` (default 100, not 80)
//! - `tab_spaces` → `ConfigIR.indent_size` (default 4)
//! - `edition` → `"2021"` by default
//! - `use_small_heuristics` → `"Default"` (matches rustfmt stable default)
//!
//! # Config Adapter
//!
//! Reads `rustfmt.toml` or `.rustfmt.toml`:
//! - `max_width` → `print_width`
//! - `tab_spaces` → `indent_size`
//! - `hard_tabs` → `indent_style`
//! - `newline_style` → `end_of_line`
//!
//! The `// rustfmt::skip` suppression comment is detected by the core
//! comment anchoring engine (crates/core/src/comments.rs).
//!
//! # Implementation Status
//!
//! Phase 4 scaffold. Format logic is a pass-through stub.

pub mod adapter;
pub mod format;

use wasm_bindgen::prelude::*;
use protocol::ConfigIR;

#[wasm_bindgen]
pub fn format_rust(source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
    match format::format(source_bytes, &config) {
        Ok(f) => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn config_schema() -> String { include_str!("../schema.json").to_string() }

#[wasm_bindgen]
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }

#[wasm_bindgen]
pub fn language_id() -> &'static str { "rust" }

#[wasm_bindgen]
pub fn aliases() -> Vec<JsValue> {
    vec![JsValue::from_str(".rs")]
}
