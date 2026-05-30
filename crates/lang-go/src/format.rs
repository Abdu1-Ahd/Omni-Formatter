//! Go formatting logic — gofmt parity stub.

use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format Go source bytes.
///
/// # Phase 4 Status
///
/// Pass-through stub. Full gofmt-parity algorithm:
/// 1. Parse with Tree-sitter Go grammar.
/// 2. Apply gofmt's strict tab-based indentation.
/// 3. Organise imports (goimports style).
/// 4. Enforce consistent blank-line rules between declarations.
/// 5. Re-attach comments and assert idempotency.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    let _ = config;
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_source_unchanged() {
        let src = b"package main\n\nfunc main() {}\n";
        assert_eq!(format(src, &ConfigIR::default()).unwrap(), src);
    }
}
