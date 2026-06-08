//! Swift / Objective-C / Objective-C++ — swift-format Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_swift(
    source_bytes: &[u8],
    config_json: &str,
    _language_id: &str,
) -> Result<Vec<u8>, JsValue> {
    let config = adapter::config_from_json(config_json);
    match format::format(source_bytes, &config) {
        Ok(f)  => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String { include_str!("../schema.json").to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String { env!("CARGO_PKG_VERSION").to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String { "swift".to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("objective-c"),
        JsValue::from_str("objective-cpp"),
        JsValue::from_str(".swift"),
        JsValue::from_str(".m"),
        JsValue::from_str(".mm"),
    ]
}
