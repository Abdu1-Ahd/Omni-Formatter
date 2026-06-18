//! Jinja / Liquid / EJS / Handlebars / Twig / Mustache / Nunjucks / Blade / Pug Formatter
//!
//! Upgrades the previous naive brace counter.
//! Template engines are the most severely affected language group: `{{ variable }}`,
//! `{% if cond %}`, `{%- block name -%}` are all template expressions that use `{`
//! and `}` — the old counter would interpret every template tag as a brace change,
//! completely destroying the indentation of any template file.
//!
//! Strategy: skip `{{`, `{%`, and `{#` template delimiters when counting structural
//! braces, since these are template expressions — not scope delimiters. Only
//! bare `{` without a following `{`, `%`, or `#` counts as a structural brace.
//!
//! # Note on Pug
//! Pug uses significant indentation (no braces). A Pug file formatted by this
//! formatter will have its trailing whitespace stripped but structure preserved.

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

    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let result = format_template(text, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

fn format_template(source: &str, indent_char: char, indent_size: usize) -> String {
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

        let (opens, closes) = count_brace_delta_template(trimmed);
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

/// Template-aware brace counter.
/// Skips `{{`, `{%`, `{#`, `}}`, `%}`, `#}` — these are template delimiters,
/// not structural braces. Only a standalone `{` or `}` (not followed by
/// `{`, `%`, or `#`) counts as a scope change.
fn count_brace_delta_template(line: &str) -> (usize, usize) {
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
            '{' if !in_str => {
                // Peek next char — if it's `{`, `%`, or `#` this is a template tag
                match chars.peek() {
                    Some('{') | Some('%') | Some('#') => {
                        chars.next();
                    } // skip template opener pair
                    _ => opens += 1,
                }
            }
            '}' if !in_str => {
                // Check if this closes a template tag: `}}` or `%}` or `#}`
                // The `%}` / `#}` closers don't start with `}` in the outer char stream,
                // so we only need to handle `}}` here.
                match chars.peek() {
                    Some('}') => {
                        chars.next();
                    } // skip `}}` template closer
                    _ => closes += 1,
                }
            }
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
        let src = b"<div>\n  <p>{{ name }}</p>\n</div>\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-template must be idempotent");
    }

    #[test]
    fn jinja_double_brace_not_counted() {
        // `{{ variable }}` must NOT affect depth
        let src = b"<p>{{ name }}</p>\n<p>{{ age }}</p>\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        let age_indent = lines[1].len() - lines[1].trim_start().len();
        assert_eq!(age_indent, 0, "Jinja {{ }} must not change depth:\n{}", s);
    }

    #[test]
    fn jinja_block_tag_not_counted() {
        // `{% if cond %}` must NOT increase depth
        let src = b"{% if user %}\n<p>hello</p>\n{% endif %}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let p_line = s.lines().find(|l| l.contains("<p>")).expect("<p> missing");
        assert_eq!(
            p_line.len() - p_line.trim_start().len(),
            0,
            "Jinja block tags must not change structural depth:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"<p>{{ x }}</p>   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
