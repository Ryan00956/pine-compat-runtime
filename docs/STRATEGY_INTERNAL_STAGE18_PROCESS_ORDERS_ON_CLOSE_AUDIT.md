# Strategy Internal Stage 18e Process Orders On Close Audit

Status: closed on 2026-09-02 after `scripts/verify.sh`. Const bool
`process_orders_on_close` is stored on strategy settings and fills eligible
market entry, generic order, close, and close-all intents at the creation bar
close after script statements. `immediately=true` still fills during the close
command. Public JSON shape is unchanged.

Official review date: 2026-09-02.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-docs/faq/strategies/

## Behavior

- Default remains next-bar-open market fills.
- When `process_orders_on_close=true`, the scheduler runs
  `BarCloseMarketFills` after script statements and before pending-exit
  evaluation, filling same-bar market closes then market entries at
  `bar.close`.
- Script-visible series on the signal bar remain pre-fill; public trades and
  equity include the fill on that bar.
- `immediately=true` fills during the command, so later same-bar statements
  see the close even when the declaration also sets
  `process_orders_on_close=true`.
- Series `process_orders_on_close` and other timing flags
  (`calc_on_order_fills`, `calc_on_every_tick`, `use_bar_magnifier`,
  `fill_orders_on_standard_ohlc`) remain rejected.

## Named Runtime Goldens

- `runtime_strategy_process_orders_on_close.json`
- `runtime_strategy_process_orders_on_close_close.json`
- `runtime_strategy_process_orders_on_close_immediately.json`
- `matrix.json` (conformance notes/fixtures)

## Remaining Exclusions

Historical OHLC path-candidate ordering is 18f. Recalculation and realtime
tick scheduling are Stage 21.
