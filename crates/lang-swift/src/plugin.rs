use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// SwiftPlugin plugin
pub struct SwiftPlugin;

impl LanguagePlugin for SwiftPlugin {
    fn name(&self) -> &str {
        "lang-swift"
    }

    fn extensions(&self) -> &[&str] {
        &["swift", "m", "mm"]
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
