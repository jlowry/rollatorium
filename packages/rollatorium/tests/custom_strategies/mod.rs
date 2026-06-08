//! Custom proptest strategies for property-based testing of dice expressions.

use proptest::prelude::*;

/// Main expression strategy - entry point for generating dice expressions
pub fn expr_strategy() -> impl Strategy<Value = String> {
    num_strategy()
}

/// Numeric expression with optional comparison operators
fn num_strategy() -> impl Strategy<Value = String> {
    comparison_strategy()
}

/// Comparison expressions: a_num (comp_op a_num)*
fn comparison_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(a_num_strategy(), 1..=3).prop_flat_map(|parts| {
        if parts.len() == 1 {
            Just(parts[0].clone()).boxed()
        } else {
            let comp_op = prop::sample::select(vec!["==", ">=", "<=", "!=", "<", ">"]);
            comp_op
                .prop_map(move |op| {
                    parts
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(&format!(" {} ", op))
                })
                .boxed()
        }
    })
}

/// Additive expression: m_num (add_op m_num)*
fn a_num_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(m_num_strategy(), 1..=3).prop_flat_map(|parts| {
        if parts.len() == 1 {
            Just(parts[0].clone()).boxed()
        } else {
            let add_op = prop::sample::select(vec!["+", "-"]);
            add_op
                .prop_map(move |op| {
                    parts
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(&format!(" {} ", op))
                })
                .boxed()
        }
    })
}

/// Multiplicative expression: u_num (mul_op u_num)*
fn m_num_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(u_num_strategy(), 1..=3).prop_flat_map(|parts| {
        if parts.len() == 1 {
            Just(parts[0].clone()).boxed()
        } else {
            let mul_op = prop::sample::select(vec!["*", "//", "/", "%"]);
            mul_op
                .prop_map(move |op| {
                    parts
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(&format!(" {} ", op))
                })
                .boxed()
        }
    })
}

/// Unary expression: (unary_op)* numexpr
fn u_num_strategy() -> impl Strategy<Value = String> {
    // Limit recursion depth to avoid stack overflow
    prop::sample::select(vec!["+", "-", ""])
        .prop_flat_map(|prefix| {
            numexpr_strategy().prop_map(move |expr| format!("{}{}", prefix, expr))
        })
        .boxed()
}

/// Base numeric expression: dice | set | literal, optionally with annotations
fn numexpr_strategy() -> impl Strategy<Value = String> {
    (
        prop_oneof![dice_strategy(), set_strategy(), literal_strategy(),],
        prop::collection::vec(annotation_strategy(), 0..=2),
    )
        .prop_map(|(expr, annotations)| format!("{}{}", expr, annotations.join("")))
}

/// Literal number: integer or decimal
fn literal_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (0u32..1000u32).prop_map(|n| n.to_string()),
        decimal_strategy(),
    ]
}

/// Decimal number strategy
fn decimal_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // a.b format
        (0u32..100u32, 0u32..1000u32).prop_map(|(a, b)| format!("{}.{}", a, b)),
        // .b format
        (1u32..1000u32).prop_map(|b| format!(".{}", b)),
    ]
}

/// Set expression with optional operations: (expr, ...) set_op*
fn set_strategy() -> impl Strategy<Value = String> {
    (
        setexpr_strategy(),
        prop::collection::vec(set_op_strategy(), 0..=2),
    )
        .prop_map(|(set, ops)| format!("{}{}", set, ops.join("")))
}

/// Set operation: k|p selector
fn set_op_strategy() -> impl Strategy<Value = String> {
    (prop::sample::select(vec!["k", "p"]), selector_strategy())
        .prop_map(|(op, sel)| format!("{}{}", op, sel))
}

/// Set expression: () | (num, ...)
fn setexpr_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("()".to_string()),
        prop::collection::vec(literal_strategy(), 1..=5)
            .prop_map(|nums| { format!("({})", nums.join(", ")) }),
    ]
}

/// Dice expression with optional operations
pub fn dice_with_ops_strategy() -> impl Strategy<Value = String> {
    dice_strategy()
}

/// Dice expression: [quantity]d[size] dice_op*
fn dice_strategy() -> impl Strategy<Value = String> {
    (
        diceexpr_strategy(),
        prop::collection::vec(dice_op_strategy(), 0..=3),
    )
        .prop_map(|(dice, ops)| format!("{}{}", dice, ops.join("")))
}

/// Dice operation: (rr|ro|ra|e|mi|ma|k|p) selector
fn dice_op_strategy() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec!["rr", "ro", "ra", "e", "mi", "ma", "k", "p"]),
        selector_strategy(),
    )
        .prop_map(|(op, sel)| format!("{}{}", op, sel))
}

/// Dice expression: [quantity]d[size]
fn diceexpr_strategy() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("".to_string()),
            (1u32..=20u32).prop_map(|n| n.to_string()),
        ],
        prop_oneof![
            (1u32..=100u32).prop_map(|n| n.to_string()),
            Just("%".to_string()),
        ],
    )
        .prop_map(|(qty, size)| format!("{}d{}", qty, size))
}

/// Selector: [type]count where type is h|l|<|>
fn selector_strategy() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec!["", "h", "l", "<", ">", "==", "!="]),
        (1u32..=10u32),
    )
        .prop_map(|(sel_type, count)| format!("{}{}", sel_type, count))
}

/// Annotation: [text]
fn annotation_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,20}".prop_map(|text| format!("[{}]", text))
}

/// Set expression for focused testing
pub fn set_expr_strategy() -> impl Strategy<Value = String> {
    set_strategy()
}

/// Arithmetic expression (no dice) for focused testing
pub fn arithmetic_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(literal_strategy(), 1..=5).prop_flat_map(|parts| {
        let ops = prop::collection::vec(
            prop::sample::select(vec!["+", "-", "*", "/", "//"]),
            parts.len() - 1,
        );
        ops.prop_map(move |operators| {
            let mut result = parts[0].clone();
            for (i, op) in operators.iter().enumerate() {
                result.push_str(&format!(" {} {}", op, parts[i + 1]));
            }
            result
        })
    })
}
