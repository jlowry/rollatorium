mod common;
use common::r;

// ============================================================================
// Invalid Rolls
// ============================================================================

#[test]
#[should_panic(expected = "Exceeded maximum number of rolls")]
fn test_too_many_rolls() {
    let _ = r("1001d6");
}

#[test]
#[should_panic(expected = "die size must be positive")]
fn test_zero_sided_die() {
    let _ = r("6d0");
}

#[test]
#[should_panic(expected = "selector target must be positive")]
fn test_invalid_minimum() {
    let _ = r("10d6mil1");
}

#[test]
#[should_panic(expected = "DivisionByZero")]
fn test_divide_by_zero() {
    let _ = r("967 / 0");
}

#[test]
#[should_panic(expected = "DivisionByZero")]
fn test_int_divide_by_zero() {
    let _ = r("10 // 0");
}

#[test]
#[should_panic(expected = "DivisionByZero")]
fn test_modulo_by_zero() {
    let _ = r("5 % 0");
}

// A dice/set expression that drops all of its dice totals to zero is also a
// zero divisor (the shrunk `test_any_valid_roll` counterexample), not just a
// literal `0`.
#[test]
#[should_panic(expected = "DivisionByZero")]
fn test_divide_by_dropped_dice_total() {
    let _ = r("1 / (1, 2)p<9");
}
