mod common;
use common::r;

// ============================================================================
// PEMDAS (Order of Operations)
// ============================================================================

#[test]
fn addition_multiplication_equivalence() {
    assert_eq!(r("1 + 3 * 6"), r("1 + (3 * 6)"));
}

#[test]
fn addition_multiplication_result() {
    assert_eq!(r("1 + 3 * 6"), 19.0);
}

#[test]
fn parentheses_change_precedence() {
    assert_eq!(r("(1 + 3) * 6"), 24.0);
}

#[test]
fn addition_left_associative_equiv_left() {
    assert_eq!(r("1 + 2 + 3"), r("(1 + 2) + 3"));
}

#[test]
fn addition_left_associative_equiv_right() {
    assert_eq!(r("1 + 2 + 3"), r("1 + (2 + 3)"));
}

#[test]
fn addition_sum_result() {
    assert_eq!(r("1 + 2 + 3"), 6.0);
}

#[test]
fn comparison_without_parentheses() {
    assert_eq!(r("1 + 2 == 2"), 0.0);
}

#[test]
fn comparison_with_parentheses_affects_sum() {
    assert_eq!(r("1 + (2 == 2)"), 2.0);
}
