# Strategy Internal Stage 22a Risk Configuration And Triggered-State Skeleton Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Broker state stores
`StrategyRiskRules` separately from `StrategyRiskState`. Hooks exist before
order admission, after fill, at UTC-day reset, and before forced close.
Clone/rollback preserves configured and tripped state. Every `strategy.risk.*`
call stays rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Configuration (`allow_entry_direction`, drawdown/loss/size/filled-order
  limits) is separate from tripped rules, blocked admission, and intraday
  counters.
- `check_risk_before_order` rejects later entries and pending market-long
  placement only after state is tripped. Unconfigured brokers still admit
  orders.
- `check_risk_after_fill` counts open fills and can trip a configured
  filled-order limit in tests. Pine cannot set that limit yet.
- `reset_intraday_risk_state` clears window counters on UTC-day change and
  keeps permanent tripped state.
- `check_risk_before_forced_close` is invoked from margin evaluation.
- `strategy.risk.allow_entry_in`, `max_drawdown`, `max_intraday_loss`,
  `max_position_size`, and `max_intraday_filled_orders` remain semantic
  rejections.

## Files

- `crates/pine-runtime/src/strategy/broker/risk.rs`
- `crates/pine-runtime/src/strategy/broker/risk_storage_tests.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/state.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/fills.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 662/107 passed, saved as
`{SCRATCH}/stage22a-baseline-1.log`, `{SCRATCH}/stage22a-baseline-2.log`,
`{SCRATCH}/stage22a-sema-baseline-1.log`, and
`{SCRATCH}/stage22a-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime --lib risk` 7 passed.
`cargo test -p pine-runtime strategy` 669 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 599 passed. Host parity 523 required
runtime goldens. WASM 628 passed. Log: `{SCRATCH}/stage22a-verify.sh.log`.

## Remaining Exclusions

22b accepts `strategy.risk.allow_entry_in()` with documented direction
constants. Other `strategy.risk.*` calls stay rejected. Public risk-state
schema stays private.
