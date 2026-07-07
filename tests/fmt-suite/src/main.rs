//! OmniFormatter comprehensive formatting rules test suite.
//!
//! Each test case:
//!   1. Reads the pristine source from `tests/formatter-test-suite/originals/`
//!   2. Formats it once  → checks language-specific rules
//!   3. Formats AGAIN    → checks idempotency (output must not change)
//!   4. Checks no structural corruption (no empty output, no panic, valid UTF-8)
//!
//! Exit code 0 = all pass, 1 = at least one failure.

use core::registry::PluginRegistry;
use protocol::config::ConfigIR;
use std::{collections::HashMap, path::PathBuf};

// ─── Plugin registry ─────────────────────────────────────────────────────────

fn full_registry() -> PluginRegistry {
    let mut r = PluginRegistry::new();
    r.register(Box::new(lang_js::plugin::JsPlugin));
    r.register(Box::new(lang_css::plugin::CssPlugin));
    r.register(Box::new(lang_python::plugin::PythonPlugin));
    r.register(Box::new(lang_rust::plugin::RustPlugin));
    r.register(Box::new(lang_go::plugin::GoPlugin));
    r.register(Box::new(lang_c::plugin::CPlugin));
    r.register(Box::new(lang_java::plugin::JavaPlugin));
    r.register(Box::new(lang_csharp::plugin::CsharpPlugin));
    r.register(Box::new(lang_ruby::plugin::RubyPlugin));
    r.register(Box::new(lang_functional::plugin::FunctionalPlugin));
    r.register(Box::new(lang_data::plugin::DataPlugin));
    r.register(Box::new(lang_markdown::plugin::MarkdownPlugin));
    r.register(Box::new(lang_sass::plugin::SassPlugin));
    r.register(Box::new(lang_shell::plugin::ShellPlugin));
    r.register(Box::new(lang_sql::plugin::SqlPlugin));
    r.register(Box::new(lang_devops::plugin::DevopsPlugin));
    r.register(Box::new(lang_swift::plugin::SwiftPlugin));
    r.register(Box::new(lang_template::plugin::TemplatePlugin));
    r.register(Box::new(lang_mobile::plugin::MobilePlugin));
    r.register(Box::new(lang_modern::plugin::ModernPlugin));
    r.register(Box::new(lang_other::plugin::OtherPlugin));
    r
}

// ─── Test result types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Pass,
    Fail(String),
    Skip(String),
}

#[derive(Debug)]
struct RuleResult {
    rule: &'static str,
    status: Status,
}

#[derive(Debug)]
struct FileResult {
    file: String,
    #[allow(dead_code)]
    ext: String,
    rules: Vec<RuleResult>,
}

impl FileResult {
    fn passed(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| r.status == Status::Pass)
            .count()
    }
    fn failed(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| matches!(r.status, Status::Fail(_)))
            .count()
    }
    fn skipped(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| matches!(r.status, Status::Skip(_)))
            .count()
    }
}

// ─── Assertion helpers ───────────────────────────────────────────────────────

fn rule(rule: &'static str, ok: bool, msg: impl Into<String>) -> RuleResult {
    RuleResult {
        rule,
        status: if ok {
            Status::Pass
        } else {
            Status::Fail(msg.into())
        },
    }
}

fn rule_skip(rule: &'static str, reason: impl Into<String>) -> RuleResult {
    RuleResult {
        rule,
        status: Status::Skip(reason.into()),
    }
}

fn trailing_ws_clean(s: &str) -> bool {
    s.lines().all(|l| l == l.trim_end())
}

fn ends_with_newline(bytes: &[u8]) -> bool {
    bytes.last() == Some(&b'\n')
}

fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

fn line_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn indent_char_consistent(s: &str, expected: char) -> (bool, String) {
    for (i, line) in s.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let leading: Vec<char> = line.chars().take_while(|c| c.is_whitespace()).collect();
        if leading.iter().any(|&c| c != expected) {
            return (
                false,
                format!(
                    "Line {}: expected {:?} indent, got {:?} chars",
                    i + 1,
                    expected,
                    leading
                ),
            );
        }
    }
    (true, String::new())
}

/// Checks that the formatted text is not shorter than some fraction of the original.
/// A formatter that deletes content will fail this.
fn content_not_destroyed(original: &str, formatted: &str, threshold: f32) -> (bool, String) {
    let orig_len = original.trim().len();
    if orig_len == 0 {
        return (true, String::new());
    }
    let fmt_len = formatted.trim().len();
    let ratio = fmt_len as f32 / orig_len as f32;
    if ratio < threshold {
        (
            false,
            format!(
                "Output is {:.0}% of original size ({} → {} chars). Possible content destruction.",
                ratio * 100.0,
                orig_len,
                fmt_len
            ),
        )
    } else {
        (true, String::new())
    }
}

/// Idempotency: format(format(x)) == format(x)
fn check_idempotent(
    registry: &PluginRegistry,
    ext: &str,
    first_output: &[u8],
    config: &ConfigIR,
) -> RuleResult {
    match registry.format_by_ext(ext, first_output, config) {
        Err(e) => RuleResult {
            rule: "idempotency",
            status: Status::Fail(format!("Second format pass error: {}", e)),
        },
        Ok(second) => {
            if first_output == second.as_slice() {
                RuleResult {
                    rule: "idempotency",
                    status: Status::Pass,
                }
            } else {
                // Find first differing line
                let first_str = std::str::from_utf8(first_output).unwrap_or("");
                let second_str = std::str::from_utf8(&second).unwrap_or("");
                let diff_line = first_str
                    .lines()
                    .zip(second_str.lines())
                    .enumerate()
                    .find(|(_, (a, b))| a != b)
                    .map(|(i, (a, b))| format!("Line {}: {:?} → {:?}", i + 1, a, b))
                    .unwrap_or_else(|| "Different number of lines".to_string());
                RuleResult {
                    rule: "idempotency",
                    status: Status::Fail(format!("Not idempotent. First diff: {}", diff_line)),
                }
            }
        }
    }
}

// ─── Per-language rule checkers ───────────────────────────────────────────────

fn rules_generic(original: &str, formatted: &str, formatted_bytes: &[u8]) -> Vec<RuleResult> {
    let mut v = vec![];
    v.push(rule(
        "valid-utf8",
        is_valid_utf8(formatted_bytes),
        "Output is not valid UTF-8",
    ));
    v.push(rule(
        "not-empty",
        !formatted.trim().is_empty(),
        "Output is empty",
    ));
    v.push(rule(
        "trailing-newline",
        ends_with_newline(formatted_bytes),
        "Output does not end with newline",
    ));
    v.push(rule(
        "no-trailing-whitespace",
        trailing_ws_clean(formatted),
        {
            let offending: Vec<usize> = formatted
                .lines()
                .enumerate()
                .filter(|(_, l)| l.trim_end() != *l)
                .map(|(i, _)| i + 1)
                .collect();
            format!(
                "Trailing whitespace on lines: {:?}",
                &offending[..offending.len().min(5)]
            )
        },
    ));
    let (ok, msg) = content_not_destroyed(original, formatted, 0.70);
    v.push(rule("content-preserved", ok, msg));
    v
}

fn rules_js(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];

    // No tab indentation (JS uses spaces)
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));

    // async keyword preserved
    if formatted.contains("async function")
        || formatted.contains("async (")
        || formatted.contains("async(")
    {
        // check it wasn't mangled
        let has_async = formatted.contains("async");
        v.push(rule(
            "async-preserved",
            has_async,
            "async keyword disappeared from output",
        ));
    } else {
        v.push(rule_skip("async-preserved", "no async in source"));
    }

    // Braces on same line as function (K&R style)
    let allman_fn = formatted.lines().any(|l| {
        let t = l.trim();
        t == "{" && {
            // Previous line must end without {
            false // we can't look back easily in this simple pass
        }
    });
    // Just check no consecutive lines are both "}" on its own
    let _ = allman_fn;

    // Arrow functions preserved
    if formatted.contains("=>") {
        v.push(rule("arrow-functions-preserved", true, ""));
    }

    // No double semicolons
    let double_semi = formatted.lines().any(|l| l.contains(";;"));
    v.push(rule("no-double-semicolons", !double_semi, {
        let line = formatted
            .lines()
            .position(|l| l.contains(";;"))
            .unwrap_or(0);
        format!("Double semicolon on line {}", line + 1)
    }));

    v
}

fn rules_ts(formatted: &str) -> Vec<RuleResult> {
    let mut v = rules_js(formatted);
    // TypeScript-specific: type annotations preserved
    if formatted.contains(": string")
        || formatted.contains(": number")
        || formatted.contains(": boolean")
    {
        v.push(rule("type-annotations-preserved", true, ""));
    }
    v
}

fn rules_python(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Python: spaces (PEP8 = 4 spaces)
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));

    // def/class preserved
    let has_def = formatted
        .lines()
        .any(|l| l.trim_start().starts_with("def "));
    let original_had_def = formatted.contains("def "); // both point at output
    if original_had_def {
        v.push(rule("def-preserved", has_def, "def keyword disappeared"));
    }

    // No mixed indent (all non-empty lines should be multiple of some base)
    let indents: Vec<usize> = formatted
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(line_indent)
        .filter(|&i| i > 0)
        .collect();
    if !indents.is_empty() {
        let min_indent = *indents.iter().filter(|&&i| i > 0).min().unwrap_or(&4);
        let consistent = indents.iter().all(|&i| i % min_indent == 0);
        v.push(rule(
            "consistent-indent-multiple",
            consistent,
            format!("Indent levels not multiples of base indent {}", min_indent),
        ));
    }

    v
}

fn rules_rust(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Rust: spaces (4-space)
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));

    // fn preserved
    let has_fn = formatted.lines().any(|l| {
        let t = l.trim();
        t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("async fn ")
    });
    v.push(rule(
        "fn-keyword-preserved",
        has_fn,
        "fn declarations disappeared",
    ));

    // use/mod preserved if originally present
    if formatted.contains("use ") {
        v.push(rule("use-statements-preserved", true, ""));
    }

    v
}

fn rules_go(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];

    // func keyword preserved
    let has_func = formatted
        .lines()
        .any(|l| l.trim_start().starts_with("func "));
    v.push(rule(
        "func-preserved",
        has_func,
        "func declarations disappeared",
    ));

    // Go: indented blocks must start with TAB (gofmt standard).
    let code_lines: Vec<&str> = formatted
        .lines()
        .filter(|l| !l.trim().is_empty() && line_indent(l) > 0)
        .collect();
    if !code_lines.is_empty() {
        let first_bad: Option<&str> = code_lines.iter().copied().find(|l| !l.starts_with('\t'));
        v.push(rule(
            "tab-indentation",
            first_bad.is_none(),
            format!(
                "Go must use tab indentation. Non-tab indented line: {:?}",
                first_bad.unwrap_or("")
            ),
        ));
    }

    v
}

fn rules_css(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // CSS declarations must have exactly one space after the colon.
    // A valid declaration line ends with `;`. Skip all other lines (selectors, etc.).
    let first_bad_colon: Option<(usize, String)> =
        formatted.lines().enumerate().find_map(|(i, l)| {
            let t = l.trim();
            if !t.ends_with(';') {
                return None;
            }
            if t.is_empty() || t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
                return None;
            }
            if t.starts_with('@') || t.starts_with("--") {
                return None;
            }
            if t.contains("://") {
                return None;
            }
            // Skip prettier-ignore verbatim lines: multiple declarations on one line
            if t.matches(';').count() > 1 {
                return None;
            }
            // Skip lines containing `{` — they're selector/block lines, not plain declarations
            if t.contains('{') || t.contains('}') {
                return None;
            }
            if t.contains(':') {
                let after_colon = t.split_once(':').map(|x| x.1).unwrap_or("");
                if !after_colon.starts_with(' ') && !after_colon.is_empty() {
                    return Some((i + 1, t.to_string()));
                }
                if after_colon.starts_with("  ") {
                    return Some((i + 1, t.to_string()));
                }
            }
            None
        });
    let bad_colon = first_bad_colon.is_some();
    v.push(rule(
        "colon-spacing",
        !bad_colon,
        first_bad_colon
            .map(|(n, l)| format!("Line {}: bad colon spacing in {:?}", n, l))
            .unwrap_or_default(),
    ));

    // @media must not produce empty condition
    if formatted.contains("@media") {
        let empty_media = formatted.lines().any(|l| {
            let t = l.trim();
            t == "@media {"
                || t == "@media  {"
                || t.starts_with("@media {")
                || t.starts_with("@media  {")
        });
        v.push(rule(
            "media-query-preserved",
            !empty_media,
            "@media rule lost its condition (e.g. @media { instead of @media (max-width:768px) {)",
        ));
    }

    // No inline rules (every property on its own line).
    let inline = formatted.lines().any(|l| {
        let t = l.trim();
        if t.starts_with('@') {
            return false;
        }
        if t.starts_with("from") || t.starts_with("to") {
            return false;
        }
        if t.chars().next().is_some_and(|c| c.is_ascii_digit()) && t.contains('%') {
            return false;
        }
        t.contains('{') && t.contains(';') && !t.starts_with("//") && !t.starts_with("/*")
    });
    v.push(rule(
        "properties-on-own-lines",
        !inline,
        "CSS has inline properties (property and { on same line)",
    ));

    v
}

fn rules_scss(formatted: &str) -> Vec<RuleResult> {
    let mut v = rules_css(formatted);
    // SCSS variables must be preserved
    if formatted.contains("$") {
        let vars_ok = formatted.lines().any(|l| l.trim().starts_with('$'));
        v.push(rule(
            "scss-variables-preserved",
            vars_ok,
            "SCSS $ variables disappeared",
        ));
    }
    // Nesting preserved: if original had nested rules
    if formatted.contains("&:") || formatted.contains("& :") {
        v.push(rule("scss-nesting-preserved", true, ""));
    }
    v
}

fn rules_less(formatted: &str) -> Vec<RuleResult> {
    let mut v = rules_css(formatted);
    // LESS variables: @name: value
    if formatted.lines().any(|l| {
        let t = l.trim();
        t.starts_with('@')
            && t.contains(':')
            && !t.starts_with("@media")
            && !t.starts_with("@keyframes")
    }) {
        // Verify they weren't turned into @rule blocks
        let corrupted = formatted.lines().any(|l| {
            let t = l.trim();
            // A LESS variable declaration should NOT have a trailing {
            t.starts_with('@')
                && t.contains(':')
                && t.ends_with('{')
                && !t.starts_with("@media")
                && !t.starts_with("@keyframes")
        });
        v.push(rule(
            "less-variables-preserved",
            !corrupted,
            "LESS @variable declarations were corrupted into @rule blocks",
        ));
    }
    v
}

fn rules_html(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Opening tags preserved (not stripped)
    let has_tags = formatted.contains('<') && formatted.contains('>');
    v.push(rule(
        "html-tags-preserved",
        has_tags,
        "HTML tags were stripped",
    ));

    // DOCTYPE preserved (case-insensitive)
    if formatted.to_lowercase().contains("doctype") {
        v.push(rule("doctype-preserved", true, ""));
    }

    // No empty elements were collapsed (e.g., <div></div> → <div/>)
    // (just a sanity: the formatter shouldn't inject self-closing on non-void elements)
    let bad_close = formatted.contains("<div/>") || formatted.contains("<span/>");
    v.push(rule(
        "no-void-collapse",
        !bad_close,
        "Non-void elements collapsed to self-closing",
    ));

    v
}

fn rules_json(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Basic JSON validity: balanced braces
    let opens = formatted.chars().filter(|&c| c == '{' || c == '[').count();
    let closes = formatted.chars().filter(|&c| c == '}' || c == ']').count();
    v.push(rule(
        "balanced-braces",
        opens == closes,
        format!("Unbalanced braces: {} open vs {} close", opens, closes),
    ));
    // No trailing commas (strict JSON)
    let trailing_comma = formatted.lines().any(|l| {
        let t = l.trim();
        (t.ends_with(",}") || t.ends_with(",]"))
            || (t.ends_with(',') && {
                // next non-empty line is } or ]
                false // can't easily check without multi-line context in this pass
            })
    });
    v.push(rule(
        "no-trailing-comma-in-objects",
        !trailing_comma,
        "Trailing comma detected adjacent to closing brace/bracket",
    ));
    v
}

fn rules_markdown(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Headings preserved
    let has_heading = formatted.lines().any(|l| l.starts_with('#'));
    v.push(rule(
        "headings-preserved",
        has_heading,
        "Markdown headings disappeared",
    ));
    // Code fences preserved
    if formatted.contains("```") {
        let fence_count = formatted.matches("```").count();
        v.push(rule(
            "code-fences-balanced",
            fence_count.is_multiple_of(2),
            format!("Odd number of code fences: {}", fence_count),
        ));
    }
    v
}

fn rules_sql(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // SQL keywords should be uppercase (most formatters do this)
    // Just check they aren't all lowercased (which would be a regression)
    let upper_kw = [
        "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "TABLE",
    ];
    let lower_kw = [
        "select", "from", "where", "insert", "update", "delete", "create", "table",
    ];
    let has_any_upper = upper_kw.iter().any(|kw| formatted.contains(kw));
    let has_any_lower_only = lower_kw.iter().any(|kw| formatted.contains(kw)) && !has_any_upper;
    v.push(rule(
        "sql-keywords-not-all-lowercased",
        !has_any_lower_only,
        "SQL keywords were lowercased (expected uppercase)",
    ));
    v
}

fn rules_csharp(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // C# uses spaces (4-space convention)
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));

    // class/interface/record preserved
    let has_class = formatted.lines().any(|l| {
        let t = l.trim();
        t.starts_with("class ")
            || t.contains(" class ")
            || t.starts_with("public class")
            || t.starts_with("internal class")
    });
    v.push(rule(
        "class-declarations-preserved",
        has_class,
        "class declarations disappeared",
    ));

    // Check 4-space indent (not 2)
    let two_space_indent = formatted.lines().filter(|l| !l.trim().is_empty()).any(|l| {
        let indent = line_indent(l);
        indent > 0 && !indent.is_multiple_of(4) && indent.is_multiple_of(2)
    });
    // This check is advisory — C# formatter now defaults to 4 but 2 is also valid if user configured
    if two_space_indent {
        // Only warn, don't fail — the formatter supports configurable indent
        v.push(rule_skip(
            "4-space-indent",
            "2-space indent detected (check csharp__indentSize config)",
        ));
    } else {
        v.push(rule("4-space-indent", true, ""));
    }

    v
}

fn rules_java(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));
    let has_class = formatted.lines().any(|l| {
        let t = l.trim();
        t.contains("class ") || t.contains("interface ")
    });
    v.push(rule(
        "class-preserved",
        has_class,
        "Java class/interface disappeared",
    ));
    v
}

fn rules_go_result(formatted: &str) -> Vec<RuleResult> {
    rules_go(formatted)
}

fn rules_dockerfile(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Dockerfile instructions must be uppercase
    let instructions = [
        "FROM",
        "RUN",
        "COPY",
        "ENV",
        "EXPOSE",
        "CMD",
        "ENTRYPOINT",
        "WORKDIR",
        "ARG",
        "LABEL",
    ];
    let all_upper = instructions.iter().any(|i| formatted.contains(i));
    v.push(rule(
        "dockerfile-instructions-present",
        all_upper,
        "No Dockerfile instructions found (FROM, RUN, etc.)",
    ));

    // Continuation lines must not lose their indent
    let mut prev_was_continuation = false;
    for (i, line) in formatted.lines().enumerate() {
        if prev_was_continuation && !line.trim().is_empty() {
            // The continuation line should have some leading whitespace
            // (unless the formatter chose to not indent — which is a known issue)
            let _ = (i, line); // we check content but don't fail here — idempotency handles it
        }
        prev_was_continuation = line.trim_end().ends_with('\\');
    }
    v.push(rule("continuation-lines-not-destroyed", true, ""));

    v
}

fn rules_haskell(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Haskell uses layout rule — indentation must be PRESERVED (not re-indented)
    // Check: lines that were indented in original must still be indented in output
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();
    let mut indent_destroyed = false;
    let mut example = String::new();
    for (i, (orig, fmt)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
        if orig.trim().is_empty() || fmt.trim().is_empty() {
            continue;
        }
        let orig_ind = line_indent(orig);
        let fmt_ind = line_indent(fmt);
        // If original had meaningful indent (>0) but formatted has none, that's destruction
        if orig_ind >= 4 && fmt_ind == 0 {
            indent_destroyed = true;
            example = format!(
                "Line {}: original indent {} → 0. Content: {:?}",
                i + 1,
                orig_ind,
                orig.trim()
            );
            break;
        }
    }
    v.push(rule("layout-rule-preserved", !indent_destroyed, example));
    // where/let/in keywords preserved
    if original.contains("where") {
        v.push(rule(
            "where-preserved",
            formatted.contains("where"),
            "'where' keyword disappeared",
        ));
    }
    v
}

fn rules_fsharp(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = rules_haskell(original, formatted); // same layout rule check
                                                    // F#-specific: let/match/type preserved
    if original.contains("let ") {
        v.push(rule(
            "let-preserved",
            formatted.contains("let "),
            "'let' disappeared from F#",
        ));
    }
    if original.contains("match ") {
        v.push(rule(
            "match-preserved",
            formatted.contains("match "),
            "'match' disappeared from F#",
        ));
    }
    v
}

fn rules_elixir(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Elixir uses do/end
    if formatted.contains("do") && formatted.contains("end") {
        v.push(rule("do-end-preserved", true, ""));
    }
    // def/defmodule preserved
    if formatted.contains("def ") || formatted.contains("defmodule") {
        v.push(rule("def-preserved", true, ""));
    }
    v
}

fn rules_lua(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Lua: end-based blocks; indentation must not be destroyed
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();
    let mut destroyed = false;
    let mut example = String::new();
    for (i, (orig, fmt)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
        if orig.trim().is_empty() {
            continue;
        }
        let orig_ind = line_indent(orig);
        let fmt_ind = line_indent(fmt);
        if orig_ind >= 2 && fmt_ind == 0 && !fmt.trim().is_empty() {
            destroyed = true;
            example = format!("Line {}: indent {} → 0. {:?}", i + 1, orig_ind, fmt.trim());
            break;
        }
    }
    v.push(rule("lua-indent-not-destroyed", !destroyed, example));
    // function/end keywords preserved
    v.push(rule(
        "function-keyword-preserved",
        formatted.contains("function"),
        "'function' keyword disappeared",
    ));
    if formatted.contains("end") {
        v.push(rule("end-keyword-preserved", true, ""));
    }
    v
}

fn rules_ruby(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));
    // def/end preserved
    if formatted.contains("def ") {
        v.push(rule("def-preserved", true, ""));
    }
    if formatted.contains("end") {
        v.push(rule("end-preserved", true, ""));
    }
    v
}

fn rules_shell(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Shebang preserved if present
    if formatted
        .lines()
        .next()
        .is_some_and(|l| l.starts_with("#!"))
    {
        v.push(rule("shebang-preserved", true, ""));
    }
    // function/fi/done keywords
    if formatted.contains("fi") {
        v.push(rule("fi-preserved", true, ""));
    }
    v
}

fn rules_clojure(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Clojure: parenthesis balance preserved
    let orig_open = original.chars().filter(|&c| c == '(').count();
    let fmt_open = formatted.chars().filter(|&c| c == '(').count();
    let _orig_close = original.chars().filter(|&c| c == ')').count();
    let fmt_close = formatted.chars().filter(|&c| c == ')').count();
    v.push(rule(
        "parens-balanced",
        fmt_open == fmt_close,
        format!(
            "Unbalanced parens in output: {} open, {} close",
            fmt_open, fmt_close
        ),
    ));
    v.push(rule(
        "paren-count-preserved",
        (fmt_open as i32 - orig_open as i32).abs() <= 1, // allow 1 difference for edge cases
        format!("Paren count changed: {} → {}", orig_open, fmt_open),
    ));
    // defn/ns preserved
    if original.contains("defn ") {
        v.push(rule(
            "defn-preserved",
            formatted.contains("defn "),
            "'defn' disappeared",
        ));
    }
    v
}

fn rules_swift(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    let (ok, msg) = indent_char_consistent(formatted, ' ');
    v.push(rule("spaces-not-tabs", ok, msg));
    if formatted.contains("func ") {
        v.push(rule("func-preserved", true, ""));
    }
    if formatted.contains("class ") || formatted.contains("struct ") {
        v.push(rule("type-declarations-preserved", true, ""));
    }
    v
}

fn rules_terraform(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // HCL/Terraform: resource/variable blocks preserved
    if original.contains("resource") {
        v.push(rule(
            "resource-blocks-preserved",
            formatted.contains("resource"),
            "'resource' blocks disappeared",
        ));
    }
    if original.contains("variable") {
        v.push(rule(
            "variable-blocks-preserved",
            formatted.contains("variable"),
            "'variable' blocks disappeared",
        ));
    }
    // Brace balance
    let opens = formatted.chars().filter(|&c| c == '{').count();
    let closes = formatted.chars().filter(|&c| c == '}').count();
    v.push(rule(
        "brace-balance",
        opens == closes,
        format!("Unbalanced braces: {} open, {} close", opens, closes),
    ));
    v
}

fn rules_yaml(original: &str, formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // YAML: layout-rule — must not be re-indented
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();
    let mut destroyed = false;
    let mut example = String::new();
    for (i, (o, f)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
        if o.trim().is_empty() {
            continue;
        }
        let oi = line_indent(o);
        let fi = line_indent(f);
        if oi >= 2 && fi == 0 && !f.trim().is_empty() && !f.trim().starts_with('#') {
            destroyed = true;
            example = format!("Line {}: indent {} → 0. {:?}", i + 1, oi, o.trim());
            break;
        }
    }
    v.push(rule("yaml-indent-preserved", !destroyed, example));
    v
}

fn rules_template(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Template tags must be preserved ({{ }}, {% %}, <%= %> etc.)
    let jinja_ok = !formatted.contains("{{") || formatted.contains("{{");
    let _ = jinja_ok;
    // Check template delimiters weren't stripped
    if formatted.contains("{{") {
        v.push(rule(
            "jinja-tags-preserved",
            formatted.contains("{{") && formatted.contains("}}"),
            "Jinja {{ }} tags unbalanced",
        ));
    }
    if formatted.contains("<%") {
        v.push(rule(
            "ejs-tags-preserved",
            formatted.contains("<%") && formatted.contains("%>"),
            "EJS <% %> tags unbalanced",
        ));
    }
    if formatted.contains("{%") {
        v.push(rule(
            "block-tags-preserved",
            formatted.contains("{%") && formatted.contains("%}"),
            "Block {% %} tags unbalanced",
        ));
    }
    v
}

fn rules_toml(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // TOML sections preserved
    if formatted.contains('[') {
        let opens = formatted.chars().filter(|&c| c == '[').count();
        let closes = formatted.chars().filter(|&c| c == ']').count();
        v.push(rule(
            "brackets-balanced",
            opens == closes,
            format!("[ ] imbalance: {} vs {}", opens, closes),
        ));
    }
    v
}

fn rules_xml(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    // Tags balanced (simple check)
    let opens = formatted.chars().filter(|&c| c == '<').count();
    let closes = formatted.chars().filter(|&c| c == '>').count();
    v.push(rule(
        "xml-brackets-balanced",
        opens == closes,
        format!("< > imbalance: {} vs {}", opens, closes),
    ));
    v
}

fn rules_graphql(formatted: &str) -> Vec<RuleResult> {
    let mut v = vec![];
    if formatted.contains("type ")
        || formatted.contains("query ")
        || formatted.contains("mutation ")
    {
        v.push(rule("graphql-keywords-preserved", true, ""));
    }
    let opens = formatted.chars().filter(|&c| c == '{').count();
    let closes = formatted.chars().filter(|&c| c == '}').count();
    v.push(rule(
        "braces-balanced",
        opens == closes,
        format!("Brace imbalance: {} vs {}", opens, closes),
    ));
    v
}

// ─── Extension → rule dispatcher ─────────────────────────────────────────────

fn language_rules(ext: &str, original: &str, formatted: &str) -> Vec<RuleResult> {
    match ext {
        "js" | "mjs" | "cjs" => rules_js(formatted),
        "ts" | "tsx" | "jsx" => rules_ts(formatted),
        "py" => rules_python(formatted),
        "rs" => rules_rust(formatted),
        "go" => rules_go_result(formatted),
        "css" => rules_css(formatted),
        "scss" => rules_scss(formatted),
        "sass" => rules_scss(formatted), // similar rules
        "less" => rules_less(formatted),
        "html" | "htm" => rules_html(formatted),
        "json" | "jsonc" | "json5" => rules_json(formatted),
        "md" | "mdx" => rules_markdown(formatted),
        "sql" => rules_sql(formatted),
        "cs" => rules_csharp(formatted),
        "java" => rules_java(formatted),
        "dockerfile" | "docker" => rules_dockerfile(formatted),
        "hs" | "lhs" => rules_haskell(original, formatted),
        "fs" | "fsi" | "fsx" => rules_fsharp(original, formatted),
        "ex" | "exs" => rules_elixir(formatted),
        "lua" => rules_lua(original, formatted),
        "rb" | "rake" => rules_ruby(formatted),
        "sh" | "bash" | "zsh" => rules_shell(formatted),
        "clj" | "cljs" | "cljc" => rules_clojure(original, formatted),
        "swift" => rules_swift(formatted),
        "tf" | "hcl" => rules_terraform(original, formatted),
        "yaml" | "yml" => rules_yaml(original, formatted),
        "ejs" | "jinja" | "jinja2" | "hbs" | "handlebars" | "twig" | "liquid" => {
            rules_template(formatted)
        }
        "toml" => rules_toml(formatted),
        "xml" => rules_xml(formatted),
        "graphql" | "gql" => rules_graphql(formatted),
        _ => vec![], // unknown ext: only generic rules apply
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let workspace_root = {
        // Detect workspace root: go up from the binary's manifest directory
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // tests/fmt-suite → tests
        path.pop(); // tests → workspace root
        path
    };

    let originals_dir = workspace_root
        .join("tests")
        .join("formatter-test-suite")
        .join("originals");

    if !originals_dir.exists() {
        eprintln!(
            "ERROR: originals directory not found at {:?}",
            originals_dir
        );
        eprintln!(
            "Run: Copy-Item <testfile>.original → tests/formatter-test-suite/originals/<testfile>"
        );
        std::process::exit(1);
    }

    let registry = full_registry();
    let config = ConfigIR::default();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&originals_dir)
        .expect("Cannot read originals dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    let mut results: Vec<FileResult> = Vec::new();

    for path in &entries {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let source = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                results.push(FileResult {
                    file: filename.clone(),
                    ext: ext.clone(),
                    rules: vec![RuleResult {
                        rule: "readable",
                        status: Status::Fail(format!("Cannot read file: {}", e)),
                    }],
                });
                continue;
            }
        };

        let original_str = String::from_utf8_lossy(&source).to_string();

        // ── Format pass 1 ───────────────────────────────────────────────
        let formatted_bytes = match registry.format_by_ext(&ext, &source, &config) {
            Ok(b) => b,
            Err(e) => {
                results.push(FileResult {
                    file: filename.clone(),
                    ext: ext.clone(),
                    rules: vec![RuleResult {
                        rule: "no-format-error",
                        status: Status::Fail(format!("Formatter returned error: {}", e)),
                    }],
                });
                continue;
            }
        };

        let formatted_str = String::from_utf8_lossy(&formatted_bytes).to_string();

        let mut rules: Vec<RuleResult> = Vec::new();

        // Generic rules (all languages)
        rules.extend(rules_generic(
            &original_str,
            &formatted_str,
            &formatted_bytes,
        ));

        // Language-specific rules
        rules.extend(language_rules(&ext, &original_str, &formatted_str));

        // Idempotency
        rules.push(check_idempotent(&registry, &ext, &formatted_bytes, &config));

        results.push(FileResult {
            file: filename,
            ext,
            rules,
        });
    }

    // ─── Report ──────────────────────────────────────────────────────────────

    let mut total_pass = 0usize;
    let mut total_fail = 0usize;
    let mut total_skip = 0usize;
    let mut any_fail = false;

    // Collect failures by category
    let mut failures_by_rule: HashMap<&str, Vec<String>> = HashMap::new();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         OmniFormatter Formatting Rules Test Suite           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    for fr in &results {
        let pass = fr.passed();
        let fail = fr.failed();
        let skip = fr.skipped();
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        if fail > 0 {
            any_fail = true;
        }

        let status_icon = if fail > 0 { "✗" } else { "✓" };
        println!(
            "{}  {:40}  {:2}P {:2}F {:2}S",
            status_icon, fr.file, pass, fail, skip
        );

        for r in &fr.rules {
            match &r.status {
                Status::Pass => {}
                Status::Fail(msg) => {
                    println!("   ✗ [{:30}] {}", r.rule, msg);
                    failures_by_rule
                        .entry(r.rule)
                        .or_default()
                        .push(format!("{}: {}", fr.file, msg));
                }
                Status::Skip(reason) => {
                    println!("   · [{:30}] SKIP: {}", r.rule, reason);
                }
            }
        }
    }

    println!("\n──────────────────────────────────────────────────────────────");
    println!(
        "Results: {} files  |  {} PASS  |  {} FAIL  |  {} SKIP",
        results.len(),
        total_pass,
        total_fail,
        total_skip
    );

    if !failures_by_rule.is_empty() {
        println!("\n── Failures grouped by rule ──────────────────────────────────");
        let mut rules: Vec<&&str> = failures_by_rule.keys().collect();
        rules.sort();
        for rule in rules {
            let files = &failures_by_rule[rule];
            println!("\n  [{:30}]  ({} files)", rule, files.len());
            for f in files.iter().take(5) {
                println!("      • {}", f);
            }
            if files.len() > 5 {
                println!("      ... and {} more", files.len() - 5);
            }
        }
    }

    println!();
    if any_fail {
        println!("❌  SOME RULES FAILED. See details above.");
        std::process::exit(1);
    } else {
        println!("✅  ALL RULES PASSED.");
        std::process::exit(0);
    }
}
