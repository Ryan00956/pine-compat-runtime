# Strategy Internal Stage 22g Consecutive-Loss-Day Closeout Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`.
`strategy.risk.max_cons_loss_days` accepts a simple positive finite integer
`count` and optional simple `alert_message`. Each completed 22e window with
negative realized closed-trade profit counts as a loss day. A profitable or
no-trade window resets the streak. Missing-bar gaps do not insert a no-trade
window. After `count` consecutive observed loss windows the broker cancels
pending orders, flattens, and permanently blocks later `strategy.entry` and
`strategy.order` actions. Undocumented `strategy.risk.*` names stay rejected.
Public `StrategyResult` is unchanged. Stage 22 is closed.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- Realized window PnL is the sum of closed-trade profits recorded in that
  window. Open mark-to-market is not a loss day by itself.
- Window classification runs when the 22e key changes. Negative PnL increments
  `consecutive_loss_days`; zero or positive PnL resets it to 0.
- Reaching `count` trips `MaxConsLossDays` at the start of the next window,
  which is a permanent stop like `max_drawdown`. Leftover open exposure is
  flattened at that window's bar-open mark, not at window equity.
- Zero, negative, non-integer, non-finite, series, and indicator-mode calls
  are rejected (`E_CALL_ARG_VALUE` / simple type / `E_STRATEGY_MODE`).

## Named Runtime Goldens

- `runtime_strategy_risk_max_cons_loss_days.json`
- `runtime_strategy_risk_max_cons_loss_days_no_trade.json`
- `matrix.json` (conformance notes and fixtures)

## Incremental / Realtime

Consecutive-day and tripped state live on `StrategyRiskState` and are
cloned/restored with broker snapshots, so Stage 21c forming rollback covers
abandoned forming trips.

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
- `crates/pine-runtime/src/strategy/broker/closed_trades.rs`
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
- `tests/fixtures/sema/supported_strategy_risk_max_cons_loss_days.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_cons_loss_days_*.pine`
- `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`
- `tests/fixtures/runtime/strategy_risk_max_cons_loss_days.pine`
- `tests/fixtures/runtime/strategy_risk_max_cons_loss_days_bars.csv`
- `tests/fixtures/runtime/strategy_risk_max_cons_loss_days_no_trade.pine`
- `tests/fixtures/runtime/strategy_risk_max_cons_loss_days_no_trade_bars.csv`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 712/133 passed, saved as `{SCRATCH}/stage22g-baseline-1.log`,
`{SCRATCH}/stage22g-baseline-2.log`, `{SCRATCH}/stage22g-sema-baseline-1.log`,
and `{SCRATCH}/stage22g-sema-baseline-2.log`.

Fail-closed: zero, fraction, series, indicator mode, remaining undocumented
`strategy.risk.not_a_rule` rejection, profit-day reset, no-trade reset,
missing-bar gap continuing the observed streak, and leftover open exposure
flattened at bar-open mark rather than window equity.

Owner-local: `cargo test -p pine-runtime --lib risk` 57 passed after the
mark/equity split, including
`max_cons_loss_days_flattens_open_position_at_mark_not_equity`.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0 without `UPDATE_SNAPSHOTS`. Python 619 passed.
Host parity 543 required runtime goldens. WASM 648 passed. Log:
`{SCRATCH}/verify.sh.log`. Flatten uses the new window's bar-open mark;
leftover open exposure is covered by
`max_cons_loss_days_flattens_open_position_at_mark_not_equity`.

## Remaining Exclusions

Undocumented `strategy.risk.*` names stay rejected. Public risk-state schema
stays private. This runtime still has no instrument session calendar.
