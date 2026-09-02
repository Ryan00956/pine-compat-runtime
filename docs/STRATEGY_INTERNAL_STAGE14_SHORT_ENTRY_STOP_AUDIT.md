# Strategy Internal Stage 14k Short Stop Entry Audit

Status: closed. This slice adds fixture-backed short `strategy.entry` stop
fills against the current market short-entry subset.

## Closed Subset

- Analyzer accepts `strategy.entry(..., strategy.short, stop=price)` with a
  positive stop.
- Analyzer still rejected short `stop+limit` with `E_CALL_ARG_NAME` until
  Stage 14l.
- Runtime places a pending short stop entry while flat or already short.
- Placement while net long is a no-op and does not reverse.
- The pending order never fills on its creation bar.
- A later historical bar fills at the stop price when `low <= stop`.
- Fills reuse short-entry accounting: negative position size, cash credits
  `qty * fill_price`, and `strategy.max_contracts_held_short` tracking.
- Same-tick eligible short stops in one fill pass can exceed the
  `strategy.entry()` pyramiding limit, matching the long stop exception.
- Short `strategy.order(..., stop=...)` and short stop-limit entries stay
  rejected.

## Evidence

- `tests/fixtures/sema/supported_strategy_entry_stop_short.pine`
- `tests/fixtures/runtime/strategy_entry_stop_short.pine`
- `tests/snapshots/runtime_strategy_entry_stop_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_entry_stop_short.json`
- Broker tests `stage14k_*`
- Runtime test `strategy_entry_stop_short_fills_on_later_low_crossing_bar`

## Unchanged Claims

Short price-based `strategy.order()`, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short stop-limit entries
landed in Stage 14l.
