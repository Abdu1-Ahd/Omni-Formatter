use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// Rust language plugin.
pub struct RustPlugin;

impl LanguagePlugin for RustPlugin {
    fn name(&self) -> &str {
        "lang-rust"
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
