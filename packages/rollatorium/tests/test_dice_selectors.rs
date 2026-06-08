mod common;
use common::r;

// ============================================================================
// Selectors
// ============================================================================

#[test]
fn test_selector_k3() {
    assert_eq!(r("(1, 2, 3, 4, 5)k3"), 3.0);
}

#[test]
fn test_selector_k_less_than_3() {
    assert_eq!(r("(1, 2, 3, 4, 5)k<3"), 3.0);
}

#[test]
fn test_selector_k_greater_than_3() {
    assert_eq!(r("(1, 2, 3, 4, 5)k>3"), 9.0);
}

#[test]
fn test_selector_kl2() {
    assert_eq!(r("(1, 2, 3, 4, 5)kl2"), 3.0);
}

#[test]
fn test_selector_kh2() {
    assert_eq!(r("(1, 2, 3, 4, 5)kh2"), 9.0);
}

#[test]
fn test_selector_k1_single_element() {
    assert_eq!(r("(1)k1"), 1.0);
}

#[test]
fn test_selector_k2_single_element() {
    assert_eq!(r("(1)k2"), 0.0);
}

#[test]
fn test_selector_k_equal_to_literal() {
    assert_eq!(r("(1, 2, 3, 4)k==3"), 3.0);
}

#[test]
fn test_selector_k_not_equal_literal() {
    assert_eq!(r("(1, 2, 3, 4)k!=3"), 7.0);
}

#[test]
fn test_selector_k_greater_equal_expression() {
    assert_eq!(r("(1, 2, 3, 4)k>=(2+1)"), 7.0);
}

#[test]
fn test_selector_k_less_equal_expression() {
    assert_eq!(r("(1, 2, 3, 4)k<=(1d1+1)"), 3.0);
}

#[test]
fn test_selector_kh_dynamic_count_expression() {
    assert_eq!(r("(1, 2, 3, 4)kh(1+1)"), 7.0);
}
