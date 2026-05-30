//! Python config adapter — reads `pyproject.toml` [tool.black] section
//! and translates to ConfigIR (L-10 mitigation).

use protocol::config::{ConfigIR, QuoteStyle};
use serde::Deserialize;

/// The [tool.black] section of pyproject.toml.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct BlackConfig {
    /// Line length (maps to ConfigIR.print_width). Default: 88.
    pub line_length: Option<u16>,
    /// Skip string normalization (keep original quotes). Default: false.
    pub skip_string_normalization: Option<bool>,
    /// Skip magic trailing comma (Black 22.x+). Default: false.
    pub skip_magic_trailing_comma: Option<bool>,
    /// Target Python version(s). E.g. ["py39", "py310"].
    #[serde(default)]
    pub target_version: Vec<String>,
}

impl BlackConfig {
    pub fn apply_to(&self, base: &mut ConfigIR) {
        // Black default is 88, not Prettier's 80
        base.print_width = self.line_length.unwrap_or(88);
        if self.skip_string_normalization.unwrap_or(false) {
            base.quote_style = QuoteStyle::Single;
        }
        // skip_magic_trailing_comma → no trailing comma enforcement
        if self.skip_magic_trailing_comma.unwrap_or(false) {
            base.trailing_comma = false;
        }
    }
}

/// Parse the [tool.black] section from a pyproject.toml string.
///
/// We extract only the [tool.black] subsection as JSON for simplicity.
/// Unknown fields are silently ignored.
pub fn config_from_pyproject_toml(toml_str: &str) -> ConfigIR {
    // Phase 4 scaffold: TOML parsing requires the `toml` crate (added in Phase 4).
    // For now, return Black defaults.
    let mut config = ConfigIR::default();
    config.print_width = 88; // Black's default differs from Prettier's 80
    let _ = toml_str;
    config
}

/// Parse [tool.black] from a pre-extracted JSON representation.
pub fn config_from_black_json(json: &str) -> ConfigIR {
    let mut config = ConfigIR::default();
    config.print_width = 88; // Black default
    match serde_json::from_str::<BlackConfig>(json) {
        Ok(bc) => bc.apply_to(&mut config),
        Err(_) => {}
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_default_line_length_is_88() {
        let config = config_from_black_json("{}");
        assert_eq!(config.print_width, 88);
    }

    #[test]
    fn black_line_length_overrides_default() {
        let config = config_from_black_json(r#"{"line-length": 100}"#);
        assert_eq!(config.print_width, 100);
    }

    #[test]
    fn skip_string_normalization_sets_single_quote() {
        let config = config_from_black_json(r#"{"skip-string-normalization": true}"#);
        assert_eq!(config.quote_style, QuoteStyle::Single);
    }
}
