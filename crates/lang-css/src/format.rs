//! CSS/SCSS/Less/HTML Formatting Logic — Prettier 3.x Parity
//!
//! Implements Prettier's CSS printer algorithm using Tree-sitter:
//!   1. Parse with tree-sitter-css (CSS/SCSS/Less) or tree-sitter-html (HTML).
//!   2. For HTML: invoke zone detection → dispatch `<script>` zones to lang-js
//!      and `<style>` zones back to lang-css.
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
        Line {
            indent,
            content: content.into(),
        }
    }

    fn render(&self, indent_size: usize) -> String {
        if self.content.is_empty() {
            String::new()
        } else {
            format!("{}{}", " ".repeat(self.indent * indent_size), self.content)
        }
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
        Self {
            source,
            config,
            dialect,
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
            "stylesheet" => {
                let mut first = true;
                let mut cursor = node.walk();
                // Collect named children so we can peek ahead
                let children: Vec<tree_sitter::Node> = node
                    .children(&mut cursor)
                    .filter(|n| n.is_named())
                    .collect();
                let mut i = 0;
                while i < children.len() {
                    let child = children[i];
                    // SCSS variable heuristic: tree-sitter-css parses `$var: val;` as
                    // ERROR("$") + declaration("var: val;") at the stylesheet level.
                    // Detect that pattern and reassemble as `$var: val;`.
                    if child.kind() == "ERROR" {
                        let err_text = self.text_of(&child).trim();
                        if err_text == "$" || err_text.ends_with('$') {
                            if let Some(&next) = children.get(i + 1) {
                                if next.kind() == "declaration" {
                                    let decl_raw = self.text_of(&next);
                                    let normalized = if let Some(pos) = decl_raw.find(':') {
                                        let prop =
                                            format!("{}{}", err_text, decl_raw[..pos].trim());
                                        let val =
                                            decl_raw[pos + 1..].trim().trim_end_matches(';').trim();
                                        format!("{}: {};", prop, val)
                                    } else {
                                        format!("{}{};", err_text, decl_raw.trim())
                                    };
                                    if !first {
                                        out.push(Line::new(0, ""));
                                    }
                                    first = false;
                                    out.push(Line::new(indent, normalized));
                                    i += 2;
                                    continue;
                                }
                            }
                        }
                    }

                    // Detect if previous sibling was a `/* prettier-ignore */` comment.
                    // Only suppress blank line in THAT case; regular comments still
                    // get a blank separator before the following rule (Prettier style).
                    let prev_comment = if i > 0 && children[i - 1].kind() == "comment" {
                        Some(self.text_of(&children[i - 1]))
                    } else {
                        None
                    };
                    let after_prettier_ignore = prev_comment
                        .as_deref()
                        .map(|c| c.contains("prettier-ignore"))
                        .unwrap_or(false);

                    if !first && !after_prettier_ignore {
                        out.push(Line::new(0, ""));
                    }

                    first = false;

                    if after_prettier_ignore {
                        // Emit verbatim: dedent by the minimum leading whitespace of
                        // non-first non-empty lines so relative structure is preserved
                        // but any source-level absolute indentation is stripped.
                        let raw = self.text_of(&child);
                        let all_lines: Vec<&str> = raw.lines().collect();
                        let min_indent = all_lines
                            .iter()
                            .skip(1)
                            .filter(|l| !l.trim().is_empty())
                            .map(|l| l.len() - l.trim_start().len())
                            .min()
                            .unwrap_or(0);
                        for (idx, line) in all_lines.iter().enumerate() {
                            let dedented = if idx == 0 || min_indent == 0 {
                                line.trim_end()
                            } else if line.trim().is_empty() {
                                ""
                            } else {
                                line.get(min_indent..)
                                    .unwrap_or(line.trim_start())
                                    .trim_end()
                            };
                            if dedented.is_empty() {
                                out.push(Line::new(0, ""));
                            } else {
                                out.push(Line::new(indent, dedented));
                            }
                        }
                    } else {
                        self.walk(child, indent, out);
                    }
                    i += 1;
                }
            }
            "rule_set" => self.walk_rule_set(node, indent, out),
            "at_rule" => self.walk_at_rule(node, indent, out),
            "media_statement" => self.walk_media(node, indent, out),
            "import_statement" => {
                out.push(Line::new(
                    indent,
                    format!("{};", self.text_of(&node).trim_end_matches(';')),
                ));
            }
            "comment" => {
                out.push(Line::new(indent, self.text_of(&node)));
            }
            // SCSS $variable declarations — must come BEFORE the plain "declaration" arm
            "declaration" if self.text_of(&node).trim_start().starts_with('$') => {
                let raw = self.text_of(&node);
                let (prop, val) = if let Some(pos) = raw.find(':') {
                    let p = raw[..pos].trim();
                    let v = raw[pos + 1..].trim().trim_end_matches(';').trim();
                    (p, v)
                } else {
                    (raw.trim(), "")
                };
                out.push(Line::new(indent, format!("{}: {};", prop, val)));
            }
            "declaration" => {
                out.push(Line::new(indent, self.format_declaration(node)));
            }
            "block" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_named() {
                        continue;
                    }
                    self.walk(child, indent, out);
                }
            }
            // SCSS/Less extensions — normalize spacing in declaration-like statements
            "mixin_statement"
            | "include_statement"
            | "extend_statement"
            | "each_statement"
            | "for_statement"
            | "while_statement"
            | "if_statement"
            | "else_statement"
            | "apply_statement"
            | "variable_declaration"
            | "use_statement"
            | "forward_statement"
            | "error_statement"
            | "warn_statement"
            | "debug_statement"
            | "function_statement"
            | "return_statement" => {
                let raw = self.text_of(&node).trim();
                // Normalize `$var: value ;` → `$var: value;`
                let normalized = if raw.contains(':') && !raw.contains('{') {
                    if let Some(pos) = raw.find(':') {
                        let prop = raw[..pos].trim();
                        let val = raw[pos + 1..].trim().trim_end_matches(';').trim();
                        format!("{}: {};", prop, val)
                    } else {
                        let s = raw.to_string();
                        if s.ends_with(';') || s.ends_with('}') {
                            s
                        } else {
                            format!("{};", s)
                        }
                    }
                } else {
                    let s = raw.to_string();
                    if s.ends_with(';') || s.ends_with('}') {
                        s
                    } else {
                        format!("{};", s)
                    }
                };
                out.push(Line::new(indent, normalized));
            }
            "ERROR" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.walk(child, indent, out);
                }
            }
            _ => {
                let raw = self.text_of(&node);
                let text = raw.trim();
                if !text.is_empty() {
                    // If it looks like a lone declaration (has `:`, no block braces), normalize it
                    let normalized =
                        if text.contains(':') && !text.contains('{') && !text.contains('}') {
                            if let Some(pos) = text.find(':') {
                                let prop = text[..pos].trim().to_lowercase();
                                let val = text[pos + 1..].trim().trim_end_matches(';').trim();
                                format!("{}: {};", prop, val)
                            } else {
                                text.to_string()
                            }
                        } else {
                            text.to_string()
                        };
                    out.push(Line::new(indent, normalized));
                }
            }
        }
    }

    fn walk_rule_set(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        let selectors = node
            .children(&mut cursor)
            .find(|n| n.kind() == "selectors" || n.kind() == "selector")
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

        let block = {
            let mut cursor = node.walk();
            let res = node.children(&mut cursor).find(|n| n.kind() == "block");
            res
        };
        if let Some(block) = block {
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
        let keyword = node
            .child_by_field_name("keyword")
            .map(|n| self.text_of(&n))
            .unwrap_or("at");
        let query = node
            .child_by_field_name("query")
            .map(|n| format!(" {}", self.text_of(&n)))
            .unwrap_or_default();

        let block = {
            let mut cursor = node.walk();
            let res = node.children(&mut cursor).find(|n| n.kind() == "block");
            res
        };
        if let Some(block) = block {
            out.push(Line::new(indent, format!("@{}{} {{", keyword, query)));
            self.walk_block_inner(block, indent + 1, out);
            out.push(Line::new(indent, "}".to_string()));
        } else {
            out.push(Line::new(indent, format!("@{}{};", keyword, query)));
        }
    }

    fn walk_media(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let query = node
            .child_by_field_name("query")
            .map(|n| self.text_of(&n))
            .unwrap_or("");
        out.push(Line::new(indent, format!("@media {} {{", query)));
        if let Some(block) = node.child_by_field_name("body") {
            let mut cursor = block.walk();
            for child in block.children(&mut cursor) {
                if !child.is_named() {
                    continue;
                }
                if !out.is_empty() {
                    out.push(Line::new(0, ""));
                }
                self.walk(child, indent + 1, out);
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
            match child.kind() {
                "declaration" => {
                    out.push(Line::new(indent, self.format_declaration(child)));
                }
                "rule_set" => {
                    // Nested rules (SCSS/Less)
                    if !out.is_empty() {
                        out.push(Line::new(0, ""));
                    }
                    self.walk_rule_set(child, indent, out);
                }
                "comment" => {
                    out.push(Line::new(indent, self.text_of(&child)));
                }
                _ => {
                    let raw = self.text_of(&child);
                    let text = raw.trim();
                    if !text.is_empty() {
                        // Normalize declaration-like text regardless of node kind
                        let normalized =
                            if text.contains(':') && !text.contains('{') && !text.contains('}') {
                                if let Some(pos) = text.find(':') {
                                    let prop = text[..pos].trim().to_lowercase();
                                    let val = text[pos + 1..].trim().trim_end_matches(';').trim();
                                    format!("{}: {};", prop, val)
                                } else {
                                    text.to_string()
                                }
                            } else {
                                text.to_string()
                            };
                        out.push(Line::new(indent, normalized));
                    }
                }
            }
        }
    }

    fn format_declaration(&self, node: tree_sitter::Node) -> String {
        let raw = self.text_of(&node).trim();
        // Split on first colon: normalises `prop : val` → `prop: val;`
        if let Some(colon_pos) = raw.find(':') {
            let prop = raw[..colon_pos].trim().to_lowercase();
            let after = raw[colon_pos + 1..].trim().trim_end_matches(';').trim();
            let (val, imp) = if after.trim_end().ends_with("!important") {
                let v = after[..after.rfind('!').unwrap_or(after.len())].trim();
                (v, " !important")
            } else {
                (after, "")
            };
            format!("{}: {}{};", prop, val, imp)
        } else {
            raw.to_string()
        }
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

    fn strip_indent(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let min_indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);

        let mut out = String::new();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                if i < lines.len() - 1 {
                    out.push('\n');
                }
            } else {
                out.push_str(line.get(min_indent..).unwrap_or(line.trim_start()));
                if i < lines.len() - 1 {
                    out.push('\n');
                }
            }
        }
        out
    }

    fn is_block_or_special(node: tree_sitter::Node, source: &[u8]) -> bool {
        match node.kind() {
            "comment" | "script_element" | "style_element" => true,
            "element" => {
                let mut cursor = node.walk();
                if let Some(start) = node
                    .children(&mut cursor)
                    .find(|c| c.kind() == "start_tag" || c.kind() == "self_closing_tag")
                {
                    let name = {
                        let mut s_cursor = start.walk();
                        let n = start
                            .children(&mut s_cursor)
                            .find(|c| c.kind() == "tag_name");
                        n.map(|tag_name| tag_name.utf8_text(source).unwrap_or("").to_lowercase())
                    };
                    if let Some(name) = name {
                        return matches!(
                            name.as_str(),
                            "div" | "p" | "ul" | "li" | "header" | "footer" | "main" | "section"
                        );
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn walk_html(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut add_blank = |out: &mut Vec<Line>| {
            if out.last().map_or(false, |l| !l.content.is_empty()) {
                out.push(Line::new(0, ""));
            }
        };

        match node.kind() {
            "document" => {
                let mut cursor = node.walk();
                let mut first = true;
                for child in node.children(&mut cursor) {
                    if !first && Self::is_block_or_special(child, self.source) {
                        add_blank(out);
                    }
                    self.walk_html(child, indent, out);
                    first = false;
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
        const VOID: &[&str] = &[
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
            "source", "track", "wbr",
        ];

        let mut start_tag = None;
        let mut has_end_tag = false;
        {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "start_tag" | "self_closing_tag" if start_tag.is_none() => {
                        start_tag = Some(child)
                    }
                    "end_tag" => has_end_tag = true,
                    _ => {}
                }
            }
        }

        let start = match start_tag {
            Some(s) => s,
            None => return,
        };
        let tag_name = {
            let mut cursor = start.walk();
            let n = start
                .children(&mut cursor)
                .find(|ch| ch.kind() == "tag_name");
            n.map(|n| self.text_of(&n).to_lowercase())
                .unwrap_or_default()
        };

        if tag_name.is_empty() {
            return;
        }

        let attrs = self.format_attrs(start);
        let is_void = VOID.contains(&tag_name.as_str());
        let is_self_closing = start.kind() == "self_closing_tag";

        let mut inline_tag = format!("<{}", tag_name);
        for attr in &attrs {
            inline_tag.push_str(" ");
            inline_tag.push_str(attr);
        }
        inline_tag.push_str(if is_self_closing { " />" } else { ">" });

        let wrap_attrs = inline_tag.len() + indent * self.config.indent_size as usize
            > self.config.print_width as usize;

        let mut start_lines = Vec::new();
        if wrap_attrs && !attrs.is_empty() {
            start_lines.push(Line::new(indent, format!("<{}", tag_name)));
            for attr in attrs {
                start_lines.push(Line::new(indent + 1, attr));
            }
            start_lines.push(Line::new(
                indent,
                if is_self_closing { "/>" } else { ">" }.to_string(),
            ));
        } else {
            start_lines.push(Line::new(indent, inline_tag));
        }

        if is_void || is_self_closing {
            for l in start_lines {
                out.push(l);
            }
            return;
        }

        let children: Vec<tree_sitter::Node> = {
            let mut cursor = node.walk();
            node.children(&mut cursor)
                .filter(|ch| !matches!(ch.kind(), "start_tag" | "end_tag" | "self_closing_tag"))
                .collect()
        };

        let element_children: Vec<&tree_sitter::Node> = children
            .iter()
            .filter(|ch| matches!(ch.kind(), "element" | "script_element" | "style_element"))
            .collect();
        let text_children: Vec<&str> = children
            .iter()
            .filter(|ch| ch.kind() == "text")
            .map(|ch| self.text_of(ch).trim())
            .filter(|s| !s.is_empty())
            .collect();

        if element_children.is_empty() && text_children.len() == 1 {
            // Text only, but if we wrapped attrs, don't inline the text.
            if wrap_attrs {
                for l in start_lines {
                    out.push(l);
                }
                out.push(Line::new(indent + 1, text_children[0]));
                out.push(Line::new(indent, format!("</{}>", tag_name)));
            } else {
                let single_line = format!(
                    "{}{}</{}>",
                    start_lines[0].content, text_children[0], tag_name
                );
                if single_line.len() + indent * self.config.indent_size as usize
                    <= self.config.print_width as usize
                {
                    out.push(Line::new(indent, single_line));
                } else {
                    for l in start_lines {
                        out.push(l);
                    }
                    out.push(Line::new(indent + 1, text_children[0]));
                    out.push(Line::new(indent, format!("</{}>", tag_name)));
                }
            }
            return;
        }

        for l in start_lines {
            out.push(l);
        }
        let mut first = true;
        for child in &children {
            if !first && Self::is_block_or_special(*child, self.source) {
                if out.last().map_or(false, |l| !l.content.is_empty()) {
                    out.push(Line::new(0, ""));
                }
            }
            self.walk_html(*child, indent + 1, out);
            first = false;
        }
        if has_end_tag {
            out.push(Line::new(indent, format!("</{}>", tag_name)));
        }
    }

    /// Format a `<script>` element: extract JS content, format it via lang_js, re-wrap.
    fn walk_script(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Find raw_text child (JS content)
        let (open_tag, js_content, close_tag) = self.extract_zone_content(node, "raw_text");

        out.push(Line::new(indent, open_tag));

        if !js_content.trim().is_empty() {
            let config = self.config;
            let stripped_js = Self::strip_indent(&js_content);
            let formatted_js = lang_js::format::format(stripped_js.as_bytes(), config)
                .unwrap_or_else(|_| stripped_js.as_bytes().to_vec());
            let formatted_str = String::from_utf8_lossy(&formatted_js);
            for line in formatted_str.trim_end().lines() {
                let t = line.trim_end();
                if t.is_empty() {
                    out.push(Line::new(0, ""));
                } else {
                    out.push(Line::new(indent + 1, t));
                }
            }
        }

        out.push(Line::new(indent, close_tag));
    }

    /// Format a `<style>` element: extract CSS content, format it via lang_css, re-wrap.
    fn walk_style(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let (open_tag, css_content, close_tag) = self.extract_zone_content(node, "raw_text");

        out.push(Line::new(indent, open_tag));

        if !css_content.trim().is_empty() {
            let config = self.config;
            let stripped_css = Self::strip_indent(&css_content);
            let formatted_css =
                crate::format::format(stripped_css.as_bytes(), config, CssDialect::Css)
                    .unwrap_or_else(|_| stripped_css.as_bytes().to_vec());
            let formatted_str = String::from_utf8_lossy(&formatted_css);
            for line in formatted_str.trim_end().lines() {
                let t = line.trim_end();
                if t.is_empty() {
                    out.push(Line::new(0, ""));
                } else {
                    out.push(Line::new(indent + 1, t));
                }
            }
        }

        out.push(Line::new(indent, close_tag));
    }

    /// Extract (open_tag_text, inner_content, close_tag_text) from a script/style element.
    fn extract_zone_content(
        &self,
        node: tree_sitter::Node,
        content_kind: &str,
    ) -> (String, String, String) {
        let mut open_tag = String::new();
        let mut content = String::new();
        let mut close_tag = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "start_tag" => {
                    open_tag = self.text_of(&child).trim().to_string();
                }
                k if k == content_kind => {
                    content = self.text_of(&child).to_string();
                }
                "end_tag" => {
                    close_tag = self.text_of(&child).trim().to_string();
                }
                _ => {}
            }
        }

        if open_tag.is_empty() {
            // Fallback: reconstruct from raw text
            let raw = self.text_of(&node);
            if let (Some(s), Some(e)) = (raw.find('>'), raw.rfind('<')) {
                open_tag = raw[..=s].trim().to_string();
                content = raw[s + 1..e].to_string();
                close_tag = raw[e..].trim().to_string();
            } else {
                return (String::new(), raw.to_string(), String::new());
            }
        }

        (open_tag, content, close_tag)
    }

    fn format_attrs(&self, tag: tree_sitter::Node) -> Vec<String> {
        let mut attrs = Vec::new();
        let mut cursor = tag.walk();
        for child in tag.children(&mut cursor) {
            match child.kind() {
                "attribute" => {
                    let mut name = String::new();
                    let mut value = String::new();
                    let mut child_cursor = child.walk();
                    for attr_child in child.children(&mut child_cursor) {
                        match attr_child.kind() {
                            "attribute_name" => {
                                name = self.text_of(&attr_child).to_lowercase();
                            }
                            "quoted_attribute_value" | "attribute_value" => {
                                let raw = self.text_of(&attr_child);
                                let inner = raw.trim_matches('"').trim_matches('\'');
                                value = format!("=\"{}\"", inner);
                            }
                            _ => {}
                        }
                    }
                    if !name.is_empty() {
                        attrs.push(format!("{}{}", name, value));
                    }
                }
                "attribute_name" => {
                    attrs.push(self.text_of(&child).to_lowercase());
                }
                _ => {}
            }
        }
        attrs
    }
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Format CSS/SCSS/Less/HTML source bytes (Prettier 3.x parity).
fn format_internal(
    source: &[u8],
    config: &ConfigIR,
    dialect: CssDialect,
) -> Result<Vec<u8>, FormatError> {
    let t_start = std::time::Instant::now();
    let indent_size = config.indent_size as usize;

    let out = match dialect {
        CssDialect::Html => {
            let language = html_language();
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&language)
                .map_err(|e| FormatError::Internal {
                    message: format!("html grammar load: {}", e),
                })?;
            let tree = parser
                .parse(source, None)
                .ok_or_else(|| FormatError::ParseFailed {
                    message: "tree-sitter None for HTML".into(),
                })?;
            let t_parse = t_start.elapsed();

            let t_format_start = std::time::Instant::now();
            let fmt = HtmlFormatter::new(source, config);
            let lines = fmt.format(tree.root_node());
            let t_format = t_format_start.elapsed();

            let t_emit_start = std::time::Instant::now();
            let rendered = lines
                .iter()
                .map(|l| l.render(indent_size))
                .collect::<Vec<_>>()
                .join("\n");
            let t_emit = t_emit_start.elapsed();

            eprintln!(
                "[HTML] Parse: {:.2}ms, Format: {:.2}ms, Emit: {:.2}ms",
                t_parse.as_secs_f64() * 1000.0,
                t_format.as_secs_f64() * 1000.0,
                t_emit.as_secs_f64() * 1000.0
            );
            rendered
        }
        _ => {
            let language = css_language();
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&language)
                .map_err(|e| FormatError::Internal {
                    message: format!("css grammar load: {}", e),
                })?;
            let tree = parser
                .parse(source, None)
                .ok_or_else(|| FormatError::ParseFailed {
                    message: "tree-sitter None for CSS".into(),
                })?;
            let t_parse = t_start.elapsed();

            let t_format_start = std::time::Instant::now();
            let fmt = CssFormatter::new(source, config, dialect);
            let lines = fmt.format_tree(tree.root_node());
            let t_format = t_format_start.elapsed();

            let t_emit_start = std::time::Instant::now();
            let rendered = lines
                .iter()
                .map(|l| l.render(indent_size))
                .collect::<Vec<_>>()
                .join("\n");
            let t_emit = t_emit_start.elapsed();

            let lang_name = match dialect {
                CssDialect::Scss => "SCSS",
                CssDialect::Less => "LESS",
                _ => "CSS",
            };
            eprintln!(
                "[{}] Parse: {:.2}ms, Format: {:.2}ms, Emit: {:.2}ms",
                lang_name,
                t_parse.as_secs_f64() * 1000.0,
                t_format.as_secs_f64() * 1000.0,
                t_emit.as_secs_f64() * 1000.0
            );
            rendered
        }
    };

    let mut out = out;
    if !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out.into_bytes())
}

pub fn format(
    source: &[u8],
    config: &ConfigIR,
    dialect: CssDialect,
) -> Result<Vec<u8>, FormatError> {
    let out = format_internal(source, config, dialect)?;

    #[cfg(debug_assertions)]
    {
        let second = format_internal(&out, config, dialect)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-css: format is not idempotent!"
        );
    }

    Ok(out)
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
        assert!(
            s.contains("color: red;"),
            "missing normalized declaration: {}",
            s
        );
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
