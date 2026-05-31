use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// Go language plugin.
pub struct GoPlugin;

impl LanguagePlugin for GoPlugin {
    fn name(&self) -> &str {
        "lang-go"
    }

    fn extensions(&self) -> &[&str] {
        &["go"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
