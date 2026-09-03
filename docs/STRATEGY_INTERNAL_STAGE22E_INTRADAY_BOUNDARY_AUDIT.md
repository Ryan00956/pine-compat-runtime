# Strategy Internal Stage 22e Intraday Boundary Foundation Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Host-neutral
intraday windows are keyed from bar timestamps and the chart timeframe
already available to the runtime. Chart timeframes at or below 1D use the
UTC day of `time`. Timeframes higher than 1D use the bar timestamp so one
chart bar is one window. A new window zeros the filled-order count, seeds a
finite equity baseline, and clears window-scoped trips while permanent
`max_drawdown` stops remain. Same-window bars keep the baseline and
counters. Missing-bar gaps start a new window. Non-positive timeframes fail
closed to the UTC-day key. This runtime has no session calendar.
`strategy.risk.max_intraday_loss` and
`strategy.risk.max_intraday_filled_orders` stay rejected. Public
`StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- `intraday_window_key(time_ms, timeframe_seconds)` uses UTC day when
  `timeframe_seconds <= 86400` and the bar timestamp when
  `timeframe_seconds > 86400`.
- Non-positive `timeframe_seconds` uses the UTC-day key.
- `reset_intraday_risk_state` no-ops on the same window, otherwise zeros
  `intraday_filled_orders`, seeds `intraday_equity_baseline` from finite
  equity, and leaves the baseline unset for non-finite equity.
- Window-scoped trips (`MaxIntradayLoss`, `MaxIntradayFilledOrders`) clear
  on reset; `MaxDrawdown` stays tripped and blocked.
- Ordinary same-UTC-day bars share one window. A gap that skips a UTC day
  starts a new window. Higher-than-daily bars on the same UTC day each start
  a new window.
- Historical pre-script scheduling passes the default chart timeframe
  seconds and open-mark equity.
- Pine `strategy.risk.max_intraday_*` calls remain semantic rejections.

## Files

- `crates/pine-runtime/src/strategy/broker/risk.rs`
- `crates/pine-runtime/src/strategy/broker/risk_storage_tests.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 692/122 passed, saved as
`{SCRATCH}/stage22e-baseline-1.log`, `{SCRATCH}/stage22e-baseline-2.log`,
`{SCRATCH}/stage22e-sema-baseline-1.log`, and
`{SCRATCH}/stage22e-sema-baseline-2.log`.

Fail-closed: non-positive timeframe uses UTC-day; exactly 1D stays UTC-day;
same-window bars keep counters and baseline; non-finite equity does not seed
a baseline; `MaxDrawdown` survives reset.

Owner-local: `cargo test -p pine-runtime --lib risk` 41 passed.
`cargo test -p pine-sema strategy` 122 passed.
`cargo test -p pine-runtime strategy` 703 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 613 passed. Host parity 537 required
runtime goldens. WASM 642 passed. Log: `{SCRATCH}/stage22e-verify.sh.log`.

## Remaining Exclusions

22f accepts `strategy.risk.max_intraday_loss` and
`strategy.risk.max_intraday_filled_orders` on this window model. 22g adds
consecutive-loss-day closeout. Public risk-state schema stays private. This
runtime still has no instrument session calendar.
