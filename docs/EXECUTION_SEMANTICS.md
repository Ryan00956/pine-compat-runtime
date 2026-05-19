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
is reached on a bar. The range is inclusive. The runtime steps by `1` for
ascending ranges and `-1` for descending ranges unless an explicit non-zero int
`by step` is provided. The counter is scoped to the loop body.
`break` exits the nearest enclosing loop. `continue` skips the rest of the
current iteration and advances to the next loop counter value.

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
hanging execution.

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
`array.new_float` declaration allocates a fresh array each time it executes. A
`var` array declaration keeps the same id and backing storage across bars, so
mutations such as `array.push` or `values.push(...)` persist.

Array bounds are stable in the current subset: `array.get` outside the current
array length returns `na`, `array.set` outside the current length is ignored,
and `array.pop` on an empty array returns `na`. A negative `array.new_float`
size is a runtime error. Runtime execution limits each float array to 100,000
elements; oversized creation and `array.push` beyond the limit return runtime
errors.

Read-only array operations are allowed inside inlined user-defined functions.
The supported method-call syntax lowers to the same `array.*` runtime calls, so
the same bounds, persistence, and UDF side-effect rules apply.
Array mutation inside user-defined functions is rejected as a function
side-effect boundary.

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
that parameter. Recursive functions, output side effects inside functions,
global reassignment inside functions, and side-effecting calls as UDF arguments
are rejected in the current executable subset.

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
length = input.int(20, "Length")
mode = input.string("SMA", "Mode")
start = input.time(0, "Start")
```

The runtime should collect input metadata during compilation or a dry run, then
execute with host-provided input values. Inputs should carry the `input`
qualifier.

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

The runtime should collect:

- numeric series
- horizontal lines
- fills
- bar colors
- background colors
- later markers and shapes

## Determinism

A compiled program must produce the same result for the same bars and inputs.
Host time, network access, randomness, and file system access should not exist
in the core runtime.
