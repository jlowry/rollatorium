mod common;
use common::r;
// ============================================================================
// Node Tests - Dice
// ============================================================================

#[test]
fn test_zero_dice_returns_zero() {
    for _ in 0..1000 {
        assert_eq!(r("0d6"), 0.0);
    }
}

#[test]
fn test_single_die_no_count() {
    for _ in 0..1000 {
        let val = r("d6");
        assert!((1.0..=6.0).contains(&val), "d6 out of range: {}", val);
    }
}

#[test]
fn test_single_die_with_count() {
    for _ in 0..1000 {
        let val = r("1d6");
        assert!((1.0..=6.0).contains(&val), "1d6 out of range: {}", val);
    }
}

#[test]
fn test_two_dice() {
    for _ in 0..1000 {
        let val = r("2d6");
        assert!((2.0..=12.0).contains(&val), "2d6 out of range: {}", val);
    }
}

#[test]
fn test_zero_d_percent_returns_zero() {
    for _ in 0..1000 {
        assert_eq!(r("0d%"), 0.0);
    }
}

#[test]
fn test_single_d_percent_no_count() {
    for _ in 0..1000 {
        let val = r("d%");
        assert!((0.0..=90.0).contains(&val), "d% out of range: {}", val);
    }
}

#[test]
fn test_single_d_percent_with_count() {
    for _ in 0..1000 {
        let val = r("1d%");
        assert!((0.0..=90.0).contains(&val), "1d% out of range: {}", val);
    }
}

#[test]
fn test_two_d_percent() {
    for _ in 0..1000 {
        let val = r("2d%");
        assert!((0.0..=180.0).contains(&val), "2d% out of range: {}", val);
    }
}
