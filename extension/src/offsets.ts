/**
 * UTF-16 ↔ UTF-8 Byte Offset Conversion (L-14 mitigation)
 *
 * VS Code uses UTF-16 code unit offsets in all position APIs.
 * Rust/WASM operates on UTF-8 bytes.
 *
 * This module converts between the two at the extension host boundary,
 * before any data enters the WASM core. Conversion is O(edit region size),
 * not O(file size), because we only convert the affected region's offsets.
 *
 * Unicode handling:
 * - ASCII characters: 1 UTF-8 byte = 1 UTF-16 code unit (fast path)
 * - BMP characters (U+0080–U+FFFF): 2–3 UTF-8 bytes = 1 UTF-16 code unit
 * - Supplementary characters (U+10000+): 4 UTF-8 bytes = 2 UTF-16 code units (surrogate pair)
 * - CJK characters: counted as 1 UTF-16 code unit, but 3 UTF-8 bytes
 * - The `unicode-width` crate handles DISPLAY columns separately (Rust side)
 */

/**
 * Convert a VS Code UTF-16 code unit position offset to a UTF-8 byte offset.
 *
 * @param text The full document text as a JavaScript string (UTF-16).
 * @param utf16Offset The UTF-16 code unit offset (0-indexed).
 * @returns The corresponding UTF-8 byte offset.
 */
export function toUtf8ByteOffset(text: string, utf16Offset: number): number {
  let utf8Bytes = 0;
  let utf16CodeUnits = 0;

  for (let i = 0; i < text.length && utf16CodeUnits < utf16Offset; i++) {
    const codePoint = text.codePointAt(i)!;

    if (codePoint <= 0x7f) {
      // ASCII: 1 UTF-8 byte, 1 UTF-16 code unit
      utf8Bytes += 1;
      utf16CodeUnits += 1;
    } else if (codePoint <= 0x7ff) {
      // 2-byte UTF-8, 1 UTF-16 code unit
      utf8Bytes += 2;
      utf16CodeUnits += 1;
    } else if (codePoint <= 0xffff) {
      // 3-byte UTF-8, 1 UTF-16 code unit
      utf8Bytes += 3;
      utf16CodeUnits += 1;
    } else {
      // 4-byte UTF-8, 2 UTF-16 code units (surrogate pair)
      utf8Bytes += 4;
      utf16CodeUnits += 2;
      i++; // skip the low surrogate
    }
  }

  return utf8Bytes;
}

/**
 * Convert a UTF-8 byte offset back to a VS Code UTF-16 code unit offset.
 *
 * @param text The full document text as a JavaScript string (UTF-16).
 * @param utf8ByteOffset The UTF-8 byte offset (0-indexed).
 * @returns The corresponding UTF-16 code unit offset.
 */
export function toUtf16CodeUnitOffset(text: string, utf8ByteOffset: number): number {
  let utf8Bytes = 0;
  let utf16CodeUnits = 0;

  for (let i = 0; i < text.length && utf8Bytes < utf8ByteOffset; i++) {
    const codePoint = text.codePointAt(i)!;

    if (codePoint <= 0x7f) {
      utf8Bytes += 1;
      utf16CodeUnits += 1;
    } else if (codePoint <= 0x7ff) {
      utf8Bytes += 2;
      utf16CodeUnits += 1;
    } else if (codePoint <= 0xffff) {
      utf8Bytes += 3;
      utf16CodeUnits += 1;
    } else {
      utf8Bytes += 4;
      utf16CodeUnits += 2;
      i++; // skip the low surrogate
    }
  }

  return utf16CodeUnits;
}

/**
 * Convert a VS Code Position (line, character) to a UTF-8 byte offset
 * within the given document text.
 *
 * @param text The full document text.
 * @param line 0-indexed line number.
 * @param character 0-indexed UTF-16 code unit within the line.
 * @returns The UTF-8 byte offset from the start of the document.
 */
export function positionToUtf8ByteOffset(
  text: string,
  line: number,
  character: number
): number {
  let currentLine = 0;
  let lineStartUtf8 = 0;

  // Walk to the target line
  const utf8 = Buffer.from(text, "utf8");
  let utf8Pos = 0;

  while (utf8Pos < utf8.length && currentLine < line) {
    if (utf8[utf8Pos] === 0x0a /* \n */) {
      currentLine++;
      lineStartUtf8 = utf8Pos + 1;
    }
    utf8Pos++;
  }

  // From lineStartUtf8, advance `character` UTF-16 code units
  const lineText = text.split("\n")[line] ?? "";
  const characterByteOffset = toUtf8ByteOffset(lineText, character);

  return lineStartUtf8 + characterByteOffset;
}
