//! Unicode Display Column Width Calculation (L-14 mitigation)
//!
//! Print width limits (e.g. `printWidth: 80`) are enforced in display columns,
//! not in character count, byte count, or UTF-16 code units.
//!
//! Display column rules:
//! - ASCII: 1 display column per character.
//! - Latin/Cyrillic/Arabic (non-CJK BMP): 1 display column.
//! - CJK Unified Ideographs (U+4E00–U+9FFF etc.): 2 display columns.
//! - Combining characters (accents, diacritics): 0 display columns.
//! - Emoji sequences (ZWJ sequences): the width of the first visible character.
//! - Null character: 0 display columns.
//! - Tab: 1 display column (caller is responsible for tab-stop expansion).
//!
//! This matches the behaviour of the `unicode-width` Rust crate.

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

/// Compute the display column width of a string (UTF-8).
///
/// Returns the number of display columns the string occupies on a terminal
/// with standard CJK-aware column counting.
///
/// # Examples
///
/// ```rust
/// assert_eq!(display_width("hello"), 5);
/// assert_eq!(display_width("你好"), 4);   // CJK: 2 columns each
/// assert_eq!(display_width("e\u{0301}"), 1); // 'é' = e + combining accent
/// ```
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Compute the display column width of a single Unicode scalar value.
///
/// Returns 0 for combining characters and control characters.
/// Returns 2 for CJK wide characters.
/// Returns 1 for all other printable characters.
pub fn char_display_width(c: char) -> usize {
    UnicodeWidthChar::width(c).unwrap_or(0)
}

/// Find the UTF-8 byte offset where a line exceeds `max_columns` display columns.
///
/// Returns `None` if the line does not exceed `max_columns`.
/// Returns `Some(byte_offset)` at the first character that would push the line over.
///
/// Used by language modules to determine where to insert line breaks.
pub fn find_line_break_point(line: &str, max_columns: u16) -> Option<usize> {
    let max = max_columns as usize;
    let mut col = 0usize;
    let mut byte_offset = 0usize;

    for c in line.chars() {
        let w = char_display_width(c);
        if col + w > max {
            return Some(byte_offset);
        }
        col += w;
        byte_offset += c.len_utf8();
    }

    None
}

/// Compute the display column of a UTF-8 byte offset within a line.
///
/// Returns the display column position of the character at `byte_offset`.
/// Panics in debug builds if `byte_offset` is not on a character boundary.
pub fn byte_offset_to_display_column(line: &str, byte_offset: usize) -> usize {
    let slice = &line[..byte_offset.min(line.len())];
    UnicodeWidthStr::width(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn cjk_double_width() {
        // CJK Unified Ideograph = 2 display columns each
        assert_eq!(display_width("你好"), 4);
        assert_eq!(display_width("世界"), 4);
    }

    #[test]
    fn combining_character_zero_width() {
        // 'e' + combining acute accent U+0301 = 'é' = 1 display column
        assert_eq!(display_width("e\u{0301}"), 1);
    }

    #[test]
    fn find_break_point_ascii() {
        let line = "a".repeat(100);
        assert_eq!(find_line_break_point(&line, 80), Some(80));
    }

    #[test]
    fn find_break_point_cjk() {
        // 40 CJK chars = 80 display columns — should not break at 80
        let line: String = "字".repeat(40);
        assert_eq!(find_line_break_point(&line, 80), None);

        // 41 CJK chars = 82 display columns — should break at byte 40*3=120
        let line: String = "字".repeat(41);
        assert_eq!(find_line_break_point(&line, 80), Some(120));
    }

    #[test]
    fn byte_offset_to_column_ascii() {
        let line = "hello world";
        assert_eq!(byte_offset_to_display_column(line, 5), 5);
    }

    #[test]
    fn byte_offset_to_column_cjk() {
        // "你" is 3 bytes, display width 2. Byte offset 3 = after first CJK char = column 2.
        let line = "你好";
        assert_eq!(byte_offset_to_display_column(line, 3), 2);
        assert_eq!(byte_offset_to_display_column(line, 6), 4);
    }
}
