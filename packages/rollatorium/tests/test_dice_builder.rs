use rand::{SeedableRng, rngs::StdRng};
use rollatorium::{
    EvalConfig, Value,
    build::{Compare, Roll},
    eval_with_rng, parse,
};

#[derive(Clone, Debug, PartialEq)]
enum Damage {
    Slashing,
    Fire,
}

#[test]
fn test_builder_tags_attach_at_wrapper_level() {
    // 4d6kh3 [Slashing] + 2d6 [Fire]
    let expr = Roll::<Damage>::dice(4, 6)
        .keep_highest(3)
        .tag(Damage::Slashing)
        .add(Roll::dice(2, 6).tag(Damage::Fire))
        .build();

    let result = eval_with_rng(&expr, EvalConfig::default(), StdRng::seed_from_u64(7)).unwrap();

    match result.value {
        Value::Binary { left, right, .. } => {
            match left.value {
                Value::Annotated { annotations, expr } => {
                    assert_eq!(annotations.len(), 1);
                    assert_eq!(annotations[0].tag, Damage::Slashing);
                    assert!(matches!(expr.value, Value::Dice(_)));
                }
                other => panic!("expected annotated dice on the left, got {:?}", other),
            }
            match right.value {
                Value::Annotated { annotations, expr } => {
                    assert_eq!(annotations.len(), 1);
                    assert_eq!(annotations[0].tag, Damage::Fire);
                    assert!(matches!(expr.value, Value::Dice(_)));
                }
                other => panic!("expected annotated dice on the right, got {:?}", other),
            }
        }
        other => panic!("expected a binary sum, got {:?}", other),
    }
}

#[test]
fn test_builder_matches_parsed_expression() {
    // The builder produces the same evaluatable tree as the parser; with the
    // same seed, totals match. Tags do not consume RNG.
    let seed = 0xD20_u64;

    let built = Roll::<Damage>::dice(4, 6)
        .keep_highest(3)
        .tag(Damage::Slashing)
        .add(Roll::dice(2, 6).tag(Damage::Fire))
        .build();
    let parsed = parse(&"4d6kh3[slashing] + 2d6[fire]").unwrap();

    let built_total = eval_with_rng(&built, EvalConfig::default(), StdRng::seed_from_u64(seed))
        .unwrap()
        .total;
    let parsed_total = eval_with_rng(&parsed, EvalConfig::default(), StdRng::seed_from_u64(seed))
        .unwrap()
        .total;

    assert!((built_total - parsed_total).abs() < 1e-9);
}

#[test]
fn test_builder_compare_selectors_match_parsed() {
    let seed = 99_u64;

    // 6d6rr<3e==6
    let built = Roll::<&str>::dice(6, 6)
        .reroll(Compare::Lt, 3.0)
        .explode(Compare::Eq, 6.0)
        .build();
    let parsed = parse(&"6d6rr<3e==6").unwrap();

    let built_total = eval_with_rng(&built, EvalConfig::default(), StdRng::seed_from_u64(seed))
        .unwrap()
        .total;
    let parsed_total = eval_with_rng(&parsed, EvalConfig::default(), StdRng::seed_from_u64(seed))
        .unwrap()
        .total;

    assert!((built_total - parsed_total).abs() < 1e-9);
}

#[test]
fn test_builder_multiple_tags() {
    let expr = Roll::<&str>::dice(2, 8).tags(["fire", "magic"]).build();
    let result = eval_with_rng(&expr, EvalConfig::default(), StdRng::seed_from_u64(1)).unwrap();

    match result.value {
        Value::Annotated { annotations, .. } => {
            let tags: Vec<_> = annotations.iter().map(|a| a.tag).collect();
            assert_eq!(tags, ["fire", "magic"]);
        }
        other => panic!("expected annotated value, got {:?}", other),
    }
}
