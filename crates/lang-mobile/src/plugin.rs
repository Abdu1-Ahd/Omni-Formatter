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
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal {
                message: e.to_string(),
            }),
        }
    }
}
