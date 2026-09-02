# Strategy Internal Stage 14o Short Stop-Limit Order Audit

Status: closed. This slice adds fixture-backed short `strategy.order`
stop-limit fills against the current short-entry subset.

## Closed Subset

- Analyzer accepts `strategy.order(..., strategy.short, qty=..., stop=price,
  limit=price)` with positive stop and limit prices and explicit positive qty.
- Runtime places a pending short stop-limit that bypasses the
  `strategy.entry()` pyramiding limit.
- Placement while net long is a no-op and does not reverse or reduce.
- The pending order never fills on its creation bar.
- A later historical bar activates when `low <= stop` and does not fill on the
  activation bar.
- A subsequent historical bar fills at the limit price when `high >= limit`, or
  above the configured verified limit threshold.
- Fills reuse short-entry accounting while flat or already short, including
  adding to an existing short beyond the `pyramiding` cap.
- Market `strategy.order(..., strategy.short)` stays reduce-only and remains a
  no-op while flat.
- Omitted `qty` for `strategy.short` stays rejected.

## Evidence

- `tests/fixtures/sema/supported_strategy_order.pine`
- `tests/fixtures/sema/unsupported_strategy_orders.pine`
- `tests/fixtures/runtime/strategy_order_stop_limit_short.pine`
- `tests/snapshots/runtime_strategy_order_stop_limit_short.json`
- CLI/Python/WASM host parity for
  `runtime_strategy_order_stop_limit_short.json`
- Broker tests `stage14o_*`
- Runtime test
  `strategy_order_stop_limit_short_adds_to_existing_short_without_pyramiding`

## Unchanged Claims

Generic `strategy.order()` netting remains unsupported. Short `margin_short`
capital held and affordability landed in Stage 15a. Short forced liquidation
remains unsupported.
