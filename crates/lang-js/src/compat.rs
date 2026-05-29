//! Prettier 3.x Compatibility Layer (L-08 mitigation)
//!
//! In "opinionated" mode (the default), this module's output must be
//! byte-for-byte identical to Prettier 3.x for all formatting options in
//! the standard Prettier test suite.
//!
//! # Compat Mode
//!
//! Strict compat mode is the default (`ModuleMode::Opinionated`).
//! In strict compat mode:
//! - The formatting algorithm is a re-implementation of Prettier's algorithm in Rust.
//! - Output is tested against Prettier 3.x's own fixture corpus.
//! - If any divergence is detected, the CI job fails and a fix PR is opened.
//!
//! # Weekly Compat CI
//!
//! A GitHub Actions job runs weekly that:
//! 1. Downloads the latest Prettier 3.x release.
//! 2. Formats the fixture corpus with both Prettier 3.x and OmniFormatter.
//! 3. Asserts byte-for-byte equality on every fixture file.
//! 4. Opens a fix PR automatically if divergence is found.
//!
//! # Implementation Status
//!
//! Phase 3 scaffold: compat mode detection and version string implemented.
//! Full algorithm in Phase 4.

use protocol::config::{ConfigIR, ModuleMode};

/// The Prettier version this module targets for compat.
pub const PRETTIER_COMPAT_VERSION: &str = "3.x";

/// Returns whether this config is in strict Prettier compat mode.
pub fn is_strict_compat_mode(config: &ConfigIR) -> bool {
    config.mode == ModuleMode::Opinionated
}

/// Returns the formatter chain label for the status bar.
///
/// In compat mode: `"lang-js 0.1.0 (Prettier 3.x compat)"`
/// In advanced mode: `"lang-js 0.1.0 (advanced mode)"`
pub fn formatter_chain_label(config: &ConfigIR) -> String {
    let version = env!("CARGO_PKG_VERSION");
    if is_strict_compat_mode(config) {
        format!("lang-js {} (Prettier {} compat)", version, PRETTIER_COMPAT_VERSION)
    } else {
        format!("lang-js {} (advanced mode)", version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::config::ModuleMode;

    #[test]
    fn opinionated_mode_is_strict_compat() {
        let mut config = ConfigIR::default();
        config.mode = ModuleMode::Opinionated;
        assert!(is_strict_compat_mode(&config));
    }

    #[test]
    fn advanced_mode_is_not_strict_compat() {
        let mut config = ConfigIR::default();
        config.mode = ModuleMode::Advanced;
        assert!(!is_strict_compat_mode(&config));
    }

    #[test]
    fn label_includes_prettier_version_in_compat_mode() {
        let config = ConfigIR::default();
        let label = formatter_chain_label(&config);
        assert!(label.contains("Prettier 3.x compat"));
    }
}
