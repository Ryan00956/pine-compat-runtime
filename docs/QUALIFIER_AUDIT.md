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
- `AtMostInputNumeric` accepts `const` or `input` numeric values and rejects
  `simple`/`series` values.
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
- Built-in signature docs use descriptive Pine-like terms, while code still uses
  a smaller `Accepts` enum. The analyzer now has shared `qualifier_at_most` and
  kind-filter helpers, but not every Pine-style signature phrase has a distinct
  data-model variant.
- History offsets accept non-negative integer literals plus integer expressions
  at any implemented qualifier, including `series int`.
- Scalar array, scalar slice, label-array, label-slice, line-array,
  line-slice, box-slice, linefill-array, linefill-slice, polyline-array,
  polyline-slice, box-array, table-array, table-slice, chart.point-array,
  chart.point-slice, and same-local scalar-field UDT-array ids can now receive
  series storage for fixture-backed array history snapshots.
- Matrix history snapshots are fixture-backed for committed matrix values,
  dynamic matrix offsets including `na` offset predicates, and
  while-expression matrix results. Map values,
  UDT/imported-UDT value history, drawing-object collections beyond
  fixture-backed id arrays/slices, and broader aliasing rules remain
  undesigned or rejected.

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
const, input, simple, and series integers. Shared qualifier-bound argument
helpers back the current "at most input" and "at most simple" acceptors.
Remaining qualifier work is not a blocker for Phase C history support:

1. Keep the shared qualifier-bound argument helpers covered as more built-in
   signatures move from bespoke acceptors to "at most input" or "at most
   simple" semantics.
2. Keep `docs/BUILTIN_SIGNATURES.md` aligned with code acceptors as new
   built-ins are added, because built-in length parameters already rely on
   `SimpleInt` semantics.
3. Revisit scalar `simple` inference if later built-ins require stricter Pine
   qualifier behavior than the current subset.
