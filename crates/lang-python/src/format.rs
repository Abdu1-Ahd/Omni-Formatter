//! Python formatting logic — Black 24.x parity (stub).

use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format Python source bytes.
///
/// # Phase 4 Status
///
/// Pass-through stub. Full Black-parity algorithm:
/// 1. Parse with Tree-sitter Python grammar.
/// 2. Apply Black's "formatting modes": string normalization,
///    magic trailing comma, line-length enforcement.
/// 3. Re-attach comments via anchor map.
/// 4. Assert idempotency in debug builds.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let _ = config;
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_source_unchanged() {
        let src = b"x = 1\n";
        assert_eq!(format(src, &ConfigIR::default()).unwrap(), src);
    }
}
