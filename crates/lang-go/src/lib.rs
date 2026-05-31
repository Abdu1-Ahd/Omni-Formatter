//! Go Language Module
//!
//! Implements the OmniFormatterModule interface for Go (.go).
//! Targets `gofmt` output parity (L-08 mitigation).
//!
//! # gofmt Parity
//!
//! gofmt is opinionated and has zero configuration: it has one correct
//! formatting for every valid Go program. OmniFormatter's Go module matches
//! this output byte-for-byte in opinionated mode.
//!
//! Key gofmt rules:
//! - Tabs for indentation (mandatory — Go has no tab/space option)
//! - No configurable line length (gofmt does not enforce one)
//! - `goimports` style: organise and group imports
//!
//! # Config Adapter
//!
//! Go has no config file for gofmt. The adapter only reads `.editorconfig`
//! for `end_of_line`. All other settings are ignored in opinionated mode.
//!
//! # Implementation Status
//!
//! Phase 4 scaffold. Format logic is pass-through stub.

pub mod adapter;
pub mod format;
pub mod plugin;

use protocol::ConfigIR;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn format_go(source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config = adapter::config_from_go_json(config_json);
    match format::format(source_bytes, &config) {
        Ok(f) => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn language_id() -> String {
    "go".to_string()
}

#[wasm_bindgen]
pub fn aliases() -> Vec<JsValue> {
    vec![JsValue::from_str(".go")]
}
