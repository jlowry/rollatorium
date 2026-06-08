fn main() {
    // The macro argument must be a string literal, not a runtime value.
    let expr = "2d6";
    let _ = rollatorium::dice!(expr);
}
