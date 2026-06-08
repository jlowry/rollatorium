mod common;
use common::r;

// ============================================================================
// Dice Operators - Maximum
// ============================================================================

#[test]
fn test_ma_op_ma1() {
    assert_eq!(r("10d6ma1"), 10.0);
}

#[test]
fn test_ma_op_ma0() {
    assert_eq!(r("10d6ma0"), 0.0);
}

#[test]
fn test_ma_op_ma5_in_range() {
    for _ in 0..100 {
        let val = r("10d6ma5");
        assert!(
            (10.0..=50.0).contains(&val),
            "10d6ma5 out of range: {}",
            val
        );
    }
}
