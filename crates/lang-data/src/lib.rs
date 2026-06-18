//! JSON / YAML / TOML / XML / INI — prettier / taplo Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
///
/// `config_json` is a JSON-serialised `ConfigIR` — including any
/// language-specific extras (e.g. `json__trailingComma`).
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_data(
    source_bytes: &[u8],
    config_json: &str,
    _language_id: &str,
) -> Result<Vec<u8>, JsValue> {
    let config: protocol::config::ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
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
    "json".to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("json5"),
        JsValue::from_str("yaml"),
        JsValue::from_str("toml"),
        JsValue::from_str("xml"),
        JsValue::from_str("ini"),
        JsValue::from_str(".json"),
        JsValue::from_str(".jsonc"),
        JsValue::from_str(".yaml"),
        JsValue::from_str(".yml"),
        JsValue::from_str(".toml"),
        JsValue::from_str(".xml"),
        JsValue::from_str(".ini"),
    ]
}
