use crate::format;
use protocol::{config::ConfigIR, FormatError, LanguagePlugin};

/// TemplatePlugin plugin
pub struct TemplatePlugin;

impl LanguagePlugin for TemplatePlugin {
    fn name(&self) -> &str {
        "lang-template"
    }

    fn extensions(&self) -> &[&str] {
        &["jinja", "jinja2", "liquid", "ejs", "hbs", "handlebars", "twig", "adoc", "asciidoc"]
    }

    fn format(&self, source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
        match format::format(source, &config.into()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FormatError::Internal { message: e.to_string() }),
        }
    }
}
