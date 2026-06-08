mod common;
use common::r;

// ============================================================================
// Dice Operators - Explode
// ============================================================================

#[test]
fn test_e_op() {
    for _ in 0..100 {
        let val = r("1d2e2");
        assert_eq!(val % 2.0, 1.0, "1d2e2 should be odd: {}", val);
    }
}

#[test]
#[should_panic(expected = "Exceeded maximum number of rolls")]
fn test_e_op_infinite_loop_under() {
    let _ = r("1d20e<21");
}

#[test]
#[should_panic(expected = "Exceeded maximum number of rolls")]
fn test_e_op_infinite_loop_all() {
    let _ = r("1d1e1");
}
