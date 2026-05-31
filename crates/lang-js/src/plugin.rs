use protocol::{config::ConfigIR, FormatError, LanguagePlugin};
use crate::format;

/// JS/TS/JSX/TSX language plugin.
pub struct JsPlugin;

impl LanguagePlugin for JsPlugin {
    fn name(&self) -> &str { "lang-js" }

    fn extensions(&self) -> &[&str] {
        &["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
