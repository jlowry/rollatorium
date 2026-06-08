mod common;
use common::r;

// ============================================================================
// Dice Operators - Reroll Once
// ============================================================================

#[test]
fn test_reroll_once_exact_match() {
    assert_eq!(r("1d1ro1"), 1.0);
}

#[test]
fn test_reroll_once_range() {
    for _ in 0..100 {
        let val = r("1d6rol1");
        assert!((1.0..=6.0).contains(&val), "1d6rol1 out of range: {}", val);
    }
}
