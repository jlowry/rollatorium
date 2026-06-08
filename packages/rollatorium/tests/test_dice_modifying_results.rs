// ============================================================================
// Modifying Results (TODO: Implement if needed)
// ============================================================================

// TODO: The Python version allows modifying the AST after rolling.
// This may not be a pattern we want to support in Rust's type system,
// but if needed, we could implement similar functionality with interior mutability.
//
// #[test]
// fn test_correct_results() {
//     let mut result = roll("1+2+3").unwrap();
//     assert_eq!(result.total, 6.0);
//     // result.expr.roll = BinOp(result.expr.roll, "+", Literal(4));
//     // assert_eq!(result.total, 10.0);
// }
