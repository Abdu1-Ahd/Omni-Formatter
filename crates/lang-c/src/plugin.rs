use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// CPlugin plugin
pub struct CPlugin;

impl LanguagePlugin for CPlugin {
    fn name(&self) -> &str {
        "lang-c"
    }

    fn extensions(&self) -> &[&str] {
        &["c", "h", "cpp", "hpp", "cc", "cxx"]
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
