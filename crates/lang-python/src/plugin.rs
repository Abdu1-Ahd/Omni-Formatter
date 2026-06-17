use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// Python language plugin.
pub struct PythonPlugin;

impl LanguagePlugin for PythonPlugin {
    fn name(&self) -> &str {
        "lang-python"
    }

    fn extensions(&self) -> &[&str] {
        &["py", "pyi", "pyw", "ipynb"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
