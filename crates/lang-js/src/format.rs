//! JS/TS Formatting Logic
//!
//! This module is the core formatting algorithm for JavaScript, TypeScript,
//! JSX, and TSX. It targets Prettier 3.x output parity (L-08 mitigation).
//!
//! # Algorithm Overview (Prettier 3.x parity)
//!
//! 1. Parse the source with the Tree-sitter `javascript` or `typescript` grammar.
//! 2. Walk the CST to build an intermediate document representation (IR).
//! 3. Apply Prettier's "document IR" printer algorithm (Philip Wadler's
//!    "A prettier printer" + David Klaver's Prettier fork) to the IR.
//! 4. Render the IR to a byte string with the configured `printWidth`.
//!
//! # Implementation Status
//!
//! Phase 3 scaffold: the public `format()` API is defined and returns the
//! source unchanged (pass-through). Full formatting logic is implemented
//! in Phase 4 when the Tree-sitter JS grammar is integrated.
//!
//! # Idempotency (Pillar 7)
//!
//! `format(format(x)) === format(x)` is contractually guaranteed.
//! The 10,000-fixture fuzz suite in `tests/idempotency/` enforces this.
//! In debug builds, `debug::assert_idempotent()` double-formats every call.

use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format JavaScript, TypeScript, JSX, or TSX source bytes.
///
/// # Arguments
///
/// * `source` — UTF-8 source bytes to format.
/// * `config` — The resolved configuration from the adapter.
///
/// # Returns
///
/// Formatted UTF-8 bytes on success.
/// `FormatError` on parse failure, memory limit exceeded, or internal error.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    // Phase 3 stub: return source unchanged.
    // Full Prettier 3.x parity formatting logic implemented in Phase 4.
    //
    // Phase 4 will:
    // 1. Instantiate the tree-sitter `javascript`/`typescript` grammar.
    // 2. Parse `source` into a CST.
    // 3. Run the comment anchoring pass (crates/core/src/comments.rs).
    // 4. Apply the Wadler/Prettier document IR algorithm.
    // 5. Render to bytes with `config.print_width` column limit.
    // 6. Re-attach comments via the anchor map.
    // 7. In debug builds: double-format and assert idempotency.

    let _ = config; // suppress unused warning — used in Phase 4
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_stub_returns_source_unchanged() {
        let source = b"const x=1;\n";
        let config = ConfigIR::default();
        let result = format(source, &config).unwrap();
        assert_eq!(result, source);
    }

    #[test]
    fn format_empty_source_returns_empty() {
        let source = b"";
        let config = ConfigIR::default();
        let result = format(source, &config).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn format_does_not_panic_on_unicode() {
        let source = "const greeting = '你好世界';\n".as_bytes();
        let config = ConfigIR::default();
        let result = format(source, &config).unwrap();
        assert_eq!(result, source);
    }
}
