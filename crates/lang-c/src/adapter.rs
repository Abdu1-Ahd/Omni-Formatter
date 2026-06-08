//! C/C++ config adapter.
//!
//! Deserialises the JSON config blob passed from the extension host,
//! mapping it to the internal `CConfig` struct.

use serde::Deserialize;

/// Internal config for the C/C++ formatter.
#[derive(Debug, Clone, Deserialize)]
pub struct CConfig {
    /// Indentation style: "spaces" or "tabs"
    #[serde(default = "default_indent_style")]
    pub indent_style: String,

    /// Number of spaces per indent level
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,

    /// Maximum column width (clang-format ColumnLimit)
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,

    /// Brace style: "Attach", "Linux", "Allman", "GNU"
    #[serde(default = "default_brace_style")]
    pub brace_style: String,

    /// Pointer alignment: "Left", "Right", "Middle"
    #[serde(default = "default_pointer_alignment")]
    pub pointer_alignment: String,
}

fn default_indent_style() -> String {
    "spaces".to_string()
}
fn default_indent_size() -> usize {
    4
}
fn default_column_limit() -> usize {
    80
}
fn default_brace_style() -> String {
    "Attach".to_string()
}
fn default_pointer_alignment() -> String {
    "Right".to_string()
}

impl Default for CConfig {
    fn default() -> Self {
        Self {
            indent_style: default_indent_style(),
            indent_size: default_indent_size(),
            column_limit: default_column_limit(),
            brace_style: default_brace_style(),
            pointer_alignment: default_pointer_alignment(),
        }
    }
}

/// Parse config from the JSON blob sent by the extension host.
pub fn config_from_json(json: &str) -> CConfig {
    serde_json::from_str(json).unwrap_or_default()
}
