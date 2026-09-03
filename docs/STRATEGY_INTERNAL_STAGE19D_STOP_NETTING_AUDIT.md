# Strategy Internal Stage 19d Stop And Stop-Limit Generic-Order Netting Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Stop and stop-limit
`strategy.order` long and short fills reuse Stage 19b signed netting after
trigger selection (stop) or activation plus a later limit fill (stop-limit).
Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Eligible stop generic orders fill at the stop price through
  `apply_generic_market_order_netting`.
- Stop-limit generic orders keep the existing activation state across bars and
  fill at the limit price on a later bar.
- Creation-bar prices do not fill. Cancel before fill, including after
  activation, removes the intent without cash, trade, or position mutation.
- Short stop and stop-limit orders may now be placed while net long.
- Unaffordable open remainder rejects the whole fill (`E_STRATEGY_MARGIN`).
- Price-based `strategy.entry()` reversal stays unrouted for 19e.

## Named Runtime Goldens

- `runtime_strategy_order_stop_long_against_short.json`
- `runtime_strategy_order_stop_short_against_long.json`
- `runtime_strategy_order_stop_long_flatten_short.json`
- `runtime_strategy_order_stop_short_reduce_long.json`
- `runtime_strategy_order_stop_limit_long_against_short.json`
- `runtime_strategy_order_stop_limit_short_against_long.json`
- `runtime_strategy_order_stop_limit_long_flatten_short.json`
- `runtime_strategy_order_stop_limit_short_reduce_long.json`
- `matrix.json` (conformance notes and fixtures)

## Remaining Exclusions

19e routes price-based `strategy.entry()` reversal. 19f covers replacement,
cancellation collisions, and close-rule interaction. Omitted `qty` for
`strategy.short` stays unsupported.
