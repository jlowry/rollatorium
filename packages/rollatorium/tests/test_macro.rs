//! Tests for the compile-time `dice!` macro.
//!
//! The macro parses with the very same parser as [`rollatorium::parse`], so the
//! point of these tests is that its *code generation* faithfully rebuilds the
//! AST: `dice!(expr)` must equal `parse(expr)` for every node shape, and the
//! result must evaluate.
#![cfg(feature = "macros")]

use std::sync::LazyLock;

use rand::{SeedableRng, rngs::StdRng};
use rollatorium::{EvalConfig, Node, Value, dice, eval, eval_with_rng, parse};

// The expansion is `Node<&'static str>`, so it drops straight into a `'static`
// initializer with no borrow of any local.
static ATTACK: LazyLock<Node<&'static str>> = LazyLock::new(|| dice!("4d6kh3 [strength]"));

#[track_caller]
fn assert_roundtrips(expr: &'static str, generated: Node<&'static str>) {
    assert_eq!(
        generated,
        parse(expr).unwrap(),
        "dice!({expr:?}) did not reconstruct the same AST as parse({expr:?})"
    );
}

#[test]
fn macro_reconstructs_every_node_shape() {
    // Literal / Binary / Unary / grouping.
    assert_roundtrips("42", dice!("42"));
    assert_roundtrips("1 + 2 * 3", dice!("1 + 2 * 3"));
    assert_roundtrips("-(2 + 3) * 4", dice!("-(2 + 3) * 4"));
    assert_roundtrips("1 + +2 + -(-3)", dice!("1 + +2 + -(-3)"));
    // Comparisons.
    assert_roundtrips("2d6 >= 7", dice!("2d6 >= 7"));
    // Dice with a count, a single die, and a percentile die.
    assert_roundtrips("4d6", dice!("4d6"));
    assert_roundtrips("1d20 + 5", dice!("1d20 + 5"));
    assert_roundtrips("d%", dice!("d%"));
    assert_roundtrips("3d%kh1", dice!("3d%kh1"));
    // DiceWithOps: selectors, reroll/explode, min/max clamps.
    assert_roundtrips("4d6kh3", dice!("4d6kh3"));
    assert_roundtrips("3d6rr<3", dice!("3d6rr<3"));
    assert_roundtrips("1d6ra==6", dice!("1d6ra==6"));
    assert_roundtrips("1d6e==6", dice!("1d6e==6"));
    assert_roundtrips("2d6mi3ma5", dice!("2d6mi3ma5"));
    // Annotated, including multiple tags.
    assert_roundtrips("4d6 [fire damage]", dice!("4d6 [fire damage]"));
    assert_roundtrips("4d6kh3 [strength]", dice!("4d6kh3 [strength]"));
    assert_roundtrips("4d6 [fire] [magic]", dice!("4d6 [fire] [magic]"));
}

#[test]
fn macro_static_is_evaluable() {
    // Same seed as the equivalent parse-based test; the trees are identical so
    // the totals must be too.
    let from_macro = eval_with_rng(&ATTACK, EvalConfig::default(), StdRng::seed_from_u64(7))
        .unwrap()
        .total;
    let from_parse = eval_with_rng(
        &parse("4d6kh3 [strength]").unwrap(),
        EvalConfig::default(),
        StdRng::seed_from_u64(7),
    )
    .unwrap()
    .total;
    assert!((from_macro - from_parse).abs() < 1e-9);
    assert!((3.0..=18.0).contains(&from_macro));
}

#[test]
fn macro_literal_arithmetic_is_exact() {
    assert_eq!(eval(&dice!("1 + 2 * 3")).unwrap().total, 7.0);
    assert_eq!(eval(&dice!("(1 + 2) * 3")).unwrap().total, 9.0);
}

#[test]
fn macro_tags_expand_to_static_str() {
    let node: Node<&'static str> = dice!("2d8 [fire] [magic]");
    match node {
        Node::Annotated { annotations, .. } => {
            let tags: Vec<&str> = annotations
                .iter()
                .map(|annotation| annotation.tag)
                .collect();
            assert_eq!(tags, ["fire", "magic"]);
        }
        other => panic!("expected an annotated node, got {other:?}"),
    }
}

#[test]
fn macro_percent_die_is_evaluable() {
    let result = eval_with_rng(
        &dice!("d%"),
        EvalConfig::default(),
        StdRng::seed_from_u64(3),
    )
    .unwrap();
    match result.value {
        Value::Dice(_) => {}
        other => panic!("expected a dice value, got {other:?}"),
    }
    assert!((0.0..=90.0).contains(&result.total));
}
