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
    if !may_have_embedded_zones(host_language_id) {
        return ZoneMap {
            zones: vec![single_zone(source, host_language_id)],
            has_embedded_zones: false,
        };
    }

    if host_language_id == "html" {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_html::language()).expect("Failed to load HTML grammar");
        let tree = parser.parse(source, None).expect("Failed to parse HTML");
        
        let mut zones = Vec::new();
        let mut cursor = tree.walk();
        let mut has_embedded = false;
        
        traverse_html(&mut cursor, source, &mut zones, &mut has_embedded);
        
        if !has_embedded {
            return ZoneMap {
                zones: vec![single_zone(source, host_language_id)],
                has_embedded_zones: false,
            };
        }
        
        return ZoneMap {
            zones,
            has_embedded_zones: true,
        };
    }
    
    // Phase 3 stub for other languages (svelte, vue, etc)
    ZoneMap {
        zones: vec![single_zone(source, host_language_id)],
        has_embedded_zones: false,
    }
}

fn single_zone(source: &[u8], host_language_id: &str) -> Zone {
    Zone {
        language_id: host_language_id.to_string(),
        range: ByteRange {
            start: 0,
            end: source.len(),
        },
        indent_column: 0,
        suppressed: false,
        kind: ZoneKind::Language(host_language_id.to_string()),
    }
}

fn traverse_html(cursor: &mut tree_sitter::TreeCursor, source: &[u8], zones: &mut Vec<Zone>, has_embedded: &mut bool) {
    loop {
        let node = cursor.node();
        if node.kind() == "script_element" {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                if child.kind() == "raw_text" {
                    *has_embedded = true;
                    zones.push(Zone {
                        language_id: "javascript".to_string(),
                        range: ByteRange { start: child.start_byte(), end: child.end_byte() },
                        indent_column: child.start_position().column as u16,
                        suppressed: false,
                        kind: ZoneKind::Language("javascript".to_string()),
                    });
                }
            }
        } else if node.kind() == "style_element" {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                if child.kind() == "raw_text" {
                    *has_embedded = true;
                    zones.push(Zone {
                        language_id: "css".to_string(),
                        range: ByteRange { start: child.start_byte(), end: child.end_byte() },
                        indent_column: child.start_position().column as u16,
                        suppressed: false,
                        kind: ZoneKind::Language("css".to_string()),
                    });
                }
            }
        }

        if cursor.goto_first_child() {
            traverse_html(cursor, source, zones, has_embedded);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            break;
        }
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
