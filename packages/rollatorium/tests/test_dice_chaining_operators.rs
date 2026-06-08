mod common;
use common::r;

// ============================================================================
// Chaining Operators
// ============================================================================
#[test]
fn test_chaining_keep_operators() {
    for _ in 0..100 {
        let val = r("10d6k1k2k3");
        assert!(
            (0.0..=30.0).contains(&val),
            "10d6k1k2k3 out of range: {}",
            val
        );
    }
}

#[test]
fn test_chaining_keep_and_drop_operators() {
    for _ in 0..100 {
        let val = r("10d6k1ph1");
        assert!(
            (0.0..=9.0).contains(&val),
            "10d6k1ph1 out of range: {}",
            val
        );
    }
}

#[test]
fn test_chaining_keep_on_literal_set() {
    assert_eq!(r("(1, 2, 3)k1k2"), 3.0);
}

// TODO: Implement CritType tracking
// #[test]
// fn test_crit() {
//     // roll until we get a crit
//     loop {
//         let result = roll("1d20").unwrap();
//         if result.total == 20.0 {
//             assert_eq!(result.crit, CritType::Crit);
//             break;
//         }
//     }
// }
