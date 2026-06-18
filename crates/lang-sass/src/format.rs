//! Sass / SCSS / Less / Stylus Structural Formatter
//!
//! Upgrades the previous naive brace counter.
//! SCSS is severely affected: `content: "{"` or `attr(data-open, "{")`
//! would register as a scope opener and corrupt nesting depth permanently.
//!
//! Additionally, Sass (indented syntax) does NOT use braces at all —
//! it uses significant whitespace like Python. For `.sass` files we
//! preserve the whitespace structure verbatim to avoid corrupting it.
//!
//! Strategy:
//! - SCSS/Less/Stylus: string-aware brace counter (ignores `//` and `/*`)
//! - Sass (indented): verbatim pass-through (no brace reindentation)
//!
//! # Schema keys consumed
//!
//! | Key               | Type | Default   | Effect                             |
//! |-------------------|------|-----------|------------------------------------|
//! | `sass__syntax`    | str  | `"scss"`  | `"sass"` disables brace reindent  |

use protocol::config::{ConfigIR, IndentStyle};
use protocol::FormatError;

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s) => s,
        Err(_) => return Ok(source.to_vec()),
    };
    if text.trim().is_empty() {
        return Ok(b"\n".to_vec());
    }

    // Indented Sass syntax uses significant whitespace — cannot brace-reindent
    let syntax = config.get_extra_str("sass__syntax").unwrap_or("scss");
    if syntax == "sass" {
        // Pass-through: normalize trailing whitespace and ensure trailing newline
        let cleaned: String = text
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        let mut out = cleaned.into_bytes();
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }
        return Ok(out);
    }

    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let result = format_scss(text, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

fn format_scss(source: &str, indent_char: char, indent_size: usize) -> String {
    let mut out: Vec<String> = Vec::with_capacity(source.lines().count());
    let mut depth: i32 = 0;
    let mut in_block_comment = false;
    let mut consecutive_blank = 0u32;

    for raw in source.lines() {
        let trimmed = raw.trim();

        if in_block_comment {
            let pfx = make_indent(indent_char, indent_size, depth.max(0) as usize);
            let content = if trimmed.starts_with('*') {
                format!(" {}", trimmed)
            } else {
                trimmed.to_string()
            };
            out.push(format!("{}{}", pfx, content));
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            consecutive_blank = 0;
            continue;
        }

        if trimmed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= 1 {
                out.push(String::new());
            }
            continue;
        }
        consecutive_blank = 0;

        let (opens, closes) = count_brace_delta_css(trimmed);
        if closes > 0 && opens == 0 {
            depth = (depth - closes as i32).max(0);
        }
        if closes > 0 && opens > 0 && trimmed.starts_with('}') {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        if opens > 0 && closes == 0 {
            depth += opens as i32;
        } else if opens > 0 && closes > 0 && !trimmed.starts_with('}') {
            depth += (opens as i32 - closes as i32).max(0);
        } else if opens > 0 && closes > 0 && trimmed.starts_with('}') {
            depth += opens as i32;
        }

        if trimmed.contains("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_indent(c: char, size: usize, depth: usize) -> String {
    std::iter::repeat_n(c, size * depth).collect()
}

/// CSS/SCSS-aware brace counter. Handles `//` SCSS comments, `/* */` blocks,
/// and both `"..."` and `'...'` string literals (e.g. `content: "{"` in CSS).
fn count_brace_delta_css(line: &str) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    let mut str_char = '"';

    while let Some(c) = chars.next() {
        match c {
            '/' if !in_str => match chars.peek() {
                Some('/') | Some('*') => break,
                _ => {}
            },
            '"' | '\'' if !in_str => {
                in_str = true;
                str_char = c;
            }
            c2 if in_str && c2 == str_char => {
                in_str = false;
            }
            '\\' if in_str => {
                chars.next();
            }
            '{' if !in_str => opens += 1,
            '}' if !in_str => closes += 1,
            _ => {}
        }
    }
    (opens, closes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ConfigIR {
        ConfigIR::default()
    }

    #[test]
    fn format_empty() {
        assert_eq!(format(b"", &cfg()).unwrap(), b"\n");
    }

    #[test]
    fn format_idempotent() {
        let src = b".foo {\n  color: red;\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-sass must be idempotent");
    }

    #[test]
    fn brace_in_content_property_not_counted() {
        // CSS: content: "{" — brace in string must not increase depth
        let src = b".foo {\n  content: \"{\";\n  color: red;\n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let color_line = s
            .lines()
            .find(|l| l.contains("color"))
            .expect("color missing");
        let content_line = s
            .lines()
            .find(|l| l.contains("content"))
            .expect("content missing");
        assert_eq!(
            color_line.len() - color_line.trim_start().len(),
            content_line.len() - content_line.trim_start().len(),
            "brace in content property must not shift indentation:\n{}",
            s
        );
    }

    #[test]
    fn sass_syntax_preserves_whitespace() {
        // Indented Sass must be returned with whitespace preserved
        let mut config = cfg();
        config.extras.insert(
            "sass__syntax".to_string(),
            serde_json::Value::String("sass".to_string()),
        );
        let src = b".foo\n  color: red\n  .bar\n    color: blue\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains("  color: red"),
            "indented sass must preserve whitespace:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b".foo { color: red; }   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
