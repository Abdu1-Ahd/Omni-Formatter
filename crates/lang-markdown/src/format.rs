//! lang-markdown formatting logic.
//!
//! Strategy: conservative line-level pass-through.
//!
//! The previous AST round-trip (pulldown-cmark → cmark) discarded ALL
//! indentation that is not structurally significant to CommonMark — including
//! intentionally indented ASCII diagrams, tables drawn in plain text, and
//! aligned prose blocks.  That is the wrong trade-off for a code editor where
//! the author's layout intent must be preserved.
//!
//! This formatter only touches what is unambiguously garbage:
//!   • trailing whitespace on every line (invisible, serves no purpose)
//!   • runs of more than 2 consecutive blank lines (cosmetic cleanup)
//!   • missing final newline
//!
//! Everything else — indentation, spacing, heading style, list markers — is
//! left exactly as the author wrote it.
//!
//! # Schema keys consumed
//!
//! | Key                   | Type | Default    | Effect                          |
//! |-----------------------|------|------------|---------------------------------|
//! | `md__maxBlankLines`   | u64  | 2          | Max consecutive blank lines kept|

use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format markdown source with a conservative line-level pass-through.
/// Returns source verbatim if it cannot be decoded as UTF-8.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s) => s,
        Err(_) => return Ok(source.to_vec()), // binary: pass through verbatim
    };

    // ponytail: max_blank_lines is the only tunable; everything else is fixed.
    let max_blank: usize = config
        .get_extra_u64("md__maxBlankLines")
        .unwrap_or(2)
        .min(10) as usize;

    let mut out = String::with_capacity(source.len());
    let mut consecutive_blank: usize = 0;
    let mut in_code_fence = false;

    for raw in text.lines() {
        // Detect fenced code blocks so we never touch their content.
        // A fence starts with ``` or ~~~ (optionally indented).
        let trimmed = raw.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            // Code fence delimiters: strip trailing whitespace only.
            out.push_str(raw.trim_end());
            out.push('\n');
            consecutive_blank = 0;
            continue;
        }

        if in_code_fence {
            // Inside a fence: preserve the line exactly (indentation matters).
            out.push_str(raw);
            out.push('\n');
            consecutive_blank = 0;
            continue;
        }

        // Outside a fence: strip trailing whitespace, limit blank lines.
        let stripped = raw.trim_end();
        if stripped.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= max_blank {
                out.push('\n');
            }
        } else {
            consecutive_blank = 0;
            out.push_str(stripped);
            out.push('\n');
        }
    }

    // Ensure the file ends with exactly one newline.
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_empty() {
        assert_eq!(format(b"", &ConfigIR::default()).unwrap(), b"\n");
    }

    #[test]
    fn preserves_indented_diagram() {
        // ASCII diagrams inside Markdown paragraphs must not be re-indented.
        let src = b"Here is a diagram:\n\n    +---+\n    | A |\n    +---+\n\nEnd.\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains("    +---+"),
            "diagram indentation must be preserved:\n{}",
            s
        );
        assert!(
            s.contains("    | A |"),
            "diagram indentation must be preserved:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace_outside_fence() {
        let src = b"# Hello   \nSome text   \n";
        let result = format(src, &ConfigIR::default()).unwrap();
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
    fn preserves_content_inside_code_fence() {
        // Trailing spaces inside a fence must not be stripped (they may matter).
        let src = b"```\n  indented code   \n    more indent\n```\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains("  indented code   "),
            "fence content must be preserved verbatim:\n{}",
            s
        );
    }

    #[test]
    fn collapses_excess_blank_lines() {
        let src = b"A\n\n\n\n\nB\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // Default max_blank = 2, so at most 2 blank lines between A and B.
        let blanks = s
            .split("A\n")
            .nth(1)
            .unwrap_or("")
            .split("B")
            .next()
            .unwrap_or("")
            .chars()
            .filter(|&c| c == '\n')
            .count();
        assert!(
            blanks <= 3,
            "expected ≤2 blank lines between A and B, got {}:\n{}",
            blanks - 1,
            s
        );
    }

    #[test]
    fn ends_with_single_newline() {
        let src = b"# Title\n\nParagraph.\n\n\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        assert!(result.ends_with(b"\n"), "must end with newline");
        assert!(
            !result.ends_with(b"\n\n"),
            "must not end with double newline"
        );
    }

    #[test]
    fn headings_preserved_as_written() {
        // The formatter must NOT reformat ATX heading spacing.
        let src = b"# H1\n## H2\n### H3\n";
        let result = format(src, &ConfigIR::default()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(s.contains("# H1"), "H1 must be preserved");
        assert!(s.contains("## H2"), "H2 must be preserved");
        assert!(s.contains("### H3"), "H3 must be preserved");
    }

    #[test]
    fn md_max_blank_lines_config_respected() {
        let mut config = ConfigIR::default();
        config.extras.insert(
            "md__maxBlankLines".to_string(),
            serde_json::Value::Number(1.into()),
        );
        let src = b"A\n\n\n\nB\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // max_blank=1 → at most 1 blank line kept
        assert!(
            !s.contains("A\n\n\n"),
            "max 1 blank line must be enforced:\n{}",
            s
        );
    }

    #[test]
    fn non_utf8_bytes_returned_verbatim() {
        let src: &[u8] = &[0xFF, 0xFE, 0x00];
        let result = format(src, &ConfigIR::default()).unwrap();
        assert_eq!(result, src);
    }

    #[test]
    fn preserves_leading_spaces_inside_text_info_fence() {
        // Regression: ASCII diagram connector lines inside a ```text fence
        // must survive formatting with their leading spaces intact.
        // The README architecture diagram uses this exact pattern.
        // NOTE: raw string r#"..."# is required so \n\ doesn't eat leading spaces.
        let src = "```text\n\
┌─────────────────┬─────────────────┐\n";
        // Build the string with explicit leading spaces (not stripped by \n\).
        let connector = "                  │";
        let label     = "          [ Zero-Copy IPC ]";
        let full = format!("{src}{connector}\n{label}\n```\n");

        let result = super::format(full.as_bytes(), &ConfigIR::default()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains(connector),
            "leading-space connector must be preserved inside ```text fence:\n{}",
            s
        );
        assert!(
            s.contains(label),
            "leading-space label must be preserved inside ```text fence:\n{}",
            s
        );
    }
}
