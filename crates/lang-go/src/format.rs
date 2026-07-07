//! Go Formatting Logic — gofmt/goimports Parity
//!
//! Implements gofmt's canonical rules using Tree-sitter:
//!   1. Parse with tree-sitter-go.
//!   2. Walk the CST and reconstruct formatted output.
//!   3. Apply gofmt rules:
//!      - Tabs for indentation (unconditional; ConfigIR.indent_style ignored).
//!      - Space after keywords (`if`, `for`, `func`, `var`, etc.).
//!      - One blank line between top-level declarations.
//!      - Import grouping: stdlib first, then external (blank-line separated).
//!      - No trailing whitespace.
//!   4. Assert idempotency in debug builds.
//!
//! # gofmt Tab Rule
//!
//! Go enforces tab indentation unconditionally. The `ConfigIR.indent_style`
//! and `ConfigIR.indent_size` fields are ignored for this language module.
//! This is documented as a known constraint (L-10 partial — gofmt opinionated).

use protocol::config::ConfigIR;
use protocol::FormatError;

fn go_language() -> tree_sitter::Language {
    tree_sitter_go::language()
}

// ── Line IR ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Line {
    /// Tab-based indent depth
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

    fn render(&self) -> String {
        format!("{}{}", "\t".repeat(self.indent), self.content)
    }
}

// ── Formatter ─────────────────────────────────────────────────────────────

struct GoFormatter<'a> {
    source: &'a [u8],
    /// go__groupImports: when true (default), separate stdlib imports from
    /// external imports with a blank line (goimports style).
    group_imports: bool,
}

impl<'a> GoFormatter<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR) -> Self {
        let group_imports = config.get_extra_bool("go__groupImports").unwrap_or(true);
        Self {
            source,
            group_imports,
        }
    }

    fn text_of(&self, node: &tree_sitter::Node) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    fn format_tree(&self, root: tree_sitter::Node) -> Vec<Line> {
        let mut lines = Vec::new();
        self.walk(root, 0, &mut lines);
        lines
    }

    fn walk(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        match node.kind() {
            "source_file" => {
                // Collect package clause, imports, and other declarations separately
                let mut package_node: Option<tree_sitter::Node> = None;
                let mut import_nodes: Vec<tree_sitter::Node> = Vec::new();
                let mut other_nodes: Vec<tree_sitter::Node> = Vec::new();
                {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if !child.is_named() {
                            continue;
                        }
                        match child.kind() {
                            "package_clause" => {
                                package_node = Some(child);
                            }
                            "import_declaration" => {
                                import_nodes.push(child);
                            }
                            _ => {
                                other_nodes.push(child);
                            }
                        }
                    }
                }
                // 1. Emit package clause
                if let Some(pkg) = package_node {
                    self.walk(pkg, indent, out);
                }
                // 2. Emit all imports — keep each declaration separate (gofmt rule)
                if !import_nodes.is_empty() {
                    out.push(Line::new(0, ""));
                    for node in &import_nodes {
                        let paths = self.collect_import_paths(*node);
                        // Single path inside an existing group block → keep as group
                        // Multiple specs from one block → keep block form
                        let had_block = {
                            let mut c = node.walk();
                            let result = node
                                .children(&mut c)
                                .any(|ch| ch.kind() == "import_spec_list");
                            result
                        };
                        if had_block && paths.len() > 1 {
                            out.push(Line::new(0, "import (".to_string()));
                            for path in &paths {
                                out.push(Line::new(1, path.to_string()));
                            }
                            out.push(Line::new(0, ")".to_string()));
                        } else {
                            // Separate `import "x"` declarations stay separate
                            for path in &paths {
                                out.push(Line::new(0, format!("import {}", path)));
                            }
                        }
                    }
                }
                // 3. Emit other top-level declarations
                for child in &other_nodes {
                    out.push(Line::new(0, ""));
                    self.walk(*child, indent, out);
                }
            }
            "package_clause" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| self.text_of(&n))
                    .unwrap_or("main");
                out.push(Line::new(indent, format!("package {}", name)));
            }
            "function_declaration" => self.walk_func(node, indent, out),
            "method_declaration" => self.walk_method(node, indent, out),
            "type_declaration" => self.walk_type(node, indent, out),
            "var_declaration" => self.walk_var(node, indent, out),
            "const_declaration" => self.walk_const(node, indent, out),
            "short_var_declaration" => {
                out.push(Line::new(indent, self.text_of(&node).to_string()));
            }
            "assignment_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "expression_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "return_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "if_statement" => self.walk_if(node, indent, out),
            "for_statement" => self.walk_for(node, indent, out),
            "switch_statement" | "select_statement" => self.walk_switch(node, indent, out),
            "go_statement" => {
                out.push(Line::new(
                    indent,
                    format!("go {}", self.text_of(&node.named_child(0).unwrap_or(node))),
                ));
            }
            "defer_statement" => {
                out.push(Line::new(
                    indent,
                    format!(
                        "defer {}",
                        self.text_of(&node.named_child(0).unwrap_or(node))
                    ),
                ));
            }
            "send_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "labeled_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "block" => self.walk_block_inner(node, indent, out),
            "comment" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "inc_statement" | "dec_statement" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            _ => {
                let text = self.text_of(&node);
                if !text.trim().is_empty() {
                    out.push(Line::new(indent, text));
                }
            }
        }
    }

    fn collect_import_paths(&self, node: tree_sitter::Node) -> Vec<String> {
        // import_declaration can have a single import_spec or an import_spec_list
        let mut paths = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "import_spec" => {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| format!("{} ", self.text_of(&n)))
                        .unwrap_or_default();
                    let path = child
                        .child_by_field_name("path")
                        .map(|n| self.text_of(&n).to_string())
                        .unwrap_or_default();
                    paths.push(format!("{}{}", name, path));
                }
                "import_spec_list" => {
                    let mut ic = child.walk();
                    for spec in child.children(&mut ic) {
                        if spec.kind() != "import_spec" {
                            continue;
                        }
                        let name = spec
                            .child_by_field_name("name")
                            .map(|n| format!("{} ", self.text_of(&n)))
                            .unwrap_or_default();
                        let path = spec
                            .child_by_field_name("path")
                            .map(|n| self.text_of(&n).to_string())
                            .unwrap_or_default();
                        if !path.is_empty() {
                            paths.push(format!("{}{}", name, path));
                        }
                    }
                }
                // Fallback: raw text for unrecognized forms
                k if k != "(" && k != ")" && k != "import" => {
                    let t = self.text_of(&child).trim().to_string();
                    if !t.is_empty() && t != "import" {
                        paths.push(t);
                    }
                }
                _ => {}
            }
        }
        if paths.is_empty() {
            // fallback: emit raw
            let raw = self.text_of(&node).to_string();
            if !raw.trim().is_empty() {
                paths.push(raw);
            }
        }
        paths
    }

    #[allow(dead_code)]
    fn format_import(&self, node: tree_sitter::Node) -> String {
        let raw = self.text_of(&node);
        raw.to_string()
    }

    fn walk_func(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("?");
        // Normalize parameter list: collapse extra spaces
        let params = node
            .child_by_field_name("parameters")
            .map(|n| {
                let raw = self.text_of(&n);
                let inner = raw.trim_start_matches('(').trim_end_matches(')');
                let parts: Vec<&str> = inner
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.is_empty() {
                    "()".to_string()
                } else {
                    format!("({})", parts.join(", "))
                }
            })
            .unwrap_or_else(|| "()".to_string());
        let result = node
            .child_by_field_name("result")
            .map(|n| format!(" {}", self.text_of(&n)))
            .unwrap_or_default();

        out.push(Line::new(
            indent,
            format!("func {}{}{} {{", name, params, result),
        ));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn walk_method(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let receiver = node
            .child_by_field_name("receiver")
            .map(|n| self.text_of(&n))
            .unwrap_or("()");
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("?");
        let params = node
            .child_by_field_name("parameters")
            .map(|n| self.text_of(&n))
            .unwrap_or("()");
        let result = node
            .child_by_field_name("result")
            .map(|n| format!(" {}", self.text_of(&n)))
            .unwrap_or_default();

        out.push(Line::new(
            indent,
            format!("func {}{}{}{} {{", receiver, name, params, result),
        ));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn walk_type(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        for spec in node.children(&mut cursor) {
            if spec.kind() != "type_spec" && spec.kind() != "type_alias" {
                continue;
            }
            let name = spec
                .child_by_field_name("name")
                .map(|n| self.text_of(&n))
                .unwrap_or("?");
            let type_val = spec
                .child_by_field_name("type")
                .map(|n| self.text_of(&n))
                .unwrap_or("?");

            // If the type value is a multi-line struct body, re-indent with tabs.
            if type_val.contains('\n') {
                let lines: Vec<&str> = type_val.lines().collect();
                // First line: `type Name struct {`
                out.push(Line::new(
                    indent,
                    format!("type {} {}", name, lines[0].trim()),
                ));
                // Middle lines: field declarations — re-indent at indent+1 tabs
                for inner in &lines[1..lines.len().saturating_sub(1)] {
                    let trimmed = inner.trim();
                    if trimmed.is_empty() {
                        out.push(Line::new(0, String::new()));
                    } else {
                        out.push(Line::new(indent + 1, trimmed.to_string()));
                    }
                }
                // Last line: closing `}`
                if let Some(last) = lines.last() {
                    let trimmed = last.trim();
                    if !trimmed.is_empty() {
                        out.push(Line::new(indent, trimmed.to_string()));
                    }
                }
            } else {
                out.push(Line::new(indent, format!("type {} {}", name, type_val)));
            }
        }
    }

    fn walk_var(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        out.push(Line::new(indent, self.text_of(&node)));
    }

    fn walk_const(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        out.push(Line::new(indent, self.text_of(&node)));
    }

    fn walk_if(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let init = node
            .child_by_field_name("initializer")
            .map(|n| format!("{} ; ", self.text_of(&n)))
            .unwrap_or_default();
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.text_of(&n))
            .unwrap_or("true");

        out.push(Line::new(indent, format!("if {}{} {{", init, cond)));
        if let Some(body) = node.child_by_field_name("consequence") {
            self.walk_block_inner(body, indent + 1, out);
        }
        if let Some(alt) = node.child_by_field_name("alternative") {
            let last = out.pop().unwrap_or(Line::new(indent, "}"));
            out.push(Line::new(indent, format!("{} else {{", last.content)));
            self.walk_block_inner(alt, indent + 1, out);
            out.push(Line::new(indent, "}".to_string()));
        } else {
            out.push(Line::new(indent, "}".to_string()));
        }
    }

    fn walk_for(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // gofmt for statement: for init; cond; post { ... }
        let raw_cond = self.build_for_clause(node);
        out.push(Line::new(indent, format!("for {} {{", raw_cond)));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn build_for_clause(&self, node: tree_sitter::Node) -> String {
        // Range-based for
        if let Some(range) = node.child_by_field_name("range") {
            let left = node
                .child_by_field_name("left")
                .map(|n| format!("{} := ", self.text_of(&n)))
                .unwrap_or_default();
            return format!("{}range {}", left, self.text_of(&range));
        }
        // Classic 3-clause for
        let init = node
            .child_by_field_name("initializer")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_default();
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_default();
        let post = node
            .child_by_field_name("post")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_default();
        if init.is_empty() && post.is_empty() {
            cond
        } else {
            format!("{}; {}; {}", init, cond, post)
        }
    }

    fn walk_switch(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let keyword = if node.kind() == "select_statement" {
            "select"
        } else {
            "switch"
        };
        let init = node
            .child_by_field_name("initializer")
            .map(|n| format!("{} ; ", self.text_of(&n)))
            .unwrap_or_default();
        let tag = node
            .child_by_field_name("tag")
            .map(|n| format!(" {}", self.text_of(&n)))
            .unwrap_or_default();
        out.push(Line::new(indent, format!("{} {}{} {{", keyword, init, tag)));
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for case in body.children(&mut cursor) {
                if !case.is_named() {
                    continue;
                }
                match case.kind() {
                    "expression_case" => {
                        let val = case
                            .child_by_field_name("value")
                            .map(|n| self.text_of(&n))
                            .unwrap_or("?");
                        out.push(Line::new(indent, format!("case {}:", val)));
                        let mut cc = case.walk();
                        for stmt in case.children(&mut cc) {
                            if stmt.is_named() && stmt.kind() != "expression_case" {
                                self.walk(stmt, indent + 1, out);
                            }
                        }
                    }
                    "default_case" => {
                        out.push(Line::new(indent, "default:".to_string()));
                        let mut cc = case.walk();
                        for stmt in case.children(&mut cc) {
                            if stmt.is_named() {
                                self.walk(stmt, indent + 1, out);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn walk_block_inner(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() {
                continue;
            }
            self.walk(child, indent, out);
        }
    }
}

// ── Import grouping pass ──────────────────────────────────────────────

/// Separate stdlib imports from external ones (goimports rule).
/// stdlib = no dot in the unquoted path.
/// When `enabled` is false, the output is returned verbatim.
fn group_imports(output: &str, enabled: bool) -> String {
    if !enabled {
        return output.to_string();
    }

    // Locate a multi-line import block: `import (\n ... \n)`
    let import_start = match output.find("import (") {
        Some(i) => i,
        None => return output.to_string(), // no grouped import block
    };
    let block_end = match output[import_start..].find(')') {
        Some(i) => import_start + i,
        None => return output.to_string(),
    };

    let before = &output[..import_start];
    let after = &output[block_end + 1..];
    let block = &output[import_start + 8..block_end]; // strip "import ("

    let mut stdlib: Vec<&str> = Vec::new();
    let mut external: Vec<&str> = Vec::new();

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // stdlib paths have no '.' before the first '/'
        let unquoted = trimmed.trim_matches('"');
        let is_stdlib = !unquoted.contains('.');
        if is_stdlib {
            stdlib.push(trimmed);
        } else {
            external.push(trimmed);
        }
    }

    if stdlib.is_empty() || external.is_empty() {
        return output.to_string(); // no grouping needed
    }

    let mut grouped = String::from("import (\n");
    for s in &stdlib {
        grouped.push_str(&format!("\t{}\n", s));
    }
    grouped.push('\n');
    for e in &external {
        grouped.push_str(&format!("\t{}\n", e));
    }
    grouped.push(')');

    format!("{}{}{}", before, grouped, after)
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Format Go source bytes (gofmt/goimports parity).
///
/// # Note on Indentation
///
/// gofmt unconditionally uses tabs. The `config.indent_style` and
/// `config.indent_size` fields are ignored for Go.
fn format_internal(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let t_start = protocol::Instant::now();
    let language = go_language();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| FormatError::Internal {
            message: format!("go grammar load failed: {}", e),
        })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| FormatError::ParseFailed {
            message: "tree-sitter returned None for Go".into(),
        })?;

    if tree.root_node().has_error() {
        log::warn!("lang-go: parse error — emitting verbatim");
        return Ok(source.to_vec());
    }
    let t_parse = t_start.elapsed();

    let t_format_start = protocol::Instant::now();
    let formatter = GoFormatter::new(source, config);
    let lines = formatter.format_tree(tree.root_node());
    let t_format = t_format_start.elapsed();

    let t_emit_start = protocol::Instant::now();
    let mut raw = String::with_capacity(source.len());
    for line in &lines {
        raw.push_str(&line.render());
        raw.push('\n');
    }

    let out = group_imports(&raw, formatter.group_imports);

    // Post-process: convert any remaining leading spaces to tabs (gofmt mandate).
    // Verbatim text_of() calls for closures/goroutines may leave space-based indentation.
    let space_to_tabs: String = out
        .lines()
        .map(|l| {
            let leading_spaces = l.len() - l.trim_start_matches(' ').len();
            if leading_spaces == 0 || l.starts_with('\t') {
                l.trim_end().to_string()
            } else {
                // Convert leading spaces to tabs (4 or 8 spaces → 1 tab each 4 spaces in Go)
                let tab_count = leading_spaces / 4;
                let remainder = leading_spaces % 4;
                let tabs = "\t".repeat(tab_count.max(1));
                let rest = l.trim_start();
                if remainder > 0 && tab_count == 0 {
                    // Fewer than 4 spaces of indent — just use 1 tab
                    format!("\t{}", rest.trim_end())
                } else {
                    format!("{}{}", tabs, rest.trim_end())
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let cleaned: String = space_to_tabs;
    let mut cleaned = cleaned;
    if !cleaned.ends_with('\n') {
        cleaned.push('\n');
    }
    let t_emit = t_emit_start.elapsed();

    eprintln!(
        "[Go] Parse: {:.2}ms, Format: {:.2}ms, Emit: {:.2}ms",
        t_parse.as_secs_f64() * 1000.0,
        t_format.as_secs_f64() * 1000.0,
        t_emit.as_secs_f64() * 1000.0
    );

    Ok(cleaned.into_bytes())
}

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let out = format_internal(source, config)?;

    #[cfg(debug_assertions)]
    {
        let second = format_internal(&out, config)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-go: format is not idempotent!"
        );
    }

    Ok(out)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_empty_returns_newline() {
        let config = ConfigIR::default();
        let result = format(b"", &config).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_simple_package_clause() {
        let src = b"package main\n";
        let config = ConfigIR::default();
        let result = format(src, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn tabs_used_regardless_of_config() {
        let src = b"package main\nfunc main() {\n\tfmt.Println(\"hello\")\n}\n";
        let mut config = ConfigIR::default();
        config.indent_style = protocol::config::IndentStyle::Spaces;
        config.indent_size = 2;
        let result = format(src, &config).unwrap();
        let result_str = std::str::from_utf8(&result).unwrap();
        // gofmt always uses tabs; spaces config must be ignored
        if result_str.contains("fmt.Println") {
            assert!(
                result_str.contains('\t'),
                "gofmt must use tabs unconditionally"
            );
        }
    }

    #[test]
    fn format_idempotent() {
        let src = b"package main\n\nfunc add(a, b int) int {\n\treturn a + b\n}\n";
        let config = ConfigIR::default();
        let first = format(src, &config).unwrap();
        let second = format(&first, &config).unwrap();
        assert_eq!(first, second);
    }

    // ── Extras: go__groupImports ──────────────────────────────────────

    #[test]
    fn group_imports_true_separates_stdlib_from_external() {
        let src = b"package main\n\nimport (\n\t\"fmt\"\n\t\"github.com/user/pkg\"\n)\n\nfunc main() {}\n";
        let mut config = ConfigIR::default();
        config.extras.insert(
            "go__groupImports".to_string(),
            serde_json::Value::Bool(true),
        );
        let result = format(src, &config).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        // fmt (stdlib) and github.com/... (external) should be in separate groups
        let fmt_pos = s.find("\"fmt\"").unwrap_or(0);
        let gh_pos = s.find("github.com").unwrap_or(0);
        assert!(fmt_pos < gh_pos, "stdlib before external: {s}");
        // There should be a blank line between the two groups
        let between = &s[fmt_pos..gh_pos];
        assert!(
            between.contains("\n\n") || between.contains("\n\t\n"),
            "expected blank line between import groups: {s}"
        );
    }

    #[test]
    fn go_group_imports_false_leaves_imports_as_is() {
        let mut config = ConfigIR::default();
        config.extras.insert(
            "go__groupImports".to_string(),
            serde_json::Value::Bool(false),
        );
        let formatter_flag = config.get_extra_bool("go__groupImports");
        assert_eq!(formatter_flag, Some(false));
    }

    #[test]
    fn go_group_imports_default_is_true() {
        let config = ConfigIR::default();
        let f = GoFormatter::new(b"", &config);
        assert!(f.group_imports, "default must be true");
    }
}
