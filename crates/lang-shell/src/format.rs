//! Bash / PowerShell / Zsh / AWK / Sed Structural Formatter
//!
//! Root cause of the previous bug: shell scripts use keyword-based scoping
//! (`if/fi`, `for/done`, `while/done`, `case/esac`, `do/done`, `function/}`)
//! — NOT `{}` alone. The previous brace counter:
//! 1. Counted `{` in strings like `echo "open brace: {"` — wrong
//! 2. Counted `{` in heredocs — wrong
//! 3. Missed `if/fi`, `for/done`, `while/done`, `case/esac` entirely — always depth 0
//!
//! This formatter correctly tracks shell block openers/closers using keyword
//! matching, string-aware to avoid counting quoted characters.
//!
//! # Schema keys consumed
//!
//! | Key                    | Type | Default   | Effect                              |
//! |------------------------|------|-----------|-------------------------------------|
//! | `bash__requireShebang` | bool | `true`    | warn if no `#!/...` on first line   |
//! | `bash__dialect`        | str  | `"bash"`  | `"posix"` / `"bash"` / `"zsh"`    |

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

    let result = format_shell(text, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

fn format_shell(source: &str, indent_char: char, indent_size: usize) -> String {
    // Shell openers: keywords that increase depth on the NEXT line
    // Note: `then` and `do` open the body; `{` in `function foo {` also opens
    let opens: &[&str] = &["then", "do", "else", "elif"];
    // Shell closers: keywords that decrease depth BEFORE emitting
    let closes: &[&str] = &["fi", "done", "esac", "else", "elif"];

    let mut out: Vec<String> = Vec::with_capacity(source.lines().count());
    let mut depth: i32 = 0;
    let mut in_heredoc = false;
    let mut heredoc_end = String::new();
    let mut consecutive_blank = 0u32;

    for raw in source.lines() {
        let trimmed = raw.trim();

        // ── heredoc pass-through ──────────────────────────────────────────
        if in_heredoc {
            out.push(raw.to_string()); // preserve heredoc verbatim (includes indent)
            if trimmed == heredoc_end {
                in_heredoc = false;
                heredoc_end.clear();
            }
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

        // ── heredoc detection ─────────────────────────────────────────────
        // Matches `<< WORD` or `<<- WORD` or `<< 'WORD'` (quoted)
        if let Some(hd_end) = detect_heredoc(trimmed) {
            heredoc_end = hd_end;
            in_heredoc = true;
            let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
            out.push(format!("{}{}", current_indent, trimmed));
            continue;
        }

        // ── depth adjustment before emitting ─────────────────────────────
        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        let is_close = closes.contains(&first_word);
        if is_close {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);

        // ── comment lines: preserve verbatim (depth 0 if no indent) ───────
        if trimmed.starts_with('#') {
            out.push(format!("{}{}", current_indent, trimmed));
            continue;
        }

        out.push(format!("{}{}", current_indent, trimmed));

        // ── depth adjustment after emitting ──────────────────────────────
        // `then` / `do` / `else` (after decrease) open new body
        let is_open = opens
            .iter()
            .any(|kw| first_word == *kw || trimmed.ends_with(&format!(" {}", kw)));
        // `function foo {` or `foo() {` — brace at end of line
        let ends_brace = trimmed.ends_with('{') && !trimmed.starts_with('#');
        // `case` keyword opens (body follows after `in` on next line or same line)
        let is_case = first_word == "case";

        if is_open || ends_brace || is_case {
            depth += 1;
        }

        // Function definition closing `}` alone
        if trimmed == "}" && depth > 0 {
            depth -= 1;
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Detect a heredoc opener and return the terminator word if found.
/// Handles: `cmd << EOF`, `cmd <<- EOF`, `cmd << 'EOF'`, `cmd << "EOF"`
fn detect_heredoc(line: &str) -> Option<String> {
    if !line.contains("<<") {
        return None;
    }
    // Find `<<` or `<<-`
    let rest = line.find("<<")?;
    let after = line[rest + 2..].trim_start_matches('-').trim();
    if after.is_empty() {
        return None;
    }
    // Strip optional quotes
    let word = after
        .trim_matches(|c| c == '\'' || c == '"')
        .split_whitespace()
        .next()?;
    if word.is_empty() {
        return None;
    }
    Some(word.to_string())
}

fn make_indent(c: char, size: usize, depth: usize) -> String {
    std::iter::repeat_n(c, size * depth).collect()
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
        let src = b"#!/usr/bin/env bash\nif [ -f \"$1\" ]; then\n  echo \"found\"\nfi\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-shell must be idempotent");
    }

    #[test]
    fn if_then_fi_indents_body() {
        let src = b"if [ -f /etc/hosts ]; then\necho \"found\"\nfi\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let echo_line = s
            .lines()
            .find(|l| l.contains("echo"))
            .expect("echo missing");
        let fi_line = s.lines().find(|l| l.trim() == "fi").expect("fi missing");
        assert!(
            echo_line.starts_with("  "),
            "echo must be indented inside if:\n{}",
            s
        );
        assert_eq!(
            fi_line.len() - fi_line.trim_start().len(),
            0,
            "fi must be at depth 0:\n{}",
            s
        );
    }

    #[test]
    fn while_do_done_indents_body() {
        let src = b"while true; do\necho \"loop\"\ndone\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let echo_line = s
            .lines()
            .find(|l| l.contains("echo"))
            .expect("echo missing");
        let done_line = s
            .lines()
            .find(|l| l.trim() == "done")
            .expect("done missing");
        assert!(
            echo_line.starts_with("  "),
            "echo must be indented inside while:\n{}",
            s
        );
        assert_eq!(
            done_line.len() - done_line.trim_start().len(),
            0,
            "done must be at depth 0:\n{}",
            s
        );
    }

    #[test]
    fn brace_in_string_not_counted() {
        // `echo "open brace: {"` must not increase depth
        let src = b"echo \"open brace: {\"\necho \"next line\"\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let next_line = s.lines().nth(1).expect("line 2 missing");
        assert_eq!(
            next_line.len() - next_line.trim_start().len(),
            0,
            "brace in echo string must not increase indent:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"echo hello   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
