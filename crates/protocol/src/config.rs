//! Configuration intermediate representation (IR).
//!
//! All language module config adapters translate their native config format
//! (`.prettierrc`, `pyproject.toml`, `rustfmt.toml`, `.editorconfig`) into
//! this IR before passing it to the WASM core. Language modules receive this
//! IR, never the raw config files.
//!
//! The adapter search and priority order (highest to lowest):
//! 1. `.omnifmt.json` in workspace root (optional override)
//! 2. Language-native config file
//! 3. `.editorconfig` (base layer)
//! 4. Module defaults

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Print width mode for line-length limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PrintWidthMode {
    /// Column limit enforced (default). Units are display columns (L-14).
    #[default]
    Columns,
    /// No line-length limit. Format freely.
    Unlimited,
}

/// Indentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
    #[default]
    Spaces,
    Tabs,
}

/// Quote style for string literals (relevant for JS/TS/CSS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
    Single,
    #[default]
    Double,
}

/// End-of-line style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EndOfLine {
    #[default]
    Lf,
    Crlf,
    Cr,
    Auto,
}

/// Module operating mode (L-12 mitigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ModuleMode {
    /// Zero-config, reference formatter output parity guaranteed.
    #[default]
    Opinionated,
    /// Full option surface exposed. Compat guarantee voided.
    Advanced,
}

/// The universal configuration IR passed to every language module.
///
/// Language modules read this struct rather than any native config file.
/// Unknown options in native configs are silently ignored by the adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigIR {
    /// Maximum line length in display columns (L-14). Default: 80.
    pub print_width: u16,

    /// Print width enforcement mode. Default: Columns.
    pub print_width_mode: PrintWidthMode,

    /// Number of spaces per indentation level. Default: 2.
    pub indent_size: u8,

    /// Indentation style. Default: Spaces.
    pub indent_style: IndentStyle,

    /// Quote style for string literals. Default: Double.
    pub quote_style: QuoteStyle,

    /// Whether to add a trailing comma where valid. Default: true.
    pub trailing_comma: bool,

    /// Whether to add a semicolon at end of statements (JS/TS). Default: true.
    pub semicolons: bool,

    /// End-of-line style. Default: Lf.
    pub end_of_line: EndOfLine,

    /// Module operating mode (L-12). Default: Opinionated.
    pub mode: ModuleMode,

    /// Optional named preset (e.g. `"airbnb"`, `"google"`). Default: None.
    pub preset: Option<String>,

    /// Post-format chain: list of additional formatters to run after the
    /// primary formatter (e.g. `["eslint-fix", "import-sort"]`). Default: [].
    pub post_format: Vec<String>,

    /// Language-specific schema overrides.
    ///
    /// Populated by the TypeScript host from `.omnifmt.json` or native configs.
    /// Keys follow the `<lang>__<option>` convention
    /// (e.g. `swift__braceStyle`, `objc__nullabilityAnnotations`,
    ///        `kotlin__trailingComma`, `java__maxAnnotationFieldLength`).
    ///
    /// `#[serde(flatten)]` makes these keys invisible in the struct layout but
    /// fully round-trippable at the JSON level — they appear at the top level
    /// of the serialised object, not nested under an `"extras"` key.
    /// Unknown keys from the TS host are collected here without error.
    #[serde(flatten)]
    pub extras: HashMap<String, serde_json::Value>,
}

impl ConfigIR {
    /// Return a language-specific string override, or `None` if absent / not a
    /// JSON string.
    ///
    /// ```rust,ignore
    /// let style = config.get_extra_str("swift__braceStyle").unwrap_or("k&r");
    /// ```
    pub fn get_extra_str<'a>(&'a self, key: &str) -> Option<&'a str> {
        self.extras.get(key)?.as_str()
    }

    /// Return a language-specific boolean override, or `None` if absent / not
    /// a JSON boolean.
    ///
    /// ```rust,ignore
    /// let annots = config.get_extra_bool("objc__nullabilityAnnotations").unwrap_or(true);
    /// ```
    pub fn get_extra_bool(&self, key: &str) -> Option<bool> {
        self.extras.get(key)?.as_bool()
    }

    /// Return a language-specific unsigned integer override, or `None`.
    ///
    /// ```rust,ignore
    /// let w = config.get_extra_u64("java__maxAnnotationFieldLength").unwrap_or(40);
    /// ```
    pub fn get_extra_u64(&self, key: &str) -> Option<u64> {
        self.extras.get(key)?.as_u64()
    }

    /// Return a language-specific floating-point override, or `None`.
    pub fn get_extra_f64(&self, key: &str) -> Option<f64> {
        self.extras.get(key)?.as_f64()
    }

    /// Generic typed accessor — deserialises the stored JSON value into `T`.
    /// Prefer the specialised helpers above for the common primitive types.
    pub fn get_extra<'de, T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.extras
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Check whether a language-specific key is present, regardless of its
    /// JSON type.
    pub fn has_extra(&self, key: &str) -> bool {
        self.extras.contains_key(key)
    }
}

impl Default for ConfigIR {
    fn default() -> Self {
        ConfigIR {
            print_width: 80,
            print_width_mode: PrintWidthMode::default(),
            indent_size: 2,
            indent_style: IndentStyle::default(),
            quote_style: QuoteStyle::default(),
            trailing_comma: true,
            semicolons: true,
            end_of_line: EndOfLine::default(),
            mode: ModuleMode::default(),
            preset: None,
            post_format: Vec::new(),
            extras: HashMap::new(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Baseline fields still round-trip correctly after adding extras.
    #[test]
    fn roundtrip_baseline_fields() {
        let ir = ConfigIR {
            print_width: 120,
            indent_size: 4,
            trailing_comma: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&ir).unwrap();
        let back: ConfigIR = serde_json::from_str(&json).unwrap();
        assert_eq!(back.print_width, 120);
        assert_eq!(back.indent_size, 4);
        assert!(!back.trailing_comma);
        assert!(back.extras.is_empty());
    }

    /// Language-specific keys survive the JSON round-trip via `extras`.
    #[test]
    fn roundtrip_language_specific_extras() {
        let json = r#"{
            "printWidth": 100,
            "swift__braceStyle": "k&r",
            "objc__nullabilityAnnotations": true,
            "java__maxAnnotationFieldLength": 40,
            "kotlin__trailingComma": "always"
        }"#;
        let ir: ConfigIR = serde_json::from_str(json).unwrap();
        assert_eq!(ir.print_width, 100);
        assert_eq!(ir.get_extra_str("swift__braceStyle"), Some("k&r"));
        assert_eq!(
            ir.get_extra_bool("objc__nullabilityAnnotations"),
            Some(true)
        );
        assert_eq!(ir.get_extra_u64("java__maxAnnotationFieldLength"), Some(40));
        assert_eq!(ir.get_extra_str("kotlin__trailingComma"), Some("always"));
    }

    /// Unknown keys from the TS host do NOT cause a deserialisation error.
    #[test]
    fn unknown_keys_are_silently_collected() {
        let json = r#"{"printWidth": 80, "futureLanguage__unknownOption": "yes"}"#;
        let ir: ConfigIR = serde_json::from_str(json).unwrap();
        assert_eq!(ir.print_width, 80);
        assert!(ir.has_extra("futureLanguage__unknownOption"));
    }

    /// Serialised output contains language-specific keys at the top level
    /// (i.e. `flatten` is transparent to JSON consumers like the TS host).
    #[test]
    fn extras_serialised_at_top_level() {
        let mut ir = ConfigIR::default();
        ir.extras.insert(
            "swift__braceStyle".to_string(),
            serde_json::Value::String("k&r".to_string()),
        );
        let json = serde_json::to_string(&ir).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["swift__braceStyle"], "k&r");
    }

    /// Accessor helpers return None for absent keys — never panic.
    #[test]
    fn accessor_helpers_return_none_for_missing_keys() {
        let ir = ConfigIR::default();
        assert!(ir.get_extra_str("nonexistent").is_none());
        assert!(ir.get_extra_bool("nonexistent").is_none());
        assert!(ir.get_extra_u64("nonexistent").is_none());
        assert!(!ir.has_extra("nonexistent"));
    }
}
