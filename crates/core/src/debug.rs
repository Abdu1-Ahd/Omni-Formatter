//! Debug Double-Format Idempotency Check (L-09 mitigation)
//!
//! In debug builds (compiled with `--features debug`), the WASM core double-formats
//! every file after the primary format pass. If `format(format(x)) != format(x)`,
//! this module panics with a diff showing the divergence.
//!
//! This check is DISABLED in release builds for performance. It runs only in:
//! - Local development builds with `--features debug`.
//! - CI test runs (via `cargo test --features debug`).
//!
//! Non-idempotent output is a silent, catastrophic bug that causes infinite format
//! loops in VS Code. Catching it in development prevents it from reaching users.
//!
//! # Idempotency Contract (Pillar 7)
//!
//! `format(format(x)) === format(x)` is a contractual guarantee.
//! Every language module must pass the 10,000-fixture idempotency fuzz suite
//! before release. This module is the in-process enforcement layer.

/// Check that formatting is idempotent: `format(result) == result`.
///
/// # Arguments
///
/// * `first_pass` — The output of the first format pass.
/// * `second_pass` — The output of formatting `first_pass` again.
/// * `language_id` — For use in the panic message.
///
/// # Panics (debug builds with --features debug only)
///
/// Panics with a detailed diff if `first_pass != second_pass`.
///
/// # Release Builds
///
/// This function is a no-op in release builds.
#[cfg(feature = "debug")]
pub fn assert_idempotent(first_pass: &[u8], second_pass: &[u8], language_id: &str) {
    if first_pass != second_pass {
        let diff = compute_diff(first_pass, second_pass);
        panic!(
            "IDEMPOTENCY VIOLATION in language '{}': format(format(x)) != format(x)\n\n{}",
            language_id, diff
        );
    }
}

/// No-op in release builds.
#[cfg(not(feature = "debug"))]
#[inline(always)]
pub fn assert_idempotent(_first: &[u8], _second: &[u8], _language_id: &str) {}

/// Compute a simple line-level diff between two byte slices.
///
/// Returns a string showing which lines differ, for use in panic messages.
#[cfg(feature = "debug")]
fn compute_diff(a: &[u8], b: &[u8]) -> String {
    let a_str = String::from_utf8_lossy(a);
    let b_str = String::from_utf8_lossy(b);

    let a_lines: Vec<&str> = a_str.lines().collect();
    let b_lines: Vec<&str> = b_str.lines().collect();

    let mut diff = String::new();
    let max_lines = a_lines.len().max(b_lines.len());

    for i in 0..max_lines.min(50) {
        let a_line = a_lines.get(i).copied().unwrap_or("<missing>");
        let b_line = b_lines.get(i).copied().unwrap_or("<missing>");

        if a_line != b_line {
            diff.push_str(&format!("Line {}: first pass: {:?}\n", i + 1, a_line));
            diff.push_str(&format!("Line {}:second pass: {:?}\n\n", i + 1, b_line));
        }
    }

    if max_lines > 50 {
        diff.push_str("... (diff truncated at 50 lines) ...\n");
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_idempotent_passes_for_equal_output() {
        let output = b"const x = 1;\n".to_vec();
        // Should not panic
        assert_idempotent(&output, &output, "javascript");
    }

    #[cfg(not(feature = "debug"))]
    #[test]
    fn assert_idempotent_noop_in_release() {
        // In release builds, this must never panic regardless of input
        let first = b"one output".to_vec();
        let second = b"different output".to_vec();
        assert_idempotent(&first, &second, "javascript"); // no panic
    }
}
