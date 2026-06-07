// ---------- AST ----------

/// A node in the Rollatorium abstract syntax tree.
///
/// The grammar for the language supports a rich set of operations for dice
/// expressions (selectors, modifiers, annotations, etc.).  The `Node` enum is
/// intentionally expressive enough to represent those constructs, even though
/// the current parser only emits a small subset today.  The extra variants and
/// supporting types make it possible to extend the parser without having to
/// redesign the tree structure later on.
///
/// The type parameter `T` is the *tag* attached to [`Node::Annotated`]
/// wrappers.  Parsing a string yields `Node<&str>` whose tags borrow directly
/// from the input (no allocation); callers can retag to their own type with
/// [`Node::map_tags`] or construct a tree from scratch with the
/// [`crate::build`] builder.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Node<T> {
    /// A numeric literal.
    Literal(f64),
    /// A unary operation such as negation.
    Unary {
        /// The unary operator applied to `operand`.
        operator: UnaryOperator,
        /// The expression the operator is applied to.
        operand: Box<Node<T>>,
    },
    /// A binary arithmetic operation (addition, multiplication, etc.).
    Binary {
        /// The binary operator combining `left` and `right`.
        operator: BinaryOperator,
        /// The left-hand operand.
        left: Box<Node<T>>,
        /// The right-hand operand.
        right: Box<Node<T>>,
    },
    /// A dice roll expression, e.g. `4d6` or `d%`.
    Dice {
        /// The number of dice to roll; `None` means a single die.
        num: Option<Box<Node<T>>>,
        /// The size (number of faces) of each die.
        size: DiceSize<T>,
    },
    /// A set literal (with optional set-style operations).
    Set {
        /// The expressions that make up the set.
        elements: Vec<Node<T>>,
        /// Keep/drop operations applied to the set.
        operations: Vec<SetOperation<T>>,
    },
    /// A dice expression with additional keep/drop/reroll/etc. operations.
    DiceWithOps {
        /// The underlying [`Node::Dice`] expression.
        dice: Box<Node<T>>,
        /// The operations applied to the dice pool, in order.
        operations: Vec<SetOperation<T>>,
    },
    /// An annotated expression, e.g. `4d6 [strength]`.
    Annotated {
        /// The expression being annotated.
        expr: Box<Node<T>>,
        /// The tags attached to `expr`.
        annotations: Vec<Annotation<T>>,
    },
}

/// The size of a die (e.g. 6 for d6 or percent for d%).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiceSize<T> {
    /// A fixed number of faces, given by an expression (e.g. the `6` in `d6`).
    Value(Box<Node<T>>),
    /// A percentile die (`d%`), rolling multiples of ten from 0 to 90.
    Percent,
}

/// Unary operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    /// Unary plus (`+x`), a no-op that returns its operand unchanged.
    Plus,
    /// Unary negation (`-x`).
    Minus,
}

/// Binary operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BinaryOperator {
    /// Addition (`+`).
    Add,
    /// Subtraction (`-`).
    Subtract,
    /// Multiplication (`*`).
    Multiply,
    /// Floating-point division (`/`).
    Divide,
    /// Truncating integer division (`//`).
    IntDivide,
    /// Remainder (`%`).
    Modulo,
    /// Equality comparison (`==`), yielding `1.0` or `0.0`.
    Equal,
    /// Inequality comparison (`!=`).
    NotEqual,
    /// Greater-than comparison (`>`).
    Greater,
    /// Greater-than-or-equal comparison (`>=`).
    GreaterEqual,
    /// Less-than comparison (`<`).
    Less,
    /// Less-than-or-equal comparison (`<=`).
    LessEqual,
}

/// A selector targets a subset of a dice pool (e.g. highest, lowest).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Selector<T> {
    /// How the targeted subset is chosen.
    pub kind: SelectorKind,
    /// The expression supplying the selector's argument (e.g. the count or
    /// threshold).
    pub target: Box<Node<T>>,
}

/// The kind of subset a [`Selector`] picks out of a dice pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum SelectorKind {
    /// Dice whose value equals the literal target.
    Literal,
    /// The highest *n* dice, where *n* is the target.
    Highest,
    /// The lowest *n* dice, where *n* is the target.
    Lowest,
    /// Dice strictly greater than the target.
    GreaterThan,
    /// Dice greater than or equal to the target.
    GreaterThanOrEqual,
    /// Dice strictly less than the target.
    LessThan,
    /// Dice less than or equal to the target.
    LessThanOrEqual,
    /// Dice equal to the target.
    EqualTo,
    /// Dice not equal to the target.
    NotEqual,
}

/// The different set operations that can be applied to a dice pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum SetOperator {
    /// Keep only the selected dice.
    Keep,
    /// Drop the selected dice.
    Drop,
    /// Reroll selected dice until none match.
    Reroll,
    /// Reroll selected dice exactly once.
    RerollOnce,
    /// Reroll selected dice, keeping the original and adding the new die.
    RerollAdd,
    /// Explode selected dice, rolling an additional die for each match.
    Explode,
    /// Exploding where additional rolls are summed into the original die.
    ExplodeCompound,
    /// Exploding where each additional die is penalised by one.
    ExplodePenetrate,
    /// Penetrating dice (each extra die is reduced by one).
    Penetrate,
    /// Clamp selected dice up to a minimum value.
    Minimum,
    /// Clamp selected dice down to a maximum value.
    Maximum,
    /// Count the selected dice as successes.
    CountSuccess,
    /// Count the selected dice as failures.
    CountFailure,
}

/// A modifier applied to a dice set, potentially using a selector.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetOperation<T> {
    /// The operation to apply.
    pub operator: SetOperator,
    /// The selectors describing which dice the operation targets.
    pub selectors: Vec<Selector<T>>,
}

/// Represents an annotation applied to a node.
///
/// The tag is caller-defined: parsing produces `Annotation<&str>` borrowing the
/// input text, while [`Node::map_tags`] or the builder can supply any type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Annotation<T> {
    /// The caller-defined tag value.
    pub tag: T,
}

impl<T> Node<T> {
    /// Transform every tag in the tree, producing a `Node` over a new tag type.
    ///
    /// Only [`Node::Annotated`] wrappers carry tags; every other node is rebuilt
    /// unchanged with its children mapped. This is the primary way to go from
    /// the borrowed `&str` tags produced by [`crate::parse`] to a caller-defined
    /// type, e.g. `parse("4d6 [fire]")?.map_tags(|s| Damage::from(s))`.
    pub fn map_tags<U, F>(self, mut f: F) -> Node<U>
    where
        F: FnMut(T) -> U,
    {
        self.map_tags_inner(&mut f)
    }

    fn map_tags_inner<U, F>(self, f: &mut F) -> Node<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            Node::Literal(v) => Node::Literal(v),
            Node::Unary { operator, operand } => Node::Unary {
                operator,
                operand: Box::new(operand.map_tags_inner(f)),
            },
            Node::Binary {
                operator,
                left,
                right,
            } => Node::Binary {
                operator,
                left: Box::new(left.map_tags_inner(f)),
                right: Box::new(right.map_tags_inner(f)),
            },
            Node::Dice { num, size } => Node::Dice {
                num: num.map(|n| Box::new(n.map_tags_inner(f))),
                size: size.map_tags_inner(f),
            },
            Node::Set {
                elements,
                operations,
            } => Node::Set {
                elements: elements.into_iter().map(|e| e.map_tags_inner(f)).collect(),
                operations: operations
                    .into_iter()
                    .map(|o| o.map_tags_inner(f))
                    .collect(),
            },
            Node::DiceWithOps { dice, operations } => Node::DiceWithOps {
                dice: Box::new(dice.map_tags_inner(f)),
                operations: operations
                    .into_iter()
                    .map(|o| o.map_tags_inner(f))
                    .collect(),
            },
            Node::Annotated { expr, annotations } => Node::Annotated {
                expr: Box::new(expr.map_tags_inner(f)),
                annotations: annotations
                    .into_iter()
                    .map(|a| Annotation { tag: f(a.tag) })
                    .collect(),
            },
        }
    }
}

impl<T> DiceSize<T> {
    fn map_tags_inner<U, F>(self, f: &mut F) -> DiceSize<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            DiceSize::Value(inner) => DiceSize::Value(Box::new(inner.map_tags_inner(f))),
            DiceSize::Percent => DiceSize::Percent,
        }
    }
}

impl<T> SetOperation<T> {
    pub(crate) fn map_tags_inner<U, F>(self, f: &mut F) -> SetOperation<U>
    where
        F: FnMut(T) -> U,
    {
        SetOperation {
            operator: self.operator,
            selectors: self
                .selectors
                .into_iter()
                .map(|s| s.map_tags_inner(f))
                .collect(),
        }
    }
}

impl<T> Selector<T> {
    fn map_tags_inner<U, F>(self, f: &mut F) -> Selector<U>
    where
        F: FnMut(T) -> U,
    {
        Selector {
            kind: self.kind,
            target: Box::new(self.target.map_tags_inner(f)),
        }
    }
}
