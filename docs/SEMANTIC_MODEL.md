# Semantic Model

This document defines the language semantics that must be stable before the
runtime grows broad compatibility.

The first implementation should prefer a small, explicit semantic model over a
large set of partially compatible behaviors.

## Value Kinds

Initial value kinds:

- `int`
- `float`
- `bool`
- `string`
- `color`
- `plot`
- `hline`
- `na`
- `void`

`na` is a real runtime value, but it should not become a universal static type
that erases useful diagnostics. The analyzer may use an internal "unknown due to
na" marker while it waits for contextual type information.

## Script Declarations

A script may have at most one top-level declaration call. `indicator(...)`
selects indicator mode and `strategy(...)` selects strategy mode. Phase G
accepts `strategy(...)` with the common declaration metadata subset plus
positive const numeric `initial_capital`. Phase L adds the fixed default
quantity declaration subset:
`default_qty_type=strategy.fixed, default_qty_value=N` with positive const
numeric `N`. Stage 7 Slice 31 adds
`default_qty_type=strategy.percent_of_equity, default_qty_value=N` with positive
const numeric `N`; omitted supported entry quantities resolve at placement time
from current supported equity and current close. Stage 7 Slices 17, 18, and 21
add supported commission declaration subsets:
`commission_type=strategy.commission.cash_per_contract, commission_value=N` and
`commission_type=strategy.commission.cash_per_order, commission_value=N` with
finite non-negative const numeric `N`, plus
`commission_type=strategy.commission.percent, commission_value=N` where
supported fills debit `qty * fill_price * N / 100`. Stage 7 Slice 19 adds `slippage=N` with
finite non-negative integer const ticks using the fixed `syminfo.mintick`
subset. Stage 7 Slice 20 adds `backtest_fill_limits_assumption=N` with finite
non-negative integer const ticks for supported limit-order verification.
`strategy.entry(id, strategy.long, qty=...)` is a strategy-mode side effect;
`qty` may be omitted when a supported fixed, cash, or percent-of-equity default
quantity subset is configured. Cash default quantities resolve as cash divided
by current close under the current no-currency-conversion boundary, and explicit
`qty` overrides the declaration default. The supported market-long entry creates
an internal pending entry, emits no public order while pending, and fills at the
next historical bar open before script statements on that fill bar.
`strategy.entry(..., limit=price)` creates an internal pending long limit entry,
emits no public order while pending, never fills on its creation bar, and fills
at the limit price before script statements on a later historical bar when
`low <= limit`, or below the configured verified limit threshold.
`strategy.entry(..., stop=price)` creates an internal pending
long stop entry, emits no public order while pending, never fills on its
creation bar, and fills at the stop price before script statements on a later
historical bar when `high >= stop`. `strategy.entry(..., stop=price,
limit=price)` creates an internal pending long stop-limit entry, activates an
internal limit order before script statements on a later historical bar when
`high >= stop`, does not fill on that activation bar, and fills at the limit
price before script statements on a later historical bar when `low <= limit`.
Same-calculation absolute `strategy.exit` attachment may target the active
pending market, limit, stop, or stop-limit entry id.
`strategy.close(id)` closes the full matching long position at the current bar
close. `strategy.close(id, qty=...)` and
`strategy.close(id, qty_percent=...)` can close part of the matching current
long position; both quantity forms must be finite and positive, fixed `qty` wins
when both forms are supplied, and oversized quantities clamp to the current
matching position size.
`strategy.close_all()` closes the current supported long position at the current
bar close without requiring an entry id; while flat or already closed it is a
no-op. It cancels pending exits for the closed entry and keeps the existing
public strategy output shape.
`strategy.cancel(id)` cancels matching internal pending entry ids and matching
pending exit ids in the supported order subset. Filled ids, unknown ids, and
already-cancelled ids are no-op, and cancellation does not expose public
pending-order records.
`strategy.cancel_all()` cancels all currently supported internal pending entries
and pending exits. Calling it without pending orders is a no-op and does not
expose public pending-order or cancellation records.
`strategy.exit(id, from_entry, stop=price)`,
`strategy.exit(id, from_entry, limit=price)`,
`strategy.exit(id, from_entry, profit=ticks)`, and
`strategy.exit(id, from_entry, loss=ticks)` support the current long-only
full-position single-trigger exit subset. Phase R also supports exactly one
downside leg plus one upside leg in a single bracket:
`stop + limit`, `stop + profit`, `loss + limit`, and `loss + profit`. Supported
single-trigger, bracket, and trailing exits with explicit fixed `qty` or
`qty_percent` can keep multiple pending exits for different `id + from_entry`
identities on the current matching long entry. Same-calculation absolute `stop`,
`limit`, and `trail_price` attachment may target the active pending entry id;
`profit`, `loss`, and `trail_points` attachment to a pending entry remains
unsupported until deferred price resolution is designed. Their reserved
quantities are resolved at placement time, the sum of reservations is clamped to
the current open position or matching pending entry quantity, and new
zero-reservation placements are rejected without changing existing pending
exits. Same-identity calls replace the existing pending exit.
New or replaced exits ignore same-bar triggers and can fill on a later
historical bar when `low <= stop/loss price` or `high >= limit/profit price`.
Same-side touched exits fill in placement order. If downside stop/loss and
upside limit/profit candidates are both touched on the same eligible historical
bar, only downside candidates fill on that bar in placement order; opposite-side
candidates remain pending if a long position remains. If both bracket legs are
touched on the same eligible historical bar, the downside stop/loss leg fills
first. Profit and loss use positive tick distances converted from
`strategy.position_avg_price` with the fixed default `syminfo.mintick`. Phase S
also supports exactly two
trailing forms for the current long-only broker: `trail_price + trail_offset`
and `trail_points + trail_offset`. `trail_price` is an explicit activation
price, `trail_points` converts once from `strategy.position_avg_price`, and
`trail_offset` converts once to a fixed price distance using the same fixed
default `syminfo.mintick`; trailing exits activate on a later eligible bar,
never fill on the activation bar, ratchet upward only, and later fill when
`low <= active trailing stop`. Phase U accepts optional fixed `qty` on each of
those supported single-trigger, bracket, and trailing forms. Phase V accepts
optional `qty_percent` on the same supported trigger forms. Stage 4 accepts
`qty` and `qty_percent` together on those same supported trigger forms, with
fixed `qty` determining the reserved or filled quantity. Quantity forms evaluate
once at placement time after `id` and `from_entry`, must be finite and positive,
and store an absolute requested close quantity on the pending exit. When only
`qty_percent` is used, it resolves against the current open position size, or
the matching pending entry quantity for same-calculation absolute attachment, as
`target_quantity * qty_percent / 100.0`; values above 100 are allowed because
the fill closes no more than the current position. Omitted `qty` and omitted
`qty_percent` preserve full-position one-effective-pending behavior across
supported single-trigger, bracket, and trailing forms. Different identities
replace rather than append omitted full-position pending exits, and a later
omitted full-position exit clears earlier explicit reservations for the current
matching long entry. Filled exits close
`min(requested_quantity, current position size)`, leave any remaining long
position open at the same average price, record one order event and one closed
trade for the filled quantity, and clear the pending exit. These calls are
rejected in indicator scripts and user-defined functions. Short entries,
`strategy.exit` same-side pairs, 3+ trigger or invalid trailing combinations,
multiple pending exits outside explicit fixed `qty` or `qty_percent`
single-trigger/bracket/trailing exits, omitted-quantity multiple
reservations, reservation behavior outside that subset,
unmatched missing-entry pre-placement, entry-relative exit attachment to pending
entries, `strategy.order`, broker settings beyond
`initial_capital`, fixed default quantity, supported cash commission, and
fixed-tick slippage and limit verification,
realtime strategy handoff, and
strategy metrics beyond the Phase L position/profit/equity variables remain
unsupported except for the Phase O `strategy.closedtrades` and
`strategy.opentrades` count variables, the Stage 3 outcome count variables, the
Stage 7 script-visible trade field functions, and the gross profit/loss,
profit-percent, average-trade, and max run-up/drawdown variables.
`strategy.grossprofit` is a read-only strategy-mode `series float` that sums
positive realized closed-trade profit only; losing, flat, and current open
trades do not change it. `strategy.netprofit_percent`,
`strategy.grossprofit_percent`, and `strategy.grossloss_percent` divide the
corresponding realized amount by `initial_capital` and multiply by 100.
`strategy.avg_trade_percent`, `strategy.avg_winning_trade_percent`, and
`strategy.avg_losing_trade_percent` average per-closed-trade percentage
profit/loss values, using each closed trade's entry price times quantity as the
denominator; the losing variant returns positive loss percentages.
`strategy.max_contracts_held_all`, `strategy.max_contracts_held_long`, and
`strategy.max_contracts_held_short` report the maximum
contracts/shares/lots/units held over the whole trading range. In the current
long-only subset, `all` and `long` track the maximum supported filled long
entry quantity, while `short` stays `0` because short entries are unsupported.
`strategy.grossloss` is a read-only strategy-mode
`series float` that sums realized closed-trade losses as positive values;
winning, flat, and current open trades do not change it. `strategy.avg_trade`
is a read-only strategy-mode `series float` that returns average realized
profit/loss per closed trade and `na` before the first closed trade.
`strategy.avg_winning_trade` is a read-only strategy-mode `series float` that
returns average realized profit among winning closed trades only and `na`
before the first winning closed trade. `strategy.avg_losing_trade` is a
read-only strategy-mode `series float` that returns average realized loss among
losing closed trades only as a positive value and `na` before the first losing
closed trade. `strategy.max_runup` is a read-only strategy-mode `series float`
that returns the maximum intrabar equity run-up amount over the current
supported long-only trading interval, using the supported entry equity, the
minimum equity before that entry, and the highest high reached while the
supported position is open. `strategy.max_runup_percent` divides the supported
run-up amount by entry price times current supported position quantity and
multiplies by 100.
`strategy.max_drawdown` is a read-only strategy-mode
`series float` that returns the maximum intrabar equity drawdown amount over
the current supported trading interval, using the supported entry equity, the
maximum equity before that entry, and the lowest low reached while the
supported position is open. `strategy.max_drawdown_percent` divides the
supported drawdown amount by entry price times current supported position
quantity and multiplies by 100. Other percent variants remain
unsupported. The count variables are
read-only strategy-mode `series int` values for the current long-only broker:
`strategy.closedtrades` counts closed trades recorded by broker state;
`strategy.wintrades`, `strategy.losstrades`, and `strategy.eventrades` count
closed trades with positive, negative, and zero realized profit; and
`strategy.opentrades` is `1` while the supported long position is open and `0`
when flat. The supported closed-trade namespace functions are
`strategy.closedtrades.entry_price`, `strategy.closedtrades.entry_id`,
`strategy.closedtrades.exit_price`, `strategy.closedtrades.exit_id`,
`strategy.closedtrades.entry_bar_index`, `strategy.closedtrades.exit_bar_index`,
`strategy.closedtrades.entry_time`, `strategy.closedtrades.exit_time`,
`strategy.closedtrades.commission`,
`strategy.closedtrades.size`, `strategy.closedtrades.profit`, and
`strategy.closedtrades.max_runup`, and `strategy.closedtrades.max_drawdown`;
they accept a zero-based integer `trade_num` and return `na` for missing,
negative, out-of-range, or non-integer indexes. `commission` returns `0.0`
without configured commission or supported entry-plus-exit commission when
configured. `max_runup` returns the largest
high-based favorable excursion retained for the closed trade quantity.
`max_drawdown` returns the largest low-based adverse excursion retained for the
closed trade quantity. `entry_id` returns the retained entry id, and `exit_id`
returns the retained close or exit id. Other trade details and
open-trade namespace functions remain unsupported except for
`strategy.opentrades.entry_price`, which returns the current supported long
position's entry price for `trade_num == 0`,
`strategy.opentrades.entry_id`, which returns its retained entry id, and
`strategy.opentrades.entry_bar_index`, which returns its entry fill bar, and
`strategy.opentrades.entry_time`, which returns its entry fill timestamp, and
`strategy.opentrades.size`, which returns the current open position size, and
`strategy.opentrades.profit`, which returns the current close-based floating
profit for that open position, and `strategy.opentrades.commission`, which
returns `0.0` without configured commission or the current open supported entry
commission when configured, and
`strategy.opentrades.max_runup`, which returns the largest high-based favorable
excursion seen so far for that open position, and
`strategy.opentrades.max_drawdown`, which returns the largest low-based adverse
excursion seen so far for that open position. All field functions return `na`
when flat or invalid. `strategy.opentrades.capital_held` is a read-only
strategy-mode variable and returns `na` in the current no-margin subset. With
explicit active `margin_long`, the current long-only subset returns `0.0` while
flat and current open long market value times `margin_long / 100` while open.
The same active `margin_long` setting also constrains supported long entry
fills at their actual fill price and supports the first long-only forced
liquidation subset. After a margin call, `capital_held` reflects the remaining
open long position. Short margin behavior remains unsupported.
Phase M and
Phase N keep pending-order records, partial fill fields, and exit reason fields
outside the public output model, and Phases R, S, U, V, W, X, and Y keep that
public contract unchanged for brackets, trailing exits, fixed `qty` exits,
percent `qty_percent` exits, and explicit fixed-quantity or percent-quantity
single-trigger/bracket/trailing reservations. Phase Z keeps the same public
contract unchanged for omitted-quantity replacement boundaries.
Diagnostics should describe the current strategy subset, not old phase names.

`indicator(...)` and `strategy(...)` declarations are mutually exclusive and
must be top-level. Declaration calls inside functions or local blocks are
semantic errors.

## Qualifiers

Qualifiers describe how a value changes over execution:

```text
const < input < simple < series
```

Promotion rule:

```text
The result qualifier of an expression is the strongest qualifier among the
operands and the called function's declared behavior.
```

Examples:

```pine
1 + 2              // const int
input(20)          // input int
input.int(20) + 1  // input int
input.string("S")  // input string
input.time(0)      // input int
close + 1          // series float
ta.sma(close, 20)  // series float
```

The analyzer must validate function arguments against declared qualifier
constraints. A `series` argument must not be accepted where a `simple` or
`input` value is required unless that specific built-in signature allows it.

## Numeric Coercion

Initial coercion rules:

- `int` can promote to `float`.
- `float` must not silently narrow to `int`.
- Arithmetic on `int` and `float` returns `float` when either side is `float`.
- Division returns `float`.
- Modulo requires numeric operands and should follow the selected Pine version's
  documented behavior.

The analyzer should emit diagnostics for unsupported or ambiguous coercions
instead of deferring them to runtime.

## Loops

`while` statements are statement-only in the current executable subset:

```pine
while condition
    statement
```

The condition must be `bool`. The loop body has its own local scope, and
`break`/`continue` use the same nearest-loop control-flow rules as `for`.
Runtime execution enforces an iteration guard; the semantic analyzer does not
try to prove termination.

## Operators

Arithmetic operators:

```text
+ unary plus
- unary minus
+ addition
- subtraction
* multiplication
/ division
% modulo
```

Comparison operators:

```text
== != > >= < <=
```

Logical operators:

```text
and or not
```

Conditional operator:

```text
condition ? true_expr : false_expr
```

The ternary result kind should be the least common compatible kind of both
branches. Its qualifier should be the strongest qualifier among the condition
and both branch expressions.

Switch expressions:

```pine
value = switch
    close > open => high
    close < open => low
    => close

value = switch direction
    1 => high
    -1 => low
    => close
```

The current executable subset supports expression arms only. Selector-less arm
conditions must be `bool`. Selector-form cases are compared with equality in
source order. Arm result kinds must have a common compatible kind, following the
same branch merge rules as ternary expressions. The result qualifier is the
strongest qualifier among the selector or conditions and the selected result
expressions.

## Arrays

The current array subset supports float, int, bool, string, color, label-id,
line-id, linefill-id, box-id, table-id, and chart-point arrays:

```pine
var values = array.new_float()
array.push(values, close)
values.push(close)
first = array.get(values, 0)
same = values.get(0)
count = array.size(values)

var counts = array.new_int()
counts.push(bar_index)

var flags = array.new_bool()
flags.push(close > open)

var names = array.new_string()
names.push("seed")

var shades = array.new_color()
shades.push(color.red)

var labels = array.new_label()
labels.push(label.new(bar_index, high, "label"))

var lines = array.new_line()
lines.push(line.new(bar_index, low, bar_index + 1, high))

var fills = array.new_linefill()
fills.push(linefill.new(lines.get(0), line.new(bar_index, high, bar_index + 1, low), color.blue))

var boxes = array.new_box()
boxes.push(box.new(bar_index, high, bar_index + 1, low))

var tables = array.new_table()
tables.push(table.new(position.top_right, 1, 1))
```

`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`, and
`array.new_color` return runtime-owned scalar array ids. `array.new_label`,
`array.new_line`, `array.new_linefill`, `array.new_box`, and `array.new_table`
return runtime-owned drawing-id arrays with `na` as the default initial value.
The supported scalar and drawing-id array constructors can also be written with
official `array.new<type>` syntax for float, int, bool, string, color, label,
line, linefill, box, and table. `array.new<chart.point>` returns a runtime-owned
chart-point array id. `array.from` allocates a runtime-owned array id with an
element kind inferred from its arguments; at
least one non-`na` supported typed value is required, `na` may be mixed into an
otherwise typed array, mixed int/float arguments produce a float array, and
label, line, linefill, box, or table ids infer the matching drawing-id array.
Normal declarations allocate a fresh array whenever the declaration executes.
`var` declarations preserve the array id and backing storage across bars.
Supported operations are
`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`,
`array.new_color`, `array.new_label`, `array.new_line`, `array.new_linefill`,
`array.new_box`, `array.new_table`, `array.new<chart.point>`,
`array.from`, `array.push`, `array.get`, `array.set`, `array.size`,
`array.insert`, `array.pop`, `array.remove`, `array.shift`, `array.unshift`,
`array.fill`, `array.first`, `array.last`, and `array.copy`, `array.slice`,
`array.concat`, `array.includes`, `array.indexof`, `array.lastindexof`,
`array.every`, `array.some`, `array.binary_search`, `array.binary_search_leftmost`,
`array.binary_search_rightmost`, `array.abs`, `array.min`, `array.max`,
`array.sum`, `array.avg`, `array.range`, `array.median`, `array.mode`,
`array.variance`, `array.stdev`, `array.percentile_nearest_rank`,
`array.percentile_linear_interpolation`, `array.percentrank`,
`array.covariance`, `array.standardize`, `array.sort`, `array.sort_indices`,
`array.reverse`, `array.join`, and `array.clear`;
`size/get/set/insert/push/pop/remove/shift/unshift/fill/first/last/copy/slice/concat/includes/indexof/lastindexof/reverse/join/clear`
may also be called with method syntax on a supported array receiver. Numeric
`binary_search/binary_search_leftmost/binary_search_rightmost/abs/min/max/sum/avg/range/median/mode/percentile_nearest_rank/percentile_linear_interpolation/percentrank/covariance/standardize/variance/stdev`
helpers may also be called with method syntax on float and int arrays.
`sort/sort_indices` may also be called with method syntax on float, int, and
string arrays.
`every/some` may also be called with method syntax on float, int, and bool
arrays.
Local user-defined types are part of the executable Phase J subset only for
top-level scalar `int`/`float`/`bool`/`string`/`color` fields. `Type.new(...)`
constructs runtime values, field reads are typed from the local UDT
declaration, and ordinary variables, local for-expression constructor results,
plus `var` may hold those values. Local scalar fields can be reassigned with
`value.field := expr` outside UDF/method bodies; the assigned expression must be
compatible with the declared field type. UDF parameter passthrough is supported
when the function returns the UDT
parameter itself, when a block-bodied function returns a local alias chain that
starts from that parameter, or when a nested passthrough UDF call maps back to
that parameter. Pure UDFs may also construct and return a local UDT value,
directly, through nested pure constructor-helper UDF calls, or through
same-local-UDT ternary, switch, final if/else constructor branches, or final
for bodies, from local UDT parameter scalar fields, scalar fields read through
block-local UDT aliases of those parameters, block-local scalar aliases of
those fields, parameters whose scalar types are inferred at the callsite, or
block-local scalar aliases of those scalar parameters, using positional or
named constructor field arguments.
Positional and named UDF call arguments both preserve the parameter identity,
so returned UDT values can be assigned and field-read at the callsite. UDT
history references, field mutation inside UDFs or methods, `varip`, nested UDT
fields, UDT arrays, and imported UDT identity remain outside the claim.

Pure user-defined methods are supported for local UDT receivers with scalar or
local UDT parameters, including direct UDT passthrough returns, block-local
receiver or local UDT parameter alias passthrough returns, nested method UDT
parameter passthrough returns, and local UDT constructor returns, directly,
through nested pure constructor-helper UDF calls, or through same-local-UDT
ternary, switch, final if/else constructor branches, or final for bodies, from
receiver or local UDT parameter scalar fields, scalar fields read through
block-local receiver or local UDT parameter aliases, block-local scalar aliases
of those fields, inferred scalar parameters, or block-local scalar aliases of
those parameters using positional or named constructor field arguments. The
receiver is analyzed as the first internal argument and the method body lowers
through the existing inlined UDF path, so callsite state and side-effect checks
follow the local function rules. When a
pure method returns the receiver itself, a block-local alias chain that starts
from the receiver or another local UDT parameter, another local UDT parameter,
a nested method passthrough call that maps back to one of those parameters, or
a local UDT constructed directly, through a nested pure constructor-helper UDF
call, or through same-local-UDT ternary, switch, final if/else constructor
branches, or final for bodies, from receiver or local UDT parameter scalar
fields, scalar fields read through block-local receiver or local UDT parameter
aliases, block-local scalar aliases of those fields, inferred scalar
parameters, or block-local scalar aliases of those parameters,
the callsite keeps that UDT identity so the returned value can be assigned and
field-read. Methods with side effects, recursion, unsupported
parameter families, mismatched UDT parameter identity, unknown receivers, and
imported method tables remain rejected. Non-array method calls outside the
local UDT method subset continue to fail with receiver/type diagnostics.
Float arrays accept int or float values and store them as floats. Int arrays
accept int values. Bool arrays accept bool values. String
arrays accept string values. Color arrays accept color values. Label and line
arrays accept matching drawing ids or `na`; `array.copy` is shallow for those references.
Other array
constructors and unsupported `array.*` functions are rejected. Array assignment
and UDF argument binding pass the runtime array id by reference. `array.copy`
allocates a new array id with the same current element values, so later
mutations do not affect the source. `array.slice` returns a same-kind shallow
window over the parent array's half-open `[index_from, index_to)` range; slice
mutations mirror the parent window, slice insertions widen the window, invalid
creation bounds return `na`, and later parent mutations that move the window
out of bounds are runtime errors. `array.concat` requires two arrays of the
same kind, appends the second array's current values to the first array in
place, and returns the first array id. `array.get`, `array.set`, `array.insert`,
and `array.remove`
support negative indexes from the array end. `array.insert` inserts before a
valid index, appends when the positive index equals the current size, and is a
no-op for invalid indexes. `array.remove` removes and returns a valid indexed
element, or returns `na` when the index is invalid. `array.fill`
replaces all elements by default or a half-open `[index_from, index_to)` window
when bounds are supplied; invalid ranges are no-ops.
`array.indexof` and `array.lastindexof` return `-1` when the value is not
present. Numeric binary search helpers are limited to float and int arrays and
expect the current array contents to be sorted ascending. `array.binary_search`
returns `-1` when the value is not found; leftmost/rightmost return the nearest
existing insertion-side index and return `-1` for empty arrays.
`array.every` and `array.some` are limited to float, int, and bool arrays;
false, zero, and `na` values are falsey, other numeric values are truthy, and
empty arrays return `true` for `every` and `false` for `some`. Numeric helpers
`array.abs`, `array.min`, `array.max`, `array.sum`, `array.avg`, `array.range`,
`array.median`, `array.mode`, `array.covariance`, `array.standardize`,
`array.variance`, and `array.stdev` are limited to float and int arrays.
`array.abs` allocates a new same-kind array, preserves `na`, and leaves the
source array unchanged. `array.covariance` requires two same-size numeric
arrays, skips pairs where either side is `na`, defaults to a biased population
estimate, and returns `na` when no numeric pair remains or an unbiased sample
has fewer than two numeric pairs.
`array.standardize` allocates a new float array, uses non-`na` values to
calculate mean and population standard deviation, preserves `na` element
positions when at least one numeric value exists, and returns an empty array
for empty/all-`na` arrays. The remaining helpers ignore `na` elements and
return `na` when no numeric element is present. `array.range` returns max minus
min. `array.mode`
returns the smallest value among tied most-frequent values and returns `na`
when all remaining values occur only once. Percentile helpers operate on
non-`na` values sorted ascending. Percentages outside `0..=100`,
empty/all-`na` arrays, and invalid `array.percentrank` indexes return `na`.
`array.variance` and `array.stdev`
accept an optional `biased` bool argument that defaults to `true`; passing
`false` uses the sample denominator and returns `na` when fewer than two
numeric values remain.
`array.sort` and `array.sort_indices` support float, int, and string arrays,
sort ascending by default, and accept `order.ascending` or `order.descending`.
`na` values and empty strings sort last in ascending order and first in
descending order. `array.sort` mutates the source array; `array.sort_indices`
returns a new int array of original indexes in sorted order and leaves the
source array unchanged.
`array.reverse` reverses any supported typed array in place. `array.join`
converts supported array elements to string with the default numeric format,
uses `,` as the default separator, and returns an empty string for empty arrays.
Color elements render as normalized integer color values. Out-of-range
`array.get`, empty `array.pop`, empty `array.shift`, and `array.first`/`array.last` on empty arrays
return `na`; out-of-range `array.remove` returns `na`; out-of-range
`array.set`, `array.insert`, and `array.fill` are no-ops. Negative array sizes fail at
runtime. Each array can contain at most 100,000 elements; creation, push,
unshift, insert, or concat operations beyond that limit fail at runtime.

User-defined functions may receive supported arrays and use read-only
operations such as `array.size` and `array.get`. Array mutation inside
user-defined functions is rejected until function side-effect semantics are
broader.

## `na`

`na` behavior must be implemented deliberately. It must not be represented as
Rust `None` in the evaluator because `na` participates in Pine expressions.

Initial rules:

- Arithmetic with `na` returns `na`.
- Historical references outside available history return `na`.
- `na(x)` returns `true` when `x` is `na`, otherwise `false`.
- `nz(x)` returns `x` when `x` is not `na`; otherwise it returns the default
  replacement for the overload or the explicit replacement argument.
- `nz(x, replacement)` returns `replacement` when `x` is `na`.
- A condition that evaluates to `na` should follow the selected Pine version's
  documented behavior. Phase 1 should target one version policy and test it
  explicitly.

The analyzer should track Pine version because boolean `na` behavior differs
across versions. If a version-specific behavior is not implemented, it must be
reported as unsupported instead of approximated silently.

## Declarations

Normal declarations are evaluated whenever their containing scope executes:

```pine
x = close + open
```

The declaration creates or updates the current bar value for that symbol. If the
symbol is series-qualified, the current value is committed after bar execution.

Normal and tuple block-local declarations inside `if` blocks are executable and
scoped to their branch. The analyzer records resolved symbol bindings before
lowering so locals do not leak into outer scopes. Tuple declarations in local
scopes always create local symbols, even when an outer scope already contains the
same names.

`for i = start to end` is executable for integer ranges with an optional
explicit `by step`. The loop counter is a block-local integer symbol scoped to
the loop body. Range and step expressions must be int-typed, and literal zero
steps are rejected; series-qualified bounds are evaluated for the current bar
when the loop statement executes.
`break` and `continue` are valid only inside a loop and target the nearest
enclosing loop.
Loop expressions are supported for scalar and tuple declaration values when the
body ends with an expression. The expression determines the loop result type;
bodies that do not end with an expression are rejected with `E_LOOP_RETURN`.

## Reassignment

Reassignment updates an already resolved symbol:

```pine
x := x + 1
```

Rules:

- Reassignment to an unknown name is a semantic error.
- Reassignment must respect the symbol's declared value kind.
- The assigned expression may strengthen the qualifier only when the language
  rules allow the target to become series-qualified.
- Reassignment inside a local scope must resolve according to Pine scoping
  rules, not ordinary block-local shadowing assumptions.

## `var`

`var` declarations initialize once at the first execution of their declaration
site:

```pine
var x = 0
```

Rules:

- The declaration site owns persistent storage.
- The initializer runs only when the declaration site is first reached.
- Later executions read the persistent value.
- Local `var` declarations in blocks and inlined user-defined functions use
  declaration-site storage, with independent storage per syntactic UDF callsite.
- Reassignment writes the persistent value for subsequent bars.

`var` storage is separate from ordinary per-bar local values and from committed
series history.

## Function Calls

Each built-in and user-defined function call must have a stable callsite id.

Callsite ids are needed because many functions have stateful series behavior.
For example, two separate calls to `ta.ema(close, 20)` must not accidentally
share rolling state unless the language semantics require sharing.

The lowering stage should assign callsite ids before runtime execution.

## User-Defined Functions

User-defined functions are supported by lowering each callsite inline.
Expression bodies lower directly as expressions. Multi-statement block bodies
lower as block expressions that execute local statements and return the final
expression. Positional and named arguments are resolved to the declared
parameter list before semantic analysis of the function body.

Current rules:

- Resolve parameters and local symbols.
- Infer return kind and qualifier.
- Reject recursion.
- Allocate independent callsite state for every syntactic callsite.
- Functions called conditionally follow the same conditional callsite rules as
  built-ins.
- Resolve local declarations and reassignments inside function block bodies to
  callsite-local symbols.
- Preserve parameter shadowing when a function-local declaration or loop counter
  uses the same name as a parameter.
- Reject duplicate or unknown named UDF arguments and positional arguments after
  named arguments.
- Evaluate arguments once into callsite-local temporaries before evaluating the
  inlined function body.
- Reject output side effects inside functions.
- Reject global reassignment inside functions.
- Reject side-effecting calls as UDF arguments.

Function block bodies must end with an expression.
