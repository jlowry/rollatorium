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
pub enum Node<T> {
    /// A numeric literal.
    Literal(f64),
    /// A unary operation such as negation.
    Unary {
        operator: UnaryOperator,
        operand: Box<Node<T>>,
    },
    /// A binary arithmetic operation (addition, multiplication, etc.).
    Binary {
        operator: BinaryOperator,
        left: Box<Node<T>>,
        right: Box<Node<T>>,
    },
    /// A dice roll expression, e.g. `4d6` or `d%`.
    Dice {
        num: Option<Box<Node<T>>>,
        size: DiceSize<T>,
    },
    /// A set literal (with optional set-style operations).
    Set {
        elements: Vec<Node<T>>,
        operations: Vec<SetOperation<T>>,
    },
    /// A dice expression with additional keep/drop/reroll/etc. operations.
    DiceWithOps {
        dice: Box<Node<T>>,
        operations: Vec<SetOperation<T>>,
    },
    /// An annotated expression, e.g. `4d6 [strength]`.
    Annotated {
        expr: Box<Node<T>>,
        annotations: Vec<Annotation<T>>,
    },
}

/// The size of a die (e.g. 6 for d6 or percent for d%).
#[derive(Debug, Clone, PartialEq)]
pub enum DiceSize<T> {
    Value(Box<Node<T>>),
    Percent,
}

/// Unary operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Plus,
    Minus,
}

/// Binary operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDivide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

/// A selector targets a subset of a dice pool (e.g. highest, lowest).
#[derive(Debug, Clone, PartialEq)]
pub struct Selector<T> {
    pub kind: SelectorKind,
    pub target: Box<Node<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorKind {
    Literal,
    Highest,
    Lowest,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    EqualTo,
    NotEqual,
}

/// The different set operations that can be applied to a dice pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOperator {
    Keep,
    Drop,
    Reroll,
    RerollOnce,
    RerollAdd,
    Explode,
    ExplodeCompound,
    ExplodePenetrate,
    Penetrate,
    Minimum,
    Maximum,
    CountSuccess,
    CountFailure,
}

/// A modifier applied to a dice set, potentially using a selector.
#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation<T> {
    pub operator: SetOperator,
    pub selectors: Vec<Selector<T>>,
}

/// Represents an annotation applied to a node.
///
/// The tag is caller-defined: parsing produces `Annotation<&str>` borrowing the
/// input text, while [`Node::map_tags`] or the builder can supply any type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation<T> {
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
                elements: elements
                    .into_iter()
                    .map(|e| e.map_tags_inner(f))
                    .collect(),
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
