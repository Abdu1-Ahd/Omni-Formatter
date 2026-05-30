//! CSS/SCSS/Less/HTML Language Module
//!
//! Implements the OmniFormatterModule interface for:
//! - CSS (plain CSS, `*.css`)
//! - SCSS (Sass-with-braces, `*.scss`)
//! - Less (`*.less`)
//! - HTML (`*.html`, `*.htm`) — HTML formatting + embedded JS/CSS zones
//!
//! Targets Prettier 3.x output parity for all four dialects (L-08 mitigation).
//!
//! # Zone Integration
//!
//! HTML files are multi-language. The zone detector (crates/core/src/zones.rs)
//! splits HTML into:
//! - The HTML document itself (handled here)
//! - `<script>` zones (dispatched to lang-js)
//! - `<style>` zones (dispatched back to lang-css)
//! - `style="..."` inline attributes (handled here as an inline CSS zone)
//!
//! # Config Adapter
//!
//! Reads `.stylelintrc`, `.prettierrc`, and `.editorconfig`.
//! In opinionated mode, all Stylelint config is ignored — only Prettier
//! CSS settings apply.
//!
//! # Implementation Status
//!
//! Phase 4 scaffold. Format logic is pass-through stub.

pub mod adapter;
pub mod format;

use wasm_bindgen::prelude::*;
use protocol::ConfigIR;

/// The dialect of CSS-family source being formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssDialect {
    Css,
    Scss,
    Less,
    Html,
}

impl CssDialect {
    /// Detect the dialect from a VS Code languageId.
    pub fn from_language_id(id: &str) -> Self {
        match id {
            "scss" => CssDialect::Scss,
            "less" => CssDialect::Less,
            "html" => CssDialect::Html,
            _ => CssDialect::Css,
        }
    }
}

#[wasm_bindgen]
pub fn format_css(source_bytes: &[u8], config_json: &str, language_id: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
    let dialect = CssDialect::from_language_id(language_id);
    match format::format(source_bytes, &config, dialect) {
        Ok(f) => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn config_schema() -> String { include_str!("../schema.json").to_string() }

#[wasm_bindgen]
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }

#[wasm_bindgen]
pub fn language_id() -> &'static str { "css" }

#[wasm_bindgen]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str("scss"),
        JsValue::from_str("less"),
        JsValue::from_str("html"),
        JsValue::from_str(".css"),
        JsValue::from_str(".scss"),
        JsValue::from_str(".less"),
        JsValue::from_str(".html"),
        JsValue::from_str(".htm"),
    ]
}
