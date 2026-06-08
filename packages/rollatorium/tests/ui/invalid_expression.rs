fn main() {
    // Invalid dice expression: the parser rejects it at compile time.
    let _ = rollatorium::dice!("4d6kkk");
}
