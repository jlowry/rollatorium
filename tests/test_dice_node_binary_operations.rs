mod common;
use common::r;

use rollatorium::{RollatoriumError, roll};

// ============================================================================
// Node Tests - Binary Operations
// ============================================================================

#[test]
fn test_binop_add() {
    assert_eq!(r("2 + 2"), 4.0);
}

#[test]
fn test_binop_subtract() {
    assert_eq!(r("2 - 2"), 0.0);
}

#[test]
fn test_binop_multiply() {
    assert_eq!(r("2 * 5"), 10.0);
}

#[test]
fn test_binop_float_division() {
    assert_eq!(r("15 / 2"), 7.5); // Rust does float division
}

#[test]
fn test_binop_integer_division() {
    assert_eq!(r("15 // 2"), 7.0); // Integer division
}

#[test]
fn test_binop_modulo() {
    assert_eq!(r("13 % 2"), 1.0);
}

#[test]
fn test_binop_dice_add_range() {
    for _ in 0..1000 {
        let val = r("2 + 1d6");
        assert!((3.0..=8.0).contains(&val), "2 + 1d6 out of range: {}", val);
    }
}

#[test]
fn test_binop_dice_mul_range() {
    for _ in 0..1000 {
        let val = r("2 * 1d6");
        assert!((2.0..=12.0).contains(&val), "2 * 1d6 out of range: {}", val);
    }
}

#[test]
fn test_binop_dice_div_values() {
    for _ in 0..1000 {
        let val = r("60 / 1d6");
        assert!(
            [60.0, 30.0, 20.0, 15.0, 12.0, 10.0].contains(&val),
            "60 / 1d6 unexpected value: {}",
            val
        );
    }
}

#[test]
fn test_binop_dice_intdiv_values() {
    for _ in 0..1000 {
        let val = r("60 // 1d6");
        assert!(
            [60.0, 30.0, 20.0, 15.0, 12.0, 10.0].contains(&val),
            "60 // 1d6 unexpected value: {}",
            val
        );
    }
}

#[test]
fn test_binop_dice_mod_range() {
    for _ in 0..1000 {
        let val = r("1d100 % 10");
        assert!(val <= 10.0, "1d100 % 10 out of range: {}", val);
    }
}

#[test]
fn test_binop_dice_percent_mod_range() {
    for _ in 0..1000 {
        let val = r("1d% % 10");
        assert!(val <= 10.0, "1d% % 10 out of range: {}", val);
    }
}

#[test]
fn test_div_zero_slash() {
    // Division by a zero divisor returns a handled error rather than a
    // non-finite (`inf`) total.
    assert!(matches!(
        roll(&"10 / 0"),
        Err(RollatoriumError::DivisionByZero)
    ));
}

#[test]
fn test_div_zero_double_slash() {
    // Integer division by a zero divisor errors the same way.
    assert!(matches!(
        roll(&"10 // 0"),
        Err(RollatoriumError::DivisionByZero)
    ));
}

#[test]
fn test_div_zero_modulo() {
    // A zero modulus errors rather than producing `NaN`.
    assert!(matches!(
        roll(&"10 % 0"),
        Err(RollatoriumError::DivisionByZero)
    ));
}
