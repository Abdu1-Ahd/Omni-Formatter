#!/usr/bin/env pwsh
# generate_lib_files.ps1
# Generates lib.rs for all remaining lang-* crates

$configs = @(
    @{
        crate="lang-csharp"; fn_name="format_csharp"; primary="csharp"
        aliases=@("fsharp",".cs",".fs",".fsi",".fsx")
        note="C# / F# — dotnet format / fantomas style"
    }
    @{
        crate="lang-data"; fn_name="format_data"; primary="json"
        aliases=@("json5","yaml","toml","xml","ini",".json",".jsonc",".yaml",".yml",".toml",".xml",".ini")
        note="JSON / YAML / TOML / XML / INI — prettier / taplo"
    }
    @{
        crate="lang-shell"; fn_name="format_shell"; primary="shellscript"
        aliases=@("powershell","zsh","fish",".sh",".bash",".zsh",".ps1",".psm1",".fish")
        note="Bash / PowerShell / Zsh — shfmt style"
    }
    @{
        crate="lang-markdown"; fn_name="format_markdown"; primary="markdown"
        aliases=@("latex",".md",".markdown",".tex")
        note="Markdown / LaTeX — prettier parity"
    }
    @{
        crate="lang-sql"; fn_name="format_sql"; primary="sql"
        aliases=@("graphql",".sql",".graphql",".gql")
        note="SQL / GraphQL — sqlfluff style"
    }
    @{
        crate="lang-ruby"; fn_name="format_ruby"; primary="ruby"
        aliases=@("php","perl","lua",".rb",".php",".pl",".pm",".lua")
        note="Ruby / PHP / Perl / Lua — rubocop / php-cs-fixer / stylua"
    }
    @{
        crate="lang-swift"; fn_name="format_swift"; primary="swift"
        aliases=@("objective-c","objective-cpp",".swift",".m",".mm")
        note="Swift / Objective-C / Objective-C++ — swift-format"
    }
    @{
        crate="lang-mobile"; fn_name="format_dart"; primary="dart"
        aliases=@(".dart")
        note="Dart — dart format style"
    }
    @{
        crate="lang-devops"; fn_name="format_devops"; primary="terraform"
        aliases=@("dockerfile","makefile","nix",".tf",".hcl","Dockerfile","Makefile",".nix")
        note="HCL / Dockerfile / Makefile / Nix"
    }
    @{
        crate="lang-functional"; fn_name="format_functional"; primary="haskell"
        aliases=@("elixir","erlang","ocaml","clojure","r","julia","lisp","scheme",".hs",".lhs",".ex",".exs",".erl",".hrl",".ml",".mli",".clj",".cljs",".r",".R",".jl",".lisp",".lsp",".scm",".ss")
        note="Haskell / Elixir / Erlang / OCaml / Clojure / R / Julia"
    }
    @{
        crate="lang-modern"; fn_name="format_modern"; primary="zig"
        aliases=@("nim","d",".zig",".nim",".d")
        note="Zig / Nim / D"
    }
    @{
        crate="lang-other"; fn_name="format_other"; primary="solidity"
        aliases=@("gdscript","ahk","cobol","fortran","asm",".sol",".vy",".gd",".ahk",".cob",".cbl",".f90",".f95",".asm",".s",".au3")
        note="Solidity / GDScript / AutoHotkey / COBOL / Fortran / Assembly"
    }
    @{
        crate="lang-template"; fn_name="format_template"; primary="jinja"
        aliases=@("liquid","ejs","handlebars","twig",".jinja",".jinja2",".liquid",".ejs",".hbs",".handlebars",".twig")
        note="Jinja / Liquid / EJS / Handlebars / Twig — stubs"
    }
    @{
        crate="lang-sass"; fn_name="format_sass"; primary="sass"
        aliases=@(".sass")
        note="Sass indented syntax (.sass)"
    }
)

foreach ($c in $configs) {
    $aliasLines = ($c.aliases | ForEach-Object { "        JsValue::from_str(`"$_`")," }) -join "`n"
    $librs = @"
//! $($c.note) Language Module
//!
//! Part of OmniFormatter v0.2.0 language expansion.

pub mod adapter;
pub mod format;
pub mod plugin;

use wasm_bindgen::prelude::*;

/// Format source using this module's formatter.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn $($c.fn_name)(
    source_bytes: &[u8],
    config_json: &str,
    _language_id: &str,
) -> Result<Vec<u8>, JsValue> {
    let config = adapter::config_from_json(config_json);
    match format::format(source_bytes, &config) {
        Ok(f)  => Ok(f),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String { include_str!("../schema.json").to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String { env!("CARGO_PKG_VERSION").to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String { "$($c.primary)".to_string() }

#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
$aliasLines
    ]
}
"@
    Set-Content -Path "crates/$($c.crate)/src/lib.rs" -Value $librs -Encoding UTF8
    Write-Host "lib.rs -> $($c.crate)" -ForegroundColor Green
}

Write-Host "`nAll lib.rs files generated." -ForegroundColor Cyan
