//! Zone Output Stitcher (L-05 mitigation)
//!
//! After each zone is formatted by its matching language module, this module
//! re-assembles the full file output. It:
//!
//! 1. Iterates zones in byte offset order.
//! 2. Copies unmodified host content between zone boundaries.
//! 3. Re-indents each formatted zone to match the host file's indentation level.
//! 4. Inserts the re-indented zone content at the original zone's byte range.
//!
//! # Indentation Alignment
//!
//! Each zone carries `indent_column` — the display column of its opening delimiter.
//! After formatting, the first line of the zone output needs no adjustment.
//! Subsequent lines of the zone output are indented by `indent_column` display columns.
//!
//! Example: a `<script>` at column 2 produces formatted JS where each line is
//! prefixed with 2 spaces to match the host HTML indentation.
//!
//! # Magic Comment Suppression
//!
//! If `zone.suppressed == true`, the zone content is copied byte-for-byte from
//! the source without calling the language module (L-06 mitigation).

use protocol::zone::Zone;

/// The output of a single zone's format operation.
pub struct FormattedZone<'a> {
    /// The zone that was formatted.
    pub zone: &'a Zone,
    /// The formatted bytes for this zone's content.
    /// If `zone.suppressed == true`, this equals the original source bytes.
    pub formatted: Vec<u8>,
}

/// Stitch formatted zone outputs back into a complete file.
///
/// # Arguments
///
/// * `source` — The original source file bytes.
/// * `formatted_zones` — Formatted output for each zone, in byte offset order.
///
/// # Returns
///
/// The fully assembled output bytes, with each zone replaced by its formatted
/// content and all non-zone content preserved verbatim.
pub fn stitch(source: &[u8], formatted_zones: &[FormattedZone<'_>]) -> Vec<u8> {
    if formatted_zones.is_empty() {
        return source.to_vec();
    }

    let mut output = Vec::with_capacity(source.len());
    let mut source_pos = 0usize;

    for fz in formatted_zones {
        let zone_start = fz.zone.range.start;
        let zone_end = fz.zone.range.end;

        // Copy unmodified content before this zone
        if source_pos < zone_start {
            output.extend_from_slice(&source[source_pos..zone_start]);
        }

        // Insert re-indented zone content
        let reindented = reindent(&fz.formatted, fz.zone.indent_column);
        output.extend_from_slice(&reindented);

        source_pos = zone_end;
    }

    // Copy any remaining content after the last zone
    if source_pos < source.len() {
        output.extend_from_slice(&source[source_pos..]);
    }

    output
}

/// Re-indent a formatted zone output to match the host file's indentation level.
///
/// Inserts `indent_column` spaces at the start of each line after the first.
/// The first line is left untouched (its indentation is already correct from
/// the host file context).
///
/// If `indent_column == 0`, the formatted output is returned unchanged.
fn reindent(formatted: &[u8], indent_column: u16) -> Vec<u8> {
    if indent_column == 0 || formatted.is_empty() {
        return formatted.to_vec();
    }

    let indent = " ".repeat(indent_column as usize);
    let mut output = Vec::with_capacity(formatted.len() + formatted.len() / 40);
    let mut first_line = true;

    for line in formatted.split(|&b| b == b'\n') {
        if !first_line {
            output.extend_from_slice(b"\n");
            // Don't indent empty lines
            if !line.is_empty() {
                output.extend_from_slice(indent.as_bytes());
            }
        }
        output.extend_from_slice(line);
        first_line = false;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::zone::{Zone, ZoneKind};
    use protocol::ByteRange;

    fn make_zone(start: usize, end: usize, indent_column: u16) -> Zone {
        Zone {
            language_id: "javascript".to_string(),
            range: ByteRange { start, end },
            indent_column,
            suppressed: false,
            kind: ZoneKind::Language("html".to_string()),
        }
    }

    #[test]
    fn stitch_empty_zones_returns_source() {
        let source = b"hello world";
        let result = stitch(source, &[]);
        assert_eq!(result, source);
    }

    #[test]
    fn stitch_single_zone_replaces_content() {
        let source = b"<script>const x=1</script>";
        let zone = make_zone(8, 17, 0); // "const x=1"
        let fz = FormattedZone {
            zone: &zone,
            formatted: b"const x = 1;".to_vec(),
        };
        let result = stitch(source, &[fz]);
        assert_eq!(result, b"<script>const x = 1;</script>");
    }

    #[test]
    fn reindent_adds_prefix_to_non_first_lines() {
        let formatted = b"line1\nline2\nline3";
        let result = reindent(formatted, 2);
        assert_eq!(result, b"line1\n  line2\n  line3");
    }

    #[test]
    fn reindent_zero_indent_unchanged() {
        let formatted = b"line1\nline2";
        let result = reindent(formatted, 0);
        assert_eq!(result, formatted);
    }
}
