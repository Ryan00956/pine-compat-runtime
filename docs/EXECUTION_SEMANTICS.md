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
positive const numeric fixed default entry quantity. The fixed default subset is
the only supported declaration quantity mode; percent-of-equity, cash sizing,
contracts, margin, and currency conversion remain unsupported.
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
`strategy.cancel(id)` cancels matching internal pending entry ids and matching
pending exit ids in the supported order subset. Unknown, already-filled, and
already-cancelled ids are no-op. Cancellation records no public order, trade, or
pending-order output.
`strategy.cancel_all()` cancels all currently supported internal pending entries
and pending exits. It is a no-op when no supported pending order exists and
records no public order, trade, or pending-order output.

`strategy.close(id)` closes the full matching long position at the current bar
close. Configured slippage worsens the supported long close fill price after
trigger selection. It records a closed trade with entry/exit bar indexes,
entry/exit times, entry/exit prices, quantity, and net realized profit after
supported commission when configured, then appends a flat position
snapshot with `size = 0` and `avgPrice = null`.
If no position is open, the id does not match the open entry, or the position
has already been closed, the close call is a no-op.

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
missing-entry pre-placement, or pyramiding. The only multiple-pending
reservation subset is explicit fixed `qty` or `qty_percent` single-trigger or
one-downside/one-upside bracket or trailing `strategy.exit` calls for the
current matching long entry.

Strategy-mode scripts can read `strategy.position_size` and
`strategy.position_avg_price`, `strategy.openprofit`, `strategy.netprofit`,
`strategy.grossprofit`, `strategy.grossloss`, `strategy.avg_trade`,
`strategy.avg_winning_trade`, `strategy.avg_losing_trade`,
`strategy.max_runup`, `strategy.max_drawdown`, and
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
before any drawdown from the maximum equity baseline. `strategy.max_drawdown_percent` remains
unsupported. `strategy.max_runup` returns the maximum intrabar equity run-up
amount over the current supported long-only trading interval, using the
supported entry equity, the minimum equity before that entry, and the highest
high reached while the supported position is open. `strategy.max_runup_percent`
remains unsupported.
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
`strategy.exit` id retained on the closed trade record. Stage 7 Slice 15 adds
`strategy.closedtrades.max_runup(trade_num)`, returning the largest high-based
favorable excursion retained for the closed trade quantity. Stage 7 Slice 16
adds `strategy.closedtrades.max_drawdown(trade_num)`, returning the largest
low-based adverse excursion retained for the closed trade quantity. Stage 7
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
same public-output contract. They read the current
closed-trade list with a zero-based integer `trade_num`; missing,
negative, out-of-range, or non-integer indexes return `na`. These functions are
script-observable only through ordinary series outputs and do not add public
runtime JSON, Python, or WASM fields.
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
`strategy.opentrades.commission(trade_num)`, returning `0.0` for that open
position without configured commission and the current open supported entry
commission when configured. Stage 7 Slice 13 adds
`strategy.opentrades.max_runup(trade_num)`, returning the largest high-based
favorable excursion seen so far for that open position. Stage 7 Slice 14 adds
`strategy.opentrades.max_drawdown(trade_num)`, returning the largest low-based
adverse excursion seen so far for that open position. Other open-trade
namespace functions and public open-trade record output remain unsupported.

The strategy contract is host-independent and exposed consistently by CLI JSON,
Python dictionaries, and WASM JSON. Short entries, `strategy.exit` variants
beyond the supported single-trigger, one-downside/one-upside bracket,
trailing-stop, fixed-quantity, percent-quantity, explicit single-trigger or
bracket/trailing reservation subset, `strategy.cancel(id)`, and
`strategy.cancel_all()`, `strategy.order`, rich order families, strategy
reporting helpers beyond the supported position/profit/equity/count/run-up/drawdown variables,
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
`message` as the alert event `message`.

`alert(message)` is supported for const-string messages only. It emits an event
whenever execution reaches the call and serializes `source` as `alert`.
TradingView-style `{{...}}` placeholder interpolation and frequency arguments
remain unsupported until deterministic policies are designed.

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

The executable import subset intentionally excludes remote lookup, re-exports,
unaliased imports, side-effecting exported functions, imported UDT identity,
and imported methods.

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

`for i = start to end` evaluates the integer range once when the loop statement
is reached on a bar. The range is inclusive. The runtime increments when
`start <= end` and decrements when `start > end`. An explicit non-zero int
`by step` supplies the absolute step magnitude; the sign of `step` does not
override the range direction. If `start`, `end`, or `step` evaluates to `na`,
the loop body is skipped. The counter is scoped to the loop body. `break` exits
the nearest enclosing loop. `continue` skips the rest of the current iteration
and advances to the next loop counter value.

When a `for` loop is used as a declaration value, the loop body must end with an
expression. The loop returns the last value produced by that expression. If a
`continue` skips the expression or a `break` exits before it, the previous
produced value remains the loop result. If no iteration reaches the expression,
the loop result is `na`.

`while condition` evaluates the condition before each iteration. A `true`
condition executes the body, while `false` or `na` exits the loop. `break`
exits the nearest enclosing loop. `continue` skips the remaining body statements
and re-evaluates the condition. Runtime execution enforces a maximum iteration
guard per while statement evaluation so non-terminating scripts fail instead of
hanging execution. `while` expressions are not part of the current executable
subset.

`switch` expressions evaluate arms in source order. Selector-form switches
evaluate the selector once per bar, then compare each case expression with that
selector value. Selector-less switches evaluate each arm condition until one is
`true`. Only the selected result expression executes. If no arm matches and no
default arm exists, the switch returns `na`.

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
`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`, or
`array.new_color` declaration allocates a fresh array each time it executes.
`array.from` also allocates a fresh inferred typed array and requires at least
one non-`na` supported typed value. A `var` array declaration keeps the same id
and backing storage across bars, so mutations such as `array.push` or
`values.push(...)` persist.
Assigning an array to another variable copies the id, not the backing values;
mutating either name mutates the same runtime-owned array. `array.copy` and
`values.copy()` allocate a new array id initialized with the source array's
current element values. Realtime forming-bar rollback clones the confirmed
runtime store before executing a forming update, so array mutations and copies
made during a forming update do not leak into the confirmed store until a
confirmed update is committed.

For local user-defined types, `Type.new(...)` creates an immutable runtime value
containing the supported scalar field values in declaration order. Field reads
return the stored scalar value. Normal declarations allocate a fresh UDT value
when reached on each bar; `var` UDT declarations preserve the last confirmed
UDT value across bars and roll back during realtime forming updates like other
ordinary `var` values. Field mutation, UDT history references, UDT `varip`,
nested UDT fields, UDT arrays, and imported UDT values are rejected before
runtime execution.

Pure local UDT methods execute as receiver functions. The receiver value is
passed as the first internal argument and the method body is evaluated through
the same lowered expression path as a local UDF body. Method side effects,
recursive methods, unsupported parameter families, unknown receivers, and
imported methods are rejected during semantic analysis.

### `varip`

```pine
varip ticks = 0
```

The current executable `varip` subset supports global and local scalar
`int`/`float`/`bool`/`string`/`color`/`na` declarations plus scalar typed-array
ids for float, int, bool, string, and color arrays. Local scalar declaration
sites inside `if`, `for`, `while`, and user-defined function bodies use the
same declaration-site storage model as local `var`; each lowered scalar UDF
callsite gets independent storage. Historical execution treats this subset like
`var`: the declaration initializes once when first reached and reassignment
persists across committed bars.

Realtime forming-bar execution differs from ordinary `var`. A first forming
update for a bar starts from the last confirmed runtime state. Repeated forming
updates for that same bar carry `varip` slots forward from the previous forming
update. When a carried `varip` value is a supported array id, the referenced
backing array contents and element kind are copied from the previous forming
runtime as well. Ordinary `var`, outputs, non-`varip` arrays, drawing objects,
request caches, callsite state, and history reads continue to roll back to the
confirmed baseline. A confirmed update also seeds from the latest forming
`varip` values before executing and then commits the resulting values into the
confirmed runtime for the next bar.

Skipped local declaration sites do not initialize before their first executed
reach. `array.copy` returns an independent array id, and a `varip` slot that is
reassigned to that copy retains the copied backing store across repeated forming
updates without aliasing the source. Array mutation inside UDFs remains rejected
by the existing function side-effect rules. Drawing object ids are rejected for
`varip`: retaining only the id would be unsafe while label, line, box, and table
object stores continue to roll back between forming updates. Tuples and other
value families remain unsupported until their declaration-site, backing-store,
and rollback rules are explicitly designed.

Array bounds are stable in the current subset: `array.get`, `array.set`,
`array.insert`, and `array.remove` support negative indexes from the array end.
Indexes outside the current length make `array.get` and `array.remove` return
`na`, while `array.set` and `array.insert` ignore invalid indexes. Positive
`array.insert` at `size` appends; greater-than-size insert indexes are ignored.
`array.pop` or `array.shift` on an empty array returns `na`. `array.first` and
`array.last` also return `na` for empty arrays.
`array.fill` replaces all elements by default, or a half-open `[index_from,
index_to)` window when bounds are supplied; invalid ranges are ignored.
`array.indexof` and `array.lastindexof` return `-1` when no matching value is
present. Numeric binary search helpers expect int/float arrays sorted ascending;
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
`array.sort` orders int/float/string arrays in place, sorts ascending by
default, and accepts `order.ascending` or `order.descending`. `na` values and
empty string elements sort last in ascending order and first in descending
order. `array.sort_indices` returns a new int array containing the source
indexes in sorted order, follows the same order and special-value rules, and
leaves the source array unchanged. `array.reverse` reverses any supported typed
array in place.
`array.join` converts supported array elements to string
with the default numeric format, uses `,` as the default separator, and returns
an empty string for empty arrays. Color elements render as normalized integer
color values. Joined results over 40,960 characters are runtime errors.
`array.slice` returns a same-kind array with the half-open `[index_from,
index_to)` window; negative, reversed, or out-of-range bounds return `na`.
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

Supported drawing-object calls currently cover the initial `label.*`, `line.*`,
`box.*`, and `table.*` lifecycles. Labels use deterministic ids, sparse
lifecycle snapshots, non-reused ids, and a 500-object runtime limit. Lines use
the same lifecycle rules with bar-index x coordinates, price y coordinates,
selected color/width/style and extend fields. Boxes use the same lifecycle
rules with bar-index left/right coordinates, price top/bottom coordinates, and
selected background/border fields. Tables use deterministic ids, fixed positive
dimensions, and sparse cell snapshots for text/background/text-color writes.
`*.delete(na)`, mutation of `na`, mutation after deletion, and deleting an
already deleted drawing object are no-ops where deletion exists; invalid
non-`na` ids are runtime errors. Labels, lines, and boxes each have a
500-object runtime limit; tables have a 50-object limit and 1000-cell
per-table limit.

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
same-or-higher-timeframe requests evaluate the supported scalar expression in an
isolated requested-context runtime over the provider bars. Requested-context
state is separate from chart-context state: history buffers, `ta.*` callsite
state, `var` storage, arrays, drawing objects, and outputs do not leak between
the two contexts.

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

Scalar and scalar typed-array `varip` declarations use the intrabar persistence
model described above. Drawing object ids are rejected before runtime because
their object stores are not part of the `varip` handoff; tuples and other value
families remain rejected until their realtime state partitions are designed. The
closed Phase I boundary for this subset is recorded in `docs/PHASE_I_AUDIT.md`.

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
- Dynamic offsets should be supported only after constant offsets are correct.

History can apply to variables, built-in series, and accepted expressions. Any
expression that needs history must have stable series storage assigned before
runtime execution. See [`SERIES_MODEL.md`](SERIES_MODEL.md).

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
`input` qualifier. Runtime execution currently evaluates each input's `defval`;
host-provided input override APIs are not implemented yet.

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
`polyline.*` remains unsupported until `chart.point` values and point-list
arrays have a fixture-backed execution model.

## Determinism

A compiled program must produce the same result for the same bars and inputs.
Host time, network access, randomness, and file system access should not exist
in the core runtime.
