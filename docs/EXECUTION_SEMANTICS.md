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
aliases, UDF calls, output/drawing side effects, input declarations, and array
mutation inside requested expressions remain unsupported.

### `varip`

`varip` requires precise realtime tick semantics. It is rejected until intrabar
persistence is implemented as a separate realtime state partition. It must not
be approximated with `var`.

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
that parameter. Recursive functions, output side effects and drawing side
effects inside functions, global reassignment inside functions, and
side-effecting calls as UDF arguments are rejected in the current executable
subset.

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
