# Strategy Internal Stage 15b Short Forced Liquidation Audit

Status: closed. This slice adds fixture-backed short `margin_short` forced
liquidation against the current short-entry subset.

## Closed Subset

- Historical short margin calls use `bar.high` as the adverse current price.
- A call triggers only for open short exposure with explicit active
  `margin_short` when available funds are negative.
- Cover quantity uses the documented four-times-cover algorithm with whole-unit
  truncation, then clamps to the current absolute short size.
- The public liquidation order id is `Margin Call` with direction
  `strategy.long`.
- Closed-trade quantity is signed negative and cover PnL uses
  `(exit - entry) * signed_qty`.
- Partial liquidation keeps the remaining short average price and updates
  `strategy.opentrades.capital_held`.
- Full liquidation flattens the short and clears pending exits for the affected
  entry id.
- Configured order slippage is not applied to margin-call fills.
- Short `strategy.margin_liquidation_price` landed in Stage 15c.

## Evidence

- `tests/fixtures/runtime/strategy_margin_call_short.pine`
- `tests/snapshots/runtime_strategy_margin_call_short.json`
- CLI/Python/WASM host parity for `runtime_strategy_margin_call_short.json`
- Broker tests `margin_call_partially_liquidates_short_position`,
  `margin_call_clamps_to_full_short_position`, and
  `margin_call_short_is_noop_when_available_funds_cover_margin`
- Runtime test `strategy_margin_call_short_partially_covers_on_later_high`

## Unchanged Claims

Short `strategy.margin_liquidation_price` landed in Stage 15c. Symbol precision
rounding, currency conversion, and public account schema expansion remain
unsupported.
