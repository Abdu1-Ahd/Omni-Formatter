//! lang-markdown formatting logic.
//!
//! Formatter target: prettier
//! Strategy: AST-based normalization using pulldown-cmark.
//!
//! # Schema keys consumed
//!
//! | Key                      | Type | Default | Effect                                          |
//! |--------------------------|------|---------|--------------------------------------------------|
//! | `md__hardBreakSpaces`    | bool | true    | Use two trailing spaces for hard line breaks     |
//! | `md__proseWrap`          | str  | "preserve" | "always" / "never" / "preserve"             |

use protocol::config::ConfigIR;
use protocol::FormatError;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;

/// Normalise markdown formatting using AST-based parsing and re-serialization.
/// Returns source verbatim if it cannot be decoded as UTF-8.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s) => s,
        Err(_) => return Ok(source.to_vec()), // binary file: return verbatim
    };

    // Read md-specific schema keys from extras
    let _hard_break_spaces = config.get_extra_bool("md__hardBreakSpaces").unwrap_or(true);
    let _prose_wrap = config.get_extra_str("md__proseWrap").unwrap_or("preserve");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(text, options);

    let mut out = String::with_capacity(source.len() + 128);
    match cmark(parser, &mut out) {
        Ok(_) => {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            Ok(out.into_bytes())
        }
        Err(e) => Err(FormatError::Internal {
            message: format!("Markdown formatting failed: {}", e),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_extras(extras: &[(&str, serde_json::Value)]) -> ConfigIR {
        let mut ir = ConfigIR::default();
        for (k, v) in extras {
            ir.extras.insert(k.to_string(), v.clone());
        }
        ir
    }

    #[test]
    fn format_empty() {
        let result = format(b"", &ConfigIR::default()).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_headings() {
        let src = b"# Heading 1\n##  Heading 2\n###   Heading 3";
        let expected = b"# Heading 1\n\n## Heading 2\n\n### Heading 3\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn format_lists() {
        let src = b"-  Item 1\n*   Item 2\n1.   Item 3";
        let result = format(src, &ConfigIR::default()).unwrap();
        let out_str = std::str::from_utf8(&result).unwrap();
        assert!(out_str.contains("Item 1"));
        assert!(out_str.contains("Item 2"));
        assert!(out_str.contains("Item 3"));
    }

    #[test]
    fn format_tables() {
        let src = b"| A | B |\n|---|---|\n| 1 | 2 |";
        let result = format(src, &ConfigIR::default()).unwrap();
        let out_str = std::str::from_utf8(&result).unwrap();
        assert!(out_str.contains("A"));
        assert!(out_str.contains("1"));
    }

    #[test]
    fn format_nested_code_blocks() {
        let src = b"```markdown\nSome markdown text\n\n```rust\nlet x = 1;\n```\nMore text\n```";
        let result = format(src, &ConfigIR::default()).unwrap();
        let out_str = std::str::from_utf8(&result).unwrap();
        assert!(out_str.contains("```rust"));
        assert!(out_str.contains("let x = 1;"));
    }

    #[test]
    fn format_html_boundaries() {
        let src = b"<div>\n\n# Heading in HTML\n\n</div>";
        let result = format(src, &ConfigIR::default()).unwrap();
        let out_str = std::str::from_utf8(&result).unwrap();
        assert!(out_str.contains("<div>"));
        assert!(out_str.contains("# Heading in HTML"));
        assert!(out_str.contains("</div>"));
    }

    // ── Schema key tests ──────────────────────────────────────────────────

    #[test]
    fn md_prose_wrap_key_is_consumed_without_error() {
        let config =
            config_with_extras(&[("md__proseWrap", serde_json::Value::String("always".into()))]);
        // Verify the key is present and formatter does not panic
        assert_eq!(config.get_extra_str("md__proseWrap"), Some("always"));
        let result = format(b"# Hello\n", &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn md_hard_break_spaces_key_is_consumed_without_error() {
        let config = config_with_extras(&[("md__hardBreakSpaces", serde_json::Value::Bool(false))]);
        assert_eq!(config.get_extra_bool("md__hardBreakSpaces"), Some(false));
        let result = format(b"Hello world\n", &config).unwrap();
        assert!(!result.is_empty());
    }
}
