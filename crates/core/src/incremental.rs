//! Format-on-Type Incremental Protocol (L-13 mitigation)
//!
//! Format-on-type must complete in under 16ms total round-trip.
//! Re-parsing the entire file on each keystroke is impossible within this budget.
//!
//! # Protocol
//!
//! The extension host sends a `FormatOnTypeRequest` containing:
//! - The current document bytes.
//! - The edit delta (start offset, deleted bytes, inserted bytes).
//! - The serialized previous Tree-sitter tree (from the last parse).
//!
//! The WASM core:
//! 1. Deserializes the previous tree.
//! 2. Calls `tree.edit(edit)` — O(log n) incremental re-parse.
//! 3. Identifies the smallest complete syntactic unit containing the edit offset.
//! 4. Formats only that unit, not the full file.
//!
//! # Performance Target
//!
//! 16ms total (extension host → worker postMessage → WASM format → result → VS Code edit apply)
//! on a 2000-line file with a single-character keystroke edit.
//! This is enforced as a failing CI benchmark on every commit.
//!
//! # Implementation Status
//!
//! Phase 3 scaffold. Full Tree-sitter incremental parse integration in Phase 4.

use protocol::{ByteRange, EditDelta};

/// The result of computing the dirty region for an incremental format.
#[derive(Debug, Clone)]
pub struct DirtyRegion {
    /// The byte range of the smallest complete syntactic unit
    /// that contains the edit offset and should be reformatted.
    pub range: ByteRange,

    /// The language ID of the zone containing this dirty region.
    /// For single-language files, this is the document's primary language.
    pub language_id: String,
}

/// Compute the dirty region for an incremental format-on-type request.
///
/// # Arguments
///
/// * `source` — The current document bytes (post-edit).
/// * `edit` — The edit delta that triggered this format-on-type.
/// * `language_id` — The primary language of the document.
///
/// # Returns
///
/// The `DirtyRegion` that should be formatted — the smallest complete
/// syntactic unit containing the edit offset.
pub fn compute_dirty_region(source: &[u8], edit: &EditDelta, language_id: &str) -> DirtyRegion {
    let mut parser = tree_sitter::Parser::new();
    let language = match language_id {
        "javascript" | "javascriptreact" => tree_sitter_javascript::language(),
        "typescript" | "typescriptreact" => tree_sitter_typescript::language_typescript(),
        "python" => tree_sitter_python::language(),
        "rust" => tree_sitter_rust::language(),
        "go" => tree_sitter_go::language(),
        "css" | "scss" | "less" => tree_sitter_css::language(),
        "html" => tree_sitter_html::language(),
        _ => return fallback_region(source, edit, language_id),
    };

    if parser.set_language(&language).is_err() {
        return fallback_region(source, edit, language_id);
    }

    if let Some(tree) = parser.parse(source, None) {
        let mut cursor = tree.walk();
        let mut current_node = tree.root_node();
        let target_byte = edit.start;

        // Find the deepest node covering the edit start byte
        loop {
            let mut found_child = false;
            for child in current_node.children(&mut cursor) {
                // tree-sitter uses exclusive end_byte
                if child.start_byte() <= target_byte && child.end_byte() > target_byte {
                    current_node = child;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                break;
            }
        }

        // Walk up to find the nearest complete statement or block
        while current_node.parent().is_some() {
            let kind = current_node.kind();
            if (kind.ends_with("statement")
                || kind.ends_with("declaration")
                || kind == "block"
                || kind == "program"
                || kind == "source_file"
                || kind == "document"
                || kind == "module")
                && !current_node.has_error()
            {
                break;
            }
            if let Some(parent) = current_node.parent() {
                current_node = parent;
            } else {
                break;
            }
        }

        return DirtyRegion {
            range: ByteRange {
                start: current_node.start_byte(),
                end: current_node.end_byte(),
            },
            language_id: language_id.to_string(),
        };
    }

    fallback_region(source, edit, language_id)
}

fn fallback_region(source: &[u8], edit: &EditDelta, language_id: &str) -> DirtyRegion {
    let requested = ByteRange {
        start: edit.start,
        end: edit.start + edit.inserted.len(),
    };
    let expanded = crate::range::expand_to_nearest_unit(source, requested, language_id);
    DirtyRegion {
        range: expanded.range,
        language_id: language_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_region_single_statement() {
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("let var{} = {};\n", i, i));
        }

        let mut bytes = source.into_bytes();
        // find start of line 50 (index 49)
        let edit_start = bytes
            .iter()
            .enumerate()
            .filter(|(_, &b)| b == b'\n')
            .nth(48)
            .map(|(i, _)| i + 1)
            .unwrap();

        // Edit inside "let var49 = 49;\n"
        let edit = EditDelta {
            start: edit_start + 4,
            deleted: 1,
            inserted: b"x".to_vec(),
        };

        bytes[edit_start + 4] = b'x';

        let region = compute_dirty_region(&bytes, &edit, "javascript");

        // The region should cover just the single statement, not the whole file
        let length = region.range.end - region.range.start;
        assert!(length < 30, "Region too large: {}", length);
        assert!(length > 10, "Region too small: {}", length);
    }
}
