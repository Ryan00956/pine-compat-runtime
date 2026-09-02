# Strategy Internal Stage 16b Short Same-Id Partial ANY Audit

Status: closed. This slice adds fixture-backed same-entry-id partial
`close_entries_rule="ANY"` allocation for short `strategy.exit` covers.

## Closed Subset

- Under `"ANY"`, a partial `strategy.exit(..., from_entry=id, qty=...)` covers
  matching short ledger entries that share that exact id in stable open-trade
  order.
- The oldest matching short is fully covered first; leftover quantity continues
  into later same-id shorts.
- Remaining unmatched quantity on the last consumed short stays open at its
  original average price.
- Distinct-id short `"ANY"` close/exit allocation from Stage 16a is unchanged.
- Omitted-`from_entry` exits and `strategy.close_all()` stay on the FIFO path.

## Evidence

- `tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial_short.pine`
- `tests/snapshots/runtime_strategy_close_entries_rule_any_exit_same_id_partial_short.json`
- CLI/Python/WASM host parity for that snapshot
- Broker test
  `close_entries_rule_any_internal_partial_exit_same_short_id_preserves_ledger_order`
- Runtime test
  `strategy_close_entries_rule_any_partial_exit_same_short_id_preserves_ledger_order`

## Unchanged Claims

Omitted-`from_entry` `"ANY"` allocation, `strategy.close_all()` non-FIFO
ordering, generic `strategy.order()` netting, custom OCA, and public pending
order schema expansion remain unsupported.
