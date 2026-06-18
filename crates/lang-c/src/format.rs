//! C / C++ / Objective-C / Objective-C++ Structural Formatter
//!
//! Strategy: token-aware line-by-line structural normalization (same approach
//! as lang-java). Tree-sitter grammars removed due to workspace conflicts.
//!
//! # Schema keys consumed
//!
//! | Key                  | Type | Default   | Effect                          |
//! |----------------------|------|-----------|---------------------------------|
//! | `braceStyle`         | str  | `"llvm"`  | `"k&r"` keeps {; others = allman|
//! | `includeOrdering`    | str  | `"system-first"` | sort #include blocks  |
//! | `namespaceIndent`    | str  | `"None"`  | indent inside `namespace {}`   |

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

    let brace_style = config.get_extra_str("braceStyle").unwrap_or("llvm");
    let include_order = config
        .get_extra_str("includeOrdering")
        .unwrap_or("system-first");
    let namespace_ind = config.get_extra_str("namespaceIndent").unwrap_or("None");
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let fmt = CFormatter {
        config,
        brace_style,
        include_order,
        namespace_ind,
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

struct CFormatter<'a> {
    #[allow(dead_code)]
    config: &'a ConfigIR,
    brace_style: &'a str,
    include_order: &'a str,
    namespace_ind: &'a str,
    indent_char: char,
    indent_size: usize,
}

impl<'a> CFormatter<'a> {
    fn format(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let lines = if self.include_order != "preserve" {
            self.sort_includes(lines)
        } else {
            lines.into_iter().map(str::to_string).collect()
        };
        self.reindent(lines)
    }

    // ── #include sorting ──────────────────────────────────────────────────

    fn sort_includes(&self, lines: Vec<&str>) -> Vec<String> {
        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("#include") || line.starts_with("#import") {
                let block_start = i;
                while i < lines.len() && {
                    let t = lines[i].trim();
                    t.starts_with("#include") || t.starts_with("#import") || t.is_empty()
                } {
                    i += 1;
                }
                let block: Vec<&str> = lines[block_start..i]
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .copied()
                    .collect();
                for s in self.sort_include_block(block) {
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

    fn sort_include_block(&self, includes: Vec<&str>) -> Vec<String> {
        let mut system: Vec<&str> = Vec::new(); // <...>
        let mut local: Vec<&str> = Vec::new(); // "..."

        for inc in &includes {
            let t = inc.trim();
            if t.contains('<') {
                system.push(t);
            } else {
                local.push(t);
            }
        }
        system.sort_unstable();
        local.sort_unstable();

        let mut out: Vec<String> = Vec::new();
        match self.include_order {
            "local-first" => {
                for s in &local {
                    out.push(s.to_string());
                }
                if !local.is_empty() && !system.is_empty() {
                    out.push(String::new());
                }
                for s in &system {
                    out.push(s.to_string());
                }
            }
            _ => {
                // system-first (default)
                for s in &system {
                    out.push(s.to_string());
                }
                if !system.is_empty() && !local.is_empty() {
                    out.push(String::new());
                }
                for s in &local {
                    out.push(s.to_string());
                }
            }
        }
        out
    }

    // ── Re-indentation + brace style ──────────────────────────────────────

    fn reindent(&self, lines: Vec<String>) -> String {
        // ponytail: allman = any style that isn't "k&r" splits { to its own line
        let allman = self.brace_style != "k&r";
        // indent namespace bodies only if namespaceIndent != "None"
        let indent_namespace = self.namespace_ind != "None";

        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut depth: i32 = 0;
        let mut in_block_comment = false;
        let mut consecutive_blank = 0u32;
        let mut in_namespace = false;

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

            // Track namespace for optional indent suppression
            if trimmed.starts_with("namespace ") {
                in_namespace = true;
            }

            let (opens, closes) = self.count_brace_delta(trimmed);

            // ── namespaceIndent fix ──────────────────────────────────────
            // When inside a namespace AND the user opted out of indenting
            // namespace bodies (namespaceIndent == "None"), subtract one
            // level from the displayed depth so the content appears at the
            // same level as the surrounding code.
            // We suppress it on the `namespace` line itself and the opening
            // `{` line (depth still == 0 when those are written).
            let suppress = in_namespace && !indent_namespace && depth > 0;
            let eff = if suppress {
                (depth - 1).max(0)
            } else {
                depth.max(0)
            };
            let current_indent = self.make_indent(eff as usize);

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

            // Reset namespace tracking once the closing brace is emitted
            if in_namespace && closes > 0 && depth == 0 {
                in_namespace = false;
            }
        }

        out.iter()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn make_indent(&self, depth: usize) -> String {
        let unit: String = std::iter::repeat_n(self.indent_char, self.indent_size)
            .collect();
        unit.repeat(depth)
    }

    /// Count unquoted, un-commented `{` and `}` on a line.
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
    fn format_empty_returns_newline() {
        assert_eq!(format(b"", &cfg()).unwrap(), b"\n");
    }

    #[test]
    fn format_idempotent() {
        let src = b"int main() {\n    return 0;\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "idempotency violated");
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"int x = 1;   \nint y = 2;   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end(), "trailing whitespace: {:?}", line);
        }
    }

    #[test]
    fn brace_style_allman_splits_brace() {
        let mut config = cfg();
        config.extras.insert(
            "braceStyle".to_string(),
            serde_json::Value::String("allman".to_string()),
        );
        let fmt = CFormatter {
            config: &config,
            brace_style: "allman",
            include_order: "system-first",
            namespace_ind: "None",
            indent_char: ' ',
            indent_size: 4,
        };
        let result = fmt.format("int main() {\n    return 0;\n}");
        let brace_lines: Vec<_> = result.lines().filter(|l| l.trim() == "{").collect();
        assert!(
            !brace_lines.is_empty(),
            "allman must produce standalone {{ lines:\n{}",
            result
        );
    }

    #[test]
    fn include_ordering_system_first() {
        let mut config = cfg();
        config.extras.insert(
            "includeOrdering".to_string(),
            serde_json::Value::String("system-first".to_string()),
        );
        let src = b"#include \"myfile.h\"\n#include <stdio.h>\n\nint main() {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let stdio_pos = s.find("<stdio.h>").unwrap_or(usize::MAX);
        let local_pos = s.find("\"myfile.h\"").unwrap_or(usize::MAX);
        assert!(
            stdio_pos < local_pos,
            "system headers must come first:\n{}",
            s
        );
    }

    #[test]
    fn include_ordering_local_first() {
        let mut config = cfg();
        config.extras.insert(
            "includeOrdering".to_string(),
            serde_json::Value::String("local-first".to_string()),
        );
        let src = b"#include <stdio.h>\n#include \"myfile.h\"\n\nint main() {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let stdio_pos = s.find("<stdio.h>").unwrap_or(usize::MAX);
        let local_pos = s.find("\"myfile.h\"").unwrap_or(usize::MAX);
        assert!(
            local_pos < stdio_pos,
            "local headers must come first:\n{}",
            s
        );
    }

    #[test]
    fn brace_style_key_consumed() {
        let mut config = cfg();
        config.extras.insert(
            "braceStyle".to_string(),
            serde_json::Value::String("allman".to_string()),
        );
        assert_eq!(
            config.get_extra_str("braceStyle").unwrap_or("llvm"),
            "allman"
        );
    }

    #[test]
    fn namespace_indent_none_does_not_indent_body() {
        // namespaceIndent = "None" → code inside namespace { } must NOT be indented
        let mut config = cfg();
        config.extras.insert(
            "namespaceIndent".to_string(),
            serde_json::Value::String("None".to_string()),
        );
        let fmt = CFormatter {
            config: &config,
            brace_style: "k&r",
            include_order: "system-first",
            namespace_ind: "None",
            indent_char: ' ',
            indent_size: 4,
        };
        let src = "namespace Foo {\nvoid bar();\n}";
        let result = fmt.format(src);
        let bar_line = result
            .lines()
            .find(|l| l.contains("bar()"))
            .expect("bar() missing");
        // With namespaceIndent=None, bar() must be at depth 0 (no leading spaces)
        assert_eq!(
            bar_line.len() - bar_line.trim_start().len(),
            0,
            "namespace body must not be indented when namespaceIndent=None:\n{}",
            result
        );
    }

    #[test]
    fn namespace_indent_all_indents_body() {
        // namespaceIndent = "All" → code inside namespace { } MUST be indented
        let mut config = cfg();
        config.extras.insert(
            "namespaceIndent".to_string(),
            serde_json::Value::String("All".to_string()),
        );
        let fmt = CFormatter {
            config: &config,
            brace_style: "k&r",
            include_order: "system-first",
            namespace_ind: "All",
            indent_char: ' ',
            indent_size: 4,
        };
        let src = "namespace Foo {\nvoid bar();\n}";
        let result = fmt.format(src);
        let bar_line = result
            .lines()
            .find(|l| l.contains("bar()"))
            .expect("bar() missing");
        assert!(
            bar_line.starts_with("    "),
            "namespace body must be indented when namespaceIndent=All:\n{}",
            result
        );
    }
}
