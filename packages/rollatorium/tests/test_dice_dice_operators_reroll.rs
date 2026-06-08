mod common;
use common::r;

// ============================================================================
// Dice Operators - Reroll
// ============================================================================

#[test]
fn test_rr_op_reroll_until_20() {
    assert_eq!(r("1d20rr<20"), 20.0);
}

#[test]
fn test_rr_op_reroll_until_1() {
    assert_eq!(r("1d20rr>1"), 1.0);
}

#[test]
#[should_panic(expected = "Exceeded maximum number of rolls")]
fn test_rr_op_infinite_loop_under() {
    let _ = r("1d20rr<21");
}

#[test]
#[should_panic(expected = "Exceeded maximum number of rolls")]
fn test_rr_op_infinite_loop_all() {
    let _ = r("1d1rr1");
}
