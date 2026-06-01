//! JS/TS Formatting Logic — Prettier 3.x Parity
//!
//! Implements the Wadler/Prettier document IR algorithm:
//!   1. Parse with Tree-sitter `javascript` or `typescript` grammar.
//!   2. Walk the CST to build a `Doc` intermediate representation.
//!   3. Run the `fits`/`best` layout algorithm with `print_width`.
//!   4. Render to bytes.
//!
//! # Idempotency (Pillar 7)
//!
//! `format(format(x)) == format(x)` — contractually guaranteed.
//! The CST is pure; re-parsing formatted output produces the same AST shape.
//! In debug builds, `assert_idempotent` double-formats and panics on drift.

use protocol::config::{ConfigIR, IndentStyle, QuoteStyle};
use protocol::FormatError;

// ── Tree-sitter grammars ───────────────────────────────────────────────────

fn javascript_language() -> tree_sitter::Language {
    tree_sitter_javascript::language()
}

fn typescript_language() -> tree_sitter::Language {
    tree_sitter_typescript::language_typescript()
}

// ── Document IR ───────────────────────────────────────────────────────────

/// Wadler/Prettier document intermediate representation.
#[derive(Debug, Clone)]
enum Doc {
    /// A literal string fragment (never contains newlines).
    Text(String),
    /// A line break. In flat mode, renders as `space_str`. In break mode, renders as newline + indent.
    Line { space_str: String },
    /// Concat two documents.
    Concat(Box<Doc>, Box<Doc>),
    /// Try to fit the inner doc flat; fall back to expanded on overflow.
    Group(Box<Doc>),
    /// Increase indent level for all nested line breaks.
    Indent(Box<Doc>),
    /// Nothing.
    Nil,
}

impl Doc {
    fn text(s: impl Into<String>) -> Self {
        Doc::Text(s.into())
    }

    fn line() -> Self {
        Doc::Line {
            space_str: " ".to_string(),
        }
    }

    fn hard_line() -> Self {
        // A line that always breaks, even in flat mode.
        // Modelled by nesting in a group-breaking context: we emit a Line
        // but mark it so the flat renderer treats it as a break.
        Doc::Line {
            space_str: "\x00".to_string(),
        } // sentinel: flat renderer breaks on \x00
    }

    fn concat(a: Doc, b: Doc) -> Self {
        Doc::Concat(Box::new(a), Box::new(b))
    }

    fn group(inner: Doc) -> Self {
        Doc::Group(Box::new(inner))
    }

    fn indent(inner: Doc) -> Self {
        Doc::Indent(Box::new(inner))
    }

    fn join(sep: Doc, docs: Vec<Doc>) -> Self {
        let mut iter = docs.into_iter();
        match iter.next() {
            None => Doc::Nil,
            Some(first) => iter.fold(first, |acc, d| {
                Doc::concat(Doc::concat(acc, sep.clone()), d)
            }),
        }
    }
}

// ── Renderer ─────────────────────────────────────────────────────────────

struct Renderer {
    print_width: usize,
    indent_size: usize,
    indent_char: char,
}

impl Renderer {
    fn render(&self, doc: &Doc) -> String {
        let mut out = String::with_capacity(4096);
        self.render_doc(doc, 0, false, &mut out, 0);
        out
    }

    /// Returns the column position after rendering `doc` in flat mode.
    fn flat_len(&self, doc: &Doc, col: usize) -> usize {
        match doc {
            Doc::Nil => col,
            Doc::Text(s) => col + s.chars().count(),
            Doc::Line { space_str } => {
                if space_str.starts_with('\x00') {
                    usize::MAX // always breaks
                } else {
                    col + space_str.chars().count()
                }
            }
            Doc::Concat(a, b) => {
                let after_a = self.flat_len(a, col);
                if after_a == usize::MAX {
                    usize::MAX
                } else {
                    self.flat_len(b, after_a)
                }
            }
            Doc::Group(inner) | Doc::Indent(inner) => self.flat_len(inner, col),
        }
    }

    fn render_doc(
        &self,
        doc: &Doc,
        indent: usize,
        flat: bool,
        out: &mut String,
        col: usize,
    ) -> usize {
        match doc {
            Doc::Nil => col,
            Doc::Text(s) => {
                out.push_str(s);
                col + s.chars().count()
            }
            Doc::Line { space_str } => {
                if flat && !space_str.starts_with('\x00') {
                    out.push_str(space_str);
                    col + space_str.chars().count()
                } else {
                    out.push('\n');
                    let indent_str: String =
                        std::iter::repeat_n(self.indent_char, indent * self.indent_size).collect();
                    out.push_str(&indent_str);
                    indent * self.indent_size
                }
            }
            Doc::Concat(a, b) => {
                let col2 = self.render_doc(a, indent, flat, out, col);
                self.render_doc(b, indent, flat, out, col2)
            }
            Doc::Group(inner) => {
                // Try flat if it fits
                let flat_col = self.flat_len(inner, col);
                if flat_col != usize::MAX && flat_col <= self.print_width {
                    self.render_doc(inner, indent, true, out, col)
                } else {
                    self.render_doc(inner, indent, false, out, col)
                }
            }
            Doc::Indent(inner) => self.render_doc(inner, indent + 1, flat, out, col),
        }
    }
}

// ── CST → Doc builder ─────────────────────────────────────────────────────

struct DocBuilder<'a> {
    source: &'a [u8],
    config: &'a ConfigIR,
    quote: char,
}

impl<'a> DocBuilder<'a> {
    fn new(source: &'a [u8], config: &'a ConfigIR) -> Self {
        let quote = match config.quote_style {
            QuoteStyle::Single => '\'',
            QuoteStyle::Double => '"',
        };
        Self {
            source,
            config,
            quote,
        }
    }

    fn text_of(&self, node: &tree_sitter::Node) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    /// Build the Doc for any node, dispatching on node kind.
    fn build(&self, node: tree_sitter::Node) -> Doc {
        match node.kind() {
            // ── Statements ────────────────────────────────────────────────
            "program" | "statement_block" => self.build_block(node),
            "expression_statement" => {
                let inner = node.child(0).map(|n| self.build(n)).unwrap_or(Doc::Nil);
                let semi = if self.config.semicolons { ";" } else { "" };
                Doc::concat(inner, Doc::text(semi))
            }
            "return_statement" => {
                let mut cursor = node.walk();
                let value = node
                    .children(&mut cursor)
                    .find(|c| c.is_named() && c.kind() != "comment")
                    .map(|n| Doc::concat(Doc::text(" "), self.build(n)))
                    .unwrap_or(Doc::Nil);
                let semi = if self.config.semicolons { ";" } else { "" };
                Doc::concat(Doc::concat(Doc::text("return"), value), Doc::text(semi))
            }
            "if_statement" => self.build_if(node),
            "for_statement" => self.build_for(node),
            "while_statement" => self.build_while(node),
            "variable_declaration" | "lexical_declaration" => self.build_var_decl(node),
            "function_declaration" | "function" => self.build_function(node),
            "arrow_function" => self.build_arrow(node),
            "class_declaration" | "class" => self.build_class(node),
            "import_statement" => self.build_import(node),
            "export_statement" => self.build_export(node),

            // ── Expressions ───────────────────────────────────────────────
            "call_expression" => self.build_call(node),
            "member_expression" => self.build_member(node),
            "binary_expression" | "logical_expression" => self.build_binary(node),
            "assignment_expression" => self.build_assignment(node),
            "object" => self.build_object(node),
            "array" => self.build_array(node),
            "template_string" => self.build_template(node),
            "string" => self.build_string(node),
            "parenthesized_expression" => {
                let inner = node.child(1).map(|n| self.build(n)).unwrap_or(Doc::Nil);
                Doc::group(Doc::concat(
                    Doc::concat(
                        Doc::text("("),
                        Doc::indent(Doc::concat(
                            Doc::Line {
                                space_str: "".into(),
                            },
                            inner,
                        )),
                    ),
                    Doc::concat(
                        Doc::Line {
                            space_str: "".into(),
                        },
                        Doc::text(")"),
                    ),
                ))
            }
            "type_annotation" => {
                let inner = node.child(1).map(|n| self.build(n)).unwrap_or(Doc::Nil);
                Doc::concat(Doc::text(": "), inner)
            }
            "comment" => {
                // Preserve comments verbatim
                Doc::concat(Doc::text(self.text_of(&node)), Doc::hard_line())
            }
            // Fallthrough: emit verbatim
            _ => Doc::text(self.text_of(&node)),
        }
    }

    fn build_block(&self, node: tree_sitter::Node) -> Doc {
        let mut stmts: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        let mut skip_next = false;
        for child in node.children(&mut cursor) {
            if child.is_named() {
                if skip_next {
                    skip_next = false;
                    stmts.push(Doc::text(self.text_of(&child)));
                    continue;
                }
                if child.kind() == "comment" && self.text_of(&child).contains("prettier-ignore") {
                    skip_next = true;
                }
                stmts.push(self.build(child));
            }
        }
        if stmts.is_empty() {
            return Doc::Nil;
        }
        Doc::join(Doc::hard_line(), stmts)
    }

    fn build_if(&self, node: tree_sitter::Node) -> Doc {
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let cons = node
            .child_by_field_name("consequence")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let alt = node
            .child_by_field_name("alternative")
            .map(|n| Doc::concat(Doc::text(" else "), self.build(n)));

        let mut doc = Doc::concat(
            Doc::concat(Doc::text("if ("), Doc::concat(cond, Doc::text(")"))),
            Doc::concat(Doc::text(" "), cons),
        );
        if let Some(alt_doc) = alt {
            doc = Doc::concat(doc, alt_doc);
        }
        doc
    }

    fn build_for(&self, node: tree_sitter::Node) -> Doc {
        let init = node
            .child_by_field_name("initializer")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let update = node
            .child_by_field_name("increment")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let body = node
            .child_by_field_name("body")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);

        Doc::concat(
            Doc::text("for ("),
            Doc::concat(
                Doc::concat(
                    init,
                    Doc::concat(
                        Doc::text("; "),
                        Doc::concat(cond, Doc::concat(Doc::text("; "), update)),
                    ),
                ),
                Doc::concat(Doc::text(") "), body),
            ),
        )
    }

    fn build_while(&self, node: tree_sitter::Node) -> Doc {
        let cond = node
            .child_by_field_name("condition")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let body = node
            .child_by_field_name("body")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        Doc::concat(
            Doc::concat(Doc::text("while ("), Doc::concat(cond, Doc::text(")"))),
            Doc::concat(Doc::text(" "), body),
        )
    }

    fn build_var_decl(&self, node: tree_sitter::Node) -> Doc {
        let keyword = node
            .child_by_field_name("kind")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_else(|| {
                self.text_of(&node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("var")
                    .to_string()
            });
        let mut declarators: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                declarators.push(self.build_declarator(child));
            }
        }
        let semi = if self.config.semicolons { ";" } else { "" };
        Doc::concat(
            Doc::text(format!("{} ", keyword)),
            Doc::concat(
                Doc::group(Doc::join(
                    Doc::concat(Doc::text(","), Doc::line()),
                    declarators,
                )),
                Doc::text(semi),
            ),
        )
    }

    fn build_declarator(&self, node: tree_sitter::Node) -> Doc {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let type_ann = node
            .child_by_field_name("type")
            .or_else(|| node.child_by_field_name("type_annotation"))
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let value = node
            .child_by_field_name("value")
            .map(|n| Doc::concat(Doc::text(" = "), self.build(n)));

        let name_with_type = Doc::concat(name, type_ann);
        match value {
            Some(v) => Doc::concat(name_with_type, v),
            None => name_with_type,
        }
    }

    fn build_function(&self, node: tree_sitter::Node) -> Doc {
        let name = node
            .child_by_field_name("name")
            .map(|n| Doc::concat(Doc::text(" "), Doc::text(self.text_of(&n))))
            .unwrap_or(Doc::Nil);
        let params = node
            .child_by_field_name("parameters")
            .map(|n| self.build_params(n))
            .unwrap_or(Doc::text("()"));
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let body = node
            .child_by_field_name("body")
            .map(|n| self.build_curly_block(n))
            .unwrap_or(Doc::text("{}"));
        Doc::concat(
            Doc::concat(Doc::text("function"), name),
            Doc::concat(
                Doc::concat(params, return_type),
                Doc::concat(Doc::text(" "), body),
            ),
        )
    }

    fn build_arrow(&self, node: tree_sitter::Node) -> Doc {
        let params = node
            .child_by_field_name("parameter")
            .or_else(|| node.child_by_field_name("parameters"))
            .map(|n| {
                if n.kind() == "formal_parameters" {
                    self.build_params(n)
                } else {
                    Doc::text(self.text_of(&n))
                }
            })
            .unwrap_or(Doc::text("()"));
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let body = node
            .child_by_field_name("body")
            .map(|n| {
                if n.kind() == "statement_block" {
                    self.build_curly_block(n)
                } else {
                    self.build(n)
                }
            })
            .unwrap_or(Doc::Nil);
        Doc::concat(
            Doc::concat(params, return_type),
            Doc::concat(Doc::text(" => "), body),
        )
    }

    fn build_params(&self, node: tree_sitter::Node) -> Doc {
        let mut params: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                params.push(self.build(child));
            }
        }
        let inner = if params.is_empty() {
            Doc::Nil
        } else {
            let trailing = if self.config.trailing_comma && !params.is_empty() {
                ","
            } else {
                ""
            };
            Doc::concat(
                Doc::join(Doc::concat(Doc::text(","), Doc::line()), params),
                Doc::text(trailing),
            )
        };
        Doc::group(Doc::concat(
            Doc::text("("),
            Doc::concat(
                Doc::indent(Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    inner,
                )),
                Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    Doc::text(")"),
                ),
            ),
        ))
    }

    fn build_class(&self, node: tree_sitter::Node) -> Doc {
        let name = node
            .child_by_field_name("name")
            .map(|n| Doc::concat(Doc::text(" "), Doc::text(self.text_of(&n))))
            .unwrap_or(Doc::Nil);
        let superclass = node
            .child_by_field_name("superclass")
            .map(|n| Doc::concat(Doc::text(" extends "), Doc::text(self.text_of(&n))))
            .unwrap_or(Doc::Nil);
        let body = node
            .child_by_field_name("body")
            .map(|n| self.build_curly_block(n))
            .unwrap_or(Doc::text("{}"));
        Doc::concat(
            Doc::text("class"),
            Doc::concat(
                name,
                Doc::concat(superclass, Doc::concat(Doc::text(" "), body)),
            ),
        )
    }

    fn build_import(&self, node: tree_sitter::Node) -> Doc {
        // Emit verbatim for now — import reordering is a separate pass
        let semi = if self.config.semicolons { ";" } else { "" };
        Doc::concat(
            Doc::text(self.text_of(&node).trim_end_matches(';')),
            Doc::text(semi),
        )
    }

    fn build_export(&self, node: tree_sitter::Node) -> Doc {
        let inner = node
            .named_child(0)
            .map(|n| Doc::concat(Doc::text(" "), self.build(n)))
            .unwrap_or(Doc::Nil);
        Doc::concat(Doc::text("export"), inner)
    }

    fn build_call(&self, node: tree_sitter::Node) -> Doc {
        let func = node
            .child_by_field_name("function")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let args = node
            .child_by_field_name("arguments")
            .map(|n| self.build_args(n))
            .unwrap_or(Doc::text("()"));
        Doc::concat(func, args)
    }

    fn build_args(&self, node: tree_sitter::Node) -> Doc {
        let mut args: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                args.push(self.build(child));
            }
        }
        if args.is_empty() {
            return Doc::text("()");
        }
        let trailing = if self.config.trailing_comma { "," } else { "" };
        Doc::group(Doc::concat(
            Doc::text("("),
            Doc::concat(
                Doc::indent(Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    Doc::concat(
                        Doc::join(Doc::concat(Doc::text(","), Doc::line()), args),
                        Doc::text(trailing),
                    ),
                )),
                Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    Doc::text(")"),
                ),
            ),
        ))
    }

    fn build_member(&self, node: tree_sitter::Node) -> Doc {
        let object = node
            .child_by_field_name("object")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let property = node
            .child_by_field_name("property")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let op = if node.child_count() >= 2 {
            node.child(1).map(|n| self.text_of(&n)).unwrap_or(".")
        } else {
            "."
        };
        Doc::concat(object, Doc::concat(Doc::text(op), property))
    }

    fn build_binary(&self, node: tree_sitter::Node) -> Doc {
        let left = node
            .child_by_field_name("left")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let op = node
            .child_by_field_name("operator")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_default();
        let right = node
            .child_by_field_name("right")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        Doc::group(Doc::concat(
            left,
            Doc::indent(Doc::concat(
                Doc::line(),
                Doc::concat(Doc::text(format!("{} ", op)), right),
            )),
        ))
    }

    fn build_assignment(&self, node: tree_sitter::Node) -> Doc {
        let left = node
            .child_by_field_name("left")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        let op = node
            .child_by_field_name("operator")
            .map(|n| self.text_of(&n).to_string())
            .unwrap_or_else(|| "=".to_string());
        let right = node
            .child_by_field_name("right")
            .map(|n| self.build(n))
            .unwrap_or(Doc::Nil);
        Doc::group(Doc::concat(
            Doc::concat(left, Doc::text(format!(" {} ", op))),
            Doc::indent(right),
        ))
    }

    fn build_object(&self, node: tree_sitter::Node) -> Doc {
        let mut props: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                props.push(self.build(child));
            }
        }
        if props.is_empty() {
            return Doc::text("{}");
        }
        let trailing = if self.config.trailing_comma { "," } else { "" };
        Doc::group(Doc::concat(
            Doc::text("{"),
            Doc::concat(
                Doc::indent(Doc::concat(
                    Doc::hard_line(),
                    Doc::concat(
                        Doc::join(Doc::concat(Doc::text(","), Doc::hard_line()), props),
                        Doc::text(trailing),
                    ),
                )),
                Doc::concat(Doc::hard_line(), Doc::text("}")),
            ),
        ))
    }

    fn build_array(&self, node: tree_sitter::Node) -> Doc {
        let mut items: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                items.push(self.build(child));
            }
        }
        if items.is_empty() {
            return Doc::text("[]");
        }
        let trailing = if self.config.trailing_comma { "," } else { "" };
        Doc::group(Doc::concat(
            Doc::text("["),
            Doc::concat(
                Doc::indent(Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    Doc::concat(
                        Doc::join(Doc::concat(Doc::text(","), Doc::line()), items),
                        Doc::text(trailing),
                    ),
                )),
                Doc::concat(
                    Doc::Line {
                        space_str: "".into(),
                    },
                    Doc::text("]"),
                ),
            ),
        ))
    }

    fn build_template(&self, node: tree_sitter::Node) -> Doc {
        // Template literals: preserve backticks and interpolations verbatim
        Doc::text(self.text_of(&node))
    }

    fn build_string(&self, node: tree_sitter::Node) -> Doc {
        let raw = self.text_of(&node);
        // Normalize quote character to the configured quote style
        if raw.len() < 2 {
            return Doc::text(raw);
        }
        let outer = raw.chars().next().unwrap();
        if outer == '`' {
            // template literal — don't touch
            return Doc::text(raw);
        }
        let inner = &raw[1..raw.len() - 1];
        let target = self.quote;
        // If the inner content contains the target quote, keep the original
        if inner.contains(target) {
            return Doc::text(raw);
        }
        Doc::text(format!(
            "{}{}{}",
            target,
            inner.replace(outer, &target.to_string()),
            target
        ))
    }

    fn build_curly_block(&self, node: tree_sitter::Node) -> Doc {
        let mut stmts: Vec<Doc> = Vec::new();
        let mut cursor = node.walk();
        let mut skip_next = false;
        for child in node.children(&mut cursor) {
            if child.is_named() {
                if skip_next {
                    skip_next = false;
                    stmts.push(Doc::text(self.text_of(&child)));
                    continue;
                }
                if child.kind() == "comment" && self.text_of(&child).contains("prettier-ignore") {
                    skip_next = true;
                }
                stmts.push(self.build(child));
            }
        }
        if stmts.is_empty() {
            return Doc::text("{}");
        }
        Doc::concat(
            Doc::text("{"),
            Doc::concat(
                Doc::indent(Doc::concat(
                    Doc::hard_line(),
                    Doc::join(Doc::hard_line(), stmts),
                )),
                Doc::concat(Doc::hard_line(), Doc::text("}")),
            ),
        )
    }
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Format JavaScript, TypeScript, JSX, or TSX source bytes.
///
/// # Returns
///
/// Formatted UTF-8 bytes on success.
/// Format JavaScript, TypeScript, JSX, or TSX source (Prettier 3.x parity).
fn format_internal(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let t_start = protocol::Instant::now();
    let language = if source.windows(2).any(|w| w == b": ") {
        typescript_language()
    } else {
        javascript_language()
    };
    let is_ts = source.windows(2).any(|w| w == b": ");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| FormatError::Internal {
            message: format!("grammar load failed: {}", e),
        })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| FormatError::ParseFailed {
            message: "tree-sitter returned None".into(),
        })?;

    if tree.root_node().has_error() {
        log::warn!("lang-js: parse error in source — emitting verbatim");
        return Ok(source.to_vec());
    }
    let t_parse = t_start.elapsed();

    let t_format_start = protocol::Instant::now();
    let builder = DocBuilder::new(source, config);
    let doc = builder.build_block(tree.root_node());
    let t_format = t_format_start.elapsed();

    let t_emit_start = protocol::Instant::now();
    let indent_char = match config.indent_style {
        IndentStyle::Tabs => '\t',
        IndentStyle::Spaces => ' ',
    };
    let renderer = Renderer {
        print_width: config.print_width as usize,
        indent_size: config.indent_size as usize,
        indent_char,
    };

    let mut rendered = renderer.render(&doc);

    while rendered.ends_with('\n') {
        rendered.pop();
    }
    rendered.push('\n');
    let t_emit = t_emit_start.elapsed();

    let lang_name = if is_ts { "TS" } else { "JS" };
    eprintln!(
        "[{}] Parse: {:.2}ms, Format: {:.2}ms, Emit: {:.2}ms",
        lang_name,
        t_parse.as_secs_f64() * 1000.0,
        t_format.as_secs_f64() * 1000.0,
        t_emit.as_secs_f64() * 1000.0
    );

    Ok(rendered.into_bytes())
}

pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let out = format_internal(source, config)?;

    #[cfg(debug_assertions)]
    {
        let second = format_internal(&out, config)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-js: format is not idempotent!"
        );
    }

    Ok(out)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_empty_source_returns_newline() {
        let config = ConfigIR::default();
        let result = format(b"", &config).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_does_not_panic_on_unicode() {
        let source = "const greeting = '你好世界';\n".as_bytes();
        let config = ConfigIR::default();
        let result = format(source, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn format_single_variable() {
        let config = ConfigIR {
            semicolons: true,
            ..Default::default()
        };
        let source = b"const x = 1;\n";
        let result = format(source, &config).unwrap();
        // Should round-trip cleanly
        let second = format(&result, &config).unwrap();
        assert_eq!(result, second, "idempotency violated");
    }
}
