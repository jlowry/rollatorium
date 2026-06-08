//! Compile-fail tests: invalid input to `dice!` must be a compile error, with
//! the parser's message surfaced at the string literal's span.
#![cfg(feature = "macros")]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
