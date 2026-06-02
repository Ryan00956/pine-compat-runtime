# Strategy Internal Stage 3 Utilities Audit

Status: in progress.

This audit tracks `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md` Stage 3: Small
Independent Strategy Utilities. Stage 3 should add narrow helpers that fit the
current one-net-long broker without requiring a richer order book.

## Slice 0: `strategy.close_all()`

Status: closed on 2026-06-02.

Goal: add the smallest useful close helper for the current supported broker:
`strategy.close_all()` closes the current long position without requiring an
entry id.

Context checked:

- `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md` Stage 3 scope;
- current `tests/fixtures/conformance.tsv` strategy rows;
- `crates/pine-builtins/src/namespaces/strategy.rs`;
- `crates/pine-sema/src/analyzer/strategy.rs`;
- `crates/pine-runtime/src/builtins/strategy.rs`;
- `crates/pine-runtime/src/strategy/broker/fills.rs`;
- existing `strategy.close` runtime, CLI, Python, and WASM coverage.

Implemented:

- added a no-argument `strategy.close_all()` builtin signature;
- accepted `strategy.close_all()` only in strategy-mode scripts;
- kept indicator-mode calls on the existing `E_STRATEGY_MODE` diagnostic path;
- added broker/runtime dispatch that closes the current supported long position
  at the current bar close;
- kept flat and already-closed calls as no-op;
- reused the existing close path so pending exits for the closed entry are
  cancelled and public strategy output shape remains unchanged;
- added semantic fixtures, runtime fixture, CLI snapshot coverage, Python host
  coverage, and WASM host coverage.

Compatibility boundary:

- No public JSON schema or public pending-order shape changed.
- `strategy.close_all()` only applies to the current one-net-long supported
  broker. There is no partial close, close ordering across multiple entries,
  short support, pyramiding support, or generic cancellation API.
- `strategy.cancel`, `strategy.cancel_all`, risk APIs, and rich trade namespace
  functions remain unsupported.

Validation:

```text
cargo fmt --all --check
cargo test -p pine-builtins strategy_close_all
cargo test -p pine-sema strategy_close_all
cargo test -p pine-runtime strategy_close_all
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
cargo test -p pine-wasm close_all
python3 -m pytest python/tests/test_bindings.py -q
cargo test -p pine-runtime runtime_fixtures_match_incremental_append_execution
git diff --check
scripts/verify.sh
```

Result:

- `cargo fmt --all --check` passed.
- `cargo test -p pine-builtins strategy_close_all` passed.
- `cargo test -p pine-sema strategy_close_all` passed.
- `cargo test -p pine-runtime strategy_close_all` passed.
- CLI runtime and matrix snapshots were updated and then rechecked without
  `UPDATE_SNAPSHOTS`.
- `cargo test -p pine-wasm close_all` passed.
- `python3 -m pytest python/tests/test_bindings.py -q` passed after rebuilding
  and reinstalling the Python wheel for the new runtime dispatch.
- `cargo test -p pine-runtime runtime_fixtures_match_incremental_append_execution`
  passed.
- `git diff --check` passed.
- `scripts/verify.sh` passed, including workspace formatting, clippy, tests,
  structure guardrail, wasm32 check, Python wheel rebuild/install, and Python
  tests.

Next slice candidate:

- Add win/loss/even trade count variables for the current closed-trade list if
  Slice 0 closes cleanly.
