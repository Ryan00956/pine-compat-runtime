# Strategy Internal Stage 18f Historical Fill Path Audit

Status: partial after closeout review on 2026-09-03. The implemented scheduler
subset passed `scripts/verify.sh`, but it does not meet the original Stage 18f
OHLC-path acceptance criteria. Historical broker fills are dispatched from
ordered `HistoricalFillStep` family ticks instead of ad-hoc call order in the
bar loop. Public JSON shape is unchanged except the new collision golden.

Official review date: 2026-09-02.
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Path Rule

Pre-script path ticks, in order:

1. market closes at `bar.open`
2. market entries at `bar.open`
3. long limit (low)
4. long stop (high)
5. long stop-limit (high then low)
6. short limit (high)
7. short stop (low)
8. short stop-limit (low then high)

Bar-close path ticks when `process_orders_on_close` is true:

1. same-bar market closes at `bar.close`
2. same-bar market entries at `bar.close`

This slice does not walk high-before-low vs low-before-high from bar direction.
Path ticks are family-based so existing goldens stay identical.
Same-tick pyramiding still fills every eligible order inside one family.

Price-family fills no longer `clear_all` remaining pending entries after a
successful batch, so a later family on the same bar can still fill. Market
fills still clear remaining pending entries after the first market fill.

## Named Runtime Goldens

- `runtime_strategy_fill_path_limit_stop_collision.json` (new; LIM then STP)

No other runtime goldens changed.

## Remaining Exclusions

Bar-direction OHLC walk (high-first vs low-first), same-price ties beyond
family order, path-correct stop-limit sequencing, and exit/margin candidates
on the same path key are not in this slice. These are now owned by Stage 18g in
`docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`. Later completed stages own
netting, OCA, recalculation, and risk independently of this remaining path
accuracy work.
