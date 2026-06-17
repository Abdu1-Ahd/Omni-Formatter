use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// DevopsPlugin plugin
pub struct DevopsPlugin;

impl LanguagePlugin for DevopsPlugin {
    fn name(&self) -> &str {
        "lang-devops"
    }

    fn extensions(&self) -> &[&str] {
        &["tf", "hcl", "Dockerfile", "Makefile", "nix"]
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
