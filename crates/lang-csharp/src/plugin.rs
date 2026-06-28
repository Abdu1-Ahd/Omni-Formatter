use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// CsharpPlugin plugin
pub struct CsharpPlugin;

impl LanguagePlugin for CsharpPlugin {
    fn name(&self) -> &str {
        "lang-csharp"
    }

    fn extensions(&self) -> &[&str] {
        &["cs"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
