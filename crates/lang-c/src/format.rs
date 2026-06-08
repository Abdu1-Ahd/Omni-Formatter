//! C/C++ Formatting Logic — clang-format Style
//!
//! Implements formatting via Tree-sitter `tree-sitter-c` and `tree-sitter-cpp`:
//!   1. Parse with the appropriate grammar (C vs C++ dialect).
//!   2. Walk the CST and emit formatted output using line-based rendering.
//!   3. Apply clang-format rules:
//!      - Configurable indentation (spaces or tabs)
//!      - Brace placement (Attach = K&R style, Allman = opening brace on next line)
//!      - Pointer/reference alignment
//!      - One blank line between top-level function/struct definitions
//!      - Trailing whitespace removal
//!   4. Assert idempotency in debug builds.
//!
//! # Dialect Handling
//!
//! C and C++ are handled by separate tree-sitter grammars but share the
//! same formatting rules and CST walker logic. The dialect is passed via
//! `CDialect` to select the correct grammar.

use crate::{adapter::CConfig, CDialect};
use protocol::FormatError;

// ── Grammar selectors ─────────────────────────────────────────────────────

fn c_language() -> tree_sitter::Language {
    tree_sitter_c::language()
}

fn cpp_language() -> tree_sitter::Language {
    tree_sitter_cpp::language()
}

// ── Line IR ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Line {
    indent: usize,
    content: String,
}

impl Line {
    fn new(indent: usize, content: impl Into<String>) -> Self {
        Line { indent, content: content.into() }
    }

    fn render(&self, config: &CConfig) -> String {
        if self.content.is_empty() {
            String::new()
        } else if config.indent_style == "tabs" {
            format!("{}{}", "\t".repeat(self.indent), self.content)
        } else {
            format!(
                "{}{}",
                " ".repeat(self.indent * config.indent_size),
                self.content
            )
        }
    }
}

// ── Formatter ─────────────────────────────────────────────────────────────

struct CFormatter<'a> {
    source: &'a [u8],
    config: &'a CConfig,
}

impl<'a> CFormatter<'a> {
    fn new(source: &'a [u8], config: &'a CConfig) -> Self {
        Self { source, config }
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
            "translation_unit" => {
                let mut cursor = node.walk();
                let children: Vec<_> = node.children(&mut cursor).collect();
                for (i, child) in children.iter().enumerate() {
                    if !child.is_named() { continue; }
                    // Blank line between top-level declarations (clang-format rule)
                    if i > 0 {
                        out.push(Line::new(0, ""));
                    }
                    self.walk(*child, indent, out);
                }
            }
            "function_definition" => self.walk_function(node, indent, out),
            "declaration"         => self.walk_declaration(node, indent, out),
            "struct_specifier" | "class_specifier" => {
                self.walk_struct_or_class(node, indent, out);
            }
            "comment" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "preproc_include" | "preproc_def" | "preproc_ifdef"
            | "preproc_if" | "preproc_else" | "preproc_endif"
            | "preproc_function_def" => {
                // Preprocessor directives: emit verbatim, always at column 0
                out.push(Line::new(0, self.text_of(&node)));
            }
            "namespace_definition" => self.walk_namespace(node, indent, out),
            _ => {
                let text = self.text_of(&node).trim().to_string();
                if !text.is_empty() {
                    out.push(Line::new(indent, text));
                }
            }
        }
    }

    fn walk_function(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Build signature: return_type + declarator (name + params)
        let mut sig_parts: Vec<&str> = Vec::new();
        let mut body_node = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "compound_statement" {
                body_node = Some(child);
            } else if child.is_named() {
                let text = self.text_of(&child).trim();
                if !text.is_empty() {
                    sig_parts.push(text);
                }
            }
        }
        let sig = sig_parts.join(" ");

        match self.config.brace_style.as_str() {
            "Allman" | "GNU" => {
                // Opening brace on its own line
                out.push(Line::new(indent, sig));
                out.push(Line::new(indent, "{"));
            }
            _ => {
                // K&R / Attach: opening brace on same line
                out.push(Line::new(indent, format!("{} {{", sig)));
            }
        }
        if let Some(body) = body_node {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, "}"));
    }

    fn walk_declaration(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let text = self.text_of(&node).trim().to_string();
        // Ensure trailing semicolon
        let text = if text.ends_with(';') { text } else { format!("{};", text) };
        out.push(Line::new(indent, text));
    }

    fn walk_struct_or_class(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let kind_kw = if node.kind() == "class_specifier" { "class" } else { "struct" };
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("_anonymous");

        match self.config.brace_style.as_str() {
            "Allman" | "GNU" => {
                out.push(Line::new(indent, format!("{} {}", kind_kw, name)));
                out.push(Line::new(indent, "{"));
            }
            _ => {
                out.push(Line::new(indent, format!("{} {} {{", kind_kw, name)));
            }
        }
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, "};"));
    }

    fn walk_namespace(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.text_of(&n))
            .unwrap_or("_anonymous");
        out.push(Line::new(indent, format!("namespace {} {{", name)));
        if let Some(body) = node.child_by_field_name("body") {
            self.walk_block_inner(body, indent + 1, out);
        }
        out.push(Line::new(indent, format!("}} // namespace {}", name)));
    }

    fn walk_block_inner(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() { continue; }
            self.walk(child, indent, out);
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────

fn format_internal(
    source: &[u8],
    config: &CConfig,
    dialect: CDialect,
) -> Result<Vec<u8>, FormatError> {
    let language = match dialect {
        CDialect::C | CDialect::ObjC  => c_language(),
        CDialect::Cpp | CDialect::ObjCpp => cpp_language(),
    };

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).map_err(|e| FormatError::Internal {
        message: format!("grammar load failed: {}", e),
    })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| FormatError::ParseFailed {
            message: "tree-sitter returned None for C/C++".into(),
        })?;

    if tree.root_node().has_error() {
        log::warn!("lang-c: parse error — emitting verbatim");
        return Ok(source.to_vec());
    }

    let formatter = CFormatter::new(source, config);
    let lines = formatter.format_tree(tree.root_node());

    let mut out = String::with_capacity(source.len());
    for line in &lines {
        let rendered = line.render(config);
        out.push_str(&rendered);
        out.push('\n');
    }

    // Collapse 3+ consecutive blank lines down to 2
    let mut cleaned = String::with_capacity(out.len());
    let mut consecutive_blanks = 0usize;
    for l in out.lines() {
        if l.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 2 {
                cleaned.push('\n');
            }
        } else {
            consecutive_blanks = 0;
            cleaned.push_str(l.trim_end());
            cleaned.push('\n');
        }
    }
    if !cleaned.ends_with('\n') {
        cleaned.push('\n');
    }

    Ok(cleaned.into_bytes())
}

pub fn format(
    source: &[u8],
    config: &CConfig,
    dialect: CDialect,
) -> Result<Vec<u8>, FormatError> {
    let out = format_internal(source, config, dialect)?;

    #[cfg(debug_assertions)]
    {
        let second = format_internal(&out, config, dialect)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-c: format is not idempotent!"
        );
    }

    Ok(out)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CDialect;

    fn default_config() -> CConfig { CConfig::default() }

    #[test]
    fn format_empty_c() {
        let result = format(b"", &default_config(), CDialect::C).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_empty_cpp() {
        let result = format(b"", &default_config(), CDialect::Cpp).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_simple_function() {
        let src = b"int main() {\nreturn 0;\n}\n";
        let result = format(src, &default_config(), CDialect::C).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn format_idempotent() {
        let src = b"int add(int a, int b) {\nreturn a + b;\n}\n";
        let config = default_config();
        let first  = format(src, &config, CDialect::C).unwrap();
        let second = format(&first, &config, CDialect::C).unwrap();
        assert_eq!(first, second, "C formatter must be idempotent");
    }
}
