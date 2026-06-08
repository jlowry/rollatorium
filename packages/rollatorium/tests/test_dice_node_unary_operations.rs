mod common;
use common::r;

// ============================================================================
// Node Tests - Unary Operations
// ============================================================================

#[test]
fn test_unop_plus_one_equals_one() {
    assert_eq!(r("+1"), 1.0);
}

#[test]
fn test_unop_one_equals_one() {
    assert_eq!(r("1"), 1.0);
}

#[test]
fn test_unop_minus_one_equals_minus_one() {
    assert_eq!(r("-1"), -1.0);
}

#[test]
fn test_unop_double_negative_one_equals_one() {
    assert_eq!(r("--1"), 1.0);
}

#[test]
fn test_unop_extreme_nesting() {
    assert_eq!(r("-+-++---+1"), -1.0);
}
