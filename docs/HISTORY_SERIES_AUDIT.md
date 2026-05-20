# History and Series Audit

This document starts Phase C from `docs/LONG_TERM_EXECUTION_PLAN.md`. It records
the current implementation boundary before changing dynamic history or qualifier
rules.

## Current Supported Subset

- History references use `expr[offset]` syntax.
- The offset must be an integer literal greater than or equal to zero, or a
  `const int`, `input int`, or `simple int` expression.
- `expr[0]` evaluates `expr` on the current bar.
- `expr[n]` for `n > 0` reads the committed value from `n` bars ago.
- dynamic offsets are accepted when the offset expression is `const int`,
  `input int`, or `simple int`.
- Out-of-range history reads return `na`.
- Dynamic offsets that evaluate to `na` return `na`.
- Dynamic offsets that evaluate to a negative integer fail at runtime.
- Series-qualified identifiers keep stable series ids.
- Series-qualified non-identifier expressions that are lowered with history
  receive compiler-generated series ids.
- Constant history is fixture-covered for built-in series, expression history,
  branch bodies, loop bodies, and user-defined function parameters.
- Dynamic const/input/simple history is fixture-covered for built-in series and
  expression history.

## Current Rejections

- Series-qualified history offsets such as `close[bar_index]` are rejected with
  `dynamic_history_offset`.
- Negative literal offsets such as `close[-1]` are rejected with
  `negative_history_offset`.
- Dynamic history remains unsupported when the offset source is a loop counter,
  a series value, or a user-defined function parameter bound to a series value.
- Array, object, map, matrix, and drawing-object history snapshots are not
  designed.
- `max_bars_back` inference and declarations are not implemented.

## Why Series Offsets Stay Rejected

Series offsets are more than a parser change. Supporting them safely requires:

- retention bounds for every series that can be dynamically indexed
- runtime handling for negative, `na`, fractional, or very large offsets
- qualifier rules that distinguish const, input, simple, and series integers
- stable behavior inside branches, loops, and user-defined functions
- matching full-history, incremental append, and realtime rollback behavior

Until those rules are implemented together, accepting series offsets would make
scripts appear supported while silently returning unstable or under-retained
history. The current dynamic subset is limited to `const int`, `input int`,
and `simple int` offset expressions with growable retention. The runtime keeps
all committed values needed by the current execution model and fails once total
committed series values exceed the configured runtime cap.

## Phase C Implementation Sequence

1. Harden the current static-offset boundary with fixtures.
2. Audit qualifier propagation for const, input, simple, and series values.
   Current findings are in `docs/QUALIFIER_AUDIT.md`.
3. Audit built-in signatures that currently accept broader qualifiers than the
   compatibility docs claim.
4. Add static-depth inference for constant offsets so future runtimes can
   retain less than full-history when dynamic offsets are absent.
5. Design whether any `series int` offset subset can be supported with
   max-bars-back style retention, runtime diagnostics, and memory limits.

## Acceptance Criteria For Expanding History

- The supported subset is represented in `tests/fixtures/conformance.tsv`.
- Every accepted offset form has semantic and runtime fixture coverage.
- Unsupported variants fail during semantic analysis with stable diagnostics.
- Incremental append execution matches full historical execution.
- Realtime rollback keeps history, `var`, callsite state, and outputs
  consistent for confirmed bars and forming-bar updates.
