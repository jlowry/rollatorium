use rand::{SeedableRng, rngs::StdRng};
use rollatorium::{EvalConfig, Value, eval_with_rng, parse};

#[derive(Clone, Debug, PartialEq)]
enum Damage {
    Fire,
    Cold,
    Other,
}

impl From<&str> for Damage {
    fn from(value: &str) -> Self {
        match value {
            "fire" => Damage::Fire,
            "cold" => Damage::Cold,
            _ => Damage::Other,
        }
    }
}

#[test]
fn test_map_tags_converts_str_to_enum() {
    let node = parse(&"4d6 [fire]").expect("parse").map_tags(Damage::from);

    let result = eval_with_rng(&node, EvalConfig::default(), StdRng::seed_from_u64(3)).unwrap();

    match result.value {
        Value::Annotated { annotations, .. } => {
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0].tag, Damage::Fire);
        }
        other => panic!("expected annotated value, got {:?}", other),
    }
}

#[test]
fn test_map_tags_preserves_totals_and_multiple_tags() {
    let seed = 0xBEEF_u64;
    let parsed = parse(&"3d6[cold][unknown]").expect("parse");

    // Total before mapping.
    let before = eval_with_rng(&parsed, EvalConfig::default(), StdRng::seed_from_u64(seed))
        .unwrap()
        .total;

    let mapped = parsed.map_tags(Damage::from);
    let after_result =
        eval_with_rng(&mapped, EvalConfig::default(), StdRng::seed_from_u64(seed)).unwrap();

    assert!((before - after_result.total).abs() < 1e-9);

    match after_result.value {
        Value::Annotated { annotations, .. } => {
            let tags: Vec<_> = annotations.iter().map(|a| a.tag.clone()).collect();
            assert_eq!(tags, vec![Damage::Cold, Damage::Other]);
        }
        other => panic!("expected annotated value, got {:?}", other),
    }
}

#[test]
fn test_map_tags_on_eval_result() {
    // Retagging after evaluation works symmetrically.
    let parsed = parse(&"2d4 [fire]").expect("parse");
    let result = eval_with_rng(&parsed, EvalConfig::default(), StdRng::seed_from_u64(5)).unwrap();
    let total = result.total;

    let mapped = result.map_tags(Damage::from);
    assert!((mapped.total - total).abs() < 1e-9);
    match mapped.value {
        Value::Annotated { annotations, .. } => {
            assert_eq!(annotations[0].tag, Damage::Fire);
        }
        other => panic!("expected annotated value, got {:?}", other),
    }
}
