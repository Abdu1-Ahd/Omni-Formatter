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
            // Haskell
            "hs", "lhs",
            // F# — significant whitespace, must use layout-rule pass-through
            "fs", "fsi", "fsx",
            // Elixir / Erlang
            "ex", "exs", "erl", "hrl",
            // OCaml / Elm
            "ml", "mli",
            // Clojure
            "clj", "cljs",
            // R
            "r", "R",
            // Julia
            "jl",
            // Lua — end-based blocks (not braces)
            "lua",
            // Lisp / Scheme
            "lisp", "lsp", "scm", "ss",
        ]
    }

    fn dialect_for_ext(&self, ext: &str) -> Option<&str> {
        Some(match ext {
            "ex" | "exs"                  => "elixir",
            "erl" | "hrl"                 => "erlang",
            "clj" | "cljs"               => "clojure",
            "lisp" | "lsp" | "scm" | "ss" => "lisp",
            "r" | "R"                     => "r",
            "jl"                          => "julia",
            "lua"                         => "lua",
            // Haskell, OCaml, F# — layout rule (significant whitespace)
            _ => "haskell",
        })
    }

    fn format_dialect(
        &self,
        source: &[u8],
        config: &ConfigIR,
        dialect: &str,
    ) -> Result<Vec<u8>, FormatError> {
        format::format_for(source, config, dialect)
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        // Fallback (should not be reached — dialect_for_ext always returns Some)
        format::format_for(source, config, "haskell")
    }
}
