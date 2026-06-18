//! HCL / Terraform / Dockerfile / Makefile / Nix Structural Formatter
//!
//! Upgrades the previous naive brace counter which was string-blind.
//! HCL (Terraform) is heavily affected: resource blocks contain `=` assignments
//! with string values like `description = "create { account }"` which the old
//! counter would interpret as a scope opener, corrupting the entire file.
//!
//! Strategy: string-aware, comment-aware line-by-line structural formatter
//! using `#` as the line-comment character (HCL standard).
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

    let result = format_hcl(text, brace_style, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

fn format_hcl(source: &str, brace_style: &str, indent_char: char, indent_size: usize) -> String {
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

        let (opens, closes) = count_brace_delta_hcl(trimmed);
        if closes > 0 && opens == 0 {
            depth = (depth - closes as i32).max(0);
        }
        if closes > 0 && opens > 0 && trimmed.starts_with('}') {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);

        if allman && trimmed.ends_with('{') && trimmed != "{" && !trimmed.starts_with('#') {
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

/// HCL-aware brace counter: treats `#` as a line comment (in addition to `//`).
fn count_brace_delta_hcl(line: &str) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    let mut str_char = '"';

    while let Some(c) = chars.next() {
        match c {
            '#' if !in_str => break, // HCL line comment
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
        let src = b"resource \"aws_s3_bucket\" \"b\" {\n  bucket = \"my-bucket\"\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-devops must be idempotent");
    }

    #[test]
    fn brace_in_string_not_counted() {
        // HCL: description = "create { account }" must not increase depth
        let src = b"resource \"r\" \"n\" {\n  description = \"create { account }\"\n  name = \"test\"\n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let name_line = s
            .lines()
            .find(|l| l.contains("name ="))
            .expect("name line missing");
        let desc_line = s
            .lines()
            .find(|l| l.contains("description"))
            .expect("desc line missing");
        let name_indent = name_line.len() - name_line.trim_start().len();
        let desc_indent = desc_line.len() - desc_line.trim_start().len();
        assert_eq!(
            name_indent, desc_indent,
            "string brace must not shift indentation:\n{}",
            s
        );
    }

    #[test]
    fn hash_comment_brace_not_counted() {
        // HCL uses # for comments
        let src = b"# open brace {\nresource \"r\" \"n\" {\n  x = 1\n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // The comment should be at depth 0
        let comment_line = s
            .lines()
            .find(|l| l.starts_with('#'))
            .expect("comment missing");
        assert_eq!(
            comment_line.len() - comment_line.trim_start().len(),
            0,
            "hash comment brace must not increase indent:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"x = 1   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
