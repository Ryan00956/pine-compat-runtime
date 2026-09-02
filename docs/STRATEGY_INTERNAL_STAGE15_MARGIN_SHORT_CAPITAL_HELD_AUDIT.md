# Strategy Internal Stage 15a Short Margin Capital Held Audit

Status: closed. This slice adds fixture-backed short `margin_short` capital held
and short-entry affordability against the current short-entry subset.

## Closed Subset

- `strategy.opentrades.capital_held` returns `0.0` while flat when any margin
  setting is active.
- Open short exposure with explicit active `margin_short` returns
  `abs(position_size) * close * margin_short / 100`.
- Supported short market, limit, stop, and stop-limit fills share one
  affordability helper at the actual fill price.
- Unaffordable short fills emit `E_STRATEGY_MARGIN`, leave cash, position,
  orders, and trades unchanged, and clear attached pending exits.
- Long-margin capital held, long-entry affordability, and long forced
  liquidation stay unchanged.
- Short forced liquidation landed in Stage 15b. Short
  `strategy.margin_liquidation_price` landed in Stage 15c.

## Evidence

- `tests/fixtures/runtime/strategy_margin_capital_held_short.pine`
- `tests/fixtures/runtime/strategy_margin_entry_affordability_short.pine`
- `tests/snapshots/runtime_strategy_margin_capital_held_short.json`
- `tests/snapshots/runtime_strategy_margin_entry_affordability_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `margin_short_*`
- Runtime tests
  `strategy_capital_held_history_reads_follow_short_margin_state` and
  `strategy_margin_entry_affordability_short_rejects_then_accepts_covered_fill`

## Unchanged Claims

Short forced liquidation landed in Stage 15b. Short
`strategy.margin_liquidation_price` landed in Stage 15c. Symbol precision
rounding, currency conversion, and public account schema expansion remain
unsupported.
