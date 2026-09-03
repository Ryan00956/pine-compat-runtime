# Strategy Internal Stage 20e Exit OCA Naming Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Const/simple
`strategy.exit` `oca_name` maps onto the existing implicit
`strategy.oca.reduce` reservation model. Grouped exits share overlapping
quantity, and a fill reduces same-group peers. Series `oca_name` remains
rejected. Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- `strategy.exit` accepts const/simple `oca_name`. Exit OCA type is always
  reduce; `oca_type` is not part of the supported public signature.
- Same-group exits do not exclusive-reserve against each other, so a full stop
  can cover a partial limit peer. A fill reduces remaining same-group reserved
  quantity and removes peers reduced to zero.
- Ungrouped qty/qty-percent exits keep exclusive reservation.
- Coverage includes brackets, trailing exits, fixed qty, percent qty,
  replacement, full-position cleanup, and different open-trade keys.
- Series `oca_name` stays rejected.

## Named Runtime Goldens

- `runtime_strategy_exit_oca_reduce.json` (partial limit plus full stop in one
  group; stop fills qty `2`, limit is reduced away)
- `runtime_strategy_exit_oca_reduce_bracket.json` (bracket plus qty stop; stop
  fills qty `1`, remaining bracket fills later)
- `matrix.json`

## Files

- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-builtins/src/registry.rs`
- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/state.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/oca.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/exit_placement.rs`
- `crates/pine-runtime/src/strategy/broker/pending_exits.rs`
- `crates/pine-runtime/src/strategy/broker/exit_orders.rs`
- `crates/pine-runtime/src/strategy/broker/loss_limit_brackets.rs`
- `crates/pine-runtime/src/strategy/broker/loss_profit_brackets.rs`
- `crates/pine-runtime/src/strategy/broker/stop_profit_brackets.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_reservations.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_exit_oca_name.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_oca_name_series.pine`
- `tests/fixtures/runtime/strategy_exit_oca_reduce.pine`
- `tests/fixtures/runtime/strategy_exit_oca_reduce_bracket.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 618/125 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 592 passed. Host parity 516 required
runtime goldens. Log: `{SCRATCH}/stage20e-verify.sh.log`.

## Remaining Exclusions

20f unifies `strategy.cancel` / `strategy.cancel_all` across pending families.
Mixed entry/order/exit OCA groups and series `oca_name` stay later.
