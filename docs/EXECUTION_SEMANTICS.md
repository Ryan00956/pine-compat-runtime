# Execution Semantics

Pine Compat Runtime should be designed around time-series execution, not around
ordinary one-shot script execution.

The central rule is:

```text
The program is evaluated once for each bar, in order.
Each execution can read previously committed series values.
Each execution commits new series values for later bars.
```

## Historical Batch Execution

The first runtime mode should execute a fixed historical data set:

```text
for bar_index in 0..bars.len():
  set current OHLCV built-ins
  execute global scope
  execute reached local scopes
  collect output side effects
  commit series values
```

This mode is easier to test and should be completed before realtime execution.

## Realtime Execution

Realtime execution introduces forming bars, repeated updates on the same bar,
rollback, and later `varip` behavior. This should be a separate milestone.

The first public releases should either reject realtime-only features or mark
them as approximate.

## Strategy Mode

`strategy(...)` selects strategy mode for historical execution.
Strategy-mode runtime results include a `strategy` object with `orders`,
`trades`, `position`, `equity`, and `diagnostics` arrays. Indicator-mode
runtime results do not include this key.
`strategy(..., initial_capital=N)` accepts a positive const numeric starting
cash value; when omitted, the runtime uses 100000.
`strategy(..., default_qty_type=strategy.fixed, default_qty_value=N)` accepts a
positive const numeric fixed default entry quantity.
`strategy(..., default_qty_type=strategy.cash, default_qty_value=N)` accepts a
positive const numeric cash amount. When a supported `strategy.entry` omits
`qty`, the cash subset calculates the absolute quantity once at placement time as
`N / close`, using the current close and the current no-currency-conversion
boundary.
`strategy(..., default_qty_type=strategy.percent_of_equity, default_qty_value=N)`
accepts a positive const numeric default entry percentage. When a supported
`strategy.entry` omits `qty`, the percent-of-equity subset calculates the
absolute quantity once at placement time as
`strategy.equity * N / 100 / close`, using the current supported equity and
current close. Contracts, margin constraints beyond the current explicit-margin
long-only subset, currency conversion, symbol precision rounding, and lot-step
constraints remain unsupported.
`strategy(..., commission_type=strategy.commission.cash_per_contract,
commission_value=N)` accepts a finite non-negative const numeric
cash-per-contract commission. `strategy(...,
commission_type=strategy.commission.cash_per_order, commission_value=N)`
accepts a finite non-negative const numeric cash-per-order commission.
`strategy(..., commission_type=strategy.commission.percent,
commission_value=N)` accepts a finite non-negative const numeric percentage
commission and debits `qty * fill_price * N / 100` on each supported entry and
exit fill.
`strategy(..., slippage=N)` accepts finite non-negative integer const ticks
using the fixed `syminfo.mintick` subset.
`strategy(..., backtest_fill_limits_assumption=N)` accepts finite non-negative
integer const ticks and requires supported limit-order fills to move that many
fixed `syminfo.mintick` ticks past the limit price while preserving the limit
fill price. Other commission modes and richer fill models remain unsupported.
`strategy(..., margin_long=N, margin_short=N)` accepts finite non-negative
const numeric declaration values and stores their explicit presence in the
internal strategy settings. Stage 7 Margin Slice M2 uses explicit active
`margin_long` for long-only `strategy.opentrades.capital_held`; Stage 7 Margin
Slice M3 also checks supported long entry affordability at the actual fill
price. Stage 7 Margin Slice M5 implements the first long-only forced
liquidation subset using `bar.low`, the documented available-funds algorithm,
and whole-unit truncation. `strategy.margin_liquidation_price` returns the
current long-only price where supported equity equals required long margin for
an active `margin_long` position, or `na` without active long margin, while
flat, or when the long-margin denominator is unattainable. Short margin
behavior, symbol tick rounding for the liquidation price, and margin-specific
public schema expansion remain unsupported.
`strategy(..., close_entries_rule="FIFO")` is accepted as an explicit default
FIFO close-entry allocation setting. `strategy(..., close_entries_rule="ANY")`
is stored in internal strategy settings and is fixture-backed for the current
long-only id-specific `strategy.close(id)` and
`strategy.exit(..., from_entry=id)` allocation subset. Omitted-`from_entry`
exits and `strategy.close_all()` keep the existing FIFO allocation path.

The current entry subset is `strategy.entry(id, strategy.long, qty=...)`,
`strategy.entry(id, strategy.long)` when a fixed default quantity is configured,
`strategy.entry(..., limit=price)` for long limit entries, and
`strategy.entry(..., stop=price)` for long stop entries. Supplying both `stop`
and `limit` creates a long stop-limit entry. Market entries fill at the next
historical bar open. Limit and stop entries never fill on their creation bar;
limit entries fill at the limit price before script statements on a later
historical bar when `low <= limit`, or below the configured verified limit
threshold, and stop entries fill at the stop price before script statements on
a later historical bar when `high >= stop`.
Stop-limit entries activate before script statements on a later historical bar
when `high >= stop`, do not fill on that activation bar, and fill at the limit
price before script statements on a later historical bar when `low <= limit`,
or below the configured verified limit threshold.
Pending entries emit no public order while pending. Only one net long position
is supported; repeated entry calls while a position is open are ignored under
the current no-pyramiding rule. Explicit `qty` overrides the declaration
default. The resolved quantity, limit price, and stop price must be positive,
and non-positive runtime values are reported in the strategy diagnostics array.
Configured slippage worsens supported long entry fill prices after trigger
selection.
Supported `strategy.exit` calls use the pending-exit model described below.
Same-calculation absolute `strategy.exit` attachment may target a matching
active pending entry id and remains internal until that entry fills.
`strategy.cancel(id)` cancels matching internal pending entry ids and matching
pending exit ids in the supported order subset. Unknown, already-filled, and
already-cancelled ids are no-op. Cancellation records no public order, trade, or
pending-order output.
`strategy.cancel_all()` cancels all currently supported internal pending entries
and pending exits. It is a no-op when no supported pending order exists and
records no public order, trade, or pending-order output.

`strategy.close(id)` closes the full matching long position at the current bar
close. `strategy.close(id, qty=...)` and
`strategy.close(id, qty_percent=...)` can close part of the matching long
position; fixed `qty` wins when both quantity forms are present. Fixed and
percent quantities must be finite and positive, oversized quantities clamp to
the current matching position size, and invalid quantities leave position,
pending exit, and trade state unchanged while emitting a strategy diagnostic.
Configured slippage worsens the supported long close fill price after trigger
selection. A close records a closed trade with entry/exit bar indexes,
entry/exit times, entry/exit prices, quantity, and net realized profit after
supported commission when configured. Partial closes append a remaining
position snapshot at the same average price and keep matching pending exits;
full closes append a flat position snapshot with `size = 0` and
`avgPrice = null` and cancel matching pending exits. If no position is open, the
id does not match the open entry, or the position has already been closed, the
close call is a no-op.

After each historical bar, strategy mode appends an equity snapshot with
`barIndex`, `cash`, `marketValue`, `equity`, and `netProfit`. Open long
positions are marked to the current bar close, `equity = cash + marketValue`,
and the snapshot field `netProfit = equity - initial_capital`, so that public
output field includes current open profit while a long position is open. The
expression variable `strategy.netprofit` is narrower: it is cumulative realized
closed-trade profit only and excludes current open profit. The current subset
supports only `strategy.commission.cash_per_contract`,
`strategy.commission.cash_per_order`, `strategy.commission.percent`,
fixed-tick slippage, and fixed-tick limit verification, and has no other
commission modes, richer fill models, margin,
percent sizing, currency conversion,
unmatched missing-entry future binding, entry-relative pending-entry exit
attachment, or pyramiding. The only multiple-pending
reservation subset is explicit fixed `qty` or `qty_percent` single-trigger or
one-downside/one-upside bracket or trailing `strategy.exit` calls for the
current matching long entry.

Strategy-mode scripts can read `strategy.position_size` and
`strategy.position_avg_price`, `strategy.openprofit`, `strategy.netprofit`,
`strategy.grossprofit`, `strategy.grossloss`, `strategy.avg_trade`,
`strategy.avg_winning_trade`, `strategy.avg_losing_trade`,
`strategy.buy_and_hold_return_percent`, `strategy.max_runup`,
`strategy.max_drawdown`, and
`strategy.equity` as historical series floats. They can also read `strategy.closedtrades` and
`strategy.opentrades` as historical series ints in the current count-only
reporting subset. In the current long-only subset,
`strategy.position_size` is `0` when flat and positive while long.
`strategy.position_avg_price` is `na` when flat and the current average entry
price while long. `strategy.openprofit` is `(close - avg_price) * size` while
long and `0` when flat. `strategy.netprofit` sums realized closed-trade profit.
`strategy.grossprofit` sums only positive realized closed-trade profit, so
losing, flat, and current open trades do not change it.
`strategy.grossloss` sums realized closed-trade losses as positive values, so
winning, flat, and current open trades do not change it.
`strategy.buy_and_hold_return_percent` returns the current close's percentage
change from the first loaded bar close and returns `na` when that first close is
zero or non-finite.
`strategy.avg_trade` returns `strategy.netprofit / strategy.closedtrades` once
at least one trade is closed, and `na` before the first closed trade.
`strategy.avg_winning_trade` returns the average realized profit among winning
closed trades only, and `na` before the first winning closed trade.
`strategy.avg_losing_trade` returns the average realized loss among losing
closed trades only as a positive value, and `na` before the first losing closed
trade.
`strategy.max_drawdown` returns the maximum intrabar equity drawdown amount
over the current supported trading interval, using the supported entry equity,
the maximum equity before that entry, and the lowest low reached while the
supported position is open. It returns `0`
before any drawdown from the maximum equity baseline.
`strategy.max_drawdown_percent` divides the supported drawdown amount by entry
price times current supported position quantity and multiplies by 100.
`strategy.max_runup` returns the maximum intrabar equity run-up
amount over the current supported long-only trading interval, using the
supported entry equity, the minimum equity before that entry, and the highest
high reached while the supported position is open. `strategy.max_runup_percent`
divides the supported run-up amount by entry price times current supported
position quantity and multiplies by 100.
`strategy.equity` is cash plus current market value; without configured
commission this equals `initial_capital + strategy.netprofit +
strategy.openprofit`, and with supported commission it also includes entry
commission debits on open positions. Supported slippage changes entry and exit
fill prices, so realized/floating profit and equity use those adjusted fill
prices. `strategy.closedtrades` is the
number of closed trades recorded by the broker. `strategy.wintrades`,
`strategy.losstrades`, and
`strategy.eventrades` count closed trades whose realized profit is positive,
negative, or zero. `strategy.opentrades` is `1` while the supported long
position is open and `0` when flat. Supported `strategy.entry` and
`strategy.close` calls mutate broker state immediately, so later statements on
the same bar see the updated strategy state values. Pending `strategy.exit`
fills are evaluated after script statements on a historical bar, so script
reads observe the count changes on the next bar while public strategy output
and equity include the fill on the triggering bar. These variables can be used
in the same already-supported expression contexts as other series values,
including branches, switches, loops, pure UDF arguments, and constant history
references. Their history follows the normal per-expression series history
model. Direct mutation such as `strategy.position_size := ...` or
`strategy.closedtrades := ...` is rejected because strategy state variables are
read-only.
The first supported closed-trade namespace functions are
`strategy.closedtrades.entry_price(trade_num)`,
`strategy.closedtrades.exit_price(trade_num)`,
`strategy.closedtrades.entry_bar_index(trade_num)`, and
`strategy.closedtrades.exit_bar_index(trade_num)`. Stage 7 Slice 1 adds
`strategy.closedtrades.size(trade_num)` and
`strategy.closedtrades.profit(trade_num)`. Stage 7 Slice 2 adds
`strategy.closedtrades.entry_time(trade_num)` and
`strategy.closedtrades.exit_time(trade_num)`. Stage 7 Slice 3 adds
`strategy.closedtrades.commission(trade_num)`, which returns `0.0` without
configured commission and supported entry-plus-exit commission when configured.
Stage 7
Slice 4 adds `strategy.closedtrades.entry_id(trade_num)`, which returns the
entry id already retained on the closed trade record. Stage 7 Slice 5 adds
`strategy.closedtrades.exit_id(trade_num)`, which returns the close or
`strategy.exit` id retained on the closed trade record. Strategy trade comment
helpers add `strategy.closedtrades.entry_comment(trade_num)` and
`strategy.closedtrades.exit_comment(trade_num)` for commented fixture-backed
trades; invalid, out-of-range, or uncommented reads return `na`, and comments
remain internal script-visible metadata rather than public strategy JSON
fields. Stage 7 Slice 15 adds
`strategy.closedtrades.max_runup(trade_num)`, returning the largest high-based
favorable excursion retained for the selected closed trade quantity. The
current long-only closed-trade field subset reads fixture-backed pyramided
closed trades by zero-based index. Stage 7 Slice 16
adds `strategy.closedtrades.max_drawdown(trade_num)`, returning the largest
low-based adverse excursion retained for the selected closed trade quantity. Stage 7
Slice 17 adds cash-per-contract commission accounting for supported entries and
exits without adding public schema fields. Stage 7 Slice 18 adds cash-per-order
commission accounting under the same public contract. Stage 7 Slice 19 adds
fixed-tick slippage to supported long entry, close, and exit fill prices
without changing trigger conditions or public schema. Stage 7 Slice 20 adds
fixed-tick limit-order verification for supported long limit entry and
supported long limit/profit exit fills while preserving the original limit fill
price. Stage 7 Slice 21 adds percent commission accounting for supported
entry/exit fills under the same public contract. Stage 7 Slice 22 adds
`strategy.grossprofit` as a script-visible read-only series float over the
closed-trade list without changing public output shape. Stage 7 Slice 23 adds
`strategy.grossloss` under the same public-output contract. Stage 7 Slice 24
adds `strategy.avg_trade` under the same public-output contract. Stage 7 Slice
25 adds `strategy.avg_winning_trade` under the same public-output contract.
Stage 7 Slice 26 adds `strategy.avg_losing_trade` under the same public-output
contract. Stage 7 Slice 27 adds `strategy.max_drawdown` under the same
public-output contract. Stage 7 Slice 28 adds `strategy.max_runup` under the
same public-output contract. Stage 7 Slice 32 adds
`strategy.netprofit_percent`, `strategy.grossprofit_percent`, and
`strategy.grossloss_percent` under the same public-output contract, using
`amount / initial_capital * 100`. Stage 7 Slice 33 adds
`strategy.avg_trade_percent`, `strategy.avg_winning_trade_percent`, and
`strategy.avg_losing_trade_percent` under the same public-output contract,
averaging per-closed-trade percentage profit/loss values using each trade's
entry value as denominator. These state variables read the current closed-trade
list and are script-observable only through ordinary series outputs; they do
not add public runtime JSON, Python, or WASM fields. Stage 7 Slice 34 adds
`strategy.max_contracts_held_all`, `strategy.max_contracts_held_long`, and
`strategy.max_contracts_held_short` under the same public-output contract; in
the current long-only subset, `all` and `long` track the maximum filled long
entry quantity and `short` remains `0`. Closed/open trade namespace
functions read the current trade lists with a zero-based integer `trade_num`;
missing, negative, out-of-range, or non-integer indexes return `na`.
Stage 7 Slice 6 adds `strategy.opentrades.entry_price(trade_num)` for the
current supported long position. It returns the current open position average
entry price for `trade_num == 0`; when flat, out of range, negative, or
non-integer, it returns `na`. Stage 7 Slice 7 adds
`strategy.opentrades.entry_bar_index(trade_num)` under the same contract,
returning the current open position's entry fill bar. Stage 7 Slice 8 adds
`strategy.opentrades.entry_time(trade_num)`, returning the current open
position's entry fill timestamp. Stage 7 Slice 9 adds
`strategy.opentrades.size(trade_num)`, returning the current open position
size. Stage 7 Slice 10 adds `strategy.opentrades.profit(trade_num)`, returning
the current close-based floating profit for the current open position. Stage 7
Slice 11 adds `strategy.opentrades.entry_id(trade_num)`, returning the retained
entry id for that open position. Stage 7 Slice 12 adds
`strategy.opentrades.commission(trade_num)`, returning `0.0` for the selected
open trade without configured commission and the selected open trade's
supported entry commission when configured. Stage 7 Slice 13 adds
`strategy.opentrades.max_runup(trade_num)`, returning the largest high-based
favorable excursion seen so far for the selected open trade. Stage 7 Slice 14 adds
`strategy.opentrades.max_drawdown(trade_num)`, returning the largest low-based
adverse excursion seen so far for the selected open trade. Strategy trade comment
helpers add `strategy.opentrades.entry_comment(trade_num)` for commented
fixture-backed open trades. The current long-only open-trade field subset reads
fixture-backed pyramided open trades by zero-based index; invalid, out-of-range,
flat-state, or uncommented reads return `na`, with no public strategy JSON
expansion. Other open-trade namespace functions and public open-trade record
output remain unsupported.
Stage 7 Slice 35 adds `strategy.opentrades.capital_held` as the one variable
inside the open-trade namespace. In the current no-margin subset it returns
`na`; with explicit active `margin_long`, Stage 7 Margin Slice M2 returns the
current supported open long position's market value times `margin_long / 100`,
or `0.0` while flat. Stage 7 Margin Slice M3 applies the same active
`margin_long` account model to supported long entry affordability at the actual
fill price. Stage 7 Margin Slice M5 applies the long-only forced-liquidation
subset, so `capital_held` reflects the remaining open long position after a
margin call. Short margin behavior remains unsupported.
`strategy.margin_liquidation_price` uses the same supported long-only margin
account model. It solves the current broker equation
`equity_value(price) == position_size * price * margin_long / 100` and returns
`na` without active long margin, while flat, or for the unattainable
`margin_long=100` divisor. It does not round to `syminfo.mintick` yet and does
not expose a public margin schema field.

The strategy contract is host-independent and exposed consistently by CLI JSON,
Python dictionaries, and WASM JSON. Fixture-backed market-long
`strategy.order(id, strategy.long, qty=...)`, or omitted-qty long orders using
the configured default quantity, fill on the next historical bar open and can
add to an existing long position without using the `strategy.entry()`
pyramiding limit. Fixture-backed limit-long
`strategy.order(id, strategy.long, qty=..., limit=price)` orders use the
supported long limit timing model, fill at the verified limit price on a later
historical bar, and also bypass the `strategy.entry()` pyramiding limit;
omitted long `qty` uses the configured default quantity at placement time.
Fixture-backed stop-long
`strategy.order(id, strategy.long, qty=..., stop=price)` orders use the
supported long stop timing model, fill at the stop price on a later historical
bar, and also bypass the `strategy.entry()` pyramiding limit; omitted long
`qty` uses the configured default quantity at placement time.
Fixture-backed stop-limit-long
`strategy.order(id, strategy.long, qty=..., stop=stop_price, limit=limit_price)`
orders use the supported long stop-limit model: activation occurs on a later
historical bar when `high >= stop`, and the limit fill can occur only on a
subsequent historical bar when `low <= limit` or below the configured verified
limit threshold. They also bypass the `strategy.entry()` pyramiding limit;
omitted long `qty` uses the configured default quantity at placement time.
Fixture-backed reduce-only market
`strategy.order(id, strategy.short, qty=...)` orders can reduce an existing long
position on the next historical bar open, recording a `strategy.short` order
event and clamping oversized quantities without opening short exposure; while
flat, they are no-ops. Omitted `qty` remains unsupported for `strategy.short`.
The supported `strategy.order()` subset accepts
`comment`, `alert_message`, and `disable_alert` metadata. Supported long order
fills retain entry comments, reduce-only short fills retain exit comments, and
supported order-fill alert payloads are exposed under `strategy.alerts`; the
metadata does not widen unsupported order shapes. Short entries,
`strategy.exit` variants beyond the
supported single-trigger, one-downside/one-upside bracket, trailing-stop,
fixed-quantity, percent-quantity, explicit single-trigger or bracket/trailing
reservation subset, `strategy.cancel(id)`, and `strategy.cancel_all()`,
reversal/OCA `strategy.order` forms, short exposure, short price-based orders,
rich order families, strategy reporting helpers beyond the supported
position/profit/equity/count/run-up/drawdown/buy-and-hold return variables,
requested-context strategy state, strategy state mutation, and realtime
strategy handoff remain unsupported until later strategy-maintenance slices
define and fixture those semantics. Phase M
adds narrow stop-only `strategy.exit(id, from_entry, stop=price)` and limit-only
`strategy.exit(id, from_entry, limit=price)` subsets for the current
one-net-long broker. Phase N adds profit-only
`strategy.exit(id, from_entry, profit=ticks)` and loss-only
`strategy.exit(id, from_entry, loss=ticks)` helpers. Phase R adds the first
bracket subset: exactly one downside leg plus one upside leg, covering
`stop + limit`, `stop + profit`, `loss + limit`, and `loss + profit`. Phase S
adds the first trailing subset: exactly `trail_price + trail_offset` and
`trail_points + trail_offset`. Phase U adds optional fixed `qty`; Phase V adds
optional `qty_percent` on those same supported trigger shapes. Stage 4 accepts
`qty` and `qty_percent` together on those same supported trigger shapes, with
fixed `qty` determining the reserved or filled quantity. Quantity forms evaluate
once at placement time, must be finite and positive, and store an absolute
requested close quantity on the single pending exit. When only `qty_percent` is
used, it resolves against the current open position size as
`position_size * qty_percent / 100.0`, and values above 100 are allowed because
fills clamp to the current position size. Omitted `qty` and omitted
`qty_percent` keep the previous full-position behavior.

Profit/loss and trailing tick arguments convert positive tick distances from
`strategy.position_avg_price` using the fixed default `syminfo.mintick`, then
reuse the same pending-exit lifecycle: accepted calls are not eligible on the
bar where they are created or replaced, and a later historical bar with
`low <= stop/loss price` or `high >= limit/profit price` fills at the selected
exit price. Configured limit-order verification requires supported long
limit/profit exit fills to move the configured number of ticks beyond the
limit/profit price while still filling at the original limit/profit price.
Configured slippage worsens the supported long exit fill price after trigger
selection without changing trigger conditions. Single-trigger, bracket, and
trailing exits with explicit fixed
`qty` or `qty_percent` can keep multiple pending reservations for different
identities and share one reservation pool for the current matching long entry.
Same-side touched candidates fill in placement order. When downside
stop/loss/trailing candidates and upside limit/profit candidates are both
touched on the same eligible bar, only downside candidates fill on that bar in
placement order; opposite-side candidates remain pending if a long position
remains. If both legs of one bracket are touched on the same eligible bar, that
bracket contributes its downside stop/loss candidate. Omitted-quantity
full-position exits still use one-effective-pending replacement behavior across
supported single-trigger, bracket, and trailing forms, even when the
replacement uses a different `id + from_entry` identity. A later omitted
full-position exit clears earlier explicit fixed-`qty` or `qty_percent`
reservations for the current matching long entry. A trailing exit activates on
a later eligible bar when
`high >= activation_price`, sets its active stop to `high - offset_distance`,
and does not fill on the activation bar. On later bars, an active trailing exit
fills first when `low <= active_stop`; otherwise the active stop ratchets upward
and never decreases. When an inactive trailing reservation activates on the same
bar where an upside reservation fills, the trailing reservation persists its
active state and does not contribute a fill candidate until a later bar. A
filled exit appends exactly one `strategy.exit` order event with the absolute
filled quantity, records a closed trade under the source entry id for that
quantity, reduces or clears the current long position, and updates the normal
position/equity snapshots. Phase M, Phase N, Phase R, Phase S, Phase U, Phase V,
Phase W, Phase X, Phase Y, and Phase Z do not add public pending-order records,
reservation fields, remaining-quantity fields, percent fields, bracket-leg
metadata, trailing-state fields, activation fields, exit reason fields, or
top-level runtime schema fields.
Phase O does not add public
open-trade records. Stage 7 adds only script-visible closed-trade
`entry_id`, `exit_id`, `entry_price`, `exit_price`, `entry_bar_index`,
`exit_bar_index`, `entry_time`, `exit_time`, `commission`, `size`, `profit`,
`max_runup`, and `max_drawdown` namespace functions; it does not add public
trade-namespace fields. The prior Phase L
boundary is
summarized in
`docs/PHASE_L_AUDIT.md`; the closed Phase M and Phase N exit subsets are
summarized in `docs/PHASE_M_AUDIT.md` and `docs/PHASE_N_AUDIT.md`; the Phase R
bracket subset is summarized in `docs/PHASE_R_AUDIT.md`; the Phase U fixed
quantity subset is summarized in `docs/PHASE_U_AUDIT.md`; and the Phase V
percent quantity subset is summarized in `docs/PHASE_V_AUDIT.md`; the Phase X
bracket reservation subset is summarized in `docs/PHASE_X_AUDIT.md`; the Phase Y
trailing reservation subset is summarized in `docs/PHASE_Y_AUDIT.md`; the Phase
Z omitted-quantity boundary is summarized in `docs/PHASE_Z_AUDIT.md`.

## Alert Events

`alertcondition(condition, title, message)` is a supported runtime side effect
for a narrow declarative subset. `condition` accepts bool-compatible values;
`false` and `na` do not emit an event. `title` and `message` must be const
strings. Runtime output serializes `title` as the alert event `source` and
`message` as the alert event `message` after replacing `{{open}}`, `{{high}}`,
`{{low}}`, `{{close}}`, and `{{volume}}` with triggering-bar values, plus
`{{ticker}}`, `{{interval}}`, and `{{exchange}}` with current chart metadata,
and `{{time}}` with the triggering bar timestamp using the UTC
`str.format_time` default format.

`alert(message, freq?)` is supported for string-compatible dynamic messages. It
serializes `source` as `alert`. The default frequency is
`alert.freq_once_per_bar`, which emits at most one event per alert callsite per
bar even if a loop reaches the same callsite multiple times. `alert.freq_all`
emits every reached call. `alert.freq_once_per_bar_close` emits at most one
event per alert callsite only when execution is for a historical bar or a
confirmed realtime bar update; forming realtime updates do not expose or commit
close-frequency alert events. TradingView-style `{{...}}` placeholder
interpolation outside supported `alertcondition` message placeholders and
other alert frequency values remain unsupported.

Alert conditions execute like ordinary reached statements in global flow,
including supported `if`, `switch`, `for`, and `while` bodies. This is a
deliberate runtime-event model for the current subset, not a global-only
declaration model. Multiple triggering alert sites on the same bar are emitted
in program order and use deterministic callsite ids.

Realtime forming updates expose alert events for the current forming result,
but those events are part of the forming runtime snapshot. A later forming
update starts again from the confirmed snapshot, so abandoned forming alert
events disappear. Only historical and confirmed updates become part of the
confirmed result.

Alert side effects inside user-defined functions, user-defined function
arguments, and requested-context expressions are rejected under the same
side-effect boundary as output calls, drawing calls, input declarations, and
array mutation.

## Libraries And Imports

Imports are resolved before runtime execution. Hosts may provide exact-key
library source text to semantic analysis; the core builds a deterministic
source graph, validates library declarations and aliases, then lowers the root
program. Exported const expressions are inlined into the root, and exported pure
functions lower through the same inlined UDF machinery as local functions.
Runtime execution receives one lowered HIR program and performs no filesystem,
network, registry, or library lookup.

The executable import subset supports scalar-tree imported UDT constructors,
direct and nested field reads, ordinary same-imported-UDT reassignment, and scalar-tree
imported UDT typed declarations, same-imported-identity ternary results, plus
same-imported-identity `if`, `switch`, `while`, and `for` expression results,
direct or nested UDF parameter passthrough and direct or nested constructor-return
results, ordinary imported UDT `var` declarations, and scalar-tree root-field replacement in
top-level, branch, `for`-loop, `while`-loop, and UDF-local statement contexts,
by lowering them to ordinary UDT field-vector values with source-scoped HIR
identity. It also supports scalar-tree imported UDT `varip` declarations
through the same intrabar value-clone slot model as local scalar-tree UDT
`varip`, plus receiver-style or alias-qualified scalar-tree imported UDT method calls
including direct same-identity, block-local alias, final-if alias, final-for
alias, final-while alias, switch-expression alias, nested-method passthrough
plus constructor returns, and method-local
scalar-tree root-field replacement, plus scalar-tree imported UDT value history and `array.from`
size/get/first/last, set replacement field reads, push append field reads,
unshift prepend field reads, insert insertion field reads, fill replacement
field reads, join positional stringification, includes/indexof/lastindexof
structural equality search, sort/sort_indices by int/float/string sort_field,
pop/remove/shift return field reads, clear size reset, copy independent field
reads, reverse reordered field reads, slice window field reads, concat appended
field reads, and statement/expression/index-value for-in value-copy field reads.
It intentionally excludes remote lookup, re-exports, unaliased imports,
side-effecting exported functions, imported UDT flow outside the covered same-identity scalar-tree paths, collections,
direct private imported UDT access and imported UDT value history outside the scalar-tree metadata subset, and alias-qualified imported method receiver type
mismatches.

## Variables

### Normal Declarations

Normal declarations are evaluated when their scope is evaluated on each bar.

```pine
x = close + open
```

`x` receives a new current value on every bar where the declaration executes.
The value can be committed into its series history after the bar execution.
Normal and tuple declarations inside `if` blocks are scoped to the branch. If
the branch is skipped, branch-local series slots commit `na` for that bar.

When an `if` is used as a declaration value, both branches must end with a
value-producing expression. Void side-effect calls such as `alert()` are not
valid branch results, and neither are trailing declarations or reassignment
statements.

For v6 scripts, `and` and `or` use lazy evaluation: the right operand is skipped
when the left operand already determines the result. Earlier-version scripts
keep strict operand evaluation for this runtime's legacy subset.

`for i = start to end` loops over an inclusive integer range. The runtime
increments when `start <= end` and decrements when `start > end`. An explicit
non-zero int `by step` supplies the absolute step magnitude; the sign of `step`
does not override the range direction. `start` and `step` are evaluated once
when the loop statement is reached on a bar. In v6 scripts, `end` is
re-evaluated before each iteration; earlier-version scripts evaluate `end` once
when the loop is reached. If `start`, `end`, or `step` evaluates to `na`, the
loop body is skipped. The counter is scoped to the loop body. `break` exits the
nearest enclosing loop. `continue` skips the rest of the current iteration and
advances to the next loop counter value.

When a `for` loop is used as a declaration value, the loop body must end with a
value-producing expression. Void side-effect calls such as `alert()` are not
valid loop results. The loop returns the last value produced by the final
expression. A trailing `break` or `continue` statement is not a result
expression, and neither are trailing declarations or reassignment statements.
If a `continue` skips the expression or a `break` exits before it, the previous
produced value remains the loop result. If no iteration reaches the expression,
the loop result is `na`.

Statement-form `for value in values` currently supports only `array<int>`,
`array<float>`, `array<bool>`, `array<string>`, and `array<color>` iterables.
The iterable expression is evaluated once, the initial array length is captured,
and the runtime visits indexes `0..initial_len`. Each element is read from
current array storage when its index is reached and assigned by value to the
loop-local variable. Empty arrays and typed `na` array iterables execute zero
iterations. `break` exits the nearest enclosing loop, `continue` skips the rest
of the current iteration, loop-body local declarations are scoped like other
loop bodies, and stateful built-in calls in the body advance at each reached
iteration for that callsite. `break` or `continue` outside a loop is rejected
with `E_LOOP_CONTROL`. Mutations through any alias affect not-yet-visited
existing indexes. Appended elements are not visited in the current loop. If the
array shrinks so a future initial index is out of bounds, execution raises the
same runtime error used by `array.get`. Label, line, linefill, polyline, box,
and table array loop values are shallow-copied ids, so drawing setters or lifecycle
operations through the loop local mutate the same drawing object while
assignment to the loop local does not write the source array slot. Chart-point
array and same-local or same-imported scalar-tree UDT array loop values are
copied into the loop-local variable, so local field mutation does not write back
to the source slot. The narrow `for index, value in values` form supports `array<int>`,
`array<float>`, `array<bool>`, `array<string>`, `array<color>`,
`array<label>`, `array<line>`, `array<linefill>`, `array<polyline>`,
`array<box>`, `array<table>`, `array<chart.point>`, and same-local or
same-imported scalar-tree UDT array iterables with a zero-based `series int`
index loop-local. Statement-form `for...in` also supports runtime-owned
`matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and
`matrix<color>` iterables by capturing matrix rows at loop entry and binding
each loop value to an independent row snapshot array. The optional index/value
form exposes the zero-based row number as a `series int`; later matrix shape
changes inside the loop body do not change the rows visited during that
execution. Expression-form `for value in values` supports `array<int>`,
`array<float>`, `array<bool>`, `array<string>`, `array<color>`,
`array<label>`, `array<line>`, `array<linefill>`, `array<polyline>`,
`array<box>`, `array<table>`, `array<chart.point>`, and same-local scalar-field
UDT array iterables, plus runtime-owned matrix row iterables and scalar map
iterables, and returns the last expression from the last completed iteration, or
`na` when the iterable is typed `na` or has zero rows/elements. Matrix
expression-form iteration uses the same independent row snapshots as
statement-form matrix iteration. For scalar maps, key-only forms bind the map key
to the loop value, while key/value forms bind the key to the first loop local and
the map value to the second loop local. The optional expression-form index local
is the same zero-based `series int` slot number used by statement-form
index/value iteration for arrays and matrix rows, and the map key for scalar
maps. `break` returns the previous expression result and `continue` skips the
current iteration's result expression. Index/value iteration over typed imported
or non-scalar-tree UDT arrays, expression-form `for...in` beyond the
scalar-array, drawing-id-array, chart.point-array, same-local scalar-tree
UDT-array, matrix-row, and scalar-map subset, non-array/non-matrix/non-map
iterables, and other non-scalar arrays remain outside the current subset.
When the iterable is a same-local scalar-tree UDT-array parameter of a local
UDF or typed local user method, lowering binds the value loop local with the
element identity resolved for that call. Value-only and index/value statement
loops, block-local aliases, and final expression-form loops may return a field
or other scalar result, the UDT element itself, or a same-identity UDT array
rebuilt from that element. Interleaved A-to-B-to-A UDF calls and named
typed-method arguments retain independent field layouts, returned-element
identities, and rebuilt-array identities.
Ordinary `var` scalar arrays roll back
loop-body mutation during repeated forming realtime updates, while scalar
typed-array `varip` iteration preserves carried intrabar loop-body mutation
between repeated forming updates. The scalar-array runtime fixtures,
label-array, line-array, linefill-array, polyline-array, box-array, and
table-array shallow-id fixtures, chart-point-array value-copy fixture, and
UDT-array value-copy fixture have explicit execution parity with full historical
recomputation.

`while condition` evaluates the condition before each iteration. A `true`
condition executes the body, while `false` or `na` exits the loop. `break`
exits the nearest enclosing loop. `continue` skips the remaining body statements
and re-evaluates the condition. Runtime execution enforces a maximum iteration
guard per while evaluation so non-terminating scripts fail instead of
hanging execution. `while` bodies follow ordinary statement expression rules,
including fixture-backed history reads and pure UDF calls. Scalar `while`
expressions return the latest reached final body expression, or `na` if no
iteration produces a value. Stateful callsites in reached expression bodies
advance per reached iteration, and body-local declarations including local
`var` declaration sites follow loop-local storage rules. When a while
expression is evaluated inside an outer loop, `break` and `continue` in the
expression body are consumed by the nearest while expression and do not control
the outer loop. Tuple declaration/destructuring from while-expression results
uses the same latest-produced-result rule. A while expression may return a
scalar array result, including a fresh array or an existing scalar-array alias;
callers may read or mutate that returned array with the supported scalar-array
APIs. Scalar-array zero-iteration `na`, result preservation across
break/continue, and fresh historical copies from committed history reads are
fixture-backed. While expressions can also return runtime-owned
`matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and
`matrix<color>` results; callers may read or mutate those returned matrix ids
with the supported matrix APIs. Nested collection interactions through
while-expression results remain outside this executable subset, and nested-array
while-expression results are rejected before runtime execution.
Same-imported-identity UDT while-expression results are supported;
local/imported mismatches are rejected before runtime execution.

`switch` expressions evaluate arms in source order. Selector-form switches
evaluate the selector once per bar, then compare each case expression with that
selector value. Selector-less switches evaluate each arm condition until one is
`true`. Only the selected result expression executes. For supported
statement-block arms, only the selected block's statements execute and its final
expression becomes the arm result. If no arm matches and no default arm exists,
the switch returns `na`. Same-imported-identity UDT switch results are
supported; imported/local or otherwise mismatched UDT switch results are
rejected before runtime execution.
Statement-context `switch` forms use the same arm-selection rules, execute only
the selected arm, and can perform side effects, outer reassignment, or propagate
loop control without requiring a final result expression.

### Reassignment

```pine
x := x + 1
```

Reassignment updates an existing variable in the current execution scope. The
semantic analyzer must reject reassignment to unknown names.

### `var`

```pine
var x = 0
```

`var` declarations initialize once, then preserve state across bars. A `var`
inside a local block initializes the first time that declaration site is reached,
then reuses the same persistent slot on later executions of the block. Inlined
user-defined function bodies allocate independent persistent slots for each
syntactic callsite. The runtime stores this state separately from ordinary
per-bar locals.

For arrays, the stored value is a runtime-owned array id. A normal
`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`,
`array.new_color`, `array.new_label`, `array.new_line`,
`array.new_linefill`, `array.new_box`, or `array.new_table` declaration
allocates a fresh array each time it executes. `array.new_label`,
`array.new_line`, `array.new_linefill`, `array.new_box`, and
`array.new_table` create drawing-id arrays filled with `na` when no initial id
is supplied.
`array.from` also allocates a fresh inferred typed array and requires at least
one non-`na` supported typed value, including label, line, linefill, box, and
table ids for drawing-id arrays. A `var` array declaration keeps the same id
and backing storage across bars, so mutations such as `array.push` or
`values.push(...)` persist.
Assigning an array to another variable copies the id, not the backing values;
mutating either name mutates the same runtime-owned array. Passing an array to a
user-defined function also passes the array id, so side-effect-free helpers can
read the same backing values through parameters. Array mutation inside
user-defined functions remains outside the executable subset. Top-level
branches and loops mutate the same array id they can read after control flow
continues. `array.copy` and `values.copy()` allocate a new array id initialized
with the source array's current element values. For label-id, line-id, box-id,
and table-id arrays, copied elements still reference the same drawing objects;
only the array container is independent. Realtime forming-bar rollback clones the confirmed
runtime store before executing a forming update, so array mutations and copies
made during a forming update do not leak into the confirmed store until a
confirmed update is committed.

For local user-defined types, `Type.new(...)` creates a runtime value containing
the supported scalar field values in declaration order. Field reads return the
stored scalar value. Normal declarations allocate a fresh UDT value when
reached on each bar; `var` UDT declarations preserve the last confirmed UDT
value across bars and roll back during realtime forming updates like other
ordinary `var` values, including typed `var` declarations initialized from
`na`, same-UDT constructors, same-UDT ternary expressions, same-UDT switch
expressions, same-UDT `if` expressions, or same-UDT `for` expressions. Local
scalar-field reassignment evaluates the right-hand expression, replaces that
field in the current UDT value, and writes the updated value back to the
receiver symbol, including the receiver's persistent slot when applicable.
Local `for` expressions may construct a local UDT in their final body
expression, return the final iteration's UDT value, and allow the caller to
store that value and read its fields. Local `while` expressions may likewise
construct or alias a local UDT in their final body expression, return the
latest reached UDT value, and allow the caller to store that value and read its
fields. Top-level, block-local, and loop-local typed UDT declarations may also
initialize from same-local-UDT ternary, switch, or `if` expressions and later
reassign to the same local UDT type; top-level, block-local, and loop-local
typed UDT declarations may also initialize or reassign from same-local-UDT
`for` expressions. UDFs may pass a local UDT value through a parameter and
return that same parameter, or return a
block-local, ternary-expression, final-if, final-for, final-for-in, final-while, or switch-expression alias chain
that starts from that parameter, or return a nested passthrough UDF call that
maps back to that parameter through those same alias forms. Pure UDFs may also
construct and return a local UDT, directly,
through nested pure constructor-helper UDF calls, or through same-local-UDT
ternary, switch, `if` expression, final if/else constructor branches, or final
for bodies, from local UDT parameter scalar fields, scalar fields read through
block-local UDT aliases of those parameters, block-local scalar aliases of
those fields, scalar parameters whose types are inferred from the call
arguments, or block-local scalar aliases of those scalar parameters, using
positional or named constructor field arguments.
Positional and named UDF call arguments both preserve the parameter identity.
The caller may then store the returned value and read its fields. UDF-local
UDT variables may mutate scalar fields before returning the updated value.
Mutating UDT fields on globals or parameters inside UDFs, field mutation inside
methods, local UDT value history references, UDT `varip`, nested UDT fields, UDT
arrays, and broader imported UDT values are rejected before runtime execution or
remain outside the executable subset. Imported UDTs are executable for
host-provided exported scalar-tree types constructed as `alias.Type.new(...)`
read through direct scalar fields such as `value.x` or nested fields such as
`wrapped.point.x`, and reassigned from the
same imported UDT identity, including explicit `alias.Type` typed declarations
with same-identity initialization or reassignment, same-imported-identity
ternary, `if`, `switch`, `while`, and `for` expression results, direct or nested UDF
parameter passthrough returns, direct or nested constructor-return UDFs, and
scalar-tree root-field replacement in top-level, branch, `for`-loop, `while`-loop, and
UDF-local statement contexts, plus scalar-tree value history reads and
`array.from` size/get/first/last, set replacement field reads,
push append field reads, unshift prepend field reads, insert insertion field reads, fill replacement field reads, join positional stringification, includes/indexof/lastindexof structural equality search, sort/sort_indices by int/float/string sort_field, pop/remove/shift return field reads, clear size reset,
copy independent field reads, reverse reordered field reads, slice window field
reads, concat appended field reads, and statement/expression/index-value for-in
value-copy field reads, plus `array<lib.Type>`/`lib.Type[]` declarations
initialized from `na` or same-identity imported UDT array values.
For both local and imported scalar-tree UDT arrays, ternary, `if`, `switch`,
`for`, `for...in`, and `while` results preserve the concrete element identity
through HIR lowering. Array/`na` branches are allowed, while branches carrying
different UDT identities fail semantic analysis.
Local pure UDF/local user-method calls preserve same-local scalar-tree UDT array
identity, and imported pure exported UDF/imported user-method calls preserve
same-imported scalar-tree UDT array identity, through direct parameter,
block-alias, copy, new/from, private nested-call, typed-method, and final
control-flow returns. A direct or alias return keeps the source array id;
`array.copy`, `array.new<T>`/`array.new<alias.Type>`, and `array.from` allocate a
new array id. Lowering derives the element layout from the current call
arguments, rewrites imported type positions for the active alias, and keys
expression metadata by import instance. Interleaved calls over field-order
variants A, B, then A, including two aliases of one physical library, therefore
read the correct fields on every result. Tuple-return lowering applies the same
rule independently to each destructured slot, including tuple literals,
direct/block/nested/final-flow UDF or method results, typed-`na` locals, and
different UDT array identities in different slots. Fresh lower symbols receive
the current call's slot identity before later array element field reads.
Tuple-valued ordinary declarations preserve their element types and per-slot
identities through direct and self aliases, ternary, `if`, or `switch` aliases,
fresh local shadowing, and later tuple destructuring. The first declaration
fixes each UDT-array slot identity; same-identity or `na` reassignment keeps the
existing layout, while direct or control-flow reassignment to a different
identity is rejected before HIR emission. Qualified imported UDF/method results
with a concrete same-imported scalar-tree UDT-array identity support direct
`.first()` and `.copy()` calls, including `.copy().first()`. The postfix
receiver is lowered as an array helper rather than as a same-named imported
function, and `copy` remains independent from the source array. Mixed
scalar-return identities, non-scalar imported returns, same-local or other
direct call-result array methods, and mutation through unsupported UDF/method
side-effect contexts remain unsupported boundaries.
Both caller-side `for...in` over returned arrays and in-callee `for...in` over
generic same-local scalar-tree UDT-array parameters preserve concrete identity,
including final expression results that return the loop element itself or use
it to rebuild a same-identity UDT array.
Scalar-field imported UDT `varip` declarations may persist the same imported
identity by value across forming updates.
Local/imported structural lookalikes are distinct assignment identities;
imported UDT collections beyond the fixture-backed same-scalar-tree call-return
subset above, scalar-tree `array.from`, and `array<lib.Type>`/`lib.Type[]`
size/get/first/last, set-replacement, push-append,
unshift-prepend, insert-insertion, fill-replacement, join-stringification,
search-structural-equality, sort-by-field, pop/remove/shift return, clear-size,
copy-read, reverse-read, slice-window, concat-append, and for-in-value-copy
subset,
nested field mutation, direct private imported UDT access and imported UDT value history outside the scalar-tree metadata subset, UDF
parameter/global field side effects, and method
receiver/parameter/global field side effects remain unsupported.

Pure local UDT methods execute as receiver functions. The receiver value is
passed as the first internal argument and the method body is evaluated through
the same lowered expression path as a local UDF body. A pure method may accept
additional local UDT parameters and return the receiver itself, a block-local
alias chain that starts from the receiver or another local UDT parameter,
ternary-expression aliases, final if/else, final for, final while, or
switch-expression local UDT aliases of those values, another local UDT
parameter directly or through a nested method passthrough call, or construct
and return a local UDT directly, through nested pure constructor-helper UDF
calls, or through same-local-UDT ternary, switch, `if` expression, final
if/else constructor branches, or final for bodies. Supported method
constructors may read receiver or local UDT parameter scalar fields, scalar
fields through block-local receiver or local UDT parameter aliases, block-local
scalar aliases of those fields, inferred scalar parameters, or block-local
scalar aliases of those parameters, using positional or named constructor field
arguments; the caller may store that returned UDT value and read its fields.
Receiver-style and alias-qualified scalar-tree imported UDT methods may also return
the receiver or a same-identity UDT parameter directly, through block-local,
ternary-expression, final-if, final-for, final-while, or switch-expression
aliases, and preserve that imported identity for caller-side field reads, or
directly, through a nested method call, or through a ternary expression
construct and return the same imported UDT identity for caller-side field reads.
For local methods only, a typed same-local scalar-tree UDT array parameter may
be returned directly or through block aliases, copies, fresh constructors,
nested local calls, and final control flow with call-specific identity. The
caller must bind same-local results or use namespace-form array helpers.
Parser-normalized qualified calls returning a same-imported scalar-tree UDT
array additionally support direct `.first()`/`.copy()`; unqualified local UDF
results and other array methods remain parser/semantic boundaries.
Method side effects, recursive methods, unsupported parameter families,
mismatched UDT parameter identity, unknown receivers, and alias-qualified
imported method receiver type mismatches are rejected during semantic analysis.

### `varip`

```pine
varip ticks = 0
```

The current executable `varip` subset supports global and local scalar
`int`/`float`/`bool`/`string`/`color`/`na` declarations plus scalar typed-array
ids for float, int, bool, string, and color arrays. It also supports explicitly
typed same-local scalar-tree UDT values initialized from `na`, same-UDT
constructors, same-identity aliases, or fixture-backed same-UDT
ternary/switch/if/for/for-in/while expressions, including nested same-local
scalar-tree Wrapper values, plus direct-constructor-inferred
or direct-alias-inferred same-local scalar-tree UDT values such as
`varip p = Point.new(close)` or `varip p = existingPoint`, and scalar-tree
imported UDT values initialized from `na`, same-imported constructors,
same-imported aliases, same-imported ternary/switch/if/for/for-in/while
expressions, including nested same-imported scalar-tree Wrapper values, direct
constructor inference, or direct same-imported alias inference such as
`varip p = lib.Point.new(close)`, same-local scalar-tree UDT array ids,
runtime-owned scalar maps, and
`matrix<float>`/`matrix<int>`/`matrix<bool>`/`matrix<string>`/`matrix<color>`
ids. Local scalar declaration sites
inside `if`, `for`, `while`, and user-defined function bodies use the same
declaration-site storage model as local `var`; each lowered scalar UDF callsite
gets independent storage.
Historical execution treats this subset like `var`: the declaration initializes
once when first reached and reassignment persists across committed bars.

Realtime forming-bar execution differs from ordinary `var`. A first forming
update for a bar starts from the last confirmed runtime state. Repeated forming
updates for that same bar carry `varip` slots forward from the previous forming
update. When a carried `varip` value is a supported array id, the referenced
backing array contents, element kind, and UDT element metadata are copied from
the previous forming runtime as well. Supported map and matrix ids copy their previous forming
backing stores and advance the next id counters past retained ids. Ordinary
`var`, outputs, non-`varip` arrays/maps/matrices, drawing objects, request
caches, callsite state, and history reads continue to roll back to the confirmed
baseline. A confirmed update also seeds from the latest forming `varip` values
before executing and then commits the resulting values into the confirmed
runtime for the next bar.
Committed and realtime forming history reads from supported UDT `varip` values
use the same confirmed-history baseline, including representative same-local and
same-imported nested scalar-tree Wrapper values initialized from ternary
expressions. Single `chart.point` value `varip` history reads use that same
confirmed-history baseline for constant and dynamic offsets.

Skipped local declaration sites do not initialize before their first executed
reach. Assigning a `varip` scalar typed-array slot to another variable preserves
the same array id, so mutations through either name update the same backing
store. `array.copy` returns an independent array id, and a `varip` slot that is
reassigned to that copy retains the copied backing store across repeated forming
updates without aliasing the source. A carried UDT `varip` value is cloned by
value, and scalar field mutation writes the updated field vector back to the
`varip` slot. Array mutation inside UDFs remains rejected by the existing
function side-effect rules. Drawing object ids are rejected for `varip`:
retaining only the id would be unsafe while label, line, box, and table object
stores continue to roll back between forming updates. Tuples,
non-constructor-inferred UDT `varip`, nested-field UDT `varip`, non-scalar
UDT array `varip`, and other value families remain unsupported until
their declaration-site, backing-store, identity, and rollback rules are
explicitly designed.

Array bounds are stable in the current subset: `array.get`, `array.set`,
`array.insert`, and `array.remove` support negative indexes from the array end.
Indexes outside the current positive or negative bounds are runtime errors.
Positive `array.insert` at `size` appends; greater-than-size insert indexes are
runtime errors.
`array.pop` or `array.shift` on an empty array returns `na`. `array.first` and
`array.last` also return `na` for empty arrays.
`array.fill` replaces all elements by default, or a half-open `[index_from,
index_to)` window when bounds are supplied; invalid ranges are ignored.
`array.includes`, `array.indexof`, and `array.lastindexof` use structural
equality for same-local scalar-tree UDT arrays and same-imported scalar-tree
UDT arrays constructed through `array.from`; `array.indexof` and
`array.lastindexof` return `-1` when no matching value is present. Numeric
binary search helpers expect int/float arrays sorted ascending;
`array.binary_search` returns `-1` when not found, while leftmost/rightmost
return the nearest existing insertion-side index and return `-1` for empty
arrays. `array.every` and `array.some` operate on float/int/bool arrays only:
false, zero, and `na` elements are falsey, other numeric values are truthy,
empty arrays return `true` for `every` and `false` for `some`. Numeric helpers
on int/float arrays skip `na` elements; if every
element is `na` or the array is empty, `array.min`, `array.max`, `array.sum`,
`array.avg`, `array.range`, `array.median`, `array.mode`, `array.variance`,
and `array.stdev` return `na`. `array.range` returns max minus min.
`array.abs` returns a new same-kind int/float array, preserves `na` elements,
and does not mutate the source array.
`array.median` returns the median of non-`na` values. `array.mode` returns the
smallest value among tied most-frequent values and returns `na` when all
remaining values occur only once. Percentile helpers operate on non-`na` values
sorted ascending. Percentages outside `0..=100`, empty/all-`na` arrays, and
invalid `array.percentrank` indexes return `na`.
`array.covariance` requires same-size int/float arrays, skips pairs where
either side is `na`, and returns `na` for mismatched sizes, no numeric pairs,
or unbiased calculations with fewer than two numeric pairs.
`array.standardize` returns a new float array using non-`na` values to compute
mean and population standard deviation. Empty/all-`na` arrays return an empty
array; otherwise `na` element positions are preserved.
`array.variance` and `array.stdev` use a
biased population estimate by default; with `biased=false`, they use the
sample denominator and return `na` when fewer than two numeric values remain.
`array.sort` orders int/float/string arrays in place and supports same-local
scalar-tree UDT arrays plus same-imported scalar-tree UDT arrays constructed
through `array.from` when a compile-time `sort_field` names an int, float, or
string field. It sorts ascending by default and accepts `order.ascending` or
`order.descending`. `na` values and empty string elements sort last in ascending
order and first in descending order. `array.sort_indices` returns a new int
array containing the source indexes in sorted order, follows the same order and
special-value rules, and leaves the source array unchanged. `array.reverse`
reverses any supported typed array in place.
`array.join` converts supported scalar array elements and fixture-backed
same-local or same-imported scalar-tree UDT array elements to string with the
default numeric format, uses `,` as the default separator, and returns an empty
string for empty arrays. Color elements render as normalized integer color
values. Label and line arrays are intentionally not accepted by
`array.join` or array string conversion in this subset. Joined results over 40,960 characters
are runtime errors.
`array.slice` returns a same-kind shallow window over the parent array's
half-open `[index_from, index_to)` range; reads and writes through the slice
mirror the parent window, inserting through the slice widens that window and
inserts into the parent, negative/reversed/out-of-range creation bounds return
`na`, and later parent mutations that move the window out of bounds are runtime
errors.
`array.concat` appends the second same-kind array to the first array in place
and returns the first array id. A negative array size is a runtime error.
Runtime execution limits each supported array to 100,000 elements; oversized
creation, `array.push`, `array.unshift`, `array.insert`, and `array.concat`
beyond the limit return runtime errors.

Read-only array operations are allowed inside inlined user-defined functions.
The supported method-call syntax lowers to the same `array.*` runtime calls, so
the same bounds, persistence, and UDF side-effect rules apply.
Array mutation, including push/pop/shift/unshift/set/sort/reverse/clear, inside
user-defined functions is rejected as a function side-effect boundary.

For maps, the stored value is a runtime-owned map id. The current runtime
subset supports `map.new<K, V>()` for empty maps whose key and value templates
are `int`, `float`, `bool`, `string`, or `color`, `map.size(id)` as a
current-entry-count helper, and namespace-call `map.put`, `map.get`,
`map.contains`, `map.clear`, `map.remove`, and `map.copy` over those scalar
templates. Map ids own an insertion-ordered entry list; equal keys replace
existing values, missing `map.get` calls return `na`, `map.contains` reports key
presence, `map.clear` empties the entry list while keeping the id reusable, and
`map.remove` deletes the matching key or no-ops when the key is absent.
Assigning a map to another variable copies the id, not the backing store;
`map.copy` clones the current backing store into an independent map id.
`map.keys` and `map.values` return independent array snapshots in insertion
order. `map.put_all(target, source)` mutates the target map by iterating source
entries in insertion order; equal keys replace existing values without moving
target order, and new keys append.
Realtime forming updates start from the confirmed runtime clone, so ordinary
map mutations roll back with the rest of non-`varip` runtime state. Public JSON
output and Python conversion serialize map values as `null`/`None` at output
boundaries. Equivalent method aliases lower to the same supported namespace
calls. Scalar `map<K,V>` typed declarations store the declared key/value
template in semantic metadata and accept `na` or compatible map ids. Scalar map
history reads return independent copies of committed map snapshots, so mutating
the historical id does not mutate the current map or the retained history.
Scalar map `varip` slots retain their map ids and backing stores across repeated
realtime forming updates. Read-only map helpers can consume map ids passed
through user-defined function parameters when semantic analysis can carry the
caller's scalar map template to the parameter. Direct scalar-map `for...in`
iteration visits entries in insertion order; a single loop variable receives the
key, while `[key, value]` receives the key and value. Bare map declarations and
non-scalar map templates remain unsupported.

For matrices, the stored value is a runtime-owned matrix id. The current
runtime subset supports `matrix.new<float>`, `matrix.new<int>`,
`matrix.new<bool>`, `matrix.get`, `matrix.set`,
`matrix.fill`, `matrix.copy`, `matrix.transpose`, `matrix.reverse`,
`matrix.reshape`, `matrix.kron`, `matrix.mult`, `matrix.diff`, `matrix.pow`,
`matrix.add_row`,
`matrix.add_col`, `matrix.remove_row`, `matrix.remove_col`,
`matrix.swap_rows`, `matrix.swap_columns`, `matrix.sort`,
`matrix.submatrix`, `matrix.rows`,
`matrix.columns`, `matrix.elements_count`, `matrix.is_square`,
`matrix.is_binary`, `matrix.is_diagonal`, `matrix.is_identity`,
`matrix.is_symmetric`, `matrix.is_antisymmetric`, `matrix.is_stochastic`,
`matrix.is_zero`, `matrix.sum`, `matrix.avg`, `matrix.min`, `matrix.max`,
`matrix.mode`, `matrix.trace`, `matrix.det`, `matrix.eigenvalues`,
`matrix.eigenvectors`, `matrix.inv`, `matrix.pinv`, `matrix.rank`,
`matrix.row`, and `matrix.col`.
`matrix.new<float>` allocates a fresh rectangular store when executed, fills
cells with the optional numeric/`na` initial value, and coerces int cell values
to float. `matrix.new<int>` allocates a fresh int matrix store with an optional
int-compatible/`na` initial value and is supported with `matrix.get`,
`matrix.set`, `matrix.fill`, `matrix.copy`, `matrix.transpose`,
`matrix.reverse`, `matrix.reshape`, `matrix.submatrix`, `matrix.row`,
`matrix.col`, `matrix.kron`, `matrix.mult`, `matrix.diff`, `matrix.pow`,
`matrix.add_row`, `matrix.add_col`, `matrix.remove_row`, `matrix.remove_col`,
`matrix.swap_rows`, `matrix.swap_columns`, `matrix.sort`, `matrix.rows`,
`matrix.columns`, `matrix.elements_count`, and `matrix.is_square`,
`matrix.is_binary`, `matrix.is_diagonal`,
`matrix.is_identity`, `matrix.is_symmetric`, `matrix.is_antisymmetric`,
`matrix.is_stochastic`, `matrix.is_zero`, `matrix.sum`, `matrix.avg`,
`matrix.min`, `matrix.max`, `matrix.mode`, `matrix.trace`, `matrix.det`,
`matrix.eigenvalues`, `matrix.eigenvectors`, `matrix.inv`, `matrix.pinv`, and
`matrix.rank`, including supported method
aliases. `matrix.new<bool>` allocates a fresh bool matrix store with an
optional bool-compatible/`na` initial value and is supported with structural
helpers: `matrix.get`, `matrix.set`, `matrix.fill`, `matrix.copy`,
`matrix.transpose`, `matrix.reverse`, `matrix.reshape`, `matrix.submatrix`,
`matrix.row`, `matrix.col`, `matrix.add_row`, `matrix.add_col`,
`matrix.remove_row`, `matrix.remove_col`, `matrix.swap_rows`,
`matrix.swap_columns`, `matrix.rows`, `matrix.columns`,
`matrix.elements_count`, and `matrix.is_square`, including matching supported
method aliases. Bool matrix writes through `matrix.set`/`matrix.fill` accept
bool or `na` values. Int matrix writes through `matrix.set`/`matrix.fill`
accept int or `na` values. Assigning a matrix to another variable copies the id, not the backing
cells; mutating either name with `matrix.set` or `matrix.fill` mutates the same
runtime-owned matrix. `matrix.copy` allocates an independent store with the
current cells. `matrix.transpose` allocates an independent store with swapped
row and column counts. `matrix.reverse` mutates the existing store in place by
reversing row-major cells, which moves `(row, column)` to
`(rows - 1 - row, columns - 1 - column)`. Ordinary `var` matrix ids persist
across bars, and realtime
forming-bar rollback clones the confirmed matrix store before re-executing
non-`varip` updates. Row and column counts must be non-negative, total cell
count is capped at 100,000, and out-of-bounds row or column access is a runtime
error.
`matrix.set`, `matrix.fill`, `matrix.reverse`, `matrix.reshape`, `matrix.add_row`,
`matrix.add_col`, `matrix.remove_row`, `matrix.remove_col`, and
`matrix.swap_rows`, `matrix.swap_columns`, and `matrix.sort` inside
user-defined functions are rejected as a collection side-effect boundary.
`values.fill(value)` lowers to
`matrix.fill(values, value)` for supported matrix element kinds, `values.get(row, column)`
lowers to `matrix.get(values, row, column)`, `values.set(row, column, value)`
lowers to `matrix.set(values, row, column, value)` while keeping the same UDF
side-effect rejection, `values.copy()` lowers to `matrix.copy(values)`,
`values.transpose()` lowers to `matrix.transpose(values)`, and
`values.reverse()` lowers to `matrix.reverse(values)` with the same UDF
side-effect rejection, and
`values.rows()` / `values.columns()` / `values.elements_count()` /
`values.is_square()` lower to the
corresponding shape readers, while `values.is_binary()`,
`values.is_diagonal()`, `values.is_identity()`, `values.is_symmetric()`,
`values.is_antisymmetric()`, `values.is_stochastic()`, `values.is_zero()`,
`values.sum()`, `values.avg()`, `values.min()`, `values.max()`, and
`values.mode()`, `values.trace()`, `values.det()`, `values.eigenvalues()`,
`values.eigenvectors()`, `values.kron(other)`, `values.inv()`,
`values.mult(other)`, `values.diff(other)`, `values.pow(power)`,
`values.pinv()`, and `values.rank()` lower to the matching
read-only
`matrix.is_binary(values)`,
`matrix.is_diagonal(values)`, `matrix.is_identity(values)`,
`matrix.is_symmetric(values)`, `matrix.is_antisymmetric(values)`,
`matrix.is_stochastic(values)`, `matrix.is_zero(values)`,
`matrix.sum(values)`, `matrix.avg(values)`, `matrix.min(values)`,
`matrix.max(values)`, `matrix.mode(values)`, `matrix.trace(values)`,
`matrix.det(values)`, `matrix.eigenvalues(values)`,
`matrix.eigenvectors(values)`, `matrix.kron(values, other)`,
`matrix.mult(values, other)`, `matrix.diff(values, other)`,
`matrix.pow(values, power)`, `matrix.inv(values)`, `matrix.pinv(values)`, and
`matrix.rank(values)`
helpers.
`matrix.reshape(values, rows, columns)` changes the shape in place while
preserving element order and element count; `values.reshape(rows, columns)`
lowers to the same operation. `matrix.kron(left, right)` accepts runtime-owned
float or int matrix operands and returns an independent `matrix<float>`
Kronecker product whose shape is
`left.rows() * right.rows()` by `left.columns() * right.columns()`, propagates
`na` to result cells when either source cell is `na` or non-finite, preserves
zero-dimension results, and raises a runtime error when the result exceeds the
matrix cell budget. `values.kron(other)` lowers to the same read-only helper.
Matrix-by-matrix `matrix.mult(left, right)` accepts runtime-owned float or int
matrix operands and returns an independent `matrix<float>` product whose shape
is `left.rows()` by `right.columns()`, requires
`left.columns() == right.rows()`, propagates `na` to a result cell when any
contributing source cell is `na` or non-finite, preserves zero-dimension
results, and raises a runtime error on incompatible shapes or when the result
exceeds the matrix cell budget. `values.mult(other)` lowers to the same
read-only helper for matrix or scalar-right operands. When either namespace
operand is a numeric or `na` scalar and the other operand is a matrix,
`matrix.mult` returns an independent same-shape `matrix<float>` with the scalar
multiplied into each numeric cell; `na` or non-finite cells/scalars propagate
to `na`. `matrix.mult(values, vector)` and `values.mult(vector)` accept a
right-hand numeric array as a column vector, require the array size to match the
matrix column count, and return an independent `array<float>` with one element
per matrix row. Each result element is the row/vector dot product; any `na` or
non-finite contributing cell or vector element makes that result element `na`.
Namespace `matrix.mult(vector, values)` accepts a left-hand numeric array as a
row vector, requires the array size to match the matrix row count, and returns
an independent `array<float>` with one element per matrix column. Array-pair
and non-numeric-array `matrix.mult` overloads remain unsupported.
Matrix-by-matrix `matrix.diff(left, right)` accepts runtime-owned float or int
matrix operands and returns an independent `matrix<float>` element-wise
difference whose shape matches both operands, requires identical row and column
counts, propagates `na` to a result cell when either source cell is `na` or
non-finite, preserves zero-dimension results, and raises a runtime error on
incompatible shapes. `values.diff(other)` lowers to the same read-only helper
for matrix or scalar-right operands. When either namespace operand is a numeric
or `na` scalar and the other operand is a matrix, `matrix.diff` returns an
independent same-shape `matrix<float>` using operand order: matrix-left
subtracts the scalar from each numeric cell, while scalar-left subtracts each
numeric cell from the scalar. `na` or non-finite cells/scalars propagate to
`na`.
`matrix.pow(values, power)` accepts runtime-owned square float or int matrices
and returns an independent `matrix<float>` power. Power `0` returns an identity
matrix, power `1` returns an independent copy, and larger powers multiply with
the same `na`/non-finite propagation used by matrix multiplication. Non-square
matrices and negative
powers raise runtime errors; empty `0 x 0` matrices return independent empty
`0 x 0` matrices. `values.pow(power)` lowers to the same read-only helper.
`matrix.elements_count(values)` returns the
current row-count by column-count element count, including zero for
zero-dimension matrices. `matrix.is_square(values)` returns whether the current
row and column counts match. `matrix.is_zero(values)` returns true when every
stored numeric cell is zero, false for any non-zero or `na` cell, and true for
zero-element matrices. `matrix.is_binary(values)` returns true when every
stored numeric cell is exactly zero or one, false for any other numeric value
or `na` cell, and true for zero-element matrices. `matrix.is_diagonal(values)`
returns true when every cell outside the main diagonal is zero, false for any
non-zero or `na` off-diagonal cell, allows any main-diagonal value, does not
require a square shape, and returns true for zero-element matrices.
`matrix.is_identity(values)` returns true only for square matrices whose main
diagonal cells are exactly one and whose other cells are exactly zero, false for
any `na` cell, false for non-square matrices, and true for empty `0 x 0`
matrices.
`matrix.is_symmetric(values)` returns true only for square matrices whose
stored numeric cells match their transposed counterparts, false for any `na`
cell, false for non-square matrices, and true for empty `0 x 0` matrices.
`matrix.is_antisymmetric(values)` returns true only for square matrices whose
main diagonal cells are exactly zero and whose off-diagonal cells are the
negatives of their transposed counterparts, false for any `na` cell, false for
non-square matrices, and true for empty `0 x 0` matrices.
`matrix.is_stochastic(values)` returns true when every cell is a finite
non-negative number and either every row sums exactly to one or every column
sums exactly to one, returns false for any `na` or negative cell, and returns
false for zero-element matrices.
`matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and `matrix<color>` typed
declarations accept compatible matrix values or `na`.
Committed matrix history snapshots return
fresh matrix copies. `matrix.row(values, row)` and
`matrix.col(values, column)` return independent row/column snapshots:
`array<float>` for float matrices, `array<int>` for int matrices,
`array<bool>` for bool matrices, `array<string>` for string matrices, and `array<color>` for color matrices.
`values.row(row)` lowers to
`matrix.row(values, row)` and returns the same independent row snapshot.
`values.col(column)` lowers to `matrix.col(values, column)` and returns the same
independent column snapshot. `matrix.add_row(values, row, array_id)` copies a
row array with the same element kind as the matrix into the matrix at an
insertion index in `0..=rows`, requires the array length to match the matrix
column count, and remains under the 100,000-cell matrix budget.
`values.add_row(row, array_id)` lowers to the same operation.
`matrix.add_col(values, column, array_id)` copies a column array with the same
element kind as the matrix into the matrix at an insertion index in
`0..=columns`, requires the array length to match the matrix row count, and
remains under the same cell budget. `values.add_col(column, array_id)` lowers
to the same operation.
`matrix.remove_row(values, row)` removes an existing row using the same row
index bounds as `matrix.row`, and `values.remove_row(row)` lowers to the same
operation. `matrix.remove_col(values, column)` removes an existing column using
the same column index bounds as `matrix.col`, and `values.remove_col(column)`
lowers to the same operation. `matrix.swap_rows(values, row1, row2)` swaps two
existing rows in place using row-read bounds, preserves shape, and leaves
same-row or zero-column swaps unchanged after validating both rows.
`values.swap_rows(row1, row2)` lowers to the same operation.
`matrix.swap_columns(values, column1, column2)` swaps two existing columns in
place using column-read bounds, preserves shape, and leaves same-column or
zero-row swaps unchanged after validating both columns.
`values.swap_columns(column1, column2)` lowers to the same operation.
`matrix.sort(values, column?, order?)` reorders complete rows in place by the
selected column, defaults to column `0`, accepts `order.ascending` and
`order.descending`, preserves original row order for equal sort keys, places
`na` keys last ascending and first descending, and validates the selected
column with column-read bounds. `values.sort()`, `values.sort(column)`, and
`values.sort(column, order)` lower to the same operation.
`matrix.submatrix(values, from_row?, to_row?, from_column?, to_column?)`
returns an independent matrix copy of the selected half-open row/column range,
defaulting omitted bounds to the full source matrix. Range indexes accept
`0..=rows` and `0..=columns`, allowing empty row or column slices, and reject
`na`, out-of-bounds, or reversed ranges at runtime. `values.submatrix(...)`
lowers to the same operation.
`matrix.sum(values)` sums numeric cells, ignores
`na` cells, and returns `na` for empty or all-`na` matrices; `matrix.avg(values)`
averages the same non-`na` numeric cell set; `matrix.min(values)` and
`matrix.max(values)` scan the same non-`na` numeric cell set; `matrix.mode(values)`
returns the smallest most-frequent non-`na` numeric cell when a value repeats
and otherwise returns `na`; `matrix.trace(values)` sums non-`na` numeric cells
on the main diagonal over `min(rows, columns)` positions and returns `na` when
the diagonal has no numeric cells; `matrix.det(values)` computes the
determinant of square runtime-owned float or int matrices, returns `1.0` for
empty `0 x 0` matrices, returns `na` for any `na` or non-finite cell, and
raises a runtime error for non-square matrices.
`matrix.eigenvalues(values)` returns an independent `array<float>` of real
eigenvalues for square runtime-owned float or int matrices, returns an empty
array for empty `0 x 0` matrices, returns `na` for any `na` or non-finite cell
and for non-real eigenvalue results, and raises a runtime error for non-square
matrices.
`matrix.eigenvectors(values)` returns an independent `matrix<float>` whose
columns are real eigenvectors for square runtime-owned float or int matrices,
returns an independent empty `0 x 0` matrix for empty `0 x 0` input, returns
`na` for any `na` or non-finite cell and for non-real or incomplete eigenvector
results, and raises a runtime error for non-square matrices.
`matrix.inv(values)` computes an independent inverse matrix for non-singular
square runtime-owned float or int matrices, returns an independent empty
`0 x 0` matrix for empty `0 x 0` input, returns `na` for any `na` or non-finite
cell and for singular matrices, and raises a runtime error for non-square
matrices.
`matrix.pinv(values)` computes an independent Moore-Penrose pseudo-inverse
matrix with row/column counts swapped from the source, returns an independent
zero-cell matrix for zero-row or zero-column input, returns `na` for any `na`
or non-finite cell, and supports singular and rectangular matrices.
`matrix.rank(values)` computes the rank of rectangular runtime-owned float or
int matrices, returns `0` for zero-element matrices, and returns `na` for any
`na` or non-finite cell.
Matrix method syntax beyond the fixture-backed aliases, matrix templates beyond
`float`, `int`, `bool`, `string`, and `color`, and bare matrix typed declarations
remain outside the executable subset. Matrix `varip` is fixture-backed for the
supported runtime-owned matrix element kinds, with realtime backing-store
handoff matching the general `varip` model.
Matrix allocation rejects negative dimensions and is guarded by the runtime cell
budget before storage is reserved, so invalid or oversized
`matrix.new<T>` calls report deterministic runtime errors.

Supported drawing-object calls currently cover the initial `label.*`, `line.*`,
`box.*`, and `table.*` lifecycles. Labels use deterministic ids, sparse
lifecycle snapshots, creation snapshots with bar-index or bar-time x locations,
price/abovebar/belowbar y locations, official style constants, selected
x-location and y-location snapshot mutation, snapshot cloning, non-reused ids,
and declaration-driven max-count eviction. Lines use the same lifecycle rules with
bar-index x coordinates, price y coordinates,
selected color/width/style and extend fields, snapshot cloning, non-reused ids,
and declaration-driven max-count eviction. `line.new` can initialize existing line
snapshot fields for xloc, extend, color, style, and width when `xloc` is
omitted, `xloc.bar_index`, or `xloc.bar_time`; omitted `color` records the
official `color.blue` default. The chart-point overload uses `point.index` or
`point.time` according to `xloc`; `force_overlay` is accepted but remains a
host display responsibility. Selected `line.set_*` mutators update
endpoint/color/width/style/extend snapshots, and `line.set_xloc` with
`xloc.bar_index` or `xloc.bar_time` updates the line's x1, x2, and xloc
snapshot values, including when called from ordinary and independent while-loop
control-flow blocks. Boxes use
the same lifecycle rules with bar-index left/right coordinates, price top/bottom
coordinates,
selected background/border fields, snapshot cloning, non-reused ids, and a
declaration-driven max-count eviction. `box.new` can initialize existing box snapshot fields
for xloc, background, border, extend, text, text color, text size, text
alignment, text wrap, font family, and text formatting when `xloc` is omitted,
`xloc.bar_index`, or `xloc.bar_time`; omitted `border_color` and `bgcolor`
record the official `color.blue` default, omitted `text_color` records the
official `color.black` default, and omitted `text_size` records the official
`size.auto` default. The chart-point overload uses `point.index` or `point.time`
according to `xloc`; `force_overlay` is accepted but remains a host display
responsibility.
`box.set_xloc` with `xloc.bar_index` or `xloc.bar_time` updates the box's left,
right, and xloc snapshot values. Tables use deterministic ids,
fixed positive
dimensions, optional `table.new` background-color, frame-color, frame-width,
border-color, and border-width initialization, and sparse cell snapshots for
text/background/text-color/width/height/text-size writes and final table-level
mutations with `table.set_position` and
`table.set_bgcolor`/`table.set_frame_color`/`table.set_frame_width`/
`table.set_border_color`/`table.set_border_width`, plus
`table.cell_set_text`/`table.cell_set_bgcolor`/
`table.cell_set_text_color`/`table.cell_set_width`/`table.cell_set_height`/
`table.cell_set_text_size`/`table.cell_set_text_halign`/
`table.cell_set_text_valign`/`table.cell_set_text_wrap`/
`table.cell_set_tooltip`/`table.cell_set_text_font_family`/
`table.cell_set_text_formatting` mutations
of previously populated cells and
`table.clear` inclusive rectangular cell-content removal snapshots,
`table.merge_cells` inclusive merged-cell rectangle snapshots, plus
`table.delete` deletion snapshots.
Supported label, line, box, and table id-first drawing functions can also use
Pine method-call syntax. The semantic analyzer and HIR lowering rewrite the
receiver into the first function argument, so `id.set_text("x")` has the same
runtime behavior as `label.set_text(id, "x")` when `id` is a label. This is an
alias for the already supported function subset only; unsupported drawing
methods and unsupported xloc/time variants remain unsupported.
`*.delete(na)`, mutation of `na`, mutation after deletion, and deleting an
already deleted drawing object are no-ops where deletion exists; invalid
non-`na` ids are runtime errors. Labels, lines, boxes, and polylines use the
runtime default 50-object display limit unless their declaration sets a supported
`max_*_count` value; labels, lines, and boxes accept 1 through 500, while
polylines accept 1 through 100. Tables have a 50-object limit and 1000-cell
per-table limit. `label.set_x`, `label.set_y`, `label.set_xy`,
`label.set_point`, `label.set_text`, and `label.set_size` update the latest existing label
snapshot, including when called from ordinary and independent while-loop
control-flow blocks. `label.set_point` selects `point.index` when the label's
current `xloc` is `xloc.bar_index`, selects `point.time` when it is
`xloc.bar_time`, and sets `y` from `point.price`. `label.set_color`,
`label.set_textcolor`,
`label.set_style`, `label.set_tooltip`, `label.set_textalign`,
`label.set_text_font_family`, and `label.set_text_formatting` update their
host-neutral snapshot fields, including when called from ordinary and
independent while-loop control-flow blocks. `label.set_xloc` stores
`xloc.bar_index` or `xloc.bar_time`
with the new x-coordinate in the host-neutral snapshot, including when called
from ordinary and independent while-loop control-flow blocks.
`label.set_yloc` stores the selected y-location constant, including when called
from ordinary and independent while-loop control-flow blocks.
`label.set_textalign` stores the selected
horizontal text alignment constant. `label.set_text_font_family` stores the
selected font-family constant. `label.new` can initialize `xloc`, `yloc`,
text, color, official style, text color, size, `textalign`, `text_font_family`,
and a `text_formatting` mask in the host-neutral snapshot; omitted `text`
records an empty string, omitted `color` records the official `color.blue`
default, and omitted `textcolor` records the official `color.white` default. Its
chart-point overload uses `point.index` for `xloc.bar_index`, `point.time` for
`xloc.bar_time`, and `point.price` for y. `force_overlay` is accepted but remains
a host display responsibility.
`label.set_text_formatting` stores the selected
`text.format_none`/`text.format_bold`/`text.format_italic` mask, including
bold+italic combinations. Visual placement for above/below-bar and time/index
coordinates, glyph styling, and text layout remain host responsibilities.
`label.delete` appends an `exists: false` label snapshot, including when called
from ordinary and independent while-loop control-flow blocks.
`label.copy` clones the latest existing label
snapshot into a new deterministic id, including when called from ordinary and
independent while-loop control-flow blocks, returns `na` for `na` or deleted
labels, and shares the effective label limit. `label.new` and `label.copy` use
the default 50-label runtime limit when declarations omit `max_labels_count`, or
the named declaration value from 1 through 500, and evict the oldest active
label by appending a deletion snapshot before creating the new label.
`label.get_x` reads the latest
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
control-flow blocks after label deletion or max-count eviction. `line.delete` appends an
`exists: false` line snapshot, including when called from ordinary and
independent while-loop control-flow blocks. `line.copy` clones the latest
existing line snapshot into a new deterministic id, including when called from
ordinary and independent while-loop control-flow blocks,
returns `na` for `na` or deleted lines, and shares the effective line limit.
`line.all` returns currently existing line ids in creation order, including
when read from ordinary and independent while-loop control-flow blocks after
line deletion or max-count eviction. `line.new` and `line.copy` keep at most
the effective line limit active by appending `exists: false` snapshots to the
oldest active lines before creating new ones. Omitted declarations use the
runtime's default 50-line display limit; named `max_lines_count` declaration
arguments from 1 through 500 are parsed into HIR for indicators and strategies
and consumed by this line eviction path.
`line.get_x1` reads the latest existing line x1 value, including when called
from ordinary and independent while-loop control-flow blocks, and returns `na`
for `na` or deleted lines.
`line.get_y1` reads the latest existing line y1 value, including when called
from ordinary and independent while-loop control-flow blocks, and returns `na`
for `na` or deleted lines.
`line.get_x2` reads the latest existing line x2 value, including when called
from ordinary and independent while-loop control-flow blocks, and returns `na`
for `na` or deleted lines.
`line.get_y2` reads the latest existing line y2 value, including when called
from ordinary and independent while-loop control-flow blocks, and returns `na`
for `na` or deleted lines.
`line.get_price` reads the latest existing bar-index line snapshot, including
when called from ordinary and independent while-loop control-flow blocks,
applies x1/y1/x2/y2 interpolation or extrapolation for the requested x value,
and returns `na` for `na`, deleted, vertical, nonnumeric, or time-coordinate
lines; timestamp interpolation remains unsupported. `box.copy` clones the latest
existing box snapshot into a new deterministic id, including when called from
ordinary and independent while-loop control-flow blocks, returns `na` for `na`
or deleted boxes, and shares the effective box limit. `box.new` and `box.copy`
use the default 50-box runtime limit when declarations omit `max_boxes_count`,
or the named declaration value from 1 through 500, and evict the oldest active
box by appending a deletion snapshot before creating the new box. `box.delete`
removes the latest existing box snapshot, including when called from ordinary
and independent while-loop control-flow blocks, and deleting `na` or already
deleted boxes is a no-op. `box.all` returns currently existing box ids in
creation order, including when read from ordinary and independent while-loop
control-flow blocks after box deletion or max-count eviction. `box.set_left`,
`box.set_top`,
`box.set_right`, `box.set_bottom`, `box.set_lefttop`, and
`box.set_rightbottom` update the host-neutral geometry snapshot, including when
called from ordinary and independent while-loop control-flow blocks.
`box.set_bgcolor`, `box.set_border_color`, `box.set_border_width`,
`box.set_border_style`, and `box.set_extend` update the host-neutral style
snapshot, including when called from ordinary and independent while-loop
control-flow blocks; visual extension remains a host responsibility.
`box.set_xloc` with `xloc.bar_index` or `xloc.bar_time` updates the box's left,
right, and xloc values in the host-neutral snapshot, including when called from
ordinary and independent while-loop control-flow blocks. `box.set_text` records
the box text string in the
host-neutral snapshot. `box.set_text_color` records the text color in the
host-neutral snapshot. `box.set_text_size` records the selected size constant in
the host-neutral snapshot. `box.set_text_halign` records the selected horizontal
alignment constant in the host-neutral snapshot. `box.set_text_valign` records
the selected vertical alignment constant in the host-neutral snapshot.
`box.set_text_wrap` records the selected wrapping constant in the host-neutral
snapshot. `box.set_text_font_family` records the selected font-family constant
in the host-neutral snapshot. `box.set_text_formatting` records the selected
`text.format_none`/`text.format_bold`/`text.format_italic` mask, including
bold+italic combinations. These text snapshot setters apply when called from
ordinary and independent while-loop control-flow blocks; text rendering, glyph
styling, and font layout remain host responsibilities. `box.get_left` reads the
latest existing box left value,
including when called from ordinary and independent while-loop control-flow
blocks, and returns `na` for `na` or deleted boxes. `box.get_right` reads the
latest existing box right value, including when called from ordinary and
independent while-loop control-flow blocks, and returns `na` for `na` or
deleted boxes. `box.get_top` reads the latest existing box top value, including
when called from ordinary and independent while-loop control-flow blocks, and
returns `na` for `na` or deleted boxes. `box.get_bottom` reads the latest
existing box bottom value, including when called from ordinary and independent
while-loop control-flow blocks, and returns `na` for `na` or deleted boxes.
`table.set_position`
updates the table's final position value, including when called from ordinary
and independent while-loop control-flow blocks. `table.set_bgcolor` updates the
table's final background color value, including when called from ordinary and
independent while-loop control-flow blocks. `table.set_frame_color` updates the
table's final frame-color value, including when called from ordinary and
independent while-loop control-flow blocks. `table.set_frame_width` updates the
table's final frame-width value, including when called from ordinary and
independent while-loop control-flow blocks. `table.set_border_color` updates
the table's final border-color value, including when called from ordinary and
independent while-loop control-flow blocks. `table.set_border_width` updates
the table's final border-width value, including when called from ordinary and
independent while-loop control-flow blocks. `table.new` optional
`bgcolor`,
`frame_color`, `frame_width`, `border_color`, and `border_width` initialize the
table's final background-color, frame-color, frame-width, border-color, and
border-width values.
`table.delete` appends an `exists: false` table snapshot, including when called
from ordinary and independent while-loop control-flow blocks. `table.clear`
removes already populated cells
in the inclusive rectangular range from `start_column`,
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
errors. Later table-level or cell mutations of deleted
tables are no-ops.
`table.all` returns currently existing table ids in creation order, including
when read from ordinary and independent while-loop control-flow blocks after
table deletion.
`table.set_bgcolor`, `table.set_frame_color`, `table.set_frame_width`,
`table.set_border_color`, and `table.set_border_width` update the table's final
style values; visual anchoring, border rendering, and layout remain host
responsibilities. `table.cell_set_text`, `table.cell_set_bgcolor`,
`table.cell_set_text_color`, `table.cell_set_width`, `table.cell_set_height`,
`table.cell_set_text_size`, `table.cell_set_text_halign`,
`table.cell_set_text_valign`, `table.cell_set_text_wrap`,
`table.cell_set_tooltip`, `table.cell_set_text_font_family`, and
`table.cell_set_text_formatting` update the target previously populated cell in
the host-neutral table snapshot, including when called from ordinary and
independent while-loop control-flow blocks, while preserving the cell's other
supported fields; actual table layout, text rendering, text layout, wrapping,
tooltip display, font rendering, and bold/italic rendering remain host
responsibilities.

Drawing side effects are allowed in top-level control flow, including supported
`if`, `switch`, `for`, and `while` bodies. Realtime forming updates start from
the confirmed runtime snapshot, so unconfirmed label, line, box, and table
creation, mutation, deletion, and cell writes are rolled back when a new forming
update arrives.
Drawing side effects inside user-defined functions are rejected under the same
side-effect boundary as output calls and array mutation until UDF object
semantics are deliberately expanded.

## Request Data

`request.security` is supported only for the matrix-backed subset. The runtime
does not fetch data. Hosts inject immutable requested bar streams through the
request provider contract, keyed by symbol and timeframe, and the runtime
validates duplicate keys plus sorted unique bar times before execution.

Same-context requests whose symbol and timeframe match the chart metadata
evaluate the requested expression in the chart context. Provider-backed
same-or-higher-timeframe requests evaluate the supported expression in an
isolated requested-context runtime over the provider bars. Requested-context
state is separate from chart-context state: history buffers, `ta.*` callsite
state, `var` storage, arrays, drawing objects, and outputs do not leak between
the two contexts.

The same-context identity form also supports tuple literals made from
side-effect-free elements and selected tuple-returning calls when destructured
directly. The selected tuple-returning calls currently include `ta.macd`,
`ta.bb`, `ta.kc`, `ta.supertrend`, `ta.dmi`, and
`ta.vwap(source, anchor, stdev_mult)`.

The supported provider requested expression subset includes direct OHLCV/time sources,
pure arithmetic and ternaries, history references, `na`, `nz`, selected
stateless `math.*` calls, fixed-mintick `math.round_to_mintick`, `math.sum`,
`ta.cum`, `ta.sma`, `ta.ema`, `ta.dema`, `ta.tema`, `ta.rma`, `ta.rsi`,
`ta.accdist`, `ta.iii`, `ta.nvi`, `ta.obv`, `ta.pvi`, `ta.pvt`, `ta.wvad`,
`ta.tsi`, `ta.cmo`, `ta.cci`, `ta.cog`, `ta.bop`, `ta.ao`, `ta.max`, `ta.min`, `ta.mfi`, `ta.stoch`, `ta.wpr`, `ta.sar`, `ta.tr` function calls, `ta.atr`, `ta.highest`, `ta.lowest`, `ta.highestbars`, `ta.lowestbars`, `ta.change`, `ta.mom`, `ta.roc`, `ta.range`, `ta.dev`, `ta.vwap`,
`ta.bbw`, `ta.kcw`, `ta.pivothigh`, `ta.pivotlow`, `ta.correlation`,
`ta.covariance`, `ta.median`, `ta.mode`, `ta.percentile_nearest_rank`,
`ta.percentile_linear_interpolation`,
`ta.percentrank`, `ta.stdev`, `ta.variance`, `ta.wma`, `ta.vwma`, `ta.swma`,
`ta.hma`, `ta.alma`, `ta.linreg`, `ta.rising`, `ta.falling`, `ta.barssince`, `ta.valuewhen`,
`ta.cross`, `ta.crossover`, and `ta.crossunder`.
Rolling callsite state for `math.sum` and `ta.*` calls is owned by the isolated
requested context. Stateful math calls such as `math.random` and the `ta.tr`
variable form remain outside the requested-expression subset.
Provider-backed tuple literals whose elements are in the supported scalar subset
are supported when destructured directly from the request. Selected
provider-backed tuple-returning calls are also supported, currently `ta.macd`,
`ta.bb`, `ta.kc`, `ta.supertrend`, `ta.dmi`, and
`ta.vwap(source, anchor, stdev_mult)`. Other provider-backed tuple expressions
remain unsupported.

Requested-context results are cached deterministically by callsite, requested
symbol, requested timeframe, and expression identity for the duration of one
runtime execution. Repeated identical calls reuse that cache instead of
mutating provider data or chart state.

Same-timeframe provider requests require an exact requested-bar timestamp
match. Higher-timeframe provider requests use the default `gaps_off` and
`lookahead_off` subset: a requested value is visible only after the requested
bar has closed relative to the current chart bar, missing confirmed requested
bars forward-fill the last confirmed requested value, and chart bars before the
first confirmed requested bar return `na`.

Lower-timeframe `request.security`, `request.security_lower_tf`, optional
parameters, explicit gaps/lookahead, advanced request families, provider local
aliases, UDF calls, output/drawing/alert side effects, input declarations, and
array mutation inside requested expressions remain unsupported.

### `varip`

Scalar, scalar typed-array, single `chart.point` value, same-local scalar-tree
UDT array, explicitly typed same-local or same-imported scalar-tree UDT, and
direct-constructor-inferred or direct-alias-inferred same-local or same-imported
scalar-tree UDT `varip` declarations use the intrabar persistence model
described above. The explicitly typed UDT subset
includes `na`, same-UDT constructors, same-identity aliases, and fixture-backed
same-UDT ternary/switch/if/for/for-in/while initializers, including nested
same-local and same-imported scalar-tree Wrapper values. Drawing object ids are rejected before runtime because their
object stores are not part of the `varip` handoff; tuples,
broader untyped non-constructor-inferred UDT expressions, nested-field UDT values, non-scalar UDT
array `varip`, and other value families remain rejected until their
realtime state partitions are designed. The closed Phase I boundary for the
original scalar and scalar
typed-array subset is recorded in
`docs/PHASE_I_AUDIT.md`.

## User-Defined Functions

Expression-body user-defined functions are lowered by inlining the body at each
callsite:

```pine
smooth(src, len) => ta.sma(src, len)
plot(smooth(close, 20))

spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))

range2(hi, lo) =>
    value = hi - lo
    value * 2

select_value(x, y) =>
    result = y
    if x > y
        result := x
    result
```

Inlining gives stateful calls inside the function body independent callsite
state for each syntactic UDF call. Named arguments are resolved before
lowering, and arguments are evaluated once into callsite-local temporaries.
Multi-statement function bodies execute local statements and return the final
expression. Local declarations and reassignments inside function block bodies
are scoped to the function callsite. A local declaration or loop counter can
shadow a parameter without changing references that were already resolved to
that parameter. Recursive functions, output side effects, drawing side effects,
and alert side effects inside functions, global reassignment inside functions,
and side-effecting calls as UDF arguments are rejected in the current
executable subset.

## Series and History References

History references use committed values:

```pine
close[1]
x[2]
```

Rules:

- Offset `0` refers to the current value.
- Positive offsets refer to previous committed bars.
- Out-of-range references return `na` unless the feature later needs stricter
  runtime errors for selected cases.
- Dynamic `na` offsets return `na`, but still evaluate the source expression on
  the current bar so expression and UDF-result series histories stay aligned.
- Dynamic offsets should be supported only after constant offsets are correct.
  Runtime profiles count dynamic history reads whose requested offset exceeds
  the effective retained depth in `historyDynamicRetentionMisses` and expose
  their maximum requested offset as `historyDynamicRetentionMaxMissedOffset`;
  those reads still return `na` in the current subset.

History can apply to variables, built-in series, and accepted expressions. Any
expression that needs history must have stable series storage assigned before
runtime execution. See [`SERIES_MODEL.md`](SERIES_MODEL.md).

Array variable history is supported for the fixture-backed scalar array,
scalar slice, label-array, label-slice, line-array, line-slice, box-slice,
linefill-array, linefill-slice, polyline-array, polyline-slice, box-array,
table-array, table-slice, and chart.point-array and chart.point-slice read
paths, including the official `previous = a[1]` and
`na(previous) ? na : previous.get(0)` pattern. When a
retained series value is an array id, runtime stores an independent array
snapshot for history and returns a fresh copy from positive-offset history
reads, so reading or mutating the historical copy does not alias the current
array id. Broader array-history edges, including future collection families and
richer mutation/aliasing cases, remain outside the current contract.

## `na`

`na` must be a first-class runtime value.

Rules:

- Arithmetic involving `na` usually produces `na`.
- Comparisons involving `na` should follow Pine-compatible behavior.
- Functions such as `nz()` and `na()` must be implemented deliberately, not as
  ordinary Python or Rust null checks.

## Inputs

Inputs are stable across a run:

```pine
generic = input(20, "Length")
length = input.int(20, "Length")
mode = input.string("SMA", "Mode")
start = input.time(0, "Start")
```

The analyzer accepts the supported input metadata subset and inputs carry the
`input` qualifier. Runtime execution evaluates each input's `defval` unless the
Rust runtime is run with call-site keyed `InputOverrides`, the CLI supplies
`--input-override CALL_SITE_ID=value`, or the Python host supplies a call-site
keyed `input_overrides` dictionary to `Program.run()` or `run_script()`, or the
WASM host supplies an `inputOverridesJson` object to a `*WithInputOverrides` run
API. Host-side `input.source` overrides are not implemented yet.

## Built-In OHLCV Series

The initial runtime should provide:

```text
open
high
low
close
volume
time
hl2
hlc3
ohlc4
bar_index
```

Each is a series value with a current value on each bar.

## Plotting Side Effects

Plot functions should not render charts. They should collect normalized output.

Examples:

```pine
p = plot(ta.sma(close, 20), color=color.orange)
h = hline(70)
fill(p, h)
```

The runtime currently collects normalized output fields, not full TradingView
rendering metadata:

- `plot`: numeric values and id.
- `hline`: price and id.
- `fill`: first/second plot or hline ids.
- `barcolor` and `bgcolor`: color values and id.
- `plotchar`: marker values, chars, and colors.
- `plotshape`: marker values, styles, locations, colors, texts, text colors,
  and sizes.
- `plotarrow`: marker values, up/down colors, min heights, and max heights.
- `plotbar`: open, high, low, close, and color values.
- `plotcandle`: open, high, low, close, body colors, wick colors, and border
  colors.
- `label.*`: deterministic label ids and sparse creation, mutation, and
  deletion snapshots for the supported label subset.
- `line.*`: deterministic line ids and sparse creation, mutation, and deletion
  snapshots for the supported line subset.
- `box.*`: deterministic box ids and sparse creation, mutation, and deletion
  snapshots for the supported box subset.
- `table.*`: deterministic table ids, dimensions, and sparse cell snapshots for
  the supported table subset.

Accepted metadata such as `offset`, `show_last`, `display`, `force_overlay`,
and `editable` does not yet transform, filter, or annotate these output series.
`linefill.new` creates runtime-owned linefill ids over supported line ids,
emits sparse color snapshots, and replaces the previous linefill for the same
line pair. `linefill.set_color` appends sparse color mutation snapshots for
existing linefill ids, while `na` ids and already-replaced linefill ids are
no-ops. `linefill.get_line1` and `linefill.get_line2` return the referenced
line ids for existing linefill ids; `na` ids and already-replaced linefill ids
return `na`. `linefill.delete` appends an `exists: false` snapshot for existing
linefill ids and is a no-op for `na`, replaced, or already deleted ids.
`linefill.all` returns a snapshot array of currently existing linefill ids in
creation order and omits linefills replaced by later same-pair creations or
deleted by `linefill.delete`. `array.new_linefill`, `array.from` over linefill
ids, and generic object-array mutation/search/read helpers support linefill
arrays with shallow reference elements. Numeric, truth, sorting, and string
join helpers remain limited to their existing scalar-compatible array families.
`chart.point` values have a fixture-backed execution model for constructors,
`copy`, `time`/`index`/`price` field reads, top-level field mutation, and
`array.new<chart.point>()` plus `array.from(chart.point, ...)` point-array
storage/read/mutation/search. `line.new` can create line snapshots from two
`chart.point` values, selecting `point.index` for `xloc.bar_index` and
`point.time` for `xloc.bar_time`, while retaining the existing line style,
extend, color, and width snapshot fields. `box.new` can create box snapshots
from two `chart.point` values using the same `xloc` coordinate selection while
retaining the existing box style, text, and fill snapshot fields, including the
official default `color.blue` for omitted `border_color`/`bgcolor`, `color.black`
for omitted `text_color`, and `size.auto` for omitted `text_size`. `label.new`
can create label snapshots from one `chart.point`, selecting `point.index` for
`xloc.bar_index` and `point.time` for `xloc.bar_time`, while using
`point.price` for y and retaining the existing label text/style fields,
including the empty-string default for omitted `text`, the official default
`color.blue` for omitted `color`, and `color.white` for omitted `textcolor`.
`line.set_first_point` and
`line.set_second_point` update the selected endpoint from a `chart.point`,
using the line's current `xloc` to choose `point.index` or `point.time` for the
x-coordinate. `box.set_top_left_point` and `box.set_bottom_right_point` update
the selected corner from a `chart.point`, using the box's current `xloc` to
choose `point.index` or `point.time` for the x-coordinate. `label.set_point`
updates a label's x/y coordinates from a `chart.point`, using the label's
current `xloc` to choose `point.index` or `point.time` for x and `point.price`
for y. `polyline.new`
creates runtime-owned polyline ids from an
`array<chart.point>` input, copies the current point-list values into a
host-neutral `polylines[].snapshots[]` entry, and records `curved`, `closed`,
`xloc`, `lineColor`, `fillColor`, `lineStyle`, `lineWidth`, and
`forceOverlay`. When `line_color` is omitted, `polyline.new` records the
official default `color.blue`; omitted `fill_color` remains `na`.
`polyline.delete` appends an `exists: false` snapshot for an existing id and
treats `na` or already-deleted ids as no-ops.
`polyline.all`
returns currently existing polyline ids in creation order. Realtime forming-bar
updates roll back abandoned polyline creations, deletions, copied point lists,
and `polyline.all` reads from the last confirmed drawing state. The historical
runtime keeps at most the effective polyline limit active by appending
`exists: false` snapshots to the oldest active polylines before creating new
ones. Omitted declarations use the runtime's default 50-polyline display limit;
named `max_polylines_count` declaration arguments from 1 through 100 are parsed
into HIR for indicators and strategies and consumed by this polyline eviction
path. The current polyline id array subset is fixture-backed through
`array.new_polyline`, official `array.new<polyline>` template syntax,
typed `array<polyline>`/`polyline[]` declarations, `array.from(polyline, ...)`,
and generic object-array storage, mutation, read, search, copy, slice, concat,
reverse, clear, and array/slice history snapshots.

## Determinism

A compiled program must produce the same result for the same bars and inputs.
Host time, network access, randomness, and file system access should not exist
in the core runtime.
