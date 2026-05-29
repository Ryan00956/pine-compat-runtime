# Phase Q Audit: Strategy Exit Bracket Design Gate

Status: in progress.

Phase Q is a design-gate and diagnostic-hardening phase for future
`strategy.exit` bracket support. It must not widen executable strategy
compatibility, conformance status, runtime output schemas, Python dictionaries,
WASM JSON, or runtime snapshots unless a later slice is explicitly changed into
a fixture-backed behavior phase.

## Completed Slices

- Slice 0 locked the Phase P/O strategy baseline, confirmed Phase Q as a design
  gate, synchronized the long-term roadmap with the planned Phase Q target, and
  recorded the current unsupported combined-trigger boundary before any
  diagnostic or behavior changes.
- Slice 1 hardened user-visible `strategy.exit` diagnostics so they describe
  the current strategy subset instead of old phase names, added a
  diagnostic-only four-trigger combined-exit fixture, and refreshed conformance
  metadata plus the matrix metadata snapshot without changing runtime behavior.

## Slice 0 Baseline

Phase P is closed for structural broker maintenance. It split strategy broker
internals without changing the Pine compatibility surface, public runtime
schema, host output shapes, or existing strategy behavior. The current broker
layout remains:

```text
crates/pine-runtime/src/strategy/
   mod.rs
   broker/
      mod.rs
      exits.rs
      fills.rs
      accounting.rs
      tests.rs
```

Phase O is closed for the current fixture-backed strategy reporting count
subset. `strategy.closedtrades` and `strategy.opentrades` remain script-state
count variables only; no public open-trade records, pending-order records,
partial-fill fields, exit-reason fields, or schema bump were added.

The current conformance boundary remains conservative:

- `strategy`, `strategy.entry`, `strategy.close`, strategy equity, strategy
  state variables, `strategy.closedtrades`, `strategy.opentrades`, and
  `strategy.exit` are `partial`.
- Broad `strategy.*` remains `unsupported`.
- The supported `strategy.exit` subset remains stop-only, limit-only,
  profit-only, or loss-only full-position exits for the current one-net-long
  broker.
- Combined trigger, trailing, partial quantity, missing-entry, multiple pending
  exit, short exposure, pyramiding, richer order, commission, slippage, margin,
  strategy alert, and realtime strategy forms remain unsupported.

Existing combined-trigger semantic fixtures cover:

- `tests/fixtures/sema/unsupported_strategy_exit_stop_limit.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_profit_loss.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_stop_profit.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_limit_loss.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_stop_loss.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_limit_profit.pine`
- `tests/fixtures/sema/unsupported_strategy_exit_three_triggers.pine`

The analyzer rejects combined trigger families before runtime behavior is
reachable. Runtime extraction still selects the first supported single trigger
family in the order stop, limit, profit, loss, but that fallback is protected by
semantic rejection for combined trigger calls. The broker still stores a single
`pending_exit` with one `PendingExitTrigger`, and `evaluate_pending_exits`
preserves creation-bar ineligibility with
`last_update_bar_index >= bar_index`.

Phase Q therefore remains appropriate as a design gate rather than a bracket
implementation phase. The next slice should harden user-visible
`strategy.exit` diagnostics and add a diagnostic-only four-trigger fixture
without changing runtime behavior or public host contracts.

## Slice 1 Diagnostic Boundary

Slice 1 kept diagnostic codes unchanged while replacing stale Phase N/Slice 1
wording in `validate_strategy_exit_args` with phase-neutral current-subset
messages:

- positional `profit`/`loss` rejection:
  `` `strategy.exit` profit and loss arguments must be named arguments ``
- unsupported option rejection:
  `` `strategy.exit` argument `{name}` is not supported in the current strategy subset ``
- combined trigger rejection:
  `` `strategy.exit` combined trigger families are not supported in the current strategy subset ``

The diagnostic-only fixture
`tests/fixtures/sema/unsupported_strategy_exit_four_triggers.pine` now covers
the maximal `stop + limit + profit + loss` trigger family directly. It is
referenced from the existing `strategy.exit` partial row and broad
`strategy.*` unsupported row in `tests/fixtures/conformance.tsv`; neither row
changed status. `tests/snapshots/matrix.json` was refreshed only for the
metadata fixture-list change.

## Verification

Slice 0 verification:

```text
cargo test -p pine-sema strategy
cargo test -p pine-runtime strategy::broker
git diff --check
```

All Slice 0 verification commands passed on the Slice 0 workspace.

Slice 1 verification:

```text
cargo fmt --check
cargo test -p pine-sema strategy
cargo test -p pine-cli conformance_metadata_references_existing_fixtures
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
cargo test -p pine-cli matrix_output_matches_golden_snapshot
git diff --check
```

All Slice 1 verification commands passed on the Slice 1 workspace.
