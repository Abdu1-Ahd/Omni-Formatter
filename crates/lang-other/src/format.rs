//! Solidity / GDScript / COBOL / Pascal / MATLAB / Assembly Structural Formatter
//!
//! Upgrades the previous naive brace counter.
//! Solidity is particularly affected: `require(msg.value >= 0, "send { ether }");`
//! would erroneously increase depth and permanently corrupt the file.
//!
//! Strategy: string-aware, comment-aware line-by-line structural formatter.
//! Uses `//` and `/*` for C-style comments (Solidity standard).
//!
//! # Schema keys consumed
//!
//! | Key           | Type | Default   | Effect                       |
//! |---------------|------|-----------|------------------------------|
//! | `braceStyle`  | str  | `"k&r"`   | `"allman"` splits `{` to own line |

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

    let brace_style = config.get_extra_str("braceStyle").unwrap_or("k&r");
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let result = format_brace_lang(text, brace_style, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

fn format_brace_lang(
    source: &str,
    brace_style: &str,
    indent_char: char,
    indent_size: usize,
) -> String {
    let allman = brace_style == "allman";
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

        let (opens, closes) = count_brace_delta(trimmed);
        if closes > 0 && opens == 0 {
            depth = (depth - closes as i32).max(0);
        }
        if closes > 0 && opens > 0 && trimmed.starts_with('}') {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);

        if allman && trimmed.ends_with('{') && trimmed != "{" && !trimmed.starts_with("//") {
            let body = trimmed[..trimmed.len() - 1].trim_end();
            out.push(format!("{}{}", current_indent, body));
            out.push(format!("{}{}", current_indent, "{"));
        } else {
            out.push(format!("{}{}", current_indent, trimmed));
        }

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

fn count_brace_delta(line: &str) -> (usize, usize) {
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
        let src = b"contract Token {\n    uint supply;\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-other must be idempotent");
    }

    #[test]
    fn brace_in_string_not_counted() {
        // Solidity: require(msg.value >= 0, "send { ether }") — brace in string literal
        let src = b"contract T {\n    function f() public {\n        require(true, \"send { ether }\");\n        uint x = 1;\n    }\n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let x_line = s
            .lines()
            .find(|l| l.contains("uint x"))
            .expect("uint x missing");
        let req_line = s
            .lines()
            .find(|l| l.contains("require"))
            .expect("require missing");
        let x_indent = x_line.len() - x_line.trim_start().len();
        let req_indent = req_line.len() - req_line.trim_start().len();
        assert_eq!(
            x_indent, req_indent,
            "string brace must not shift indentation:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"uint x = 1;   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
