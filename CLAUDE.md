# CLAUDE.md

Guidance for AI assistants working in the **rollatorium** repository.

## Project overview

Rollatorium is a Rust library for parsing and evaluating tabletop-RPG dice
expressions such as `4d6kh3`, `3d6rr<3`, `2d6mi3ma5`, and annotated rolls like
`4d6 [fire damage]`. It exposes parse/eval/roll functions and a detailed result
tree describing every die rolled, kept, dropped, rerolled, exploded, or clamped.

- Crate: `rollatorium`, version `0.0.1-snapshot`
- **Rust edition 2024**
- Dual-licensed MIT / Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`)

Evaluation pipeline:

```
input string → lexer → parser → AST (Node) → evaluator → EvalResult
```

## Repository layout

```
src/
  lib.rs      # public API + crate-wide attributes; #[cfg(test)] demo tests
  lexer.rs    # tokenizes input strings
  token.rs    # Token enum
  parser.rs   # recursive-descent / precedence-climbing parser → Node
  ast.rs      # Node, operators, selectors, annotations
  eval.rs     # evaluator: EvalResult, Value, DiceRoll, DieResult, ...
  error.rs    # RollatoriumError (thiserror)
tests/
  test_*.rs                 # integration tests, one file per feature
  common/mod.rs             # helper `r(expr) -> f64`
  custom_strategies/mod.rs  # proptest strategies
  test_proptest.rs          # property tests (excluded by default, see below)
examples/repl.rs            # interactive REPL
fuzz/                       # cargo-fuzz harness; bin `parser` → fuzz_targets/parse_and_eval.rs
ci/run-fuzz.sh              # local fuzz runner
.config/nextest.toml        # nextest profile / default filter
.github/workflows/          # CI: pr-checks, fuzz, cocogitto
```

## Public API (`src/lib.rs`)

```rust
pub fn parse<I: AsRef<str>>(input: &I) -> Result<Node>
pub fn eval(expr: &Node) -> Result<EvalResult>
pub fn roll<I: AsRef<str>>(input: &I) -> Result<EvalResult>
```

Re-exported from `eval`:

- `eval_with_config(expr, EvalConfig) -> Result<EvalResult>`
- `eval_with_rng(expr, EvalConfig, rng) -> Result<EvalResult>` — inject any
  `rand::RngCore`. Tests use `StdRng::seed_from_u64(..)` for determinism.
- Types: `EvalConfig`, `EvalResult`, `Value`, `DiceRoll`, `DieResult`,
  `DieOrigin`, `DieAdjustment`, `SetRoll`, `SetElement`.

`pub type Result<T> = std::result::Result<T, RollatoriumError>;`

`Node` is not publicly exported as a type to construct directly — obtain one via
`parse`.

## Commands

| Task   | Command |
| ------ | ------- |
| Format | `cargo fmt --all -- --check` |
| Lint   | `cargo clippy -- -D warnings` |
| Check  | `cargo check --workspace --all-targets` |
| Build  | `cargo build --workspace --all-targets` |
| Test   | `cargo nextest run --workspace --all-targets --no-fail-fast` |
| REPL   | `cargo run --example repl` |
| Fuzz   | `cargo +nightly fuzz run parser -- -runs=1000` (or `ci/run-fuzz.sh`) |

Notes:

- The CI test runner is **cargo-nextest**, not `cargo test`. The default profile
  in `.config/nextest.toml` filters out `binary_id(rollatorium::test_proptest)`,
  so the proptest binary does **not** run by default — invoke it explicitly
  (e.g. `cargo nextest run -E 'binary_id(rollatorium::test_proptest)'` or
  `cargo test --test test_proptest`) when you need it.
- Fuzzing needs nightly + `cargo install cargo-fuzz`. `ci/run-fuzz.sh` honors
  `CARGO`, `FUZZ_TARGET` (default `parser`), and `FUZZ_RUNS` (default `10000`).

## Conventions

- **Conventional Commits are mandatory.** Commit messages are validated by
  cocogitto in CI (`cocogitto_push.yml`, `cocogitto_pull_request.yml`). Use
  prefixes like `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`.
- **No unsafe code.** The crate sets `#![forbid(unsafe_code)]`.
- **Warnings are errors.** Keep `cargo clippy -- -D warnings` clean. An optional
  `fail-on-warnings` feature adds `#![deny(warnings)]`; new entry points (`lib.rs`,
  examples) carry `#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]`.
- **Errors** use `thiserror` via `RollatoriumError` in `error.rs`.
- **Testing**: integration tests live in `tests/` as `test_<feature>.rs` with
  functions named `test_<feature>_<scenario>`. Use the `r(expr)` helper from
  `tests/common/mod.rs` for total-only assertions, and seeded RNG via
  `eval_with_rng` when asserting on individual dice. When changing `parser.rs` or
  `eval.rs`, add or extend property tests (`tests/test_proptest.rs`,
  `tests/custom_strategies/`) and consider the fuzz target.
- `Cargo.lock` and `/target` are gitignored (this is a library).

## CI workflows (`.github/workflows/`)

- `pr-checks.yml` — stable toolchain: fmt → clippy → check → build → nextest.
  Runs on PRs and pushes to `main`/`d20port`.
- `fuzz.yml` — nightly: smoke-runs the `parser` fuzzer for 1000 runs.
- `cocogitto_pull_request.yml` / `cocogitto_push.yml` — conventional-commit
  validation.

All of the above must pass before merge.
