use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// RubyPlugin plugin
pub struct RubyPlugin;

impl LanguagePlugin for RubyPlugin {
    fn name(&self) -> &str {
        "lang-ruby"
    }

    fn extensions(&self) -> &[&str] {
        &["rb", "php", "pl", "pm", "lua"]
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
