# Strategy Internal Stage 14n Short Stop Order Audit

Status: closed. This slice adds fixture-backed short `strategy.order` stop
fills against the current short-entry subset.

## Closed Subset

- Analyzer accepts `strategy.order(..., strategy.short, qty=..., stop=price)`
  with a positive stop and explicit positive qty.
- Analyzer still rejects short `stop+limit` with `E_CALL_ARG_NAME`.
- Runtime places a pending short stop that bypasses the `strategy.entry()`
  pyramiding limit.
- Placement while net long is a no-op and does not reverse or reduce.
- The pending order never fills on its creation bar.
- A later historical bar fills at the stop price when `low <= stop`.
- Fills reuse short-entry accounting while flat or already short, including
  adding to an existing short beyond the `pyramiding` cap.
- Market `strategy.order(..., strategy.short)` stays reduce-only and remains a
  no-op while flat.
- Omitted `qty` for `strategy.short` stays rejected.

## Evidence

- `tests/fixtures/sema/supported_strategy_order.pine`
- `tests/fixtures/sema/unsupported_strategy_orders.pine`
- `tests/fixtures/runtime/strategy_order_stop_short.pine`
- `tests/snapshots/runtime_strategy_order_stop_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_order_stop_short.json`
- Broker tests `stage14n_*`
- Runtime test
  `strategy_order_stop_short_adds_to_existing_short_without_pyramiding`

## Unchanged Claims

Short stop-limit `strategy.order()` fills landed in Stage 14o. Generic
`strategy.order()` netting and `margin_short` runtime behavior remain
unsupported.
