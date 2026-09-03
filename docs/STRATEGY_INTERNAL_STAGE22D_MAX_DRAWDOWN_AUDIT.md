# Strategy Internal Stage 22d `strategy.risk.max_drawdown()` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`.
`strategy.risk.max_drawdown` accepts simple positive finite numeric `value`
with required `strategy.cash` or `strategy.percent_of_equity`. Cash compares
peak-equity drawdown amount, including open adverse excursion. Percent
compares that amount to maximum equity and also trips when equity is
non-positive. On trigger the broker cancels pending orders, flattens through
a risk-owned market close, and permanently blocks later `strategy.entry` and
`strategy.order` actions. UTC-day reset keeps the stop. Other
`strategy.risk.*` calls stay rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- Simple positive finite `value` and const/simple `strategy.cash` or
  `strategy.percent_of_equity` are accepted, including named const alias
  chains. Optional simple `alert_message` is stored on the risk-owned close.
- Zero, negative, non-finite, percent over 100, series, and unknown type
  values are rejected (`E_CALL_ARG_VALUE` / simple type error).
- Indicator scripts still get `E_STRATEGY_MODE`.
- Drawdown amount is the greater of the existing `strategy.max_drawdown`
  reporting metric and peak-equity minus mark-to-market equity at the
  evaluation price. Peak equity is maximum equity before the open trade.
- Cash trips when that amount is at least `value`. Percent trips when
  amount/peak*100 is at least `value`, or when peak/current equity is
  non-positive.
- Evaluation runs after historical open fills, after trade extremes, after
  margin evaluation, and after post-script fills/exits.
- On trigger: trip `MaxDrawdown`, cancel all pending orders, flatten at the
  evaluation mark, and set `blocked_order_placement`. Later entries and
  generic orders are no-ops. The stop is permanent across UTC-day reset.
- Broker snapshot/restore preserves tripped flatten state.

## Named Runtime Goldens

- `runtime_strategy_risk_max_drawdown_cash.json`
- `runtime_strategy_risk_max_drawdown_percent.json`
- `runtime_strategy_risk_max_drawdown_blocks_order.json`
- `matrix.json` (conformance notes and fixtures)

## Incremental / Realtime

Not a dedicated forming-bar fixture. Tripped state lives on
`StrategyRiskState` and is already cloned/restored with broker snapshots, so
Stage 21c forming rollback covers abandoned forming trips. Recalculation
after fills sees the flattened blocked book.

## Files

- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-builtins/src/registry.rs`
- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-sema/src/analyzer/unsupported.rs`
- `crates/pine-sema/src/analyzer/calls/helpers.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/risk.rs`
- `crates/pine-runtime/src/strategy/broker/risk_storage_tests.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-runtime/src/tests/builtin_registry.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-cli/src/runtime_snapshots/bars.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_risk_max_drawdown.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_zero.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_percent_over.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_unknown_type.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_series.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_indicator.pine`
- `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`
- `tests/fixtures/runtime/strategy_risk_max_drawdown_bars.csv`
- `tests/fixtures/runtime/strategy_risk_max_drawdown_cash.pine`
- `tests/fixtures/runtime/strategy_risk_max_drawdown_percent.pine`
- `tests/fixtures/runtime/strategy_risk_max_drawdown_blocks_order.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 687/116 passed, saved as `{SCRATCH}/stage22d-baseline-1.log`,
`{SCRATCH}/stage22d-baseline-2.log`, `{SCRATCH}/stage22d-sema-baseline-1.log`,
and `{SCRATCH}/stage22d-sema-baseline-2.log`.

Fail-closed: zero, percent over 100, unknown type, series value, indicator
mode, remaining `strategy.risk.*` rejections, and below-threshold no-flatten
before cash/percent trip, pending cancel, order block, and snapshot restore.

Owner-local: `cargo test -p pine-runtime --lib risk` 30 passed.
`cargo test -p pine-sema strategy` 122 passed.
`cargo test -p pine-runtime strategy` 692 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 613 passed. Host parity 537 required
runtime goldens. WASM 642 passed. Log: `{SCRATCH}/stage22d-verify.sh.log`.

## Remaining Exclusions

22e establishes the intraday/session reset foundation before
`max_intraday_loss` and `max_intraday_filled_orders`. Remaining
`strategy.risk.*` calls stay rejected. Public risk-state schema stays private.
