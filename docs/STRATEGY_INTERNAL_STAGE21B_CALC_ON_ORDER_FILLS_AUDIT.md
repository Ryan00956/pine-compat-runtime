# Strategy Internal Stage 21b Historical calc_on_order_fills Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Const bool
`calc_on_order_fills=true` re-executes strategy statements after historical
fills, refreshes live `strategy.*` state, and can fill later Stage 18 price
ticks on the same bar. Series or non-bool values stay rejected.
`calc_on_every_tick` stays rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Only const bool `calc_on_order_fills` is accepted. Series/non-bool values
  fail with `expects const bool`.
- After a pre-script, margin, immediately, bar-close, or exit fill, the
  scheduler restores the bar-start `ta.*`/series checkpoint and runs another
  script pass. `var` persists. Plots overwrite the current bar.
- Price-based pending entries placed on an extra pass may fill on a later
  Stage 18 tick of the same bar. Market entries still fill on a later bar, or
  at bar close when `process_orders_on_close` is set.
- Extra passes are counted on internal profiles and stop with
  `strategy recalculation pass limit exceeded` when the 21a guardrail trips.
- `calc_on_order_fills=false` keeps the previous one-pass-per-bar path.

## Named Runtime Goldens

- `runtime_strategy_calc_on_order_fills.json` (market fill then same-bar limit)
- `runtime_strategy_calc_on_order_fills_false.json` (same script, no extra fill)
- `runtime_strategy_calc_on_order_fills_exit_avg.json` (exit from post-entry
  average)
- `matrix.json`

## Files

- `crates/pine-ir/src/strategy.rs`
- `crates/pine-builtins/src/namespaces/core.rs`
- `crates/pine-builtins/src/registry.rs`
- `crates/pine-sema/src/analyzer/strategy/declaration.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/strategy/broker/state.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_calc_on_order_fills.pine`
- `tests/fixtures/sema/unsupported_strategy_calc_on_order_fills_series.pine`
- `tests/fixtures/sema/unsupported_strategy_declaration_properties.pine`
- `tests/fixtures/runtime/strategy_calc_on_order_fills.pine`
- `tests/fixtures/runtime/strategy_calc_on_order_fills_false.pine`
- `tests/fixtures/runtime/strategy_calc_on_order_fills_exit_avg.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 647/102 passed, saved as
`{SCRATCH}/stage21b-baseline-1.log`, `{SCRATCH}/stage21b-baseline-2.log`,
`{SCRATCH}/stage21b-sema-baseline-1.log`, and
`{SCRATCH}/stage21b-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime strategy` 652 passed.
`cargo test -p pine-sema strategy` 104 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 598 passed. Host parity 522 required
runtime goldens. WASM 627 passed. Log: `{SCRATCH}/stage21b-verify.sh.log`.

## Remaining Exclusions

21c realtime broker rollback checkpoints. 21d `calc_on_every_tick`. 21e bar
magnifier host contract. Same-bar market re-entry still needs
`process_orders_on_close` or the next bar open. Stop-limit limit legs still
fill on a later bar after activation. Public `StrategyResult` stays unchanged.
