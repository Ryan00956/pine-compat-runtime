# Series Model

This document defines how time-series values, history references, and stateful
call sites are represented.

Pine-compatible execution is not ordinary expression evaluation. The runtime
must distinguish:

- current values for the bar being evaluated
- committed historical values from previous bars
- persistent `var` storage
- state owned by built-in and user-defined function callsites

## Core Rule

```text
For each bar:
  set current built-in series values
  execute the program for the current bar
  collect side effects for the current bar
  commit current series values
```

Historical references read committed values, not mutable current locals from a
previous execution frame.

## Series Identity

A series id may belong to:

- a built-in series such as `close`
- a user variable whose value is series-qualified
- a compiler-generated temporary expression that needs history
- a function callsite with a series result
- a plot output series

The analyzer or lowering stage should assign stable ids:

```rust
struct SeriesId(u32);
struct CallSiteId(u32);
```

The runtime should not infer series identity from source text at execution time.

## Current Values

During one bar execution, the runtime maintains current values separately from
committed history:

```rust
struct Frame {
    current: ValueMap,
}
```

`x[0]` reads the current value of `x` for the current bar. `x[1]` reads the last
committed value from the series store.

## Commit

At the end of each bar:

- series-qualified variables commit their current value
- series-qualified built-in values are already known for the bar and commit
- stateful callsite outputs commit when their result is series-qualified
- plot series commit the value emitted for that bar

If a declaration site is not reached on a bar, the implementation must define
whether its current value is `na` or whether the previous current value carries.
Phase 1 should avoid unsupported ambiguous cases by rejecting scripts that rely
on conditionally skipped series declarations until this behavior is specified
and tested.

## History References

History reference syntax:

```pine
expr[offset]
```

Initial rules:

- `offset == 0` reads the current value.
- `offset > 0` reads committed history.
- out-of-range history returns `na`.
- negative offsets are rejected.
- dynamic offsets are rejected until Phase C defines retention and qualifier
  rules.

The lowering stage should determine whether `expr` needs a compiler-generated
series id. For example:

```pine
(close + open)[1]
ta.sma(close, 20)[1]
```

Both expressions need stable storage if accepted.

The current implementation boundary is tracked in
`docs/HISTORY_SERIES_AUDIT.md`.

## Callsite State

Many built-ins need state per syntactic callsite:

```pine
a = ta.ema(close, 20)
b = ta.ema(close, 20)
```

`a` and `b` have two distinct callsites. Their state must be separate even when
their arguments are textually identical.

Stateful built-ins include, at minimum:

- `ta.ema`
- `ta.rma`
- `ta.rsi`
- `ta.atr`
- rolling highest/lowest implementations if optimized incrementally

The initial implementation may compute some built-ins from committed history for
simplicity, but the public behavior must match the callsite-state model.

## Conditional Calls

Conditional calls are a compatibility hazard:

```pine
x = close > open ? ta.ema(close, 20) : na
```

Before supporting them broadly, define whether the callsite advances state only
when executed or whether the analyzer should require stateful series calls to be
evaluated on every bar. Phase 1 may reject stateful calls in conditional or
looped execution contexts with a clear diagnostic.

## Persistent State

`var` state is keyed by declaration site, not by variable name:

```rust
struct VarSlotId(u32);
```

This distinction matters once local scopes and functions are supported. A `var`
inside a function must have storage behavior defined per declaration site and
callsite.

## Buffer Sizing

Phase 1 can use growable buffers for correctness.

Later phases should add:

- static history-depth inference for constant offsets
- a runtime guard for unsupported dynamic offsets
- optional `max_bars_back` handling
- memory limits and diagnostics for excessive history

## Realtime Preview

Realtime execution adds a second layer:

- forming bar updates
- rollback to last committed state
- `varip`
- intrabar side effects

Historical execution should be completed first. Realtime-only features must be
rejected or explicitly marked approximate until rollback semantics are tested.
