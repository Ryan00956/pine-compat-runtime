# Strategy Internal Stage 14f Short Exit Audit

Status: closed. This slice adds fixture-backed single-trigger `strategy.exit`
stop and limit covers against the current market short-entry subset.

## Closed Subset

- `strategy.exit(id, from_entry, stop=price)` against a matching open or pending
  short entry covers when a later historical bar has `high >= stop`.
- `strategy.exit(id, from_entry, limit=price)` against a matching open or pending
  short entry covers when a later historical bar has
  `low <= limit - verification_offset`.
- Cover fills use short-exit slippage (`price + slippage`).
- Closed-trade `qty` is signed negative. Realized profit is
  `(exit - entry) * signed_qty - commission`.
- Cash debits `qty * cover_price + commission`.
- Profit/loss ticks landed in Stage 14g, brackets in Stage 14h, and trailing
  in Stage 14i.

## Evidence

- `tests/fixtures/runtime/strategy_exit_stop_short.pine`
- `tests/fixtures/runtime/strategy_exit_limit_short.pine`
- `tests/snapshots/runtime_strategy_exit_stop_short.json`
- `tests/snapshots/runtime_strategy_exit_limit_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14f_*`
- Runtime test `strategy_exit_stop_short_covers_on_later_high_crossing_bar`

## Unchanged Claims

Short limit/stop/stop-limit entries, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short profit/loss ticks
landed in Stage 14g, brackets in Stage 14h, and trailing in Stage 14i.
