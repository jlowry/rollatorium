//! Compile-time dice-expression macro for [`rollatorium`].
//!
//! This crate provides the [`dice!`] procedural macro, which the `rollatorium`
//! crate re-exports as `rollatorium::dice!` (enabled by its default `macros`
//! feature). Use it from there rather than depending on this crate directly.
//!
//! [`rollatorium`]: https://docs.rs/rollatorium
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codegen;

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Parse and validate a dice expression at compile time, expanding to owned
/// construction code that rebuilds the AST at runtime.
///
/// The argument must be a string literal. It is parsed by the same parser as
/// [`rollatorium::parse`](https://docs.rs/rollatorium), so invalid input is a
/// **compile error** pointing at the literal — there is no runtime parsing. The
/// expansion is an expression of type `rollatorium::Node<&'static str>`
/// (annotation text becomes a `&'static str` literal), usable anywhere an
/// expression is, including the initializer of a `LazyLock`/`OnceLock`:
///
/// ```ignore
/// use std::sync::LazyLock;
/// use rollatorium::Node;
///
/// static ATTACK: LazyLock<Node<&'static str>> =
///     LazyLock::new(|| rollatorium::dice!("4d6kh3 [strength]"));
///
/// let total = rollatorium::eval(&ATTACK).unwrap().total;
/// assert!((3.0..=18.0).contains(&total));
/// ```
#[proc_macro]
pub fn dice(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let source = lit.value();
    match rollatorium_core::parse(&source) {
        Ok(node) => {
            // Ascribe the tag type so the expansion is always `Node<&'static str>`,
            // even for expressions with no annotations to pin `T` from a tag.
            let constructed = codegen::emit(&node);
            quote!({
                let __dice: ::rollatorium::__private::Node<&'static str> = #constructed;
                __dice
            })
            .into()
        }
        Err(err) => syn::Error::new(lit.span(), err.to_string())
            .to_compile_error()
            .into(),
    }
}
