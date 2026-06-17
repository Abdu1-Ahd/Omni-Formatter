use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// SassPlugin plugin
pub struct SassPlugin;

impl LanguagePlugin for SassPlugin {
    fn name(&self) -> &str {
        "lang-sass"
    }

    fn extensions(&self) -> &[&str] {
        &["sass"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
