# Strategy Internal Stage 19f Replacement, Cancellation, And Close-Rule Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Same-id generic-order
replacement, shared-id cancellation with pending exits, and generic reduction
allocation under FIFO plus id-specific ANY are fixture-backed. Public JSON
shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Same-direction same-id `strategy.order` replacement updates quantity, kind,
  price, metadata, and created bar while preserving the internal key.
  Stop-limit replacement clears activation.
- Opposite-direction same-id replacement cancels the old intent and places a
  new one with a new internal key. `strategy.entry` and `strategy.order` share
  this id namespace.
- `strategy.exit` ids are a separate namespace that may match an entry/order
  id. `strategy.cancel(id)` clears matching pending generic orders and pending
  exits without mutating filled position or cash.
- Generic-order reductions allocate FIFO by default.
- When `close_entries_rule` is ANY and the generic-order id matches an open
  entry of the close direction, allocation uses that entry id. Unmatched ANY
  stays FIFO. Broader non-id-specific ANY is unchanged.

## Named Runtime Goldens

- `runtime_strategy_order_replace_limit_with_stop.json`
- `runtime_strategy_order_replace_long_with_short.json`
- `runtime_strategy_order_cancel_shared_id.json`
- `runtime_strategy_order_reduce_fifo.json` (closes oldest entry `A`)
- `runtime_strategy_order_reduce_any_matching_id.json` (closes matching `B`)
- `matrix.json` (conformance notes and fixtures)

Inspected goldens: schemaVersion 8; strategy keys remain
`orders`/`trades`/`position`/`equity`/`alerts`/`diagnostics`.

## Files

- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/close_orders.rs`
- `crates/pine-runtime/src/strategy/broker/netting_matrix_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/runtime/strategy_order_replace_limit_with_stop.pine`
- `tests/fixtures/runtime/strategy_order_replace_long_with_short.pine`
- `tests/fixtures/runtime/strategy_order_cancel_shared_id.pine`
- `tests/fixtures/runtime/strategy_order_reduce_fifo.pine`
- `tests/fixtures/runtime/strategy_order_reduce_any_matching_id.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` twice, 595 passed, saved as
`{SCRATCH}/stage19f-baseline-1.log` and `{SCRATCH}/stage19f-baseline-2.log`.

Fail-closed: `{SCRATCH}/stage19f-failclosed.log`. Direction-change replacement
and ANY matching-id reduction failed under the old implementation; FIFO,
unmatched ANY, same-direction kind replacement, and shared-id cancel already
passed.

Owner-local after implement: `cargo test -p pine-runtime strategy` 603 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 586 passed. Host parity 510 required
runtime goldens. Log: `{SCRATCH}/stage19f-verify.sh.log`.

## Remaining Exclusions

Stage 20 owns OCA groups and unified cancellation across remaining families.
Omitted `qty` for `strategy.short` stays unsupported.
