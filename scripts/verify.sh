#!/bin/sh
set -eu

run() {
    printf '+ %s\n' "$*"
    "$@"
}

run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run python3 scripts/check_structure.py
run python3 -m unittest scripts/tests/test_check_host_parity.py
run python3 -m unittest scripts/tests/test_build_wheel_manifest.py
run python3 scripts/check_host_parity.py
run scripts/check_wasm_node.sh
run maturin build --manifest-path crates/pine-python/Cargo.toml --out dist
run python3 -m pip install --force-reinstall dist/*.whl
run python3 -m pytest python/tests
