//! LanguagePlugin trait — the single interface all language crates implement.
//!
//! Adding a new language to OmniFormatter:
//!   1. Create a new crate `lang-<name>`.
//!   2. Implement `LanguagePlugin` (this file).
//!   3. Call `registry.register(Box::new(MyPlugin))` in `core::registry::default_registry`.
//!
//! No WASM, no hardcoded `match` on extensions — purely trait dispatch.

use crate::{config::ConfigIR, FormatError};

/// A language formatting plugin.
///
/// Each implementation handles one language family (JS/TS, Python, Rust, …).
/// Extensions that a plugin claims are mapped into the global `PluginRegistry`.
pub trait LanguagePlugin: Send + Sync {
    /// The human-readable name of this plugin (e.g. `"lang-js"`).
    fn name(&self) -> &str;

    /// Canonical file extensions this plugin handles, **without** the leading dot.
    /// e.g. `&["js", "ts", "jsx", "tsx", "mjs", "cjs"]`
    fn extensions(&self) -> &[&str];

    /// Format `source` bytes according to `config`.
    ///
    /// On success, returns formatted UTF-8 bytes.
    /// On failure, returns a [`FormatError`] — **never panics**.
    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError>;

    /// Optional: format `source` with an explicit dialect tag (used by lang-css for
    /// CSS vs SCSS vs Less vs HTML). Defaults to calling `format` with no dialect.
    fn format_dialect(&self, source: &[u8], config: &ConfigIR, _dialect: &str) -> Result<Vec<u8>, FormatError> {
        self.format(source, config)
    }

    /// Infer the dialect string from a file extension.
    /// Used when the caller only has an extension, not a VS Code languageId.
    /// Return `None` if this plugin does not use dialects.
    fn dialect_for_ext(&self, _ext: &str) -> Option<&str> {
        None
    }
}
