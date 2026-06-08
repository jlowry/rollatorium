mod common;
use common::r;

// ============================================================================
// Dice Operators - Minimum
// ============================================================================

#[test]
fn test_mi_op_all_dice_min_6() {
    assert_eq!(r("10d6mi6"), 60.0);
}

#[test]
fn test_mi_op_all_dice_min_10() {
    assert_eq!(r("10d6mi10"), 100.0);
}

#[test]
fn test_mi_op_min_2_in_range() {
    for _ in 0..100 {
        let val = r("10d6mi2");
        assert!(
            (20.0..=60.0).contains(&val),
            "10d6mi2 out of range: {}",
            val
        );
    }
}
