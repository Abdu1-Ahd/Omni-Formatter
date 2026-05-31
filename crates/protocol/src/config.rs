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
        }
    }
}
