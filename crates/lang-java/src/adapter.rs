//! Java/Kotlin config adapter.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct JavaConfig {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,
    #[serde(default = "default_brace_style")]
    pub brace_style: String,
}

fn default_indent_size() -> usize {
    4
}
fn default_column_limit() -> usize {
    100
}
fn default_brace_style() -> String {
    "Attach".to_string()
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            indent_size: default_indent_size(),
            column_limit: default_column_limit(),
            brace_style: default_brace_style(),
        }
    }
}

pub fn config_from_json(json: &str) -> JavaConfig {
    serde_json::from_str(json).unwrap_or_default()
}
