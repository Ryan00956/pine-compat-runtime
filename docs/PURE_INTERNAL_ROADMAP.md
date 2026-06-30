# Pure Internal Roadmap

Status: planning document.

This document tracks interpreter-internal work only. It is intentionally narrower
than `docs/LONG_TERM_EXECUTION_PLAN.md` and `docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`.
It does not claim new compatibility. A feature becomes supported only after the
matching syntax, semantic analysis, runtime behavior, fixtures, conformance
metadata, snapshots, documentation, and release verification are complete.

## Scope Boundary

In scope:

- parser, AST, semantic analysis, type and qualifier checks;
- HIR/runtime execution semantics;
- series history, persistence, realtime rollback, and deterministic guards;
- pure built-in functions and constants that do not require host services;
- internal collection storage models;
- local and imported user-defined type semantics;
- strategy broker emulation, account math, and script-visible strategy variables;
- conformance metadata, snapshots, runtime profiles, and structural guardrails.

Out of scope for this roadmap:

- chart rendering, visual layout, drag behavior, or host UI;
- external market-data lookup, symbol discovery, or remote request execution;
- webhook, email, push, or other external alert delivery;
- real broker connectivity or live trading adapters;
- hosted dashboards, notebooks, application integrations, or product UI;
- widening public JSON, Python, or WASM host contracts unless a slice explicitly
  designs that contract as part of an interpreter behavior change.

`request.*`, drawing objects, and alert delivery are therefore not roadmap drivers
here. Their existing docs still matter for current behavior, but new work in
those areas should stay outside this pure-internal plan unless the slice is
strictly about analyzer/runtime semantics with no host-service dependency.

## Source Of Truth

Use these files before selecting a slice:

- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/SEMANTIC_MODEL.md`
- `docs/HISTORY_SERIES_AUDIT.md`
- `docs/QUALIFIER_AUDIT.md`
- `docs/ARRAY_STAGE_AUDIT.md`
- `docs/STRATEGY_INTERNAL_GAP_AUDIT.md`
- latest relevant phase or stage audit

Roadmap text is not support evidence. If a roadmap and the current matrix
disagree, trust the matrix, snapshots, fixtures, and latest audit first.

## Execution Rules

- Work one small slice at a time.
- Start every slice by rechecking conformance, matrix output, current docs, and
  the relevant runtime/analyzer modules.
- Keep unsupported variants rejected with stable diagnostics until their behavior
  is deliberately designed.
- Prefer one behavior through the full stack over several parser-only changes.
- Avoid accepting no-op syntax for future compatibility claims.
- Preserve public output shape unless the slice explicitly designs a schema
  change.
- Update `tests/fixtures/conformance.tsv` only after fixture-backed behavior
  exists.
- Close every behavior slice with `git diff --check` and `scripts/verify.sh`.

## Current Baseline

The interpreter already has a broad fixture-backed subset:

- historical and incremental bar execution;
- realtime forming-bar rollback for supported runtime state;
- `if`/`else`, partial `switch`, partial `for`, and partial `while`;
- user-defined functions with local declarations and independent callsite state;
- guarded integer history offsets, including `series int`;
- partial typed arrays and array history snapshots for supported element
  families;
- local scalar-field user-defined types and pure local methods;
- many pure `ta.*`, `math.*`, `str.*`, time, timeframe, session, color, and
  symbol helpers;
- long-only strategy runtime with a fixture-backed Stage 13 multi-entry ledger
  and `pyramiding` subset.

The remaining work is mostly about closing large semantic families, not creating
the first executable runtime.

## Direction 1: Language And Control Flow

Goal: make ordinary Pine control-flow and expression behavior more complete while
preserving deterministic execution and diagnostics.

Current baseline:

- `if`/`else` blocks and scalar `if` expressions are fixture-backed.
- `switch` supports expression arms plus fixture-backed statement-block arms
  whose block ends in a result expression, including selected-arm outer
  reassignment, branch-local no-leak fixtures, and loop-control propagation
  from selected arms inside loop bodies, plus tuple declaration/destructuring
  results, same-local UDT results from selected block arms,
  same-imported-identity UDT results from selected block arms, and message-level
  diagnostics for no-final-expression block arms.
- `for` and `while` loops support statement execution, expression loops where
  currently claimed, local declarations, loop control, stateful callsite
  interaction fixtures, statement-form `for...in` over supported array element
  families including the narrow `array<int>`/`array<float>`/`array<bool>`/
  `array<string>`/`array<color>`/`array<label>`/`array<line>`/
  `array<linefill>`/`array<polyline>`/`array<box>`/`array<table>`/
  `array<chart.point>`/same-local scalar-field UDT array index/value form, and
  `while` statement-body history-read/pure-UDF interaction fixtures, with
  fixture-backed diagnostics for loop control used outside loops.
- Scalar, tuple, same-local UDT, scalar-array, and `matrix<float>` `while`
  expression results with caller-side reads and mutation are fixture-backed
  through parser, semantic analysis, HIR lowering, and runtime execution. The
  collection subsets cover fresh results, existing-alias returns, scalar-array
  history reads returning fresh historical copies, and array/matrix
  zero-iteration `na` results, including array/matrix result preservation across
  `continue` and `break`, plus committed matrix history reads that return fresh
  historical copies. They return the latest reached final body
  expression or `na` when no iteration produces a value, and share
  statement-loop condition, break/continue, scoping, and iteration-guard rules.
  Same-imported-identity UDT results are supported, while nested-array results
  through `while` expressions remain rejected with fixture-backed semantic
  diagnostics.

Remaining internal work:

- broader positive `while` expression nested collection interaction semantics;
- broader `for...in` index/value element families and collection iteration;
- better diagnostics for other unsupported switch forms;
- additional stress fixtures for nested control flow and stateful built-ins.

Non-goals:

- host-driven scheduling;
- visual outputs as a reason to widen language semantics;
- unbounded recursion or execution that can bypass runtime guardrails.

Good next slice:

- one unsupported control-flow form should first get a design note and negative
  fixtures, then one narrow positive fixture-backed subset.

The statement-block `switch` arm design gate is closed in
`docs/PURE_INTERNAL_SWITCH_BLOCK_DESIGN.md`, and its scalar expression-arm block
subsets are implemented. The `while` expression design gate is closed in
`docs/PURE_INTERNAL_WHILE_EXPRESSION_DESIGN.md`. Use those documents before
widening broader `switch` block result variants or `while` expression support.

## Direction 2: Type, Qualifier, And History Semantics

Goal: make the static model closer to Pine without weakening runtime safety.

Current baseline:

- qualifiers use the current `const < input < simple < series` model;
- history offsets accept non-negative integer literals and guarded dynamic integer
  expressions, including `series int`;
- static-only scripts use HIR history metadata to trim committed history;
- dynamic-history scripts keep full committed history up to the runtime cap;
- `indicator(..., max_bars_back=N)` bounds dynamic retention;
- runtime diagnostics and profiles expose dynamic-retention misses and maximum
  missed offsets when dynamic reads exceed the explicit retained bound.

Remaining internal work:

- more complete scalar `simple` inference;
- broader use of existing qualifier-bound helper APIs for "at most input" and
  "at most simple" signature rules;
- per-variable `max_bars_back` declarations or inference;
- broader first-bar, `na`, UDF, loop, array-history, and built-in interaction
  fixtures.

Non-goals:

- silently accepting non-integer or negative history offsets;
- unbounded history retention;
- changing built-in qualifier acceptance without synchronized docs and fixtures.

Good next slice:

- add a narrow qualifier helper or diagnostic improvement that reduces bespoke
  built-in signature handling without changing broad compatibility claims.

## Direction 3: Collections

Goal: move from a large partial array subset toward deliberate collection
semantics.

Current baseline:

- runtime-owned array ids;
- reference assignment and explicit `array.copy` independence;
- supported scalar and existing object-id array element families as recorded in
  conformance;
- many creation, mutation, search, ordering, numeric, slice, concat, and method
  call helpers;
- array history snapshots for the fixture-backed element families.

Remaining internal work:

- map storage model and key/value type rules;
- matrix storage model and two-dimensional indexing rules;
- UDT array behavior beyond the same-local scalar-field subset;
- generic or bare `array` declarations beyond current fixture-backed element
  kinds;
- `for...in` iteration over arrays and future collections;
- richer aliasing, nested collection, history, and rollback rules;
- `varip` support for non-scalar collection families only after realtime handoff
  is designed.

Non-goals:

- treating `array.*` as broadly complete because many helpers exist;
- adding map syntax before storage lifetime and mutation rules are written down;
- widening matrix syntax beyond the fixture-backed `matrix<float>`
  `new/get/set/copy/rows/columns` subset before history, rollback, and typed
  declaration semantics are written down;
- host-visible collection output as part of the first internal collection slice.

Good next slice:

- add one missing array helper only for an already-supported scalar element
  family, add one missing negative fixture for a closed design gate, or start
  the semantic-only shared array element-kind refactor. The map design gate is
  closed in
  `docs/PURE_INTERNAL_MAP_DESIGN.md`; the matrix design gate is closed in
  `docs/PURE_INTERNAL_MATRIX_DESIGN.md`; the UDT array design gate is closed in
  `docs/PURE_INTERNAL_UDT_ARRAY_DESIGN.md`; the generic/bare array declaration
  design gate is closed in `docs/PURE_INTERNAL_ARRAY_DECLARATION_DESIGN.md`; the
  `for...in` design gate is closed in `docs/PURE_INTERNAL_FOR_IN_DESIGN.md`.
  Use those documents before any positive `map.*`, any broader `matrix.*`, UDT
  array, declaration-widening, or `for...in` support.

## Direction 4: User-Defined Types, Methods, And Imports

Goal: expand structured data while preserving type identity, method dispatch, and
side-effect boundaries.

Current baseline:

- local scalar-field UDT construction, reads, ordinary variables, and `var`
  persistence;
- local typed UDT declarations from fixture-backed same-UDT expressions;
- pure local UDT methods with receiver, local UDT parameter passthrough, nested
  method passthrough, constructor helpers, and selected control-flow returns;
- exact-key source graph import subset for exported const expressions, pure
  exported functions, scalar-field imported UDT constructors with direct field
  reads, ordinary same-imported-UDT reassignment, and scalar-field imported UDT
  typed declarations initialized or reassigned from the same imported identity,
  imported UDT ternary, `if`, `switch`, `while`, and `for` expression results
  from the same imported identity, plus imported UDT UDF direct or nested parameter
  passthrough, direct or nested constructor-return results, and ordinary
  imported UDT `var` declarations, scalar-field same-imported-identity
  `varip` declarations, and scalar-field mutation in top-level, branch,
  `for`-loop, `while`-loop, and UDF-local statement contexts.

Remaining internal work:

- broader imported UDT identity flow across source graphs, including history
  and collections;
- imported methods;
- UDT arrays beyond the same-local scalar-field subset and UDT history
  references;
- broader `varip` UDT values beyond the typed same-local scalar-field subset;
- side effects inside methods or UDFs, if ever accepted;
- clearer diagnostics for unsupported imported UDT, imported method, and method
  side-effect boundaries.

Non-goals:

- cross-library UDT identity without a source-graph design;
- method side effects as a small syntax patch;
- recursive types or recursive functions without an explicit termination model.

Good next slice:

- one diagnostics fixture/message improvement for an unsupported UDT or method
  boundary, or a design gate for imported UDT identity.

The imported UDT identity design gate is closed in
`docs/PURE_INTERNAL_IMPORTED_UDT_DESIGN.md`. Use it before any positive imported
UDT constructor, value, assignment, or method support.

The UDT `varip` design gate is closed in
`docs/PURE_INTERNAL_UDT_VARIP_DESIGN.md`. The typed and direct-constructor
same-local scalar-field subset is fixture-backed; use the gate before broadening
UDT `varip` value support.

## Direction 5: Pure Built-In Coverage

Goal: improve ordinary script compatibility through small pure built-in slices.

Current baseline:

- broad fixture-backed coverage across common `ta.*`, `math.*`, `str.*`, time,
  timeframe, session, syminfo, color, and cast helpers;
- many edge-case fixtures for numeric rolling windows, `na`, tuple returns, and
  supported qualifier families.

Remaining internal work:

- missing high-use pure `ta.*` helpers;
- more `math.*` and `str.*` edge cases;
- more time/session/timezone helper semantics that do not require exchange data;
- tighter diagnostics for unsupported argument families;
- qualifier alignment between `docs/BUILTIN_SIGNATURES.md` and code acceptors.

Non-goals:

- built-ins that require remote data, account state, chart UI, or services;
- approximate behavior without fixture evidence;
- broad claims such as "all `ta.*`" or "all string formatting".

Good next slice:

- choose one high-use pure built-in gap from real fixtures, implement the smallest
  documented subset, and update only the corresponding conformance row.

## Direction 6: Strategy Broker And Account Semantics

Goal: continue strategy compatibility only where the internal broker model can
prove deterministic state transitions.

Current baseline:

- long-only broker with Stage 13 fixture-backed multi-entry ledger and positive
  integer `pyramiding` subset;
- supported long market, limit, stop, and stop-limit entries;
- supported `strategy.close`, `strategy.close_all`, `strategy.cancel`, and
  `strategy.cancel_all` subsets;
- broad supported `strategy.exit` subset across single triggers, brackets,
  trailing exits, partial quantities, reservations, omitted-quantity replacement,
  and long-only multi-entry allocation;
- script-visible strategy variables and trade namespace subsets;
- supported cash-per-contract, cash-per-order, and percent commission modes,
  fixed-tick slippage, fixed-tick limit verification, cash default sizing,
  percent-of-equity default sizing, explicit `close_entries_rule="FIFO"`,
  fixture-backed id-specific long-only `close_entries_rule="ANY"`, and selected
  long-margin behavior.

Remaining internal work:

- short exposure;
- automatic long/short reversal;
- `strategy.order()` behavior beyond the fixture-backed long
  market/limit/stop/stop-limit add-or-increase subset and explicit-quantity
  reduce-only market-short subset;
- broader `close_entries_rule="ANY"` behavior beyond fixture-backed
  id-specific long-only close/exit allocation;
- custom OCA behavior across order families;
- `process_orders_on_close`, `calc_on_order_fills`, `calc_on_every_tick`, and bar
  magnifier style timing;
- `margin_short`, richer account constraints, currency conversion, broader
  short-side, rounded, and currency-aware `strategy.margin_liquidation_price`
  behavior;
- remaining strategy information variables and trade namespace fields;
- `strategy.risk.*` rules after broker/account foundations are stronger.

Non-goals:

- reopening broad broker foundations immediately after Stage 13;
- accepting new `strategy()` properties as inert no-ops;
- public pending-order, reservation, or open-trade ledgers before a schema design;
- real broker connectivity.

Good next slice:

- a narrow no-op, diagnostic, accounting, or script-visible field slice that keeps
  the current public strategy result shape unchanged. Short/reversal and generic
  order work should start with a design gate, not an implementation patch.

The strategy short/reversal design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_SHORT_REVERSAL_DESIGN.md`. Use it before any
positive `strategy.short` entry, short exposure, or automatic reversal support.

The generic strategy order design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_ORDER_DESIGN.md`. Use it before any positive
`strategy.order()` support beyond the current fixture-backed subset, generic
order netting, or generic-order OCA work.

The strategy close-entries-rule reference is in
`docs/PURE_INTERNAL_STRATEGY_CLOSE_ENTRIES_RULE_DESIGN.md`. Use it before any
non-default close allocation behavior.

The strategy OCA design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_OCA_DESIGN.md`. Use it before any positive
`oca_name`, `strategy.oca.*`, or cross-order-family OCA behavior.

The strategy execution-timing design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_EXECUTION_TIMING_DESIGN.md`. Use it before any
positive `process_orders_on_close`, `calc_on_order_fills`,
`calc_on_every_tick`, bar magnifier, or standard-OHLC fill timing support.

The strategy margin-short/account design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_MARGIN_SHORT_ACCOUNT_DESIGN.md`. Use it before any
positive `margin_short` runtime behavior, broader/short/rounded/currency-aware
`strategy.margin_liquidation_price`, symbol precision rounding, or
currency-conversion account behavior.

The strategy risk-rule design gate is closed in
`docs/PURE_INTERNAL_STRATEGY_RISK_DESIGN.md`. Use it before any positive
`strategy.risk.*` support, including entry-direction, drawdown/loss, position
size, or filled-order-count risk rules.

## Direction 7: Runtime Guardrails And Verification

Goal: keep the runtime maintainable as compatibility widens.

Current baseline:

- conformance matrix guards;
- golden runtime snapshots;
- strict public `schemaVersion` checks;
- structure guardrail;
- runtime profiles for history and callsite state;
- full release gate in `scripts/verify.sh`.

Remaining internal work:

- more profile fields when new storage families land;
- focused stress fixtures for loops, collection growth, UDF call depth, and
  history retention;
- clearer runtime errors for storage and guardrail limits;
- periodic audits to keep roadmap, conformance, snapshots, and docs aligned.

Non-goals:

- weakening guardrails to accept broader scripts;
- using roadmap text as a substitute for fixture-backed behavior;
- updating snapshots without rerunning the non-update verification path.

Good next slice:

- add a guardrail or profile assertion only when it protects an existing or
  immediately upcoming runtime behavior slice.

## Recommended Order

1. Small pure built-in or diagnostic slices from real fixture gaps.
2. Type/qualifier/history hardening that unlocks multiple later built-ins.
3. Collection design gates for map, matrix, UDT arrays, generic declarations,
   and iteration before runtime support.
4. UDT/import identity design before imported UDTs or imported methods.
5. Conservative strategy maintenance slices that preserve public output shape.
6. Large strategy broker work only after a fresh design gate for short/reversal,
   generic order, OCA, or account-model behavior.
7. Runtime guardrail work whenever a new semantic family would otherwise grow
   state or execution cost without visibility.

Avoid opening request, drawing, alert delivery, or host-integration work from this
roadmap. Those belong in the broader platform plans unless the change is purely a
semantic analyzer or runtime-core boundary fixture.

## Completion Gate

Before any pure-internal slice is closed:

```text
git diff --check
scripts/verify.sh
```

The closeout note or audit must state:

- what changed;
- what remains unsupported;
- which fixtures prove the new boundary;
- whether public output shape changed;
- which docs and conformance rows were updated.
