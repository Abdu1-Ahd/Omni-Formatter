use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// ShellPlugin plugin
pub struct ShellPlugin;

impl LanguagePlugin for ShellPlugin {
    fn name(&self) -> &str {
        "lang-shell"
    }

    fn extensions(&self) -> &[&str] {
        &["sh", "bash", "zsh", "ps1", "psm1", "fish", "awk", "sed"]
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
