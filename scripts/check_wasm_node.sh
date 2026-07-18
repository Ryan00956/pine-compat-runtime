#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

if ! command -v node >/dev/null 2>&1; then
    printf '%s\n' 'error: Node.js is required for the wasm smoke gate' >&2
    exit 1
fi

target_dir=$(cargo metadata --no-deps --format-version 1 \
    | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
if [ -z "$target_dir" ]; then
    printf '%s\n' 'error: could not determine the Cargo target directory' >&2
    exit 1
fi

output_dir=$(mktemp -d "${TMPDIR:-/tmp}/pine-wasm-node.XXXXXX")
trap 'rm -rf "$output_dir"' EXIT HUP INT TERM

printf '%s\n' '+ cargo build -p pine-wasm --target wasm32-unknown-unknown'
cargo build -p pine-wasm --target wasm32-unknown-unknown

wasm="$target_dir/wasm32-unknown-unknown/debug/pine_wasm.wasm"
if [ ! -f "$wasm" ]; then
    printf 'error: expected wasm artifact was not produced: %s\n' "$wasm" >&2
    exit 1
fi

printf '%s\n' '+ cargo run -p pine-wasm --example generate_node_bindings'
host=$(rustc -vV | sed -n 's/^host: //p')
if [ -z "$host" ]; then
    printf '%s\n' 'error: could not determine the native Rust host target' >&2
    exit 1
fi
cargo run --quiet -p pine-wasm --example generate_node_bindings --target "$host" -- \
    "$wasm" "$output_dir"

bindings="$output_dir/pine_wasm.js"
if [ ! -f "$bindings" ]; then
    printf 'error: expected Node bindings were not produced: %s\n' "$bindings" >&2
    exit 1
fi

printf '%s\n' '+ node scripts/tests/wasm_node_smoke.cjs'
node scripts/tests/wasm_node_smoke.cjs "$bindings"
