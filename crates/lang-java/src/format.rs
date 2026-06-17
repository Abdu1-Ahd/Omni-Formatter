//! Java / Kotlin Formatting Logic — Stub
//!
//! Grammars removed temporarily to fix workspace conflicts.
//! Currently acts as an identity pass-through.

use crate::adapter::Config;
use protocol::FormatError;

pub fn format(
    source: &[u8],
    _config: &Config,
) -> Result<Vec<u8>, FormatError> {
    Ok(source.to_vec())
}
