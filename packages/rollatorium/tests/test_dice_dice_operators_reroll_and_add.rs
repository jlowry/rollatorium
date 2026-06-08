mod common;
use common::r;

// ============================================================================
// Dice Operators - Reroll and Add
// ============================================================================

#[test]
fn test_ra_op_1d1ra1_returns_2() {
    assert_eq!(r("1d1ra1"), 2.0);
}

#[test]
fn test_ra_op_1d6ral1_within_expected_range() {
    for _ in 0..100 {
        let val = r("1d6ral1");
        assert!((2.0..=12.0).contains(&val), "1d6ral1 out of range: {}", val);
    }
}
