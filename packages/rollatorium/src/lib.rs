#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// The entire library lives in `rollatorium-core`; this crate re-exports it and
// adds the compile-time `dice!` macro. Splitting the parser/AST into a separate
// crate is what lets `rollatorium-macros` call the parser at compile time
// without forming a dependency cycle with this crate.
pub use rollatorium_core::*;

/// Parse and validate a dice expression at **compile time**, expanding to owned
/// AST construction (no runtime parsing).
///
/// Re-exported from `rollatorium-macros` and enabled by the default `macros`
/// feature; disable it with `default-features = false` to drop the
/// `rollatorium-macros` dependency. See the [`dice!`](macro@dice) documentation
/// for details.
#[cfg(feature = "macros")]
pub use rollatorium_macros::dice;
