# Conformance

This document defines how compatibility is measured.

The project should make compatibility claims by tested feature, not by broad
language name.

## Fixture Categories

Fixtures should be grouped by behavior:

```text
tests/fixtures/
  syntax/
  sema/
  runtime/
  builtins/
  unsupported/
  regressions/
```

Each fixture should include:

- source script
- expected diagnostics or expected output
- selected Pine version
- fixture ownership or license metadata when not original

## Snapshot Outputs

Runtime snapshots should be normalized JSON:

```json
{
  "schemaVersion": 7,
  "plots": [],
  "plotChars": [],
  "plotShapes": [],
  "plotArrows": [],
  "plotBars": [],
  "plotCandles": [],
  "bgColors": [],
  "barColors": [],
  "hlines": [],
  "fills": [],
  "labels": [],
  "lines": [],
  "lineFills": [],
  "polylines": [],
  "boxes": [],
  "tables": [],
  "alerts": [],
  "diagnostics": []
}
```

The snapshot format should avoid host-specific charting details.

Every machine-readable public output must include top-level `schemaVersion`.
Runtime outputs use `PUBLIC_RUNTIME_SCHEMA_VERSION`; analysis outputs use
`PUBLIC_ANALYSIS_SCHEMA_VERSION`; matrix JSON uses
`PUBLIC_MATRIX_SCHEMA_VERSION`. Runtime output is currently `schemaVersion: 7`
because the top-level `alerts` array is reserved, strategy order-fill alert
payloads are exposed under `strategy.alerts`, table cell snapshots include
host-neutral `textWrap`, linefill snapshots are exposed under `lineFills`, and
polyline creation and lifecycle snapshots are exposed under `polylines`;
analysis and matrix JSON remain `schemaVersion: 2`.
The contracts are separate so runtime-only fields do not force analysis or
matrix schema changes. The text-only CLI `analyze` output is
diagnostic console output and is not part of the machine-readable schema until
a JSON mode is added.

## Built-In Runtime Contract

Built-in compatibility is claimed one fixture-backed subset at a time. The
current timeframe metadata subset exposes `timeframe.period` and
`timeframe.main_period` as the runtime's single chart timeframe string. Main
timeframe declaration overrides and requested-context differences are not
claimed until a separate fixture-backed slice designs that context model.

## Strategy Runtime Contract

Phase G marks `strategy` as partial. The executable subset accepts
`strategy(title, shorttitle, overlay, max_bars_back, initial_capital,
default_qty_type, default_qty_value, commission_type, commission_value,
slippage, backtest_fill_limits_assumption, margin_long, margin_short,
pyramiding, named max_labels_count, named max_boxes_count, named max_lines_count, named max_polylines_count)` where
`initial_capital` must be a positive const numeric value when provided. Phase L
accepts `default_qty_type=strategy.fixed` with positive const numeric
`default_qty_value`; Strategy Internal Stage 12 accepts
`default_qty_type=strategy.cash` with positive const numeric
`default_qty_value`, resolving omitted entry `qty` at placement time as cash
divided by current close under the current no-currency-conversion boundary;
Stage 7 Slice 31 accepts `default_qty_type=strategy.percent_of_equity` with
positive const numeric `default_qty_value`, resolving omitted entry `qty` at
placement time from the current supported equity and current close. Margin
parameters currently support
declaration/IR storage, long-only `strategy.opentrades.capital_held`, and
long-entry affordability checks for the supported entry subset when explicit
active `margin_long` is configured. They also enable the first long-only forced
liquidation subset using `bar.low`, the documented available-funds algorithm,
and whole-unit truncation. They do not enable short margin behavior,
margin-specific public schema expansion, symbol precision rounding, or
`strategy.margin_liquidation_price`. Stage 13 Slice 10 accepts positive integer
const `pyramiding` values for same-direction long `strategy.entry()` market
entries, with the default staying at `1`; short entries, reversals,
`strategy.order()`, same-tick price-based entry exceptions beyond the
fixture-backed long subset, and broader
multi-entry exit/reporting semantics remain outside the supported subset unless
fixture-backed. Stage 13 Slice 80 adds WASM public JSON host-parity coverage for
the base `pyramiding`, `strategy.close(id)`, and `strategy.close_all()` fixtures.
Stage 13 Slice 81 adds matching Python binding public JSON host-parity coverage
for those fixtures. Stage 13 Slice 82 adds WASM public JSON host-parity coverage
for the absolute `strategy.exit(from_entry)`, relative profit
`strategy.exit(from_entry)`, and same-id fan-out fixtures from Slices 14-16.
Stage 13 Slice 83 adds matching Python binding public JSON host-parity coverage
for those fixtures. Stage 13 Slice 84 adds WASM public JSON host-parity coverage
for the bracket and trailing `strategy.exit(from_entry)` fixtures from Slices
17-18. Stage 13 Slice 85 adds matching Python binding public JSON host-parity
coverage for those fixtures. Stage 13 Slice 86 adds WASM public JSON
host-parity coverage for the current same-id omitted `profit`/`loss`
`strategy.exit` fixtures from Slices 59-60. Stage 13 Slice 87 adds matching
Python binding public JSON host-parity coverage for those fixtures. Stage 13
Slice 88 adds WASM public JSON host-parity coverage for the current same-id
omitted `loss+profit` and `stop+profit` bracket fixtures from Slices 61-62.
Stage 13 Slice 89 adds matching Python binding public JSON host-parity coverage
for those fixtures. Stage 13 Slice 90 adds WASM public JSON host-parity
coverage for the current same-id omitted `loss+limit` and `stop+limit` bracket
fixtures from Slices 63-64. Stage 13 Slice 91 adds matching Python binding
public JSON host-parity coverage for those fixtures. Stage 13 Slice 92 adds
WASM public JSON host-parity coverage for the current same-id omitted
`trail_points+trail_offset` and `trail_price+trail_offset` trailing fixtures
from Slices 65-66. Stage 13 Slice 93 adds matching Python binding public JSON
host-parity coverage for those fixtures. Stage 13 Slice 94 adds WASM public
JSON host-parity coverage for the same-id omitted `profit`/`loss`
future-entry persistence fixtures from Slices 67-68. Stage 13 Slice 95 adds
matching Python binding public JSON host-parity coverage for those fixtures.
Stage 13 Slice 96 adds WASM public JSON host-parity coverage for the same-id
omitted `loss+profit` and `stop+profit` bracket future-entry persistence
fixtures from Slices 69-70. Stage 13 Slice 97 adds matching Python binding
public JSON host-parity coverage for those fixtures. Stage 13 Slice 98 adds
WASM public JSON host-parity coverage for the same-id omitted `loss+limit` and
`stop+limit` bracket future-entry persistence fixtures from Slices 71-72. Stage
13 Slice 99 adds matching Python binding public JSON host-parity coverage for
those fixtures. Stage 13 Slice 100 adds WASM public JSON host-parity coverage
for the same-id omitted `trail_price+trail_offset` and
`trail_points+trail_offset` trailing future-entry persistence fixtures from
Slices 73-74. Stage 13 Slice 101 adds matching Python binding public JSON
host-parity coverage for those fixtures.
Stage 13 Slice 75 adds fixture-backed same-tick limit-entry
pyramiding-limit exceptions. Stage 13 Slice 76 adds matching stop-entry
exceptions. Stage 13 Slice 77 adds matching stop-limit-entry exceptions while
preserving the existing activation-bar delay. Stage 13 Slice 78 adds WASM
public JSON host-parity coverage for those same-tick long limit, stop, and
stop-limit entry fixtures. Stage 13 Slice 79 adds matching Python binding
public JSON host-parity coverage for those fixtures. Stage 13 Slice 11 adds
fixture-backed `strategy.close(id)`
matching for a requested pyramided long entry id. Stage 13 Slice 12 adds
fixture-backed `strategy.close_all()` flattening across all open long ledger
entries. Stage 13 Slice 14 adds fixture-backed absolute stop/limit
`strategy.exit` matching by requested open pyramided long entry id. Stage 13
Slice 15 adds fixture-backed single-trigger `profit`/`loss` tick conversion from
the matched open pyramided entry price. Stage 13 Slice 16 adds fixture-backed
same-entry-id `strategy.exit` allocation fan-out into one public exit order and
closed trade per matched open trade. Stage 13 Slice 17 adds fixture-backed
bracket `profit`/`loss` relative leg conversion from the matched open pyramided
entry price. Stage 13 Slice 18 adds fixture-backed trailing `trail_points`
activation conversion from the matched open pyramided entry price. Stage 13
Slice 19 adds fixture-backed omitted-`from_entry` current open-entry absolute
stop/limit all-entry exits. Stage 13 Slice 20 extends that absolute stop/limit
subset so the omitted-`from_entry` exit persists for later open long entries
until the position closes. Stage 13 Slice 21 adds fixture-backed omitted-
`from_entry` current unique-entry-id profit-tick all-entry exits. Stage 13 Slice
22 adds the symmetric fixture-backed loss-tick subset for current unique entry
ids. Stage 13 Slice 23 adds the fixture-backed omitted-`from_entry`
current unique-entry-id `loss+profit` bracket subset; broader multi-entry
`strategy.exit` semantics remain outside this claim. Stage 13 Slice 24 adds the
fixture-backed omitted-`from_entry` current unique-entry-id `stop+profit`
bracket subset. Stage 13 Slice 25 adds the fixture-backed omitted-`from_entry`
current unique-entry-id `loss+limit` bracket subset. Stage 13 Slice 26 adds the
fixture-backed omitted-`from_entry` current all-entry `stop+limit` bracket
subset. Stage 13 Slice 27 adds the fixture-backed omitted-`from_entry` current
all-entry `trail_price+trail_offset` trailing subset. Stage 13 Slice 28 adds the
fixture-backed omitted-`from_entry` current unique-entry-id
`trail_points+trail_offset` trailing subset. Stage 13 Slices 29-34 add
fixture-backed omitted-`from_entry` future-entry persistence for profit-tick,
loss-tick, `loss+profit`, `stop+profit`, `loss+limit`, and `stop+limit` exits.
Stage 13 Slice 35 adds fixture-backed omitted-`from_entry`
`trail_price+trail_offset` future-entry persistence.
Stage 13 Slice 36 adds fixture-backed omitted-`from_entry`
`trail_points+trail_offset` future-entry persistence for unique entry ids.
Stage 13 Slice 37 adds WASM public JSON host-parity coverage for the Slice 36
omitted trail-points persistence fixture without widening the runtime subset.
Stage 13 Slice 38 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 39 adds WASM public JSON host-parity coverage for the Slice 35
omitted trail-price persistence fixture without widening the runtime subset.
Stage 13 Slice 40 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 41 adds WASM public JSON host-parity coverage for the Slice 29
omitted profit persistence fixture without widening the runtime subset.
Stage 13 Slice 42 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 43 adds WASM public JSON host-parity coverage for the Slice 30
omitted loss persistence fixture without widening the runtime subset.
Stage 13 Slice 44 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 45 adds WASM public JSON host-parity coverage for the Slice 31
omitted loss+profit bracket persistence fixture without widening the runtime
subset.
Stage 13 Slice 46 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 47 adds WASM public JSON host-parity coverage for the Slice 32
omitted stop+profit bracket persistence fixture without widening the runtime
subset.
Stage 13 Slice 48 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 49 adds WASM public JSON host-parity coverage for the Slice 33
omitted loss+limit bracket persistence fixture without widening the runtime
subset.
Stage 13 Slice 50 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 51 adds WASM public JSON host-parity coverage for the Slice 34
omitted stop+limit bracket persistence fixture without widening the runtime
subset.
Stage 13 Slice 52 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 53 adds WASM public JSON host-parity coverage for the Slice 19
omitted current all-entry absolute exit fixture without widening the runtime
subset.
Stage 13 Slice 54 adds the matching Python binding public JSON host-parity
coverage for the same fixture without widening the runtime subset.
Stage 13 Slice 55 adds internal broker open-trade keys for future per-open-trade
exit identity work without widening the runtime subset.
Stage 13 Slice 56 adds internal key-scoped ledger exit allocation for future
per-open-trade exit identity work without widening the runtime subset.
Stage 13 Slice 57 adds internal pending-exit trade-key scoping for future
per-open-trade exit identity work without widening the runtime subset.
Stage 13 Slice 58 adds internal key binding for existing unique-entry-id
omitted relative exit expansion without widening the runtime subset.
Stage 13 Slice 59 adds fixture-backed current same-entry-id omitted
`from_entry` profit-tick exits, using each open trade's entry price.
Stage 13 Slice 60 adds matching fixture-backed current same-entry-id omitted
`from_entry` loss-tick exits. Stage 13 Slice 61 adds matching fixture-backed
current same-entry-id omitted `from_entry` `loss+profit` bracket exits.
Stage 13 Slice 62 adds matching fixture-backed current same-entry-id omitted
`from_entry` `stop+profit` bracket exits. Stage 13 Slice 63 adds matching
fixture-backed current same-entry-id omitted `from_entry` `loss+limit` bracket
exits. Stage 13 Slice 64 adds explicit fixture-backed current same-entry-id
omitted `from_entry` `stop+limit` bracket coverage. Stage 13 Slice 65 adds
matching fixture-backed current same-entry-id omitted `from_entry`
`trail_points+trail_offset` trailing exits. Stage 13 Slice 66 adds explicit
fixture-backed current same-entry-id omitted `from_entry`
`trail_price+trail_offset` trailing coverage. Stage 13 Slice 67 adds
fixture-backed same-entry-id omitted `from_entry` profit-tick future-entry
persistence. Stage 13 Slice 68 adds matching loss-tick future-entry
persistence. Stage 13 Slice 69 adds matching `loss+profit` bracket
future-entry persistence. Stage 13 Slice 70 adds matching `stop+profit`
bracket future-entry persistence. Stage 13 Slice 71 adds matching `loss+limit`
bracket future-entry persistence. Stage 13 Slice 72 adds explicit
`stop+limit` bracket future-entry persistence. Stage 13 Slice 73 adds explicit
`trail_price+trail_offset` future-entry persistence. Stage 13 Slice 74 adds
matching `trail_points+trail_offset` future-entry persistence.
Stage 7 Slice 17 accepts
`commission_type=strategy.commission.cash_per_contract`, and Stage 7 Slice 18
accepts `commission_type=strategy.commission.cash_per_order`, both with finite
non-negative const numeric `commission_value`. Stage 7 Slice 21 accepts
`commission_type=strategy.commission.percent` and debits
`qty * fill_price * commission_value / 100` on each supported entry and exit
fill. Stage 7 Slice 19 accepts finite non-negative integer const `slippage`
ticks using the fixed `syminfo.mintick` subset. Stage 7 Slice 20 accepts finite
non-negative integer const
`backtest_fill_limits_assumption` ticks for supported limit-order
verification. Contracts, commission modes outside the three listed above, richer
fill models, currency conversion, symbol precision rounding, and lot-step
constraints remain unsupported. Strategy
mode output includes `orders`, `trades`, `position`, `equity`, and
`diagnostics`. Equity snapshots are emitted once per historical bar with
`barIndex`, `cash`, `marketValue`, `equity`, and `netProfit`, using current
bar-close mark-to-market accounting for the long-only order subset and applying
supported commission debits and slippage-adjusted fill prices when configured.
Supported fixed-tick limit verification can delay supported long limit entry and
limit/profit exit fills while preserving the original limit fill price.
The fixed symbol metadata subset uses a default `NASDAQ:AAPL` chart identity and
now includes `syminfo.main_tickerid` plus `syminfo.mincontract` alongside the
existing ticker, exchange, currency, session, mintick, pointvalue, minmove, and
pricescale fields. Host-configurable symbol metadata remains outside the current
runtime contract.
Currency conversion, symbol precision rounding, lot-step constraints, pyramiding,
short orders,
`strategy.exit` same-side/3+ trigger/invalid trailing variants, reservation
behavior outside the explicit fixed-`qty` or `qty_percent`
single-trigger/bracket/trailing subset, omitted-quantity multiple
pending exits, `strategy.order`, realtime strategy handoff, and most strategy
reporting variables remain outside the supported matrix.

Phase L adds the first read-only strategy state variables for historical
strategy-mode scripts. `strategy.position_size` is a series float that is `0`
when flat and positive for the current long-only position. `strategy.position_avg_price`
is a series float that is `na` when flat and the current average entry price
when long. `strategy.openprofit` is unrealized profit for the current long
position marked to the current close and is `0` when flat. `strategy.netprofit`
is cumulative realized closed-trade profit only, excluding any current open
profit. Stage 7 Slice 22 adds `strategy.grossprofit` as cumulative positive
realized closed-trade profit only, excluding losing, flat, and current open
trades. Stage 7 Slice 23 adds `strategy.grossloss` as cumulative realized
closed-trade loss as a positive value, excluding winning, flat, and current
open trades. Stage 7 Slice 32 adds `strategy.netprofit_percent`,
`strategy.grossprofit_percent`, and `strategy.grossloss_percent` by dividing the
corresponding realized amount by `initial_capital` and multiplying by 100.
Stage 7 Slice 24 adds `strategy.avg_trade` as average realized
profit/loss per closed trade, returning `na` until at least one trade is
closed. Stage 7 Slice 25 adds `strategy.avg_winning_trade` as average realized
profit among winning closed trades only, returning `na` until at least one
winning trade exists. Stage 7 Slice 33 adds `strategy.avg_trade_percent`,
`strategy.avg_winning_trade_percent`, and
`strategy.avg_losing_trade_percent` as averages of per-closed-trade percentage
profit/loss values, using each trade's entry price times quantity as the
denominator and returning `na` until the matching trade set exists.
winning trade is closed. Stage 7 Slice 26 adds `strategy.avg_losing_trade` as
average realized loss among losing closed trades only as a positive value,
returning `na` until at least one losing trade is closed. Stage 7 Slice 27 adds
`strategy.max_drawdown` as the maximum intrabar equity drawdown amount
over the current supported trading interval, using the supported entry equity,
the maximum equity before that entry, and the lowest low reached while the
supported position is open. Stage 7 Slice 28 adds `strategy.max_runup` as the maximum intrabar
equity run-up amount over the current supported long-only trading interval,
using the supported entry equity, the minimum equity before that entry, and the
highest high reached while the supported position is open. Stage 7 Slice 30
adds `strategy.max_runup_percent` and `strategy.max_drawdown_percent` by
dividing the supported run-up or drawdown amount by entry price times current
supported position quantity and multiplying by 100. `strategy.equity` is
cash plus current market value; without configured
commission this is equivalent to `initial_capital + strategy.netprofit +
strategy.openprofit` in the current subset, and with supported commission it
also reflects entry commission debits on open positions.
Supported market `strategy.entry`
calls create an internal pending entry and fill on the next historical bar open.
Supported long limit entries fill at the limit price before script statements
on a later historical bar when `low <= limit`. Supported long stop entries fill
at the stop price before script statements on a later historical bar when
`high >= stop`. Supported long stop-limit entries activate before script
statements on a later historical bar when `high >= stop`, do not fill on that
activation bar, and fill at the limit price before script statements on a later
historical bar when `low <= limit`. These variables reflect filled entries
before script statements on the fill bar, not on the creation or activation bar.
When explicit active `margin_long` is configured, these supported long entry
fills are rejected at the actual fill price if simulated equity cannot cover
the required margin. Rejected fills emit a strategy diagnostic, produce no
public order/position/trade event, remove the triggered pending entry, and clear
attached pending exits for that entry id.
Supported same-calculation absolute `strategy.exit` attachment may target an
active pending entry id. The attachment remains internal while the entry is
pending and can fill through the existing `strategy.exit` public order/trade
shape after the matching entry fills. Supported explicit-`from_entry` exit calls
with no matching open entry or active pending entry are no-ops and do not create
exit orders. Entry-relative pending-entry exits using `profit`, `loss`, or
`trail_points` remain unsupported.
Supported `strategy.close` and
`strategy.close_all` calls still update immediately for later statements on the
same bar. They behave like read-only series floats in supported expression
contexts, including branches, switches, loops, pure UDF arguments, and constant
history references. They do not change the public runtime JSON shape because
scripts observe them through ordinary outputs such as `plot`.

Phase O adds the first narrow strategy reporting count variables for
historical strategy-mode scripts. `strategy.closedtrades` is a read-only
series int count of closed trades recorded by the current broker state.
Stage 3 adds `strategy.wintrades`, `strategy.losstrades`, and
`strategy.eventrades` as read-only series int counts of closed trades with
positive, negative, and zero realized profit.
`strategy.opentrades` is a read-only series int count of open trades in the
current long-only broker. It is `0` when flat, `1` for the default
no-pyramiding behavior, and can rise to the accepted positive `pyramiding`
limit for fixture-backed same-direction long market entries. Supported market
`strategy.entry` calls fill on the next historical bar open and update both
counts before script statements on that fill bar. Supported `strategy.close`
and `strategy.close_all` calls update both counts immediately for later
statements on the same bar. Pending `strategy.exit` fills are still evaluated
after script statements on the bar, so script reads see the count changes on
the next bar. Stage 7 Slice 0 adds
`strategy.closedtrades.entry_price(trade_num)`,
`strategy.closedtrades.exit_price(trade_num)`,
`strategy.closedtrades.entry_bar_index(trade_num)`, and
`strategy.closedtrades.exit_bar_index(trade_num)` as script-visible
strategy-mode field functions over the current closed-trade list. Stage 7 Slice
1 adds `strategy.closedtrades.size(trade_num)` and
`strategy.closedtrades.profit(trade_num)` under the same contract. Stage 7
Slice 2 adds `strategy.closedtrades.entry_time(trade_num)` and
`strategy.closedtrades.exit_time(trade_num)`. Stage 7 Slice 3 adds
`strategy.closedtrades.commission(trade_num)`, returning `0.0` without
configured commission and supported entry-plus-exit commission when configured.
Stage 7 Slice 4 adds
`strategy.closedtrades.entry_id(trade_num)`, returning the retained entry id.
Stage 7 Slice 5 adds `strategy.closedtrades.exit_id(trade_num)`, returning the
retained close or exit id. Stage 7 Slice 6 adds
`strategy.closedtrades.entry_comment(trade_num)` and
`strategy.closedtrades.exit_comment(trade_num)`, returning stored entry and
exit comments for commented fixture-backed trades without expanding public
runtime JSON. Stage 7 Slice 7 adds
`strategy.opentrades.entry_price(trade_num)`, returning the current supported
long position's entry price for `trade_num == 0`. Stage 7 Slice 8 adds
`strategy.opentrades.entry_bar_index(trade_num)`, returning the current
supported long position's entry fill bar for `trade_num == 0`. Stage 7 Slice 9
adds `strategy.opentrades.entry_time(trade_num)`, returning the current
supported long position's entry fill timestamp for `trade_num == 0`. Stage 7
Slice 10 adds `strategy.opentrades.size(trade_num)`, returning the current
supported long position size for `trade_num == 0`. Stage 7 Slice 11 adds
`strategy.opentrades.profit(trade_num)`, returning the current close-based
floating profit for that same supported open position. Stage 7 Slice 12 adds
`strategy.opentrades.entry_id(trade_num)`, returning the retained entry id for
that same supported open position. Stage 7 Slice 13 adds
`strategy.opentrades.commission(trade_num)`, returning `0.0` without configured
commission and the current open supported entry commission when configured.
Stage 7 Slice 14
adds `strategy.opentrades.max_runup(trade_num)`, returning the largest
high-based favorable excursion seen so far for that current supported open
position. Stage 7 Slice 15 adds
`strategy.opentrades.max_drawdown(trade_num)`, returning the largest low-based
adverse excursion seen so far for that current supported open position. Stage
7 Slice 16 adds `strategy.opentrades.entry_comment(trade_num)`, returning the
stored entry comment for the current commented fixture-backed open trade without
expanding public runtime JSON. Stage 7 Slice 35 adds
`strategy.opentrades.capital_held` as a read-only variable.
In the no-margin subset it returns `na`; Stage 7 Margin Slice M2 returns
current open long market value times `margin_long / 100` when explicit active
`margin_long` is configured. Strategy Internal Margin Slice M5 adds a
long-only forced-liquidation subset: historical checks use `bar.low` before
script statements, apply the documented available-funds and four-times-cover
algorithm with temporary whole-unit truncation, and emit only existing
order/trade/position/equity output fields. Stage 7 Slice 15 adds
`strategy.closedtrades.max_runup(trade_num)`, returning the
largest high-based favorable excursion retained for the closed trade quantity.
Stage 7 Slice 16 adds `strategy.closedtrades.max_drawdown(trade_num)`,
returning the largest low-based adverse excursion retained for the closed trade
quantity. Stage 7 Slice 17 adds cash-per-contract commission accounting for
supported entries and exits without adding public schema fields. Stage 7 Slice
18 adds cash-per-order commission accounting under the same public contract.
Stage 7 Slice 19 adds fixed-tick slippage for supported long entry, close, and
exit fill prices without changing trigger conditions or public schema.
Stage 7 Slice 20 adds fixed-tick limit-order verification for supported long
limit entry and supported long limit/profit exit fills while preserving the
original limit fill price. Stage 7 Slice 21 adds percent commission accounting
for supported entry/exit fills under the same public contract. Stage 7 Slice 22
adds `strategy.grossprofit` as a script-visible read-only series float summing
only positive realized closed-trade profit. Stage 7 Slice 23 adds
`strategy.grossloss` as a script-visible read-only series float summing
realized closed-trade losses as positive values. Stage 7 Slice 24 adds
`strategy.avg_trade` as a script-visible read-only series float for average
realized profit/loss per closed trade. Stage 7 Slice 25 adds
`strategy.avg_winning_trade` as a script-visible read-only series float for
average realized profit among winning closed trades only. Stage 7 Slice 26
adds `strategy.avg_losing_trade` as a script-visible read-only series float for
average realized loss among losing closed trades only as a positive value.
Stage 7 Slice 27 adds `strategy.max_drawdown` as a script-visible read-only
series float for maximum intrabar equity drawdown amount. Stage 7 Slice
28 adds `strategy.max_runup` as a script-visible read-only series float for
maximum intrabar equity run-up amount. Stage 7 Slice 30 adds
`strategy.max_runup_percent` and `strategy.max_drawdown_percent` as
script-visible read-only series floats for the corresponding intrabar
percentage values.
`trade_num` is zero-based and integer-only; no matching trade, a negative
index, an out-of-range index, or a non-integer argument returns `na`. Public
open-trade records, open-trade namespace functions outside `entry_price`,
`entry_id`, `entry_bar_index`, `entry_time`, `size`, `profit`, `commission`,
`max_runup`, and `max_drawdown`, closed-trade namespace functions outside this
subset, rich reporting metrics, and public output schema changes remain out of
scope.

Phase M adds the first executable `strategy.exit` subset:
`strategy.exit(id, from_entry, stop=price)` and
`strategy.exit(id, from_entry, limit=price)` for full-position exits from the
current one-net-long broker. Accepted exits create or replace one internal
pending exit for the matching entry, do not trigger on the creation or
replacement bar, and fill on a later historical bar when `low <= stop` or
`high >= limit`. The fill uses the configured exit price and is represented by
the existing strategy output fields. No public pending-order, partial-fill, or
exit-reason fields are added.

Phase N adds the first `strategy.exit` tick-distance helpers:
`strategy.exit(id, from_entry, profit=ticks)` and
`strategy.exit(id, from_entry, loss=ticks)`. The current subset accepts one
trigger family per call. Profit exits convert to a pending limit at
`strategy.position_avg_price + ticks * syminfo.mintick`; loss exits convert to
a pending stop at `strategy.position_avg_price - ticks * syminfo.mintick`.
Ticks must evaluate to a finite positive number, and the implementation uses
the same fixed default `syminfo.mintick` subset as `math.round_to_mintick`.
Converted exits reuse the Phase M pending-exit lifecycle and public strategy
output contract.

Phase R adds the first `strategy.exit` bracket subset. Supported brackets have
exactly one downside leg plus one upside leg for the current long-only broker:
`stop + limit`, `stop + profit`, `loss + limit`, and `loss + profit`. A bracket
is one broker-owned pending full-position exit. Filling either leg cancels the
other leg, emits exactly one `strategy.exit` order event, and records the closed
trade under the source entry id. If both legs are touched on the same later
eligible historical bar, the downside stop/loss leg fills first. Public runtime
JSON, Python dictionaries, and WASM JSON keep the existing strategy result
shape and runtime `schemaVersion: 3`. Same-side pairs `stop + loss` and
`limit + profit`, 3+ trigger calls, partial exits, and arbitrary future binding
for unmatched `from_entry` ids remain unsupported.

Phase S adds the first `strategy.exit` trailing-stop subset. Supported trailing
forms are exactly `trail_price + trail_offset` and
`trail_points + trail_offset` for the current long-only broker, with no fixed
`stop`, `limit`, `profit`, or `loss` arguments in the same call. `trail_price`
is the activation price. `trail_points` converts once from the current average
entry price, or from the matched open pyramided entry price when that
fixture-backed `from_entry` subset applies, using the fixed default
`syminfo.mintick`; `trail_offset` converts once to a fixed price distance. A
trailing exit starts inactive, is not
eligible on its creation or replacement bar, activates on a later bar when
`high >= activation`, never fills on the activation bar, then fills on a later
bar when `low <= active_stop` before any same-bar ratchet. When not filled, the
active stop ratchets upward to
`max(active_stop, high - offset_distance)`. The public output stays on the
existing strategy result shape and runtime `schemaVersion: 3`; there are no
public trailing-state, pending-order, or exit-reason fields. Invalid trailing
combinations remain fixture-backed unsupported.

Phase U adds fixed partial quantities to the supported `strategy.exit` trigger
shapes. `qty` is accepted on the single-trigger stop, limit, profit, and loss
forms, on the supported one-downside/one-upside bracket forms, and on the
supported trailing forms. `qty` evaluates once at placement time, must be
finite and positive, and stores an absolute requested close quantity on the
single pending exit. If omitted, the exit keeps the previous full-position
behavior. On fill, the closed quantity is `min(qty, current position size)`:
partial fills emit one existing `strategy.exit` order event and one closed
trade for the filled quantity, leave the remaining long position open at the
same average price, and clear the filled pending exit. Quantities at or above
the current position size close the full position. The public output shape and
runtime `schemaVersion: 3` are unchanged. Phase U did not add `qty_percent`,
multiple pending exits, quantity reservation, or missing-entry pre-placement.

Phase V adds percent partial quantities to the same supported `strategy.exit`
trigger shapes. `qty_percent` evaluates once at placement time, must be finite
and positive, and resolves against the current open position size to an absolute
requested close quantity. Fills use `min(resolved_qty, current position size)`,
so `qty_percent > 100` is allowed but closes no more than the current position.
Partial fills emit the same existing order/trade fields with absolute `qty`,
leave any remaining long position open at the same average price, and do not add
public pending, remaining, percent, or schema fields. Phase W adds the first
multiple-pending reservation subset for explicit fixed `qty` or `qty_percent`
single-trigger exits on the current matching long entry. Phase X extends that
reservation subset to explicit fixed `qty` or `qty_percent` one-downside plus
one-upside bracket exits. Phase Y extends the same reservation model to the
supported trailing forms. Reservations are resolved at placement time, clamped
to currently unreserved position quantity, and same-identity calls replace the
previous reservation. Single-trigger, bracket, and trailing reservations can
share the same pool. Same-side touched exits fill in placement order. If
downside stop/loss/trailing and upside limit/profit candidates are both touched
on one eligible bar, downside candidates fill on that bar in placement order and
opposite-side candidates remain pending if a long position remains. When both
legs of one bracket are touched, that bracket contributes its downside
candidate. Inactive trailing reservations activate on a later eligible bar and
never fill on the activation bar; active trailing reservations fill as downside
candidates before same-bar ratchets and otherwise ratchet upward only. Phase Z
closes the omitted-quantity boundary: omitted `qty` and omitted
`qty_percent` keep full-position one-effective-pending behavior across
supported single-trigger, bracket, and trailing forms, and a later omitted
full-position exit clears earlier explicit reservations for the current
matching long entry. `qty` and `qty_percent` in the same call remain supported
on those same trigger shapes with fixed `qty` determining the reserved or
filled quantity. Stage 9 supports same-calculation active-entry single-trigger
attachment for absolute `stop`, `limit`, and `trail_price` plus entry-relative
`profit`, `loss`, and `trail_points + trail_offset` against a matching active
pending long entry. Active-entry relative brackets remain a Stage 10 design
target, while multiple pending exits outside the explicit fixed-`qty` or
`qty_percent` single-trigger/bracket/trailing reservation subset remain
unsupported, including omitted-quantity multiple reservations, reservation
behavior outside that subset, and arbitrary future binding for unmatched
`from_entry` ids. Supported unmatched explicit-`from_entry` exit placements are
no-ops without public exit orders or strategy diagnostics.

The closed Phase L boundary is summarized in `docs/PHASE_L_AUDIT.md`. The
closed Phase M boundary is summarized in `docs/PHASE_M_AUDIT.md`. The closed
Phase N boundary is summarized in `docs/PHASE_N_AUDIT.md`. The closed Phase O
count-variable boundary is summarized in `docs/PHASE_O_AUDIT.md`. Phase P's
structural broker split is summarized in `docs/PHASE_P_AUDIT.md`. Phase Q's
bracket design gate is recorded in `docs/PHASE_Q_AUDIT.md`. Phase R's
fixture-backed bracket implementation is summarized in `docs/PHASE_R_AUDIT.md`.
Phase U's fixed quantity subset is summarized in `docs/PHASE_U_AUDIT.md`. Phase
V's percent quantity subset is summarized in `docs/PHASE_V_AUDIT.md`. Phase X's
bracket reservation subset is summarized in `docs/PHASE_X_AUDIT.md`. Phase Y's
trailing reservation subset is summarized in `docs/PHASE_Y_AUDIT.md`. Phase Z's
omitted-quantity boundary is summarized in `docs/PHASE_Z_AUDIT.md`. Stage 9's
active-entry single-trigger relative exit closeout is summarized in
`docs/STRATEGY_INTERNAL_STAGE9_ENTRY_RELATIVE_EXIT_AUDIT.md`. Stage 10's
active-entry relative bracket design gate is recorded in
`docs/STRATEGY_INTERNAL_STAGE10_ACTIVE_ENTRY_BRACKET_PLAN.md`.

## Source Graph Host Contract

Phase J adds a host-neutral source graph scaffold and a narrow executable
import subset. `tests/fixtures/conformance.tsv` marks `import` as `partial`
only for host-provided exact-key imports with aliases, exported const
expressions, and pure exported functions. Library declarations, imported UDTs,
imported methods, re-exports, remote lookup, and side-effecting exported
functions remain outside the supported matrix.

Local user-defined types are partial. The supported subset is limited to
top-level `type` declarations with scalar int/float/bool/string/color fields,
`Type.new(...)` construction, field reads on local values, ordinary variables,
local for-expression constructor results, `var` persistence, and UDF parameter
passthrough/returns through positional or named arguments with direct returns,
block-local aliases, or nested passthrough calls, plus UDF construction/returns,
directly, through nested pure
constructor-helper UDF calls, or through same-local-UDT ternary, switch, final
if/else constructor branches, or final for bodies, from local UDT parameter
scalar fields, scalar fields read through block-local UDT aliases of those
parameters, block-local scalar aliases of those fields, inferred scalar
parameters, or block-local scalar aliases of those scalar parameters using
positional or named constructor field arguments.
Local scalar fields can be reassigned outside UDF/method bodies, including in
branch and `for` loop bodies. Field mutation inside UDFs or methods, `varip`,
history references on UDT values, UDT fields, UDT arrays, and imported UDTs
remain outside the supported matrix.
User-defined methods are partial for pure methods on local UDT receivers with
scalar or local UDT parameters and direct UDT passthrough returns, block-local
receiver or local UDT parameter alias passthrough returns, final if/else or
final for local UDT alias passthrough returns, nested-method UDT parameter
passthrough returns, plus local UDT constructor returns, directly, through
nested pure constructor-helper UDF calls, or through same-local-UDT ternary,
switch, final if/else constructor branches, or final for bodies, from receiver
or local UDT parameter scalar fields, scalar fields read through block-local
receiver or local UDT parameter aliases, block-local scalar aliases of those
fields, inferred scalar parameters, or block-local scalar aliases of those
parameters using positional or named constructor field arguments. The receiver
is passed as the first internal parameter. Returned receiver values,
block-local receiver aliases, final if/else or final for local UDT aliases,
local UDT parameter values, block-local local UDT parameter aliases, or
constructed local UDT values may be assigned and field-read at the callsite.
Side effects,
recursion, unknown receiver types, imported methods, mismatched UDT parameter
identity, and unsupported parameter families remain outside the supported
matrix.
Phase J Slice 9 deliberately keeps imported UDT identity and imported methods
as a maintenance tail: exported constants/functions are source-graph scoped,
but UDT type identity and method tables are local to the root source for now.
The closed Phase J boundary and maintenance tails are summarized in
`docs/PHASE_J_AUDIT.md`.

Hosts may pass library source text into semantic analysis as future graph input:

- CLI accepts repeated `--library-source KEY=path.pine` options for `analyze`
  and `run`. The CLI owns filesystem reads and passes source text to core.
- Python accepts `library_sources={"KEY": "source text"}` on `compile_script`,
  `analyze_script`, and `run_script`.
- WASM accepts deterministic JSON library source maps on the
  `*WithLibraries` entry points and routes them through the same shared
  `AnalysisInput` path.

Core crates must not perform filesystem, network, clock, or host registry I/O
for library resolution. Library source keys are deterministic host-provided
identifiers: empty keys, keys containing whitespace/control characters, and
duplicate keys are rejected before analysis. Cache keys include root source
name/text and every host-provided library key/name/text so future import graph
use cannot reuse stale analysis.

CLI and WASM runtime JSON must be generated through the shared runtime contract
helper so field names and nesting cannot drift. Python returns native
dictionaries, so its binding tests assert the same top-level runtime keys and
representative nested output families such as `plotShapes` and `plotCandles`.
The Phase E drawing-object scaffold adds `labels`, `lines`, `boxes`, and
`tables` as top-level runtime keys in `schemaVersion: 2`. The executable label
subset covers `label.new` creation snapshots with bar-index or bar-time x
locations, price/abovebar/belowbar y locations, official label styles, and
host-neutral text metadata, selected `label.set_*` mutators including
fixture-backed `label.set_x`, `label.set_y`, `label.set_xy`, `label.set_text`,
and `label.set_size` mutations from ordinary and independent while-loop
control-flow blocks, fixture-backed `label.set_color`, `label.set_textcolor`,
`label.set_style`, `label.set_tooltip`, `label.set_textalign`,
`label.set_text_font_family`, and `label.set_text_formatting` mutations from
ordinary and independent while-loop control-flow blocks,
fixture-backed x-location snapshot mutation for `label.set_xloc`, including
ordinary and independent while-loop control-flow mutation coverage, and
y-location snapshot mutation for `label.set_yloc`, including ordinary and
independent while-loop control-flow mutation coverage, text-alignment snapshot
mutation for `label.set_textalign`, text font-family snapshot mutation for
`label.set_text_font_family`, text-formatting snapshot mutation for
`label.set_text_formatting`, `label.delete` deletion snapshots, including
ordinary and independent while-loop control-flow deletion coverage,
fixture-backed cloning with `label.copy`, including ordinary and independent
while-loop control-flow cloning coverage, and the fixture-backed `label.get_x`,
`label.get_y`, and
`label.get_text` getters over the latest existing label snapshot, including
ordinary and independent while-loop control-flow read coverage for all three
getters, plus `label.all` existing-label id reads, including ordinary and
independent while-loop control-flow read coverage, with default 50/named 1-500
`max_labels_count` oldest-active label eviction before new creation. The
executable line subset covers `line.new` x1/y1/x2/y2 creation with optional
xloc, extend, color, style, and width initialization for existing host-neutral
snapshot fields when `xloc` is omitted, `xloc.bar_index`, or `xloc.bar_time`,
plus the `chart.point` first_point/second_point overload using point indexes or
times according to `xloc`. It also covers selected
endpoint/color/width/style/extend mutators, `line.set_first_point` and
`line.set_second_point` using point indexes or times according to each line's
current `xloc`, and `line.set_xloc` for the `xloc.bar_index`/`xloc.bar_time`
subset that updates x1, x2, and xloc snapshots, all including ordinary
and independent while-loop control-flow mutation coverage, `line.delete`
deletion snapshots, including ordinary and independent while-loop control-flow
deletion coverage,
fixture-backed cloning with `line.copy`, including ordinary and independent
while-loop control-flow cloning coverage, `line.all` reads from ordinary and
independent while-loop control-flow blocks, fixture-backed `line.get_x1`,
`line.get_y1`, `line.get_x2`, and `line.get_y2` reads from ordinary and
independent while-loop control-flow blocks, and fixture-backed
`line.get_price` getter over the latest existing line snapshot, including
ordinary and independent while-loop control-flow reads, with sparse snapshots
and default 50/named 1-500 `max_lines_count` eviction that appends deletion
snapshots to the oldest active line before creating new ones.
`line.get_price` uses bar-index x1/y1/x2/y2 interpolation and extrapolation and
returns `na` for `na`, deleted, vertical, nonnumeric, or time-coordinate lines;
timestamp interpolation remains unsupported. The executable box subset covers
`box.new` left/top/right/bottom creation with optional xloc, background,
border, extend, text, text-color, text-size, horizontal-alignment,
vertical-alignment, text-wrap, font-family, and text-formatting initialization
for existing host-neutral snapshot fields when `xloc` is omitted,
`xloc.bar_index`, or `xloc.bar_time`; chart-point overloads remain unsupported.
It also covers selected geometry mutators from ordinary and independent
while-loop control-flow blocks, selected background/border/extend mutators from
ordinary and independent while-loop control-flow blocks, selected
text/text-color/text-size/horizontal-alignment/vertical-alignment/text-wrap/
font-family/text-formatting mutators from ordinary and independent while-loop
control-flow blocks, and `box.set_top_left_point` plus
`box.set_bottom_right_point` using point indexes or times according to each
box's current `xloc`,
`box.set_xloc` for the `xloc.bar_index`/`xloc.bar_time` subset that updates
left, right, and xloc snapshots from ordinary and independent while-loop
control-flow blocks,
`box.delete` from ordinary and independent while-loop control-flow blocks, and
fixture-backed cloning with `box.copy` over the latest existing box
snapshot from ordinary and independent while-loop control-flow blocks, `box.all`
reads from ordinary and independent while-loop control-flow blocks after deletion, fixture-backed
`box.get_left`, `box.get_right`, `box.get_top`, and `box.get_bottom` reads from
ordinary and independent while-loop control-flow blocks after mutation, with
sparse snapshots plus default 50/named 1-500 `max_boxes_count` oldest-active
box eviction before new creation.
The executable table subset covers
`table.new` position/dimension creation with optional `bgcolor`,
`frame_color`, `frame_width`, `border_color`, and `border_width` initialization
plus `table.cell` text/background/text-color/tooltip/font-family/text-formatting
cell writes,
`table.set_position` final-position mutations, including ordinary and
independent while-loop control-flow mutation coverage,
`table.set_bgcolor` final background-color mutations, including ordinary and
independent while-loop control-flow mutation coverage,
`table.set_frame_color` final frame-color mutations, including ordinary and
independent while-loop control-flow mutation coverage,
`table.set_frame_width` final frame-width mutations, including ordinary and
independent while-loop control-flow mutation coverage,
`table.set_border_color` final border-color mutations, including ordinary and
independent while-loop control-flow mutation coverage,
`table.set_border_width` final border-width mutations, including ordinary and
independent while-loop control-flow mutation coverage, `table.delete` deletion
snapshots, including ordinary and independent while-loop control-flow deletion
coverage, `table.clear` inclusive rectangular
cell-content removal snapshots, including ordinary and independent while-loop
control-flow clearing coverage,
`table.merge_cells` inclusive merged-cell rectangle snapshots, including
ordinary and independent while-loop control-flow merge coverage, and
`table.cell_set_text` text mutations, including ordinary and independent
while-loop control-flow mutation coverage, plus `table.cell_set_bgcolor`
background color mutations, including ordinary and independent while-loop
control-flow mutation coverage, plus `table.cell_set_text_color` text-color
mutations, including ordinary and independent while-loop control-flow mutation
coverage, plus `table.cell_set_width` width mutations, including ordinary and
independent while-loop control-flow mutation coverage, plus
`table.cell_set_height` height mutations, including ordinary and independent
while-loop control-flow mutation coverage, plus `table.cell_set_text_size`
text-size mutations, including ordinary and independent while-loop
control-flow mutation coverage, plus `table.cell_set_text_halign` horizontal
text-alignment mutations, including ordinary and independent while-loop
control-flow mutation coverage, plus `table.cell_set_text_valign` vertical
text-alignment mutations, including ordinary and independent while-loop
control-flow mutation coverage, plus `table.cell_set_text_wrap` text-wrap
mutations, including ordinary and independent while-loop control-flow mutation
coverage, plus `table.cell_set_tooltip` tooltip mutations, including ordinary
and independent while-loop control-flow mutation coverage, plus
`table.cell_set_text_font_family` font-family mutations, including ordinary and
independent while-loop control-flow mutation coverage, plus
`table.cell_set_text_formatting` text-formatting mutations, including
ordinary and independent while-loop control-flow mutation coverage, for previously populated cells
with
deterministic table dimensions, a 50-table runtime limit, and a 1000-cell
per-table limit. `table.all` returns currently existing table ids in creation
order, including when read from ordinary and independent while-loop control-flow
blocks after table deletion. Supported label, line, box, and table id-first drawing
functions can also use Pine method syntax, where the object receiver becomes
the first function argument; for example, `id.set_text("x")` is analyzed and
lowered as `label.set_text(id, "x")` when `id` is a label. This method syntax
does not widen the supported method set: unsupported drawing methods,
chart-point overloads and unsupported box chart-point coordinate variants
remain unsupported.
Deleting `na`,
mutating `na`, or mutating an already deleted
drawing object is a no-op where deletion exists; supported label getters return
`na` for `na` or deleted label ids; invalid non-`na` ids are runtime errors; ids
are stable and not reused. `label.new` can initialize host-neutral label
`xloc` values `xloc.bar_index`/`xloc.bar_time`, `yloc` values `yloc.price`,
`yloc.abovebar`, and `yloc.belowbar`, color/text-color fields, official
`label.style_*` values, size constants or integer sizes, `textalign`,
`text_font_family`, and `text_formatting` snapshot fields; its `force_overlay`
argument is accepted but left to the host display layer.
`label.set_x`, `label.set_y`, `label.set_xy`, `label.set_text`, and
`label.set_size` update the latest existing label snapshot, including when
called from ordinary and independent while-loop control-flow blocks.
`label.set_color`, `label.set_textcolor`, `label.set_style`,
`label.set_tooltip`, `label.set_textalign`, `label.set_text_font_family`, and
`label.set_text_formatting` update their host-neutral snapshot fields, including
when called from ordinary and independent while-loop control-flow blocks.
`label.set_style` accepts the official label style constants and stores the
selected constant in the latest label snapshot.
`label.set_xloc` records `xloc.bar_index` or `xloc.bar_time` plus the new `x`
value in label snapshots, including when called from ordinary and independent
while-loop control-flow blocks; `label.set_yloc` records `yloc.price`,
`yloc.abovebar`, or `yloc.belowbar`, including when called from ordinary and
independent while-loop control-flow blocks; `label.set_textalign` records horizontal text alignment
in label snapshots. `label.set_text_font_family` records font
family in label snapshots. `label.set_text_formatting` records a
`text.format_none`/`text.format_bold`/`text.format_italic` bitmask, including
bold+italic combinations, while actual glyph styling remains host-specific.
Text layout remains host-specific. `label.delete` appends an `exists: false`
label snapshot, including when called from ordinary and independent while-loop
control-flow blocks.
`label.copy` clones the latest existing label
snapshot into a new deterministic id, including when called from ordinary and
independent while-loop control-flow blocks, returns `na` for `na` or deleted
labels, and shares the effective label limit. `label.get_x` reads the latest
existing label x-coordinate,
including when called from ordinary and independent while-loop control-flow
blocks, and returns `na` for `na` or deleted labels. `label.get_y` reads the
latest existing label y-coordinate, including when called from ordinary and
independent while-loop control-flow blocks, and returns `na` for `na` or deleted
labels.
`label.get_text` reads the latest existing label text, including when called
from ordinary and independent while-loop control-flow blocks, and returns `na`
for `na` or deleted labels. `label.all` returns currently existing label ids in
creation order, including when read from ordinary and independent while-loop
control-flow blocks after label deletion or max-count eviction. Selected
`line.set_*` mutators update
endpoint/color/width/style/extend snapshots, and `line.set_xloc` with
`xloc.bar_index` or `xloc.bar_time` updates x1, x2, and xloc snapshot values,
including when called from ordinary and independent while-loop control-flow
blocks. `line.delete` appends an
`exists: false` line snapshot, including when called from ordinary and
independent while-loop control-flow blocks. `line.copy` clones the latest
existing line snapshot into a new deterministic id, including when called from
ordinary and independent while-loop control-flow blocks,
returns `na` for `na` or deleted lines, and shares the effective line runtime
limit.
`line.all` returns currently existing line ids in creation order, including
when read from ordinary and independent while-loop control-flow blocks after
line deletion or max-count eviction.
`box.copy` clones the latest existing box
snapshot into a new deterministic id, including when called from ordinary and
independent while-loop control-flow blocks, returns `na` for `na` or deleted
boxes, and shares the effective box limit. `box.set_bgcolor`,
`box.set_border_color`, `box.set_border_width`, `box.set_border_style`, and
`box.set_extend` update box style snapshots, including when called from
ordinary and independent while-loop control-flow blocks. `box.set_xloc` with
`xloc.bar_index` or `xloc.bar_time` updates left, right, and xloc values in box
snapshots, including when called from ordinary and independent while-loop
control-flow blocks. `box.set_left`, `box.set_top`,
`box.set_right`, `box.set_bottom`, `box.set_lefttop`, and
`box.set_rightbottom` update box geometry snapshots, including when called from
ordinary and independent while-loop control-flow blocks. `box.set_text`,
`box.set_text_color`, `box.set_text_size`, `box.set_text_halign`,
`box.set_text_valign`, `box.set_text_wrap`, `box.set_text_font_family`, and
`box.set_text_formatting` update box text snapshots, including when called from
ordinary and independent while-loop control-flow blocks. `box.set_text_formatting`
records a `text.format_none`/`text.format_bold`/`text.format_italic` bitmask,
including bold+italic combinations, while actual glyph styling remains
host-specific.
Richer box text layout remains unsupported. `box.get_left`,
`box.get_right`, `box.get_top`, and `box.get_bottom` read the latest existing
box snapshot, including when called from ordinary and independent while-loop
control-flow blocks, and return `na` for `na` or deleted boxes; other box
methods remain unsupported. `table.set_position` updates only the table's final
position value, including when called from ordinary and independent while-loop
control-flow blocks, with table layout left to hosts. `table.new` optional `bgcolor`,
`frame_color`, `frame_width`, `border_color`, and `border_width` initialize only
the table's final background-color, frame-color, frame-width, border-color, and
border-width values.
`table.delete` appends an `exists: false` table snapshot, including when called
from ordinary and independent while-loop control-flow blocks. `table.clear` removes
already populated cells in the inclusive rectangular range from `start_column`,
`start_row` to `end_column`, `end_row`, including when called from ordinary and
independent while-loop control-flow blocks; it also removes merged-cell records
that intersect the cleared range, while preserving the table object and
table-level style fields.
`table.merge_cells` appends inclusive
`start_column`/`start_row` to `end_column`/`end_row` merge rectangles to the
host-neutral table snapshot, including when called from ordinary and independent
while-loop control-flow blocks; deleted or `na` table ids are no-ops, invalid
non-`na` ids are runtime
errors, and out-of-bounds, reversed, or overlapping merge ranges are runtime
errors. Later table-level and cell mutations of
deleted tables are no-ops.
`table.set_bgcolor`, `table.set_frame_color`, `table.set_frame_width`,
`table.set_border_color`, and `table.set_border_width` update only the table's
final style values, including when called from ordinary and independent
while-loop control-flow blocks; border rendering and table layout remain host
responsibilities. `table.cell_set_text`, `table.cell_set_bgcolor`,
`table.cell_set_text_color`, `table.cell_set_width`, `table.cell_set_height`,
`table.cell_set_text_size`, `table.cell_set_text_halign`,
`table.cell_set_text_valign`, `table.cell_set_text_wrap`,
`table.cell_set_tooltip`, `table.cell_set_text_font_family`, and
`table.cell_set_text_formatting` update only previously populated cell
snapshots, including when called from ordinary and independent while-loop
control-flow blocks, while preserving other supported fields. Visual layout,
text wrapping, tooltip display, font rendering, and bold/italic rendering
remain host-specific;
other table cell text rendering remains host-specific.
Supported drawing creation, mutation, cloning, getter, and cell writes are covered under realtime rollback where state
changes, and drawing side effects inside user-defined functions are rejected
under the existing side-effect policy. Keep unsupported coordinate modes and advanced object
methods out of the supported matrix until they have fixtures and public-output
coverage. `linefill.new` and `linefill.set_color` are partial: they create
runtime-owned linefill ids over supported line ids, emit sparse color snapshots,
mutate colors, return referenced line ids through `linefill.get_line1` and
`linefill.get_line2`, and replace the previous linefill for the same line pair.
`linefill.delete` appends deletion snapshots, including while-loop
control-flow deletes, and `linefill.all` returns currently existing linefill ids
in creation order while omitting replaced or deleted linefills.
`array.new_linefill` and `array.from` support linefill id arrays for generic
object-array storage, mutation, reads, and search; numeric, truth, sorting, and
join helpers still reject linefill arrays with type diagnostics. `chart.point`
is partial: `chart.point.new`, `chart.point.now`,
`chart.point.from_index`, `chart.point.from_time`, and `chart.point.copy`
construct/copy point values, and top-level `time`, `index`, and `price` field
reads and mutation are fixture-backed. `array.new<chart.point>()` can construct
chart-point arrays, `array.from(chart.point, ...)` can infer chart-point arrays,
and the generic storage/read/mutation/search subset can carry point values.
`polyline.new` can consume those arrays and copy their values into host-neutral
runtime snapshots, while `polyline.delete` and `polyline.all` cover the
historical and forming-bar rollback lifecycle subset. `polyline.new` uses the
runtime default 50-polyline limit and named `max_polylines_count` declaration
values from 1 through 100 to evict oldest active polyline snapshots before
creating new ones. `line.new`, `line.set_first_point`,
`line.set_second_point`, `box.set_top_left_point`, and
`box.set_bottom_right_point` can consume `chart.point` values. Typed
declarations, remaining drawing point overloads, and general polyline arrays
remain outside the supported subset; see
`docs/PHASE_E_POLYLINE_GATE.md`.

Phase H reserves `alerts` as a top-level runtime key in `schemaVersion: 3`.
The first supported alert subsets are `alertcondition(condition, title,
message)` with bool-compatible conditions, const-string titles, and
const-string messages, plus fixture-backed OHLCV placeholder interpolation for
`{{open}}`, `{{high}}`, `{{low}}`, `{{close}}`, and `{{volume}}`, plus
`{{ticker}}`, `{{interval}}`, `{{exchange}}`, and `{{time}}`, in the
`alertcondition` message only. `{{time}}` uses the existing UTC
`str.format_time` default format for the triggering bar timestamp.
`alert(message, freq?)` supports string-compatible dynamic messages and the fixture-backed
`alert.freq_once_per_bar`/`alert.freq_all`/`alert.freq_once_per_bar_close`
frequency subset. Reached true alert conditions and reached alert calls emit
`{id, barIndex, time, message, source}` events in program order, subject to
supported `alert()` frequency filtering; false and `na` alert conditions emit
nothing. Forming realtime events are visible in the forming result and roll back
until a confirmed update commits an event, while close-frequency alert calls are
suppressed on forming updates and emitted only on historical or confirmed
bar-close execution. Repeated forming updates recompute alert events from the
confirmed snapshot, so abandoned forming events are neither retained nor
duplicated, and a confirmed update matches the equivalent historical execution
where the same final bar data is available. TradingView-style `{{...}}` alert
placeholder interpolation outside the supported `alertcondition` message
placeholder subset remains unsupported; supported `alert()` messages are
serialized literally.

The official Pine Logs functions `log.info()`, `log.warning()`, and
`log.error()` remain unsupported. They require a host-owned log pane or runtime
output contract, so the analyzer reports them as explicit unsupported `log.*`
calls instead of treating them as unknown functions.

Pine map and matrix collections remain unsupported. `map.*` requires a
dedicated key/value storage model, and `matrix.*` requires a dedicated
two-dimensional storage model, so both namespaces are reported as explicit
unsupported collection families until their semantics are designed and
fixture-backed.

Runtime `schemaVersion: 4` added strategy order-fill alert payloads under
`strategy.alerts` for supported strategy fills. Runtime `schemaVersion: 5` adds
host-neutral `textWrap` to table cell snapshots. Runtime `schemaVersion: 6`
adds top-level `lineFills` snapshots for the supported linefill subset. Runtime
`schemaVersion: 7` adds top-level `polylines` snapshots for the supported
`polyline.new` and lifecycle subset. The
top-level `alerts[]` array remains limited to reached `alert()` and
`alertcondition()` callsites. Explicit Python, CLI, and WASM host helpers can render
`{{strategy.order.alert_message}}` from selected public strategy fill events,
while external alert delivery remains unsupported.

Checked-in golden JSON snapshots live in `tests/snapshots/`. Snapshot tests are
strict string comparisons against deterministic compact JSON; a public field
rename, omitted `schemaVersion`, or matrix shape change should fail tests. To
refresh snapshots after an intentional public-output change, run:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
UPDATE_SNAPSHOTS=1 cargo test -p pine-wasm analysis_outputs_match_golden_snapshots
cargo test --workspace
```

Review the resulting JSON diff before committing. Do not update snapshots to
hide accidental public contract changes.

Snapshot maintenance rules:

- Treat checked-in snapshots as public contract evidence, not generated noise.
- Update snapshots only with the targeted `UPDATE_SNAPSHOTS=1` commands above.
- Include the source change, snapshot diff, and documentation update in the
  same commit when a public output change is intentional.
- Run `scripts/verify.sh` after any snapshot refresh so CLI, WASM, Python, and
  matrix contracts are checked together.

## Numeric Tolerance

Floating point outputs should be compared with an explicit tolerance:

```text
absolute tolerance: 1e-10
relative tolerance: 1e-9
```

Some built-ins may need per-function tolerances if their documented formulas
accumulate rounding differently. Any wider tolerance must be justified in the
fixture metadata.

## Test Data

OHLCV fixtures should be small and deterministic.

Rules:

- Include enough bars to test warmup behavior.
- Include gaps or flat sections where indicators often fail.
- Include first-bar and out-of-range history cases.
- Do not depend on external market data downloads in unit tests.

## Unsupported Features

Unsupported fixtures are first-class tests.

Examples:

- unsupported `request.security` variants outside the same-context identity
  scalar/tuple-literal subset and same-or-higher-timeframe provider
  scalar/tuple-literal subset
- unsupported strategy declaration contexts and strategy order functions such as
  `strategy.order`; `strategy.exit` same-side pairs, custom OCA parameters, 3+
  triggers, invalid trailing combinations, and arbitrary future binding for
  unmatched `from_entry` ids remain fixture-backed unsupported cases.
  Stop-only `strategy.exit(id, from_entry, stop=price)`, limit-only
  `strategy.exit(id, from_entry, limit=price)`, profit-only
  `strategy.exit(id, from_entry, profit=ticks)`, loss-only
  `strategy.exit(id, from_entry, loss=ticks)`, and exactly one-downside plus
  one-upside brackets (`stop + limit`, `stop + profit`, `loss + limit`,
  `loss + profit`), plus trailing stops (`trail_price + trail_offset` and
  `trail_points + trail_offset`), optionally with fixed `qty` or `qty_percent`,
  are the narrow supported subsets for the current one-net-long broker.
  Supported brackets use
  stop/loss-first precedence when both legs are touched on the same eligible
  historical bar. Supported trailing stops do not fill on the activation bar
  and ratchet only upward after activation. Supported fixed `qty` exits close
  `min(qty, position_size)`, keep any remaining long position open at the same
  average price, and do not add public pending-order or remaining-quantity
  fields. Supported `qty_percent` exits evaluate the percent at placement time,
  resolve it to an absolute quantity against the current position size or the
  matching pending entry quantity for same-calculation absolute exit attachment,
  clamp fills to the current position, and expose only the absolute filled `qty`.
  When supported `strategy.exit` shapes supply both `qty` and `qty_percent`,
  fixed `qty` determines the reserved or filled quantity and `qty_percent` is
  ignored.
- minimal `strategy.entry` long market, long limit, long stop, and long
  stop-limit entries in strategy-mode scripts; market entries fill at the next
  historical bar open, limit entries fill at the limit price on a later
  historical bar when `low <= limit`, stop entries fill at the stop price on a
  later historical bar when `high >= stop`, stop-limit entries activate on a
  later historical bar when `high >= stop` and fill at the limit price on a
  subsequent historical bar when `low <= limit`, and no public pending-order
  output is exposed; unsupported short/indicator-mode variants are
  fixture-backed; entries may omit `qty` only when the strategy declaration
  configures the fixed default quantity subset
- minimal `strategy.close` full-position closes for matching long entry ids,
  with flat, wrong-entry, or repeated closes treated as no-op
- minimal `strategy.close_all` full-position closes for the current supported
  long position, with flat or already-closed calls treated as no-op
- minimal `strategy.cancel(id)` cancellation for matching supported internal
  pending entry ids and pending exit ids; filled, unknown, and already-cancelled
  ids are no-op, and no public pending-order or cancellation records are
  exposed
- minimal `strategy.cancel_all()` cancellation for all supported internal
  pending entries and pending exits; calling it without pending orders is a
  no-op, and no public pending-order or cancellation records are exposed
- minimal strategy equity snapshots with bar-close mark-to-market accounting,
  with broader broker settings and rich strategy reporting variables
  unsupported
- unsupported strategy reporting helpers beyond the supported position,
  profit, equity, run-up/drawdown, and trade-count variables, plus unknown `strategy.*`
  reporting helpers
- unsupported collection families or unsupported array variants
- unsupported label and line methods
- unsupported import variants outside the host-provided alias/exported
  const/pure-function subset
- unsupported `varip` forms such as drawing ids, tuples, and value families
  outside the scalar and scalar typed-array subset
- non-integer or negative history offsets
- unsupported function side effects, including drawing, alert, and strategy
  order side effects

Expected result:

- no panic
- stable diagnostic code
- source span
- machine-readable compatibility report entry

## Diagnostic Stability

Diagnostics should include:

```text
code
severity
message
span
feature id when applicable
help text when useful
```

Messages can improve over time, but codes should remain stable once published.

## Comparison Policy

Allowed comparison sources:

- public language documentation
- original mathematical formulas
- project-owned fixtures
- permissively licensed scripts with metadata
- user-provided scripts when the user has the right to use them

Disallowed:

- copied proprietary scripts
- private TradingView APIs
- scraped TradingView data
- copied official documentation text beyond short references
- TradingView UI or error text reproduction

## Release Compatibility Matrix

Every release should publish a generated or manually maintained matrix:

```text
feature              status       notes
indicator            supported
input.int            supported
ta.sma               supported
ta.ema               supported
ta.rsi               supported    fixture-derived executable subset
request.security     partial      same-context identity scalar-expression subset plus same-context tuple literals made from side-effect-free elements and selected same-context tuple-returning calls destructured directly from the request, currently ta.macd, ta.bb, ta.kc, ta.supertrend, ta.dmi, and ta.vwap(source, anchor, stdev_mult), and same-or-higher-timeframe provider scalar-expression subset with direct sources, arithmetic, history, na/nz, selected stateless math.* calls, fixed-mintick math.round_to_mintick, math.sum, ta.cum, ta.sma, ta.ema, ta.dema, ta.tema, ta.rma, ta.rsi, ta.accdist, ta.iii, ta.nvi, ta.obv, ta.pvi, ta.pvt, ta.wvad, ta.tsi, ta.cmo, ta.cci, ta.cog, ta.bop, ta.ao, ta.max, ta.min, ta.mfi, ta.stoch, ta.wpr, ta.sar, ta.tr function calls, ta.atr, ta.highest, ta.lowest, ta.highestbars, ta.lowestbars, ta.change, ta.mom, ta.roc, ta.range, ta.dev, ta.vwap source-call, ta.bbw, ta.kcw, ta.pivothigh, ta.pivotlow, ta.correlation, ta.covariance, ta.median, ta.mode, ta.percentile_nearest_rank, ta.percentile_linear_interpolation, ta.percentrank, ta.stdev, ta.variance, ta.wma, ta.vwma, ta.swma, ta.hma, ta.alma, ta.linreg, ta.rising, ta.falling, ta.barssince, ta.valuewhen, ta.cross, ta.crossover, and ta.crossunder only, plus provider-backed tuple literals made from supported scalar elements and provider-backed ta.macd, ta.bb, ta.kc, ta.supertrend, ta.dmi, and ta.vwap(source, anchor, stdev_mult) tuple expressions destructured directly from the request; other provider-backed tuple expressions remain unsupported
alertcondition       partial      bool-compatible condition plus const-string title/message runtime events, with OHLCV plus ticker/interval/exchange/time placeholders in alertcondition messages only
alert                partial      string-compatible dynamic message runtime events when execution reaches the call, with const-string frequency subset only
log.*                unsupported  Pine Logs output functions require a host-owned log pane/output contract and are not implemented
strategy             partial      declaration plus strategy-mode runtime result; positive const numeric initial_capital, fixed, cash, and percent-of-equity default_qty subsets, supported cash-per-contract, cash-per-order, and percent commission modes, finite non-negative integer slippage ticks, and finite non-negative integer limit-verification ticks only
strategy.entry       partial      long market entry filled at next historical bar open plus long limit entry filled at limit price on a later historical bar when low <= limit or below the configured verified limit threshold, long stop entry filled at stop price on a later historical bar when high >= stop, and long stop-limit entry activated on a later historical bar when high >= stop then filled at limit price on a subsequent historical bar when low <= limit or below the configured verified limit threshold; configured slippage worsens long entry fill prices after trigger selection; explicit positive qty, fixed default qty, cash default qty resolved as cash/current close, or percent-of-equity default qty resolved at placement time from current supported equity and close; explicit active margin_long rejects fills whose required margin exceeds simulated equity at the actual fill price; same-direction long market entries honor the configured positive integer pyramiding limit, while multiple long limit, stop, or stop-limit entries triggered on the same historical fill pass can exceed that limit when they are all eligible in that pass; comment, alert_message, and disable_alert metadata syntax is semantically accepted and stored internally on pending and filled entries; supported fill payloads are exposed in `strategy.alerts`; explicit Python, CLI, and WASM host helpers can render `{{strategy.order.alert_message}}` for selected public fill events, while external alert delivery remains unsupported; same-tick exceptions beyond the fixture-backed long price-based entry subset remain unsupported; no public pending-order output
strategy.close       partial      full long-position close, fixed-qty partial close, or qty_percent partial close of the matching current long entry id at current bar close; fixed qty and qty_percent must be finite and positive; qty_percent resolves against the current matching position size; qty wins when both quantity forms are supplied; oversized quantities clamp to the current matching position size, keep remaining long position state open at the same average price, preserve the public strategy JSON shape without close order events, and cancel matching pending exits only when the close fully flattens the entry; configured slippage worsens the long close fill price; flat, wrong-entry, or repeated closes are no-op; comment, alert_message, and disable_alert syntax is accepted and stored internally on closed-trade metrics; supported fill payloads are exposed in `strategy.alerts`; explicit Python, CLI, and WASM host helpers can render `{{strategy.order.alert_message}}` for selected public fill events, while external alert delivery remains unsupported; immediately, partial strategy.close_all, and multi-entry close allocation remain unsupported
strategy.close_all   partial      full close of the current supported long position at current bar close; flat or already-closed calls are no-op; closed trade output uses the current entry id; comment, alert_message, and disable_alert syntax is accepted and stored internally on closed-trade metrics; supported fill payloads are exposed in `strategy.alerts`; explicit Python, CLI, and WASM host helpers can render `{{strategy.order.alert_message}}` for selected public fill events, while external alert delivery remains unsupported
strategy.cancel      partial      cancels matching internal pending entry ids and pending exit ids in the supported order subset; filled, unknown, and already-cancelled ids are no-op; no public pending-order output or cancellation records
strategy.cancel_all  partial      cancels all supported internal pending entries and pending exits; no-op when there are no pending orders; no public pending-order output or cancellation records
strategy equity      partial      per-bar cash, marketValue, equity, and netProfit snapshots; supports strategy.commission.cash_per_contract, strategy.commission.cash_per_order, and strategy.commission.percent commission debits plus declaration slippage applied to supported fill prices
strategy.position_size partial    current long-only position size read-only series in strategy-mode scripts only; supports fixture-backed control-flow, UDF argument, and history-reference interactions
strategy.position_avg_price partial current long-only average entry price read-only series, na when flat, in strategy-mode scripts only
strategy.max_contracts_held_all partial maximum contracts/shares/lots/units held over the whole trading range as a read-only series float in strategy-mode scripts only; aliases the supported long-only maximum while shorts are unsupported
strategy.max_contracts_held_long partial maximum long contracts/shares/lots/units held over the whole trading range as a read-only series float in strategy-mode scripts only
strategy.max_contracts_held_short partial maximum short contracts/shares/lots/units held over the whole trading range as a read-only series float in strategy-mode scripts only; remains 0.0 because short entries are unsupported
strategy.openprofit partial       current long-only unrealized profit read-only series, 0 when flat, in strategy-mode scripts only; supports fixture-backed control-flow, UDF argument, and history-reference interactions
strategy.netprofit  partial       cumulative realized closed-trade profit read-only series, excluding current open profit, in strategy-mode scripts only
strategy.netprofit_percent partial cumulative realized closed-trade profit as a percentage of initial_capital, excluding current open profit, in strategy-mode scripts only
strategy.grossprofit partial      cumulative positive realized closed-trade profit read-only series, excluding losing, flat, and current open trades, in strategy-mode scripts only
strategy.grossprofit_percent partial cumulative positive realized closed-trade profit as a percentage of initial_capital, excluding losing, flat, and current open trades, in strategy-mode scripts only
strategy.grossloss partial        cumulative realized closed-trade loss read-only series as a positive value, excluding winning, flat, and current open trades, in strategy-mode scripts only
strategy.grossloss_percent partial cumulative realized closed-trade loss as a positive percentage of initial_capital, excluding winning, flat, and current open trades, in strategy-mode scripts only
strategy.avg_trade partial        average realized profit/loss per closed trade read-only series, na before the first closed trade and excluding current open trades, in strategy-mode scripts only
strategy.avg_trade_percent partial average realized per-trade profit/loss percentage read-only series, using each closed trade entry value as denominator, na before the first closed trade and excluding current open trades, in strategy-mode scripts only
strategy.avg_winning_trade partial average realized profit among winning closed trades only, na before the first winning closed trade and excluding losing, flat, and current open trades, in strategy-mode scripts only
strategy.avg_winning_trade_percent partial average realized percentage gain among winning closed trades only, using each closed trade entry value as denominator, na before the first winning trade and excluding losing, flat, and current open trades, in strategy-mode scripts only
strategy.avg_losing_trade partial average realized loss among losing closed trades only as a positive value, na before the first losing closed trade and excluding winning, flat, and current open trades, in strategy-mode scripts only
strategy.avg_losing_trade_percent partial average realized percentage loss among losing closed trades only as a positive value, using each closed trade entry value as denominator, na before the first losing trade and excluding winning, flat, and current open trades, in strategy-mode scripts only
strategy.max_runup partial        maximum intrabar equity run-up amount read-only series over the current supported long-only trading interval, using supported entry equity, minimum equity before that entry, and the highest high reached while the supported position is open
strategy.max_runup_percent partial maximum intrabar equity run-up percentage read-only series over the current supported long-only trading interval, dividing the supported run-up amount by entry price times current supported position quantity and multiplying by 100
strategy.max_drawdown partial     maximum intrabar equity drawdown amount read-only series over the current supported long-only trading interval, using supported entry equity, maximum equity before that entry, and the lowest low reached while the supported position is open
strategy.max_drawdown_percent partial maximum intrabar equity drawdown percentage read-only series over the current supported long-only trading interval, dividing the supported drawdown amount by entry price times current supported position quantity and multiplying by 100
strategy.equity     partial       cash plus current market value read-only series in strategy-mode scripts only; without configured commission or slippage this matches initial_capital plus realized net profit plus current open profit, and with supported commission/slippage it reflects entry commission debits on open positions and slippage-adjusted fill prices
strategy.closedtrades partial     closed-trade count read-only series int in strategy-mode scripts only; immediate after strategy.close or strategy.close_all and next-bar visible after pending strategy.exit fills
strategy.closedtrades.* partial   closed-trade entry_price, entry_comment, entry_id, exit_price, exit_comment, exit_id, entry_bar_index, exit_bar_index, entry_time, exit_time, commission, size, profit, max_runup, and max_drawdown field functions in strategy-mode scripts only; entry_comment returns the retained entry comment when present; entry_id returns the retained entry id; exit_comment returns the retained close or selected exit comment when present; exit_id returns the retained close or exit id; commission is 0.0 without configured commission or supported entry-plus-exit commission when configured; max_runup returns the largest high-based favorable excursion retained for the closed trade quantity; max_drawdown returns the largest low-based adverse excursion retained for the closed trade quantity; trade_num is zero-based integer-only and invalid, negative, non-integer, or out-of-range indexes return na; no public runtime schema expansion
strategy.closedtrades.max_runup partial closed-trade max runup field function in strategy-mode scripts only; uses the largest high-based favorable excursion retained for the closed trade quantity; no public runtime schema expansion
strategy.closedtrades.max_drawdown partial closed-trade max drawdown field function in strategy-mode scripts only; uses the largest low-based adverse excursion retained for the closed trade quantity; no public runtime schema expansion
strategy.closedtrades.entry_comment partial closed-trade entry comment field function in strategy-mode scripts only; returns the retained entry comment when present; no public runtime schema expansion
strategy.closedtrades.exit_comment partial closed-trade exit comment field function in strategy-mode scripts only; returns the retained close or selected exit comment when present; no public runtime schema expansion
strategy.wintrades partial        closed winning-trade count read-only series int in strategy-mode scripts only; counts closed trades with positive realized profit
strategy.losstrades partial       closed losing-trade count read-only series int in strategy-mode scripts only; counts closed trades with negative realized profit
strategy.eventrades partial       closed even-trade count read-only series int in strategy-mode scripts only; counts closed trades with zero realized profit
strategy.opentrades partial       open-trade count read-only series int in strategy-mode scripts only; 1 for the current supported long position and 0 when flat
strategy.opentrades.* partial     open-trade field function subset limited to entry_price, entry_comment, entry_id, entry_bar_index, entry_time, size, profit, commission, max_runup, and max_drawdown for the current supported long position, plus the capital_held variable; entry_comment returns the retained entry comment when present; trade_num must be 0 and invalid or flat-state function reads return na; commission returns 0.0 without configured commission or current open supported entry commission when configured; max_runup returns the largest high-based favorable excursion seen so far; max_drawdown returns the largest low-based adverse excursion seen so far; capital_held returns na without active margin, 0.0 while flat with active margin, and current open long market value times margin_long / 100 with explicit active margin_long; no public runtime schema expansion
strategy.opentrades.capital_held partial open-trade capital held variable in strategy-mode scripts only; returns na in the no-margin subset, 0.0 while flat with active margin, and current open long market value times margin_long / 100 while the supported long position is open, including after long-only forced liquidation reduces the position; short margin remains unsupported; no public runtime schema expansion
strategy.opentrades.entry_price partial current open-trade entry price field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.entry_comment partial current open-trade entry comment field function in strategy-mode scripts only; returns the retained entry comment when present; no public runtime schema expansion
strategy.opentrades.entry_id partial current open-trade entry id field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.entry_bar_index partial current open-trade entry bar index field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.entry_time partial current open-trade entry time field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.size partial  current open-trade size field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.profit partial current open-trade floating profit field function in strategy-mode scripts only; no public runtime schema expansion
strategy.opentrades.commission partial current open-trade commission field function in strategy-mode scripts only; returns 0.0 without configured commission or current open supported entry commission when configured; no public runtime schema expansion
strategy.opentrades.max_runup partial current open-trade max runup field function in strategy-mode scripts only; uses the largest high-based favorable excursion seen so far; no public runtime schema expansion
strategy.opentrades.max_drawdown partial current open-trade max drawdown field function in strategy-mode scripts only; uses the largest low-based adverse excursion seen so far; no public runtime schema expansion
strategy.exit       partial      stop-only, limit-only, profit-only, loss-only, one-downside/one-upside bracket, trailing, and optional fixed-qty or qty-percent long exits; absolute stop/limit exits can match a requested open pyramided long entry id by `from_entry`, and omitted-`from_entry` absolute stop/limit exits can close all currently open pyramided long entries and persist for later open long entries until the position closes; single-trigger and bracket profit/loss tick exits plus trailing trail_points activation for an open pyramided long entry convert from the matched entry price; omitted-`from_entry` full profit/loss-tick exits and full stop+limit, stop+profit, loss+limit, or loss+profit brackets can close currently open pyramided long entries with unique entry ids using each entry price for relative legs when present; omitted-`from_entry` current full profit/loss-tick exits, full trail_points+trail_offset and trail_price+trail_offset trailing exits, plus full loss+profit, stop+profit, loss+limit, and stop+limit brackets can also close same-entry-id pyramided long trades using each open trade entry price; omitted-`from_entry` full profit/loss-tick exits, full trail_points+trail_offset and trail_price+trail_offset trailing exits, plus full loss+profit, stop+profit, loss+limit, and stop+limit brackets can also persist for later same-entry-id pyramided long trades using each later open trade entry price for relative legs when present; omitted-`from_entry` full profit/loss-tick exits and full loss+profit, stop+profit, and loss+limit brackets can also persist for later open long entries with unique entry ids until the position closes; omitted-`from_entry` full stop+limit brackets can also persist for later open long entries until the position closes; omitted-`from_entry` full trail_price+trail_offset trailing exits can close currently open pyramided long entries and persist for later open long entries until the position closes, and full trail_points+trail_offset trailing exits can do the same for currently open unique entry ids and persist for later open long entries with unique entry ids using each entry price for activation; exits matching multiple open trades with the same entry id emit one public exit order and one closed trade per matched ledger allocation; single-trigger same-calculation absolute stop/limit/trail_price attachment and single-trigger same-calculation entry-relative profit/loss/trail_points attachment to a pending entry are supported for the active entry id; active-entry relative bracket forms remain unsupported until Stage 10 behavior slices resolve deferred bracket legs; bracket forms are stop+limit, stop+profit, loss+limit, and loss+profit for the current one-net-long entry; trailing forms are trail_price+trail_offset and trail_points+trail_offset; profit/loss/trailing ticks convert with fixed syminfo.mintick; configured limit verification requires long limit/profit exit fills to move beyond the limit/profit price while preserving the original limit/profit fill price; qty is placement-time finite positive absolute quantity; qty_percent is placement-time finite positive percent resolved to an absolute quantity against current position size, matching open pyramided entry quantity, or matching pending entry quantity; when qty and qty_percent are both supplied, qty determines the reserved or filled quantity; omitted qty and qty_percent keep full-position one-effective-pending replacement behavior; explicit fixed-qty or qty-percent single-trigger, bracket, and trailing calls can keep multiple reserved pending exits; comment, comment_profit, comment_loss, comment_trailing, alert_message, alert_profit, alert_loss, alert_trailing, and disable_alert metadata syntax is semantically accepted and stored internally on pending and deferred exits; supported fill payloads are exposed in `strategy.alerts`; explicit Python, CLI, and WASM host helpers can render `{{strategy.order.alert_message}}` for selected public fill events, while external alert delivery remains unsupported; fills clamp to current position size, leave remaining long position open when partial, expose only absolute filled qty, and apply configured slippage to the long exit fill price after trigger selection; later-bar low <= stop/loss/active trailing stop or high >= verified limit/profit/activation price drives fills/activation; same-side touched exits fill in placement order; mixed downside/upside same-bar touches fill downside candidates only; bracket both-leg touches contribute the downside candidate; trailing activation bars do not fill; branch/switch/loop/state/history/incremental/host interactions fixture-backed
strategy.*           unsupported  strategy order functions beyond strategy.entry/strategy.close/strategy.close_all/strategy.cancel/strategy.cancel_all and the supported single-trigger, one-downside/one-upside bracket, trailing, optional fixed-qty and qty-percent strategy.exit subset, and fixed-qty or qty-percent single-trigger/bracket/trailing multiple-exit reservation subset; strategy.exit same-side pairs stop+loss and limit+profit, 3+ trigger/invalid trailing/multiple-pending outside that subset/omitted-quantity multiple reservations/reservation outside that subset/arbitrary future binding for unmatched `from_entry` ids; rich order types, cash/contracts sizing, mutable strategy state, margin behavior beyond long-entry affordability, long-only capital_held, and long-only forced liquidation, open-trade namespace functions outside entry_price/entry_comment/entry_id/entry_bar_index/entry_time/size/profit/commission/max_runup/max_drawdown/capital_held, closed-trade namespace functions outside entry_price/entry_comment/entry_id/exit_price/exit_comment/exit_id/entry_bar_index/exit_bar_index/entry_time/exit_time/commission/size/profit/max_runup/max_drawdown, commission modes outside strategy.commission.cash_per_contract, strategy.commission.cash_per_order, and strategy.commission.percent, fill models beyond fixed-tick slippage and fixed-tick limit verification on supported long fills, rich reporting metrics, and strategy reporting helpers beyond the supported position/profit/equity/count/held-quantity/runup/drawdown and supported trade field variables are not implemented
array.*              partial      float/int/bool/string/color/label/line/linefill/box/table creation plus chart.point array.new<chart.point> construction and array.from inference, reference, copy, get/set/insert/remove with negative indexes, fill, slice/concat, search including linefill and chart.point object-array search, binary search, float/int/bool truth helpers, numeric abs/statistics/range/median/mode/percentile/covariance/standardize/variance/stdev, numeric/string sort and sort_indices, branch/loop sort and reverse mutation fixtures, scalar-array join, mutation, and helper fixture subset only; polyline.all returns read-only snapshot polyline id arrays, while general polyline array construction and mutation remain unsupported
chart.point          partial      chart.point.new/now/from_index/from_time/copy constructors, time/index/price field reads, top-level field mutation, and array.new<chart.point>/array.from chart-point array storage/read/mutation/search are fixture-backed; line.new, line point setters, and box point setters consume chart.point values through dedicated partial rows, and polyline.new consumes chart-point arrays through its dedicated partial row, including default/declaration-driven max-count eviction; typed declarations, remaining drawing point overloads, and general polyline arrays remain unsupported
map.*                unsupported  map collections require a dedicated key/value storage model and are not implemented
matrix.*             unsupported  matrix collections require a dedicated two-dimensional storage model and are not implemented
linefill.new         partial      linefill object creation between existing line ids with color snapshots and official same-pair replacement semantics; na or deleted line ids return na; linefill array construction is supported through array.new_linefill and array.from for linefill ids
linefill.set_color   partial      linefill color mutation for existing linefill ids, including namespace-call and method-call dispatch, na id no-op behavior, and no-op behavior after the linefill has been replaced/deleted by a same-pair linefill.new call
linefill.get_line1   partial      returns the first line id referenced by an existing linefill, including namespace-call dispatch; na ids and replaced linefill ids return na
linefill.get_line2   partial      returns the second line id referenced by an existing linefill, including method-call dispatch; na ids and replaced linefill ids return na
linefill.delete      partial      linefill id deletion snapshots, including ordinary and independent while-loop control-flow deletion; deleting replaced, deleted, or na linefills is no-op; ids are not reused
linefill.all         partial      snapshot array of currently existing linefill ids in creation order, including ordinary and while-loop control-flow reads; replaced or deleted linefills are omitted from subsequent reads while linefill array construction is supported through array.new_linefill and array.from for linefill ids
request.security_lower_tf unsupported lower-timeframe array-returning request API is not implemented
request.*            unsupported  request families beyond the narrow request.security subsets
import               partial      host-provided exact-key imports with aliases, exported const expressions, and pure exported functions only
user-defined types   partial      local scalar-field type declarations, Type.new constructors, field reads, ordinary variables, local for-expression constructor results, var persistence, scalar field mutation outside UDF/method bodies including branch and for-loop bodies, UDF parameter passthrough/returns through positional or named scalar arguments with direct returns, UDT block-local aliases, or nested passthrough calls, and UDF constructor returns, directly, through nested pure constructor-helper UDF calls, or through same-local-UDT ternary, switch, final if/else constructor branches, or final for bodies, from local UDT parameter scalar fields, scalar fields read through block-local UDT aliases of those parameters, block-local scalar aliases of those fields, inferred scalar parameters, or block-local scalar aliases of those scalar parameters using positional or named constructor field arguments only
user-defined methods partial      pure methods on local UDT receivers with scalar or local UDT parameters, direct UDT passthrough returns, block-local receiver or local UDT parameter alias passthrough returns, final if/else or final for local UDT alias passthrough returns, nested-method UDT parameter passthrough returns, and local UDT constructor returns, directly, through nested pure constructor-helper UDF calls, or through same-local-UDT ternary, switch, final if/else constructor branches, or final for bodies, from receiver or local UDT parameter scalar fields, scalar fields read through block-local receiver or local UDT parameter aliases, block-local scalar aliases of those fields, inferred scalar parameters, or block-local scalar aliases of those parameters using positional or named constructor field arguments only
```

The matrix should be generated from conformance metadata once the test harness
exists.

Request support must cite request-specific fixtures. The conformance validator
rejects supported or partial `request.*` rows that only point at unrelated
runtime fixtures, so public request claims stay tied to request host-data,
semantic, or runtime coverage. The closed Phase F request boundary and
maintenance tails are summarized in `docs/PHASE_F_AUDIT.md`.

Current CLI output:

```text
pine-compat matrix
pine-compat matrix --format json
```

The generated matrix is derived from `tests/fixtures/conformance.tsv`. Each row
declares a feature, status, notes, and one or more fixture paths that back the
claim. CLI tests verify that every matrix entry references at least one existing
fixture. The text matrix includes the fixture paths, and the JSON matrix exposes
top-level matrix `schemaVersion` plus a `features` array whose entries expose
fixture paths as `fixtures`.

Conformance metadata is validated before matrix output is trusted:

- `feature` must be non-empty and unique.
- `status` must be `supported`, `partial`, or `unsupported`.
- `notes` must be non-empty.
- `fixtures` must contain at least one path and no empty `;` entries.
- Every fixture path must exist in the workspace.
- `supported` and `partial` entries must cite executable, realtime, syntax,
  positive semantic, or regression coverage.
- `unsupported` entries must cite unsupported semantic diagnostic fixtures.
- Every supported built-in registry entry and known unsupported platform family
  must remain represented.

Malformed rows, duplicate feature names, invalid statuses, missing fixture paths,
and status/fixture mismatches are first-class tests. The matrix command is
derived from the same validated metadata used by those tests.

Matrix maintenance rules:

- Edit `tests/fixtures/conformance.tsv` first; do not hand-edit generated
  matrix output.
- Add or update fixture paths in the same change as any new supported,
  partial, or unsupported claim.
- Keep unsupported platform families represented even when they remain outside
  the executable subset.
- If the JSON matrix shape changes, refresh `tests/snapshots/matrix.json` and
  document the public contract change in release notes.
- Use `pine-compat matrix --format json` to inspect the release matrix exposed
  to consumers.

The current scalar typed-array subset is summarized in
`docs/ARRAY_STAGE_AUDIT.md`. Keep `array.*` marked `partial` until the deferred
generic, object, UDT, map/matrix, history, and slice-aliasing semantics are
designed and fixture-backed.

The current `varip` subset is summarized in `docs/PHASE_I_AUDIT.md`. Keep
`varip` marked `partial` until drawing object ids, tuples, maps, matrices, UDTs,
imports, object arrays, generic arrays, and other value families have designed
rollback semantics and fixture coverage.
