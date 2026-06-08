mod common;
use common::r;

// ============================================================================
// Node Tests - Literals
// ============================================================================
#[test]
fn test_literal_integer_one() {
    assert_eq!(r("1"), 1.0);
}

#[test]
fn test_literal_large_integer() {
    assert_eq!(r("10000"), 10000.0);
}

#[test]
fn test_literal_float_one_point_five() {
    assert_eq!(r("1.5"), 1.5); // Rust preserves float values
}

#[test]
fn test_literal_float_zero_point_five() {
    assert_eq!(r("0.5"), 0.5);
}

#[test]
fn test_literal_float_dot_five() {
    assert_eq!(r(".5"), 0.5);
}
