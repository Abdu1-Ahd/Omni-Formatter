//! WASM linear memory management constants and guards (L-01 mitigation).
//!
//! WASM uses a flat linear memory model. The limits here are enforced at
//! the extension host before any data enters WASM, and also as build-time
//! configuration for the WASM linker.
//!
//! # Memory Budget
//!
//! | Allocation | Size |
//! |---|---|
//! | Initial WASM heap | 16 MB |
//! | Maximum WASM heap | 64 MB |
//! | Maximum source file | 10 MB |
//! | Per-request arena initial cap | 64 KB |
//!
//! The 64MB maximum is sufficient for any practical source file. A 10MB
//! TypeScript file with deeply nested ASTs requires at most ~30MB of
//! Tree-sitter node memory.

/// Maximum source file size in bytes (L-01 mitigation).
///
/// Files above this limit are rejected at the extension host before being
/// sent to the WASM core. The WASM `format()` function also checks this
/// as a defence-in-depth guard.
pub const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024; // 10 MB

/// WASM initial memory size in bytes.
///
/// Passed to `wasm-pack` via `--` `--initial-memory=16777216`.
/// Stored here as documentation. The actual limit is set in `build-wasm.sh`.
pub const WASM_INITIAL_MEMORY: usize = 16 * 1024 * 1024; // 16 MB

/// WASM maximum memory size in bytes.
///
/// Passed to `wasm-pack` via `--` `--max-memory=67108864`.
/// Stored here as documentation. The actual limit is set in `build-wasm.sh`.
pub const WASM_MAX_MEMORY: usize = 64 * 1024 * 1024; // 64 MB

/// Check that `size_bytes` does not exceed `MAX_SOURCE_BYTES`.
///
/// Returns `Ok(())` if within limits, `Err(size_bytes)` if exceeded.
pub fn check_source_size(size_bytes: usize) -> Result<(), usize> {
    if size_bytes > MAX_SOURCE_BYTES {
        Err(size_bytes)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_source_size_accepts_valid() {
        assert!(check_source_size(0).is_ok());
        assert!(check_source_size(1024).is_ok());
        assert!(check_source_size(MAX_SOURCE_BYTES).is_ok());
    }

    #[test]
    fn check_source_size_rejects_oversized() {
        assert!(check_source_size(MAX_SOURCE_BYTES + 1).is_err());
    }
}
