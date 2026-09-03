# Strategy Internal Stage 20b Explicit `strategy.oca.none` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Const/simple `oca_name`
with explicit `strategy.oca.none` is accepted on `strategy.order`. Grouped
pending orders stay independent. `strategy.oca.cancel` and
`strategy.oca.reduce` remain rejected. Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- `strategy.order` accepts const/simple `oca_name` and const `oca_type` equal
  to `strategy.oca.none`, including named const alias chains. Omitted `oca_type`
  defaults to none.
- Membership is stored as OCA group `(name, none)` on the pending generic
  order. Peers in that group do not cancel or reduce each other.
- `oca_type=strategy.oca.cancel` and `strategy.oca.reduce` remain semantic
  errors (`E_CALL_ARG_VALUE`). Series `oca_name` remains rejected.
- `strategy.exit` `oca_name` stays rejected.

## Named Runtime Goldens

- `runtime_strategy_order_oca_none.json` (two same-group limit orders both
  fill; size `2`)
- `matrix.json` (conformance notes and fixtures)

## Files

- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/oca.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_order_oca_none.pine`
- `tests/fixtures/sema/unsupported_strategy_order_oca_cancel.pine`
- `tests/fixtures/sema/unsupported_strategy_order_oca_reduce.pine`
- `tests/fixtures/sema/unsupported_strategy_order_oca_series_name.pine`
- `tests/fixtures/runtime/strategy_order_oca_none.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 607/124 passed, saved as `{SCRATCH}/stage20b-baseline-*.log`
and `{SCRATCH}/stage20b-sema-baseline-*.log`.

Fail-closed: supported `oca.none` and runtime fixture rejected with
`E_CALL_ARG_NAME`; cancel/reduce lacked the none-only diagnostic.

Owner-local after implement: `cargo test -p pine-sema strategy_order` and
`cargo test -p pine-runtime strategy_order_oca`.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 587 passed. Host parity 511 required
runtime goldens. Log: `{SCRATCH}/stage20b-verify.sh.log`.

## Remaining Exclusions

20c implements `strategy.oca.cancel` peer cancellation. 20d implements
`strategy.oca.reduce`. `strategy.exit` `oca_name` stays later in 20e.
