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

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
