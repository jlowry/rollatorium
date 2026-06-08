# rollatorium

Parse and evaluate tabletop-RPG dice expressions (`4d6kh3`, `3d6rr<3`,
`2d6mi3ma5`, annotated rolls like `4d6 [fire damage]`) into a detailed result
tree.

This repository is a Cargo workspace with three crates under `packages/`:

| Crate | What it is |
| ----- | ---------- |
| [`rollatorium`](packages/rollatorium) | The published library: re-exports the core API and adds the compile-time `dice!` macro. **Start here.** |
| [`rollatorium-core`](packages/rollatorium-core) | Lexer, parser, AST, evaluator, and fluent builder — no proc macro, no compile-time codegen. |
| [`rollatorium-macros`](packages/rollatorium-macros) | The `dice!` procedural macro, which parses and validates an expression at compile time. |

See [`packages/rollatorium/README.md`](packages/rollatorium/README.md) for usage,
and [`CLAUDE.md`](CLAUDE.md) for the layout and development commands.

## Quick start

```rust
// Runtime parse + evaluate:
let result = rollatorium::roll("4d6kh3 + 2 [strength]")?;

// Compile-time parse + validate (a bad string is a compile error):
let ast = rollatorium::dice!("4d6kh3 [strength]"); // Node<&'static str>
# Ok::<(), rollatorium::RollatoriumError>(())
```

Licensed under either of MIT (`LICENSE-MIT`) or Apache-2.0 (`LICENSE-APACHE`) at
your option.
