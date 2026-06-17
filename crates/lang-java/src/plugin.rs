use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// JavaPlugin plugin
pub struct JavaPlugin;

impl LanguagePlugin for JavaPlugin {
    fn name(&self) -> &str {
        "lang-java"
    }

    fn extensions(&self) -> &[&str] {
        &["java", "class", "jar", "kt", "kts", "scala", "sc", "groovy"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
