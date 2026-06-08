# CLAUDE.md

Guidance for AI assistants working in the **rollatorium** repository.

## Project overview

Rollatorium is a Rust library for parsing and evaluating tabletop-RPG dice
expressions such as `4d6kh3`, `3d6rr<3`, `2d6mi3ma5`, and annotated rolls like
`4d6 [fire damage]`. It exposes parse/eval/roll functions and a detailed result
tree describing every die rolled, kept, dropped, rerolled, exploded, or clamped.
It also provides a `dice!` proc macro that parses and validates an expression at
**compile time**, expanding to owned AST construction (no runtime parsing).

- **Cargo workspace** (virtual root) with three crates under `packages/`,
  all version `0.0.1-snapshot`:
  - `rollatorium` — public facade: re-exports `rollatorium-core` and adds `dice!`
  - `rollatorium-core` — lexer, parser, AST, evaluator, builder (no proc macro)
  - `rollatorium-macros` — the `dice!` procedural macro (depends on `-core`)

  The split exists so `rollatorium-macros` can call the parser at compile time
  without a dependency cycle with the `rollatorium` facade.
- **Rust edition 2024**
- Dual-licensed MIT / Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`)

Evaluation pipeline:

```
input string → lexer → parser → AST (Node) → evaluator → EvalResult
```

## Repository layout

```
Cargo.toml                    # virtual workspace root (members = packages/*)
packages/
  rollatorium/                # public facade crate
    src/lib.rs                # re-exports rollatorium-core + the dice! macro
    tests/
      test_*.rs               # integration tests, one file per feature
      test_macro.rs           # dice! macro tests (AST roundtrip vs parse)
      trybuild_ui.rs          # compile-fail tests; fixtures + .stderr in tests/ui/
      common/mod.rs           # helper `r(expr) -> f64`
      custom_strategies/mod.rs# proptest strategies
      test_proptest.rs        # property tests (excluded by default, see below)
    examples/repl.rs          # interactive REPL
    fuzz/                     # cargo-fuzz harness (excluded from the workspace)
    README.md                 # crate docs (included via #![doc = ...])
  rollatorium-core/
    src/lib.rs                # public API + Result + parse/eval/roll; demo tests
    src/{lexer,token,parser,ast,eval,error,builder}.rs
  rollatorium-macros/
    src/lib.rs                # #[proc_macro] dice
    src/codegen.rs            # AST → token-stream reconstruction (Node<&'static str>)
ci/run-fuzz.sh                # local fuzz runner (runs from packages/rollatorium)
.config/nextest.toml          # nextest profile / default filter
.github/workflows/            # CI: pr-checks, fuzz, cocogitto
```

The library itself lives entirely in `rollatorium-core`; `rollatorium` is a thin
re-export facade. The hidden `rollatorium_core::__private` module re-exports the
AST support types so the `dice!`-generated code can name them
(`::rollatorium::__private::…`).

## Public API (`rollatorium`, re-exported from `rollatorium-core`)

```rust
pub fn parse<I: AsRef<str> + ?Sized>(input: &I) -> Result<Node<&str>>
pub fn eval<T: Clone>(expr: &Node<T>) -> Result<EvalResult<T>>
pub fn roll<I: AsRef<str> + ?Sized>(input: &I) -> Result<EvalResult<&str>>

// compile-time parse + validate; expands to Node<&'static str> (feature `macros`, on by default)
rollatorium::dice!("4d6kh3 [strength]")
```

Re-exported from `eval`:

- `eval_with_config(expr, EvalConfig) -> Result<EvalResult>`
- `eval_with_rng(expr, EvalConfig, rng) -> Result<EvalResult>` — inject any
  `rand::RngCore`. Tests use `StdRng::seed_from_u64(..)` for determinism.
- Types: `EvalConfig`, `EvalResult`, `Value`, `DiceRoll`, `DieResult`,
  `DieOrigin`, `DieAdjustment`, `SetRoll`, `SetElement`.

`pub type Result<T> = std::result::Result<T, RollatoriumError>;`

`Node<T>` (generic over the annotation tag type) and `Annotation<T>` are
re-exported, but you normally obtain a `Node` via `parse`, the `dice!` macro, or
the `build::Roll` builder (`Roll::{lit,dice,die,d_percent,set}` constructors plus
chained combinators, finished with `.build()`). The other AST types
(`DiceSize`, operators, selectors, `SetOperation`) are intentionally not exported.

## Commands

| Task   | Command |
| ------ | ------- |
| Format | `cargo fmt --all -- --check` |
| Lint   | `cargo clippy --workspace --all-targets -- -D warnings` |
| Check  | `cargo check --workspace --all-targets` |
| Build  | `cargo build --workspace --all-targets` |
| Test   | `cargo nextest run --workspace --all-targets --no-fail-fast` |
| REPL   | `cargo run -p rollatorium --example repl` |
| Fuzz   | `ci/run-fuzz.sh` (or `cd packages/rollatorium && cargo +nightly fuzz run parser -- -runs=1000`) |

Notes:

- The CI test runner is **cargo-nextest**, not `cargo test`. The default profile
  in `.config/nextest.toml` filters out `binary_id(rollatorium::test_proptest)`,
  so the proptest binary does **not** run by default — invoke it explicitly when
  you need it, either with `cargo test --test test_proptest` or with
  `cargo nextest run -E 'binary_id(rollatorium::test_proptest)' --ignore-default-filter`
  (the `--ignore-default-filter` is required, otherwise the default filter still
  excludes it and nothing runs).
- Fuzzing needs nightly + `cargo install cargo-fuzz`. `ci/run-fuzz.sh` honors
  `CARGO`, `FUZZ_TARGET` (default `parser`), and `FUZZ_RUNS` (default `10000`).

## Conventions

- **Conventional Commits are mandatory.** Commit messages are validated by
  cocogitto in CI (`cocogitto_push.yml`, `cocogitto_pull_request.yml`). Use
  prefixes like `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`.
- **No unsafe code.** The crate sets `#![forbid(unsafe_code)]`.
- **Warnings are errors.** Keep `cargo clippy --workspace --all-targets -- -D warnings`
  clean. An optional per-crate `fail-on-warnings` feature adds `#![deny(warnings)]`;
  each crate's `lib.rs` and the examples carry
  `#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]`.
- **Errors** use `thiserror` via `RollatoriumError` in
  `packages/rollatorium-core/src/error.rs`.
- **Testing**: integration tests live in `packages/rollatorium/tests/` as
  `test_<feature>.rs` with functions named `test_<feature>_<scenario>`. Use the
  `r(expr)` helper from `tests/common/mod.rs` for total-only assertions, and
  seeded RNG via `eval_with_rng` when asserting on individual dice. When changing
  `parser.rs` or `eval.rs` (both in `rollatorium-core`), add or extend property
  tests (`tests/test_proptest.rs`, `tests/custom_strategies/`) and consider the
  fuzz target. When changing the macro's codegen, extend
  `tests/test_macro.rs` (AST roundtrip) and the `tests/ui/` compile-fail cases.
- `Cargo.lock` and `/target` are gitignored (this is a library).

## CI workflows (`.github/workflows/`)

- `pr-checks.yml` — stable toolchain: fmt → clippy → check → build → nextest.
  Runs on PRs and pushes to `main`/`d20port`.
- `fuzz.yml` — nightly: smoke-runs the `parser` fuzzer for 1000 runs.
- `cocogitto_pull_request.yml` / `cocogitto_push.yml` — conventional-commit
  validation.

All of the above must pass before merge.
