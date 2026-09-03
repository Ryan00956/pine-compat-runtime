# Strategy Internal Stage 18a Scheduler Characterization Audit

Status: closed on 2026-09-02. This slice does not change fill timing, public
strategy output, conformance, or snapshots.

Stage 18a documents the current historical broker phase order and routes those
calls through a scheduler facade without reordering them.

## Current Historical Order

For each historical bar in a strategy script:

1. Eligible entry/order fills (market, then long limit/stop/stop-limit, then
   short limit/stop/stop-limit)
2. Open-trade extreme update
3. Margin call (long then short)
4. Builtin refresh
5. Script statements
6. Pending exit fills
7. Equity snapshot
8. Output commit

Official review date: 2026-09-02. Sources:
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

Default `strategy.close()` still fills during script statements on the
creation bar. Stage 18c changes that.

## Files

- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/runtime/mod.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `docs/STRATEGY_INTERNAL_STAGE18_SCHEDULER_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`
- `docs/RELEASE_NOTES.md`

## Tests

Scheduler traces cover market entry, price entry, close, exit, and margin
call. Indicator runs emit no strategy phase trace. Strategy goldens stay
byte-identical.

## Remaining Exclusions

Pending market-close storage is Stage 18b. Default next-tick close is Stage
18c. `immediately` and `process_orders_on_close` are 18d/18e.
