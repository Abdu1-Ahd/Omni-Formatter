//! CSS/SCSS/Less config adapter.
//!
//! Priority order: .omnifmt.json → .prettierrc → .stylelintrc → .editorconfig → defaults.
//! In opinionated mode, Stylelint config is ignored.

use protocol::config::{ConfigIR, QuoteStyle};
use serde::Deserialize;

/// Prettier CSS-specific fields.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrettierCssConfig {
    pub print_width: Option<u16>,
    pub tab_width: Option<u8>,
    pub use_tabs: Option<bool>,
    pub single_quote: Option<bool>,
    pub end_of_line: Option<String>,
}

impl PrettierCssConfig {
    pub fn apply_to(&self, base: &mut ConfigIR) {
        if let Some(w) = self.print_width { base.print_width = w; }
        if let Some(s) = self.tab_width { base.indent_size = s; }
        if let Some(true) = self.single_quote {
            base.quote_style = QuoteStyle::Single;
        }
    }
}

pub fn config_from_prettier_json(json: &str) -> ConfigIR {
    let mut config = ConfigIR::default();
    if let Ok(pc) = serde_json::from_str::<PrettierCssConfig>(json) {
        pc.apply_to(&mut config);
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_defaults_match_prettier() {
        let config = config_from_prettier_json("{}");
        assert_eq!(config.print_width, 80);
        assert_eq!(config.indent_size, 2);
    }
}
