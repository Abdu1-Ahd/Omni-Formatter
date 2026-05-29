//! OmniFormatter shared protocol types.
//!
//! All types are `serde`-serializable so they can cross the WASM boundary
//! as JSON strings. Every language module and the WASM core depend on this crate.

pub mod config;
pub mod error;
pub mod zone;

pub use config::ConfigIR;
pub use error::FormatError;
pub use zone::Zone;

use serde::{Deserialize, Serialize};

/// A request sent from the extension host (TypeScript) into the WASM core.
///
/// All positions are in **UTF-8 byte offsets**. The extension host is
/// responsible for converting VS Code's UTF-16 code unit positions to
/// UTF-8 before constructing this struct (see L-14 mitigation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequest {
    /// UTF-8 encoded source text.
    pub source: Vec<u8>,

    /// VS Code language identifier (e.g. `"typescript"`, `"python"`).
    pub language_id: String,

    /// Resolved configuration, already translated by the config adapter.
    pub config: ConfigIR,

    /// Optional: if set, format only the range [start, end) in byte offsets.
    /// The WASM core expands this to the nearest complete syntactic unit (L-15).
    pub range: Option<ByteRange>,

    /// Optional: serialised previous Tree-sitter tree for incremental re-parse.
    /// Present only for format-on-type requests (L-13).
    pub previous_tree: Option<Vec<u8>>,

    /// Optional: the edit that triggered format-on-type.
    /// Present only when `previous_tree` is Some.
    pub edit: Option<EditDelta>,
}

/// A response from the WASM core back to the extension host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResponse {
    /// Minimal set of text edits to transform the source into the formatted output.
    /// Empty if the document is already formatted.
    pub edits: Vec<TextEdit>,

    /// Human-readable description of what formatted the document
    /// (e.g. `"lang-js 0.1.0 (Prettier 3.x compat)"`).
    pub formatter_chain: String,

    /// Whether the format operation was a no-op (source was already formatted).
    pub is_noop: bool,
}

/// A contiguous byte range within the source document.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ByteRange {
    /// Inclusive start offset (UTF-8 bytes).
    pub start: usize,
    /// Exclusive end offset (UTF-8 bytes).
    pub end: usize,
}

/// A single text replacement edit.
///
/// Corresponds to VS Code's `vscode.TextEdit` after the extension host
/// converts byte offsets back to UTF-16 positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// The range of the original source to replace.
    pub range: ByteRange,
    /// The replacement text (UTF-8).
    pub new_text: String,
}

/// An incremental edit delta for format-on-type (L-13 mitigation).
///
/// Describes the single keypress/paste edit that triggered the format request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDelta {
    /// Byte offset where the edit starts.
    pub start: usize,
    /// Number of bytes deleted from the original source starting at `start`.
    pub deleted: usize,
    /// Bytes inserted at `start` (the new text typed/pasted).
    pub inserted: Vec<u8>,
}
