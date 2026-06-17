use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// CsharpPlugin plugin
pub struct CsharpPlugin;

impl LanguagePlugin for CsharpPlugin {
    fn name(&self) -> &str {
        "lang-csharp"
    }

    fn extensions(&self) -> &[&str] {
        &["cs", "fs", "fsi", "fsx"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
