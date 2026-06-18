//! Ruby / PHP / Perl / Lua Structural Formatter
//!
//! Upgrades the previous brace-only counter which was incorrect for Ruby
//! (Ruby uses `def/class/do/if/module ... end`, not `{}`).
//!
//! Strategy per dialect:
//! - Ruby: keyword-based depth tracker (def/class/do/if/unless/while/module/begin → end)
//! - PHP/Perl: brace-based (string-aware, like lang-c)
//! - Lua: keyword-based (function/if/do/for/while → end)
//!
//! # Schema keys consumed
//!
//! | Key                          | Type | Default    | Effect                      |
//! |------------------------------|------|------------|-----------------------------|
//! | `ruby__frozenStringLiteral`  | bool | `false`    | prepend frozen_string comment|
//! | `ruby__rubocopEnabled`       | bool | `true`     | apply RuboCop conventions   |
//! | `php__braceStyle`            | str  | `"psr2"`   | PHP brace placement         |

use protocol::config::{ConfigIR, IndentStyle};
use protocol::FormatError;

// ── Public entry point ─────────────────────────────────────────────────────
// ponytail: single entry point; caller passes language_id from lib.rs

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    format_for(source, config, "ruby")
}

pub fn format_for(source: &[u8], config: &ConfigIR, lang: &str) -> Result<Vec<u8>, FormatError> {
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

    let result = match lang {
        "php" | "perl" => format_brace_lang(text, config, indent_char, indent_size),
        "lua" => format_lua(text, indent_char, indent_size),
        _ => format_ruby(text, config, indent_char, indent_size),
    };

    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

// ── Ruby formatter ────────────────────────────────────────────────────────

fn format_ruby(source: &str, config: &ConfigIR, indent_char: char, indent_size: usize) -> String {
    let frozen_literal = config
        .get_extra_bool("ruby__frozenStringLiteral")
        .unwrap_or(false);

    // Keywords that open a new indentation level in Ruby
    let opens: &[&str] = &[
        "def ", "class ", "module ", "do", "do |", "begin", "if ", "unless ", "while ", "until ",
        "for ", "case ", "rescue", "ensure",
    ];
    // Single-word openers (no trailing space required)
    let opens_exact: &[&str] = &["do", "begin", "rescue", "ensure"];

    let mut out: Vec<String> = Vec::with_capacity(source.lines().count() + 2);
    let mut depth: i32 = 0;
    let mut consecutive_blank = 0u32;

    if frozen_literal {
        out.push("# frozen_string_literal: true".to_string());
    }

    for raw in source.lines() {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= 1 {
                out.push(String::new());
            }
            continue;
        }
        consecutive_blank = 0;

        // Decrease depth for `end` (and `rescue`/`ensure` which are at same level as body)
        let is_end =
            trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end\t");
        let is_rescue = trimmed == "rescue"
            || trimmed.starts_with("rescue ")
            || trimmed.starts_with("rescue\n");
        let is_ensure = trimmed == "ensure";
        let is_else = trimmed == "else" || trimmed == "elsif" || trimmed.starts_with("elsif ");
        let is_when = trimmed.starts_with("when ");

        if is_end || is_rescue || is_ensure || is_else || is_when {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        // Increase depth after writing the line
        let line_opens = opens.iter().any(|kw| {
            if opens_exact.contains(kw) {
                trimmed == *kw
                    || trimmed.starts_with(&format!("{} ", kw))
                    || trimmed.starts_with(&format!("{}|", kw))
            } else {
                trimmed.starts_with(kw)
            }
        });
        // Inline `if`/`unless` (postfix) don't open a block: `return x if cond`
        let is_postfix_if = (trimmed.contains(" if ") || trimmed.contains(" unless "))
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("unless ");
        // Single-line `def foo = expr` (Ruby 3.x) doesn't open a block
        let is_endless_method =
            trimmed.starts_with("def ") && trimmed.contains(" = ") && !trimmed.ends_with("do");

        if line_opens && !is_postfix_if && !is_endless_method {
            depth += 1;
        }
        // `else`/`elsif`/`when`/`rescue`/`ensure` re-open one level
        if is_else || is_when || is_rescue || is_ensure {
            depth += 1;
        }
        // `end` closes without re-opening
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Brace-language formatter (PHP / Perl) ─────────────────────────────────

fn format_brace_lang(
    source: &str,
    config: &ConfigIR,
    indent_char: char,
    indent_size: usize,
) -> String {
    let brace_style = config.get_extra_str("php__braceStyle").unwrap_or("psr2");
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

// ── Lua formatter ─────────────────────────────────────────────────────────

fn format_lua(source: &str, indent_char: char, indent_size: usize) -> String {
    let opens_kw: &[&str] = &["function ", "if ", "for ", "while ", "do", "repeat"];
    let closes_kw: &[&str] = &["end", "until "];

    let mut out: Vec<String> = Vec::with_capacity(source.lines().count());
    let mut depth: i32 = 0;
    let mut consecutive_blank = 0u32;

    for raw in source.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= 1 {
                out.push(String::new());
            }
            continue;
        }
        consecutive_blank = 0;

        let is_close = closes_kw
            .iter()
            .any(|kw| trimmed == kw.trim() || trimmed.starts_with(kw));
        let is_else = trimmed == "else" || trimmed == "elseif" || trimmed.starts_with("elseif ");

        if is_close || is_else {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        let line_opens = opens_kw
            .iter()
            .any(|kw| trimmed.starts_with(kw) || trimmed == *kw);
        // Single-line `function` definitions (`local f = function() return 1 end`)
        let is_singleline = trimmed.contains(" end") && trimmed.contains("function");
        if line_opens && !is_singleline {
            depth += 1;
        }
        if is_else {
            depth += 1;
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Shared helpers ────────────────────────────────────────────────────────

fn make_indent(c: char, size: usize, depth: usize) -> String {
    let unit: String = std::iter::repeat_n(c, size).collect();
    unit.repeat(depth)
}

fn count_brace_delta(line: &str) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    let mut str_char = '"';

    while let Some(c) = chars.next() {
        match c {
            '#' if !in_str => break,
            '/' if !in_str => {
                if chars.peek() == Some(&'/') {
                    break;
                }
            }
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

// ── Tests ─────────────────────────────────────────────────────────────────

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
        let src = b"def foo\n  bar\nend\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-ruby must be idempotent");
    }

    #[test]
    fn ruby_def_indents_body() {
        let src = b"def greet\nputs 'hello'\nend\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let puts_line = s
            .lines()
            .find(|l| l.trim().starts_with("puts"))
            .expect("puts line missing");
        let indent_size = cfg().indent_size as usize;
        assert_eq!(
            puts_line.len() - puts_line.trim_start().len(),
            indent_size,
            "puts must be 1 level inside def:\n{}",
            s
        );
    }

    #[test]
    fn ruby_class_indents_methods() {
        let src = b"class Foo\ndef bar\n42\nend\nend\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let def_line = s
            .lines()
            .find(|l| l.trim().starts_with("def "))
            .expect("def line missing");
        let body_line = s
            .lines()
            .find(|l| l.trim() == "42")
            .expect("body line missing");
        let indent_size = cfg().indent_size as usize;
        let def_indent = def_line.len() - def_line.trim_start().len();
        let body_indent = body_line.len() - body_line.trim_start().len();
        assert_eq!(
            def_indent, indent_size,
            "def must be 1 level inside class:\n{}",
            s
        );
        assert_eq!(
            body_indent,
            indent_size * 2,
            "body must be 2 levels inside class:\n{}",
            s
        );
    }

    #[test]
    fn ruby_frozen_string_literal_prepended() {
        let mut config = cfg();
        config.extras.insert(
            "ruby__frozenStringLiteral".to_string(),
            serde_json::Value::Bool(true),
        );
        let src = b"puts 'hi'\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.starts_with("# frozen_string_literal: true"),
            "magic comment missing:\n{}",
            s
        );
    }

    #[test]
    fn ruby_postfix_if_no_extra_indent() {
        // `return x if cond` must NOT add an extra indent level
        let src = b"def foo\nreturn 1 if true\n42\nend\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let indent_size = cfg().indent_size as usize;
        let return_line = s
            .lines()
            .find(|l| l.trim().starts_with("return"))
            .expect("return missing");
        let body_line = s.lines().find(|l| l.trim() == "42").expect("42 missing");
        let ret_indent = return_line.len() - return_line.trim_start().len();
        let body_indent = body_line.len() - body_line.trim_start().len();
        assert_eq!(
            ret_indent, indent_size,
            "return if must be at depth 1:\n{}",
            s
        );
        assert_eq!(
            body_indent, indent_size,
            "42 must be at same depth as return:\n{}",
            s
        );
    }
}
