//! C/C++ Formatting Logic — Stub
//!
//! Grammars removed temporarily to fix workspace conflicts.
//! Currently acts as an identity pass-through.

use crate::{adapter::CConfig, CDialect};
use protocol::FormatError;

pub fn format(source: &[u8], _config: &CConfig, _dialect: CDialect) -> Result<Vec<u8>, FormatError> {
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_idempotent() {
        let src = b"int add(int a, int b) {\nreturn a + b;\n}\n";
        let config = CConfig::default();
        let first = format(src, &config, CDialect::C).unwrap();
        let second = format(&first, &config, CDialect::C).unwrap();
        assert_eq!(first, second);
    }
}
