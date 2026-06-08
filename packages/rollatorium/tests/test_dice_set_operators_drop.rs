mod common;
use common::r;

// ============================================================================
// Set Operators - Drop
// ============================================================================

#[test]
fn test_drop_p3() {
    assert_eq!(r("(1, 2, 3, 4, 5)p3"), 12.0);
}

#[test]
fn test_drop_p1p2() {
    assert_eq!(r("(1, 2, 3, 4, 5)p1p2"), 12.0);
}

#[test]
fn test_drop_ph1pl1() {
    assert_eq!(r("(1, 2, 3, 4, 5)ph1pl1"), 9.0);
}
