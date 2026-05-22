#!/bin/sh
set -eu

run() {
    printf '+ %s\n' "$*"
    "$@"
}

run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run cargo check -p pine-wasm --target wasm32-unknown-unknown
run maturin build --manifest-path crates/pine-python/Cargo.toml --out dist
run python3 -m pip install --force-reinstall dist/*.whl
run python3 -m pytest python/tests
