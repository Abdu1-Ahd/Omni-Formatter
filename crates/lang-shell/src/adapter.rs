//! lang-shell config adapter.
use protocol::config::ConfigIR;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,
}

fn default_indent_size() -> usize {
    2
}
fn default_column_limit() -> usize {
    80
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indent_size: 2,
            column_limit: 80,
        }
    }
}

impl From<&ConfigIR> for Config {
    fn from(ir: &ConfigIR) -> Self {
        Self {
            indent_size: ir.indent_size as usize,
            column_limit: ir.print_width as usize,
        }
    }
}

pub fn config_from_json(json: &str) -> Config {
    serde_json::from_str(json).unwrap_or_default()
}
