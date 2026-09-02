# Strategy Internal Stage 15c Short Margin Liquidation Price Audit

Status: closed. This slice adds fixture-backed short
`strategy.margin_liquidation_price` against the current short-margin subset.

## Closed Subset

- Open short exposure with explicit active `margin_short` returns the broker
  price where equity equals required short margin:
  `cash / (abs(position_size) * (1 + margin_short / 100))`.
- The value updates after supported short fills, covers, and forced
  liquidation on the same timing boundary as other strategy state variables.
- It returns `na` while flat, without active `margin_short` for the current
  short exposure, or when the solved price is not finite.
- Unlike long `margin_long=100`, a full short margin ratio remains solvable.
- Long liquidation-price fixtures stay unchanged.
- Symbol tick rounding stays unsupported.

## Evidence

- `tests/fixtures/runtime/strategy_margin_call_short.pine`
- `tests/snapshots/runtime_strategy_margin_call_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_margin_call_short.json`
- Broker tests `margin_liquidation_price_is_na_without_active_short_margin_state`
  and `margin_liquidation_price_is_finite_for_full_short_margin`
- Runtime test `strategy_margin_call_short_partially_covers_on_later_high`

## Unchanged Claims

Symbol precision rounding, currency conversion, and public account schema
expansion remain unsupported.
