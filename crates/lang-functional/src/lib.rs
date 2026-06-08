//! Haskell / Elixir / Erlang / OCaml / Clojure / R / Julia Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_functional(
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
pub fn language_id() -> String { "haskell".to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("elixir"),
        JsValue::from_str("erlang"),
        JsValue::from_str("ocaml"),
        JsValue::from_str("clojure"),
        JsValue::from_str("r"),
        JsValue::from_str("julia"),
        JsValue::from_str("lisp"),
        JsValue::from_str("scheme"),
        JsValue::from_str(".hs"),
        JsValue::from_str(".lhs"),
        JsValue::from_str(".ex"),
        JsValue::from_str(".exs"),
        JsValue::from_str(".erl"),
        JsValue::from_str(".hrl"),
        JsValue::from_str(".ml"),
        JsValue::from_str(".mli"),
        JsValue::from_str(".clj"),
        JsValue::from_str(".cljs"),
        JsValue::from_str(".r"),
        JsValue::from_str(".R"),
        JsValue::from_str(".jl"),
        JsValue::from_str(".lisp"),
        JsValue::from_str(".lsp"),
        JsValue::from_str(".scm"),
        JsValue::from_str(".ss"),
    ]
}
