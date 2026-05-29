//! Zone — an embedded language region within a multi-language source file.
//!
//! The Zone Detector (crates/core/src/zones.rs) walks the Tree-sitter CST
//! and produces a Vec<Zone> before any language module is invoked (L-05 mitigation).
//!
//! Each Zone is passed to its matching language module independently.
//! The formatted output of each Zone is re-indented and stitched by
//! crates/core/src/stitch.rs.

use serde::{Deserialize, Serialize};

use crate::ByteRange;

/// A contiguous embedded language region within a host source file.
///
/// Example: the `<script>` block inside an HTML file is a JavaScript Zone.
/// Example: a `` css`...` `` tagged template literal in TypeScript is a CSS Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    /// The language identifier of this zone's content.
    /// Matches a VS Code `languageId` (e.g. `"javascript"`, `"css"`).
    pub language_id: String,

    /// Byte range in the host source file that this zone occupies.
    /// Covers the content only — not the delimiters.
    /// All offsets are UTF-8 byte offsets.
    pub range: ByteRange,

    /// Display column of the opening delimiter of this zone in the host file.
    /// Used by the stitcher to re-indent the formatted zone content (L-05).
    pub indent_column: u16,

    /// Whether a magic suppression comment immediately precedes this zone
    /// (`// prettier-ignore`, `# fmt: off`, `// rustfmt::skip`).
    /// If true, the zone content is emitted verbatim without formatting (L-06).
    pub suppressed: bool,

    /// The kind of embedding that created this zone.
    pub kind: ZoneKind,
}

/// The structural reason a zone was created.
///
/// Used by the stitcher to handle delimiter re-insertion correctly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    /// `<script>` or `<script type="...">` inside HTML/Svelte/Vue/Astro.
    HtmlScript,
    /// `<style>` or `<style lang="...">` inside HTML/Svelte/Vue/Astro.
    HtmlStyle,
    /// Inline `style="..."` attribute value (CSS subset).
    InlineStyle,
    /// Tagged template literal: `` css`...` ``, `` html`...` ``, `` gql`...` ``.
    TaggedTemplate { tag: String },
    /// Svelte `<template>` block.
    SvelteTemplate,
    /// Vue `<template>` block.
    VueTemplate,
    /// Astro `---` frontmatter block (JS/TS).
    AstroFrontmatter,
    /// A `py"..."` or `py"""..."""` embedded string in Rust source.
    RustEmbeddedPython,
}
