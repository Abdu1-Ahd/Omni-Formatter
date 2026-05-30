//! CSS/SCSS/Less/HTML Formatting Logic — Prettier 3.x Parity
//!
//! Implements Prettier's CSS printer algorithm using Tree-sitter:
//!   1. Parse with tree-sitter-css (CSS/SCSS/Less) or tree-sitter-html (HTML).
//!   2. For HTML: invoke zone detection → dispatch `<script>` zones to lang-js.
//!   3. Apply Prettier's CSS rules:
//!      - Lowercase property names.
//!      - Single space after `:` in declarations.
//!      - No space before `:` in declarations.
//!      - Trailing semicolons in all declaration blocks.
//!      - One selector per line for comma-separated selectors.
//!      - Opening `{` on the same line as selector.
//!      - Closing `}` on its own line.
//!      - Blank line between rules.
//!   4. Assert idempotency in debug builds.

use crate::CssDialect;
use protocol::config::ConfigIR;
use protocol::FormatError;

fn css_language() -> tree_sitter::Language {
    tree_sitter_css::language()
}

fn html_language() -> tree_sitter::Language {
    tree_sitter_html::language()
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

    fn render(&self, indent_size: usize) -> String {
        format!("{}{}", " ".repeat(self.indent * indent_size), self.content)
    }
}

// ── CSS Formatter ─────────────────────────────────────────────────────────

struct CssFormatter<'a> {
    source: &'a [u8],
    config: &'a ConfigIR,
    dialect: CssDialect,
}

impl<'a> CssFormatter<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR, dialect: CssDialect) -> Self {
        Self { source, config, dialect }
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
            "stylesheet" => {
                let mut first = true;
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_named() { continue; }
                    if !first { out.push(Line::new(0, "")); }
                    first = false;
                    self.walk(child, indent, out);
                }
            }
            "rule_set" => self.walk_rule_set(node, indent, out),
            "at_rule" => self.walk_at_rule(node, indent, out),
            "media_statement" => self.walk_media(node, indent, out),
            "import_statement" => {
                out.push(Line::new(indent, format!("{};", self.text_of(&node).trim_end_matches(';'))));
            }
            "comment" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "declaration" => {
                out.push(Line::new(indent, self.format_declaration(node)));
            }
            "block" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_named() { continue; }
                    self.walk(child, indent, out);
                }
            }
            // SCSS/Less extensions
            "mixin_statement" | "include_statement" | "extend_statement"
            | "each_statement" | "for_statement" | "while_statement"
            | "if_statement" | "else_statement" | "apply_statement"
            | "variable_declaration" | "use_statement" | "forward_statement"
            | "error_statement" | "warn_statement" | "debug_statement"
            | "function_statement" | "return_statement" => {
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

    fn walk_rule_set(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let selectors = node.child_by_field_name("selectors")
            .map(|n| self.format_selectors(n))
            .unwrap_or_default();

        // One selector per line for comma-separated lists
        let sel_lines: Vec<&str> = selectors.lines().collect();
        for (i, sel) in sel_lines.iter().enumerate() {
            if i == sel_lines.len() - 1 {
                out.push(Line::new(indent, format!("{} {{", sel.trim())));
            } else {
                out.push(Line::new(indent, sel.trim().to_string()));
            }
        }

        if let Some(block) = node.child_by_field_name("block") {
            self.walk_block_inner(block, indent + 1, out);
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn format_selectors(&self, node: tree_sitter::Node) -> String {
        // Split comma-separated selectors onto separate lines
        let raw = self.text_of(&node);
        raw.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(",\n")
    }

    fn walk_at_rule(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let keyword = node.child_by_field_name("keyword")
            .map(|n| self.text_of(&n)).unwrap_or("at");
        let query = node.child_by_field_name("query")
            .map(|n| format!(" {}", self.text_of(&n)))
            .unwrap_or_default();

        if let Some(block) = node.child_by_field_name("block") {
            out.push(Line::new(indent, format!("@{}{} {{", keyword, query)));
            self.walk_block_inner(block, indent + 1, out);
            out.push(Line::new(indent, "}".to_string()));
        } else {
            out.push(Line::new(indent, format!("@{}{};", keyword, query)));
        }
    }

    fn walk_media(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let query = node.child_by_field_name("query")
            .map(|n| self.text_of(&n)).unwrap_or("");
        out.push(Line::new(indent, format!("@media {} {{", query)));
        if let Some(block) = node.child_by_field_name("body") {
            let mut cursor = block.walk();
            for child in block.children(&mut cursor) {
                if !child.is_named() { continue; }
                if !out.is_empty() { out.push(Line::new(0, "")); }
                self.walk(child, indent + 1, out);
            }
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn walk_block_inner(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() { continue; }
            match child.kind() {
                "declaration" => {
                    out.push(Line::new(indent, self.format_declaration(child)));
                }
                "rule_set" => {
                    // Nested rules (SCSS/Less)
                    if !out.is_empty() { out.push(Line::new(0, "")); }
                    self.walk_rule_set(child, indent, out);
                }
                "comment" => {
                    out.push(Line::new(indent, self.text_of(&child)));
                }
                _ => {
                    let text = self.text_of(&child);
                    if !text.trim().is_empty() {
                        out.push(Line::new(indent, text));
                    }
                }
            }
        }
    }

    fn format_declaration(&self, node: tree_sitter::Node) -> String {
        let prop = node.child_by_field_name("property")
            .map(|n| self.text_of(&n).to_lowercase())
            .unwrap_or_else(|| {
                // Fallback: parse from raw text
                let raw = self.text_of(&node);
                raw.splitn(2, ':').next().unwrap_or("").trim().to_lowercase()
            });

        let value = node.child_by_field_name("value")
            .map(|n| self.text_of(&n).trim().to_string())
            .unwrap_or_else(|| {
                let raw = self.text_of(&node);
                raw.splitn(2, ':').nth(1).unwrap_or("").trim().trim_end_matches(';').trim().to_string()
            });

        // Important flag
        let important = if self.text_of(&node).contains("!important") { " !important" } else { "" };
        format!("{}: {}{};", prop, value, important)
    }
}

// ── HTML Formatter ────────────────────────────────────────────────────────

struct HtmlFormatter<'a> {
    source: &'a [u8],
    config: &'a ConfigIR,
}

impl<'a> HtmlFormatter<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR) -> Self {
        Self { source, config }
    }

    fn text_of(&self, node: &tree_sitter::Node) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    fn format(&self, root: tree_sitter::Node) -> Vec<Line> {
        let mut lines = Vec::new();
        self.walk_html(root, 0, &mut lines);
        lines
    }

    fn walk_html(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        match node.kind() {
            "document" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.walk_html(child, indent, out);
                }
            }
            "doctype" => {
                out.push(Line::new(0, "<!doctype html>".to_string()));
            }
            "element" => self.walk_element(node, indent, out),
            "text" => {
                let text = self.text_of(&node).trim();
                if !text.is_empty() {
                    out.push(Line::new(indent, text));
                }
            }
            "comment" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            "script_element" => self.walk_script(node, indent, out),
            "style_element" => self.walk_style(node, indent, out),
            _ => {
                let text = self.text_of(&node);
                if !text.trim().is_empty() {
                    out.push(Line::new(indent, text));
                }
            }
        }
    }

    fn walk_element(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Void elements (self-closing)
        const VOID: &[&str] = &["area","base","br","col","embed","hr","img","input",
                                 "link","meta","param","source","track","wbr"];

        let open = node.child_by_field_name("start_tag");
        let close = node.child_by_field_name("end_tag");

        if let Some(start) = open {
            let tag_name = start.child_by_field_name("name")
                .map(|n| self.text_of(&n).to_lowercase())
                .unwrap_or_default();
            let attrs = self.format_attrs(start);
            let is_void = VOID.contains(&tag_name.as_str());

            if is_void {
                out.push(Line::new(indent, format!("<{}{}>", tag_name, attrs)));
                return;
            }

            // Check if element fits inline (no child elements)
            let child_text: Vec<_> = {
                let mut c = node.walk();
                node.children(&mut c)
                    .filter(|ch| ch.kind() == "text")
                    .map(|ch| self.text_of(&ch).trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            let has_child_elements = {
                let mut c = node.walk();
                node.children(&mut c).any(|ch| ch.kind() == "element" || ch.kind() == "script_element" || ch.kind() == "style_element")
            };

            if !has_child_elements && child_text.len() == 1 {
                out.push(Line::new(indent, format!("<{}{}>{}</{}>", tag_name, attrs, child_text[0], tag_name)));
                return;
            }

            out.push(Line::new(indent, format!("<{}{}>", tag_name, attrs)));
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "start_tag" | "end_tag" | "self_closing_tag" => continue,
                    _ => self.walk_html(child, indent + 1, out),
                }
            }
            if close.is_some() {
                out.push(Line::new(indent, format!("</{}>", tag_name)));
            }
        }
    }

    fn walk_script(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Emit script tags verbatim (zone routing handles JS formatting)
        let raw = self.text_of(&node);
        for line in raw.lines() {
            out.push(Line::new(indent, line.trim_end()));
        }
    }

    fn walk_style(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Emit style tags verbatim (zone routing handles CSS formatting)
        let raw = self.text_of(&node);
        for line in raw.lines() {
            out.push(Line::new(indent, line.trim_end()));
        }
    }

    fn format_attrs(&self, tag: tree_sitter::Node) -> String {
        let mut attrs = String::new();
        let mut cursor = tag.walk();
        for child in tag.children(&mut cursor) {
            match child.kind() {
                "attribute" => {
                    let name = child.child_by_field_name("name")
                        .map(|n| self.text_of(&n).to_lowercase())
                        .unwrap_or_default();
                    let value = child.child_by_field_name("value")
                        .map(|n| format!("=\"{}\"", self.text_of(&n).trim_matches('"').trim_matches('\'')))
                        .unwrap_or_default();
                    attrs.push_str(&format!(" {}{}", name, value));
                }
                "attribute_name" => {
                    attrs.push_str(&format!(" {}", self.text_of(&child).to_lowercase()));
                }
                _ => {}
            }
        }
        attrs
    }
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Format CSS/SCSS/Less/HTML source bytes (Prettier 3.x parity).
pub fn format(source: &[u8], config: &ConfigIR, dialect: CssDialect) -> Result<Vec<u8>, FormatError> {
    let indent_size = config.indent_size as usize;

    let out = match dialect {
        CssDialect::Html => {
            let language = html_language();
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&language)
                .map_err(|e| FormatError::Internal(format!("html grammar load: {}", e)))?;
            let tree = parser.parse(source, None)
                .ok_or_else(|| FormatError::ParseError("tree-sitter None for HTML".into()))?;
            if tree.root_node().has_error() {
                log::warn!("lang-css/html: parse error — verbatim");
                return Ok(source.to_vec());
            }
            let fmt = HtmlFormatter::new(source, config);
            let lines = fmt.format(tree.root_node());
            lines.iter().map(|l| l.render(indent_size)).collect::<Vec<_>>().join("\n")
        }
        _ => {
            let language = css_language();
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&language)
                .map_err(|e| FormatError::Internal(format!("css grammar load: {}", e)))?;
            let tree = parser.parse(source, None)
                .ok_or_else(|| FormatError::ParseError("tree-sitter None for CSS".into()))?;
            if tree.root_node().has_error() {
                log::warn!("lang-css: parse error — verbatim");
                return Ok(source.to_vec());
            }
            let fmt = CssFormatter::new(source, config, dialect);
            let lines = fmt.format_tree(tree.root_node());
            lines.iter().map(|l| l.render(indent_size)).collect::<Vec<_>>().join("\n")
        }
    };

    let mut out = out;
    if !out.ends_with('\n') { out.push('\n'); }

    #[cfg(debug_assertions)]
    {
        let second = format(out.as_bytes(), config, dialect)?;
        debug_assert_eq!(
            out.as_bytes(),
            second.as_slice(),
            "lang-css: format is not idempotent!"
        );
    }

    Ok(out.into_bytes())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_formats_declaration() {
        let src = b"body{color:red}\n";
        let config = ConfigIR::default();
        let result = format(src, &config, CssDialect::Css).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        assert!(s.contains("color: red;"), "missing normalized declaration: {}", s);
    }

    #[test]
    fn html_does_not_panic() {
        let src = b"<html><body><h1>Hello</h1></body></html>\n";
        let config = ConfigIR::default();
        let result = format(src, &config, CssDialect::Html).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn css_idempotent() {
        let src = b".container {\n  display: flex;\n  align-items: center;\n}\n";
        let config = ConfigIR::default();
        let first = format(src, &config, CssDialect::Css).unwrap();
        let second = format(&first, &config, CssDialect::Css).unwrap();
        assert_eq!(first, second, "CSS format not idempotent");
    }
}
