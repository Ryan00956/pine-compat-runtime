# Strategy Internal Gap Audit

Status: planning audit.

This audit compares the current fixture-backed strategy subset with the
internal strategy behavior exposed by TradingView Pine Script strategy
documentation. It is intentionally limited to interpreter, semantic-analysis,
broker-emulator, runtime-output, and host-binding work. It excludes chart UI,
Strategy Tester UI panels, settings dialogs, external alert delivery, real
broker connectivity, and remote market-data services.

This document does not claim new support. Support claims still come from
`tests/fixtures/conformance.tsv`, snapshots, phase audits, and verification
results.

## Sources Reviewed

Repository evidence:

- `tests/fixtures/conformance.tsv`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/SEMANTIC_MODEL.md`
- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-runtime/src/strategy/`

TradingView Pine documentation reviewed on 2026-06-02:

- https://www.tradingview.com/pine-script-docs/concepts/strategies/
- https://www.tradingview.com/pine-script-docs/language/declaration-statements/
- https://www.tradingview.com/pine-script-docs/language/execution-model/
- https://www.tradingview.com/support/solutions/43000628599-strategy-properties/

## Current Internal Baseline

The current runtime can execute a narrow historical, long-only, one-net-position
strategy model.

Implemented and fixture-backed:

- `strategy(...)` declaration with selected metadata, positive const
  `initial_capital`, and fixed default quantity.
- `strategy.entry(id, strategy.long, qty=...)` and default fixed quantity when
  configured.
- `strategy.close(id)` as a full close of the matching long position.
- Public strategy output with `orders`, `trades`, `position`, `equity`, and
  `diagnostics`.
- Read-only state/count variables:
  - `strategy.position_size`
  - `strategy.position_avg_price`
  - `strategy.openprofit`
  - `strategy.netprofit`
  - `strategy.equity`
  - `strategy.closedtrades`
  - `strategy.opentrades`
- `strategy.exit` for:
  - single triggers: `stop`, `limit`, `profit`, `loss`;
  - one-downside/one-upside brackets: `stop + limit`, `stop + profit`,
    `loss + limit`, `loss + profit`;
  - trailing forms: `trail_price + trail_offset`,
    `trail_points + trail_offset`;
  - optional `qty` and `qty_percent`;
  - explicit fixed-`qty` or `qty_percent` multiple reservations across
    single-trigger, bracket, and trailing exits;
  - omitted-quantity replacement and explicit-reservation clearing boundaries.

Current public output remains intentionally smaller than Pine's full strategy
tester model. It does not expose pending orders, reservation ledgers, exit
reasons, bracket legs, trailing state, commission, runup/drawdown, or individual
trade namespace records.

## Gap Scale

- Small: likely one focused slice using the existing broker and public output
  shape.
- Medium: requires new internal state or semantics, but can probably preserve the
  current public runtime schema.
- Large: requires a broader broker model, new state lifetimes, or coordinated
  CLI/Python/WASM public-contract work.
- Foundation: should be designed before multiple later gaps, because many
  features depend on it.

## Priority Correction: Active Entry Attachment, Not Arbitrary Future Binding

The next strategy gap should not be described as "bind an exit to any future
entry with a matching id." Pine's documented behavior is narrower:

- an exit with `from_entry` applies to existing open trades with that entry id;
- exits can be created around active entry orders in the same strategy logic;
- if `from_entry` does not match a current position or active entry order, the
  command creates no exit orders.

Current docs and plans should use a narrower phrase such as active-entry exit
attachment or same-calculation entry/exit attachment. The unsupported boundary
should remain: unmatched missing-entry exits are no-op or diagnostic behavior,
not persistent future reservations.

## Internal Gap Inventory

### 1. Strategy Declaration Properties

Current state: only a small declaration subset is modeled. `initial_capital` is
accepted when positive and const. Fixed default quantity is the only supported
quantity mode.

Missing internal behavior:

- `pyramiding`
- `calc_on_order_fills`
- `calc_on_every_tick`
- `process_orders_on_close`
- `backtest_fill_limits_assumption`
- `default_qty_type=strategy.cash`
- `default_qty_type=strategy.percent_of_equity`
- `currency`
- `slippage`
- `commission_type` and `commission_value`
- `margin_long` and `margin_short`
- `close_entries_rule`
- `risk_free_rate`
- `use_bar_magnifier`
- `fill_orders_on_standard_ohlc`
- strategy-specific alert and order-fill settings that affect runtime output

Gap size: large.

Best first slice: add a declaration-property audit fixture that keeps each
unsupported property explicitly rejected, then choose one low-blast-radius
property. `pyramiding=1` as an accepted no-op alias is low value; real pyramiding
requires a larger open-trade model.

### 2. Broker Execution Timing And Fill Model

Current state: historical execution is one pass per bar. Entries and closes fill
at the current bar close in the current subset. Pending exits fill on later bars
using simple OHLC trigger checks and fixed prices.

Missing internal behavior:

- default next-tick order fill timing;
- active entry orders that exist before they fill;
- same-calculation `strategy.entry` plus `strategy.exit` attachment before the
  entry fill;
- order processing on close;
- recalculation after order fills;
- realtime strategy rollback and repeated tick execution;
- historical intrabar tick assumptions;
- bar magnifier lower-timeframe fills;
- better/worse price behavior for limit and stop orders;
- stricter limit fill verification.

Gap size: foundation.

Best first slice: introduce an internal pending-entry state without changing
public output, then fixture the Pine-compatible case where an exit attaches to a
same-calculation active entry order. Keep arbitrary future unmatched exits out of
scope.

### 3. Entry Orders

Current state: `strategy.entry` supports long market entries only, with explicit
positive quantity or configured fixed default quantity. Repeated entries while
long are ignored under the current no-pyramiding rule.

Missing internal behavior:

- short entries;
- automatic reversal when an opposite entry is placed;
- limit, stop, and stop-limit entry orders;
- pyramiding with multiple open trades in the same direction;
- entry comments and alert-message metadata;
- richer default quantity modes;
- interaction with `strategy.risk.allow_entry_in`.

Gap size: large.

Best first slice: not short/reversal first. Add the active pending-entry model
from gap 2, because stop/limit entries, next-tick fills, and exit attachment all
depend on it.

### 4. Market Close Commands

Current state: `strategy.close(id)` closes the full matching long position at
the current bar close and cancels matching pending exits.

Missing internal behavior:

- `strategy.close_all()`;
- partial `strategy.close(..., qty=...)` and `qty_percent`;
- `immediately`;
- `comment`, `alert_message`, and alert suppression options;
- close behavior across multiple entries and pyramiding;
- close-entry ordering such as FIFO versus entry-specific close rules.

Gap size: medium to large.

Best first slice: `strategy.close_all()` for the current one-net-long model.
This is internally small and should preserve the current public output shape.

### 5. Generic Orders And Cancellation

Current state: `strategy.order` remains unsupported. Stage 6 added the
fixture-backed supported `strategy.cancel(id)` and `strategy.cancel_all()`
subsets for current internal pending entries and exits.

Missing internal behavior:

- `strategy.order()` as a generic long/short order that can open, reduce,
  reverse, or close positions;
- market, limit, stop, and stop-limit order forms;
- cancellation of pending entries and exits by id;
- cancel-all behavior across all pending orders;
- order metadata comments and alert messages.

Gap size: large.

Best first slice: do not start here. Cancellation needs a richer pending-order
book, and `strategy.order` needs short/reversal/netting semantics.

### 6. Exit Orders

Current state: this is the most developed part of the strategy runtime. The
current subset supports stop/limit/profit/loss, the first bracket subset,
trailing stops, partial quantities, percent quantities, explicit multiple
reservations, and omitted-quantity replacement behavior.

Missing internal behavior:

- optional `from_entry` that exits all matching open entries when omitted;
- Pine-compatible behavior where `qty` wins when both `qty` and `qty_percent`
  are supplied;
- exact semantics for overlapping price and tick alternatives in one exit call;
- exit attachment to active entry orders before those entries fill;
- exits across multiple open trades and pyramiding;
- custom OCA names and OCA behavior;
- exit comments and alert-message metadata;
- no-op behavior for invalid `from_entry` ids where Pine does not create exit
  orders.

Gap size: medium.

Best first slice: active-entry exit attachment after the pending-entry foundation
is in place. The current `qty + qty_percent` rejection is intentionally narrower
than Pine and should be changed only with semantic and runtime fixtures proving
that `qty` wins.

### 7. OCA Groups And Reservation Semantics

Current state: explicit `qty` and `qty_percent` reservations exist internally for
supported single-trigger, bracket, and trailing exits. Public output does not
expose the reservation book.

Missing internal behavior:

- `strategy.oca.reduce`, `strategy.oca.cancel`, and `strategy.oca.none`;
- custom `oca_name`;
- OCA behavior for generic orders, not only exits;
- OCA behavior across pyramided entries and mixed order families.

Gap size: large.

Best first slice: defer until generic pending-order state exists. The current
reservation model is a useful base, but it is exit-specific.

### 8. Multiple Entries, Pyramiding, Shorts, And Reversals

Current state: there is one net long position, no short exposure, no reversal,
and no pyramiding.

Missing internal behavior:

- multiple open trades with separate entry ids;
- same-direction pyramiding;
- short positions;
- automatic reversal from long to short or short to long;
- entry-id-specific exits across multiple trades;
- FIFO and configured close ordering;
- net position versus individual trade accounting.

Gap size: foundation and large.

Best first slice: defer until pending-entry timing and trade-ledger design are
settled. This is the largest broker-model gap.

### 9. Position Sizing And Account Model

Current state: explicit positive quantities and fixed default quantities are
supported. Cash, market value, equity, and net profit are calculated for the
current long-only model without costs.

Missing internal behavior:

- cash-based default quantity;
- percent-of-equity default quantity;
- contract/share minimum and rounding behavior;
- currency selection and conversion;
- margin requirements and forced liquidation;
- capital held for open trades;
- account constraints that can prevent fills.

Gap size: large.

Best first slice: percent-of-equity default quantity for long market entries
could be isolated, but it should wait until the account model policy is explicit.

### 10. Costs And Price Adjustments

Current state: no commission, slippage, or stricter limit-fill assumption.

Missing internal behavior:

- percentage commission;
- cash-per-contract commission;
- cash-per-order commission;
- slippage applied to fills;
- limit-order verification in ticks;
- cost fields in trade data and performance variables.

Gap size: medium to large.

Best first slice: flat cash-per-order commission would be the smallest internal
cost model, but it affects cash, profit, trade records, and public output
expectations if costs are exposed.

### 11. Strategy Information Variables

Current state: only position size, average price, open profit, net profit,
equity, closed-trade count, and open-trade count are supported.

Missing internal behavior:

- `strategy.wintrades`, `strategy.losstrades`, `strategy.eventrades`;
- average trade, winning trade, losing trade, runup, drawdown, and percent
  variants;
- max contracts/shares held;
- capital held;
- built-ins whose value depends on costs, margin, or individual trade records.

Gap size: medium.

Best first slice: win/loss/even trade counts for the current long-only closed
trade list. This preserves the current output shape if exposed only as
read-only script variables.

### 12. Individual Trade Namespaces

Current state: `strategy.closedtrades` and `strategy.opentrades` are count
variables. Stage 7 Slice 0 also supports script-visible
`strategy.closedtrades.entry_price()`, `.exit_price()`, `.entry_bar_index()`,
`.exit_bar_index()`, `.entry_time()`, `.exit_time()`, `.commission()`,
`.entry_id()`, `.exit_id()`, `.size()`, and `.profit()` over the current
closed-trade list without public runtime schema expansion. Stage 7 Slice 6
also supports `strategy.opentrades.entry_price()` for the current supported
single open long position. Stage 7 Slice 7 adds
`strategy.opentrades.entry_bar_index()` for that same open position. Stage 7
Slice 8 adds `strategy.opentrades.entry_time()`, and Slice 9 adds
`strategy.opentrades.size()`. Stage 7 Slice 10 adds
`strategy.opentrades.profit()`. Stage 7 Slice 11 adds
`strategy.opentrades.entry_id()`. Stage 7 Slice 12 adds
`strategy.opentrades.commission()`. Stage 7 Slice 13 adds
`strategy.opentrades.max_runup()`. Stage 7 Slice 14 adds
`strategy.opentrades.max_drawdown()`. Other namespace functions are
unsupported.

Missing internal behavior:

- `strategy.closedtrades.*()` fields beyond the supported price/id/bar-index
  and time subset, plus runup and drawdown;
- indexed trade access;
- open-trade and closed-trade records with enough retained metadata.

Gap size: large.

First slice closed: closed-trade `entry_price`, `exit_price`,
`entry_bar_index`, and `exit_bar_index` for the current closed-trade list are
script-variable only. Public JSON, Python, and WASM runtime schema remains
unchanged.

Second slice closed: closed-trade `size` and `profit` follow the same
script-variable-only, zero-based `trade_num` contract without changing public
runtime schema.

Third slice closed: closed-trade `entry_time` and `exit_time` expose the
already-retained timestamps under the same script-variable-only contract.

Fourth slice closed: closed-trade `commission` returns `0.0` under the current
no-commission account model and keeps the same script-variable-only contract.

Fifth slice closed: closed-trade `entry_id` exposes the retained entry id under
the same script-variable-only contract.

Sixth slice closed: closed-trade `exit_id` exposes the retained close or exit id
under the same script-variable-only contract.

### 13. Risk Management

Current state: `strategy.risk.*` is unsupported.

Missing internal behavior:

- entry-direction restrictions;
- max intraday loss, max drawdown, max position size, and similar risk stops;
- risk rules interacting with pending orders and recalculation behavior.

Gap size: large.

Best first slice: defer. Risk rules need a more complete broker state, order
book, and account model.

### 14. Strategy Alerts And Order Metadata

Current state: indicator-style alert events are supported in a narrow runtime
event model. Strategy order-fill alert metadata is not modeled.

Missing internal behavior:

- order-fill alert messages;
- `alert_message` arguments on order commands;
- `disable_alert`;
- strategy-specific placeholder data for order fills;
- alert events tied to broker fills rather than reached alert calls.

Gap size: medium.

Best first slice: store order `alert_message` metadata internally for supported
orders but keep external delivery out of scope.

## Recommended Internal Roadmap

1. Pending-entry timing foundation:
   - add active entry orders;
   - preserve current public output unless an entry actually fills;
   - fixture default next-bar or next-tick timing policy;
   - fixture same-calculation entry/exit attachment.
2. `strategy.close_all()` for the current one-net-long model.
3. Win/loss/even trade count variables for the current closed-trade list.
4. Pine-compatible `qty + qty_percent` handling for `strategy.exit`, where `qty`
   wins, if semantic and runtime fixtures confirm the intended behavior.
5. Entry limit/stop/stop-limit orders after pending-entry timing is stable.
6. `strategy.cancel()` and `strategy.cancel_all()` after a general pending-order
   book exists.
7. Individual trade namespace functions for a small closed-trade subset.
8. Commission/slippage/account-model slices.
9. Pyramiding, shorts, reversals, and multi-entry trade ledgers.
10. Generic `strategy.order()` and full OCA behavior.

## Completion Gates For Any Slice

Each internal strategy slice should include:

- semantic fixtures for accepted and rejected forms;
- broker unit tests for state transitions and accounting;
- runtime fixtures and golden snapshots;
- incremental/runtime interaction tests where state timing matters;
- CLI, Python, and WASM parity tests if public output or host behavior changes;
- synchronized `tests/fixtures/conformance.tsv`, matrix snapshot, docs, and
  release notes;
- a phase audit recording the supported and unsupported boundary;
- `scripts/verify.sh` before closeout.
