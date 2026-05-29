//! JS/TS Config Adapter (L-10 mitigation)
//!
//! Reads native config files for JavaScript/TypeScript and translates them
//! into OmniFormatter's `ConfigIR`. The developer never needs to create a
//! new config format.
//!
//! # Adapter Search Order (highest to lowest priority)
//!
//! 1. `.omnifmt.json` in the workspace root
//! 2. `.prettierrc` (JSON), `.prettierrc.json`, `.prettierrc.yaml`, `.prettierrc.yml`
//! 3. `prettier.config.js` / `prettier.config.mjs` (read `printWidth`, `tabWidth`, etc.)
//! 4. `.prettierrc.js` / `.prettierrc.mjs`
//! 5. `.editorconfig` (base layer: indent style, indent size, end-of-line)
//! 6. Module defaults
//!
//! # Field Mapping
//!
//! | Prettier field | ConfigIR field | Notes |
//! |---|---|---|
//! | `printWidth` | `print_width` | Direct mapping |
//! | `tabWidth` | `indent_size` | Direct mapping |
//! | `useTabs` | `indent_style` | true → Tabs |
//! | `singleQuote` | `quote_style` | true → Single |
//! | `semi` | `semicolons` | Direct mapping |
//! | `trailingComma` | `trailing_comma` | "all"/"es5" → true, "none" → false |
//! | `endOfLine` | `end_of_line` | "lf"/"crlf"/"cr"/"auto" mapping |
//!
//! Unknown Prettier options are silently ignored.

use protocol::config::{ConfigIR, EndOfLine, IndentStyle, QuoteStyle};
use serde::Deserialize;

/// Raw Prettier config fields (subset that OmniFormatter supports).
/// Unknown fields are collected in `extra` and ignored.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrettierConfig {
    pub print_width: Option<u16>,
    pub tab_width: Option<u8>,
    pub use_tabs: Option<bool>,
    pub single_quote: Option<bool>,
    pub semi: Option<bool>,
    pub trailing_comma: Option<String>, // "all", "es5", "none"
    pub end_of_line: Option<String>,    // "lf", "crlf", "cr", "auto"
    pub jsx_single_quote: Option<bool>,
    pub bracket_spacing: Option<bool>,
    pub prose_wrap: Option<String>,
}

impl PrettierConfig {
    /// Apply this Prettier config on top of an existing `ConfigIR`.
    /// Prettier-specific options override the base config.
    pub fn apply_to(&self, base: &mut ConfigIR) {
        if let Some(w) = self.print_width {
            base.print_width = w;
        }
        if let Some(s) = self.tab_width {
            base.indent_size = s;
        }
        if let Some(true) = self.use_tabs {
            base.indent_style = IndentStyle::Tabs;
        } else if let Some(false) = self.use_tabs {
            base.indent_style = IndentStyle::Spaces;
        }
        if let Some(true) = self.single_quote {
            base.quote_style = QuoteStyle::Single;
        } else if let Some(false) = self.single_quote {
            base.quote_style = QuoteStyle::Double;
        }
        if let Some(semi) = self.semi {
            base.semicolons = semi;
        }
        if let Some(ref tc) = self.trailing_comma {
            base.trailing_comma = tc != "none";
        }
        if let Some(ref eol) = self.end_of_line {
            base.end_of_line = match eol.as_str() {
                "lf" => EndOfLine::Lf,
                "crlf" => EndOfLine::Crlf,
                "cr" => EndOfLine::Cr,
                _ => EndOfLine::Auto,
            };
        }
    }
}

/// Parse a `.prettierrc` JSON string into a `PrettierConfig`.
///
/// Unknown fields are silently ignored (L-10: non-destructive adapter).
pub fn parse_prettierrc_json(json: &str) -> Result<PrettierConfig, String> {
    serde_json::from_str(json).map_err(|e| format!("Failed to parse .prettierrc: {}", e))
}

/// Build a `ConfigIR` from a raw Prettier config JSON string.
///
/// Starts from `ConfigIR::default()` and overlays Prettier settings.
pub fn config_from_prettierrc_json(json: &str) -> ConfigIR {
    let mut config = ConfigIR::default();
    match parse_prettierrc_json(json) {
        Ok(prettier) => prettier.apply_to(&mut config),
        Err(_) => {} // Malformed .prettierrc — use defaults
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_prettierrc() {
        let json = r#"{
            "printWidth": 100,
            "tabWidth": 4,
            "useTabs": true,
            "singleQuote": true,
            "semi": false,
            "trailingComma": "none",
            "endOfLine": "crlf"
        }"#;

        let config = config_from_prettierrc_json(json);
        assert_eq!(config.print_width, 100);
        assert_eq!(config.indent_size, 4);
        assert_eq!(config.indent_style, IndentStyle::Tabs);
        assert_eq!(config.quote_style, QuoteStyle::Single);
        assert!(!config.semicolons);
        assert!(!config.trailing_comma);
        assert_eq!(config.end_of_line, EndOfLine::Crlf);
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{"printWidth": 80, "unknownOption": true, "anotherFake": "value"}"#;
        let config = config_from_prettierrc_json(json);
        assert_eq!(config.print_width, 80); // known field applied
    }

    #[test]
    fn malformed_json_returns_defaults() {
        let config = config_from_prettierrc_json("{ invalid json }");
        assert_eq!(config.print_width, 80); // default
        assert_eq!(config.indent_size, 2);  // default
    }

    #[test]
    fn trailing_comma_all_sets_true() {
        let json = r#"{"trailingComma": "all"}"#;
        let config = config_from_prettierrc_json(json);
        assert!(config.trailing_comma);
    }

    #[test]
    fn trailing_comma_none_sets_false() {
        let json = r#"{"trailingComma": "none"}"#;
        let config = config_from_prettierrc_json(json);
        assert!(!config.trailing_comma);
    }
}
