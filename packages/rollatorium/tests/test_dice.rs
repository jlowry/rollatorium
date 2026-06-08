mod common;
use common::r;
use rollatorium::roll;

// Standard test expressions from Python reference
const STANDARD_EXPRESSIONS: &[&str] = &[
    "1d20",
    "1d%",
    "1+1",
    "4d6kh3",
    "(1)",
    "(1,)",
    "((1d6))",
    "4*(3d8kh2+9[fire]+(9d2e2+3[cold])/2)",
    "(1d4, 2+2, 3d6kl1)kh1",
    "((10d6kh5)kl2)kh1",
];

const GRAMMAR_EXPRESSIONS: &[&str] = &[
    " -1d6 + +2 ",
    "3d%kh1 // 2",
    "4d6kh3pl1[damage][fire]",
    "(1d4, 2d6, 3d8ro<3)k>1[set]",
];

// ============================================================================
// High Level Tests
// ============================================================================

#[test]
fn test_rolls_dont_error() {
    for expr in STANDARD_EXPRESSIONS {
        assert!(roll(expr).is_ok(), "Failed to roll: {}", expr);
    }
}

#[test]
fn test_extended_grammar_rolls_dont_error() {
    for expr in GRAMMAR_EXPRESSIONS {
        assert!(roll(expr).is_ok(), "Failed to roll: {}", expr);
    }
}

#[test]
fn test_roll_types() {
    for expr in STANDARD_EXPRESSIONS {
        let result = roll(expr).unwrap();
        assert!(result.total.is_finite());
        // In Rust, we have EvalResult with total and value
        // The value field contains the detailed roll structure
    }
}

#[test]
fn test_sane_total_1d20() {
    for _ in 0..1000 {
        let val = r("1d20");
        assert!((1.0..=20.0).contains(&val), "1d20 out of range: {}", val);
    }
}

#[test]
fn test_sane_total_1d_percent() {
    for _ in 0..1000 {
        let val = r("1d%");
        assert!((0.0..=90.0).contains(&val), "1d% out of range: {}", val);
        assert_eq!(val % 10.0, 0.0, "1d% not multiple of 10: {}", val);
    }
}

#[test]
fn test_floor_division_produces_integers() {
    for _ in 0..1000 {
        let val = r("4d6kh3 // 2");
        assert!(
            (1.0..=9.0).contains(&val),
            "4d6kh3 // 2 out of range: {}",
            val
        );
        assert_eq!(val.fract(), 0.0, "4d6kh3 // 2 not an integer: {}", val);
    }
}

#[test]
fn test_modulo_restricts_range() {
    for _ in 0..1000 {
        let val = r("5d4kh3 % 3");
        assert!(
            (0.0..=2.0).contains(&val),
            "5d4kh3 % 3 out of range: {}",
            val
        );
        assert_eq!(val.fract(), 0.0, "5d4kh3 % 3 not an integer: {}", val);
    }
}

#[test]
fn test_sane_total_4d6kh3() {
    for _ in 0..1000 {
        let val = r("4d6kh3");
        assert!((3.0..=18.0).contains(&val), "4d6kh3 out of range: {}", val);
    }
}

#[test]
fn test_sane_total_nested_1d6() {
    for _ in 0..1000 {
        let val = r("(((1d6)))");
        assert!(
            (1.0..=6.0).contains(&val),
            "(((1d6))) out of range: {}",
            val
        );
    }
}

#[test]
fn test_sane_total_set_kh1() {
    for _ in 0..1000 {
        let val = r("(1d4, 2+2, 3d6kl1)kh1");
        assert!(
            (4.0..=6.0).contains(&val),
            "(1d4, 2+2, 3d6kl1)kh1 out of range: {}",
            val
        );
    }
}

#[test]
fn test_sane_total_complex_nested_kh_kl() {
    for _ in 0..1000 {
        let val = r("((10d6kh5)kl2)kh1");
        assert!(
            (1.0..=6.0).contains(&val),
            "((10d6kh5)kl2)kh1 out of range: {}",
            val
        );
    }
}

#[test]
fn test_annotations_do_not_change_totals() {
    use rand::{SeedableRng, rngs::StdRng};
    use rollatorium::{EvalConfig, eval_with_rng, parse};

    let seed = 0x0BAD_5EED;
    let base_ast = parse(&"4d6kh3").unwrap();
    let annotated_ast = parse(&"4d6kh3[fire][damage]").unwrap();

    let base_result = eval_with_rng(
        &base_ast,
        EvalConfig::default(),
        StdRng::seed_from_u64(seed),
    )
    .unwrap();
    let annotated_result = eval_with_rng(
        &annotated_ast,
        EvalConfig::default(),
        StdRng::seed_from_u64(seed),
    )
    .unwrap();

    assert_eq!(base_result.total, annotated_result.total);
}
