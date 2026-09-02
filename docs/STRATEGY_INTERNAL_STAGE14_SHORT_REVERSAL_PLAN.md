# Strategy Internal Stage 14 Short And Reversal Plan

Status: active. Slices 14a-14o closed. Later slices follow
`docs/PURE_INTERNAL_STRATEGY_SHORT_REVERSAL_DESIGN.md` and must not start until
the previous slice is fixture-backed.

Stage 13 closed the long-only multi-entry ledger. Stage 14 is the first
short/reversal program. Short `margin_short` capital held and affordability
landed in Stage 15a.

## Goal

Turn the current long-only broker into a direction-aware internal model, then
add the smallest executable short subset:

1. freeze the current rejection and zero-short reporting boundary;
2. make ledger, pending-entry, and pending-exit books side-aware without
   changing public output;
3. accept one explicit-quantity market short entry without reversal;
4. close that short exposure;
5. add automatic reversal under one deterministic historical-bar rule.

## Non-Goals

- v1-v4 `strategy()` compatibility;
- `process_orders_on_close`, `calc_on_order_fills`, `calc_on_every_tick`, or bar
  magnifier timing;
- `strategy.risk.*`;
- custom OCA;
- public pending-order, reservation, or open-trade JSON;
- short stop-limit `strategy.order()` in the first positive slice (closed in
  14o);
- `margin_short` runtime behavior before a dedicated account slice (capital
  held and affordability landed in Stage 15a);

## Slice Order

### 14a. Boundary lock

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_BOUNDARY_AUDIT.md`.

Keep `strategy.entry(..., strategy.short)` rejected, keep generic short
price-based `strategy.order()` rejected, and keep
`strategy.max_contracts_held_short` at `0.0` through long-only fills and
reduce-only market-short orders.

### 14b. Side-aware internal model

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SIDE_AWARE_LEDGER_AUDIT.md`.

Store an explicit long/short side on open trades, derive signed net position
and side-specific average price, filter long close/exit allocation by
direction, and expose pending-entry/pending-exit trade direction without
changing public `StrategyResult`.

### 14c. Market short entry without reversal

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_MARKET_SHORT_ENTRY_AUDIT.md`.

Accept `strategy.entry(id, strategy.short, qty=...)` only while flat or already
short. Opposite-side entries are no-op until 14e. Short limit/stop entries stay
rejected. Short closes landed in Stage 14d. Short stop/limit `strategy.exit`
landed in Stage 14f.

### 14d. Short close subset

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_CLOSE_AUDIT.md`.

Close the first supported short exposure with `strategy.close` and
`strategy.close_all` and prove realized cover PnL. Short stop/limit
`strategy.exit` landed in Stage 14f.

### 14e. Automatic reversal

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_REVERSAL_AUDIT.md`.

A market `strategy.entry` in the opposite direction first flattens the current
net position at the reverse fill price through the existing close-all path,
then opens the requested quantity on the new side. Public records are two-step:
closed opposite trades plus a new entry order of the requested qty.

### 14f. Short stop/limit exits

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_EXIT_AUDIT.md`.

Single-trigger `strategy.exit` `stop` and `limit` cover matching open or pending
short entries. Short stop triggers when `high >= stop`. Short limit triggers
when `low <= limit` minus the configured verification offset. Cover fills use
short-exit slippage and signed closed-trade quantity.

### 14g. Short profit/loss ticks

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_EXIT_TICKS_AUDIT.md`.

Single-trigger `strategy.exit` `profit` and `loss` ticks cover matching open
short entries. Profit converts to a limit below the short entry price. Loss
converts to a stop above it.

### 14h. Short brackets

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_EXIT_BRACKET_AUDIT.md`.

One-downside/one-upside `stop+limit`, `stop+profit`, `loss+limit`, and
`loss+profit` brackets cover matching open shorts. The stop/loss leg fills on
`high >= stop`; the limit/profit leg fills on `low <= limit` minus verification
offset. Same-bar both-touch prefers the stop/loss leg.

### 14i. Short trailing

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_EXIT_TRAILING_AUDIT.md`.

`trail_price + trail_offset` and `trail_points + trail_offset` cover matching
open shorts. Activation is `low <= activation`; the active stop is
`low + offset` and ratchets downward only. A later bar fills when
`high >= active_stop`. `trail_points` converts from the short entry price as
`entry - ticks * mintick`.

### 14j. Short limit entry

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_ENTRY_LIMIT_AUDIT.md`.

Accept `strategy.entry(id, strategy.short, qty=..., limit=price)` while flat or
already short. Fill on a later historical bar when `high >= limit` plus the
configured verification offset. Do not reverse a net long. Short
stop/stop-limit entries stay rejected.

### 14k. Short stop entry

Status: closed. See `docs/STRATEGY_INTERNAL_STAGE14_SHORT_ENTRY_STOP_AUDIT.md`.

Accept `strategy.entry(id, strategy.short, qty=..., stop=price)` while flat or
already short. Fill on a later historical bar when `low <= stop`. Do not
reverse a net long. Short stop-limit entries stay rejected.

### 14l. Short stop-limit entry

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE14_SHORT_ENTRY_STOP_LIMIT_AUDIT.md`.

Accept `strategy.entry(id, strategy.short, qty=..., stop=price, limit=price)`
while flat or already short. Activate on a later historical bar when
`low <= stop` without filling that bar, then fill at the limit price on a
subsequent historical bar when `high >= limit` plus the configured
verification offset. Do not reverse a net long.

### 14m. Short limit order

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE14_SHORT_ORDER_LIMIT_AUDIT.md`.

Accept `strategy.order(id, strategy.short, qty=..., limit=price)` while flat or
already short. Fill on a later historical bar when `high >= limit` plus the
configured verification offset. Bypass the `strategy.entry()` pyramiding
limit. Do not reverse or reduce a net long. Market `strategy.order` short stays
reduce-only. Short stop/stop-limit orders stay rejected.

### 14n. Short stop order

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE14_SHORT_ORDER_STOP_AUDIT.md`.

Accept `strategy.order(id, strategy.short, qty=..., stop=price)` while flat or
already short. Fill on a later historical bar when `low <= stop`. Bypass the
`strategy.entry()` pyramiding limit. Do not reverse or reduce a net long.
Short stop-limit orders stay rejected.

### 14o. Short stop-limit order

Status: closed. See
`docs/STRATEGY_INTERNAL_STAGE14_SHORT_ORDER_STOP_LIMIT_AUDIT.md`.

Accept `strategy.order(id, strategy.short, qty=..., stop=price, limit=price)`
while flat or already short. Activate on a later historical bar when
`low <= stop` without filling that bar, then fill at the limit price on a
subsequent historical bar when `high >= limit` plus the configured
verification offset. Bypass the `strategy.entry()` pyramiding limit. Do not
reverse or reduce a net long. Market `strategy.order` short stays reduce-only.

## Compatibility Rules

- `tests/fixtures/conformance.tsv` remains the support authority.
- Public strategy JSON, Python dictionaries, and WASM JSON stay on the current
  schema unless a later slice designs a change.
- Unsupported short/reversal forms stay diagnostic-rejected until their runtime
  slice lands.
- Existing long-only fixtures must keep their current serialized outputs.

## Completion Gate

Each slice closes with broker tests, semantic or runtime fixtures where
behavior is user-visible, synchronized docs, and `scripts/verify.sh`.
