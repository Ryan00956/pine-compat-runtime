# Strategy Internal Stage 14i Short Exit Trailing Audit

Status: closed. This slice adds fixture-backed trailing `strategy.exit` covers
against the current market short-entry subset.

## Closed Subset

- `trail_price + trail_offset` and `trail_points + trail_offset` cover a matching
  open short entry.
- `trail_points` converts activation from the matching short entry price as
  `entry - ticks * mintick`.
- Activation occurs on a later eligible bar when `low <= activation_price` and
  never fills on the activation bar. The active stop is `low + offset`.
- After activation, an eligible bar fills when `high >= active_stop`; otherwise
  the active stop ratchets downward only (`low + offset`) and never increases.
- Cover fills reuse the Stage 14f short stop/limit path.
- Pending-short relative `trail_points` attachment and omitted-`from_entry`
  all-entry fan-out remain no-op on shorts.

## Evidence

- `tests/fixtures/runtime/strategy_exit_trail_price_fill_short.pine`
- `tests/fixtures/runtime/strategy_exit_trail_points_fill_short.pine`
- `tests/snapshots/runtime_strategy_exit_trail_price_fill_short.json`
- `tests/snapshots/runtime_strategy_exit_trail_points_fill_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14i_*`
- Runtime test `strategy_exit_trail_price_short_activates_then_covers`

## Unchanged Claims

Short stop/stop-limit entries, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short limit entries landed
in Stage 14j.
