# Strategy Internal Stage 14g Short Exit Ticks Audit

Status: closed. This slice adds fixture-backed single-trigger `strategy.exit`
profit and loss tick covers against the current market short-entry subset.

## Closed Subset

- `strategy.exit(id, from_entry, profit=ticks)` against a matching open short
  converts a positive tick distance from the matching short entry price into a
  limit below that price (`entry - ticks * mintick`) and covers when a later
  historical bar has `low <= limit - verification_offset`.
- `strategy.exit(id, from_entry, loss=ticks)` against a matching open short
  converts into a stop above the matching short entry price
  (`entry + ticks * mintick`) and covers when a later historical bar has
  `high >= stop`.
- Cover fills reuse the Stage 14f short stop/limit path: short-exit slippage,
  signed closed-trade quantity, and cover PnL.
- Pending-short relative-tick attachment and omitted-`from_entry` all-entry
  fan-out remain no-op on shorts. Brackets landed in Stage 14h and trailing in
  Stage 14i.

## Evidence

- `tests/fixtures/runtime/strategy_exit_profit_short.pine`
- `tests/fixtures/runtime/strategy_exit_loss_short.pine`
- `tests/snapshots/runtime_strategy_exit_profit_short.json`
- `tests/snapshots/runtime_strategy_exit_loss_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14g_*`
- Runtime tests `strategy_exit_profit_ticks_short_cover_below_entry` and
  `strategy_exit_loss_ticks_short_cover_above_entry`

## Unchanged Claims

Short limit/stop/stop-limit entries, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short brackets landed in
Stage 14h and trailing in Stage 14i.
