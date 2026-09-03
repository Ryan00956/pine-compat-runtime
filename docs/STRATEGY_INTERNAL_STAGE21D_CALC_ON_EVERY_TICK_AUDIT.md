# Strategy Internal Stage 21d calc_on_every_tick Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Const bool
`calc_on_every_tick=true` executes strategy code on each host-provided forming
update, rolling `var` back from the confirmed checkpoint and keeping `varip`
across forming updates. Default false skips forming strategy execution.
Historical bars are unchanged. Series or non-bool values stay rejected.
Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Only const bool `calc_on_every_tick` is accepted.
- `true` replays each forming update from the 21c confirmed checkpoint, so
  abandoned forming orders, fills, alerts, plots, and drawings do not leak.
- `var` starts from confirmed state on each forming replay; `varip` is seeded
  from the previous forming update.
- `false` (default) does not execute strategy code on forming updates; the
  confirmed update runs the bar once.
- Historical execution is identical with the flag on or off because historical
  bars have no host-provided realtime ticks.

## Named Runtime Goldens

- `runtime_strategy_calc_on_every_tick.json` (historical market entry, same
  fills as default timing)
- `matrix.json`

## Files

- `crates/pine-ir/src/strategy.rs`
- `crates/pine-builtins/src/namespaces/core.rs`
- `crates/pine-builtins/src/registry.rs`
- `crates/pine-sema/src/analyzer/strategy/declaration.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-runtime/src/runtime/realtime.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_calc_on_every_tick.pine`
- `tests/fixtures/sema/unsupported_strategy_calc_on_every_tick_series.pine`
- `tests/fixtures/sema/unsupported_strategy_declaration_properties.pine`
- `tests/fixtures/sema/unsupported_strategy_process_orders_on_close_with_recalc.pine`
- `tests/fixtures/runtime/strategy_calc_on_every_tick.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 657/104 passed, saved as
`{SCRATCH}/stage21d-baseline-1.log`, `{SCRATCH}/stage21d-baseline-2.log`,
`{SCRATCH}/stage21d-sema-baseline-1.log`, and
`{SCRATCH}/stage21d-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime strategy` 661 passed.
`cargo test -p pine-sema strategy` 106 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 599 passed. Host parity 523 required
runtime goldens. WASM 628 passed. Log: `{SCRATCH}/stage21d-verify.sh.log`.

## Remaining Exclusions

21e bar magnifier host contract. `use_bar_magnifier` and
`fill_orders_on_standard_ohlc` stay rejected. External tick feeds and alert
delivery stay out of scope. Public `StrategyResult` stays unchanged.
