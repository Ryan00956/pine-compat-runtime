# Qualifier Audit

This document records the Phase C qualifier audit that follows
`docs/HISTORY_SERIES_AUDIT.md`.

## Implemented Qualifier Model

The IR has four qualifiers:

```text
const < input < simple < series
```

Current inference:

- literals and named constants are `const`
- `input.*` functions return `input` values, except `input.source`, which
  returns `series float`
- OHLCV, time components, derived price sources, and `bar_index` are `series`
- binary, ternary, and switch expressions use the strongest operand or branch
  qualifier
- history references always return `series` values
- array ids are `simple` values, while array element reads and aggregate helpers
  generally return `series` values
- user-defined functions are typed from the inlined body with callsite argument
  types

## Parameter Acceptance Rules

The analyzer validates built-in arguments using the `Accepts` enum in
`pine-builtins` and `accepts_type` in `pine-sema`.

Important current rules:

- Exact type checks allow weaker qualifiers to flow into stronger targets.
- `series` targets accept weaker same-kind values.
- `int` may widen to `float`; `float` does not narrow to `int`.
- `SimpleInt` accepts `const`, `input`, or `simple` integers and rejects
  `series int`.
- `SeriesFloat` requires an actual `series float`.
- Compatibility names such as `SeriesOrSimpleNumeric` currently mean any
  numeric qualifier at or below `series`; this includes `const` and `input`.
- Coarse acceptors such as `Numeric`, `Kind`, and `Array` do not narrow by
  qualifier beyond the value kind.

## Current Gaps

- Scalar `simple` inference is not complete. Most non-array scalar values are
  either `const`, `input`, or `series`.
- There is no separate runtime input immutability model beyond the qualifier
  assigned by semantic analysis.
- Built-in signature docs use descriptive Pine-like terms, while code uses a
  smaller set of coarse acceptors.
- History offsets accept non-negative integer literals plus `const int`,
  `input int`, and `simple int` dynamic offset expressions.
- Series-qualified array ids and array history snapshots are not designed.

## Impact On Dynamic History

The next dynamic-offset decision should not use "is this an integer?" alone.
It needs an explicit qualifier policy:

- `const int`: supported; non-negative literals lower to a constant offset,
  while other const int expressions are evaluated by the runtime guard.
- `input int`: supported with growable retention and runtime validation.
- `simple int`: supported with growable retention and runtime validation.
- `series int`: remains rejected until max-bars-back style retention,
  runtime validation, and memory limits exist.

## Recommended Next Steps

1. Add retention and memory-limit design for accepted const/input/simple
   dynamic offsets.
2. Add a precise helper for qualifier-bound argument acceptance so built-in
   signatures can say "at most input" or "at most simple" without bespoke enum
   variants for every kind.
3. Keep `close[bar_index]` and other `series int` offsets rejected until the
   runtime has explicit max-bars-back behavior.
4. Keep `docs/BUILTIN_SIGNATURES.md` aligned with code acceptors before
   broadening dynamic history, because built-in length parameters already rely
   on `SimpleInt` semantics.
