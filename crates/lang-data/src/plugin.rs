use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// DataPlugin plugin
pub struct DataPlugin;

impl LanguagePlugin for DataPlugin {
    fn name(&self) -> &str {
        "lang-data"
    }

    fn extensions(&self) -> &[&str] {
        &["json", "json5", "yaml", "yml", "toml", "xml", "ini", "csv"]
    }

    /// Format the source using the full ConfigIR so that language-specific
    /// schema overrides in `config.extras` (e.g. `json__trailingComma`)
    /// are forwarded to the formatter — not discarded by the slim adapter.
    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
