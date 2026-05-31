//! Rust config adapter — reads `rustfmt.toml` / `.rustfmt.toml` and translates to ConfigIR.
//!
//! # Priority chain (this module handles Layer 2)
//!
//! .omnifmt.json → **rustfmt.toml** → .rustfmt.toml → .editorconfig → defaults
//!
//! # TOML Parsing
//!
//! Uses the `toml` crate (v0.8) to parse `rustfmt.toml`.
//! Only the rustfmt stable options documented in
//! <https://rust-lang.github.io/rustfmt/?version=stable> are mapped.
//! Unknown/unstable options are silently ignored.

use protocol::config::{ConfigIR, EndOfLine, IndentStyle};
use serde::Deserialize;
use std::path::Path;

// ── rustfmt config schema ─────────────────────────────────────────────────

/// Stable rustfmt options that OmniFormatter maps to ConfigIR.
/// Unstable options (requires `unstable_features = true`) are ignored.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct RustfmtConfig {
    /// Maximum line width. Default: 100.
    pub max_width: Option<u16>,
    /// Number of spaces per indentation level. Default: 4.
    pub tab_spaces: Option<u8>,
    /// Use hard tabs instead of spaces. Default: false.
    pub hard_tabs: Option<bool>,
    /// Newline style: "Auto" | "Unix" | "Windows" | "Native". Default: "Auto".
    pub newline_style: Option<String>,
    /// Rust edition: "2015" | "2018" | "2021" | "2024". Default: "2015".
    pub edition: Option<String>,
    /// Where trailing commas are placed in function args/struct literals.
    /// "Vertical" | "Always" | "Never". Default: "Vertical".
    pub trailing_comma: Option<String>,
    /// Add trailing semicolons in match arms. Default: false.
    pub match_arm_leading_pipes: Option<String>,
}

impl RustfmtConfig {
    pub fn apply_to(&self, base: &mut ConfigIR) {
        // rustfmt defaults differ from Prettier
        base.print_width = self.max_width.unwrap_or(100);
        base.indent_size = self.tab_spaces.unwrap_or(4) as u8;
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
        // "Vertical" or "Always" → trailing commas; "Never" → no trailing commas
        if let Some(ref tc) = self.trailing_comma {
            base.trailing_comma = !tc.eq_ignore_ascii_case("never");
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────

/// Parse a `rustfmt.toml` or `.rustfmt.toml` file from disk.
///
/// Tries `rustfmt.toml` first, then `.rustfmt.toml`.
/// Returns `ConfigIR` with rustfmt defaults if neither file exists or is malformed.
pub fn config_from_rustfmt_toml_dir(dir: &Path) -> ConfigIR {
    let candidates = ["rustfmt.toml", ".rustfmt.toml"];
    for name in &candidates {
        let path = dir.join(name);
        if path.exists() {
            return config_from_rustfmt_toml_path(&path);
        }
    }
    rustfmt_defaults()
}

/// Parse a specific `rustfmt.toml` path.
pub fn config_from_rustfmt_toml_path(path: &Path) -> ConfigIR {
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return rustfmt_defaults(),
    };
    config_from_rustfmt_toml_str(&content)
}

/// Parse rustfmt config from a TOML string.
pub fn config_from_rustfmt_toml_str(toml_str: &str) -> ConfigIR {
    let mut config = rustfmt_defaults();
    match toml::from_str::<RustfmtConfig>(toml_str) {
        Ok(rc) => rc.apply_to(&mut config),
        Err(e) => {
            log::warn!("rustfmt.toml parse error: {} — using defaults", e);
        }
    }
    config
}

/// Build ConfigIR from a JSON representation of rustfmt options.
/// Used by the extension host TypeScript → Rust bridge.
pub fn config_from_rustfmt_json(json: &str) -> ConfigIR {
    let mut config = rustfmt_defaults();
    if let Ok(rc) = serde_json::from_str::<RustfmtConfig>(json) {
        rc.apply_to(&mut config);
    }
    config
}

/// Returns the rustfmt stable defaults as a ConfigIR.
fn rustfmt_defaults() -> ConfigIR {
    let mut config = ConfigIR::default();
    config.print_width = 100;
    config.indent_size = 4;
    config.trailing_comma = true; // Vertical → trailing comma in multi-line
    config
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustfmt_defaults_are_100_4() {
        let config = rustfmt_defaults();
        assert_eq!(config.print_width, 100);
        assert_eq!(config.indent_size, 4);
    }

    #[test]
    fn toml_parse_max_width() {
        let toml = "max_width = 120\n";
        let config = config_from_rustfmt_toml_str(toml);
        assert_eq!(config.print_width, 120);
    }

    #[test]
    fn toml_parse_hard_tabs() {
        let toml = "hard_tabs = true\n";
        let config = config_from_rustfmt_toml_str(toml);
        assert_eq!(config.indent_style, IndentStyle::Tabs);
    }

    #[test]
    fn toml_parse_trailing_comma_never() {
        let toml = "trailing_comma = \"Never\"\n";
        let config = config_from_rustfmt_toml_str(toml);
        assert!(!config.trailing_comma);
    }

    #[test]
    fn toml_parse_newline_windows() {
        let toml = "newline_style = \"Windows\"\n";
        let config = config_from_rustfmt_toml_str(toml);
        assert_eq!(config.end_of_line, EndOfLine::Crlf);
    }

    #[test]
    fn json_max_width_override() {
        let config = config_from_rustfmt_json(r#"{"max_width": 80}"#);
        assert_eq!(config.print_width, 80);
    }

    #[test]
    fn malformed_toml_returns_defaults() {
        let config = config_from_rustfmt_toml_str("this is not toml {{{{");
        assert_eq!(config.print_width, 100);
    }
}
