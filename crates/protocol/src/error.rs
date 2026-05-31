//! Format error types returned by the WASM core and language modules.

use serde::{Deserialize, Serialize};

/// All errors that can occur during a format operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum FormatError {
    /// The source file exceeds the 10MB limit (L-01 mitigation).
    FileTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },

    /// Tree-sitter failed to parse the source into a valid CST.
    /// This should not happen in normal operation — Tree-sitter is error-tolerant.
    ParseFailed { message: String },

    /// The source contains a syntax error at the selected range boundary,
    /// making range formatting impossible (L-15 mitigation).
    SyntaxErrorInRange { offset: usize },

    /// A language module returned an error during formatting.
    ModuleError {
        language_id: String,
        message: String,
    },

    /// The language module for this file is not installed and could not be fetched.
    ModuleNotFound { language_id: String },

    /// The WASM module failed SHA-256 verification (L-02 registry security).
    ModuleVerificationFailed {
        language_id: String,
        expected: String,
        actual: String,
    },

    /// Idempotency check failed in debug builds (L-09 mitigation).
    /// This is a panic in release builds.
    #[cfg(debug_assertions)]
    IdempotencyViolation { language_id: String, diff: String },

    /// The config adapter encountered a malformed config file.
    ConfigParseError { path: String, message: String },

    /// An internal error that should never reach production.
    Internal { message: String },
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::FileTooLarge {
                size_bytes,
                limit_bytes,
            } => {
                write!(
                    f,
                    "File too large: {} bytes (limit: {} bytes)",
                    size_bytes, limit_bytes
                )
            }
            FormatError::ParseFailed { message } => {
                write!(f, "Parse failed: {}", message)
            }
            FormatError::SyntaxErrorInRange { offset } => {
                write!(
                    f,
                    "Syntax error at byte offset {} — range formatting skipped",
                    offset
                )
            }
            FormatError::ModuleError {
                language_id,
                message,
            } => {
                write!(f, "Module error [{}]: {}", language_id, message)
            }
            FormatError::ModuleNotFound { language_id } => {
                write!(f, "No formatter module found for language: {}", language_id)
            }
            FormatError::ModuleVerificationFailed { language_id, .. } => {
                write!(f, "Module verification failed for: {}", language_id)
            }
            #[cfg(debug_assertions)]
            FormatError::IdempotencyViolation { language_id, .. } => {
                write!(
                    f,
                    "IDEMPOTENCY VIOLATION in {}: format(format(x)) != format(x)",
                    language_id
                )
            }
            FormatError::ConfigParseError { path, message } => {
                write!(f, "Config parse error in {}: {}", path, message)
            }
            FormatError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for FormatError {}
