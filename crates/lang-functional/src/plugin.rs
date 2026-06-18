use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// FunctionalPlugin plugin
pub struct FunctionalPlugin;

impl LanguagePlugin for FunctionalPlugin {
    fn name(&self) -> &str {
        "lang-functional"
    }

    fn extensions(&self) -> &[&str] {
        &[
            "hs", "lhs", "ex", "exs", "erl", "hrl", "ml", "mli", "clj", "cljs", "r", "R", "jl",
            "lisp", "lsp", "scm", "ss",
        ]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        format::format(source, config)
    }
}
