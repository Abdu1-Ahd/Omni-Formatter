//! Comment Anchoring Engine (L-06 mitigation)
//!
//! Before the language module formats a zone, this pass walks the Tree-sitter CST
//! and attaches every comment node to the nearest meaningful syntactic sibling.
//! After formatting, the re-attachment pass inserts comments at the position
//! of their anchored node in the formatted output.
//!
//! # Anchor Rules
//!
//! - Block comments before a function/class declaration → anchored to that declaration.
//! - Line comments between statements → anchored to the next statement.
//! - Trailing line comments after a statement → anchored to that statement.
//! - Floating comments at end of block with no following sibling → anchored to previous.
//!
//! # Magic Suppression Comments
//!
//! The following comment tokens suppress formatting for the next syntactic sibling:
//! - `// prettier-ignore` (JavaScript/TypeScript/CSS)
//! - `# fmt: off` (Python)
//! - `// rustfmt::skip` (Rust)
//! - `/* omnifmt-ignore */` (universal)
//!
//! When a suppression comment is detected, its next sibling node is emitted
//! verbatim (byte-for-byte copy from source) without calling the language module.
//!
//! # Implementation Status
//!
//! Phase 3 stub: the public API is defined. Full Tree-sitter CST traversal
//! is implemented in Phase 4 when Tree-sitter grammars are integrated.

/// A comment anchor: maps a comment's position in the source to a node ID
/// in the CST that it should follow in the formatted output.
#[derive(Debug, Clone)]
pub struct CommentAnchor {
    /// Byte offset of the comment in the original source.
    pub comment_start: usize,
    /// Byte offset end of the comment in the original source.
    pub comment_end: usize,
    /// The full text of the comment (including `//`, `#`, `/*`, etc.).
    pub comment_text: Vec<u8>,
    /// Whether this is a magic suppression comment.
    pub is_suppression: bool,
    /// Anchor kind: before or after the sibling node.
    pub anchor_kind: AnchorKind,
    /// The byte range of the anchored sibling node in the source.
    pub sibling_range: protocol::ByteRange,
}

/// Whether the comment appears before or after its anchored node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorKind {
    /// Comment appears before the anchored node (doc comment, section marker).
    Before,
    /// Comment appears after the anchored node on the same line (trailing comment).
    After,
}

/// Magic suppression comment tokens recognized by the engine.
pub const SUPPRESSION_TOKENS: &[&str] = &[
    "// prettier-ignore",
    "# fmt: off",
    "// rustfmt::skip",
    "/* omnifmt-ignore */",
    "// omnifmt-ignore",
];

/// Build the comment anchor map for a source file.
pub fn build_anchor_map(source: &[u8], language_id: &str) -> Vec<CommentAnchor> {
    let lang = match language_id {
        "javascript" | "javascriptreact" => tree_sitter_javascript::language(),
        "typescript" | "typescriptreact" => tree_sitter_typescript::language_tsx(),
        "python" => tree_sitter_python::language(),
        "rust" => tree_sitter_rust::language(),
        "go" => tree_sitter_go::language(),
        "css" | "scss" | "less" => tree_sitter_css::language(),
        "html" | "svelte" | "vue" | "astro" => tree_sitter_html::language(),
        _ => return Vec::new(),
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut anchors = Vec::new();
    let mut cursor = tree.walk();

    traverse_for_comments(&mut cursor, source, &mut anchors);

    anchors
}

pub fn find_suppressed_zones(source: &[u8], language_id: &str) -> Vec<protocol::zone::Zone> {
    let anchors = build_anchor_map(source, language_id);
    anchors
        .into_iter()
        .filter(|a| a.is_suppression && a.anchor_kind == AnchorKind::Before)
        .map(|a| protocol::zone::Zone {
            language_id: language_id.to_string(),
            range: a.sibling_range,
            indent_column: 0,
            suppressed: true,
            kind: protocol::zone::ZoneKind::Comment,
        })
        .collect()
}

fn traverse_for_comments(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    anchors: &mut Vec<CommentAnchor>,
) {
    loop {
        let node = cursor.node();
        if node.kind() == "comment" {
            let comment_text = &source[node.start_byte()..node.end_byte()];
            let is_suppression = is_suppression_comment(comment_text);

            let prev_sibling = node.prev_named_sibling();
            let next_sibling = node.next_named_sibling();

            let mut anchor_kind = AnchorKind::Before;
            let mut sibling_range = protocol::ByteRange { start: 0, end: 0 };

            if let Some(prev) = prev_sibling {
                if prev.end_position().row == node.start_position().row {
                    anchor_kind = AnchorKind::After;
                    sibling_range = protocol::ByteRange {
                        start: prev.start_byte(),
                        end: prev.end_byte(),
                    };
                } else if let Some(next) = next_sibling {
                    anchor_kind = AnchorKind::Before;
                    sibling_range = protocol::ByteRange {
                        start: next.start_byte(),
                        end: next.end_byte(),
                    };
                } else {
                    anchor_kind = AnchorKind::After;
                    sibling_range = protocol::ByteRange {
                        start: prev.start_byte(),
                        end: prev.end_byte(),
                    };
                }
            } else if let Some(next) = next_sibling {
                anchor_kind = AnchorKind::Before;
                sibling_range = protocol::ByteRange {
                    start: next.start_byte(),
                    end: next.end_byte(),
                };
            }

            // Only add if we found a sibling
            if prev_sibling.is_some() || next_sibling.is_some() {
                anchors.push(CommentAnchor {
                    comment_start: node.start_byte(),
                    comment_end: node.end_byte(),
                    comment_text: comment_text.to_vec(),
                    is_suppression,
                    anchor_kind,
                    sibling_range,
                });
            }
        }

        if cursor.goto_first_child() {
            traverse_for_comments(cursor, source, anchors);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

/// Check if a byte slice starts with any known suppression token.
pub fn is_suppression_comment(text: &[u8]) -> bool {
    SUPPRESSION_TOKENS.iter().any(|token| {
        text.len() >= token.len() && text[..token.len()].eq_ignore_ascii_case(token.as_bytes())
    })
}

/// Re-attach comments to their anchored positions in the formatted output.
///
/// # Arguments
///
/// * `formatted` — The bytes produced by the language module.
/// * `anchors` — The anchor map built from the original source.
///
/// # Returns
///
/// The formatted bytes with comments re-inserted at their correct positions.
///
/// # Phase 3 Status
///
/// Returns `formatted` unchanged (stub). Full implementation in Phase 4.
pub fn reattach_comments(formatted: Vec<u8>, anchors: &[CommentAnchor]) -> Vec<u8> {
    // Phase 3 stub: pass through unchanged.
    let _ = anchors;
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_anchor_map_finds_anchor() {
        let source = b"// hello\nconst x = 1;";
        let anchors = build_anchor_map(source, "javascript");
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].anchor_kind, AnchorKind::Before);
    }

    #[test]
    fn is_suppression_detects_prettier_ignore() {
        assert!(is_suppression_comment(b"// prettier-ignore"));
        assert!(is_suppression_comment(b"// prettier-ignore\n"));
    }

    #[test]
    fn is_suppression_detects_fmt_off() {
        assert!(is_suppression_comment(b"# fmt: off"));
    }

    #[test]
    fn is_suppression_rejects_normal_comment() {
        assert!(!is_suppression_comment(b"// This is a comment"));
        assert!(!is_suppression_comment(b"# normal comment"));
    }

    #[test]
    fn reattach_comments_stub_passthrough() {
        let formatted = b"const x = 1;".to_vec();
        let result = reattach_comments(formatted.clone(), &[]);
        assert_eq!(result, formatted);
    }
}
