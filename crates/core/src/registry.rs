//! PluginRegistry — maps file extensions to `LanguagePlugin` implementations.
//!
//! # Adding a new language
//!
//! ```rust,ignore
//! // In your new crate:
//! struct MyPlugin;
//! impl protocol::LanguagePlugin for MyPlugin { ... }
//!
//! // In core::registry::default_registry():
//! registry.register(Box::new(MyPlugin));
//! ```
//!
//! That's it. The native runner, the WASM host, and all tests will
//! automatically pick up the new language.

use protocol::{config::ConfigIR, FormatError, LanguagePlugin};
use std::collections::HashMap;

/// A registry mapping file extensions → language plugins.
pub struct PluginRegistry {
    /// Maps lowercase extension (without dot) → plugin index in `plugins`.
    ext_map: HashMap<String, usize>,
    plugins: Vec<Box<dyn LanguagePlugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            ext_map: HashMap::new(),
            plugins: Vec::new(),
        }
    }

    /// Register a language plugin.
    ///
    /// All extensions claimed by `plugin` are mapped to it.
    /// If an extension is already claimed, the new plugin wins.
    pub fn register(&mut self, plugin: Box<dyn LanguagePlugin>) {
        let idx = self.plugins.len();
        for ext in plugin.extensions() {
            self.ext_map.insert(ext.to_lowercase(), idx);
        }
        self.plugins.push(plugin);
    }

    /// Format `source` by looking up the plugin for `ext`.
    ///
    /// `ext` should be the file extension **without** the leading dot, e.g. `"js"`, `"py"`.
    pub fn format_by_ext(
        &self,
        ext: &str,
        source: &[u8],
        config: &ConfigIR,
    ) -> Result<Vec<u8>, FormatError> {
        let key = ext.to_lowercase();
        match self.ext_map.get(&key) {
            None => Err(FormatError::Internal {
                message: format!("No plugin registered for extension '{}'", ext),
            }),
            Some(&idx) => {
                let plugin = &self.plugins[idx];
                match plugin.dialect_for_ext(&key) {
                    Some(dialect) => plugin.format_dialect(source, config, dialect),
                    None => plugin.format(source, config),
                }
            }
        }
    }

    /// List all registered extension → plugin name mappings (for diagnostics).
    pub fn registered_extensions(&self) -> Vec<(&str, &str)> {
        let mut out: Vec<(&str, &str)> = self
            .ext_map
            .iter()
            .map(|(ext, &idx)| (ext.as_str(), self.plugins[idx].name()))
            .collect();
        out.sort_by_key(|&(ext, _)| ext);
        out
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default registry with all built-in language plugins registered.
///
/// This is the registry used by the WASM core's `format()` entry point.
/// Language plugins are registered in this order (last one wins on extension conflict):
/// CSS/HTML → JS/TS → Python → Rust → Go.
pub fn default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(lang_css::plugin::CssPlugin));
    registry.register(Box::new(lang_js::plugin::JsPlugin));
    registry.register(Box::new(lang_python::plugin::PythonPlugin));
    registry.register(Box::new(lang_rust::plugin::RustPlugin));
    registry.register(Box::new(lang_go::plugin::GoPlugin));
    registry.register(Box::new(lang_markdown::plugin::MarkdownPlugin));
    registry.register(Box::new(lang_c::plugin::CPlugin));
    registry.register(Box::new(lang_csharp::plugin::CsharpPlugin));
    registry.register(Box::new(lang_data::plugin::DataPlugin));
    registry.register(Box::new(lang_devops::plugin::DevopsPlugin));
    registry.register(Box::new(lang_functional::plugin::FunctionalPlugin));
    registry.register(Box::new(lang_java::plugin::JavaPlugin));
    registry.register(Box::new(lang_mobile::plugin::MobilePlugin));
    registry.register(Box::new(lang_modern::plugin::ModernPlugin));
    registry.register(Box::new(lang_other::plugin::OtherPlugin));
    registry.register(Box::new(lang_ruby::plugin::RubyPlugin));
    registry.register(Box::new(lang_sass::plugin::SassPlugin));
    registry.register(Box::new(lang_shell::plugin::ShellPlugin));
    registry.register(Box::new(lang_sql::plugin::SqlPlugin));
    registry.register(Box::new(lang_swift::plugin::SwiftPlugin));
    registry.register(Box::new(lang_template::plugin::TemplatePlugin));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoPlugin;
    impl LanguagePlugin for EchoPlugin {
        fn name(&self) -> &str {
            "echo"
        }
        fn extensions(&self) -> &[&str] {
            &["echo"]
        }
        fn format(&self, source: &[u8], _config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
            Ok(source.to_vec())
        }
    }

    #[test]
    fn registry_roundtrip() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(EchoPlugin));
        let config = ConfigIR::default();
        let result = reg.format_by_ext("echo", b"hello", &config).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn unknown_extension_errors() {
        let reg = PluginRegistry::new();
        let config = ConfigIR::default();
        let err = reg.format_by_ext("xyz", b"", &config).unwrap_err();
        assert!(err.to_string().contains("xyz"));
    }
}
