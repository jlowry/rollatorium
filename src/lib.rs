#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![forbid(unsafe_code)]

mod ast;
mod builder;
mod error;
mod eval;
mod lexer;
mod parser;
mod token;

pub use crate::ast::{Annotation, Node};
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
    pub use crate::builder::{Compare, Roll, d_percent, dice, die, lit, set};
}

pub type Result<T> = std::result::Result<T, error::RollatoriumError>;

/// Parse a dice expression into an AST.
///
/// The returned tree borrows annotation tags directly from `input` as `&str`
/// slices (zero-copy), so it cannot outlive the input. Use [`Node::map_tags`]
/// to convert the borrowed tags into a caller-defined type.
pub fn parse<I: AsRef<str> + ?Sized>(input: &I) -> Result<Node<&str>> {
    let mut parser = parser::Parser::new(input.as_ref())?;
    parser.parse()
}

pub fn eval<T: Clone>(expr: &Node<T>) -> Result<EvalResult<T>> {
    eval_expression(expr)
}

/// Parse and evaluate a dice expression. The result borrows annotation tags
/// from `input` as `&str` (zero-copy).
pub fn roll<I: AsRef<str> + ?Sized>(input: &I) -> Result<EvalResult<&str>> {
    let ast = parse(input)?;
    eval(&ast)
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    // ---------- Demo ----------
    #[test]
    fn test_simple_expression() {
        let input = "1 + 2 * 3";
        let expected = 7.0;
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_parentheses_expression() {
        let input = "(1 + 2) * 3";
        let expected = 9.0;
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_negative_and_parentheses() {
        let input = "-3 + 4 * (2 - 5)";
        let expected = -15.0;
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_unary_operators() {
        let input = "1 + +2 + -(-3)";
        let expected = 6.0;
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_single_number() {
        let input = "42";
        let expected = 42.0;
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let result = eval(&ast).unwrap();
        assert_eq!(result.total, expected);
    }

    #[test]
    fn test_keep_highest_drops_lowest() {
        let input = "4d6kh3";
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
        let mut parser = Parser::new(input).unwrap();
        let ast = parser.parse().unwrap();
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
