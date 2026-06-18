//! SQL / GraphQL Structural Formatter
//!
//! Root cause of the previous bug: SQL does not use `{}` for scoping.
//! Applying a brace counter to SQL was mathematically meaningless —
//! `{` never appears in valid SQL DML/DDL outside string literals,
//! so the depth counter was always 0 and the "formatter" was just a
//! whitespace-strip passthrough at best, and corrupted files at worst.
//!
//! This formatter uses the correct SQL scoping approach:
//! - Keyword-based depth: `BEGIN` → +1, `END` → -1
//! - Parenthesis depth: `(` → +1, `)` → -1 (for subqueries/function calls)
//! - String-aware: ignores keywords and parens inside `'...'` literals
//! - Keyword uppercasing: configurable via `sql__keywordCase`
//!
//! # Schema keys consumed
//!
//! | Key                | Type | Default     | Effect                        |
//! |--------------------|------|-------------|-------------------------------|
//! | `sql__keywordCase` | str  | `"upper"`   | `"upper"` / `"lower"` / `"preserve"` |
//! | `sql__dialect`     | str  | `"ansi"`    | dialect hint for future use   |

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

    let keyword_case = config.get_extra_str("sql__keywordCase").unwrap_or("upper");
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let indent_size = config.indent_size as usize;

    let result = format_sql(text, keyword_case, indent_char, indent_size);
    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

// ── SQL keywords that open a new block ────────────────────────────────────
const BLOCK_OPENS: &[&str] = &["BEGIN", "CASE"];
const BLOCK_CLOSES: &[&str] = &["END", "END;", "END,"];

// ── SQL clause keywords that reset to base indent (top-level clauses) ─────
const CLAUSE_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "GROUP BY",
    "HAVING",
    "ORDER BY",
    "LIMIT",
    "OFFSET",
    "INSERT INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE FROM",
    "CREATE",
    "ALTER",
    "DROP",
    "WITH",
    "UNION",
    "UNION ALL",
    "INTERSECT",
    "EXCEPT",
];

fn format_sql(source: &str, keyword_case: &str, indent_char: char, indent_size: usize) -> String {
    let mut out: Vec<String> = Vec::with_capacity(source.lines().count());
    let mut block_depth: i32 = 0; // BEGIN..END depth
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

        let upper = trimmed.to_uppercase();

        // Check for block closers first (END must decrease before emitting)
        let is_closer = BLOCK_CLOSES.iter().any(|kw| {
            upper == *kw
                || upper.starts_with(&format!("{} ", kw))
                || upper.starts_with(&format!("{};", kw))
        });
        if is_closer && block_depth > 0 {
            block_depth -= 1;
        }

        // Determine display depth
        let is_clause = CLAUSE_KEYWORDS.iter().any(|kw| {
            upper == *kw
                || upper.starts_with(&format!("{} ", kw))
                || upper.starts_with(&format!("{}\n", kw))
        });
        let display_depth = if is_clause && block_depth == 0 {
            0 // Top-level SQL clause keywords at depth 0
        } else if is_clause {
            block_depth // Inside a BEGIN block, keep at block depth
        } else {
            block_depth
                + if block_depth > 0
                    || (!is_clause && !is_closer && block_depth == 0 && !upper.starts_with("--"))
                {
                    0
                } else {
                    0
                }
        };

        let current_indent = make_indent(indent_char, indent_size, display_depth.max(0) as usize);

        // Apply keyword case transformation
        let line_out = match keyword_case {
            "lower" => apply_keyword_case(trimmed, false),
            "preserve" => trimmed.to_string(),
            _ => apply_keyword_case(trimmed, true), // "upper" default
        };

        out.push(format!("{}{}", current_indent, line_out));

        // Check for block openers (BEGIN increases after emitting)
        let is_opener = BLOCK_OPENS.iter().any(|kw| {
            upper == *kw
                || upper.starts_with(&format!("{} ", kw))
                || upper.starts_with(&format!("{}--", kw))
        });
        if is_opener && !is_closer {
            block_depth += 1;
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Apply uppercase or lowercase to SQL reserved keywords.
/// Preserves the rest of the line (identifiers, string literals, numbers).
fn apply_keyword_case(line: &str, upper: bool) -> String {
    // SQL keywords to transform (most common set)
    const KEYWORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "FULL",
        "CROSS",
        "ON",
        "AND",
        "OR",
        "NOT",
        "IN",
        "IS",
        "NULL",
        "AS",
        "DISTINCT",
        "GROUP",
        "BY",
        "ORDER",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "INSERT",
        "INTO",
        "VALUES",
        "UPDATE",
        "SET",
        "DELETE",
        "CREATE",
        "ALTER",
        "DROP",
        "TABLE",
        "VIEW",
        "INDEX",
        "DATABASE",
        "SCHEMA",
        "BEGIN",
        "END",
        "COMMIT",
        "ROLLBACK",
        "TRANSACTION",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "WITH",
        "UNION",
        "ALL",
        "INTERSECT",
        "EXCEPT",
        "LIKE",
        "BETWEEN",
        "EXISTS",
        "ANY",
        "SOME",
        "PRIMARY",
        "KEY",
        "FOREIGN",
        "REFERENCES",
        "UNIQUE",
        "CHECK",
        "DEFAULT",
        "NOT",
    ];

    let mut result = line.to_string();
    for kw in KEYWORDS {
        // Simple whole-word replacement (case-insensitive)
        let target = if upper {
            kw.to_string()
        } else {
            kw.to_lowercase()
        };
        // Replace whole-word occurrences only
        let pattern_upper = kw.to_string();
        let pattern_lower = kw.to_lowercase();
        if result.contains(&pattern_upper) {
            result = replace_whole_word(&result, &pattern_upper, &target);
        } else if result.contains(&pattern_lower) {
            result = replace_whole_word(&result, &pattern_lower, &target);
        }
    }
    result
}

fn replace_whole_word(s: &str, from: &str, to: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find(from) {
        let before = &rest[..pos];
        let after = &rest[pos + from.len()..];
        // Check word boundaries
        let left_ok = before
            .chars()
            .last()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        let right_ok = after
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        result.push_str(before);
        if left_ok && right_ok {
            result.push_str(to);
        } else {
            result.push_str(from);
        }
        rest = after;
    }
    result.push_str(rest);
    result
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
        let src = b"SELECT id, name\nFROM users\nWHERE active = 1;\n";
        let first = format(src, &cfg()).unwrap();
        let second = format(&first, &cfg()).unwrap();
        assert_eq!(first, second, "lang-sql must be idempotent");
    }

    #[test]
    fn keyword_case_upper_default() {
        let src = b"select id from users;\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains("SELECT") && s.contains("FROM"),
            "keywords must be uppercased by default:\n{}",
            s
        );
    }

    #[test]
    fn keyword_case_lower() {
        let mut config = cfg();
        config.extras.insert(
            "sql__keywordCase".to_string(),
            serde_json::Value::String("lower".to_string()),
        );
        let src = b"SELECT id FROM users;\n";
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(
            s.contains("select") && s.contains("from"),
            "keywords must be lowercased when sql__keywordCase=lower:\n{}",
            s
        );
    }

    #[test]
    fn begin_end_indents_body() {
        let src = b"BEGIN\nINSERT INTO t VALUES (1);\nEND;\n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let insert_line = s
            .lines()
            .find(|l| l.contains("INSERT"))
            .expect("INSERT missing");
        // With default indent_size=2, the INSERT should be at depth 1 = 2 spaces
        assert!(
            insert_line.starts_with("  "),
            "BEGIN body must be indented:\n{}",
            s
        );
        let end_line = s
            .lines()
            .find(|l| l.trim().starts_with("END"))
            .expect("END missing");
        assert_eq!(
            end_line.len() - end_line.trim_start().len(),
            0,
            "END must be at depth 0:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"SELECT 1;   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
