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
- Dynamic offsets that evaluate to a negative integer fail at runtime.
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
  history, series-qualified offsets, loop-produced offsets, user-defined
  function parameters, and realtime forming rollback.

## Current Rejections

- Negative literal offsets such as `close[-1]` are rejected with
  `negative_history_offset`.
- Non-integer dynamic offsets such as `close[close]` are rejected with
  `dynamic_history_offset`.
- Scalar array, scalar slice, and label-array variable history snapshots are
  fixture-backed for the official `previous = a[1]` and
  `na(previous) ? na : previous.get(0)` pattern. Other object arrays, map,
  matrix, drawing-object collection, and non-scalar slice-history snapshots
  remain undesigned.
- Per-variable `max_bars_back` inference and declarations are not implemented.

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
- runtime profiles expose the retention mode and HIR history requirement fields

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
  presence, and `max_bars_back`.
- Added fixture coverage for historical, incremental, and realtime rollback
  paths.

Deferred:

- Per-variable `max_bars_back` declarations and inference.
- Remaining object, map, matrix, drawing-object collection, and non-scalar
  slice-history snapshots.
- More precise diagnostics when a dynamic offset asks for history beyond an
  explicit retention bound.

## Acceptance Criteria For Expanding History

- The supported subset is represented in `tests/fixtures/conformance.tsv`.
- Every accepted offset form has semantic and runtime fixture coverage.
- Unsupported variants fail during semantic analysis with stable diagnostics.
- Incremental append execution matches full historical execution.
- Realtime rollback keeps history, `var`, callsite state, and outputs
  consistent for confirmed bars and forming-bar updates.
