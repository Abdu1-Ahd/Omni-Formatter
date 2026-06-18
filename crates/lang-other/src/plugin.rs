use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// OtherPlugin plugin
pub struct OtherPlugin;

impl LanguagePlugin for OtherPlugin {
    fn name(&self) -> &str {
        "lang-other"
    }

    fn extensions(&self) -> &[&str] {
        &[
            "sol", "vy", "gd", "ahk", "au3", "cob", "cbl", "f90", "f95", "asm", "s",
        ]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
