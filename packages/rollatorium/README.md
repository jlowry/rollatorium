# rollatorium

A Rust library for parsing and evaluating tabletop-RPG dice expressions such as
`4d6kh3`, `3d6rr<3`, `2d6mi3ma5`, and annotated rolls like `4d6 [fire damage]`.

It exposes `parse`/`eval`/`roll` functions and a detailed result tree describing
every die rolled, kept, dropped, rerolled, exploded, or clamped.

## Usage

```rust
use rollatorium::roll;

// Parse and evaluate in one call.
let result = roll("2d6 + 3").unwrap();
assert!((5.0..=15.0).contains(&result.total));
```

For more control, parse once and evaluate with an injected RNG for deterministic
results:

```rust
use rand::{SeedableRng, rngs::StdRng};
use rollatorium::{EvalConfig, eval_with_rng, parse};

let node = parse("4d6kh3").unwrap();
let rng = StdRng::seed_from_u64(42);
let result = eval_with_rng(&node, EvalConfig::default(), rng).unwrap();
assert!((3.0..=18.0).contains(&result.total));
```

Inspect the result tree to see exactly what happened to each die:

```rust
use rollatorium::{Value, roll};

if let Value::Dice(pool) = roll("4d6kh3").unwrap().value {
    for die in &pool {
        println!("die {} kept={}", die.value(), die.kept());
    }
}
```

## Compile-time expressions: the `dice!` macro

`dice!` parses and validates an expression at **compile time** — an invalid
string is a compile error pointing at the literal — and expands to owned AST
construction with no runtime parsing. The result is a
[`Node`]`<&'static str>`, so it drops straight into a `LazyLock`/`OnceLock`:

```rust
use std::sync::LazyLock;
use rollatorium::{Node, dice, eval};

static ATTACK: LazyLock<Node<&'static str>> =
    LazyLock::new(|| dice!("4d6kh3 [strength]"));

let total = eval(&ATTACK).unwrap().total;
assert!((3.0..=18.0).contains(&total));
```

The macro is enabled by the default `macros` feature; turn it off with
`default-features = false` if you only need runtime parsing.

## Expression syntax

| Syntax        | Meaning                                            |
| ------------- | -------------------------------------------------- |
| `4d6`         | roll four six-sided dice                           |
| `d%`          | a percentile die                                   |
| `4d6kh3`      | keep the highest three                             |
| `4d6kl1`      | keep the lowest one                                |
| `4d6ph1`      | drop the highest one                               |
| `3d6rr<3`     | reroll dice below 3 until none match               |
| `3d6ro<4`     | reroll dice below 4 exactly once                   |
| `1d6ra==6`    | reroll-and-add on a 6                              |
| `1d6e==6`     | explode on a 6                                      |
| `2d6mi3ma5`   | clamp each die to the range 3–5                    |
| `4d6 [fire]`  | annotate a roll with a caller-defined tag          |

The [`build`] module offers a fluent builder for constructing expressions in
code without going through a string.

## Cargo features

- `macros` *(default)* — the compile-time [`dice!`] macro. Disabling it drops the
  `rollatorium-macros` dependency.
- `serde` — derive `Serialize`/`Deserialize` on the AST and result types.
- `fail-on-warnings` — turn warnings (including missing docs) into hard errors;
  intended for CI.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
