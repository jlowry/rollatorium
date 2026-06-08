//! Turns a parsed [`Node`] into a token stream that rebuilds it at runtime with
//! owned `Box`/`Vec`/literal construction in the caller's crate.
//!
//! Every emitted path is fully qualified (`::rollatorium::__private::…` for the
//! AST types, `::std::…`/`::core::…` for the standard library) so the expansion
//! is hygienic regardless of what the caller has in scope. The crate types are
//! reached through `rollatorium`'s hidden `__private` re-export, so the caller
//! only needs the `rollatorium` crate in scope.

use proc_macro2::TokenStream;
use quote::quote;
use rollatorium_core::__private::{
    BinaryOperator, DiceSize, Selector, SelectorKind, SetOperation, SetOperator, UnaryOperator,
};
use rollatorium_core::Node;

/// Emit an expression of type `rollatorium::Node<&'static str>` equal to `node`.
pub(crate) fn emit(node: &Node<&str>) -> TokenStream {
    match node {
        Node::Literal(value) => {
            debug_assert!(
                value.is_finite(),
                "the parser never produces non-finite literals"
            );
            let lit = proc_macro2::Literal::f64_suffixed(*value);
            quote!(::rollatorium::__private::Node::Literal(#lit))
        }
        Node::Unary { operator, operand } => {
            let operator = emit_unary(*operator);
            let operand = emit(operand);
            quote!(::rollatorium::__private::Node::Unary {
                operator: #operator,
                operand: ::std::boxed::Box::new(#operand),
            })
        }
        Node::Binary {
            operator,
            left,
            right,
        } => {
            let operator = emit_binary(*operator);
            let left = emit(left);
            let right = emit(right);
            quote!(::rollatorium::__private::Node::Binary {
                operator: #operator,
                left: ::std::boxed::Box::new(#left),
                right: ::std::boxed::Box::new(#right),
            })
        }
        Node::Dice { num, size } => {
            let num = emit_opt_box(num.as_deref());
            let size = emit_dice_size(size);
            quote!(::rollatorium::__private::Node::Dice { num: #num, size: #size })
        }
        Node::Set {
            elements,
            operations,
        } => {
            let elements = elements.iter().map(emit);
            let operations = operations.iter().map(emit_set_operation);
            quote!(::rollatorium::__private::Node::Set {
                elements: ::std::vec![ #(#elements),* ],
                operations: ::std::vec![ #(#operations),* ],
            })
        }
        Node::DiceWithOps { dice, operations } => {
            let dice = emit(dice);
            let operations = operations.iter().map(emit_set_operation);
            quote!(::rollatorium::__private::Node::DiceWithOps {
                dice: ::std::boxed::Box::new(#dice),
                operations: ::std::vec![ #(#operations),* ],
            })
        }
        Node::Annotated { expr, annotations } => {
            let expr = emit(expr);
            let annotations = annotations.iter().map(|annotation| {
                let tag = annotation.tag;
                quote!(::rollatorium::__private::Annotation { tag: #tag })
            });
            quote!(::rollatorium::__private::Node::Annotated {
                expr: ::std::boxed::Box::new(#expr),
                annotations: ::std::vec![ #(#annotations),* ],
            })
        }
    }
}

fn emit_opt_box(inner: Option<&Node<&str>>) -> TokenStream {
    match inner {
        Some(node) => {
            let node = emit(node);
            quote!(::core::option::Option::Some(::std::boxed::Box::new(#node)))
        }
        None => quote!(::core::option::Option::None),
    }
}

fn emit_dice_size(size: &DiceSize<&str>) -> TokenStream {
    match size {
        DiceSize::Value(inner) => {
            let inner = emit(inner);
            quote!(::rollatorium::__private::DiceSize::Value(::std::boxed::Box::new(#inner)))
        }
        DiceSize::Percent => quote!(::rollatorium::__private::DiceSize::Percent),
    }
}

fn emit_set_operation(op: &SetOperation<&str>) -> TokenStream {
    let operator = emit_set_operator(op.operator);
    let selectors = op.selectors.iter().map(emit_selector);
    quote!(::rollatorium::__private::SetOperation {
        operator: #operator,
        selectors: ::std::vec![ #(#selectors),* ],
    })
}

fn emit_selector(selector: &Selector<&str>) -> TokenStream {
    let kind = emit_selector_kind(selector.kind);
    let target = emit(&selector.target);
    quote!(::rollatorium::__private::Selector {
        kind: #kind,
        target: ::std::boxed::Box::new(#target),
    })
}

fn emit_unary(op: UnaryOperator) -> TokenStream {
    let variant = match op {
        UnaryOperator::Plus => quote!(Plus),
        UnaryOperator::Minus => quote!(Minus),
    };
    quote!(::rollatorium::__private::UnaryOperator::#variant)
}

fn emit_binary(op: BinaryOperator) -> TokenStream {
    // `BinaryOperator` is `#[non_exhaustive]`, so matching it from this crate
    // needs a wildcard arm. The value came from the parser, so it can only be a
    // known variant; the arm is genuinely unreachable.
    let variant = match op {
        BinaryOperator::Add => quote!(Add),
        BinaryOperator::Subtract => quote!(Subtract),
        BinaryOperator::Multiply => quote!(Multiply),
        BinaryOperator::Divide => quote!(Divide),
        BinaryOperator::IntDivide => quote!(IntDivide),
        BinaryOperator::Modulo => quote!(Modulo),
        BinaryOperator::Equal => quote!(Equal),
        BinaryOperator::NotEqual => quote!(NotEqual),
        BinaryOperator::Greater => quote!(Greater),
        BinaryOperator::GreaterEqual => quote!(GreaterEqual),
        BinaryOperator::Less => quote!(Less),
        BinaryOperator::LessEqual => quote!(LessEqual),
        _ => unreachable!("the parser produced an unknown BinaryOperator variant"),
    };
    quote!(::rollatorium::__private::BinaryOperator::#variant)
}

fn emit_selector_kind(kind: SelectorKind) -> TokenStream {
    // `SelectorKind` is `#[non_exhaustive]`; see `emit_binary`.
    let variant = match kind {
        SelectorKind::Literal => quote!(Literal),
        SelectorKind::Highest => quote!(Highest),
        SelectorKind::Lowest => quote!(Lowest),
        SelectorKind::GreaterThan => quote!(GreaterThan),
        SelectorKind::GreaterThanOrEqual => quote!(GreaterThanOrEqual),
        SelectorKind::LessThan => quote!(LessThan),
        SelectorKind::LessThanOrEqual => quote!(LessThanOrEqual),
        SelectorKind::EqualTo => quote!(EqualTo),
        SelectorKind::NotEqual => quote!(NotEqual),
        _ => unreachable!("the parser produced an unknown SelectorKind variant"),
    };
    quote!(::rollatorium::__private::SelectorKind::#variant)
}

fn emit_set_operator(op: SetOperator) -> TokenStream {
    // `SetOperator` is `#[non_exhaustive]`; see `emit_binary`.
    let variant = match op {
        SetOperator::Keep => quote!(Keep),
        SetOperator::Drop => quote!(Drop),
        SetOperator::Reroll => quote!(Reroll),
        SetOperator::RerollOnce => quote!(RerollOnce),
        SetOperator::RerollAdd => quote!(RerollAdd),
        SetOperator::Explode => quote!(Explode),
        SetOperator::ExplodeCompound => quote!(ExplodeCompound),
        SetOperator::ExplodePenetrate => quote!(ExplodePenetrate),
        SetOperator::Penetrate => quote!(Penetrate),
        SetOperator::Minimum => quote!(Minimum),
        SetOperator::Maximum => quote!(Maximum),
        SetOperator::CountSuccess => quote!(CountSuccess),
        SetOperator::CountFailure => quote!(CountFailure),
        _ => unreachable!("the parser produced an unknown SetOperator variant"),
    };
    quote!(::rollatorium::__private::SetOperator::#variant)
}
