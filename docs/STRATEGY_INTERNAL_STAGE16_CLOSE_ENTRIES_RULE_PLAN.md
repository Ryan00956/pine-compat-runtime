# Strategy Internal Stage 16 Close-Entries-Rule Expansion Plan

Status: 16a-16b closed. Later slices follow
`docs/PURE_INTERNAL_STRATEGY_CLOSE_ENTRIES_RULE_DESIGN.md`.

Stage 15 closed the short-margin account subset. Stage 16 extends the existing
id-specific `close_entries_rule="ANY"` allocation to the current short-entry
ledger.

## Goal

Prove that `"ANY"` close/exit allocation already stored in strategy settings
selects short ledger entries by exact id, without changing omitted-`from_entry`
or `close_all` FIFO behavior.

## Non-Goals

- v1-v4 `strategy()` compatibility;
- omitted-`from_entry` `"ANY"` allocation;
- `strategy.close_all()` non-FIFO ordering;
- generic `strategy.order()` netting;
- public pending-order schema expansion.

## Slice Order

### 16a. Id-specific ANY for shorts

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE16_CLOSE_ENTRIES_RULE_ANY_SHORT_AUDIT.md`.

`strategy.close(id)` and `strategy.exit(..., from_entry=id)` under
`close_entries_rule="ANY"` allocate matching short ledger entries by exact id.

### 16b. Same-id partial ANY for shorts

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE16_CLOSE_ENTRIES_RULE_ANY_SHORT_PARTIAL_AUDIT.md`.

A partial `strategy.exit(..., from_entry=id, qty=...)` under `"ANY"` covers
same-id short ledger entries in stable open-trade order.

## Compatibility Rules

- `tests/fixtures/conformance.tsv` remains the support authority.
- Public strategy JSON, Python dictionaries, and WASM JSON stay on the current
  schema unless a later slice designs a change.
- Existing long `"ANY"` fixtures must keep their current serialized outputs.

## Completion Gate

Each slice closes with broker tests, runtime fixtures where behavior is
user-visible, synchronized docs, and `scripts/verify.sh`.
