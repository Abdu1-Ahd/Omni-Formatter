//! C / C++ / Objective-C / Objective-C++ Language Module
//!
//! Implements the OmniFormatterModule interface for:
//! - C     (`.c`, `.h`)
//! - C++   (`.cpp`, `.hpp`, `.cc`, `.cxx`, `.hh`)
//! - Objective-C   (`.m`)
//! - Objective-C++ (`.mm`)
//!
//! Targets clang-format style (Google / LLVM / Chromium variants)
//! via a Wadler/Prettier-style CST formatter built on Tree-sitter.
//!
//! # Config
//!
//! Reads `.clang-format` (YAML) in the project root / ancestor dirs.
//! Falls back to Google style (indent=2, ColumnLimit=80) if absent.
//!
//! # Dialect Detection
//!
//! The `CDialect` enum is derived from the file extension passed via
//! `language_id`. C++ grammars handle Objective-C++ syntax too.
//!
//! # Implementation Status
//!
//! v0.2.0 scaffold: CST-based formatter with clang-format indent/brace rules.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// C-family dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CDialect {
    C,
    Cpp,
    ObjC,
    ObjCpp,
}

impl CDialect {
    pub fn from_language_id(id: &str) -> Self {
        match id {
            "cpp" | "cuda-cpp" => CDialect::Cpp,
            "objective-c" => CDialect::ObjC,
            "objective-cpp" => CDialect::ObjCpp,
            _ => CDialect::C,
        }
    }
}

/// Format C/C++/ObjC source using the detected dialect.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_c(
    source_bytes: &[u8],
    config_json: &str,
    language_id: &str,
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
    "c".to_string()
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("cpp"),
        JsValue::from_str("objective-c"),
        JsValue::from_str("objective-cpp"),
        JsValue::from_str("cuda-cpp"),
        JsValue::from_str(".c"),
        JsValue::from_str(".h"),
        JsValue::from_str(".cpp"),
        JsValue::from_str(".hpp"),
        JsValue::from_str(".cc"),
        JsValue::from_str(".cxx"),
        JsValue::from_str(".hh"),
        JsValue::from_str(".m"),
        JsValue::from_str(".mm"),
    ]
}
