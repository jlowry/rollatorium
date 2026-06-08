mod common;
use common::r;

// ============================================================================
// Set Operators - Keep
// ============================================================================

#[test]
fn test_keep_lowest_n() {
    assert_eq!(r("(1, 2, 3, 4, 5)k3"), 3.0);
}

#[test]
fn test_chained_keep_lowest() {
    assert_eq!(r("(1, 2, 3, 4, 5)k1k2"), 3.0);
}

#[test]
fn test_keep_highest_and_lowest() {
    assert_eq!(r("(1, 2, 3, 4, 5)kh1kl1"), 6.0);
}
