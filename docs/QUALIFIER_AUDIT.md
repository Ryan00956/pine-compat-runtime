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
- History offsets accept non-negative integer literals plus integer expressions
  at any implemented qualifier, including `series int`.
- Scalar array, scalar slice, label-array, label-slice, line-array,
  line-slice, linefill-array, box-array, and table-array ids can now receive series storage
  for fixture-backed array history snapshots. Polyline arrays, remaining
  non-scalar slice history, map/matrix values, and broader aliasing rules remain
  undesigned.

## Impact On Dynamic History

Dynamic history offsets now use an explicit integer-kind policy:

- `const int`: supported; non-negative literals lower to a constant offset,
  while other const int expressions are evaluated by the runtime guard.
- `input int`: supported with runtime validation.
- `simple int`: supported with runtime validation.
- `series int`: supported with runtime validation and conservative full-history
  retention up to the runtime cap.
- non-integer offsets remain rejected.

## Phase C Closeout

The history-offset qualifier policy is implemented and fixture-covered for
const, input, simple, and series integers. Remaining qualifier work is not a
blocker for Phase C history support:

1. Add a precise helper for qualifier-bound argument acceptance so built-in
   signatures can say "at most input" or "at most simple" without bespoke enum
   variants for every kind.
2. Keep `docs/BUILTIN_SIGNATURES.md` aligned with code acceptors as new
   built-ins are added, because built-in length parameters already rely on
   `SimpleInt` semantics.
3. Revisit scalar `simple` inference if later built-ins require stricter Pine
   qualifier behavior than the current subset.
