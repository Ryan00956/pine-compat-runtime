# Strategy Internal Stage 14l Short Stop-Limit Entry Audit

Status: closed. This slice adds fixture-backed short `strategy.entry`
stop-limit fills against the current market short-entry subset.

## Closed Subset

- Analyzer accepts `strategy.entry(..., strategy.short, stop=price,
  limit=price)` with positive stop and limit prices.
- Runtime places a pending short stop-limit entry while flat or already short.
- Placement while net long is a no-op and does not reverse.
- The pending order never fills on its creation bar.
- A later historical bar activates when `low <= stop` and does not fill on the
  activation bar.
- A subsequent historical bar fills at the limit price when `high >= limit`, or
  above the configured verified limit threshold.
- Fills reuse short-entry accounting: negative position size, cash credits
  `qty * fill_price`, and `strategy.max_contracts_held_short` tracking.
- Same-tick eligible short stop-limits in one fill pass can exceed the
  `strategy.entry()` pyramiding limit, matching the long stop-limit exception.
- Short `strategy.order(..., stop=..., limit=...)` stays rejected.

## Evidence

- `tests/fixtures/sema/supported_strategy_entry_stop_limit_short.pine`
- `tests/fixtures/runtime/strategy_entry_stop_limit_short.pine`
- `tests/snapshots/runtime_strategy_entry_stop_limit_short.json`
- CLI/Python/WASM host parity for
  `runtime_strategy_entry_stop_limit_short.json`
- Broker tests `stage14l_*`
- Runtime test
  `strategy_entry_stop_limit_short_activates_then_fills_on_later_high_crossing_bar`

## Unchanged Claims

Short limit `strategy.order` fills landed in Stage 14m. Short stop
`strategy.order` fills landed in Stage 14n. Short stop-limit `strategy.order`
fills landed in Stage 14o. Generic `strategy.order()` netting and
`margin_short` runtime behavior remain unsupported.
