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
run python3 -m unittest scripts/tests/test_analyze_legacy_corpus.py
run python3 -m unittest scripts/tests/test_audit_legacy_corpus_dedup.py
run python3 -m unittest scripts/tests/test_import_legacy_corpus.py
run python3 -m unittest scripts/tests/test_merge_legacy_corpus_manifests.py
run python3 -m unittest scripts/tests/test_compare_tradingview_outputs.py
run python3 -m unittest scripts/tests/test_normalize_tradingview_bars.py
run python3 -m unittest scripts/tests/test_profile_legacy_release.py
run python3 scripts/check_host_parity.py
run scripts/check_wasm_node.sh

# Build into a fresh directory so wheels left by an earlier version cannot be
# expanded into the same pip command and make the release gate fail for an
# unrelated dependency-resolution conflict.
wheel_output_dir=$(mktemp -d "${TMPDIR:-/tmp}/pine-python-wheel.XXXXXX")
trap 'rm -rf "$wheel_output_dir"' EXIT
trap 'exit 1' HUP INT TERM
run maturin build --manifest-path crates/pine-python/Cargo.toml --out "$wheel_output_dir"
set -- "$wheel_output_dir"/*.whl
if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
    printf 'error: expected exactly one freshly built wheel in %s\n' \
        "$wheel_output_dir" >&2
    exit 1
fi
run python3 -m pip install --force-reinstall "$1"
run python3 -m pytest python/tests
