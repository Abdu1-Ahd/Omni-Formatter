//! Solidity / GDScript / AutoHotkey / COBOL / Fortran / Assembly Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_other(
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
    "solidity".to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("gdscript"),
        JsValue::from_str("ahk"),
        JsValue::from_str("cobol"),
        JsValue::from_str("fortran"),
        JsValue::from_str("asm"),
        JsValue::from_str(".sol"),
        JsValue::from_str(".vy"),
        JsValue::from_str(".gd"),
        JsValue::from_str(".ahk"),
        JsValue::from_str(".cob"),
        JsValue::from_str(".cbl"),
        JsValue::from_str(".f90"),
        JsValue::from_str(".f95"),
        JsValue::from_str(".asm"),
        JsValue::from_str(".s"),
        JsValue::from_str(".au3"),
    ]
}
