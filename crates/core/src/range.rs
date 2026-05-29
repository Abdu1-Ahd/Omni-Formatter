//! Range Expansion to Nearest Complete Syntactic Unit (L-15 mitigation)
//!
//! When a range format request is received (VS Code's "Format Selection" command),
//! the user's selection may land mid-statement or mid-block. Formatting an
//! incomplete AST node produces broken output.
//!
//! # Algorithm
//!
//! 1. Walk the Tree-sitter CST from the start of the requested range.
//! 2. Find the smallest CST node whose span fully contains the requested range.
//! 3. Walk UP the CST from that node until reaching a "formattable boundary":
//!    a statement, declaration, function definition, or block.
//! 4. Return the byte range of that boundary node.
//!
//! # Error Case
//!
//! If the selection is inside an ERROR node (parse error), range formatting
//! returns a no-op and sets a status bar warning.
//!
//! # Implementation Status
//!
//! Phase 3 scaffold. Tree-sitter CST traversal implemented in Phase 4.

use protocol::ByteRange;

/// The result of range expansion.
#[derive(Debug, Clone)]
pub struct ExpandedRange {
    /// The expanded range covering the nearest complete syntactic unit.
    pub range: ByteRange,
    /// Whether the expansion found a valid syntactic unit.
    pub is_valid: bool,
    /// If `!is_valid`, a human-readable reason (e.g. "selection contains a syntax error").
    pub error: Option<String>,
}

/// Expand a requested format range to the nearest complete syntactic unit.
///
/// # Arguments
///
/// * `source` — The source bytes.
/// * `requested` — The user's selected range (may be mid-statement).
///
/// # Returns
///
/// An `ExpandedRange` covering the smallest complete syntactic unit that
/// fully contains the requested range.
///
/// # Phase 3 Status
///
/// Returns the requested range expanded to full lines (stub).
/// Full Tree-sitter CST traversal implemented in Phase 4.
pub fn expand_to_nearest_unit(source: &[u8], requested: ByteRange) -> ExpandedRange {
    // Phase 3 stub: expand to full lines.
    let start = expand_to_line_start(source, requested.start);
    let end = expand_to_line_end(source, requested.end);

    ExpandedRange {
        range: ByteRange { start, end },
        is_valid: true,
        error: None,
    }
}

/// Walk backwards to the start of the line containing `offset`.
fn expand_to_line_start(source: &[u8], offset: usize) -> usize {
    let clamped = offset.min(source.len());
    let mut pos = clamped;
    while pos > 0 && source[pos - 1] != b'\n' {
        pos -= 1;
    }
    pos
}

/// Walk forwards to the end of the line containing `offset`.
fn expand_to_line_end(source: &[u8], offset: usize) -> usize {
    let mut pos = offset.min(source.len());
    while pos < source.len() && source[pos] != b'\n' {
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_covers_full_line() {
        let source = b"const x = 1;\nconst y = 2;\n";
        // Select just "x" at offset 6..7
        let result = expand_to_nearest_unit(source, ByteRange { start: 6, end: 7 });
        assert!(result.is_valid);
        // Should expand to cover the entire first line
        assert_eq!(result.range.start, 0);
        assert_eq!(result.range.end, 13); // up to the \n
    }

    #[test]
    fn expand_handles_empty_selection() {
        let source = b"line1\nline2\n";
        let result = expand_to_nearest_unit(source, ByteRange { start: 3, end: 3 });
        assert!(result.is_valid);
        assert_eq!(result.range.start, 0);
    }

    #[test]
    fn expand_clamps_at_source_boundaries() {
        let source = b"hello";
        let result = expand_to_nearest_unit(source, ByteRange { start: 0, end: 100 });
        assert!(result.is_valid);
        assert!(result.range.end <= source.len());
    }
}
