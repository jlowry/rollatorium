mod common;
use common::r;

// ============================================================================
// Comparison Operations
// ============================================================================

#[test]
fn test_eq_true() {
    assert_eq!(r("1 == 1"), 1.0);
}

#[test]
fn test_eq_false() {
    assert_eq!(r("1 == 2"), 0.0);
}

#[test]
fn test_gt_false() {
    assert_eq!(r("1 > 1"), 0.0);
}

#[test]
fn test_gt_true() {
    assert_eq!(r("2 > 1"), 1.0);
}

#[test]
fn test_lt_false() {
    assert_eq!(r("1 < 1"), 0.0);
}

#[test]
fn test_lt_true() {
    assert_eq!(r("1 < 2"), 1.0);
}

#[test]
fn test_gte_true() {
    assert_eq!(r("1 >= 1"), 1.0);
}

#[test]
fn test_gte_false() {
    assert_eq!(r("1 >= 2"), 0.0);
}

#[test]
fn test_lte_true() {
    assert_eq!(r("1 <= 1"), 1.0);
}

#[test]
fn test_lte_false() {
    assert_eq!(r("2 <= 1"), 0.0);
}

#[test]
fn test_neq_false() {
    assert_eq!(r("1 != 1"), 0.0);
}

#[test]
fn test_neq_true() {
    assert_eq!(r("1 != 2"), 1.0);
}
