# Strategy Internal Stage 16a Short Close-Entries-Rule ANY Audit

Status: closed. This slice adds fixture-backed id-specific
`close_entries_rule="ANY"` allocation for short `strategy.close(id)` and
`strategy.exit(..., from_entry=id)`.

## Closed Subset

- Analyzer acceptance of `close_entries_rule="ANY"` is unchanged.
- Runtime `strategy.close(id)` under `"ANY"` closes matching short ledger
  entries with that exact id before other open shorts.
- Runtime `strategy.exit(..., from_entry=id)` under `"ANY"` covers matching
  short ledger entries with that exact id.
- Remaining unmatched shorts stay open.
- Omitted-`from_entry` exits and `strategy.close_all()` stay on the FIFO path.
- Same-entry-id partial long `"ANY"` allocation is unchanged. Same-entry-id
  partial short `"ANY"` allocation landed in Stage 16b.
- Broader non-id-specific `"ANY"` allocation stays unsupported.

## Evidence

- `tests/fixtures/runtime/strategy_close_entries_rule_any_close_short.pine`
- `tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry_short.pine`
- `tests/snapshots/runtime_strategy_close_entries_rule_any_close_short.json`
- `tests/snapshots/runtime_strategy_close_entries_rule_any_exit_from_entry_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests
  `close_entries_rule_any_internal_close_uses_exact_short_entry_id_allocation`
  and
  `close_entries_rule_any_internal_exit_from_entry_uses_exact_short_entry_id_allocation`
- Runtime test
  `strategy_close_entries_rule_any_uses_short_entry_id_allocation`

## Unchanged Claims

Omitted-`from_entry` `"ANY"` allocation, `strategy.close_all()` non-FIFO
ordering, generic `strategy.order()` netting, custom OCA, and public pending
order schema expansion remain unsupported.
