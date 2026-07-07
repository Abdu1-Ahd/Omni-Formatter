//! Haskell / Elixir / Erlang / OCaml / Clojure / Lisp / R / Julia / Scala / Groovy Formatter
//!
//! Root cause of the previous bug: a single naive brace counter was applied to ALL
//! of these languages — but they have radically different scoping models:
//!
//! | Language family    | Scoping model                              | Previous bug                     |
//! |--------------------|---------------------------------------------|----------------------------------|
//! | Haskell, OCaml, Elm| Significant whitespace (layout rule)        | Brace counter stripped layout    |
//! | Elixir, Erlang     | `do/end` keyword blocks                     | Never tracked — always depth 0   |
//! | Clojure, Lisp, Racket | Parenthesis depth                        | Never tracked — always depth 0   |
//! | Scala, Groovy      | `{...}` braces (but string-aware needed)    | Counted `{` in strings           |
//! | R, Julia           | `{...}` braces (significant whitespace too)| Counted `{` in strings           |
//!
//! Strategy: dispatch on language_id (passed from the adapter) and apply the
//! correct scoping model per language family. When language_id is unavailable
//! we fall back to a safe whitespace-normalizing pass-through.

use protocol::config::{ConfigIR, IndentStyle};
use protocol::FormatError;

// ── Public entry point ─────────────────────────────────────────────────────

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    format_for(source, config, "haskell")
}

/// Called from `lib.rs` with the resolved language_id.
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
        // Elixir / Erlang — `do/end` keyword blocks
        "elixir" | "erlang" | ".ex" | ".exs" | ".erl" | ".hrl" => {
            format_do_end(text, indent_char, indent_size)
        }
        // Lua — end-based blocks (function/if/for/while...end)
        "lua" => format_lua(text, indent_char, indent_size),
        // Clojure / Lisp / Scheme / Racket — paren depth
        "clojure" | "lisp" | "scheme" | "racket" | ".clj" | ".cljs" | ".lisp" | ".lsp" | ".scm"
        | ".ss" => format_lisp(text, indent_char, indent_size),
        // Scala / Groovy — brace-based, string-aware
        "scala" | "groovy" | ".scala" | ".groovy" | ".kts" => {
            format_brace_lang(text, indent_char, indent_size)
        }
        // R / Julia — brace-based, string-aware
        "r" | "julia" | ".r" | ".R" | ".jl" => format_brace_lang(text, indent_char, indent_size),
        // Haskell / OCaml / Elm / F# — significant whitespace: normalize trailing WS only
        // Running a structural re-indenter on these would destroy the layout rule.
        "haskell" | "ocaml" | "elm" | "fsharp" | ".hs" | ".lhs" | ".ml" | ".mli" | ".elm"
        | ".fs" | ".fsi" | ".fsx" => format_layout_rule(text, config),
        // Unknown functional language — safe whitespace normalization only
        _ => format_layout_rule(text, config),
    };

    let mut out = result.into_bytes();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    Ok(out)
}

// ── Elixir / Erlang: do..end keyword blocks ────────────────────────────────

fn format_do_end(source: &str, indent_char: char, indent_size: usize) -> String {
    // Keywords that open a new level AFTER the current line is emitted
    let opens_kw: &[&str] = &["do", "fn", "try", "cond", "case", "with", "receive"];
    // Keywords that close a level BEFORE the current line is emitted
    let closes_kw: &[&str] = &["end"];
    // Keywords that close and re-open (rescue, else, catch, after, ->)
    let reopen_kw: &[&str] = &["rescue", "catch", "else", "after"];

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

        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // Close before emitting
        let is_close = closes_kw.contains(&first_word);
        let is_reopen = reopen_kw.contains(&first_word);
        if is_close || is_reopen {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        // Open after emitting
        let ends_do = trimmed.ends_with(" do") || trimmed == "do";
        let line_opens = opens_kw.iter().any(|kw| first_word == *kw || ends_do);
        let inline_do = trimmed.contains(", do:") || trimmed.contains(",do:");
        if (line_opens || is_reopen) && !is_close && !inline_do {
            depth += 1;
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Lua: function/if/for/while/repeat/do...end blocks ─────────────────────

fn format_lua(source: &str, indent_char: char, indent_size: usize) -> String {
    // Keywords that open a new level AFTER the current line is emitted
    let opens_kw: &[&str] = &["function", "if", "for", "while", "repeat", "do"];
    // Keywords that close a level BEFORE the current line is emitted
    let closes_kw: &[&str] = &["end", "until"];
    // Keywords that close and re-open (else, elseif)
    let reopen_kw: &[&str] = &["else", "elseif"];

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

        // Ignore pure comment lines for depth tracking
        let is_comment = trimmed.starts_with("--");

        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // ── Brace counting for table constructors ──────────────────────────
        // Count unquoted { and } to track table-constructor depth changes.
        let (brace_opens, brace_closes) = if !is_comment {
            count_unquoted_braces(trimmed)
        } else {
            (0i32, 0i32)
        };

        // Close before emitting: keyword closers AND lines that start with `}`
        let is_kw_close = closes_kw.contains(&first_word);
        let is_reopen = reopen_kw.contains(&first_word);
        let is_brace_close_line =
            !is_comment && trimmed.starts_with('}') && brace_closes > brace_opens;

        if is_kw_close || is_reopen || is_brace_close_line {
            depth = (depth - 1).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        if !is_comment {
            // Keyword-based opens
            let ends_with_open = trimmed.ends_with(" do")
                || trimmed == "do"
                || trimmed.ends_with(" then")
                || trimmed == "then"
                || trimmed == "repeat"
                || (first_word == "function" && trimmed.ends_with(')'))
                || (trimmed.starts_with("local function") && trimmed.ends_with(')'));
            let starts_open =
                opens_kw.contains(&first_word) || trimmed.starts_with("local function");
            let kw_opens = (starts_open && ends_with_open) || is_reopen;

            // Brace-based opens: net opens on this line (excluding brace-close lines)
            let net_brace_open = brace_opens - brace_closes;
            let brace_opens_depth = !is_brace_close_line && net_brace_open > 0;

            if kw_opens || brace_opens_depth {
                depth += 1;
            }
        }
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Count unquoted `{` and `}` in a Lua line, ignoring string contents.
fn count_unquoted_braces(line: &str) -> (i32, i32) {
    let mut opens = 0i32;
    let mut closes = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '{' if !in_single && !in_double => opens += 1,
            '}' if !in_single && !in_double => closes += 1,
            '-' if !in_single && !in_double && chars.peek() == Some(&'-') => {
                break; // rest of line is a comment
            }
            _ => {}
        }
    }
    (opens, closes)
}

fn format_lisp(source: &str, indent_char: char, indent_size: usize) -> String {
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

        // Lisp: `;` is a line comment
        let (opens, closes) = count_paren_delta(trimmed);
        // Closing parens that start the line decrease depth before emitting
        let leading_closes = trimmed.chars().take_while(|&c| c == ')').count() as i32;
        if leading_closes > 0 {
            depth = (depth - leading_closes).max(0);
        }

        let current_indent = make_indent(indent_char, indent_size, depth.max(0) as usize);
        out.push(format!("{}{}", current_indent, trimmed));

        // Net depth change: (opens - closes) ignoring the leading closes already handled
        let net = (opens as i32 - closes as i32).max(0);
        depth = (depth + net).max(0);
    }

    out.iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_paren_delta(line: &str) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut in_str = false;
    for c in line.chars() {
        match c {
            ';' if !in_str => break,
            '"' => in_str = !in_str,
            '(' if !in_str => opens += 1,
            ')' if !in_str => closes += 1,
            _ => {}
        }
    }
    (opens, closes)
}

// ── Scala / Groovy / R / Julia: string-aware brace formatter ─────────────

fn format_brace_lang(source: &str, indent_char: char, indent_size: usize) -> String {
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

fn count_brace_delta(line: &str) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    let mut str_char = '"';

    while let Some(c) = chars.next() {
        match c {
            '#' if !in_str => break, // R uses # for comments
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

// ── Haskell / OCaml / Elm: significant-whitespace safe pass-through ───────

/// These languages use the Haskell layout rule (significant indentation).
/// Re-indenting them with a structural formatter would destroy their semantics.
/// We do a safe whitespace-only normalization:
/// - Strip trailing whitespace on each line
/// - Normalize Haskell `import` ordering if configured
/// - Ensure a trailing newline
fn format_layout_rule(source: &str, config: &ConfigIR) -> String {
    let import_ordering = config
        .get_extra_str("haskell__importOrdering")
        .unwrap_or("preserve");

    let lines: Vec<&str> = source.lines().collect();
    let lines = if import_ordering != "preserve" {
        sort_haskell_imports(lines, import_ordering)
    } else {
        lines.into_iter().map(str::to_string).collect()
    };

    lines
        .iter()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sort_haskell_imports(lines: Vec<&str>, ordering: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().starts_with("import ") {
            let block_start = i;
            while i < lines.len()
                && (lines[i].trim().starts_with("import ") || lines[i].trim().is_empty())
            {
                i += 1;
            }
            let mut imports: Vec<String> = lines[block_start..i]
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect();
            match ordering {
                "alphabetical" => imports.sort_unstable(),
                "qualified-first" => imports.sort_unstable_by(|a, b| {
                    let a_q = a.contains("qualified");
                    let b_q = b.contains("qualified");
                    b_q.cmp(&a_q).then(a.cmp(b))
                }),
                _ => {}
            }
            for imp in imports {
                result.push(imp);
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }
    result
}

fn make_indent(c: char, size: usize, depth: usize) -> String {
    std::iter::repeat_n(c, size * depth).collect()
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
    fn haskell_layout_preserved() {
        // Haskell source must NOT be re-indented (layout rule)
        let src = b"module Main where\n\nmain :: IO ()\nmain = do\n  putStrLn \"hello\"\n  putStrLn \"world\"\n";
        let result = format_for(src, &cfg(), "haskell").unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // The `putStrLn` lines must retain their 2-space indent
        let put_line = s
            .lines()
            .find(|l| l.contains("putStrLn"))
            .expect("putStrLn missing");
        assert!(
            put_line.starts_with("  "),
            "Haskell layout must be preserved:\n{}",
            s
        );
    }

    #[test]
    fn elixir_do_end_indents_body() {
        let src = b"defmodule Foo do\ndef bar do\n42\nend\nend\n";
        let result = format_for(src, &cfg(), "elixir").unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let line_42 = s.lines().find(|l| l.trim() == "42").expect("42 missing");
        assert!(
            line_42.starts_with("  "),
            "Elixir body must be indented:\n{}",
            s
        );
    }

    #[test]
    fn lisp_paren_depth_tracked() {
        let src = b"(defn foo [x]\n(+ x 1))\n";
        let result = format_for(src, &cfg(), "lisp").unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // Second line starts with `(` which has no leading close parens — depth 1
        let body_line = s.lines().nth(1).expect("line 2 missing");
        assert!(
            body_line.starts_with("  "),
            "Lisp body must be indented:\n{}",
            s
        );
    }

    #[test]
    fn scala_brace_in_string_not_counted() {
        let src = b"class Foo {\n  val msg = \"open { brace\"\n  val x = 1\n}\n";
        let result = format_for(src, &cfg(), "scala").unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let x_line = s
            .lines()
            .find(|l| l.contains("val x"))
            .expect("val x missing");
        let msg_line = s
            .lines()
            .find(|l| l.contains("val msg"))
            .expect("val msg missing");
        assert_eq!(
            x_line.len() - x_line.trim_start().len(),
            msg_line.len() - msg_line.trim_start().len(),
            "string brace must not shift indent:\n{}",
            s
        );
    }

    #[test]
    fn haskell_import_ordering_alphabetical() {
        let mut config = cfg();
        config.extras.insert(
            "haskell__importOrdering".to_string(),
            serde_json::Value::String("alphabetical".to_string()),
        );
        let src = b"import Z.Z\nimport A.A\nimport M.M\n\nmain = return ()\n";
        let result = format_for(src, &config, "haskell").unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let a_pos = s.find("A.A").unwrap_or(usize::MAX);
        let m_pos = s.find("M.M").unwrap_or(usize::MAX);
        let z_pos = s.find("Z.Z").unwrap_or(usize::MAX);
        assert!(
            a_pos < m_pos && m_pos < z_pos,
            "imports must be alphabetical:\n{}",
            s
        );
    }

    #[test]
    fn strips_trailing_whitespace() {
        let src = b"main = return ()   \n";
        let result = format(src, &cfg()).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        for line in s.lines() {
            assert_eq!(line, line.trim_end());
        }
    }
}
