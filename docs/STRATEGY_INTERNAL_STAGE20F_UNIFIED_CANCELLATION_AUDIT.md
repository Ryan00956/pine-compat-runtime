# Strategy Internal Stage 20f Unified Cancellation Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. `strategy.cancel(id)`
searches pending entries, generic orders, exits, deferred relative exits, and
pending closes through one order-book lookup, including shared public ids.
`strategy.cancel_all()` clears those families plus reservations, stop-limit
activation, and OCA membership exactly once. Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- One `OrderBook::cancel_id` lookup collects matching members, then removes
  pending entries/orders, pending exits, deferred relative exits, pending
  closes, and OCA membership for that public id.
- `OrderBook::clear_all` is the single `strategy.cancel_all()` path.
- `clear_exits_for_entry` prunes leftover OCA membership.
- Shared-id collisions cancel every matching family. Unknown, filled, and
  already-cancelled ids remain no-op.
- No public pending-order or OCA schema.

## Named Runtime Goldens

- `runtime_strategy_cancel_shared_id_entry_exit.json` (pending entry and exit
  with id `X` are both cancelled)
- `runtime_strategy_cancel_shared_id_close_exit.json` (pending close and exit
  with id `L` are cancelled; filled long remains size `1`)
- `runtime_strategy_cancel_all_families.json` (OCA orders, reserved exits, and
  pending close are cancelled; filled long remains size `2`)
- `matrix.json`

## Files

- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/runtime/strategy_cancel_shared_id_entry_exit.pine`
- `tests/fixtures/runtime/strategy_cancel_shared_id_close_exit.pine`
- `tests/fixtures/runtime/strategy_cancel_all_families.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 626/125 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 595 passed. Host parity 519 required
runtime goldens. Log: `{SCRATCH}/stage20f-verify.sh.log`.

## Remaining Exclusions

Stage 21 recalculation and realtime scheduling. Mixed entry/order/exit OCA
groups and series `oca_name` stay later. Public pending-order/OCA schema stays
private.
