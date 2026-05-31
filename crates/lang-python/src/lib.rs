//! Python Language Module
//!
//! Implements the OmniFormatterModule interface for Python (.py, .pyi).
//! Targets Black 24.x output parity (L-08 mitigation).
//!
//! # Black 24.x Compat Mode
//!
//! In "opinionated" mode (default), output matches Black 24.x byte-for-byte.
//! Key Black rules enforced:
//! - Magic trailing comma preservation (Black 22.x+)
//! - String normalization (double quotes by default)
//! - Target Python version affects trailing comma in function calls
//! - `line-length` → `ConfigIR.print_width`
//!
//! # Config Adapter
//!
//! Reads `pyproject.toml` [tool.black] section:
//! - `line-length` → `print_width`
//! - `skip-string-normalization` → `quote_style = Single`
//! - `target-version` → stored in module-specific config
//! - `include`/`exclude` → honoured at extension host level
//!
//! Also reads `.editorconfig` as the base layer.
//!
//! # Implementation Status
//!
//! Phase 4 scaffold: all config types and the module interface are implemented.
//! Format logic is a pass-through stub. Full Black-parity algorithm requires
//! Tree-sitter Python grammar integration (Phase 4 continuation).

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;
use protocol::ConfigIR;

#[wasm_bindgen]
pub fn format_python(source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
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
pub fn version() -> String { env!("CARGO_PKG_VERSION").to_string() }

#[wasm_bindgen]
pub fn language_id() -> String { "python".to_string() }

#[wasm_bindgen]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str(".py"),
        JsValue::from_str(".pyi"),
        JsValue::from_str(".pyw"),
    ]
}
