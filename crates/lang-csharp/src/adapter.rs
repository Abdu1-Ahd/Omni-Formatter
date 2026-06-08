//! lang-csharp config adapter.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,
}

fn default_indent_size()  -> usize { 2 }
fn default_column_limit() -> usize { 80 }

impl Default for Config {
    fn default() -> Self { Self { indent_size: default_indent_size(), column_limit: default_column_limit() } }
}

pub fn config_from_json(json: &str) -> Config {
    serde_json::from_str(json).unwrap_or_default()
}
