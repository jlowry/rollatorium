//! The crate-wide error type.

use thiserror::Error;

/// The error type returned by every fallible operation in this crate.
///
/// Each variant wraps a human-readable message describing what went wrong and
/// at which stage of the [parse → evaluate](crate) pipeline. The type is
/// `Send + Sync + 'static`, so it interoperates with `anyhow`,
/// `Box<dyn std::error::Error>`, and code that moves errors across threads.
///
/// This enum is `#[non_exhaustive]`: future releases may add variants, so
/// downstream `match` expressions should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum RollatoriumError {
    /// The lexer could not turn the input string into tokens.
    #[error("Lexer error: {0}")]
    Lexer(String),
    /// The parser could not build a valid AST from the tokens.
    #[error("Parser error: {0}")]
    Parser(String),
    /// A well-formed expression could not be evaluated, e.g. a non-positive die
    /// size or exceeding the configured maximum number of rolls.
    #[error("Evaluation error: {0}")]
    Eval(String),
    /// An expression divided (or took a remainder) by a zero divisor.
    ///
    /// Floating-point division by zero would otherwise produce a non-finite
    /// total (`inf`/`NaN`); this variant surfaces it as a handled error instead.
    #[error("Division by zero")]
    DivisionByZero,
}
