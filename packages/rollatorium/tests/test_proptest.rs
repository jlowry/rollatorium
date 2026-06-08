use proptest::prelude::*;
use rollatorium::{eval, parse};

mod custom_strategies;
use custom_strategies::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        max_shrink_iters: 1000,
        timeout: 3000,
        .. ProptestConfig::default()
    })]

    /// Every valid dice expression should either return a valid result or raise a handled error
    #[test]
    fn test_any_valid_roll(expr in expr_strategy()) {
        match parse(&expr) {
            Ok(ast) => {
                // If it parses, evaluation should either succeed or return a handled error
                match eval(&ast) {
                    Ok(result) => {
                        // Verify the result is sensible
                        prop_assert!(result.total.is_finite());
                    }
                    Err(e) => {
                        // Errors are acceptable - just verify they're handled gracefully
                        let err_msg = format!("{}", e);
                        prop_assert!(!err_msg.is_empty(), "Error should have a message");
                    }
                }
            }
            Err(e) => {
                // Parse errors are acceptable for generated strings
                let err_msg = format!("{}", e);
                prop_assert!(!err_msg.is_empty(), "Error should have a message");
            }
        }
    }

    /// Test that dice expressions with operations don't panic
    #[test]
    fn test_dice_with_operations_no_panic(expr in dice_with_ops_strategy()) {
        let _ = parse(&expr).and_then(|ast| eval(&ast));
    }

    /// Test that set expressions don't panic
    #[test]
    fn test_sets_no_panic(expr in set_expr_strategy()) {
        let _ = parse(&expr).and_then(|ast| eval(&ast));
    }

    /// Test that arithmetic expressions produce finite results
    #[test]
    fn test_arithmetic_finite(expr in arithmetic_strategy()) {
        if let Ok(ast) = parse(&expr)
            && let Ok(result) = eval(&ast) {
                prop_assert!(result.total.is_finite(), "Result should be finite: {}", result.total);
            }
    }
}
