# Strategy Internal Stage 15 Short Margin Plan

Status: closed through 15c. Slices 15a-15c follow
`docs/PURE_INTERNAL_STRATEGY_MARGIN_SHORT_ACCOUNT_DESIGN.md`.

Stage 14 closed the short entry, close, reversal, exit, and order families.
Stage 15 is the first short-margin account program.

## Goal

Turn the stored `margin_short` declaration into a runtime account model for the
current short-entry subset:

1. report short `strategy.opentrades.capital_held`;
2. reject unaffordable short exposure increases at fill time;
3. keep long-margin fixtures unchanged;
4. force-liquidate underwater shorts using `bar.high`;
5. expose short `strategy.margin_liquidation_price`.

## Non-Goals

- v1-v4 `strategy()` compatibility;
- symbol precision rounding or currency conversion;
- public pending-order, reservation, or account JSON.

## Slice Order

### 15a. Short capital held and affordability

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE15_MARGIN_SHORT_CAPITAL_HELD_AUDIT.md`.

`strategy.opentrades.capital_held` returns absolute short market value times
`margin_short / 100` while a supported short position is open. Supported short
fills are rejected when equity cannot cover required short margin at the actual
fill price. Short forced liquidation landed in Stage 15b.

### 15b. Short forced liquidation

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE15_MARGIN_SHORT_LIQUIDATION_AUDIT.md`.

Open shorts with explicit active `margin_short` are force-liquidated on a
historical bar when available funds are negative at `bar.high`. Cover quantity
uses the documented four-times-cover algorithm with whole-unit truncation. The
public order direction is `strategy.long`. Short
`strategy.margin_liquidation_price` landed in Stage 15c.

### 15c. Short margin liquidation price

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE15_MARGIN_SHORT_LIQUIDATION_PRICE_AUDIT.md`.

Open shorts with explicit active `margin_short` expose
`strategy.margin_liquidation_price` as the solved price where equity equals
required short margin. The value updates after fills and forced liquidation.

## Compatibility Rules

- `tests/fixtures/conformance.tsv` remains the support authority.
- Public strategy JSON, Python dictionaries, and WASM JSON stay on the current
  schema unless a later slice designs a change.
- Existing long-margin fixtures must keep their current serialized outputs.

## Completion Gate

Each slice closes with broker tests, semantic or runtime fixtures where
behavior is user-visible, synchronized docs, and `scripts/verify.sh`.
