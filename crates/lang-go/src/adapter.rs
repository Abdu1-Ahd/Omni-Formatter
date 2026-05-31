//! Go Config Adapter
//!
//! Go has no config file for gofmt. This adapter provides a consistent interface
//! with other language crates but enforces gofmt's opinionated defaults (e.g. Tabs).

use protocol::config::{ConfigIR, IndentStyle};

/// Build a `ConfigIR` from a JSON representation of Go formatting options.
/// Since Go formatting is opinionated, this mostly ignores the input and
/// returns the Go defaults, except for potentially `end_of_line` which might
/// be mapped from `.editorconfig` in the future.
pub fn config_from_go_json(json: &str) -> ConfigIR {
    let mut config = if let Ok(parsed) = serde_json::from_str::<ConfigIR>(json) {
        parsed
    } else {
        ConfigIR::default()
    };

    // Go ALWAYS uses tabs for indentation
    config.indent_style = IndentStyle::Tabs;

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_always_uses_tabs() {
        let json = r#"{"indent_style": "spaces"}"#;
        let config = config_from_go_json(json);
        assert_eq!(config.indent_style, IndentStyle::Tabs);
    }
}
