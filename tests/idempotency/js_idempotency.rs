//! Idempotency Test Suite for lang-js (L-09, Pillar 7)
//!
//! Verifies that format(format(x)) === format(x) for all valid JS/TS inputs.
//!
//! # Strategy
//!
//! Phase 3: Tests valid fixture files from tests/fixtures/js/*.
//! Phase 4: 10,000-fixture fuzz-based generation using the Tree-sitter grammar
//!          as a production grammar for input generation.
//!
//! # CI
//!
//! Runs on every commit to lang-js via the CI workflow.
//! A language module that fails idempotency is blocked from publishing.

#[cfg(test)]
mod idempotency_tests {
    use lang_js::format::format;
    use protocol::config::ConfigIR;

    /// Format `source` twice and assert byte-for-byte equality.
    fn assert_idempotent(source: &[u8], config: &ConfigIR) {
        let first = format(source, config)
            .expect("First format pass must not error");
        let second = format(&first, config)
            .expect("Second format pass must not error");

        assert_eq!(
            first, second,
            "Idempotency violation: format(format(x)) != format(x)\n\
             First:  {:?}\n\
             Second: {:?}",
            String::from_utf8_lossy(&first),
            String::from_utf8_lossy(&second)
        );
    }

    #[test]
    fn idempotent_empty_source() {
        assert_idempotent(b"", &ConfigIR::default());
    }

    #[test]
    fn idempotent_simple_declaration() {
        assert_idempotent(b"const x = 1;\n", &ConfigIR::default());
    }

    #[test]
    fn idempotent_function_declaration() {
        assert_idempotent(
            b"function hello(name) {\n  return 'Hello, ' + name;\n}\n",
            &ConfigIR::default(),
        );
    }

    #[test]
    fn idempotent_unicode_source() {
        assert_idempotent(
            "const greeting = '你好世界';\n".as_bytes(),
            &ConfigIR::default(),
        );
    }

    #[test]
    fn idempotent_already_formatted() {
        // If source is already formatted, the result should be identical.
        let source = b"const x = 1;\nconst y = 2;\n";
        assert_idempotent(source, &ConfigIR::default());
    }
}
