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
profit-percent, average-trade, max run-up/drawdown, and buy-and-hold return
variables.
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
`strategy.buy_and_hold_return_percent` is a read-only strategy-mode
`series float` that returns the current close's percentage change from the
first loaded bar close and returns `na` when that baseline is zero or
non-finite.
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

Range `for` loops execute over inclusive integer bounds. Statement-form
`for...in` loops are fixture-backed for `array<int>`, `array<float>`,
`array<bool>`, `array<string>`, `array<color>`, drawing-id object arrays,
`array<chart.point>`, and same-local or same-imported scalar-tree UDT array
values:

```pine
for value in values
    statement
```

The narrow index/value form is fixture-backed for `array<int>`, `array<float>`,
`array<bool>`, `array<string>`, `array<color>`, `array<label>`, `array<line>`,
`array<linefill>`, `array<polyline>`, `array<box>`, `array<table>`,
`array<chart.point>`, and same-local or same-imported scalar-tree UDT arrays
only. The index loop-local is a zero-based `series int` for the current visited
slot:

```pine
for index, value in values
    statement
```

The iterable is evaluated once before the loop starts. Runtime snapshots the
initial array length, visits indexes from `0` to `initial_len - 1`, reads each
element when that index is reached, and assigns the element by value to the
loop-local variable. `break` and `continue` use the same nearest-loop
control-flow rules as range `for` and `while`; loop-body local declarations are
scoped to the loop body; stateful built-in calls in the body advance at each
reached iteration for that callsite. Empty arrays and typed `na` array iterables
execute zero iterations. Mutating the iterated array through any
alias affects not-yet-visited existing indexes, appends do not extend the
current loop, and shrinkage that makes a future initial index out-of-bounds
raises the same runtime error as `array.get`. Label, line, linefill, polyline,
box, and table array loop values are shallow-copied ids, so drawing setters or
lifecycle operations through the loop local mutate the same drawing object while
assignment to the loop local does not write the source array slot. Chart-point
array and same-local or same-imported scalar-tree UDT array loop values are
copied into the loop-local variable, so local field mutation does not write back
to the source slot. Expression-form `for value in values` supports only `array<int>`,
`array<float>`, `array<bool>`, `array<string>`, `array<color>`,
`array<label>`, `array<line>`, `array<linefill>`, `array<polyline>`,
`array<box>`, `array<table>`, `array<chart.point>`, and same-local
scalar-tree UDT array iterables, plus runtime-owned matrix row iterables, in
the current subset. Matrix expression-form iteration binds the loop value to an
independent row snapshot array. The optional expression-form index local is the
same zero-based `series int` slot number used by statement-form index/value
iteration. It returns the last expression from the last
completed iteration, returns `na` for empty arrays, empty matrices, or typed
`na` iterables, returns the previous completed result on `break`, and skips the
current result expression on `continue`. Index/value iteration over iterables
other than `array<int>`,
`array<float>`, `array<bool>`, `array<string>`, `array<color>`,
`array<label>`, `array<line>`, `array<linefill>`, `array<polyline>`,
`array<box>`, `array<table>`, `array<chart.point>`, same-local or same-imported
scalar-tree UDT arrays, runtime-owned matrix rows, or scalar maps,
expression-form `for...in` beyond the scalar-array, drawing-id-array,
chart.point-array, same-local or same-imported scalar-tree UDT-array, matrix-row,
and scalar-map subset, non-array/non-matrix/non-map iterables, non-scalar-tree
UDT arrays, other non-scalar arrays, and broader collection mutation families
remain outside the current subset.
Ordinary
`var` scalar arrays roll back loop-body mutation during repeated forming
realtime updates, while scalar typed-array
`varip` iteration preserves carried intrabar loop-body mutation between repeated
forming updates. The scalar-array, label-array, line-array, linefill-array,
polyline-array, box-array, and table-array shallow-id fixtures, chart-point-array
value-copy fixture, and UDT-array value-copy fixture have explicit incremental
append execution parity with full historical recomputation.
Inside a local UDF or typed local user method, `for...in` over a same-local
scalar-tree UDT-array parameter gives each value loop local the concrete
element identity resolved for that call. Value-only and index/value statement
loops, block-local aliases of the parameter, and final expression-form loops
returning a UDT field/scalar result, the UDT element itself, or a same-identity
UDT array rebuilt from that element are supported. Named method arguments and
interleaved A-to-B-to-A calls preserve their own field layouts, returned
element identities, and rebuilt array identities.

`while` supports statement loops and a scalar expression subset:

```pine
while condition
    statement
```

The condition must be `bool`. The loop body has its own local scope, and
`break`/`continue` use the same nearest-loop control-flow rules as `for`.
`while` expressions must have a final body expression. Scalar, tuple,
same-local UDT, scalar-array, and runtime-owned matrix result subsets return
the latest reached final body expression, or `na` if no iteration produces a
value. Callers may read or mutate a returned scalar array or returned
`matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, or
`matrix<color>` id with supported APIs, including fresh arrays/matrices,
existing scalar-array or matrix aliases, zero-iteration `na`, break/continue
result preservation, and fresh historical copies from committed history reads.
Stateful callsites in a reached expression-loop body advance on each reached
iteration, and body-local declarations including local `var` declaration sites
follow the same loop-local storage rules as statement bodies. Nested collection
interactions through while-expression results remain outside the current subset,
with nested-array and imported UDT constructor result expressions rejected
during semantic analysis even though top-level scalar-tree imported UDT
constructors are supported.
Bodies without a final result expression remain rejected until explicit
semantics are fixture-backed.
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

value = switch
    close > open =>
        local = high
        local
    => close

switch direction
    1 =>
        total := total + high
    =>
        total := total + low
```

The current executable subset supports expression arms in condition and selector
forms, plus expression statement-block arms that end with a result expression.
Statement-context switch block arms can execute selected condition, selector,
and default arms for side effects, outer reassignment, and loop control without
requiring a final result expression. Selector-less arm conditions must be
`bool`. Selector-form cases are compared with equality in source order. Arm
result kinds must have a common compatible kind for expression-context switches,
following the same branch merge rules as ternary expressions. The result
qualifier is the strongest qualifier among the selector or conditions and the
selected result expressions. Same-imported-identity UDT constructor or
block-local alias results are supported for expression statement-block arms;
local/imported identity mismatches remain rejected during semantic analysis.

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
Same-local scalar-tree UDT arrays may be declared with `array<T>` or `T[]`
when initialized with `na` or a same-UDT array expression; the declaration keeps
the concrete local UDT identity for later assignment and helper checks.
Same-local and same-imported scalar-tree UDT array identities also flow through
ternary, `if`, `switch`, `for`, `for...in`, and `while` results. Array/`na`
branches and block-local aliases retain the known element identity for typed or
inferred declarations, helper calls, history, and iteration; different element
identities produce an `E_BRANCH_TYPE` diagnostic instead of an unlowerable HIR.
Generic UDF lowering resolves array parameters, local flow aliases, element
helper results, and `array.from` reconstruction against the current call's UDT
identity rather than shared function-body span metadata.
Local pure UDF and user-method return analysis extends that rule to same-local
scalar-tree UDT arrays, while imported pure exported UDF and imported
user-method return analysis covers same-imported scalar-tree UDT arrays. The
fixture-backed return paths include direct parameters, block-local aliases,
`array.copy`, `array.new<T>`/`array.new<alias.Type>`, `array.from`, private nested
calls, typed methods with named/reordered arguments, and final control-flow
expressions. Positional, named, and reordered arguments seed a call-local
identity environment. Imported type positions are rewritten for the active
alias, and source-aware expression metadata separates import instances, so
repeated calls over different field orders or two aliases of the same physical
library preserve the concrete A-to-B-to-A layout instead of inheriting another
call's span metadata. Mixed return identities and incompatible explicitly typed
destinations remain semantic errors. Tuple returns may contain same-local or
same-imported scalar-tree UDT arrays: tuple literals and UDF/method direct,
block, nested, and final-control-flow results preserve one concrete identity per
destructured slot, including typed-`na` locals and distinct UDT identities in
different slots. Tuple-valued ordinary declarations also retain their element
types and per-slot identities through direct and self aliases,
ternary/`switch` results, assigned `if` results, shadowing, and later tuple
destructuring. The first declaration fixes each UDT-array slot identity.
Same-identity or `na` reassignment preserves it, while direct or control-flow
reassignment to a different identity and unresolved nested tuple consumers
emit root-spanned `E_TUPLE_UDT_ARRAY_IDENTITY` diagnostics. Qualified same-local
user-method and imported UDF/method results carrying a concrete scalar-tree
UDT-array identity support direct `.first()` and `.copy()` dispatch, including
a nested `.copy().first()` chain; explicit same-named local methods and imported
functions remain distinct. Non-scalar UDT array returns, unqualified local UDF
results, other direct call-result array methods, and mutation through
unsupported UDF/method side-effect contexts remain outside this subset.
Generic UDT-array parameters are therefore iterable inside local UDFs and typed
local methods for the fixture-backed statement and final-expression forms,
including final results that return the UDT element itself or rebuild a
same-identity array from that element.
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
Same-local scalar-tree UDT array `includes`, `indexof`, and `lastindexof`
compare UDT values structurally across every scalar field using the runtime
value equality relation. Different local UDT identities remain incompatible
even when their field shapes match.
Same-local scalar-tree UDT array `fill` replaces the whole array or a valid
half-open range with a same-local UDT value; values from different local UDT
identities remain rejected.
Same-local scalar-tree UDT arrays and same-imported scalar-tree UDT arrays
constructed through `array.from` support `join` stringification. Each element is
rendered as `TypeName(field0, field1, ...)`, using field declaration order and
the existing scalar `array.join` formatting for field values. This does not
enable `str.tostring(UDT)`.
Field mutation on a UDT value read from a same-local scalar-tree UDT array
mutates only that local value; it does not change the source array slot unless a
later same-UDT `array.set`/`set()` explicitly writes the value back.
Direct chained slot field mutation is supported for
`array.get(points, index).field := value` and `points.get(index).field := value`
when `points` is a same-local scalar-tree UDT array; it mutates a copy of the
selected slot and writes that updated UDT value back to the same array index,
including slice-window parent mirroring. The same operation remains rejected
inside UDFs under the function side-effect policy.
When a same-local scalar-tree UDT value is read from a UDT array, that value
may be passed to local pure UDFs that read scalar fields, passthrough the value,
or return a constructed same-local UDT. When bound to a local variable, that
local value may also be used as the receiver for local pure UDT methods. Local
and imported UDT constructor or method call-result receiver chains, such as
`Point.new(...).method(...)` or `lib.Point.new(...).method(...)`, are
fixture-backed for same-identity pure methods, including scalar and UDT
caller-side history reads. Broader non-constructor call-result receiver
expressions such as `array.get(...).method(...)` remain outside the parser
subset.
Local user-defined types are part of the executable Phase J subset only for
top-level scalar `int`/`float`/`bool`/`string`/`color` fields. `Type.new(...)`
constructs runtime values, field reads are typed from the local UDT
declaration, and ordinary variables, local for-expression and while-expression
constructor results, top-level/block-local/loop-local typed declarations
initialized from same-local-UDT ternary, switch, or `if` expressions,
top-level/block-local/loop-local typed declarations initialized or reassigned
from same-local-UDT `for` expressions, plus `var` declarations initialized from
`na`, same-UDT constructors, same-UDT ternary expressions, same-UDT switch
expressions, same-UDT `if` expressions, same-UDT `for` expressions, same-UDT
`for...in` expressions, or same-UDT `while` expressions may hold those values.
Different local UDT declarations remain distinct identities even when their
field shapes match. Mismatched assignment, typed initializer, nested UDT field
assignment, constructor argument, ternary branch, switch arm, and final-if branch
paths are rejected with fixture-backed user-facing diagnostics that name the
failed identity boundary.
Explicitly typed same-local scalar-tree UDT `varip` declarations initialized
from `na`, same-UDT constructors, same-identity aliases, or fixture-backed
same-UDT ternary/switch/if/for/for...in/while expressions, plus
direct-constructor-inferred or direct-alias-inferred same-local scalar-tree UDT
`varip` declarations, may also hold those values and persist them intrabar by
value.
Local scalar fields can be reassigned with `value.field := expr` in top-level,
branch, `for` loop, `while` loop, UDF-local variable bodies, and method-local
variable bodies; the assigned expression must be compatible with the declared
field type. UDF parameters may carry explicit same-local UDT types for the
same value-flow subset as inferred parameters. UDF parameter passthrough is
supported when the function returns the UDT
parameter itself, when a block-bodied function returns a local alias chain that
starts from that parameter through a block-local, ternary-expression, final-if, final-for,
final-for-in, final-while, or switch-expression alias, or when a nested passthrough UDF call
maps back to that parameter through those same alias forms. Pure UDFs may also construct and return a local UDT value,
directly, through nested pure constructor-helper UDF calls, or through
same-local-UDT ternary, switch, `if` expression, final if/else constructor
branches, final for bodies, final for-in bodies, or final while bodies, from
local UDT parameter scalar fields, scalar fields read through block-local UDT
aliases of those parameters, block-local scalar aliases of those fields,
parameters whose scalar types are explicitly declared or inferred at the
callsite, or block-local scalar aliases of those scalar parameters, using
positional or named constructor field arguments.
Positional and named UDF call arguments both preserve the parameter identity,
so returned UDT values can be assigned and field-read at the callsite.
Same-local scalar-tree UDT values read from UDT arrays may also be passed to
local pure UDFs, including passthrough and constructor-return UDFs.
Same-local or same-imported scalar-tree UDT arrays may be declared as `varip`
when the declaration carries explicit UDT array identity, allowing realtime
handoff to retain the array id, backing contents, and UDT metadata between
forming updates.
UDF-local and method-local UDT variables may mutate scalar fields before
returning the updated value. Local scalar-tree UDT value history references,
including dynamic and `na` offsets plus local scalar-tree UDT field-produced history offsets and UDF- and method-returned passthrough and constructor-returned values, including nested scalar-tree UDT returns, plus same-UDT `if`, `switch`, `for`,
`for...in`, and `while` expression results, global or
parameter field mutation inside UDFs, receiver/parameter/global field mutation
inside methods, non-constructor-inferred UDT `varip`, nested-field UDT `varip`,
and non-scalar UDT arrays remain outside the claim.
Imported UDT identity is supported for scalar-tree constructors, direct and nested field-read,
ordinary same-imported-UDT reassignment, explicit scalar-tree imported typed
declarations, and same-imported scalar-field typed array declarations, plus
direct UDF parameter passthrough, imported UDF block-local,
ternary-expression, final-if, final-for, final-for-in, final-while, or
switch-expression alias passthrough,
nested UDF parameter passthrough over those forms, exported-function same-imported UDT
typed parameters, same-imported scalar-tree UDT array typed UDF parameters
including named arguments and caller-side history reads from returned array elements,
same-imported scalar-tree UDT array typed method parameters
including named arguments and caller-side history reads from returned array elements,
and direct, nested, ternary, if, for, for-in, while, or switch constructor-return results, and
same-imported-identity ternary,
`if`, `switch`, `while`, `for`, or `for...in` expression results with
caller-side history reads, ordinary
imported UDT `var` declarations, scalar-tree imported UDT `varip` declarations,
same-imported scalar-tree UDT array `varip` declarations, and scalar-tree
root-field replacement in top-level, branch, `for`-loop, `while`-loop, and UDF-local
  statement contexts, plus method-local scalar-tree root-field replacement and scalar-tree
  value history, including dynamic and `na` offsets, imported scalar-tree UDT
  field-produced history offsets from direct/nested imported fields and fields on
  imported UDF- and method-returned values, and UDF- and method-returned
passthrough and constructor-returned
values, plus `array.from` size/get/first/last plus set replacement field
reads, push append field reads, unshift prepend field reads, insert insertion
field reads, fill replacement field reads, join positional stringification,
includes/indexof/lastindexof structural equality search, sort/sort_indices by
int/float/string sort_field, pop/remove/shift return field reads, clear-size
reset, copy independent field reads, reverse reordered field reads, slice
window field reads, concat appended field reads, and
statement/expression/index-value for-in value-copy field reads;
local/imported structural lookalikes remain distinct assignment identities.
Exported imported UDTs whose scalar-tree metadata depends on private library
UDTs can be carried as typed `na` values and read through value history without
exposing the private dependency. Local and imported non-scalar UDT identities can
also be carried as direct, `var`, or explicit typed-na `varip` values, flow through ternary,
`if`, `switch`, `for`, `for...in`, and `while` identity results, pass through
local UDFs, imported exported UDFs, local methods, and imported receiver-style
or alias-qualified methods including same-imported non-receiver method
parameters, read through history, expose direct fields from typed `na` values
through field reads/history, and be tested with `na()`, while their constructors
remain outside the supported non-scalar subset.
The imported collection claim also includes same-imported scalar-tree UDT array
returns from imported UDFs and user methods within the direct/alias,
copy/new/from, private-nested, typed-method, final-control-flow, and dual-alias
isolation subset above. Imported UDT collections beyond those returns and the
scalar-tree `array.from` size/get/first/last, set-replacement, push-append,
unshift-prepend, insert-insertion, fill-replacement, join-stringification,
search-structural-equality, sort-by-field, pop/remove/shift return, clear-size,
copy-read, reverse-read, slice-window, concat-append, and for-in-value-copy
subset remain outside the claim. The same applies to direct private imported UDT
access, mixed or non-scalar imported array-return identities, conflicting
identities within one tuple UDT-array slot, unqualified local UDF or
non-`first`/`copy` direct call-result array methods, nested field mutation, UDF
parameter/global field side effects, and method receiver/parameter/global field
side effects. Qualified same-local user-method and same-imported UDT-array
results support direct `.first()`/`.copy()`.
Tuple-contained
same-imported scalar-tree UDT arrays are supported when destructured, with
identity tracked independently per slot. Non-scalar UDT value history outside the local/imported
label/line/box/chart.point-field fixture with direct `chart.point` field chains,
and imported UDT value history outside the scalar-tree metadata and typed-`na`
non-scalar identity subsets, also remain outside the claim.

Pure user-defined methods are supported for local UDT receivers with scalar,
`chart.point`, scalar array, object-id array, chart.point array, local UDT,
same-local scalar-tree UDT array, or same-imported scalar-tree UDT array typed
parameters, including direct `chart.point` constructor/passthrough returns with
caller-side history reads, direct UDT passthrough returns with caller-side history reads, nested scalar-tree UDT method returns with caller-side history reads, block-local
or ternary-expression receiver or local UDT parameter alias passthrough
returns, final if/else, final for, final for-in, final while, or
switch-expression local UDT alias passthrough returns, nested method UDT
parameter passthrough returns, and local and nested scalar-tree UDT
constructor returns with caller-side history reads, directly,
through nested pure constructor-helper UDF calls, or through same-local-UDT
ternary, switch, `if` expression, final if/else constructor branches, final for
bodies, final for-in bodies, or final while bodies, from receiver or local UDT
parameter scalar fields, scalar fields read through block-local receiver or
local UDT parameter aliases, block-local scalar aliases of those fields,
inferred scalar parameters, or block-local scalar aliases of those parameters
using positional or named constructor field arguments. The
receiver is analyzed as the first internal argument and the method body lowers
through the existing inlined UDF path, so callsite state and side-effect checks
follow the local function rules. Scalar method returns may be used as dynamic
history offsets, including returned `na` offsets, and method-returned scalar
series values may be read through constant, dynamic, or `na` history offsets at
the caller. Method returns preserve fixture-backed `input` and `simple`
qualifiers from scalar and simple-string parameters through expression,
block-local, final-if, final-loop, and switch block return shapes when the
receiver is not used in the returned value. Method final-if branch and switch
block loops plus final `for`, `for...in`, and `while` loop returns include the
loop header, iterable, or condition qualifier, so series-controlled method loop
results remain rejected by simple-only arguments. When a
pure method returns the receiver itself, a block-local alias chain that starts
from the receiver or another local UDT parameter, a ternary-expression alias of
one of those values, another local UDT parameter, a nested method passthrough
call that maps back to one of those parameters, or a local UDT constructed
directly, through a nested pure constructor-helper UDF call, or through
same-local-UDT ternary, switch, final if/else constructor
branches, same-local-UDT `if` expressions, final for bodies, final for-in
bodies, or final while bodies, from receiver or local UDT parameter scalar
fields, scalar fields read through block-local receiver or local UDT parameter
aliases, block-local scalar aliases of those fields, inferred scalar
parameters, or block-local scalar aliases of those parameters,
the callsite keeps that UDT identity so the returned value can be assigned and
field-read. Same-local scalar-tree UDT values read from UDT arrays can also be
bound to locals and used as local pure method receivers. Receiver-style and
alias-qualified scalar-tree imported UDT method calls, including the same method
name on different scalar-tree receiver types, are supported when the imported
method stays inside the scalar/imported-UDT parameter subset and the
alias-qualified form receives a same-identity imported UDT as its first
argument; non-receiver method parameters may use named or reordered arguments,
including direct receiver passthrough or same-identity parameter
passthrough returns, block-local, ternary-expression, final-if, final-for,
final-for-in, final-while, or switch-expression receiver or parameter alias passthrough
returns, nested imported method passthrough returns,
direct, nested scalar-tree, ternary, if, for, for-in, while, or switch
same-imported-identity constructor returns, and method-local scalar field
mutation before returning a local UDT value. Returned imported UDT values in
that subset support caller-side history reads followed by scalar-tree field reads.
Local methods may return same-local scalar-tree UDT arrays from typed array
parameters or fresh local array construction. Direct and block-alias returns
retain the argument identity, while copy/new/from and nested/final-control-flow
returns preserve the identity selected for the current call. Qualified
same-local method results support direct `.first()`/`.copy()` without widening
arbitrary call-result receivers. The narrow qualified same-imported UDT-array
path supports the same helper pair; unqualified local UDF results and other
helpers remain gated.
Methods with receiver/parameter/global field side effects, recursion,
unsupported parameter families, mismatched UDT parameter identity, unknown
receivers, and alias-qualified imported method receiver type mismatches remain
rejected. Non-array
method calls outside the local/imported UDT method subset continue to fail with
receiver/type diagnostics.
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
valid index, appends when the positive index equals the current size, and
raises a runtime error for out-of-bounds indexes. `array.remove` removes and
returns a valid indexed element, or raises a runtime error when the index is
out of bounds. `array.fill`
replaces all elements by default or a half-open `[index_from, index_to)` window
when bounds are supplied; invalid ranges are no-ops.
`array.includes`, `array.indexof`, and `array.lastindexof` use structural
equality for same-local scalar-tree UDT arrays and same-imported scalar-tree
UDT arrays constructed through `array.from`; `array.indexof` and
`array.lastindexof` return `-1` when the value is not present. Numeric binary
search helpers are limited to float and int arrays and
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
plus same-local scalar-tree UDT arrays and same-imported scalar-tree UDT arrays
constructed through `array.from` when a compile-time `sort_field` names an int,
float, or string field. They sort ascending by default and accept
`order.ascending` or `order.descending`.
`na` values and empty strings sort last in ascending order and first in
descending order. `array.sort` mutates the source array; `array.sort_indices`
returns a new int array of original indexes in sorted order and leaves the
source array unchanged.
`array.reverse` reverses any supported typed array in place. `array.join`
converts supported array elements to string with the default numeric format,
uses `,` as the default separator, and returns an empty string for empty arrays.
Color elements render as normalized integer color values. Same-local
scalar-tree UDT arrays and same-imported scalar-tree UDT arrays constructed
through `array.from` use `TypeName(field0, field1, ...)` element rendering.
Out-of-range
`array.get`, `array.set`, `array.insert`, and `array.remove` on an existing
array raise runtime errors; empty `array.pop`, empty `array.shift`, and
`array.first`/`array.last` on empty arrays return `na`. Out-of-range
`array.fill` ranges are no-ops. Negative array sizes fail at runtime. Each
array can contain at most 100,000 elements; creation, push, unshift, insert, or
concat operations beyond that limit fail at runtime.

User-defined functions may receive supported arrays and use read-only
operations such as `array.size` and `array.get`. Array mutation inside
user-defined functions is rejected until function side-effect semantics are
broader.

The current map subset supports runtime-owned empty map ids through
`map.new<K, V>()` where both template types are one of `int`, `float`, `bool`,
`string`, or `color`. `map.size(id)` accepts map ids and returns the current
entry count. `map.put`, `map.get`, and `map.contains` are supported for those
scalar key/value templates: put inserts or replaces the entry for an equal key,
get returns the current value or `na` for a missing key, and contains returns a
series bool. `map.clear` removes all entries from the map id and the same id can
be reused by later `map.put` calls. `map.remove` deletes a present key and is a
no-op for missing keys. Assignment passes the runtime map id by reference, and
`map.copy` returns an independent cloned backing store carrying the same scalar
key/value template. `map.keys` and `map.values` return independent array
snapshots in insertion order using the map's scalar key or value kind.
`map.put_all` requires source and target maps to have the same scalar key/value
template and mutates the target by replacing existing values without moving
their keys and appending new keys in source insertion order. Ordinary realtime
rollback clones the confirmed runtime map store for each forming update, so
unconfirmed map mutations do not leak across forming executions. Equivalent
method aliases lower to the same namespace calls for the supported subset.
Direct `for key in id` and `for [key, value] in id` map iteration snapshots the
current insertion order of scalar key/value pairs for statement and expression
loops. A single loop local receives the key using the map key template kind. In
key/value form, the first loop local receives the key and the second receives the
value using the map value template kind. If the loop body changes the map size,
runtime execution reports an error.
Scalar `map<K,V>` typed declarations preserve map template metadata for `na`
initialization and same-template assignment. Scalar bare `map` declarations
initialized from a known scalar map expression preserve the inferred template
metadata. Scalar map history references preserve that template metadata for
historical map receivers. Non-scalar templates, non-map map receivers, and bare
map declarations without a known scalar map initializer remain unsupported with
targeted diagnostics. Scalar map `varip` declarations are supported for the same
scalar key/value template subset. User-defined function parameters receive
the caller's scalar map template metadata, enabling read-only map helpers inside
pure UDF bodies while mutating map helpers remain side-effect rejected.

User-defined function parameters may also use typed array templates in canonical
`array<T>` form or `T[]` aliases for supported scalar, object-id, `chart.point`,
same-local scalar-tree UDT, and same-imported scalar-tree UDT array element
kinds. The parameter receives the caller's array id plus element-kind or UDT
identity metadata, so read-only array calls type-check through the typed
parameter and mismatched array element kinds or UDT array identities are
rejected at analysis time.

The current matrix subset supports runtime-owned float matrix ids through
`matrix.new<float>(rows, columns, initial_value?)`, `matrix.get`,
`matrix.set`, `matrix.fill`, `matrix.copy`, `matrix.transpose`,
`matrix.reverse`, `matrix.reshape`, `matrix.kron`, `matrix.mult`,
`matrix.diff`, `matrix.pow`,
`matrix.add_row`,
`matrix.add_col`, `matrix.remove_row`, `matrix.remove_col`,
`matrix.swap_rows`, `matrix.swap_columns`, `matrix.sort`,
`matrix.submatrix`,
`matrix.rows`, `matrix.columns`, `matrix.elements_count`, `matrix.is_square`,
`matrix.is_binary`, `matrix.is_diagonal`, `matrix.is_identity`,
`matrix.is_symmetric`, `matrix.is_antisymmetric`, `matrix.is_stochastic`,
`matrix.is_zero`, `matrix.sum`, `matrix.avg`, `matrix.min`, `matrix.max`,
`matrix.mode`, `matrix.trace`, `matrix.det`, `matrix.eigenvalues`,
`matrix.eigenvectors`, `matrix.inv`, `matrix.pinv`, `matrix.rank`,
`matrix.row`, and `matrix.col`.
Matrix cells
hold float or `na` values; int cell inputs are coerced to float. Matrix
assignment and UDF
argument binding pass the runtime matrix id by reference, while `matrix.copy`
allocates an independent matrix store snapshot of the current cells.
The subset also supports runtime-owned int matrix ids through
`matrix.new<int>(rows, columns, initial_value?)` plus `matrix.get`,
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
`matrix.rank`, including the corresponding supported method aliases. Int matrix
cells hold int or `na` values, and the
int initial value plus `matrix.set`/`matrix.fill` write values must be
int-compatible.
The subset also supports runtime-owned bool matrix ids through
`matrix.new<bool>(rows, columns, initial_value?)` plus `matrix.get`,
`matrix.set`, `matrix.fill`, `matrix.copy`, `matrix.transpose`,
`matrix.reverse`, `matrix.reshape`, `matrix.submatrix`, `matrix.row`,
`matrix.col`, `matrix.add_row`, `matrix.add_col`, `matrix.remove_row`,
`matrix.remove_col`, `matrix.swap_rows`, `matrix.swap_columns`, `matrix.rows`,
`matrix.columns`, `matrix.elements_count`, and `matrix.is_square`, including
the corresponding supported method aliases. Bool matrix cells hold bool or
`na` values, and the bool initial value plus `matrix.set`/`matrix.fill` write
values must be bool-compatible.
`matrix.transpose` allocates an independent matrix store with swapped row and
column counts.
`matrix.reverse` mutates the existing matrix in place, preserving shape while
moving `(row, column)` to `(rows - 1 - row, columns - 1 - column)`.
`matrix.swap_rows` mutates the existing matrix in place by exchanging two
validated row ranges, preserving shape and treating same-row or zero-column
swaps as no-ops after row validation.
`matrix.swap_columns` mutates the existing matrix in place by exchanging two
validated column positions across all rows, preserving shape and treating
same-column or zero-row swaps as no-ops after column validation.
`matrix.sort` mutates the existing matrix in place by reordering complete row
ranges according to a selected column, defaults to column `0`, accepts
`order.ascending` and `order.descending`, preserves original row order for
equal keys, and places `na` keys last ascending and first descending.
`matrix.submatrix` returns an independent matrix copy of a selected half-open
row/column range, defaulting omitted bounds to the source matrix's full row or
column extent and allowing empty row or column slices.
`matrix.row` and `matrix.col` return independent row/column snapshots:
`array<float>` for float matrices, `array<int>` for int matrices, and
`array<bool>` for bool matrices. Ordinary
`var` matrix ids persist across bars, and
realtime forming-bar rollback restores the confirmed matrix store for
non-`varip` updates. Matrix construction rejects negative row or column counts
and is bounded by a 100,000-cell runtime budget before allocation.
`matrix.set`, `matrix.fill`, `matrix.reverse`, `matrix.reshape`,
`matrix.add_row`, `matrix.add_col`, `matrix.remove_row`,
`matrix.remove_col`, `matrix.swap_rows`, `matrix.swap_columns`, and
`matrix.sort` inside
user-defined functions remain rejected by the collection side-effect boundary.
`values.fill(value)`, `values.get(row, column)`, `values.rows()`,
`values.columns()`, `values.elements_count()`, `values.is_square()`,
`values.is_binary()`, `values.is_diagonal()`, `values.is_identity()`,
`values.is_symmetric()`, `values.is_antisymmetric()`,
`values.is_stochastic()`, `values.is_zero()`, `values.sum()`,
`values.avg()`, `values.min()`, `values.max()`, `values.mode()`, and
`values.trace()`, `values.det()`, `values.eigenvalues()`,
`values.eigenvectors()`, `values.kron(other)`, `values.mult(other)`,
`values.diff(other)`, `values.pow(power)`,
`values.inv()`,
`values.pinv()`, and `values.rank()` are
supported as method-call aliases for the matching
namespace calls.
`values.set(row, column, value)` is
also supported as a
method-call alias for `matrix.set(values, row, column, value)` outside
user-defined functions, `values.copy()` is supported as a method-call alias for
`matrix.copy(values)`, `values.transpose()` lowers to
`matrix.transpose(values)`, `values.reverse()` lowers to
`matrix.reverse(values)`, `values.reshape(rows, columns)` lowers to
`matrix.reshape(values, rows, columns)`, `values.row(row)` lowers to
`matrix.row(values, row)`, and `values.col(column)` lowers to
`matrix.col(values, column)`, and `values.add_row(row, array_id)` lowers to
`matrix.add_row(values, row, array_id)`, and
`values.add_col(column, array_id)` lowers to
`matrix.add_col(values, column, array_id)`, and `values.remove_row(row)` lowers
to `matrix.remove_row(values, row)`, and `values.remove_col(column)` lowers to
`matrix.remove_col(values, column)`, and `values.swap_rows(row1, row2)` lowers
to `matrix.swap_rows(values, row1, row2)`, and
`values.swap_columns(column1, column2)` lowers to
`matrix.swap_columns(values, column1, column2)`, and
`values.sort(column?, order?)` lowers to
`matrix.sort(values, column?, order?)`, and `values.submatrix(...)` lowers to
`matrix.submatrix(values, ...)`, and `values.kron(other)` lowers to
`matrix.kron(values, other)`, and `values.mult(other)` lowers to
`matrix.mult(values, other)` for matrix or scalar-right operands, and
`values.diff(other)` lowers to `matrix.diff(values, other)` for matrix or
scalar-right operands, and `values.pow(power)`
lowers to `matrix.pow(values, power)`. Reshape preserves element order and
element count. `matrix.kron` accepts runtime-owned float or int matrix operands
and returns an independent Kronecker-product `matrix<float>` with expanded
shape, propagates `na` or non-finite source cells to `na` result cells,
preserves zero-dimension results, and is bounded by the 100,000-cell matrix
budget. Matrix-by-matrix `matrix.mult` accepts runtime-owned float or int
matrix operands and returns an independent `matrix<float>` product with shape
`left.rows()` by `right.columns()`, requires
`left.columns() == right.rows()`, propagates `na` or non-finite contributing
cells to `na` result cells, preserves zero-dimension results, and is bounded by
the 100,000-cell matrix budget. Scalar namespace `matrix.mult` accepts a
numeric or `na` scalar on either side of a matrix operand and returns an
independent same-shape `matrix<float>`. The `values.mult(scalar)` method alias
supports the scalar-right form. `matrix.mult(values, vector)` and
`values.mult(vector)` accept a right-hand `array<float>` or `array<int>` as a
column vector, require `vector.size() == values.columns()`, and return an
independent `array<float>` whose length is `values.rows()`. Namespace
`matrix.mult(vector, values)` accepts a left-hand numeric array as a row vector,
requires `vector.size() == values.rows()`, and returns an independent
`array<float>` whose length is `values.columns()`. Namespace
`matrix.mult(left_vector, right_vector)` accepts numeric array pairs with equal
length, treats them as a row vector and column vector, and returns an
independent single-element `array<float>` dot-product result. Non-numeric-array
`matrix.mult` overloads remain unsupported.
Matrix-by-matrix `matrix.diff` accepts runtime-owned float or int matrix
operands and returns an independent `matrix<float>` element-wise difference
with matching operand shape, requires identical row and column counts,
propagates `na` or non-finite source cells to `na` result cells, and preserves
zero-dimension results. Scalar namespace `matrix.diff` accepts a numeric or
`na` scalar on either side of a matrix operand and returns an independent
same-shape `matrix<float>`. The `values.diff(scalar)` method alias supports
the scalar-right form.
`matrix.pow` accepts runtime-owned square float or int matrices and returns
independent `matrix<float>` powers, with
power `0` producing an identity matrix, power `1` producing an independent
copy, larger powers using matrix multiplication `na` propagation, empty
`0 x 0` inputs returning empty `0 x 0` results, and runtime errors for
non-square matrices or negative powers.
`matrix.elements_count` returns the current row-count by
column-count element count, including zero for zero-dimension matrices.
`matrix.is_square` returns whether row and column counts are equal.
`matrix.is_zero` returns true when every stored numeric cell is zero, false for
any non-zero or `na` cell, and true for zero-element matrices.
`matrix.is_binary` returns true when every stored numeric cell is exactly zero
or one, false for any other numeric value or `na` cell, and true for
zero-element matrices.
`matrix.is_diagonal` returns true when every cell outside the main diagonal is
zero, false for any non-zero or `na` off-diagonal cell, allows any
main-diagonal value, does not require a square shape, and returns true for
zero-element matrices.
`matrix.is_identity` returns true only for square matrices whose main diagonal
cells are exactly one and whose off-diagonal cells are exactly zero, false for
any `na` cell, false for non-square matrices, and true for empty `0 x 0`
matrices.
`matrix.is_symmetric` returns true only for square matrices whose stored
numeric cells match their transposed counterparts, false for any `na` cell,
false for non-square matrices, and true for empty `0 x 0` matrices.
`matrix.is_antisymmetric` returns true only for square matrices whose main
diagonal cells are exactly zero and whose off-diagonal cells are the negatives
of their transposed counterparts, false for any `na` cell, false for non-square
matrices, and true for empty `0 x 0` matrices.
`matrix.is_stochastic` returns true when every cell is a finite non-negative
number and either every row sums exactly to one or every column sums exactly to
one, returns false for any `na` or negative cell, and returns false for
zero-element matrices.
`matrix.sum` returns the sum of numeric cells in row-major order,
ignoring `na` cells and returning `na` when no numeric cells exist;
`matrix.avg` returns the average over the same non-`na` numeric cell set.
`matrix.min` and `matrix.max` scan the same non-`na` numeric cell set.
`matrix.mode` returns the smallest most-frequent non-`na` numeric cell when a
value repeats and otherwise returns `na`.
`matrix.trace` sums non-`na` numeric cells on the main diagonal over
`min(rows, columns)` positions and returns `na` when the diagonal has no
numeric cells.
`matrix.det` computes the determinant of square runtime-owned float or int
matrices, returns `1.0` for empty `0 x 0` matrices, returns `na` for any `na`
or non-finite cell, and raises a runtime error for non-square matrices.
`matrix.eigenvalues` returns an independent `array<float>` of real eigenvalues
for square runtime-owned float or int matrices, returns an empty array for
empty `0 x 0` matrices, returns `na` for any `na` or non-finite cell and for
non-real eigenvalue results, and raises a runtime error for non-square
matrices.
`matrix.eigenvectors` returns an independent `matrix<float>` whose columns are
real eigenvectors for square runtime-owned float or int matrices, returns an
independent empty `0 x 0` matrix for empty `0 x 0` input, returns `na` for any
`na` or non-finite cell and for non-real or incomplete eigenvector results, and
raises a runtime error for non-square matrices.
`matrix.inv` computes an independent inverse matrix for non-singular square
runtime-owned float or int matrices, returns an independent empty `0 x 0`
matrix for empty `0 x 0` input, returns `na` for any `na` or non-finite cell
and for singular matrices, and raises a runtime error for non-square matrices.
`matrix.pinv` computes an independent Moore-Penrose pseudo-inverse matrix with
row/column counts swapped from the source, returns an independent zero-cell
matrix for zero-row or zero-column input, returns `na` for any `na` or
non-finite cell, and supports singular and rectangular matrices.
`matrix.rank` computes the rank of rectangular runtime-owned float or int
matrices, returns `0` for zero-element matrices, and returns `na` for any `na`
or non-finite cell.
`values.sum()`, `values.avg()`, `values.min()`, `values.max()`,
`values.mode()`, `values.trace()`, `values.det()`, `values.eigenvalues()`,
`values.inv()`, `values.pinv()`, and `values.rank()` lower to the matching read-only
namespace helpers.
`matrix.add_row` copies an element-kind-matched row array into the matrix,
requires the row length to match the current column count, and inserts at an
index in `0..=rows` while preserving row order around the insertion. Float
matrices require `array<float>` row data, int matrices require `array<int>`
row data, and bool matrices require `array<bool>` row data. `matrix.add_col` copies an element-kind-matched column array into the
matrix, requires the column length to match the current row count, and inserts
at an index in `0..=columns` while preserving column order around the
insertion. Float matrices require `array<float>` column data, int matrices
require `array<int>` column data, and bool matrices require `array<bool>`
column data.
`matrix.remove_row` deletes an existing row using the same row-index bounds as
row reads. `matrix.remove_col` deletes an existing column using the same
column-index bounds as column reads. Matrix templates beyond `float`, `int`,
`bool`, `string`, and `color`, other method syntax, and bare matrix or matrix
templates beyond float/int/bool/string/color typed declarations remain outside
the current subset. `matrix<float>`, `matrix<int>`, `matrix<bool>`,
`matrix<string>`, and `matrix<color>` typed declarations accept compatible
matrix values or `na`; statement-form matrix `for...in` binds each loop value to
an independent row snapshot array and the optional index to the zero-based row
number. Matrix history is supported for committed matrix snapshots that return
fresh copies. Single `chart.point` value history returns retained previous point
values from direct constructors, UDF returns, user-defined method returns, or
`if`/`switch`/`for`/`for...in`/`while` expression results, supports dynamic
`na` offsets, and is independent from later mutation of the current point
value. Explicit `chart.point` typed
declarations accept compatible point values, `na`, and compatible control-flow
expression results.
Single `chart.point` value `varip` declarations persist
point values intrabar by value, including field-mutation writeback. UDF
parameters may declare scalar or `chart.point` types; typed `chart.point`
parameters preserve constructor-return and read-only passthrough value flow,
including caller-side field reads and history reads. Matrix
`varip` declarations are supported for the same runtime owned matrix element
kinds, with realtime backing-store handoff across forming updates.

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
