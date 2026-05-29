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
///
/// # Phase 3 Status
///
/// Returns a stub region covering ±5 lines around the edit offset.
/// Full Tree-sitter incremental parse in Phase 4.
pub fn compute_dirty_region(
    source: &[u8],
    edit: &EditDelta,
    language_id: &str,
) -> DirtyRegion {
    // Phase 3 stub: expand to ±5 lines around the edit offset.
    // Full CST-based expansion implemented in Phase 4.
    let start = expand_to_line_start(source, edit.start, 5);
    let end = expand_to_line_end(source, edit.start + edit.inserted.len(), 5);

    DirtyRegion {
        range: ByteRange { start, end },
        language_id: language_id.to_string(),
    }
}

/// Walk backwards from `offset` by up to `lines` newlines.
fn expand_to_line_start(source: &[u8], offset: usize, lines: usize) -> usize {
    let mut pos = offset.min(source.len().saturating_sub(1));
    let mut lines_seen = 0;

    while pos > 0 {
        pos -= 1;
        if source[pos] == b'\n' {
            lines_seen += 1;
            if lines_seen >= lines {
                return pos + 1;
            }
        }
    }
    0
}

/// Walk forwards from `offset` by up to `lines` newlines.
fn expand_to_line_end(source: &[u8], offset: usize, lines: usize) -> usize {
    let mut pos = offset.min(source.len());
    let mut lines_seen = 0;

    while pos < source.len() {
        if source[pos] == b'\n' {
            lines_seen += 1;
            if lines_seen >= lines {
                return pos;
            }
        }
        pos += 1;
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_region_within_source_bounds() {
        let source = b"line1\nline2\nline3\nline4\nline5\nline6\nline7\n";
        let edit = EditDelta {
            start: 20,
            deleted: 0,
            inserted: b"x".to_vec(),
        };
        let region = compute_dirty_region(source, &edit, "javascript");
        assert!(region.range.start <= 20);
        assert!(region.range.end >= 21);
        assert!(region.range.end <= source.len());
    }

    #[test]
    fn dirty_region_clamps_to_source_start() {
        let source = b"short source";
        let edit = EditDelta {
            start: 0,
            deleted: 0,
            inserted: b"x".to_vec(),
        };
        let region = compute_dirty_region(source, &edit, "rust");
        assert_eq!(region.range.start, 0);
    }
}
