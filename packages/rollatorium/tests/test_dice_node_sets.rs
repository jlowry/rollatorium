mod common;
use common::r;
// ============================================================================
// Node Tests - Sets
// ============================================================================

#[test]
fn test_single_element_set() {
    assert_eq!(r("(1)"), 1.0);
}

#[test]
fn test_single_element_set_with_comma() {
    assert_eq!(r("(1,)"), 1.0);
}

#[test]
fn test_two_element_set() {
    assert_eq!(r("(1, 1)"), 2.0);
}
