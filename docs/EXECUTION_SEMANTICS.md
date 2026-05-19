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

Declarations inside `if` blocks are rejected in the current executable subset.
This avoids accidentally treating block-local values as global symbols before
full local-scope semantics are implemented.

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

`var` declarations initialize once, then preserve state across bars. The runtime
must store this in a persistent state table separate from ordinary per-bar
locals.

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
```

Inlining gives stateful calls inside the function body independent callsite
state for each syntactic UDF call. Named arguments are resolved before
lowering. Multi-statement functions, recursive functions, output side effects
inside functions, and stateful or side-effecting calls as UDF arguments are
rejected in the current executable subset.

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
