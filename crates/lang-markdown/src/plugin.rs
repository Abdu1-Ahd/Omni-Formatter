use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// Markdown language plugin.
pub struct MarkdownPlugin;

impl LanguagePlugin for MarkdownPlugin {
    fn name(&self) -> &str {
        "lang-markdown"
    }

    fn extensions(&self) -> &[&str] {
        &["md", "markdown", "tex"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
