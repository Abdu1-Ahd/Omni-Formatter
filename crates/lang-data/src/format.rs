//! JSON Formatting Logic — Tree-sitter CST walker
//!
//! # Strategy
//!
//!   1. Parse the source with `tree-sitter-json`.
//!   2. Walk the Concrete Syntax Tree recursively.
//!   3. Reconstruct a formatted string obeying:
//!      - `indent_size`  (from `ConfigIR.indent_size`, baseline field)
//!      - `print_width`  (from `ConfigIR.print_width`, baseline field)
//!      - `json__trailingComma`  — add `,` after the last element (JSON5/JSONC)
//!      - `json__sortKeys`       — sort object keys alphabetically
//!      - `json__indentArrays`   — always expand arrays multi-line (default: true)
//!   4. On any parse error, return the source verbatim to prevent data loss.
//!   5. Assert idempotency in debug builds.
//!
//! # Schema keys consumed
//!
//! | Key                    | Type | Default | Effect                                         |
//! |------------------------|------|---------|------------------------------------------------|
//! | `json__trailingComma`  | bool | false   | Emit `,` after last pair/element               |
//! | `json__sortKeys`       | bool | false   | Sort object keys lexicographically             |
//! | `json__indentArrays`   | bool | true    | Expand arrays across multiple lines            |

use protocol::config::ConfigIR;
use protocol::FormatError;

// ── JSON-specific config ──────────────────────────────────────────────────

/// Config extracted from `ConfigIR` for the JSON formatter.
/// Pulled once per format call so the walker doesn't need repeated map lookups.
#[derive(Debug, Clone)]
pub struct JsonConfig {
    /// Spaces per indentation level (from `ConfigIR.indent_size`).
    pub indent_size: usize,
    /// Soft max line width (from `ConfigIR.print_width`).
    pub print_width: usize,
    /// Add a trailing comma after the last element in objects/arrays.
    /// Key: `json__trailingComma` (bool, default: false).
    pub trailing_comma: bool,
    /// Sort object keys lexicographically.
    /// Key: `json__sortKeys` (bool, default: false).
    pub sort_keys: bool,
    /// Expand arrays to multiple lines (one element per line).
    /// Key: `json__indentArrays` (bool, default: true).
    pub indent_arrays: bool,
}

impl JsonConfig {
    /// Build a `JsonConfig` from the universal `ConfigIR`, pulling lang-specific
    /// overrides from `config.extras` via the typed accessor helpers.
    pub fn from_config_ir(config: &ConfigIR) -> Self {
        JsonConfig {
            indent_size: config.indent_size as usize,
            print_width: config.print_width as usize,
            trailing_comma: config
                .get_extra_bool("json__trailingComma")
                .unwrap_or(false),
            sort_keys: config.get_extra_bool("json__sortKeys").unwrap_or(false),
            indent_arrays: config.get_extra_bool("json__indentArrays").unwrap_or(true),
        }
    }
}

// ── Tree-sitter language loader ───────────────────────────────────────────

fn json_language() -> tree_sitter::Language {
    tree_sitter_json::language()
}

// ── CST Walker ───────────────────────────────────────────────────────────

struct JsonFormatter<'a> {
    source: &'a [u8],
    config: &'a JsonConfig,
}

impl<'a> JsonFormatter<'a> {
    fn new(source: &'a [u8], config: &'a JsonConfig) -> Self {
        JsonFormatter { source, config }
    }

    /// Extract the raw UTF-8 text of a node from the source buffer.
    fn text_of(&self, node: &tree_sitter::Node) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    /// Build the indentation prefix for a given depth.
    fn indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.config.indent_size)
    }

    // ── Top-level dispatch ────────────────────────────────────────────────

    /// Format any JSON value node and return its formatted string.
    fn format_value(&self, node: tree_sitter::Node, depth: usize) -> String {
        match node.kind() {
            "document" => {
                // document has a single named child: the root value
                if let Some(child) = node.named_child(0) {
                    let mut out = self.format_value(child, depth);
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out
                } else {
                    "\n".to_string()
                }
            }
            "object" => self.format_object(node, depth),
            "array" => self.format_array(node, depth),
            // All scalar types: string, number, true, false, null
            _ => self.text_of(&node).to_string(),
        }
    }

    // ── Object formatting ─────────────────────────────────────────────────

    fn format_object(&self, node: tree_sitter::Node, depth: usize) -> String {
        // Collect all named `pair` children
        let mut pairs: Vec<tree_sitter::Node> = {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .filter(|n| n.kind() == "pair")
                .collect()
        };

        if pairs.is_empty() {
            return "{}".to_string();
        }

        // Optionally sort pairs by their key text
        if self.config.sort_keys {
            pairs.sort_by(|a, b| {
                let ka = a
                    .child_by_field_name("key")
                    .map(|n| self.text_of(&n))
                    .unwrap_or("");
                let kb = b
                    .child_by_field_name("key")
                    .map(|n| self.text_of(&n))
                    .unwrap_or("");
                ka.cmp(kb)
            });
        }

        let inner_indent = self.indent(depth + 1);
        let close_indent = self.indent(depth);
        let last_idx = pairs.len() - 1;

        let mut out = "{\n".to_string();
        for (i, pair) in pairs.iter().enumerate() {
            let is_last = i == last_idx;
            let key = pair
                .child_by_field_name("key")
                .map(|n| self.text_of(&n))
                .unwrap_or("\"?\"");
            let val_node = pair.child_by_field_name("value");
            let val = val_node
                .map(|n| self.format_value(n, depth + 1))
                .unwrap_or_else(|| "null".to_string());

            // Decide whether to emit a trailing comma
            let comma = if is_last {
                if self.config.trailing_comma {
                    ","
                } else {
                    ""
                }
            } else {
                ","
            };

            out.push_str(&format!("{}{}: {}{}\n", inner_indent, key, val, comma));
        }
        out.push_str(&format!("{}}}", close_indent));
        out
    }

    // ── Array formatting ──────────────────────────────────────────────────

    fn format_array(&self, node: tree_sitter::Node, depth: usize) -> String {
        let elements: Vec<tree_sitter::Node> = {
            let mut cursor = node.walk();
            node.named_children(&mut cursor).collect()
        };

        if elements.is_empty() {
            return "[]".to_string();
        }

        // Format all elements first so we can decide inline vs multiline
        let formatted: Vec<String> = elements
            .iter()
            .map(|n| self.format_value(*n, depth + 1))
            .collect();

        // Try inline if: indent_arrays is false AND fits within print_width
        if !self.config.indent_arrays {
            let inline = format!("[{}]", formatted.join(", "));
            let inline_len = depth * self.config.indent_size + inline.len();
            if inline_len <= self.config.print_width {
                return inline;
            }
        }

        // Multi-line: one element per line
        let inner_indent = self.indent(depth + 1);
        let close_indent = self.indent(depth);
        let last_idx = formatted.len() - 1;

        let mut out = "[\n".to_string();
        for (i, elem) in formatted.iter().enumerate() {
            let is_last = i == last_idx;
            let comma = if is_last {
                if self.config.trailing_comma {
                    ","
                } else {
                    ""
                }
            } else {
                ","
            };
            out.push_str(&format!("{}{}{}\n", inner_indent, elem, comma));
        }
        out.push_str(&format!("{}]", close_indent));
        out
    }
}

// ── Public API ────────────────────────────────────────────────────────────

/// Format JSON source bytes using the full Tree-sitter CST.
///
/// Accepts the universal `ConfigIR` directly so that lang-specific schema
/// overrides in `config.extras` (e.g. `json__trailingComma`) are consumed.
///
/// Returns the source verbatim on any parse failure — never corrupts data.
pub fn format(source: &[u8], config: &ConfigIR) -> Result<Vec<u8>, FormatError> {
    // Reject invalid UTF-8 immediately (binary files).
    if std::str::from_utf8(source).is_err() {
        return Ok(source.to_vec());
    }

    let json_config = JsonConfig::from_config_ir(config);
    let out = format_with_config(source, &json_config)?;

    // Idempotency check in debug builds
    #[cfg(debug_assertions)]
    {
        let second = format_with_config(&out, &json_config)?;
        debug_assert_eq!(
            out.as_slice(),
            second.as_slice(),
            "lang-data JSON formatter is not idempotent!"
        );
    }

    Ok(out)
}

fn format_with_config(source: &[u8], config: &JsonConfig) -> Result<Vec<u8>, FormatError> {
    let language = json_language();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| FormatError::Internal {
            message: format!("tree-sitter-json grammar load failed: {}", e),
        })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| FormatError::ParseFailed {
            message: "tree-sitter-json returned None".into(),
        })?;

    if tree.root_node().has_error() {
        // Parse error: emit verbatim so we never corrupt user data
        log::warn!("lang-data: JSON parse error — emitting verbatim");
        return Ok(source.to_vec());
    }

    let formatter = JsonFormatter::new(source, config);
    let formatted = formatter.format_value(tree.root_node(), 0);
    Ok(formatted.into_bytes())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::config::ConfigIR;

    // ── Helpers ───────────────────────────────────────────────────────────

    /// Build a ConfigIR with a set of extras set.
    fn config_with_extras(extras: &[(&str, serde_json::Value)]) -> ConfigIR {
        let mut ir = ConfigIR::default();
        for (k, v) in extras {
            ir.extras.insert(k.to_string(), v.clone());
        }
        ir
    }

    fn formatted(src: &str, config: &ConfigIR) -> String {
        let result = format(src.as_bytes(), config).expect("format must not fail");
        String::from_utf8(result).expect("output must be valid UTF-8")
    }

    // ── Baseline behaviour ────────────────────────────────────────────────

    #[test]
    fn formats_simple_object() {
        let src = r#"{"b":2,"a":1}"#;
        let out = formatted(src, &ConfigIR::default());
        // Keys should be on their own lines, indented by 2 (default)
        assert!(out.contains("  \"b\": 2"), "got: {out}");
        assert!(out.contains("  \"a\": 1"), "got: {out}");
        assert!(out.ends_with("}\n"), "must end with }}\\n, got: {out}");
    }

    #[test]
    fn formats_simple_array() {
        let src = r#"[1,2,3]"#;
        let out = formatted(src, &ConfigIR::default());
        assert!(out.contains("  1"), "got: {out}");
        assert!(out.contains("  2"), "got: {out}");
        assert!(out.contains("  3"), "got: {out}");
    }

    #[test]
    fn empty_object() {
        let src = "{}";
        let out = formatted(src, &ConfigIR::default());
        assert!(out.contains("{}"), "got: {out}");
    }

    #[test]
    fn empty_array() {
        let src = "[]";
        let out = formatted(src, &ConfigIR::default());
        assert!(out.contains("[]"), "got: {out}");
    }

    #[test]
    fn scalars_pass_through() {
        for scalar in &["\"hello\"", "42", "3.14", "true", "false", "null"] {
            let out = formatted(scalar, &ConfigIR::default());
            assert!(out.trim() == *scalar, "scalar {scalar} mangled: {out}");
        }
    }

    #[test]
    fn idempotent_on_object() {
        let src = r#"{"name":"John","age":30,"active":true}"#;
        let config = ConfigIR::default();
        let first = formatted(src, &config);
        let second = formatted(&first, &config);
        assert_eq!(first, second, "formatter is not idempotent");
    }

    // ── json__trailingComma ───────────────────────────────────────────────

    #[test]
    fn trailing_comma_disabled_by_default() {
        let src = r#"{"a":1,"b":2}"#;
        let out = formatted(src, &ConfigIR::default());
        // Last line before closing } must NOT have a trailing comma
        let lines: Vec<&str> = out.lines().collect();
        let last_val_line = lines.iter().rev().find(|l| l.contains("\"b\"")).unwrap();
        assert!(
            !last_val_line.ends_with(','),
            "trailing comma should be absent by default, got: {out}"
        );
    }

    #[test]
    fn trailing_comma_enabled_adds_comma_to_last_element() {
        let src = r#"{"a":1,"b":2}"#;
        let config = config_with_extras(&[("json__trailingComma", serde_json::Value::Bool(true))]);
        let out = formatted(src, &config);
        let lines: Vec<&str> = out.lines().collect();
        let last_val_line = lines.iter().rev().find(|l| l.contains("\"b\"")).unwrap();
        assert!(
            last_val_line.trim().ends_with(','),
            "trailing comma should be present, got: {out}"
        );
    }

    #[test]
    fn trailing_comma_in_array() {
        let src = r#"[1,2,3]"#;
        let config = config_with_extras(&[("json__trailingComma", serde_json::Value::Bool(true))]);
        let out = formatted(src, &config);
        let lines: Vec<&str> = out.lines().collect();
        let last_elem_line = lines.iter().rev().find(|l| l.contains('3')).unwrap();
        assert!(
            last_elem_line.trim().ends_with(','),
            "array trailing comma should be present, got: {out}"
        );
    }

    // ── json__sortKeys ────────────────────────────────────────────────────

    #[test]
    fn sort_keys_disabled_preserves_order() {
        // Without sortKeys, insertion order from the source is preserved
        let src = r#"{"z":3,"a":1,"m":2}"#;
        let out = formatted(src, &ConfigIR::default());
        let z_pos = out.find("\"z\"").unwrap();
        let a_pos = out.find("\"a\"").unwrap();
        assert!(
            z_pos < a_pos,
            "z should appear before a (insertion order), got: {out}"
        );
    }

    #[test]
    fn sort_keys_enabled_alphabetises_keys() {
        let src = r#"{"z":3,"a":1,"m":2}"#;
        let config = config_with_extras(&[("json__sortKeys", serde_json::Value::Bool(true))]);
        let out = formatted(src, &config);
        let a_pos = out.find("\"a\"").unwrap();
        let m_pos = out.find("\"m\"").unwrap();
        let z_pos = out.find("\"z\"").unwrap();
        assert!(
            a_pos < m_pos && m_pos < z_pos,
            "keys must be sorted a→m→z, got: {out}"
        );
    }

    // ── json__indentArrays ────────────────────────────────────────────────

    #[test]
    fn indent_arrays_true_always_expands() {
        let src = r#"[1,2,3]"#;
        // Default: indent_arrays = true → always multi-line
        let out = formatted(src, &ConfigIR::default());
        let newline_count = out.chars().filter(|&c| c == '\n').count();
        assert!(newline_count > 1, "array should be multi-line, got: {out}");
    }

    #[test]
    fn indent_arrays_false_stays_inline_when_short() {
        let src = r#"[1,2,3]"#;
        let config = config_with_extras(&[("json__indentArrays", serde_json::Value::Bool(false))]);
        let out = formatted(src, &config);
        // A short array should stay on one line when indent_arrays is false
        assert!(
            !out.contains('\n') || out.trim().starts_with('[') && out.trim().ends_with(']'),
            "short array should be inline, got: {out}"
        );
    }

    // ── indent_size (baseline ConfigIR field) ─────────────────────────────

    #[test]
    fn indent_size_4_is_respected() {
        let src = r#"{"a":1}"#;
        let config = ConfigIR {
            indent_size: 4,
            ..ConfigIR::default()
        };
        let out = formatted(src, &config);
        assert!(
            out.contains("    \"a\""),
            "4-space indent expected, got: {out}"
        );
    }

    // ── Nested structures ─────────────────────────────────────────────────

    #[test]
    fn nested_object_indented_correctly() {
        let src = r#"{"outer":{"inner":42}}"#;
        let out = formatted(src, &ConfigIR::default());
        // inner key at depth 2 → 4 spaces with default indent_size=2
        assert!(
            out.contains("    \"inner\""),
            "nested depth should be 4 spaces, got: {out}"
        );
    }

    #[test]
    fn combined_sort_and_trailing_comma() {
        let src = r#"{"z":3,"a":1}"#;
        let config = config_with_extras(&[
            ("json__sortKeys", serde_json::Value::Bool(true)),
            ("json__trailingComma", serde_json::Value::Bool(true)),
        ]);
        let out = formatted(src, &config);
        let a_pos = out.find("\"a\"").unwrap();
        let z_pos = out.find("\"z\"").unwrap();
        assert!(a_pos < z_pos, "keys must be sorted, got: {out}");
        let lines: Vec<&str> = out.lines().collect();
        let z_line = lines.iter().rev().find(|l| l.contains("\"z\"")).unwrap();
        assert!(
            z_line.trim().ends_with(','),
            "trailing comma expected on last key, got: {out}"
        );
    }

    // ── Safety: invalid JSON returns verbatim ─────────────────────────────

    #[test]
    fn invalid_json_returns_verbatim() {
        let src = b"{ this is not : valid json !!!";
        let config = ConfigIR::default();
        let out = format(src, &config).unwrap();
        assert_eq!(out, src, "invalid JSON must be returned verbatim");
    }

    // ── Safety: binary source returns verbatim ────────────────────────────

    #[test]
    fn binary_source_returns_verbatim() {
        let src = b"\x80\x81\x82"; // invalid UTF-8
        let config = ConfigIR::default();
        let out = format(src, &config).unwrap();
        assert_eq!(out, src);
    }
}
