mod parser;

mod error;

trait Formatter {
    fn format(&self, input: &str) -> Result<String, crate::error::FormatError>;
}

struct OmniFormatter<T> {
    config: T,
}

impl<T> OmniFormatter<T> {
    fn new(config: T) -> Self {
        Self { config }
    }
}

// very long line exceeding 88 characters in rust to test line wrapping behavior of rustfmt
fn extremely_long_function_name_that_exceeds_max_width(arg1: &str, arg2: &str, arg3: &str) {
}
