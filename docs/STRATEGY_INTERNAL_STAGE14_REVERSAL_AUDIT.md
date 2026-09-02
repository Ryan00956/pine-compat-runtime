# Strategy Internal Stage 14e Reversal Audit

Status: closed. This slice adds fixture-backed market `strategy.entry`
reversals without short limit/stop entries or `strategy.exit` short flattening.

## Public Record Shape

A reversing market entry is two broker operations at the same fill bar/time:

1. Flatten the opposite net position through the existing `strategy.close_all`
   path at the reverse fill price. Closed trades keep signed quantity and
   cover/exit PnL. `exit_id` is the closed entry id.
2. Open a new market entry for the requested quantity in the new direction.

Net position after reversal equals the requested quantity on the new side.
The new entry order quantity is the requested qty, not qty plus opposite size.

## Closed Subset

- Market `strategy.entry(..., strategy.short)` while net long closes the long
  at the next-bar-open fill price, then opens the requested short quantity.
- Market `strategy.entry(..., strategy.long)` while net short closes the short
  at the next-bar-open fill price, then opens the requested long quantity.
- Same-direction pyramiding, reduce-only `strategy.order` shorts, and
  price-based entries are unchanged. Price-based short entries stay rejected.
- Short `strategy.exit` stop/limit covers landed in Stage 14f.

## Evidence

- `tests/fixtures/runtime/strategy_entry_short_reverses_long.pine`
- `tests/fixtures/runtime/strategy_entry_long_reverses_short.pine`
- `tests/snapshots/runtime_strategy_entry_short_reverses_long.json`
- `tests/snapshots/runtime_strategy_entry_long_reverses_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14e_*`
- Runtime test `strategy_entry_short_reverses_long_at_next_bar_open`

## Unchanged Claims

Short limit/stop/stop-limit entries, short profit/loss/bracket/trailing
`strategy.exit`, generic `strategy.order()` netting, and `margin_short` runtime
behavior remain unsupported. Short stop/limit `strategy.exit` landed in
Stage 14f.
