# Strategy Internal Stage 14a Boundary Audit

Status: closed. This slice does not change syntax acceptance, runtime fills,
conformance status, snapshots, matrix output, or public strategy output.

Stage 14a freezes the current short/reversal rejection boundary before the
broker becomes direction-aware.

## Locked Boundary

- `strategy.short` remains a readable constant.
- `strategy.entry(..., strategy.short)` stays rejected, including named const
  alias chains.
- `strategy.order(..., strategy.short, limit/stop/...)` stays rejected.
- Reduce-only market `strategy.order(..., strategy.short, qty=...)` still
  shrinks existing long exposure and is a no-op while flat. It does not open
  short exposure.
- `strategy.max_contracts_held_short` remains `0.0` after long entries, partial
  exits, full closes, and reduce-only short orders.
- `strategy.max_contracts_held_all` continues to equal the long maximum while
  shorts are unsupported.

## Evidence

- `tests/fixtures/sema/unsupported_strategy_entry_short.pine`
- `tests/fixtures/sema/unsupported_strategy_entry_named_const_short_direction.pine`
- `tests/fixtures/sema/unsupported_strategy_orders.pine`
- `tests/fixtures/runtime/strategy_position_state.pine`
- `tests/fixtures/runtime/strategy_order_reduce_long.pine`
- `tests/fixtures/runtime/strategy_order_short_flat_noop.pine`
- Conformance guards in `crates/pine-cli/src/conformance/guards/strategy.rs`
  require the short-entry, named-const short, reduce-only order, and
  `strategy.max_contracts_held_short` fixtures.
- Broker tests:
  - `stage14_boundary_lock_keeps_short_exposure_and_max_short_at_zero`
  - `stage14_reduce_only_short_order_does_not_open_short_exposure`

## Unchanged Claims

Short entries, automatic reversal, short price-based generic orders, short
margin, and short exits remain outside the supported subset.
