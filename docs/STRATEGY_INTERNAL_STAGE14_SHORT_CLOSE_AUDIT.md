# Strategy Internal Stage 14d Short Close Audit

Status: closed. This slice adds fixture-backed `strategy.close` and
`strategy.close_all` for the current market short-entry subset.

## Closed Subset

- `strategy.close(id)` closes matching open short trades at the current bar
  close, including fixed `qty` and `qty_percent` partials where `qty` wins.
- `strategy.close_all()` flattens all open short trades at the current bar
  close.
- Cover fills use short-exit slippage (`price + slippage`).
- Closed-trade `qty` is signed negative. Realized profit is
  `(exit - entry) * signed_qty - commission`.
- Cash debits `qty * cover_price + commission`.
- Wrong-id, flat, and repeated closes stay no-op.
- Short `strategy.exit` stop/limit covers landed in Stage 14f.

## Evidence

- `tests/fixtures/runtime/strategy_close_short.pine`
- `tests/fixtures/runtime/strategy_close_all_short.pine`
- `tests/snapshots/runtime_strategy_close_short.json`
- `tests/snapshots/runtime_strategy_close_all_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14d_*`
- Runtime test `strategy_close_short_records_signed_qty_and_cover_pnl`

## Unchanged Claims

Short profit/loss/bracket/trailing `strategy.exit`, short limit/stop entries,
and `margin_short` runtime behavior remain unsupported. Market reversals landed
in Stage 14e and short stop/limit `strategy.exit` in Stage 14f.
