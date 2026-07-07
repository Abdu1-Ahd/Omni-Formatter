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
    _config: &'a ConfigIR,
    _dialect: CssDialect,
}

impl<'a> CssFormatter<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR, dialect: CssDialect) -> Self {
        Self {
            source,
            _config: config,
            _dialect: dialect,
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
                        } else if err_text.starts_with('@')
                            && err_text.contains(':')
                            && !err_text.contains('{')
                        {
                            if let Some(pos) = err_text.find(':') {
                                let prop = err_text[..pos].trim();
                                let val = err_text[pos + 1..].trim().trim_end_matches(';').trim();
                                if !first {
                                    out.push(Line::new(0, ""));
                                }
                                first = false;
                                out.push(Line::new(indent, format!("{}: {};", prop, val)));
                                i += 1;
                                continue;
                            }
                        }
                    }

                    // LESS variable heuristic: tree-sitter-css sometimes parses
                    // `@base: #f938ab;` as at_rule("@base") + declaration("base: #f938ab;")
                    // or at_rule("@base") with the value as a separate sibling.
                    // Detect that pattern and reassemble as `@base: value;`.
                    if child.kind() == "at_rule" {
                        let at_raw = self.text_of(&child).trim().to_string();
                        // A LESS variable at-rule: starts with @, no block { ... }
                        let has_block_child = {
                            let mut c = child.walk();
                            let mut found = false;
                            for n in child.children(&mut c) {
                                if n.kind() == "block" {
                                    found = true;
                                    break;
                                }
                            }
                            found
                        };
                        if !has_block_child && at_raw.starts_with('@') && !at_raw.contains(':') {
                            // Strategy 1: check named next sibling for the missing value
                            let mut handled = false;
                            if let Some(&next) = children.get(i + 1) {
                                let next_raw = self.text_of(&next).trim().to_string();
                                let at_name = at_raw.trim_start_matches('@');
                                let matches_value = next_raw.starts_with(':')
                                    || (next_raw.starts_with(at_name) && next_raw.contains(':'));
                                if matches_value {
                                    let val_part =
                                        if let Some(stripped) = next_raw.strip_prefix(':') {
                                            stripped.trim().trim_end_matches(';').trim().to_string()
                                        } else if let Some(pos) = next_raw.find(':') {
                                            next_raw[pos + 1..]
                                                .trim()
                                                .trim_end_matches(';')
                                                .trim()
                                                .to_string()
                                        } else {
                                            next_raw.trim_end_matches(';').trim().to_string()
                                        };
                                    if !val_part.is_empty() {
                                        if !first {
                                            out.push(Line::new(0, "".to_string()));
                                        }
                                        first = false;
                                        out.push(Line::new(
                                            indent,
                                            format!("{}: {};", at_raw, val_part),
                                        ));
                                        i += 2;
                                        handled = true;
                                    }
                                }
                            }
                            if handled {
                                continue;
                            }
                            // Strategy 2: scan the source line at this node's start position
                            // to reconstruct the full `@name: value;` that tree-sitter split.
                            let start = child.start_byte();
                            if let Ok(src_str) =
                                std::str::from_utf8(self.source.get(start..).unwrap_or(b""))
                            {
                                let line_end = src_str.find('\n').unwrap_or(src_str.len());
                                let src_line = src_str[..line_end].trim();
                                if src_line.starts_with(&at_raw) && src_line.contains(':') {
                                    if let Some(colon_pos) = src_line.find(':') {
                                        let val = src_line[colon_pos + 1..]
                                            .trim()
                                            .trim_end_matches(';')
                                            .trim();
                                        if !val.is_empty() {
                                            if !first {
                                                out.push(Line::new(0, "".to_string()));
                                            }
                                            first = false;
                                            out.push(Line::new(
                                                indent,
                                                format!("{}: {};", at_raw, val),
                                            ));
                                            i += 1;
                                            continue;
                                        }
                                    }
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
                for line in self.format_declaration(node) {
                    out.push(Line::new(indent, line));
                }
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
            // SCSS/Less extensions
            "mixin_statement" | "include_statement" | "extend_statement" | "each_statement"
            | "for_statement" | "while_statement" | "if_statement" | "else_statement"
            | "apply_statement" | "use_statement" | "forward_statement" | "error_statement"
            | "warn_statement" | "debug_statement" | "function_statement" | "return_statement" => {
                self.walk_at_rule(node, indent, out);
            }
            "ERROR" => {
                let text = self.text_of(&node);
                for l in text.lines() {
                    out.push(Line::new(indent, l.trim().to_string()));
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
        // ── Step 1: get the raw source text for this entire at-rule node ──────
        let raw_text = self.text_of(&node).trim().to_string();

        let has_block = {
            let mut cursor = node.walk();
            let mut found = false;
            for child in node.children(&mut cursor) {
                if child.kind() == "block" {
                    found = true;
                    break;
                }
            }
            found
        };

        // ── Step 2: LESS @variable guard ─────────────────────────────────────
        // LESS variables look like `@name: value;` — they are NOT @rules.
        // Detect them by raw_text BEFORE any query reconstruction to avoid
        // the partial-parse bug where the value is dropped and we emit `@name: ;`.
        let is_known_at_rule = raw_text.starts_with("@media")
            || raw_text.starts_with("@keyframes")
            || raw_text.starts_with("@import")
            || raw_text.starts_with("@charset")
            || raw_text.starts_with("@font-face")
            || raw_text.starts_with("@supports")
            || raw_text.starts_with("@layer")
            || raw_text.starts_with("@namespace")
            || raw_text.starts_with("@use")
            || raw_text.starts_with("@forward")
            || raw_text.starts_with("@mixin")
            || raw_text.starts_with("@include")
            || raw_text.starts_with("@function")
            || raw_text.starts_with("@each")
            || raw_text.starts_with("@for")
            || raw_text.starts_with("@while")
            || raw_text.starts_with("@if")
            || raw_text.starts_with("@else")
            || raw_text.starts_with("@at-root");

        if !has_block && !is_known_at_rule && raw_text.starts_with('@') && raw_text.contains(':') {
            // LESS/SCSS variable declaration — emit verbatim, normalized.
            let normalized = raw_text.trim_end_matches(|c: char| c == ';' || c.is_whitespace());
            if let Some(colon_pos) = normalized.find(':') {
                let prop = normalized[..colon_pos].trim();
                let val = normalized[colon_pos + 1..].trim();
                if !val.is_empty() {
                    out.push(Line::new(indent, format!("{}: {};", prop, val)));
                } else {
                    out.push(Line::new(indent, format!("{};", prop)));
                }
            } else {
                out.push(Line::new(indent, format!("{};", normalized)));
            }
            return;
        }

        // ── Step 3: keyword + query for proper @rules ─────────────────────────
        // First try the named "keyword" field; if absent, parse from raw_text.
        let keyword: String = {
            let field_kw = node
                .child_by_field_name("keyword")
                .map(|n| self.text_of(&n).trim_start_matches('@').to_string());
            if let Some(kw) = field_kw {
                kw
            } else {
                // Extract keyword from raw text: `@mixin respond-to(...)` → "mixin"
                // Also try the "at-keyword" child node kind
                let from_child = {
                    let mut c = node.walk();
                    let mut kw = None;
                    for child in node.children(&mut c) {
                        if child.kind() == "at-keyword" {
                            kw = Some(self.text_of(&child).trim_start_matches('@').to_string());
                            break;
                        }
                    }
                    kw
                };
                from_child.unwrap_or_else(|| {
                    // Final fallback: parse from raw_text `@mixin ...` → "mixin"
                    raw_text
                        .trim_start_matches('@')
                        .split_whitespace()
                        .next()
                        .unwrap_or("at")
                        .split('(')
                        .next()
                        .unwrap_or("at")
                        .to_string()
                })
            }
        };

        let query = {
            let named_query = node
                .child_by_field_name("query")
                .map(|n| format!(" {}", self.text_of(&n)));

            named_query.unwrap_or_else(|| {
                let mut cursor = node.walk();
                let parts: Vec<&str> = node
                    .children(&mut cursor)
                    .filter(|n| {
                        let k = n.kind();
                        k != "@media"
                            && k != "@keyframes"
                            && k != "at-keyword"
                            && k != "block"
                            && k != ";"
                            && !self.text_of(n).trim().is_empty()
                            && self.text_of(n).trim() != keyword.as_str()
                            && self.text_of(n).trim() != format!("@{}", keyword).as_str()
                    })
                    .map(|n| self.text_of(&n))
                    .collect();
                if parts.is_empty() {
                    String::new()
                } else {
                    format!(" {}", parts.join(" "))
                }
            })
        };

        // ── Step 4: emit ──────────────────────────────────────────────────────
        // For SCSS-specific at-rules, use raw_text header verbatim to avoid
        // mangling $variable syntax during keyword+query reconstruction.
        let scss_verbatim = matches!(
            keyword.as_str(),
            "mixin"
                | "include"
                | "function"
                | "if"
                | "else"
                | "each"
                | "for"
                | "while"
                | "return"
                | "at-root"
        );

        if has_block {
            let mut block_node = None;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "block" {
                    block_node = Some(child);
                    break;
                }
            }
            let header = if scss_verbatim {
                if let Some(brace_pos) = raw_text.find('{') {
                    raw_text[..brace_pos]
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    format!("@{}{}", keyword, query)
                }
            } else {
                format!("@{}{}", keyword, query)
            };
            out.push(Line::new(indent, format!("{} {{", header)));
            if let Some(block) = block_node {
                self.walk_block_inner(block, indent + 1, out);
            }
            out.push(Line::new(indent, "}".to_string()));
        } else {
            let line_text = if scss_verbatim {
                let trimmed = raw_text.trim_end_matches(';').trim();
                format!("{};", trimmed)
            } else if query.trim().is_empty() && !raw_text.contains(':') {
                // Bare @name with no query and no colon: tree-sitter may have split
                // a LESS variable `@base: #f938ab;` into at_rule("@base") + unnamed(": #f938ab;").
                // Peek into source bytes after this node to reconstruct the value.
                let end = node.end_byte();
                let remainder = std::str::from_utf8(self.source.get(end..).unwrap_or(b""))
                    .unwrap_or("")
                    .trim_start();
                if remainder.starts_with(':') {
                    // Take up to the next `;` or newline
                    let val_end = remainder
                        .find(|c| [';', '\n'].contains(&c))
                        .unwrap_or(remainder.len());
                    let val = remainder[1..val_end].trim();
                    if !val.is_empty() {
                        format!("@{}: {};", keyword, val)
                    } else {
                        format!("@{};", keyword)
                    }
                } else {
                    format!("@{}{};", keyword, query)
                }
            } else {
                format!("@{}{};", keyword, query)
            };
            out.push(Line::new(indent, line_text));
        }
    }

    fn walk_media(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        // Try the named "query" field first (grammar-dependent).
        // Fallback: join all non-body, non-punctuation children as the condition.
        let query = node
            .child_by_field_name("query")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_else(|| {
                // Collect all named children except the block body
                let mut cursor = node.walk();
                let mut parts: Vec<String> = Vec::new();
                for child in node.children(&mut cursor) {
                    let k = child.kind();
                    if k == "block" || k == ";" {
                        continue;
                    }
                    let text = self.text_of(&child).trim().to_string();
                    if text == "@media" || text.is_empty() {
                        continue;
                    }
                    parts.push(text);
                }
                parts.join(" ")
            });

        out.push(Line::new(indent, format!("@media {} {{", query)));
        if let Some(block) = node.child_by_field_name("body") {
            let mut cursor = block.walk();
            for child in block.children(&mut cursor) {
                if !child.is_named() {
                    continue;
                }
                if !out.is_empty() {
                    out.push(Line::new(0, "".to_string()));
                }
                self.walk(child, indent + 1, out);
            }
        } else {
            // Try unnamed block (some grammar versions don't name the body field)
            let mut cursor = node.walk();
            let mut block_opt = None;
            for child in node.children(&mut cursor) {
                if child.kind() == "block" {
                    block_opt = Some(child);
                    break;
                }
            }
            if let Some(block) = block_opt {
                let mut cursor2 = block.walk();
                for child in block.children(&mut cursor2) {
                    if !child.is_named() {
                        continue;
                    }
                    if !out.is_empty() {
                        out.push(Line::new(0, "".to_string()));
                    }
                    self.walk(child, indent + 1, out);
                }
            }
        }
        out.push(Line::new(indent, "}".to_string()));
    }

    fn walk_block_inner(&self, node: tree_sitter::Node, indent: usize, out: &mut Vec<Line>) {
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|n| n.is_named())
            .collect();
        let mut i = 0;
        while i < children.len() {
            let child = children[i];
            match child.kind() {
                "declaration" => {
                    for line in self.format_declaration(child) {
                        out.push(Line::new(indent, line));
                    }
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
                "ERROR" => {
                    let raw = self.text_of(&child);
                    let text = raw.trim();
                    if text.ends_with(':') {
                        if let Some(&next) = children.get(i + 1) {
                            if next.kind() == "at_rule" {
                                let next_raw = self.text_of(&next).trim();
                                let prop = text.trim_end_matches(':').trim();
                                let val = next_raw.trim_end_matches(';').trim();
                                out.push(Line::new(indent, format!("{}: {};", prop, val)));
                                i += 2;
                                continue;
                            }
                        }
                    }
                    // tree-sitter-css may create ERROR nodes for non-standard declarations
                    // like `display:flex` (no space). Normalize them as declarations.
                    let raw = self.text_of(&child);
                    let text = raw.trim();
                    if !text.is_empty() {
                        if text.contains(':') && !text.contains('{') && !text.contains('}') {
                            let normalized = if let Some(pos) = text.find(':') {
                                let prop = text[..pos].trim().to_lowercase();
                                let val = text[pos + 1..].trim().trim_end_matches(';').trim();
                                format!("{}: {};", prop, val)
                            } else {
                                format!("{};", text.trim_end_matches(';'))
                            };
                            out.push(Line::new(indent, normalized));
                        } else {
                            // Multi-line or block ERROR
                            for l in text.lines() {
                                out.push(Line::new(indent, l.trim().to_string()));
                            }
                        }
                    }
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
            i += 1;
        }
    }

    fn format_declaration(&self, node: tree_sitter::Node) -> Vec<String> {
        let raw = self.text_of(&node).trim();
        let mut results = Vec::new();
        for part in raw.split(';') {
            let text = part.trim();
            if text.is_empty() {
                continue;
            }
            if let Some(colon_pos) = text.find(':') {
                let prop = text[..colon_pos].trim().to_lowercase();
                let after = text[colon_pos + 1..].trim();
                let (val, imp) = if after.trim_end().ends_with("!important") {
                    let v = after[..after.rfind('!').unwrap_or(after.len())].trim();
                    (v, " !important")
                } else {
                    (after, "")
                };
                results.push(format!("{}: {}{};", prop, val, imp));
            } else {
                results.push(format!("{};", text));
            }
        }
        if results.is_empty() && !raw.is_empty() {
            results.push(raw.to_string());
        }
        results
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

    /// Re-indent sub-formatter output into the HTML Line IR at `base_indent + 1`.
    ///
    /// Sub-formatters output relative indentation at depth 0 (e.g. outer
    /// function 0 spaces, inner body 2 spaces). We strip the minimum common
    /// indent to remove any residual offset, then bake the full HTML context
    /// prefix into the line content and emit via Line::new(0, …) so render()
    /// adds nothing extra. This correctly shifts the whole block without
    /// flattening the inner relative depth — fixing the double-indent flaw.
    fn reindent_zone_output(
        formatted: &str,
        base_indent: usize,
        indent_size: usize,
        out: &mut Vec<Line>,
    ) {
        // ponytail: strip absolute base, add exactly (base_indent+1)*indent_size spaces
        let stripped = Self::strip_indent(formatted.trim_end());
        let prefix = " ".repeat((base_indent + 1) * indent_size);
        for line in stripped.lines() {
            let t = line.trim_end();
            if t.is_empty() {
                out.push(Line::new(0, ""));
            } else {
                out.push(Line::new(0, format!("{}{}", prefix, t)));
            }
        }
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
        let add_blank = |out: &mut Vec<Line>| {
            if out.last().is_some_and(|l| !l.content.is_empty()) {
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
            inline_tag.push(' ');
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
            if !first
                && Self::is_block_or_special(*child, self.source)
                && out.last().is_some_and(|l| !l.content.is_empty())
            {
                out.push(Line::new(0, ""));
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
            Self::reindent_zone_output(&formatted_str, indent, config.indent_size as usize, out);
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
            Self::reindent_zone_output(&formatted_str, indent, config.indent_size as usize, out);
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
    let t_start = protocol::Instant::now();
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

            let t_format_start = protocol::Instant::now();
            let fmt = HtmlFormatter::new(source, config);
            let lines = fmt.format(tree.root_node());
            let t_format = t_format_start.elapsed();

            let t_emit_start = protocol::Instant::now();
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

            let t_format_start = protocol::Instant::now();
            let fmt = CssFormatter::new(source, config, dialect);
            let lines = fmt.format_tree(tree.root_node());
            let t_format = t_format_start.elapsed();

            let t_emit_start = protocol::Instant::now();
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
    fn html_embedded_script_not_double_indented() {
        // Regression test: embedded JS was double-indented (html_depth + sub_fmt_depth)
        // instead of (html_depth + 1 level). Fixed by reindent_zone_output.
        let src = br#"<!doctype html>
<html>
<body>
<script>
function hello() {
  console.log("hi");
}
</script>
</body>
</html>
"#;
        let config = ConfigIR::default();
        let result = format(src, &config, CssDialect::Html).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let indent_size = config.indent_size as usize;

        // Find the <script> tag line and get its indent level
        let script_leading = s
            .lines()
            .find(|l| l.trim().starts_with("<script"))
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);

        // JS content should be indented exactly one level deeper than <script>
        let fn_line = s.lines().find(|l| l.trim().starts_with("function hello"));
        if let Some(fn_line) = fn_line {
            let fn_leading = fn_line.len() - fn_line.trim_start().len();
            assert_eq!(
                fn_leading,
                script_leading + indent_size,
                "function hello() must be exactly 1 indent level inside <script>.\nFull output:\n{}",
                s
            );
        }

        // console.log is one level deeper than function hello (body of function)
        let log_line = s.lines().find(|l| l.trim().starts_with("console.log"));
        if let Some(log_line) = log_line {
            let log_leading = log_line.len() - log_line.trim_start().len();
            let fn_leading = s
                .lines()
                .find(|l| l.trim().starts_with("function hello"))
                .map(|l| l.len() - l.trim_start().len())
                .unwrap_or(0);
            assert_eq!(
                log_leading,
                fn_leading + indent_size,
                "console.log must be 1 level deeper than function hello.\nFull output:\n{}",
                s
            );
        }
    }

    #[test]
    fn html_embedded_style_not_double_indented() {
        // Regression: embedded CSS must be indented at <style>+1, not double-stacked.
        let src = br#"<html><head>
<style>
body {
  color: red;
}
</style>
</head></html>
"#;
        let config = ConfigIR::default();
        let result = format(src, &config, CssDialect::Html).unwrap();
        let s = std::str::from_utf8(&result).unwrap();
        let indent_size = config.indent_size as usize;

        let style_leading = s
            .lines()
            .find(|l| l.trim().starts_with("<style"))
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);

        // body { selector must be exactly 1 level inside <style>
        let body_line = s.lines().find(|l| l.trim().starts_with("body"));
        if let Some(body_line) = body_line {
            let body_leading = body_line.len() - body_line.trim_start().len();
            assert_eq!(
                body_leading,
                style_leading + indent_size,
                "body rule must be exactly 1 indent level inside <style>.\nFull output:\n{}",
                s
            );
        }
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
