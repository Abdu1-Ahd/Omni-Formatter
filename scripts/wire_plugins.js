const fs = require('fs');
const path = require('path');

const crates = [
  { crate: "lang-c", name: "CPlugin", ext: ["c", "h", "cpp", "hpp", "cc", "cxx"] },
  { crate: "lang-csharp", name: "CsharpPlugin", ext: ["cs", "fs", "fsi", "fsx"] },
  { crate: "lang-data", name: "DataPlugin", ext: ["json", "json5", "yaml", "yml", "toml", "xml", "ini", "csv"] },
  { crate: "lang-devops", name: "DevopsPlugin", ext: ["tf", "hcl", "Dockerfile", "Makefile", "nix"] },
  { crate: "lang-functional", name: "FunctionalPlugin", ext: ["hs", "lhs", "ex", "exs", "erl", "hrl", "ml", "mli", "clj", "cljs", "r", "R", "jl", "lisp", "lsp", "scm", "ss"] },
  { crate: "lang-java", name: "JavaPlugin", ext: ["java", "class", "jar", "kt", "kts", "scala", "sc", "groovy"] },
  { crate: "lang-mobile", name: "MobilePlugin", ext: ["dart"] },
  { crate: "lang-modern", name: "ModernPlugin", ext: ["zig", "nim", "d", "astro", "svelte", "vue"] },
  { crate: "lang-other", name: "OtherPlugin", ext: ["sol", "vy", "gd", "ahk", "au3", "cob", "cbl", "f90", "f95", "asm", "s"] },
  { crate: "lang-ruby", name: "RubyPlugin", ext: ["rb", "php", "pl", "pm", "lua"] },
  { crate: "lang-sass", name: "SassPlugin", ext: ["sass"] },
  { crate: "lang-shell", name: "ShellPlugin", ext: ["sh", "bash", "zsh", "ps1", "psm1", "fish", "awk", "sed"] },
  { crate: "lang-sql", name: "SqlPlugin", ext: ["sql", "graphql", "gql"] },
  { crate: "lang-swift", name: "SwiftPlugin", ext: ["swift", "m", "mm"] },
  { crate: "lang-template", name: "TemplatePlugin", ext: ["jinja", "jinja2", "liquid", "ejs", "hbs", "handlebars", "twig", "adoc", "asciidoc"] },
];

for (const c of crates) {
  const pluginRs = `use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// ${c.name} plugin
pub struct ${c.name};

impl LanguagePlugin for ${c.name} {
    fn name(&self) -> &str {
        "${c.crate}"
    }

    fn extensions(&self) -> &[&str] {
        &[${c.ext.map(e => `"${e}"`).join(", ")}]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
`;

  const adapterRs = `//! ${c.crate} config adapter.
use serde::Deserialize;
use protocol::config::ConfigIR;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_column_limit")]
    pub column_limit: usize,
}

fn default_indent_size() -> usize { 2 }
fn default_column_limit() -> usize { 80 }

impl Default for Config {
    fn default() -> Self {
        Self { indent_size: 2, column_limit: 80 }
    }
}

impl From<&ConfigIR> for Config {
    fn from(ir: &ConfigIR) -> Self {
        Self {
            indent_size: ir.indent_size as usize,
            column_limit: ir.print_width as usize,
        }
    }
}

pub fn config_from_json(json: &str) -> Config {
    serde_json::from_str(json).unwrap_or_default()
}
`;

  const crateDir = path.join(__dirname, '..', 'crates', c.crate, 'src');
  if (fs.existsSync(crateDir)) {
      fs.writeFileSync(path.join(crateDir, 'plugin.rs'), pluginRs);
      fs.writeFileSync(path.join(crateDir, 'adapter.rs'), adapterRs);
      console.log(`Updated ${c.crate}`);
  } else {
      console.error(`Missing directory for ${c.crate}`);
  }
}
