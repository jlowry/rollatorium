#![cfg(feature = "serde")]
//! Round-trip tests for the optional `serde` feature.

use rollatorium::{EvalResult, Node, roll};

#[test]
fn test_serde_node_round_trip() {
    let node: Node<String> = "4d6kh3 [fire]".parse().unwrap();
    let json = serde_json::to_string(&node).unwrap();
    let back: Node<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(node, back);
}

#[test]
fn test_serde_eval_result_round_trip() {
    // EvalResult contains `f64` trees and so has no `PartialEq`; compare the
    // structural `Debug` output instead.
    let result: EvalResult<String> = roll("2d6 + 1").unwrap().map_tags(|t| t.to_owned());
    let json = serde_json::to_string(&result).unwrap();
    let back: EvalResult<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{result:?}"), format!("{back:?}"));
}
