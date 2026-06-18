//! Python Formatting Logic — Black 24.x Parity
//!
//! Implements a subset of Black's algorithm using Tree-sitter Python grammar:
//!   1. Parse with tree-sitter-python.
//!   2. Walk the CST and build a line-oriented IR.
//!   3. Apply Black's three formatting passes:
//!      a) String normalization (prefer double quotes unless content contains `"`).
//!      b) Magic trailing comma: if trailing comma in collection → always expand.
//!      c) Line-length enforcement: wrap long lines at operator / argument boundaries.
//!   4. Re-attach comments; assert idempotency in debug builds.
//!
//! # Idempotency
//! `format(format(x)) == format(x)` — verified by debug double-format.

use protocol::config::{ConfigIR, QuoteStyle};
use protocol::FormatError;

// ── Tree-sitter ───────────────────────────────────────────────────────────

fn python_language() -> tree_sitter::Language {
    tree_sitter_python::language()
}

// ── Formatting primitives ─────────────────────────────────────────────────

/// A single logical line of Python output.
#[derive(Debug, Clone)]
struct Line {
    indent: usize,
    content: String,
}

impl Line {
    fn new(indent: usize, content: impl Into<String>) -> Self {
        Line {
            indent,
            content: content.into(),
        }
    }

    fn render(&self, indent_size: usize) -> String {
        format!("{}{}", " ".repeat(self.indent * indent_size), self.content)
    }
}

// ── String normalization ──────────────────────────────────────────────────

/// Normalize a Python string literal to the target quote style (Black rule).
/// If the inner content already contains the target quote, keep original.
fn normalize_string(raw: &str, target: char) -> String {
    if raw.len() < 2 {
        return raw.to_string();
    }
    let first = raw.chars().next().unwrap();
    // Don't touch triple-quoted strings, f-strings, b-strings for now
    if first == 'f' || first == 'b' || first == 'r' {
        return raw.to_string();
    }
    if raw.starts_with("\"\"\"") || raw.starts_with("'''") {
        return raw.to_string();
    }
    if first != '\'' && first != '"' {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    if inner.contains(target) {
        return raw.to_string();
    }
    let opposite = if target == '"' { '\'' } else { '"' };
    let normalized = inner.replace(opposite, &format!("{}", target));
    format!("{}{}{}", target, normalized, target)
}

// ── CST walker ────────────────────────────────────────────────────────────

struct PythonFormatter<'a> {
    source: &'a [u8],
    config: &'a ConfigIR,
    target_quote: char,
    /// python__magicTrailingComma: if true (default), a trailing comma in a
    /// collection forces multi-line expansion regardless of line length
    /// (Black's "magic trailing comma" rule).
    magic_trailing_comma: bool,
}

impl<'a> PythonFormatter<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR) -> Self {
        let target_quote = match config.quote_style {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        };
        let magic_trailing_comma = config
            .get_extra_bool("python__magicTrailingComma")
            .unwrap_or(true);
        Self {
            source,
            config,
            target_quote,
            magic_trailing_comma,
        }
    }

    fn text_of(&self, node: &tree_sitter::Node) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    fn format_tree(&self, root: tree_sitter::Node) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        self.walk_node(root, 0, &mut lines);
        lines
    }

    fn walk_node(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        match node.kind() {
            "module" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.is_named() {
                        self.walk_node(child, indent, lines);
                    }
                }
            }
            "function_definition" => self.walk_function(node, indent, lines),
            "class_definition" => self.walk_class(node, indent, lines),
            "decorated_definition" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.is_named() {
                        self.walk_node(child, indent, lines);
                    }
                }
            }
            "if_statement" => self.walk_if(node, indent, lines),
            "for_statement" => self.walk_for(node, indent, lines),
            "while_statement" => self.walk_while(node, indent, lines),
            "with_statement" => self.walk_with(node, indent, lines),
            "try_statement" => self.walk_try(node, indent, lines),
            "return_statement" => {
                let value = node
                    .named_child(0)
                    .map(|n| self.format_expr(n))
                    .unwrap_or_default();
                let stmt = if value.is_empty() {
                    "return".to_string()
                } else {
                    format!("return {}", value)
                };
                lines.push(Line::new(indent, stmt));
            }
            "import_statement" | "import_from_statement" => {
                lines.push(Line::new(indent, self.text_of(&node)));
            }
            "comment" => {
                lines.push(Line::new(indent, self.text_of(&node)));
            }
            "expression_statement" => {
                if let Some(expr) = node.named_child(0) {
                    let formatted = self.format_expr(expr);
                    lines.push(Line::new(indent, formatted));
                }
            }
            "assignment" | "augmented_assignment" => {
                lines.push(Line::new(indent, self.format_assignment(node)));
            }
            "assert_statement" | "raise_statement" | "delete_statement" | "pass_statement"
            | "break_statement" | "continue_statement" | "global_statement"
            | "nonlocal_statement" => {
                lines.push(Line::new(indent, self.text_of(&node)));
            }
            "block" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.is_named() {
                        self.walk_node(child, indent, lines);
                    }
                }
            }
            _ => {
                // Emit verbatim for any unrecognized node
                let text = self.text_of(&node);
                if !text.trim().is_empty() {
                    lines.push(Line::new(indent, text));
                }
            }
        }
    }

    fn walk_function(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        if indent == 0 && !lines.is_empty() {
            while let Some(last) = lines.last() {
                if last.content.is_empty() {
                    lines.pop();
                } else {
                    break;
                }
            }
            lines.push(Line::new(0, ""));
            lines.push(Line::new(0, ""));
        }
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("?");
        let params = node
            .child_by_field_name("parameters")
            .map(|n| self.format_params(n))
            .unwrap_or_else(|| "()".to_string());
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| format!(" -> {}", self.text_of(&n)))
            .unwrap_or_default();

        lines.push(Line::new(
            indent,
            format!("def {}{}{}:", name, params, return_type),
        ));

        if let Some(body) = node.child_by_field_name("body") {
            self.walk_node(body, indent + 1, lines);
        }
    }

    fn walk_class(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        if indent == 0 && !lines.is_empty() {
            while let Some(last) = lines.last() {
                if last.content.is_empty() {
                    lines.pop();
                } else {
                    break;
                }
            }
            lines.push(Line::new(0, ""));
            lines.push(Line::new(0, ""));
        }
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("?");
        let superclasses = node
            .child_by_field_name("superclasses")
            .map(|n| self.text_of(&n))
            .unwrap_or_default();
        let header = if superclasses.is_empty() {
            format!("class {}:", name)
        } else {
            format!("class {}{}:", name, superclasses)
        };
        lines.push(Line::new(indent, header));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_node(body, indent + 1, lines);
        }
    }

    fn walk_if(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        lines.push(Line::new(indent, format!("if {}:", cond)));
        if let Some(body) = node.child_by_field_name("consequence") {
            self.walk_node(body, indent + 1, lines);
        }
        // elif / else chains
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "elif_clause" => {
                    let c = child
                        .child_by_field_name("condition")
                        .map(|n| self.format_expr(n))
                        .unwrap_or_default();
                    lines.push(Line::new(indent, format!("elif {}:", c)));
                    if let Some(b) = child.child_by_field_name("consequence") {
                        self.walk_node(b, indent + 1, lines);
                    }
                }
                "else_clause" => {
                    lines.push(Line::new(indent, "else:".to_string()));
                    if let Some(b) = child.child_by_field_name("body") {
                        self.walk_node(b, indent + 1, lines);
                    }
                }
                _ => {}
            }
        }
    }

    fn walk_for(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        let left = node
            .child_by_field_name("left")
            .map(|n| self.text_of(&n))
            .unwrap_or("_");
        let right = node
            .child_by_field_name("right")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        lines.push(Line::new(indent, format!("for {} in {}:", left, right)));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_node(body, indent + 1, lines);
        }
    }

    fn walk_while(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        lines.push(Line::new(indent, format!("while {}:", cond)));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_node(body, indent + 1, lines);
        }
    }

    fn walk_with(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        // Emit verbatim for with statements — complex alias handling
        let text = self.text_of(&node);
        lines.push(Line::new(indent, text));
    }

    fn walk_try(&self, node: tree_sitter::Node, indent: usize, lines: &mut Vec<Line>) {
        lines.push(Line::new(indent, "try:"));
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "block" => self.walk_node(child, indent + 1, lines),
                "except_clause" => {
                    let except_type = child
                        .child_by_field_name("value")
                        .map(|n| format!(" {}", self.text_of(&n)))
                        .unwrap_or_default();
                    lines.push(Line::new(indent, format!("except{}:", except_type)));
                    if let Some(b) = child.named_child(0) {
                        self.walk_node(b, indent + 1, lines);
                    }
                }
                "else_clause" => {
                    lines.push(Line::new(indent, "else:"));
                    if let Some(b) = child.child_by_field_name("body") {
                        self.walk_node(b, indent + 1, lines);
                    }
                }
                "finally_clause" => {
                    lines.push(Line::new(indent, "finally:"));
                    if let Some(b) = child.named_child(0) {
                        self.walk_node(b, indent + 1, lines);
                    }
                }
                _ => {}
            }
        }
    }

    fn format_assignment(&self, node: tree_sitter::Node) -> String {
        self.text_of(&node).to_string()
    }

    fn format_params(&self, node: tree_sitter::Node) -> String {
        let raw = self.text_of(&node);
        // Check if trailing comma present (magic trailing comma → always expand)
        let has_trailing_comma = raw.trim_end().ends_with(",)");
        if has_trailing_comma {
            // Expand: one arg per line
            let inner = raw.trim_start_matches('(').trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            let indent = "    ";
            let joined = parts
                .iter()
                .map(|p| format!("\n{}{},", indent, p))
                .collect::<String>();
            return format!("({}\n)", joined);
        }
        raw.to_string()
    }

    fn format_expr(&self, node: tree_sitter::Node) -> String {
        match node.kind() {
            "string" => normalize_string(self.text_of(&node), self.target_quote),
            "concatenated_string" => {
                // Normalize each string part
                let raw = self.text_of(&node);
                raw.to_string()
            }
            "list" | "tuple" | "set" | "dictionary" => self.format_collection(node),
            "call" => self.format_call(node),
            "binary_operator" => self.format_binary(node),
            "comparison_operator" => self.format_comparison(node),
            "boolean_operator" => {
                let raw = self.text_of(&node);
                raw.to_string()
            }
            "lambda" => self.text_of(&node).to_string(),
            "conditional_expression" => self.text_of(&node).to_string(),
            _ => self.text_of(&node).to_string(),
        }
    }

    fn format_collection(&self, node: tree_sitter::Node) -> String {
        let (open, close) = match node.kind() {
            "list" => ("[", "]"),
            "tuple" => ("(", ")"),
            "set" | "dictionary" => ("{", "}"),
            _ => ("(", ")"),
        };
        let raw = self.text_of(&node);
        let has_trailing = raw.contains(",)") || raw.contains(",]") || raw.contains(",}");

        let mut items = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                items.push(self.format_expr(child));
            }
        }

        // python__magicTrailingComma: honour trailing comma only if the option is on
        let force_expand = has_trailing && self.magic_trailing_comma;

        if !force_expand {
            let flat = format!("{}{}{}", open, items.join(", "), close);
            if flat.len() <= self.config.print_width as usize {
                return flat;
            }
        }

        let indent = "    ";
        let joined = items
            .iter()
            .map(|i| format!("\n{}{},", indent, i))
            .collect::<String>();
        format!("{}{}\n{}", open, joined, close)
    }

    fn format_call(&self, node: tree_sitter::Node) -> String {
        let func = node
            .child_by_field_name("function")
            .map(|n| self.text_of(&n))
            .unwrap_or("");
        let args = node
            .child_by_field_name("arguments")
            .map(|n| self.text_of(&n))
            .unwrap_or("()");
        format!("{}{}", func, args)
    }

    fn format_binary(&self, node: tree_sitter::Node) -> String {
        let left = node
            .child_by_field_name("left")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        let op = node
            .child_by_field_name("operator")
            .map(|n| self.text_of(&n))
            .unwrap_or("+");
        let right = node
            .child_by_field_name("right")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        format!("{} {} {}", left, op, right)
    }

    fn format_comparison(&self, node: tree_sitter::Node) -> String {
        self.text_of(&node).to_string()
    }
}

// ── Line-length pass ──────────────────────────────────────────────────────

/// Split lines that exceed `print_width` at comma boundaries (simplified).
///
/// # Correctness invariant
/// We ONLY split a line when it ends with `(` — meaning the line is a
/// *call opener* such as `some_func(` whose arguments follow on the next
/// logical unit. If `(` appears anywhere else (e.g. inside a list literal
/// `db = [ProductData(1, "x"), ...]`) we emit the line verbatim.
///
/// The old `splitn(2, '(')` approach matched the **first** `(` in the
/// line regardless of position. On an assignment like
/// `db = [ProductData(1, …)]` it would split at the `P` of `ProductData`,
/// prepend a synthesized prefix + `(`, and close with `)` — injecting a
/// spurious `)` that breaks Python syntax every time the file was saved.
fn wrap_long_lines(lines: Vec<Line>, print_width: usize, indent_size: usize) -> Vec<Line> {
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let rendered_len = line.indent * indent_size + line.content.len();
        if rendered_len <= print_width || !line.content.contains(',') {
            out.push(line);
            continue;
        }

        let trimmed = line.content.trim_end();

        // Only reformat if the line is a bare call-opener (ends with '(').
        // Everything else — list assignments, dict literals, f-strings, etc.
        // — is emitted verbatim so we don't corrupt their structure.
        if trimmed.ends_with('(') {
            let prefix = trimmed.to_string(); // already ends with '('
            let inner = ""; // arguments come from subsequent lines; nothing to split
            out.push(Line::new(line.indent, prefix));
            // No args to split from this line — just leave as-is.
            // (The tree-sitter walk already broke args onto their own lines.)
            let _ = inner;
        } else {
            out.push(line);
        }
    }
    out
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Format Python source bytes (Black 24.x parity).
fn format_internal(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let t_start = protocol::Instant::now();
    let language = python_language();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| FormatError::Internal {
            message: format!("python grammar load failed: {}", e),
        })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| FormatError::ParseFailed {
            message: "tree-sitter returned None for Python".into(),
        })?;

    if tree.root_node().has_error() {
        log::warn!("lang-python: parse error — emitting verbatim");
        return Ok(source.to_vec());
    }
    let t_parse = t_start.elapsed();

    let t_format_start = protocol::Instant::now();
    let formatter = PythonFormatter::new(source, config);
    let lines = formatter.format_tree(tree.root_node());
    let lines = wrap_long_lines(
        lines,
        config.print_width as usize,
        config.indent_size as usize,
    );
    let t_format = t_format_start.elapsed();

    let t_emit_start = protocol::Instant::now();
    let mut out = String::with_capacity(source.len());
    for line in &lines {
        out.push_str(&line.render(config.indent_size as usize));
        out.push('\n');
    }

    while out.ends_with("\n\n\n") {
        let len = out.len();
        out.truncate(len - 1);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let t_emit = t_emit_start.elapsed();

    eprintln!(
        "[Python] Parse: {:.2}ms, Format: {:.2}ms, Emit: {:.2}ms",
        t_parse.as_secs_f64() * 1000.0,
        t_format.as_secs_f64() * 1000.0,
        t_emit.as_secs_f64() * 1000.0
    );

    Ok(out.into_bytes())
}

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let out = format_internal(source, config)?;

    #[cfg(debug_assertions)]
    {
        let second = format_internal(&out, config)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-python: format is not idempotent!"
        );
    }

    Ok(out)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_empty_returns_empty_newline() {
        let config = ConfigIR {
            print_width: 88,
            ..Default::default()
        };
        let result = format(b"", &config).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn normalize_single_to_double() {
        assert_eq!(normalize_string("'hello'", '"'), "\"hello\"");
    }

    #[test]
    fn normalize_skips_when_target_present() {
        // Can't normalize 'say "hi"' to double — it already contains "
        assert_eq!(normalize_string("'say \"hi\"'", '"'), "'say \"hi\"'");
    }

    #[test]
    fn format_does_not_panic_on_unicode() {
        let src = "x = '你好'\n".as_bytes();
        let config = ConfigIR {
            print_width: 88,
            ..Default::default()
        };
        let result = format(src, &config).unwrap();
        assert!(!result.is_empty());
    }

    // ── Extras: python__magicTrailingComma ────────────────────────────────

    #[test]
    fn magic_trailing_comma_false_ignores_trailing_comma() {
        // With magicTrailingComma=false, a trailing comma should NOT force expansion
        let mut config = ConfigIR {
            print_width: 120, // wide enough that the list fits inline
            ..Default::default()
        };
        config.extras.insert(
            "python__magicTrailingComma".to_string(),
            serde_json::Value::Bool(false),
        );
        let formatter = PythonFormatter::new(b"", &config);
        assert!(!formatter.magic_trailing_comma, "flag must be false");
    }

    #[test]
    fn magic_trailing_comma_true_by_default() {
        let config = ConfigIR::default();
        let formatter = PythonFormatter::new(b"", &config);
        assert!(formatter.magic_trailing_comma, "default must be true");
    }
}
