# Strategy Internal Stage 14j Short Limit Entry Audit

Status: closed. This slice adds fixture-backed short `strategy.entry` limit
fills against the current market short-entry subset.

## Closed Subset

- Analyzer accepts `strategy.entry(..., strategy.short, limit=price)` with a
  positive limit.
- Analyzer still rejects short `stop` and short `stop+limit` with
  `E_CALL_ARG_NAME`.
- Runtime places a pending short limit entry while flat or already short.
- Placement while net long is a no-op and does not reverse.
- The pending order never fills on its creation bar.
- A later historical bar fills at the limit price when `high >= limit`, or
  above the configured verified limit threshold.
- Fills reuse short-entry accounting: negative position size, cash credits
  `qty * fill_price`, and `strategy.max_contracts_held_short` tracking.
- Same-tick eligible short limits in one fill pass can exceed the
  `strategy.entry()` pyramiding limit, matching the long limit exception.
- Short `strategy.order(..., limit=...)` stays rejected.

## Evidence

- `tests/fixtures/sema/supported_strategy_entry_limit_short.pine`
- `tests/fixtures/sema/unsupported_strategy_entry_short.pine`
- `tests/fixtures/sema/unsupported_strategy_entry_named_const_short_direction.pine`
- `tests/fixtures/runtime/strategy_entry_limit_short.pine`
- `tests/snapshots/runtime_strategy_entry_limit_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_entry_limit_short.json`
- Broker tests `stage14j_*`
- Runtime test `strategy_entry_limit_short_fills_on_later_high_crossing_bar`

## Unchanged Claims

Short price-based `strategy.order()`, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short stop entries landed
in Stage 14k. Short stop-limit entries landed in Stage 14l.
