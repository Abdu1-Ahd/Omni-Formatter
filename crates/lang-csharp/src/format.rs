//! C# / F# Structural Formatter
//!
//! Strategy: token-aware line-by-line normalization.
//! Upgrades the previous naive brace-counter (which counted { in strings).
//!
//! # Schema keys consumed
//!
//! | Key             | Type | Default      | Effect                       |
//! |-----------------|------|--------------|------------------------------|
//! | `braceStyle`    | str  | `"allman"`   | `"k&r"` / `"allman"`        |
//! | `usingOrdering` | str  | `"system-first"` | sort `using` directives  |

use protocol::config::{ConfigIR, IndentStyle};
use protocol::FormatError;

// ── Public entry point ────────────────────────────────────────────────────

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s) => s,
        Err(_) => return Ok(source.to_vec()),
    };
    if text.trim().is_empty() {
        return Ok(b"\n".to_vec());
    }

    let brace_style = config.get_extra_str("braceStyle").unwrap_or("allman");
    let using_ordering = config
        .get_extra_str("usingOrdering")
        .unwrap_or("system-first");
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let fmt = CsharpFormatter {
        config,
        brace_style,
        using_ordering,
        indent_char,
        indent_size,
    };
    let result = fmt.format(text);

    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

// ── Formatter ─────────────────────────────────────────────────────────────

struct CsharpFormatter<'a> {
    #[allow(dead_code)]
    config: &'a ConfigIR,
    brace_style: &'a str,
    using_ordering: &'a str,
    indent_char: char,
    indent_size: usize,
}

impl<'a> CsharpFormatter<'a> {
    fn format(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let lines = if self.using_ordering != "preserve" {
            self.sort_usings(lines)
        } else {
            lines.into_iter().map(str::to_string).collect()
        };
        self.reindent(lines)
    }

    // ── `using` directive sorting ─────────────────────────────────────────

    fn sort_usings(&self, lines: Vec<&str>) -> Vec<String> {
        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("using ") && !line.contains('=') {
                let block_start = i;
                while i < lines.len() && {
                    let t = lines[i].trim();
                    (t.starts_with("using ") && !t.contains('=')) || t.is_empty()
                } {
                    i += 1;
                }
                let block: Vec<&str> = lines[block_start..i]
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .copied()
                    .collect();
                for s in self.sort_using_block(block) {
                    result.push(s);
                }
                if i < lines.len() && lines[i].trim().is_empty() {
                    result.push(String::new());
                    i += 1;
                }
            } else {
                result.push(lines[i].to_string());
                i += 1;
            }
        }
        result
    }

    fn sort_using_block(&self, usings: Vec<&str>) -> Vec<String> {
        let mut system: Vec<&str> = Vec::new();
        let mut other: Vec<&str> = Vec::new();
        let mut statics: Vec<&str> = Vec::new();

        for u in &usings {
            let t = u.trim();
            if t.starts_with("using static ") {
                statics.push(t);
            } else if t.starts_with("using System") {
                system.push(t);
            } else {
                other.push(t);
            }
        }
        system.sort_unstable();
        other.sort_unstable();
        statics.sort_unstable();

        let mut out: Vec<String> = Vec::new();
        match self.using_ordering {
            "alphabetical" => {
                let mut all: Vec<&str> = usings.iter().map(|s| s.trim()).collect();
                all.sort_unstable();
                for s in all {
                    out.push(s.to_string());
                }
            }
            _ => {
                // system-first
                for s in &system {
                    out.push(s.to_string());
                }
                if !system.is_empty() && !other.is_empty() {
                    out.push(String::new());
                }
                for s in &other {
                    out.push(s.to_string());
                }
                if (!system.is_empty() || !other.is_empty()) && !statics.is_empty() {
                    out.push(String::new());
                }
                for s in &statics {
                    out.push(s.to_string());
                }
            }
        }
        out
    }

    // ── Re-indentation ────────────────────────────────────────────────────

    fn reindent(&self, lines: Vec<String>) -> String {
        // C# defaults to Allman (braces on own lines)
        let allman = self.brace_style != "k&r";

        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut depth: i32 = 0;
        let mut in_block_comment = false;
        let mut consecutive_blank = 0u32;

        for raw in &lines {
            let trimmed = raw.trim();

            if in_block_comment {
                let pfx = self.make_indent(depth.max(0) as usize);
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

            let (opens, closes) = self.count_brace_delta(trimmed);

            if closes > 0 && opens == 0 {
                depth = (depth - closes as i32).max(0);
            }
            if closes > 0 && opens > 0 && trimmed.starts_with('}') {
                depth = (depth - 1).max(0);
            }

            let current_indent = self.make_indent(depth.max(0) as usize);

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

    fn make_indent(&self, depth: usize) -> String {
        let unit: String = std::iter::repeat_n(self.indent_char, self.indent_size).collect();
        unit.repeat(depth)
    }

    fn count_brace_delta(&self, line: &str) -> (usize, usize) {
        let mut opens = 0usize;
        let mut closes = 0usize;
        let mut chars = line.chars().peekable();
        let mut in_str = false;
        let mut str_char = '"';

        while let Some(c) = chars.next() {
            match c {
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
        let src = b"namespace App\n{\n    class Foo\n    {\n        int x = 1;\n    }\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-csharp must be idempotent");
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"int x = 1;   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end(), "trailing whitespace: {:?}", line);
        }
    }

    #[test]
    fn brace_style_allman_default() {
        // C# default is allman — { on its own line
        let config = cfg();
        let fmt = CsharpFormatter {
            config: &config,
            brace_style: "allman",
            using_ordering: "system-first",
            indent_char: ' ',
            indent_size: 4,
        };
        let result = fmt.format("class Foo {\n    int x;\n}");
        let brace_lines: Vec<_> = result.lines().filter(|l| l.trim() == "{").collect();
        assert!(
            !brace_lines.is_empty(),
            "allman must produce standalone {{ lines:\n{}",
            result
        );
    }

    #[test]
    fn using_ordering_system_first() {
        let mut config = cfg();
        config.extras.insert(
            "usingOrdering".to_string(),
            serde_json::Value::String("system-first".to_string()),
        );
        let src = b"using MyApp.Utils;\nusing System.Linq;\nusing System;\n\nclass A {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let system_pos = s.find("using System;").unwrap_or(usize::MAX);
        let myapp_pos = s.find("using MyApp").unwrap_or(usize::MAX);
        assert!(
            system_pos < myapp_pos,
            "System.* must come before third-party:\n{}",
            s
        );
    }

    #[test]
    fn using_ordering_alphabetical() {
        let mut config = cfg();
        config.extras.insert(
            "usingOrdering".to_string(),
            serde_json::Value::String("alphabetical".to_string()),
        );
        let src = b"using Z.Z;\nusing A.A;\nusing M.M;\n\nclass A {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let a_pos = s.find("A.A").unwrap_or(usize::MAX);
        let m_pos = s.find("M.M").unwrap_or(usize::MAX);
        let z_pos = s.find("Z.Z").unwrap_or(usize::MAX);
        assert!(
            a_pos < m_pos && m_pos < z_pos,
            "alphabetical ordering violated:\n{}",
            s
        );
    }

    #[test]
    fn brace_style_key_consumed() {
        let mut config = cfg();
        config.extras.insert(
            "braceStyle".to_string(),
            serde_json::Value::String("k&r".to_string()),
        );
        assert_eq!(
            config.get_extra_str("braceStyle").unwrap_or("allman"),
            "k&r"
        );
    }
}
