# Strategy Internal Stage 12 Declaration Properties Plan

Status: design gate opened on 2026-06-05. Runtime behavior, conformance claims,
and public output are unchanged in this slice.

## Background

Stage 11 closed the current partial `strategy.close()` subset. Before widening
another broker operation, the remaining strategy gaps need a clean declaration
property boundary. The live conformance matrix already documents a non-trivial
`strategy()` subset:

- positive const `initial_capital`;
- `strategy.fixed` and `strategy.percent_of_equity` default quantities;
- `strategy.commission.cash_per_contract`,
  `strategy.commission.cash_per_order`, and `strategy.commission.percent`;
- non-negative fixed-tick `slippage`;
- non-negative fixed-tick `backtest_fill_limits_assumption`;
- finite non-negative `margin_long` and `margin_short` declaration parsing;
- active `margin_long` behavior for the current long-only capital-held,
  affordability, and forced-liquidation subset.

The old gap audit still described this area as mostly unsupported and pointed at
already-closed close/exit work. Stage 12 starts by making the next declaration
property decision explicit instead of accepting a low-value keyword alias.

## Goal

Choose the next declaration-property expansion only after its semantic boundary,
broker effect, conformance wording, and public output contract are clear.

## Non-Goals

- No runtime widening in this design slice.
- No acceptance of `pyramiding`, `calc_on_order_fills`,
  `calc_on_every_tick`, `process_orders_on_close`, `default_qty_type=strategy.cash`,
  `currency`, `close_entries_rule`, `risk_free_rate`, `use_bar_magnifier`,
  `fill_orders_on_standard_ohlc`, or strategy alert/order-fill settings.
- No short exposure, reversal, multi-entry ledger, OCA, or public order-event
  schema expansion.

## Candidate Slices

### Slice 1: Boundary Lock

Refresh or add semantic fixtures proving the unsupported declaration properties
remain rejected with stable diagnostics. This should cover the properties above
without duplicating already-supported declaration forms.

Close criteria:

- Unsupported property fixture coverage is explicit.
- `tests/fixtures/conformance.tsv` keeps the supported declaration subset
  narrow.
- CLI conformance and matrix snapshots remain unchanged except for intentional
  fixture registration.

### Slice 2: Property Selection Review

Pick exactly one declaration property only if it has a defensible current-broker
semantics. The first runtime slice should prefer a property that changes real
behavior and can be fixture-backed without opening multi-entry, realtime, or
short-position semantics.

Current assessment:

- `pyramiding=0/1` as an accepted no-op is low value because the broker still has
  one net long position and ignores repeated long entries.
- `process_orders_on_close` changes order timing and should wait for a fill-timing
  design.
- `calc_on_order_fills` and `calc_on_every_tick` require recalculation/realtime
  execution semantics.
- `default_qty_type=strategy.cash` is a plausible future narrow slice, but it
  needs cash-to-quantity rounding and symbol precision rules.
- `margin_short` runtime behavior should wait for short exposure.
- `close_entries_rule` should wait for multi-entry ledgers.

### Slice 3+: Runtime Implementation

Implement only the selected property. Each property must have semantic fixtures,
runtime fixtures where behavior changes, matrix snapshots, docs, release notes,
and `scripts/verify.sh`.

## Compatibility Contract

Until a later slice closes, the supported strategy declaration subset remains the
one recorded in `tests/fixtures/conformance.tsv`. Unsupported declaration
properties must fail before runtime execution, and public strategy JSON must not
grow declaration-setting fields.
