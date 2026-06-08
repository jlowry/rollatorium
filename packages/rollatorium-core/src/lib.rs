//! Lexer, parser, AST, and evaluator for tabletop-RPG dice expressions.
//!
//! This crate is the implementation behind the [`rollatorium`] crate, which
//! re-exports everything here and additionally provides the `dice!` macro. It
//! turns a string such as `4d6kh3`, `3d6rr<3`, or `4d6 [fire damage]` into a
//! [`Node`] tree via [`parse`] and evaluates it via [`eval`].
//!
//! Most users should depend on [`rollatorium`] rather than this crate directly.
//!
//! [`rollatorium`]: https://docs.rs/rollatorium
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod ast;
mod builder;
mod error;
mod eval;
mod lexer;
mod parser;
mod token;

pub use crate::ast::{Annotation, Node};
pub use crate::error::RollatoriumError;
pub use crate::eval::{
    DiceRoll, DieAdjustment, DieOrigin, DieResult, EvalConfig, EvalResult, SetElement, SetRoll,
    Value,
};
pub use crate::eval::{
    evaluate as eval_expression, evaluate_with_config as eval_with_config,
    evaluate_with_rng as eval_with_rng,
};

/// Fluent builder for constructing dice-expression ASTs with caller-defined
/// tags, without exposing the internal [`Node`] representation. See
/// [`build::Roll`].
pub mod build {
    pub use crate::builder::{Compare, Roll};
}

/// A specialized [`Result`](std::result::Result) for this crate, fixing the
/// error type to [`RollatoriumError`].
pub type Result<T> = std::result::Result<T, RollatoriumError>;

/// Parse a dice expression into an AST.
///
/// The returned tree borrows annotation tags directly from `input` as `&str`
/// slices (zero-copy), so it cannot outlive the input. Use [`Node::map_tags`]
/// to convert the borrowed tags into a caller-defined type, or parse straight
/// into an owned tree with `"…".parse::<Node<String>>()`.
///
/// # Errors
///
/// Returns [`RollatoriumError::Lexer`] or [`RollatoriumError::Parser`] if
/// `input` is not a valid dice expression.
///
/// # Examples
///
/// ```
/// let node = rollatorium_core::parse("4d6kh3 [strength]")?;
/// # Ok::<(), rollatorium_core::RollatoriumError>(())
/// ```
pub fn parse<I: AsRef<str> + ?Sized>(input: &I) -> Result<Node<&str>> {
    let mut parser = parser::Parser::new(input.as_ref())?;
    parser.parse()
}

/// Evaluate a parsed AST with the default configuration.
///
/// This rolls dice using a fresh thread-local RNG. For deterministic results,
/// use [`eval_with_rng`] with a seeded RNG.
///
/// # Errors
///
/// Returns [`RollatoriumError::Eval`] if the expression cannot be evaluated,
/// e.g. a non-positive die size or exceeding [`EvalConfig::max_rolls`].
///
/// # Examples
///
/// ```
/// let node = rollatorium_core::parse("2d6")?;
/// let result = rollatorium_core::eval(&node)?;
/// assert!((2.0..=12.0).contains(&result.total));
/// # Ok::<(), rollatorium_core::RollatoriumError>(())
/// ```
pub fn eval<T: Clone>(expr: &Node<T>) -> Result<EvalResult<T>> {
    eval_expression(expr)
}

/// Parse and evaluate a dice expression in one call.
///
/// The result borrows annotation tags from `input` as `&str` (zero-copy), so it
/// cannot outlive the input.
///
/// # Errors
///
/// Returns a [`RollatoriumError`] if `input` cannot be parsed or evaluated.
///
/// # Examples
///
/// ```
/// let result = rollatorium_core::roll("1d20 + 5")?;
/// assert!((6.0..=25.0).contains(&result.total));
/// # Ok::<(), rollatorium_core::RollatoriumError>(())
/// ```
pub fn roll<I: AsRef<str> + ?Sized>(input: &I) -> Result<EvalResult<&str>> {
    let ast = parse(input)?;
    eval(&ast)
}

/// Parse a dice expression into an AST that owns its tags as `String`.
///
/// This is the [`FromStr`](std::str::FromStr) convenience over [`parse`], which
/// borrows from the input; here the tags are copied so the tree is `'static`.
///
/// ```
/// use rollatorium_core::Node;
///
/// let node: Node<String> = "4d6 [fire]".parse()?;
/// # Ok::<(), rollatorium_core::RollatoriumError>(())
/// ```
impl std::str::FromStr for Node<String> {
    type Err = RollatoriumError;

    fn from_str(s: &str) -> Result<Self> {
        parse(s).map(|node| node.map_tags(|tag| tag.to_owned()))
    }
}

/// Implementation detail shared with the `rollatorium` crate and the
/// `rollatorium-macros` proc macro.
///
/// This module is **not** a stable public API: the items here may change or
/// disappear at any time. It exposes the AST support types (operators,
/// selectors, set operations) that the documented surface intentionally keeps
/// out of view, so that the generated `dice!` construction code can name them.
#[doc(hidden)]
pub mod __private {
    pub use crate::ast::{
        Annotation, BinaryOperator, DiceSize, Node, Selector, SelectorKind, SetOperation,
        SetOperator, UnaryOperator,
    };
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    // ---------- Demo ----------
    #[test]
    fn test_simple_expression() {
        let input = "1 + 2 * 3";
        let expected = 7.0;
        let ast = parse(input).unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_parentheses_expression() {
        let input = "(1 + 2) * 3";
        let expected = 9.0;
        let ast = parse(input).unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_negative_and_parentheses() {
        let input = "-3 + 4 * (2 - 5)";
        let expected = -15.0;
        let ast = parse(input).unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_unary_operators() {
        let input = "1 + +2 + -(-3)";
        let expected = 6.0;
        let ast = parse(input).unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_single_number() {
        let input = "42";
        let expected = 42.0;
        let ast = parse(input).unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_keep_highest_drops_lowest() {
        let input = "4d6kh3";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(0xFACE_CAFE);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert_eq!(dice.dice.len(), 4);
        assert_eq!(dice.dice.iter().filter(|die| die.kept).count(), 3);
        assert_eq!(dice.dice.iter().filter(|die| die.dropped).count(), 1);
        let kept_sum: f64 = dice
            .dice
            .iter()
            .filter(|die| die.kept)
            .map(|die| die.value)
            .sum();
        assert!((result.total - kept_sum).abs() < 1e-9);
    }

    #[test]
    fn test_reroll_until_threshold() {
        let input = "3d6rr<3";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(2);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert_eq!(dice.dice.len(), 3);
        assert!(dice.dice.iter().all(|die| die.value >= 3.0));
        assert!(dice.dice.iter().any(|die| die.rolls.len() > 1));
    }

    #[test]
    fn test_reroll_once_only_once() {
        let input = "3d6ro<4";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(0xABCD1234);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert!(
            dice.dice
                .iter()
                .all(|die| die.rolls.len() <= 2 && die.value >= 1.0)
        );
    }

    #[test]
    fn test_reroll_and_add_creates_extra_die() {
        let input = "1d6ra==6";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(14);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert!(
            dice.dice
                .iter()
                .any(|die| matches!(die.origin, DieOrigin::RerollAdd))
        );
        assert!(dice.dice.len() >= 2);
    }

    #[test]
    fn test_explode_chains_with_limit() {
        let input = "1d6e==6";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(14);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert!(
            dice.dice
                .iter()
                .any(|die| matches!(die.origin, DieOrigin::Explosion))
        );
    }

    #[test]
    fn test_minimum_and_maximum_adjustments() {
        let input = "2d6mi3ma5";
        let ast = parse(input).unwrap();
        let rng = StdRng::seed_from_u64(0x12345678);
        let result = eval_with_rng(&ast, EvalConfig::default(), rng).unwrap();
        let dice = match &result.value {
            Value::Dice(roll) => roll,
            other => panic!("expected dice result, got {:?}", other),
        };
        assert!(dice.dice.iter().any(|die| {
            die.adjustments
                .iter()
                .any(|adj| matches!(adj, DieAdjustment::Minimum { .. }))
        }));
        assert!(dice.dice.iter().any(|die| {
            die.adjustments
                .iter()
                .any(|adj| matches!(adj, DieAdjustment::Maximum { .. }))
        }));
        assert!(
            dice.dice
                .iter()
                .all(|die| die.value >= 3.0 && die.value <= 5.0)
        );
    }
}
