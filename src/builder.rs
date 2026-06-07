//! A fluent builder for constructing dice-expression ASTs in code.
//!
//! The builder produces the same opaque [`Node`] that [`crate::parse`] yields
//! and [`crate::eval`] consumes, but with caller-defined tags instead of the
//! borrowed `&str` tags produced by parsing. The internal `Node` representation
//! stays private; callers compose expressions through the named combinators on
//! [`Roll`] and finish with [`Roll::build`].
//!
//! ```
//! use rollatorium::build::dice;
//! use rollatorium::eval;
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum Damage {
//!     Slashing,
//!     Fire,
//! }
//!
//! // 4d6kh3 [Slashing] + 2d6 [Fire]
//! let expr = dice::<Damage>(4, 6)
//!     .keep_highest(3)
//!     .tag(Damage::Slashing)
//!     .add(dice(2, 6).tag(Damage::Fire))
//!     .build();
//!
//! let result = eval(&expr).unwrap();
//! assert!(result.total >= 5.0);
//! ```
//!
//! Apply dice operations *before* tagging a sub-expression: operations attach to
//! a bare dice/set node, whereas a tag wraps the whole thing.

use crate::ast::{
    Annotation, BinaryOperator, DiceSize, Node, Selector, SelectorKind, SetOperation, SetOperator,
    UnaryOperator,
};

/// A comparison used by selector-based dice operations such as
/// [`Roll::reroll`], [`Roll::explode`], [`Roll::keep_where`], and
/// [`Roll::drop_where`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compare {
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    Ge,
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`<=`).
    Le,
    /// Equal (`==`).
    Eq,
    /// Not equal (`!=`).
    Ne,
}

impl Compare {
    fn kind(self) -> SelectorKind {
        match self {
            Compare::Gt => SelectorKind::GreaterThan,
            Compare::Ge => SelectorKind::GreaterThanOrEqual,
            Compare::Lt => SelectorKind::LessThan,
            Compare::Le => SelectorKind::LessThanOrEqual,
            Compare::Eq => SelectorKind::EqualTo,
            Compare::Ne => SelectorKind::NotEqual,
        }
    }
}

/// An expression under construction.
///
/// Build one with a constructor ([`lit`], [`dice`], [`die`], [`d_percent`],
/// [`set`]), chain combinators, then call [`Roll::build`] to obtain the opaque
/// [`Node`] for [`crate::eval`].
#[must_use = "a `Roll` is inert until you call `.build()` and evaluate the resulting node"]
pub struct Roll<T> {
    node: Node<T>,
}

// The fluent `add`/`sub`/`mul`/`div`/`rem`/`neg` names intentionally mirror the
// arithmetic they build; they are not the std operator traits.
#[allow(clippy::should_implement_trait)]
impl<T> Roll<T> {
    fn wrap(node: Node<T>) -> Self {
        Self { node }
    }

    /// Finish building and hand back the opaque AST for [`crate::eval`].
    pub fn build(self) -> Node<T> {
        self.node
    }

    fn binary(self, operator: BinaryOperator, rhs: Roll<T>) -> Roll<T> {
        Roll::wrap(Node::Binary {
            operator,
            left: Box::new(self.node),
            right: Box::new(rhs.node),
        })
    }

    fn unary(self, operator: UnaryOperator) -> Roll<T> {
        Roll::wrap(Node::Unary {
            operator,
            operand: Box::new(self.node),
        })
    }

    fn operation(self, operator: SetOperator, kind: SelectorKind, value: f64) -> Roll<T> {
        let op = SetOperation {
            operator,
            selectors: vec![Selector {
                kind,
                target: Box::new(Node::Literal(value)),
            }],
        };
        Roll::wrap(attach_operation(self.node, op))
    }

    // ----- arithmetic -----

    /// Add another expression (`self + rhs`).
    pub fn add(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Add, rhs)
    }

    /// Subtract another expression (`self - rhs`).
    pub fn sub(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Subtract, rhs)
    }

    /// Multiply by another expression (`self * rhs`).
    pub fn mul(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Multiply, rhs)
    }

    /// Divide by another expression (`self / rhs`).
    pub fn div(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Divide, rhs)
    }

    /// Integer (truncating) division (`self // rhs`).
    pub fn int_div(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::IntDivide, rhs)
    }

    /// Remainder (`self % rhs`).
    pub fn rem(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Modulo, rhs)
    }

    /// Equality comparison (`self == rhs`), yielding `1.0`/`0.0`.
    pub fn eq(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Equal, rhs)
    }

    /// Inequality comparison (`self != rhs`).
    pub fn ne(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::NotEqual, rhs)
    }

    /// Greater-than comparison (`self > rhs`).
    pub fn gt(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Greater, rhs)
    }

    /// Greater-or-equal comparison (`self >= rhs`).
    pub fn ge(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::GreaterEqual, rhs)
    }

    /// Less-than comparison (`self < rhs`).
    pub fn lt(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::Less, rhs)
    }

    /// Less-or-equal comparison (`self <= rhs`).
    pub fn le(self, rhs: Roll<T>) -> Roll<T> {
        self.binary(BinaryOperator::LessEqual, rhs)
    }

    /// Unary negation (`-self`).
    pub fn neg(self) -> Roll<T> {
        self.unary(UnaryOperator::Minus)
    }

    /// Unary plus (`+self`).
    pub fn pos(self) -> Roll<T> {
        self.unary(UnaryOperator::Plus)
    }

    // ----- dice operations -----

    /// Keep the `n` highest dice (`kh{n}`).
    pub fn keep_highest(self, n: u32) -> Roll<T> {
        self.operation(SetOperator::Keep, SelectorKind::Highest, f64::from(n))
    }

    /// Keep the `n` lowest dice (`kl{n}`).
    pub fn keep_lowest(self, n: u32) -> Roll<T> {
        self.operation(SetOperator::Keep, SelectorKind::Lowest, f64::from(n))
    }

    /// Drop the `n` highest dice (`ph{n}`).
    pub fn drop_highest(self, n: u32) -> Roll<T> {
        self.operation(SetOperator::Drop, SelectorKind::Highest, f64::from(n))
    }

    /// Drop the `n` lowest dice (`pl{n}`).
    pub fn drop_lowest(self, n: u32) -> Roll<T> {
        self.operation(SetOperator::Drop, SelectorKind::Lowest, f64::from(n))
    }

    /// Keep dice matching `cmp value` (e.g. `k>4`).
    pub fn keep_where(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::Keep, cmp.kind(), value)
    }

    /// Drop dice matching `cmp value` (e.g. `p<2`).
    pub fn drop_where(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::Drop, cmp.kind(), value)
    }

    /// Reroll matching dice until none match (`rr`).
    pub fn reroll(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::Reroll, cmp.kind(), value)
    }

    /// Reroll matching dice once (`ro`).
    pub fn reroll_once(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::RerollOnce, cmp.kind(), value)
    }

    /// Reroll matching dice, keeping the original and adding the new die (`ra`).
    pub fn reroll_add(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::RerollAdd, cmp.kind(), value)
    }

    /// Explode matching dice, rolling an extra die for each match (`e`/`!`).
    pub fn explode(self, cmp: Compare, value: f64) -> Roll<T> {
        self.operation(SetOperator::Explode, cmp.kind(), value)
    }

    /// Clamp each die up to at least `value` (`mi{value}`).
    pub fn min(self, value: f64) -> Roll<T> {
        self.operation(SetOperator::Minimum, SelectorKind::Literal, value)
    }

    /// Clamp each die down to at most `value` (`ma{value}`).
    pub fn max(self, value: f64) -> Roll<T> {
        self.operation(SetOperator::Maximum, SelectorKind::Literal, value)
    }

    // ----- tagging -----

    /// Attach a single tag to this expression.
    pub fn tag(self, tag: T) -> Roll<T> {
        self.tags([tag])
    }

    /// Attach multiple tags to this expression.
    pub fn tags(self, tags: impl IntoIterator<Item = T>) -> Roll<T> {
        let annotations: Vec<Annotation<T>> =
            tags.into_iter().map(|tag| Annotation { tag }).collect();
        Roll::wrap(attach_tags(self.node, annotations))
    }
}

fn attach_operation<T>(node: Node<T>, op: SetOperation<T>) -> Node<T> {
    match node {
        Node::Dice { num, size } => Node::DiceWithOps {
            dice: Box::new(Node::Dice { num, size }),
            operations: vec![op],
        },
        Node::DiceWithOps {
            dice,
            mut operations,
        } => {
            operations.push(op);
            Node::DiceWithOps { dice, operations }
        }
        Node::Set {
            elements,
            mut operations,
        } => {
            operations.push(op);
            Node::Set {
                elements,
                operations,
            }
        }
        // Applying an operation to anything else mirrors the parser's invariant
        // that operations target dice/sets; eval reports the mismatch.
        other => Node::DiceWithOps {
            dice: Box::new(other),
            operations: vec![op],
        },
    }
}

fn attach_tags<T>(node: Node<T>, tags: Vec<Annotation<T>>) -> Node<T> {
    if tags.is_empty() {
        return node;
    }
    match node {
        Node::Annotated {
            expr,
            mut annotations,
        } => {
            annotations.extend(tags);
            Node::Annotated { expr, annotations }
        }
        other => Node::Annotated {
            expr: Box::new(other),
            annotations: tags,
        },
    }
}

/// A numeric literal.
pub fn lit<T>(value: f64) -> Roll<T> {
    Roll::wrap(Node::Literal(value))
}

/// `count` dice of `faces` sides (e.g. `dice(4, 6)` == `4d6`).
pub fn dice<T>(count: u32, faces: u32) -> Roll<T> {
    Roll::wrap(Node::Dice {
        num: Some(Box::new(Node::Literal(f64::from(count)))),
        size: DiceSize::Value(Box::new(Node::Literal(f64::from(faces)))),
    })
}

/// A single die of `faces` sides (e.g. `die(20)` == `d20`).
pub fn die<T>(faces: u32) -> Roll<T> {
    Roll::wrap(Node::Dice {
        num: None,
        size: DiceSize::Value(Box::new(Node::Literal(f64::from(faces)))),
    })
}

/// A percentile die (`d%`).
pub fn d_percent<T>() -> Roll<T> {
    Roll::wrap(Node::Dice {
        num: None,
        size: DiceSize::Percent,
    })
}

/// A set literal from the given elements (e.g. `{a, b, c}`).
pub fn set<T>(elements: impl IntoIterator<Item = Roll<T>>) -> Roll<T> {
    Roll::wrap(Node::Set {
        elements: elements.into_iter().map(|r| r.node).collect(),
        operations: Vec::new(),
    })
}
