//! Rust formatting logic — rustfmt parity stub.

use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format Rust source bytes.
///
/// # Phase 4 Status
///
/// Pass-through stub. Full rustfmt-parity algorithm:
/// 1. Parse with Tree-sitter Rust grammar.
/// 2. Apply rustfmt's block/chain formatting rules.
/// 3. Handle `// rustfmt::skip` suppression via comment anchor map.
/// 4. Re-attach comments and assert idempotency.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let _ = config;
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_source_unchanged() {
        let src = b"fn main() {}\n";
        assert_eq!(format(src, &ConfigIR::default()).unwrap(), src);
    }
}
