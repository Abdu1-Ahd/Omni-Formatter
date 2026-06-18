use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// MobilePlugin plugin
pub struct MobilePlugin;

impl LanguagePlugin for MobilePlugin {
    fn name(&self) -> &str {
        "lang-mobile"
    }

    fn extensions(&self) -> &[&str] {
        &["dart"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
