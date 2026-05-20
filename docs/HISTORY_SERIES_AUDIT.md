# History and Series Audit

This document starts Phase C from `docs/LONG_TERM_EXECUTION_PLAN.md`. It records
the current implementation boundary before changing dynamic history or qualifier
rules.

## Current Supported Subset

- History references use `expr[offset]` syntax.
- The offset must be an integer literal greater than or equal to zero.
- `expr[0]` evaluates `expr` on the current bar.
- `expr[n]` for `n > 0` reads the committed value from `n` bars ago.
- Out-of-range history reads return `na`.
- Series-qualified identifiers keep stable series ids.
- Series-qualified non-identifier expressions that are lowered with history
  receive compiler-generated series ids.
- Constant history is fixture-covered for built-in series, expression history,
  branch bodies, loop bodies, and user-defined function parameters.

## Current Rejections

- Dynamic history offsets such as `close[length]` are rejected with
  `dynamic_history_offset`.
- Negative literal offsets such as `close[-1]` are rejected with
  `negative_history_offset`.
- Dynamic history remains unsupported even when the offset source is an input,
  a simple variable, a loop counter, or a user-defined function parameter.
- Array, object, map, matrix, and drawing-object history snapshots are not
  designed.
- `max_bars_back` inference and declarations are not implemented.

## Why Dynamic Offsets Stay Rejected

Dynamic offsets are more than a parser change. Supporting them safely requires:

- retention bounds for every series that can be dynamically indexed
- runtime handling for negative, `na`, fractional, or very large offsets
- qualifier rules that distinguish const, input, simple, and series integers
- stable behavior inside branches, loops, and user-defined functions
- matching full-history, incremental append, and realtime rollback behavior

Until those rules are implemented together, accepting dynamic offsets would make
scripts appear supported while silently returning unstable or under-retained
history.

## Phase C Implementation Sequence

1. Harden the current static-offset boundary with fixtures.
2. Audit qualifier propagation for const, input, simple, and series values.
3. Audit built-in signatures that currently accept broader qualifiers than the
   compatibility docs claim.
4. Decide whether the first dynamic-offset slice is:
   - still diagnostic-only, or
   - a guarded subset for input/simple integer offsets with explicit retention
     limits.
5. If a guarded subset is chosen, add HIR/runtime support for dynamic offsets,
   runtime diagnostics for invalid offsets, and full/incremental/realtime
   fixtures before updating conformance.

## Acceptance Criteria For Expanding History

- The supported subset is represented in `tests/fixtures/conformance.tsv`.
- Every newly accepted offset form has semantic and runtime fixture coverage.
- Unsupported variants fail during semantic analysis with stable diagnostics.
- Incremental append execution matches full historical execution.
- Realtime rollback keeps history, `var`, callsite state, and outputs
  consistent for confirmed bars and forming-bar updates.

