# Strategy Internal Stage 14c Market Short Entry Audit

Status: closed. This slice adds the first fixture-backed market `strategy.short`
entry subset without reversal, short exits, or short price-based entries.

## Closed Subset

- Analyzer accepts `strategy.entry(..., strategy.short)` for market entries,
  including named const `strategy.short` aliases.
- Analyzer rejects short `limit`/`stop` arguments with `E_CALL_ARG_NAME`.
- Runtime places a pending market short entry and fills it at the next
  historical bar open, using short-side slippage (`price - slippage`).
- Fills while flat or already short open or increase short exposure.
- Fills or placements while net long are no-op and do not reverse.
- Net long entries while short are also no-op.
- Public position size is negative. Average price is the short-side average.
- Cash increases by `qty * fill_price - commission`. Equity is `cash + size *
  close`.
- `strategy.max_contracts_held_short` tracks the filled short quantity.
- Short closes landed in Stage 14d and short stop/limit `strategy.exit` in
  Stage 14f. Supported `strategy.exit` profit/loss/bracket/trailing forms still
  do not flatten shorts.
- Reduce-only `strategy.order(..., strategy.short)` is unchanged.

## Evidence

- `tests/fixtures/sema/supported_strategy_entry_short.pine`
- `tests/fixtures/sema/supported_strategy_entry_named_const_short_direction.pine`
- `tests/fixtures/sema/unsupported_strategy_entry_short.pine`
- `tests/fixtures/sema/unsupported_strategy_entry_named_const_short_direction.pine`
- `tests/fixtures/runtime/strategy_entry_short.pine`
- `tests/fixtures/runtime/strategy_entry_short.pine`
- `tests/snapshots/runtime_strategy_entry_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_entry_short.json`
- Broker tests `stage14c_*`

## Unchanged Claims

Short limit/stop/stop-limit entries, short profit/loss/bracket/trailing
`strategy.exit`, and `margin_short` runtime behavior remain unsupported.
Market reversals landed in Stage 14e, short closes in Stage 14d, and short
stop/limit `strategy.exit` in Stage 14f.
