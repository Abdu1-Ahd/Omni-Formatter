use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// SqlPlugin plugin
pub struct SqlPlugin;

impl LanguagePlugin for SqlPlugin {
    fn name(&self) -> &str {
        "lang-sql"
    }

    fn extensions(&self) -> &[&str] {
        &["sql", "graphql", "gql"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
