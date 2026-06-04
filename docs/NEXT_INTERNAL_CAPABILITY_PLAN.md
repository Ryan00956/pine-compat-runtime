# Next Internal Capability Plan

Status: planning document.

This document groups the next interpreter-internal work into seven large task
directions. It does not claim new compatibility. A task becomes supported only
after the matching syntax, semantic analysis, runtime behavior, fixtures,
conformance metadata, snapshots, docs, and release verification are complete.

## Selection Rules

- Pick one small slice from one direction at a time.
- Prefer work that can be proven with local fixtures and host-neutral JSON.
- Keep public output fields unchanged unless the slice explicitly designs a new
  contract.
- Keep unsupported variants rejected with diagnostics.
- Do not use roadmap wording as support evidence. Use
  `tests/fixtures/conformance.tsv`, snapshots, audit docs, and verification
  results.

## Direction 1: Strategy Maintenance

Goal: make the existing basic long-only strategy subset more useful without
turning it into a full broker simulator.

Good next slices:

- Stage 8 broker-expansion design gate: document the internal ledger,
  net-position accounting, same-bar precedence, and public-output boundary
  before any short, reversal, pyramiding, generic order, or OCA work begins.
- Clearer no-position and wrong-entry diagnostics for supported exit shapes.
- More fixture-backed strategy state variables or count helpers.
- Narrow order/trade accounting improvements that keep the current public output
  shape.

Keep out of scope until separately designed:

- Short exposure, reversals, pyramiding, and multiple simultaneous entries.
- `strategy.order`, custom OCA behavior, unsupported margin/account behavior,
  and rich order types.
- Arbitrary future binding for unmatched `from_entry` ids.
- Public pending-order, reservation, remaining-quantity, or exit-reason records.
- Realtime strategy handoff and intrabar path reconstruction.

Recommended first slice: a Stage 8 broker-expansion design gate. The
active-entry absolute attachment evidence slice is already closed, as are the
current long-only Stage 7 reporting, cost, default-sizing, and active-margin
account subsets. The next strategy runtime expansion crosses into broker-model
work and should not widen conformance before the design boundary is explicit.

## Direction 2: Built-In Coverage

Goal: increase the number of ordinary indicator scripts that run by adding small
fixture-backed built-in subsets.

Good next slices:

- Additional common `ta.*` helpers.
- Additional `math.*` helpers and edge-case coverage for existing helpers.
- Additional `str.*` formatting and parsing helpers.
- More time, timeframe, session, symbol, and color helper subsets.
- Better diagnostics for known unsupported built-in argument forms.

Keep out of scope until separately designed:

- Built-ins that require host market data, account state, chart UI state, or
  remote services.
- Approximate behavior without fixture evidence.
- Broad claims such as "all `ta.*`" or "all string formatting".

Recommended first slice: choose one high-use built-in family from real fixture
gaps, add only the smallest documented subset, and update the matrix only for
that fixture-backed subset.

## Direction 3: Arrays And Collections

Goal: make the current typed-array support more complete while preserving clear
storage lifetime and mutation rules.

Good next slices:

- More array functions or method-call aliases for existing scalar element types.
- More array behavior inside branches, loops, and user-defined functions.
- More array/history/state interaction fixtures.
- Better diagnostics for unsupported element types and invalid mutations.

Keep out of scope until separately designed:

- Arrays of user-defined types.
- Drawing-object arrays and point arrays.
- Map or matrix families.
- Behavior that requires object identity or lifetime rules not already modeled.

Recommended first slice: add one missing array helper for already-supported
scalar arrays, with branch/loop/UDF interaction fixtures if the helper mutates
state.

## Direction 4: User-Defined Types And Methods

Goal: expand the local structured-data subset without weakening type identity or
method resolution.

Good next slices:

- Field mutation for local scalar-field UDT values.
- More UDT parameter and return-value fixtures.
- More pure method forms on local UDT receivers.
- Clearer diagnostics for unsupported imported UDTs, imported methods, and
  side-effecting methods.

Keep out of scope until separately designed:

- Imported UDT identity across source graphs.
- Imported methods.
- UDT arrays.
- UDT history references and `varip` UDT values.
- Side effects inside methods.

Recommended first slice: local scalar-field mutation, if the semantic and runtime
storage model can be made explicit and covered by fixtures.

## Direction 5: Request Support

Goal: widen request behavior only where the runtime can stay deterministic and
host-provided data can prove the result.

Good next slices:

- More same-symbol or exact-key higher-timeframe `request.security` cases.
- More supported scalar expression shapes inside requested contexts.
- More alignment and gap-handling fixtures.
- Better host parity coverage for CLI, Python, and WASM request data injection.

Keep out of scope until separately designed:

- Lower-timeframe array-returning requests.
- Broad `request.*` families.
- Remote data lookup inside core crates.
- Requested-context strategy state or side effects.

Recommended first slice: one additional deterministic higher-timeframe
alignment case backed by request fixtures and host parity tests.

## Direction 6: Drawing Objects

Goal: add useful drawing lifecycle behavior while keeping the output
host-neutral.

Good next slices:

- More `label.*`, `line.*`, `box.*`, and `table.*` methods.
- More deletion, mutation, no-op, and runtime-limit fixtures.
- More realtime rollback fixtures for already-supported drawing families.
- Better diagnostics for unsupported coordinate modes and invalid ids.

Keep out of scope until separately designed:

- `polyline.*` until chart-point and point-array semantics are designed.
- Host-specific visual layout, drag behavior, or chart interaction.
- Drawing behavior that cannot be represented in the current JSON contract.

Recommended first slice: one missing method from an already-supported drawing
family, with runtime snapshot and realtime rollback coverage if it changes
state.

## Direction 7: Alerts

Goal: make alert events more useful while keeping alert behavior deterministic
and local to runtime output.

Good next slices:

- Narrow alert frequency semantics that can be expressed without host services.
- Additional alert argument validation and diagnostics.
- More branch, loop, and realtime rollback fixtures.
- Better parity tests for alert output through CLI, Python, and WASM.

Keep out of scope until separately designed:

- Sending webhooks, email, push notifications, or other external delivery.
- TradingView-style placeholder interpolation.
- Host scheduling, throttling, or user notification policy.

Recommended first slice: a local alert frequency subset only if the policy can
be deterministic from bar-by-bar execution and tested without external services.

## Recommended Order

1. Strategy maintenance: Stage 8 broker-expansion design gate.
2. Built-in coverage selected from real fixture gaps.
3. Arrays and collections for already-supported scalar element types.
4. User-defined type and method maintenance.
5. Request support for one deterministic host-data case.
6. Drawing object method maintenance.
7. Alert policy maintenance.

This order keeps the runtime useful for ordinary indicator execution and the
current basic strategy subset while postponing work that needs new public
contracts, chart rendering, remote data, or external delivery systems.
