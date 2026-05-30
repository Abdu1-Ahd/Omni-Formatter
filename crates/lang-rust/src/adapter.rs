//! Rust config adapter — reads rustfmt.toml and translates to ConfigIR.

use protocol::config::{ConfigIR, EndOfLine, IndentStyle};
use serde::Deserialize;

/// The stable fields of rustfmt.toml that OmniFormatter supports.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct RustfmtConfig {
    /// Maximum line width. rustfmt default: 100.
    pub max_width: Option<u16>,
    /// Number of spaces per indentation. rustfmt default: 4.
    pub tab_spaces: Option<u8>,
    /// Use hard tabs. Default: false.
    pub hard_tabs: Option<bool>,
    /// Newline style. "Auto", "Unix", "Windows", "Native".
    pub newline_style: Option<String>,
    /// Rust edition. "2015", "2018", "2021".
    pub edition: Option<String>,
}

impl RustfmtConfig {
    pub fn apply_to(&self, base: &mut ConfigIR) {
        // rustfmt defaults to 100, not 80
        base.print_width = self.max_width.unwrap_or(100);
        base.indent_size = self.tab_spaces.unwrap_or(4);
        if self.hard_tabs.unwrap_or(false) {
            base.indent_style = IndentStyle::Tabs;
        }
        if let Some(ref style) = self.newline_style {
            base.end_of_line = match style.to_lowercase().as_str() {
                "unix" => EndOfLine::Lf,
                "windows" => EndOfLine::Crlf,
                _ => EndOfLine::Auto,
            };
        }
    }
}

/// Build ConfigIR from a rustfmt.toml JSON representation.
pub fn config_from_rustfmt_json(json: &str) -> ConfigIR {
    let mut config = ConfigIR::default();
    config.print_width = 100; // rustfmt default
    config.indent_size = 4;   // rustfmt default
    if let Ok(rc) = serde_json::from_str::<RustfmtConfig>(json) {
        rc.apply_to(&mut config);
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustfmt_defaults() {
        let config = config_from_rustfmt_json("{}");
        assert_eq!(config.print_width, 100);
        assert_eq!(config.indent_size, 4);
    }

    #[test]
    fn max_width_override() {
        let config = config_from_rustfmt_json(r#"{"max_width": 120}"#);
        assert_eq!(config.print_width, 120);
    }
}
