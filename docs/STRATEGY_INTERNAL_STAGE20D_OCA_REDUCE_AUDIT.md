# Strategy Internal Stage 20d `strategy.oca.reduce` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. After a generic
`strategy.order` fill, same-group `strategy.oca.reduce` peers reduce remaining
quantity by the filled quantity and are removed when reduced to zero. Same-bar
remaining candidates use the reduced quantity before filling. Unrelated groups
stay independent. `strategy.exit` `oca_name` remains rejected. Public JSON
shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- `strategy.order` accepts const `oca_type=strategy.oca.reduce` with
  const/simple `oca_name`.
- After a successful generic fill, same-group pending generic orders reduce
  remaining quantity by the filled quantity (`|D|` for cross-zero fills).
  Peers reduced to zero are removed in creation-key order.
- Same-tick remaining candidates that were already taken apply the reduced
  remaining quantity before filling, or skip when remaining is zero.
- Unrelated `oca_name` groups and `strategy.oca.none` / `strategy.oca.cancel`
  peers are not reduced.
- Margin-rejected fills do not reduce peers.
- `strategy.exit` `oca_name` stays rejected.
- Entry-family OCA reduce stays later.

## Named Runtime Goldens

- `runtime_strategy_order_oca_reduce.json` (A qty 1 fills, B qty 2 reduces to 1
  and fills; size `2`)
- `runtime_strategy_order_oca_reduce_zero.json` (A qty 2 fills, B qty 1 reduces
  to 0 and is removed; size `2`)
- `matrix.json`

## Files

- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/oca.rs`
- `crates/pine-runtime/src/strategy/broker/types.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_order_oca_reduce.pine`
- `tests/fixtures/sema/unsupported_strategy_orders.pine`
- `tests/fixtures/runtime/strategy_order_oca_reduce.pine`
- `tests/fixtures/runtime/strategy_order_oca_reduce_zero.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 611/126 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 590 passed. Host parity 514 required
runtime goldens. Log: `{SCRATCH}/stage20d-verify.sh.log`.

## Remaining Exclusions

20e maps `strategy.exit` `oca_name`. Entry-family OCA reduce is still later.
