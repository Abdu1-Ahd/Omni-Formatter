use crate::{format, CssDialect};
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// CSS/SCSS/Less/HTML language plugin.
pub struct CssPlugin;

impl LanguagePlugin for CssPlugin {
    fn name(&self) -> &str {
        "lang-css"
    }

    fn extensions(&self) -> &[&str] {
        &["css", "scss", "less", "html", "htm"]
    }

    fn dialect_for_ext(&self, ext: &str) -> Option<&str> {
        Some(match ext {
            "scss" => "scss",
            "less" => "less",
            "html" | "htm" => "html",
            _ => "css",
        })
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config, CssDialect::Css)
    }

    fn format_dialect(
        &self,
        source: &[u8],
        config: &ConfigIR,
        dialect: &str,
    ) -> Result<Vec<u8>, FormatError> {
        let d = match dialect {
            "scss" => CssDialect::Scss,
            "less" => CssDialect::Less,
            "html" | "htm" => CssDialect::Html,
            _ => CssDialect::Css,
        };
        format::format(source, config, d)
    }
}
