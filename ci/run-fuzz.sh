#!/usr/bin/env bash
set -euo pipefail

: "${CARGO:=cargo +nightly}"
: "${FUZZ_TARGET:=parser}"
: "${FUZZ_RUNS:=10000}"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "cargo-fuzz not found. Install with 'cargo install cargo-fuzz'." >&2
    exit 1
fi

# The fuzz crate lives alongside the `rollatorium` package it fuzzes.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${script_dir}/../packages/rollatorium"

exec ${CARGO} fuzz run "${FUZZ_TARGET}" -- -runs="${FUZZ_RUNS}"
