#!/usr/bin/env pwsh
# scaffold_crates.ps1
# Generates all remaining lang-* crate scaffolds for OmniFormatter v0.2.0
# Run from: OmniFormatter root

$ROOT = "crates"

# Format: name, description, formatter_target, language_id, aliases_array, dialect_enum_variants, grammar_deps
$CRATES = @(
    @{
        name = "lang-csharp"
        desc = "C# / F# language module"
        formatter = "dotnet format / fantomas"
        primary_id = "csharp"
        aliases = @("fsharp", ".cs", ".fs", ".fsi", ".fsx")
        dialects = @("CSharp", "FSharp")
        id_map = @{ csharp = "CSharp"; fsharp = "FSharp" }
        grammar_deps = @("tree-sitter-c-sharp")
        fn_name = "format_csharp"
    }
    @{
        name = "lang-data"
        desc = "JSON / YAML / TOML / XML / INI language module"
        formatter = "prettier / taplo"
        primary_id = "json"
        aliases = @("yaml", "toml", "xml", "ini", ".json", ".yaml", ".yml", ".toml", ".xml", ".ini")
        dialects = @("Json", "Yaml", "Toml", "Xml", "Ini")
        id_map = @{ json = "Json"; yaml = "Yaml"; toml = "Toml"; xml = "Xml"; ini = "Ini" }
        grammar_deps = @("tree-sitter-json", "tree-sitter-yaml", "tree-sitter-toml", "tree-sitter-xml")
        fn_name = "format_data"
    }
    @{
        name = "lang-shell"
        desc = "Bash / PowerShell / Zsh / Shell language module"
        formatter = "shfmt"
        primary_id = "shellscript"
        aliases = @("powershell", "zsh", "fish", ".sh", ".bash", ".zsh", ".ps1", ".psm1", ".fish")
        dialects = @("Bash", "PowerShell", "Zsh")
        id_map = @{ shellscript = "Bash"; powershell = "PowerShell"; zsh = "Zsh" }
        grammar_deps = @("tree-sitter-bash")
        fn_name = "format_shell"
    }
    @{
        name = "lang-markdown"
        desc = "Markdown / LaTeX language module"
        formatter = "prettier"
        primary_id = "markdown"
        aliases = @("latex", ".md", ".markdown", ".tex")
        dialects = @("Markdown", "Latex")
        id_map = @{ markdown = "Markdown"; latex = "Latex" }
        grammar_deps = @("tree-sitter-md")
        fn_name = "format_markdown"
    }
    @{
        name = "lang-sql"
        desc = "SQL / GraphQL language module"
        formatter = "sqlfluff / prettier"
        primary_id = "sql"
        aliases = @("graphql", ".sql", ".graphql", ".gql")
        dialects = @("Sql", "GraphQL")
        id_map = @{ sql = "Sql"; graphql = "GraphQL" }
        grammar_deps = @("tree-sitter-sql", "tree-sitter-graphql")
        fn_name = "format_sql"
    }
    @{
        name = "lang-ruby"
        desc = "Ruby / PHP / Perl / Lua language module"
        formatter = "rubocop / php-cs-fixer / stylua"
        primary_id = "ruby"
        aliases = @("php", "perl", "lua", ".rb", ".php", ".pl", ".pm", ".lua")
        dialects = @("Ruby", "Php", "Perl", "Lua")
        id_map = @{ ruby = "Ruby"; php = "Php"; perl = "Perl"; lua = "Lua" }
        grammar_deps = @("tree-sitter-ruby", "tree-sitter-php", "tree-sitter-lua")
        fn_name = "format_ruby"
    }
    @{
        name = "lang-swift"
        desc = "Swift / Objective-C / Objective-C++ language module"
        formatter = "swift-format"
        primary_id = "swift"
        aliases = @("objective-c", "objective-cpp", ".swift", ".m", ".mm")
        dialects = @("Swift", "ObjC", "ObjCpp")
        id_map = @{ swift = "Swift"; "objective-c" = "ObjC"; "objective-cpp" = "ObjCpp" }
        grammar_deps = @("tree-sitter-swift")
        fn_name = "format_swift"
    }
    @{
        name = "lang-mobile"
        desc = "Dart language module"
        formatter = "dart format"
        primary_id = "dart"
        aliases = @(".dart")
        dialects = @()
        id_map = @{}
        grammar_deps = @("tree-sitter-dart")
        fn_name = "format_dart"
    }
    @{
        name = "lang-devops"
        desc = "HCL / Terraform / Dockerfile / Makefile / Nix language module"
        formatter = "terraform fmt / dockfmt"
        primary_id = "terraform"
        aliases = @("dockerfile", "makefile", "nix", ".tf", ".hcl", "Dockerfile", "Makefile", ".nix")
        dialects = @("Hcl", "Dockerfile", "Makefile", "Nix")
        id_map = @{ terraform = "Hcl"; dockerfile = "Dockerfile"; makefile = "Makefile"; nix = "Nix" }
        grammar_deps = @("tree-sitter-dockerfile")
        fn_name = "format_devops"
    }
    @{
        name = "lang-functional"
        desc = "Haskell / Elixir / Erlang / OCaml / Clojure / R / Julia language module"
        formatter = "ormolu / mix format / erlfmt / ocamlformat"
        primary_id = "haskell"
        aliases = @("elixir", "erlang", "ocaml", "clojure", "r", "julia", "lisp", "scheme", ".hs", ".lhs", ".ex", ".exs", ".erl", ".hrl", ".ml", ".mli", ".clj", ".cljs", ".r", ".R", ".jl", ".lisp", ".lsp", ".scm", ".ss")
        dialects = @("Haskell", "Elixir", "Erlang", "OCaml", "Clojure", "R", "Julia", "Lisp", "Scheme")
        id_map = @{ haskell = "Haskell"; elixir = "Elixir"; erlang = "Erlang"; ocaml = "OCaml"; clojure = "Clojure"; r = "R"; julia = "Julia"; lisp = "Lisp"; scheme = "Scheme" }
        grammar_deps = @("tree-sitter-haskell", "tree-sitter-elixir", "tree-sitter-erlang", "tree-sitter-ocaml", "tree-sitter-julia", "tree-sitter-r")
        fn_name = "format_functional"
    }
    @{
        name = "lang-modern"
        desc = "Zig / Nim / D language module"
        formatter = "zig fmt"
        primary_id = "zig"
        aliases = @("nim", "d", ".zig", ".nim", ".d")
        dialects = @("Zig", "Nim", "D")
        id_map = @{ zig = "Zig"; nim = "Nim"; d = "D" }
        grammar_deps = @("tree-sitter-zig")
        fn_name = "format_modern"
    }
    @{
        name = "lang-other"
        desc = "Solidity / GDScript / AutoHotkey / COBOL / Fortran / Assembly stubs"
        formatter = "prettier-plugin-solidity (Solidity); stubs for rest"
        primary_id = "solidity"
        aliases = @("gdscript", "ahk", "cobol", "fortran", "asm", ".sol", ".vy", ".gd", ".ahk", ".cob", ".cbl", ".f90", ".f95", ".asm", ".s")
        dialects = @("Solidity", "GDScript", "Ahk", "Cobol", "Fortran", "Asm")
        id_map = @{ solidity = "Solidity"; gdscript = "GDScript"; ahk = "Ahk"; cobol = "Cobol"; fortran = "Fortran"; asm = "Asm" }
        grammar_deps = @("tree-sitter-solidity")
        fn_name = "format_other"
    }
    @{
        name = "lang-template"
        desc = "Jinja / Liquid / EJS / Handlebars / Twig template language stubs"
        formatter = "pass-through stubs"
        primary_id = "jinja"
        aliases = @("liquid", "ejs", "handlebars", "twig", ".jinja", ".jinja2", ".liquid", ".ejs", ".hbs", ".handlebars", ".twig")
        dialects = @("Jinja", "Liquid", "Ejs", "Handlebars", "Twig")
        id_map = @{ jinja = "Jinja"; liquid = "Liquid"; ejs = "Ejs"; handlebars = "Handlebars"; twig = "Twig" }
        grammar_deps = @()
        fn_name = "format_template"
    }
    @{
        name = "lang-sass"
        desc = "Sass indented syntax (.sass) language module"
        formatter = "whitespace normalize (no canonical formatter)"
        primary_id = "sass"
        aliases = @(".sass")
        dialects = @()
        id_map = @{}
        grammar_deps = @()
        fn_name = "format_sass"
    }
)

foreach ($crate in $CRATES) {
    $dir = "$ROOT/$($crate.name)/src"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null

    # --- schema.json ---
    $schema = @"
{
  "`$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OmniFormatter $($crate.desc) Config",
  "type": "object",
  "properties": {
    "indent_size": { "type": "integer", "default": 2 },
    "column_limit": { "type": "integer", "default": 80 }
  }
}
"@
    Set-Content -Path "$ROOT/$($crate.name)/schema.json" -Value $schema -Encoding UTF8

    # --- Cargo.toml ---
    $grammarDeps = ""
    foreach ($dep in $crate.grammar_deps) {
        $grammarDeps += "`n$dep.workspace = true"
    }
    $cargo = @"
[package]
name        = "$($crate.name)"
description = "OmniFormatter: $($crate.desc)"
version.workspace     = true
edition.workspace     = true
license.workspace     = true
authors.workspace     = true
repository.workspace  = true

[lib]
crate-type = ["cdylib", "rlib"]

[features]
standalone = []

[dependencies]
protocol     = { path = "../protocol" }
serde.workspace      = true
serde_json.workspace = true
wasm-bindgen.workspace = true
log.workspace        = true
tree-sitter.workspace = true$grammarDeps
"@
    Set-Content -Path "$ROOT/$($crate.name)/Cargo.toml" -Value $cargo -Encoding UTF8

    # --- plugin.rs ---
    $plugin = @"
//! Plugin metadata for $($crate.name).
pub const NAME: &str = "$($crate.name)";
pub const FORMATTER: &str = "$($crate.formatter)";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
"@
    Set-Content -Path "$dir/plugin.rs" -Value $plugin -Encoding UTF8

    # --- adapter.rs ---
    $adapter = @"
//! $($crate.name) config adapter.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,
}

fn default_indent_size()  -> usize { 2 }
fn default_column_limit() -> usize { 80 }

impl Default for Config {
    fn default() -> Self { Self { indent_size: default_indent_size(), column_limit: default_column_limit() } }
}

pub fn config_from_json(json: &str) -> Config {
    serde_json::from_str(json).unwrap_or_default()
}
"@
    Set-Content -Path "$dir/adapter.rs" -Value $adapter -Encoding UTF8

    # --- format.rs (pass-through with whitespace normalize) ---
    $format = @"
//! $($crate.name) formatting logic.
//!
//! Formatter target: $($crate.formatter)
//! Strategy: whitespace normalization + brace-depth indent pass.
//! Full CST-based formatting is planned for a future release.

use crate::adapter::Config;
use protocol::FormatError;

/// Normalise indentation using brace-depth tracking.
/// Returns source verbatim if it cannot be decoded as UTF-8.
pub fn format(source: &[u8], config: &Config) -> Result<Vec<u8>, FormatError> {
    let text = match std::str::from_utf8(source) {
        Ok(s)  => s,
        Err(_) => return Ok(source.to_vec()), // binary file: return verbatim
    };

    let mut out = String::with_capacity(source.len());
    let mut depth = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();

        // Count net brace change to decide if this line decreases depth first
        let opens  = trimmed.chars().filter(|&c| c == '{').count();
        let closes = trimmed.chars().filter(|&c| c == '}').count();

        if closes > opens && depth >= (closes - opens) {
            depth -= closes - opens;
        }

        if trimmed.is_empty() {
            out.push('\n');
        } else {
            out.push_str(&" ".repeat(depth * config.indent_size));
            out.push_str(trimmed);
            out.push('\n');
        }

        if opens > closes {
            depth += opens - closes;
        }
    }

    if !out.ends_with('\n') { out.push('\n'); }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_empty() {
        let result = format(b"", &Config::default()).unwrap();
        assert_eq!(result, b"\n");
    }

    #[test]
    fn format_idempotent() {
        let src = b"fn main() {\n    let x = 1;\n}\n";
        let first  = format(src, &Config::default()).unwrap();
        let second = format(&first, &Config::default()).unwrap();
        assert_eq!(first, second, "$($crate.name) must be idempotent");
    }
}
"@
    Set-Content -Path "$dir/format.rs" -Value $format -Encoding UTF8

    Write-Host "Created: $($crate.name)" -ForegroundColor Green
}

Write-Host "`nAll crate scaffolds created. lib.rs files need manual dialect wiring." -ForegroundColor Yellow
