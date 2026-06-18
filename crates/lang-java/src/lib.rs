//! Java / Kotlin / Scala / Groovy Language Module
//!
//! Targets google-java-format style for Java, ktfmt style for Kotlin.
//! Scala and Groovy use structural pass-through with indentation normalization.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JvmDialect {
    Java,
    Kotlin,
    Scala,
    Groovy,
}

impl JvmDialect {
    pub fn from_language_id(id: &str) -> Self {
        match id {
            "kotlin" => JvmDialect::Kotlin,
            "scala" => JvmDialect::Scala,
            "groovy" => JvmDialect::Groovy,
            _ => JvmDialect::Java,
        }
    }
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_java(
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
    "java".to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("kotlin"),
        JsValue::from_str("scala"),
        JsValue::from_str("groovy"),
        JsValue::from_str(".java"),
        JsValue::from_str(".kt"),
        JsValue::from_str(".kts"),
        JsValue::from_str(".scala"),
        JsValue::from_str(".sc"),
        JsValue::from_str(".groovy"),
    ]
}
