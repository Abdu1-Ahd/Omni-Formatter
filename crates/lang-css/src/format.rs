//! CSS/SCSS/Less/HTML formatting logic stub.

use crate::CssDialect;
use protocol::config::ConfigIR;
use protocol::FormatError;

/// Format CSS/SCSS/Less/HTML source bytes.
///
/// # Phase 4 Status
///
/// Pass-through stub. Full Prettier 3.x parity algorithm:
/// 1. Parse with Tree-sitter CSS/SCSS/Less/HTML grammar (dialect-aware).
/// 2. For HTML: invoke zone detector → dispatch JS zones to lang-js.
/// 3. Apply Prettier's CSS printer algorithm.
/// 4. Re-stitch HTML zones.
/// 5. Re-attach comments and assert idempotency.
pub fn format(source: &[u8], config: &ConfigIR, dialect: CssDialect) -> Result<Vec<u8>, FormatError> {
    let _ = (config, dialect);
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_stub_passthrough() {
        let src = b"body { color: red; }\n";
        assert_eq!(format(src, &ConfigIR::default(), CssDialect::Css).unwrap(), src);
    }

    #[test]
    fn html_stub_passthrough() {
        let src = b"<html><body>hello</body></html>\n";
        assert_eq!(format(src, &ConfigIR::default(), CssDialect::Html).unwrap(), src);
    }
}
