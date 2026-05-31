//! Python config adapter — reads `pyproject.toml` [tool.black] section
//! and translates to ConfigIR (L-10 mitigation).
//!
//! # Priority chain (this module handles Layer 2)
//!
//! .omnifmt.json → **pyproject.toml [tool.black]** → setup.cfg [tool:black] → .editorconfig → defaults
//!
//! # TOML Parsing
//!
//! Uses the `toml` crate (v0.8) to parse `pyproject.toml`.
//! Unknown fields are silently ignored (Black forwards-compat rule).
//! If the file is malformed TOML, returns `ConfigIR::default()` with `print_width = 88`.

use protocol::config::{ConfigIR, QuoteStyle};
use serde::Deserialize;
use std::path::Path;

// ── Black config schema ───────────────────────────────────────────────────

/// The `[tool.black]` section of `pyproject.toml`.
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
    /// Preview mode (Black 23.x+). Default: false.
    pub preview: Option<bool>,
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

// ── Root TOML structure ───────────────────────────────────────────────────

/// Top-level `pyproject.toml` structure (only the fields we care about).
#[derive(Debug, Default, Deserialize)]
struct PyprojectToml {
    tool: Option<PyprojectTool>,
}

#[derive(Debug, Default, Deserialize)]
struct PyprojectTool {
    black: Option<BlackConfig>,
}

// ── Public API ────────────────────────────────────────────────────────────

/// Parse the `[tool.black]` section from a `pyproject.toml` file path.
///
/// Returns the resolved `ConfigIR` with Black options applied.
/// Falls back to Black defaults if the file is missing, malformed, or has
/// no `[tool.black]` section.
pub fn config_from_pyproject_toml_path(path: &Path) -> ConfigIR {
    let config = ConfigIR {
        print_width: 88, // Black's default
        ..Default::default()
    };

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return config,
    };

    config_from_pyproject_toml(&content)
}

/// Parse the `[tool.black]` section from a `pyproject.toml` string.
///
/// Returns the resolved `ConfigIR` with Black options applied.
pub fn config_from_pyproject_toml(toml_str: &str) -> ConfigIR {
    let mut config = ConfigIR {
        print_width: 88, // Black's default
        ..Default::default()
    };

    let parsed: PyprojectToml = match toml::from_str(toml_str) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("pyproject.toml parse error: {} — using Black defaults", e);
            return config;
        }
    };

    if let Some(tool) = parsed.tool {
        if let Some(black) = tool.black {
            black.apply_to(&mut config);
        }
    }

    config
}

/// Parse `[tool.black]` from a pre-extracted JSON representation.
/// Used by the extension host's TypeScript → Rust bridge.
pub fn config_from_black_json(json: &str) -> ConfigIR {
    let mut config = ConfigIR {
        print_width: 88, // Black default
        ..Default::default()
    };
    if let Ok(bc) = serde_json::from_str::<BlackConfig>(json) {
        bc.apply_to(&mut config);
    }
    config
}

// ── setup.cfg parser ──────────────────────────────────────────────────────

/// Parse `[tool:black]` from a `setup.cfg` string (INI format).
///
/// `setup.cfg` uses `=` instead of TOML syntax. We do a minimal hand-parse.
pub fn config_from_setup_cfg(cfg_str: &str) -> ConfigIR {
    let mut config = ConfigIR {
        print_width: 88,
        ..Default::default()
    };

    let mut in_black_section = false;
    for line in cfg_str.lines() {
        let trimmed = line.trim();
        if trimmed == "[tool:black]" {
            in_black_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_black_section = false;
        }
        if !in_black_section {
            continue;
        }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "line-length" | "line_length" => {
                    if let Ok(n) = val.parse::<u16>() {
                        config.print_width = n;
                    }
                }
                "skip-string-normalization" | "skip_string_normalization" if val == "true" || val == "1" => {
                    config.quote_style = QuoteStyle::Single;
                }
                "skip-magic-trailing-comma" | "skip_magic_trailing_comma" if val == "true" || val == "1" => {
                    config.trailing_comma = false;
                }
                _ => {}
            }
        }
    }

    config
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_default_line_length_is_88() {
        let config = config_from_pyproject_toml("");
        assert_eq!(config.print_width, 88);
    }

    #[test]
    fn pyproject_toml_full_parse() {
        let toml = r#"
[tool.black]
line-length = 100
skip-string-normalization = true
target-version = ["py39", "py310"]
"#;
        let config = config_from_pyproject_toml(toml);
        assert_eq!(config.print_width, 100);
        assert_eq!(config.quote_style, QuoteStyle::Single);
    }

    #[test]
    fn pyproject_toml_magic_trailing_comma() {
        let toml = r#"
[tool.black]
skip-magic-trailing-comma = true
"#;
        let config = config_from_pyproject_toml(toml);
        assert!(!config.trailing_comma);
    }

    #[test]
    fn pyproject_toml_missing_black_section() {
        let toml = r#"
[tool.pytest.ini_options]
testpaths = ["tests"]
"#;
        let config = config_from_pyproject_toml(toml);
        assert_eq!(config.print_width, 88); // fallback to Black default
    }

    #[test]
    fn setup_cfg_parse() {
        let cfg = "[tool:black]\nline-length = 79\nskip-string-normalization = true\n";
        let config = config_from_setup_cfg(cfg);
        assert_eq!(config.print_width, 79);
        assert_eq!(config.quote_style, QuoteStyle::Single);
    }

    #[test]
    fn black_json_line_length_overrides_default() {
        let config = config_from_black_json(r#"{"line-length": 100}"#);
        assert_eq!(config.print_width, 100);
    }

    #[test]
    fn skip_string_normalization_sets_single_quote() {
        let config = config_from_black_json(r#"{"skip-string-normalization": true}"#);
        assert_eq!(config.quote_style, QuoteStyle::Single);
    }
}
