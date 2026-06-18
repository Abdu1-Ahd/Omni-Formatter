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
        format::format(source, config)
    }
}
