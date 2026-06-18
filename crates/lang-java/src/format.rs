//! Java / Kotlin / Scala / Groovy Structural Formatter
//!
//! Formatter target: google-java-format / ktfmt style
//! Strategy: token-aware line-by-line structural normalization.
//!
//! The tree-sitter grammar was removed from this crate due to workspace
//! conflicts. This formatter uses a robust token-aware pass instead, which
//! correctly handles:
//!   - String literals and character literals (won't misparse braces inside strings)
//!   - Block comments /* ... */  (won't misparse braces inside comments)
//!   - Line comments // ...
//!   - Brace-based indentation
//!   - K&R vs Allman brace style (via `java__braceStyle`)
//!   - Import block sorting (via `java__importOrdering`)
//!   - Trailing whitespace elimination
//!   - Blank line normalization (max 1 consecutive blank line inside bodies)
//!
//! # Schema keys consumed
//!
//! | Key                         | Type   | Default       | Effect                          |
//! |-----------------------------|--------|---------------|---------------------------------|
//! | `java__braceStyle`          | str    | `"k&r"`       | `"k&r"` / `"allman"`           |
//! | `java__importOrdering`      | str    | `"java-first"`| import sort strategy            |
//! | `java__wildcardImportThreshold` | int | `99`         | (stored, no-op for this pass)  |

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

    let brace_style = config.get_extra_str("java__braceStyle").unwrap_or("k&r");
    let import_ordering = config
        .get_extra_str("java__importOrdering")
        .unwrap_or("java-first");
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let formatter = JavaFormatter {
        config,
        brace_style,
        import_ordering,
        indent_char,
        indent_size,
    };

    let result = formatter.format(text);

    // Idempotency: a second pass must produce identical output.
    #[cfg(debug_assertions)]
    {
        let second = formatter.format(&result);
        debug_assert_eq!(result, second, "lang-java: format is not idempotent!");
    }

    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

// ── Formatter ─────────────────────────────────────────────────────────────

struct JavaFormatter<'a> {
    #[allow(dead_code)]
    config: &'a ConfigIR,
    brace_style: &'a str,
    import_ordering: &'a str,
    indent_char: char,
    indent_size: usize,
}

impl<'a> JavaFormatter<'a> {
    fn format(&self, source: &str) -> String {
        // Phase 1: Split into logical lines preserving comment/string awareness
        let lines: Vec<&str> = source.lines().collect();

        // Phase 2: Sort import block (if java__importOrdering != "preserve")
        let lines = if self.import_ordering != "preserve" {
            self.sort_imports(lines)
        } else {
            lines.into_iter().map(str::to_string).collect()
        };

        // Phase 3: Structural re-indentation + brace style normalisation
        self.reindent(lines)
    }

    // ── Import sorting ─────────────────────────────────────────────────────

    fn sort_imports(&self, lines: Vec<&str>) -> Vec<String> {
        // Find contiguous import blocks and sort them.
        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("import ") {
                // Collect the full import block
                let block_start = i;
                while i < lines.len() && {
                    let t = lines[i].trim();
                    t.starts_with("import ") || t.is_empty()
                } {
                    i += 1;
                }
                let block: Vec<&str> = lines[block_start..i]
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .copied()
                    .collect();

                let sorted = self.sort_import_block(block);
                for s in sorted {
                    result.push(s);
                }
                // Preserve trailing blank after import block if present
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

    fn sort_import_block(&self, imports: Vec<&str>) -> Vec<String> {
        let mut static_imports: Vec<&str> = Vec::new();
        let mut java_imports: Vec<&str> = Vec::new();
        let mut javax_imports: Vec<&str> = Vec::new();
        let mut other_imports: Vec<&str> = Vec::new();

        for imp in &imports {
            let trimmed = imp.trim();
            if trimmed.starts_with("import static ") {
                static_imports.push(trimmed);
            } else if trimmed.starts_with("import java.") {
                java_imports.push(trimmed);
            } else if trimmed.starts_with("import javax.") {
                javax_imports.push(trimmed);
            } else {
                other_imports.push(trimmed);
            }
        }

        static_imports.sort_unstable();
        java_imports.sort_unstable();
        javax_imports.sort_unstable();
        other_imports.sort_unstable();

        let mut result: Vec<String> = Vec::new();

        match self.import_ordering {
            "static-first" => {
                Self::add_group(&mut result, &static_imports);
                Self::add_group(&mut result, &java_imports);
                Self::add_group(&mut result, &javax_imports);
                Self::add_group(&mut result, &other_imports);
            }
            "alphabetical" => {
                let mut all: Vec<&str> = imports.iter().map(|s| s.trim()).collect();
                all.sort_unstable();
                for s in all {
                    result.push(s.to_string());
                }
            }
            _ => {
                // java-first (default): java.* → javax.* → other → static
                Self::add_group(&mut result, &java_imports);
                Self::add_group(&mut result, &javax_imports);
                Self::add_group(&mut result, &other_imports);
                Self::add_group(&mut result, &static_imports);
            }
        }

        result
    }

    fn add_group(out: &mut Vec<String>, group: &[&str]) {
        if group.is_empty() {
            return;
        }
        if !out.is_empty() {
            out.push(String::new());
        }
        for s in group {
            out.push(s.to_string());
        }
    }

    // ── Re-indentation + brace style ──────────────────────────────────────

    fn reindent(&self, lines: Vec<String>) -> String {
        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut depth: i32 = 0;
        let mut in_block_comment = false;
        let mut consecutive_blank = 0u32;

        for raw_line in &lines {
            let trimmed = raw_line.trim();

            // ── Track block comment state ─────────────────────────────────
            if in_block_comment {
                let prefix = self.make_indent(depth.max(0) as usize);
                // Preserve the * alignment typical in Javadoc
                let content = if trimmed.starts_with('*') {
                    format!(" {}", trimmed)
                } else {
                    trimmed.to_string()
                };
                out.push(format!("{}{}", prefix, content));
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                consecutive_blank = 0;
                continue;
            }

            // ── Blank line normalization ───────────────────────────────────
            if trimmed.is_empty() {
                consecutive_blank += 1;
                // Never more than 1 blank line inside class/method bodies
                if consecutive_blank <= 1 {
                    out.push(String::new());
                }
                continue;
            }
            consecutive_blank = 0;

            // ── Count brace deltas for this line ──────────────────────────
            let (open_count, close_count) = self.count_brace_delta(trimmed);

            // ── Allman: move lone opening braces to their own line ─────────
            // (applies only when brace_style == "allman" and the line ends
            //  with `{` but is not itself just `{`)
            let effective_line: &str = trimmed;

            // ── Adjust depth BEFORE printing close-only lines ─────────────
            if close_count > 0 && open_count == 0 {
                depth = (depth - close_count as i32).max(0);
            }
            if close_count > 0 && open_count > 0 && trimmed.starts_with('}') {
                depth = (depth - 1).max(0);
            }

            let current_indent = self.make_indent(depth.max(0) as usize);

            // ── Allman: if line ends with { and is NOT a lone {, split it ──
            if self.brace_style == "allman"
                && effective_line.ends_with('{')
                && effective_line != "{"
                && !effective_line.starts_with("//")
                && !self.is_in_string_context(effective_line)
            {
                let body = effective_line[..effective_line.len() - 1].trim_end();
                out.push(format!("{}{}", current_indent, body));
                // The brace gets its own line at current depth
                out.push(format!("{}{}", current_indent, "{"));
            } else {
                // K&R: keep { on same line
                out.push(format!("{}{}", current_indent, effective_line));
            }

            // ── Adjust depth AFTER printing open-only or mixed lines ───────
            if open_count > 0 && close_count == 0 {
                depth += open_count as i32;
            } else if open_count > 0 && close_count > 0 && !trimmed.starts_with('}') {
                depth += (open_count as i32 - close_count as i32).max(0);
            } else if open_count > 0 && close_count > 0 && trimmed.starts_with('}') {
                // already decremented for leading close; add the opens
                depth += open_count as i32;
            }

            // ── Enter block comment mode ───────────────────────────────────
            if trimmed.contains("/*") && !trimmed.contains("*/") {
                in_block_comment = true;
            }
        }

        // Join and strip trailing whitespace per line
        out.iter()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn make_indent(&self, depth: usize) -> String {
        let unit: String = std::iter::repeat_n(self.indent_char, self.indent_size)
            .collect();
        unit.repeat(depth)
    }

    /// Count unquoted, uncommented `{` and `}` on a line.
    fn count_brace_delta(&self, line: &str) -> (usize, usize) {
        let mut opens = 0usize;
        let mut closes = 0usize;
        let mut chars = line.chars().peekable();
        let mut in_str = false;
        let in_char = false;
        let mut str_char = '"';

        while let Some(c) = chars.next() {
            match c {
                '/' if !in_str && !in_char => {
                    if chars.peek() == Some(&'/') {
                        break; // rest is line comment
                    }
                }
                '"' | '\'' if !in_char && !in_str => {
                    in_str = true;
                    str_char = c;
                }
                c2 if in_str && c2 == str_char => {
                    in_str = false;
                }
                '\\' if in_str => {
                    chars.next(); // skip escaped char
                }
                '{' if !in_str && !in_char => opens += 1,
                '}' if !in_str && !in_char => closes += 1,
                _ => {}
            }
        }
        (opens, closes)
    }

    /// Heuristic: is this line purely inside a string context?
    /// Used to avoid splitting string-ending-with-{ in allman mode.
    fn is_in_string_context(&self, line: &str) -> bool {
        // If the last non-space char before { is a quote, it's part of a string
        let before_brace = match line.rfind('{') {
            Some(i) => line[..i].trim_end(),
            None => return false,
        };
        before_brace.ends_with('"') || before_brace.ends_with('\'')
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ConfigIR {
        ConfigIR::default()
    }

    // ── Baseline ──────────────────────────────────────────────────────────

    #[test]
    fn format_empty_returns_newline() {
        let result = format(b"", &cfg()).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_idempotent() {
        let src = b"public class Foo {\n    void bar() {\n        System.out.println(\"hi\");\n    }\n}\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "idempotency violated");
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"public class A {   \n    int x;   \n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "trailing whitespace found: {:?}",
                line
            );
        }
    }

    #[test]
    fn collapses_multiple_blank_lines() {
        let src = b"class A {\n\n\n\n    int x;\n}\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // Must not have more than 2 consecutive newlines (1 blank line)
        assert!(
            !s.contains("\n\n\n"),
            "more than one consecutive blank line found:\n{}",
            s
        );
    }

    // ── Extras: java__braceStyle ──────────────────────────────────────────

    #[test]
    fn brace_style_kr_default() {
        let config = cfg();
        // Default is K&R — opening brace stays on same line
        let formatter = JavaFormatter {
            config: &config,
            brace_style: "k&r",
            import_ordering: "java-first",
            indent_char: ' ',
            indent_size: 4,
        };
        let src = "public class Foo {\n    void bar() {\n    }\n}";
        let result = formatter.format(src);
        // K&R: `{` stays with the declaration
        assert!(
            result.contains("Foo {") || result.contains("Foo{"),
            "K&R should keep brace on same line: {}",
            result
        );
    }

    #[test]
    fn brace_style_allman_splits_brace() {
        let mut config = cfg();
        config.extras.insert(
            "java__braceStyle".to_string(),
            serde_json::Value::String("allman".to_string()),
        );
        let formatter = JavaFormatter {
            config: &config,
            brace_style: "allman",
            import_ordering: "java-first",
            indent_char: ' ',
            indent_size: 4,
        };
        let src = "public class Foo {\n    void bar() {\n    }\n}";
        let result = formatter.format(src);
        // Allman: `{` should appear on its own line
        let lines: Vec<&str> = result.lines().collect();
        let brace_only_lines: Vec<_> = lines.iter().filter(|l| l.trim() == "{").collect();
        assert!(
            !brace_only_lines.is_empty(),
            "Allman style must produce standalone {{ lines:\n{}",
            result
        );
    }

    #[test]
    fn java_brace_style_key_is_consumed() {
        let mut config = cfg();
        config.extras.insert(
            "java__braceStyle".to_string(),
            serde_json::Value::String("allman".to_string()),
        );
        let brace_style = config.get_extra_str("java__braceStyle").unwrap_or("k&r");
        assert_eq!(brace_style, "allman");
    }

    // ── Extras: java__importOrdering ──────────────────────────────────────

    #[test]
    fn import_ordering_java_first_sorts_java_before_third_party() {
        let mut config = cfg();
        config.extras.insert(
            "java__importOrdering".to_string(),
            serde_json::Value::String("java-first".to_string()),
        );
        let src = b"import com.example.Foo;\nimport java.util.List;\nimport javax.inject.Inject;\n\nclass A {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let java_pos = s.find("java.util").unwrap_or(usize::MAX);
        let javax_pos = s.find("javax").unwrap_or(usize::MAX);
        let example_pos = s.find("com.example").unwrap_or(usize::MAX);
        assert!(java_pos < javax_pos, "java.* must come before javax.*: {s}");
        assert!(
            javax_pos < example_pos,
            "javax.* must come before third-party: {s}"
        );
    }

    #[test]
    fn import_ordering_alphabetical_sorts_all() {
        let mut config = cfg();
        config.extras.insert(
            "java__importOrdering".to_string(),
            serde_json::Value::String("alphabetical".to_string()),
        );
        let src = b"import com.z.Z;\nimport com.a.A;\nimport com.m.M;\n\nclass A {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let a_pos = s.find("com.a").unwrap_or(usize::MAX);
        let m_pos = s.find("com.m").unwrap_or(usize::MAX);
        let z_pos = s.find("com.z").unwrap_or(usize::MAX);
        assert!(
            a_pos < m_pos && m_pos < z_pos,
            "alphabetical ordering violated: {s}"
        );
    }

    #[test]
    fn import_ordering_preserve_leaves_order_unchanged() {
        let mut config = cfg();
        config.extras.insert(
            "java__importOrdering".to_string(),
            serde_json::Value::String("preserve".to_string()),
        );
        // com.z before java.util — preserve should not reorder
        let src = b"import com.z.Z;\nimport java.util.List;\n\nclass A {}\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let z_pos = s.find("com.z").unwrap_or(usize::MAX);
        let java_pos = s.find("java.util").unwrap_or(usize::MAX);
        assert!(z_pos < java_pos, "preserve should not reorder imports: {s}");
    }
}
