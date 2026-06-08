//! lang-csharp formatting logic.
//!
//! Formatter target: dotnet format / fantomas
//! Strategy: whitespace normalization + brace-depth indent pass.
//! Full CST-based formatting is planned for a future release.

use crate::adapter::Config;
use protocol::FormatError;

/// Normalise indentation using brace-depth tracking.
/// Returns source verbatim if it cannot be decoded as UTF-8.
pub fn format(source: &[u8], config: &Config) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s) => s,
        Err(_) => return Ok(source.to_vec()), // binary file: return verbatim
    };

    let mut out = String::with_capacity(source.len());
    let mut depth = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();

        // Count net brace change to decide if this line decreases depth first
        let opens = trimmed.chars().filter(|&c| c == '{').count();
        let closes = trimmed.chars().filter(|&c| c == '}').count();

        if closes > opens && depth >= (closes - opens) {
            depth -= closes - opens;
        }

        if trimmed.is_empty() {
            out.push('\n');
        } else {
            out.push_str(&" ".repeat(depth * config.indent_size));
            out.push_str(trimmed);
            out.push('\n');
        }

        if opens > closes {
            depth += opens - closes;
        }
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
        let result = format(b"", &Config::default()).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_idempotent() {
        let src = b"fn main() {\n    let x = 1;\n}\n";
        let first = format(src, &Config::default()).unwrap();
        let second = format(&first, &Config::default()).unwrap();
        assert_eq!(first, second, "lang-csharp must be idempotent");
    }
}
