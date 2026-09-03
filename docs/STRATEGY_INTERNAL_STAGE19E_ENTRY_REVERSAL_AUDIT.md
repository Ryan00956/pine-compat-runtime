# Strategy Internal Stage 19e Price-Based `strategy.entry()` Reversal Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Opposite-side limit,
stop, and stop-limit `strategy.entry` fills flatten existing exposure then open
the requested quantity. Pyramiding applies to the new side, not the flatten
quantity. Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Price-based long entries may be placed while net short, and price-based short
  entries may be placed while net long (`can_place_long_entry` /
  `can_place_short_entry`). Same-side pyramiding still uses `can_open_*`.
- Fill-path early-out uses `same_side_*_entry_blocked`, so opposite pending
  entries are not cleared before trigger.
- Creation-bar prices do not fill. Cancel before fill removes the intent
  without cash, trade, or position mutation.
- On opposite fill, `fill_pending_generic_or_entry` uses
  `EntryPyramidingMode::EnforceLimit`: flatten via `close_all_position`, then
  open the requested quantity. Same-side price-based fills keep the same-tick
  pyramiding exception.
- Public orders record the new entry quantity, not `P+D`. Flatten is a
  `strategy.close_all` alert, matching market `strategy.entry` reversal.
- Stop-limit still activates without filling (`activated_bar_index < bar_index`)
  and fills at the limit price on a later bar (fixture fill price `4`).
- Pyramiding applies to the resulting new side. Two same-tick reverse longs
  after flattening a short both open when they are eligible in that pass.
- Active-entry exit attachments for the flattened side are cleared.
- Unaffordable remainder after flatten can fail the open (pre-existing
  market-reversal order: close_all then affordability).

## Named Runtime Goldens

- `runtime_strategy_entry_limit_reverses_short.json`
- `runtime_strategy_entry_limit_reverses_long.json`
- `runtime_strategy_entry_limit_reverses_short_qty.json` (short qty `2` then
  long qty `1` ends size `1`, not `P+D`)
- `runtime_strategy_entry_stop_reverses_short.json`
- `runtime_strategy_entry_stop_reverses_long.json`
- `runtime_strategy_entry_stop_limit_reverses_short.json` (fill price `4`)
- `runtime_strategy_entry_stop_limit_reverses_long.json` (fill price `4`)
- `matrix.json` (conformance notes and fixtures)

Inspected goldens: last public order qty is the requested entry qty; flatten is
`strategy.close_all`; schemaVersion 8; strategy keys remain
`orders`/`trades`/`position`/`equity`/`alerts`/`diagnostics`.

## Files

- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/netting_matrix_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/runtime/strategy_entry_{limit,stop,stop_limit}_reverses_{short,long}.pine`
- `tests/fixtures/runtime/strategy_entry_limit_reverses_short_qty.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline (before 19e behavior edits):
`cargo test -p pine-runtime strategy` twice, saved as
`{SCRATCH}/stage19e-baseline-1.log` and `{SCRATCH}/stage19e-baseline-2.log`.

Owner-local after implement: `cargo test -p pine-runtime strategy`.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 581 passed. Host parity 505 required
runtime goldens. Log: `{SCRATCH}/stage19e-verify.sh.log`.

## Remaining Exclusions

19f covers generic-order replacement, cancellation collisions, and close-rule
interaction. Omitted `qty` for `strategy.short` stays unsupported.
