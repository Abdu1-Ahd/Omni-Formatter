//! Java / Kotlin Formatting Logic — Stub
//!
//! Grammars removed temporarily to fix workspace conflicts.
//! Currently acts as an identity pass-through.

use crate::{adapter::JavaConfig, JvmDialect};
use protocol::FormatError;

pub fn format(
    source: &[u8],
    _config: &JavaConfig,
    _dialect: JvmDialect,
) -> Result<Vec<u8>, FormatError> {
    Ok(source.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_format_idempotent() {
        let src = b"class Foo {\n    void bar() {\n        return;\n    }\n}\n";
        let first = format(src, &JavaConfig::default(), JvmDialect::Java).unwrap();
        let second = format(&first, &JavaConfig::default(), JvmDialect::Java).unwrap();
        assert_eq!(first, second);
    }
}
