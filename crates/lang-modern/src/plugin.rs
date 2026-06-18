use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// ModernPlugin plugin
pub struct ModernPlugin;

impl LanguagePlugin for ModernPlugin {
    fn name(&self) -> &str {
        "lang-modern"
    }

    fn extensions(&self) -> &[&str] {
        &["zig", "nim", "d", "astro", "svelte", "vue"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
