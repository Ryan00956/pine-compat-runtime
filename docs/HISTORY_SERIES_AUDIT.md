# History and Series Audit

This document records the Phase C boundary from
`docs/LONG_TERM_EXECUTION_PLAN.md`. Phase C now has a guarded dynamic integer
history subset, static retention inference, runtime retention profiles, and
indicator-level `max_bars_back` support.

## Current Supported Subset

- History references use `expr[offset]` syntax.
- The offset must be an integer literal greater than or equal to zero, or an
  integer expression at any implemented qualifier, including `series int`.
- `expr[0]` evaluates `expr` on the current bar.
- `expr[n]` for `n > 0` reads the committed value from `n` bars ago.
- dynamic offsets are accepted when the offset expression is an integer.
- Out-of-range history reads return `na`.
- Dynamic offsets that evaluate to `na` return `na`.
- Dynamic offsets that evaluate to a negative integer fail at runtime, including
  offsets produced by built-ins, UDF returns, or
  ternary/if/switch/for/while-expression results.
- Series-qualified identifiers keep stable series ids.
- Series-qualified non-identifier expressions that are lowered with history
  receive compiler-generated series ids.
- Lowering records HIR history metadata: program-wide `max_constant_offset`,
  whether dynamic offsets exist, and per-series history requirements.
- Runtime retention uses that metadata for scripts without dynamic offsets:
  each series keeps only the maximum constant offset it needs, and unindexed
  series keep no committed history.
- The metadata includes implicit history reads used by current runtime
  implementations of `ta.tr`, `ta.atr`, `ta.change`, and `ta.cross*`.
- Constant history is fixture-covered for built-in series, expression history,
  branch bodies, loop bodies, and user-defined function parameters.
- Dynamic integer history is fixture-covered for built-in series, expression
  history, series-qualified offsets, direct ternary-produced offsets including
  returned `na` offsets and result first-bar history predicates,
  branch-produced offsets including result first-bar history predicates,
  for-loop-produced offsets including result first-bar history predicates,
  while-loop-produced offsets including result first-bar history predicates,
  stateless built-in result history reads including direct-offset result
  first-bar predicates, stateful built-in result history reads including
  direct-offset result first-bar predicates and built-in-returned offsets with
  result first-bar predicates, built-in returned offsets including returned
  `na` offsets and result first-bar history predicates,
  user-defined function parameters including `na` offsets and result first-bar
  history predicates, plus returned offsets,
  UDF-returned `na` offsets, UDF-returned offset result first-bar history
  predicates, and realtime forming rollback.

## Current Rejections

- Negative literal offsets such as `close[-1]` are rejected with
  `negative_history_offset`.
- Non-integer dynamic offsets such as `close[close]`, `close[close > open]`,
  UDF-returned float offsets, built-in-returned float offsets, or
  ternary/if/switch/for/while-expression float results are rejected with
  `dynamic_history_offset`.
- Scalar array, scalar slice, label-array, label-slice, line-array,
  line-slice, box-slice, linefill-array, linefill-slice, polyline-array,
  polyline-slice, box-array, table-array, table-slice, chart.point-array,
  chart.point-slice, and same-local scalar-field UDT-array variable history
  snapshots are fixture-backed for the official `previous = a[1]` and
  `na(previous) ? na : previous.get(0)` pattern, including ordinary
  scalar-array, scalar-slice, label-array, label-slice, line-array, line-slice,
  box-array, box-slice, linefill-array, linefill-slice, polyline-array,
  polyline-slice, table-array, table-slice, chart.point-array,
  chart.point-slice, and same-local scalar-field UDT-array first-bar
  `na(previous)` predicate output, plus scalar array/slice, label-array/slice,
  line-array/slice, box-array/slice, linefill-array/slice,
  polyline-array/slice, table-array/slice, chart.point-array/slice, and
  same-local scalar-field UDT-array dynamic `na` offset predicates.
- Scalar-array and matrix while-expression result history snapshots include
  dynamic `na` offset predicates. Matrix history snapshots are fixture-backed
  for committed matrix values, dynamic matrix offsets including `na` offset
  predicates, shape-history dynamic `na` offset predicates, and
  while-expression matrix results. Map history,
  UDT/imported-UDT value history, drawing-object collections beyond
  fixture-backed id arrays/slices, and richer aliasing cases remain undesigned
  or rejected.
- Per-variable `max_bars_back` inference, declarations, and helper calls such
  as `max_bars_back(close, 20)` are not implemented; helper calls are rejected
  with `E_UNSUPPORTED_FEATURE`.

## Series Offset Policy

Series integer offsets are supported as a guarded dynamic subset:

- the offset expression is evaluated on the current bar
- `na` offsets return `na`
- negative offsets fail at runtime
- out-of-range offsets return `na`
- scripts with any dynamic offset keep full committed series history up to the
  configured runtime cap
- `indicator(..., max_bars_back=N)` bounds dynamic retention when `N` is a
  non-negative constant integer
- runtime profiles expose the retention mode, HIR history requirement fields,
  and dynamic-retention miss counters when a runtime offset exceeds the retained
  `max_bars_back` window

Static-only scripts still use HIR metadata to trim retention.

## Phase C Closeout

Completed:

- Hardened constant history coverage.
- Audited qualifier propagation for const, input, simple, and series values.
  Current findings are in `docs/QUALIFIER_AUDIT.md`.
- Audited and tightened built-in signature docs for implemented qualifier
  behavior.
- Implemented guarded dynamic integer history, including `series int` offsets.
- Added HIR history requirement metadata and runtime static retention trimming.
- Added indicator-level `max_bars_back` bounds for dynamic history retention.
- Added profile fields for retention mode, static depth, dynamic-offset
  presence, `max_bars_back`, and dynamic-retention misses.
- Added fixture coverage for historical, incremental, and realtime rollback
  paths.

Deferred:

- Per-variable `max_bars_back` declarations, helper calls, and inference.
- Map history plus richer object-collection aliasing cases beyond the
  fixture-backed array/slice and matrix snapshots.
- UDT and imported-UDT value history remains rejected until value identity and
  copy semantics are deliberately designed.
- More precise user-facing diagnostics when a dynamic offset asks for history
  beyond an explicit retention bound.

## Acceptance Criteria For Expanding History

- The supported subset is represented in `tests/fixtures/conformance.tsv`.
- Every accepted offset form has semantic and runtime fixture coverage.
- Unsupported variants fail during semantic analysis with stable diagnostics.
- Incremental append execution matches full historical execution.
- Realtime rollback keeps history, `var`, callsite state, and outputs
  consistent for confirmed bars and forming-bar updates.
