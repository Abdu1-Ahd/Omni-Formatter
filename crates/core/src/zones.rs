//! Zone Detector — Embedded Language Region Detection (L-05 mitigation)
//!
//! This module walks a Tree-sitter CST and identifies all embedded language
//! regions within a host source file. It runs as a separate pass in the WASM
//! core before any language module is invoked.
//!
//! # Supported Embedded Cases (at launch)
//!
//! - HTML: `<script>` (JS/TS), `<style>` (CSS), inline `style=` attributes
//! - JavaScript/TypeScript: tagged template literals (css``, html``, gql``)
//! - Svelte SFC: `<script>`, `<style>`, template expressions
//! - Vue SFC: `<script>`, `<style>`, `<template>`
//! - Astro: `---` frontmatter (JS/TS), component template
//!
//! # Design Invariant (Pillar 3)
//!
//! Every language module receives ONLY its own zone — never the full multi-language
//! file content. The zone content is passed as an independent byte slice.
//! The indentation context is passed as a separate field.
//!
//! # Implementation Status
//!
//! Phase 3 stub: zone detection logic is scaffolded but requires Tree-sitter
//! grammar integration (added in Phase 4). The public API is stable.

use protocol::zone::{Zone, ZoneKind};
use protocol::ByteRange;

/// The complete result of zone detection for a single source file.
#[derive(Debug)]
pub struct ZoneMap {
    /// All detected embedded language zones, in byte offset order.
    pub zones: Vec<Zone>,
    /// Whether any zones were detected (false = single-language file).
    pub has_embedded_zones: bool,
}

/// Detect embedded language zones in a source file.
///
/// # Arguments
///
/// * `source` — The source file bytes (UTF-8).
/// * `host_language_id` — The primary language of the host file (e.g. "html").
///
/// # Returns
///
/// A `ZoneMap` describing all embedded language regions.
///
/// For single-language files (no embedded zones), returns a `ZoneMap` with
/// `has_embedded_zones = false` and one zone covering the entire file.
pub fn detect_zones(source: &[u8], host_language_id: &str) -> ZoneMap {
    // Phase 3 stub: return a single zone covering the entire file.
    // Full Tree-sitter-based zone detection is implemented in Phase 4.
    let single_zone = Zone {
        language_id: host_language_id.to_string(),
        range: ByteRange {
            start: 0,
            end: source.len(),
        },
        indent_column: 0,
        suppressed: false,
        kind: ZoneKind::HtmlScript, // placeholder for single-language files
    };

    ZoneMap {
        zones: vec![single_zone],
        has_embedded_zones: false,
    }
}

/// Check if a source file is known to contain embedded language zones,
/// based solely on the host language ID. Used as a fast pre-check before
/// running full CST-based zone detection.
pub fn may_have_embedded_zones(host_language_id: &str) -> bool {
    matches!(
        host_language_id,
        "html" | "svelte" | "vue" | "astro" | "javascript" | "typescript"
            | "javascriptreact" | "typescriptreact"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_language_file_returns_one_zone() {
        let source = b"fn main() {}";
        let map = detect_zones(source, "rust");
        assert_eq!(map.zones.len(), 1);
        assert!(!map.has_embedded_zones);
        assert_eq!(map.zones[0].range.start, 0);
        assert_eq!(map.zones[0].range.end, source.len());
    }

    #[test]
    fn html_may_have_embedded_zones() {
        assert!(may_have_embedded_zones("html"));
        assert!(may_have_embedded_zones("svelte"));
        assert!(!may_have_embedded_zones("rust"));
        assert!(!may_have_embedded_zones("go"));
    }
}
