# Strategy Internal Stage 22f `max_intraday_loss` And `max_intraday_filled_orders` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`.
`strategy.risk.max_intraday_loss` accepts simple positive finite numeric
`value` with required `strategy.cash` or `strategy.percent_of_equity`.
`strategy.risk.max_intraday_filled_orders` accepts a simple positive finite
integer `count`. Both optional simple `alert_message` values are stored on
the risk-owned close. On trigger the broker cancels pending orders, flattens
through a risk-owned market close, and blocks later `strategy.entry` and
`strategy.order` actions until the next 22e intraday window. Remaining
`strategy.risk.*` calls stay rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- Loss uses maximum window equity minus mark-to-market equity, including
  open adverse excursion. Cash trips when that amount is at least `value`.
  Percent trips when amount/max*100 is at least `value`, or when equity is
  non-positive.
- Window max equity starts from the 22e baseline and rises on new highs.
- Public filled orders increment the window counter. Risk-owned flatten
  closes do not add a second public order event. Reaching `count` trips the
  filled-order rule.
- Evaluation runs after historical fills, after trade extremes, after
  margin, and after post-script fills. Flatten of a filled-order trip waits
  until the current fill is committed, then runs before `calc_on_order_fills`
  recalculation.
- Window reset clears these trips and counters while `max_drawdown` stays
  permanent.
- Zero, negative, non-integer count, percent over 100, series, unknown type,
  and indicator-mode calls are rejected.

## Named Runtime Goldens

- `runtime_strategy_risk_max_intraday_filled_orders.json`
- `runtime_strategy_risk_max_intraday_filled_orders_reset.json`
- `runtime_strategy_risk_max_intraday_loss_cash.json`
- `runtime_strategy_risk_max_intraday_loss_percent.json`
- `matrix.json` (conformance notes and fixtures)

## Incremental / Realtime

Tripped window state lives on `StrategyRiskState` and is cloned/restored with
broker snapshots, so Stage 21c forming rollback covers abandoned forming
trips. Recalculation after fills sees the flattened blocked book.

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
- `crates/pine-runtime/src/strategy/broker/fills.rs`
- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
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
- `tests/fixtures/sema/supported_strategy_risk_max_intraday_loss.pine`
- `tests/fixtures/sema/supported_strategy_risk_max_intraday_filled_orders.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_*.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_intraday_filled_orders_*.pine`
- `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`
- `tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders.pine`
- `tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders_reset.pine`
- `tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders_reset_bars.csv`
- `tests/fixtures/runtime/strategy_risk_max_intraday_loss_cash.pine`
- `tests/fixtures/runtime/strategy_risk_max_intraday_loss_percent.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 703/122 passed, saved as `{SCRATCH}/stage22f-baseline-1.log`,
`{SCRATCH}/stage22f-baseline-2.log`, `{SCRATCH}/stage22f-sema-baseline-1.log`,
and `{SCRATCH}/stage22f-sema-baseline-2.log`.

Fail-closed: zero, fraction, percent over 100, unknown type, series, indicator
mode, remaining `strategy.risk.max_cons_loss_days` rejection, and
below-threshold no-flatten before cash/percent trip, fill-count flatten,
window reset, and order block.

Owner-local: `cargo test -p pine-runtime --lib risk` 50 passed.
`cargo test -p pine-sema strategy` 133 passed.
`cargo test -p pine-builtins registers_strategy_risk` 5 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 617 passed. Host parity 541 required
runtime goldens. WASM 646 passed. Log: `{SCRATCH}/stage22f-verify.sh.log`.

## Remaining Exclusions

22g adds consecutive-loss-day closeout. Public risk-state schema stays
private. This runtime still has no instrument session calendar.
