//! Bash / PowerShell / Zsh — shfmt style Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_shell(
    source_bytes: &[u8],
    config_json: &str,
    _language_id: &str,
) -> Result<Vec<u8>, JsValue> {
    let config = adapter::config_from_json(config_json);
    match format::format(source_bytes, &config) {
        Ok(f) => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String {
    "shellscript".to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("powershell"),
        JsValue::from_str("zsh"),
        JsValue::from_str("fish"),
        JsValue::from_str(".sh"),
        JsValue::from_str(".bash"),
        JsValue::from_str(".zsh"),
        JsValue::from_str(".ps1"),
        JsValue::from_str(".psm1"),
        JsValue::from_str(".fish"),
    ]
}
