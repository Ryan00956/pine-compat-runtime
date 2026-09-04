# Strategy Broker Next Execution Plan

Status: Stages 17, 18, and 19-22 closed. Stage 18g true OHLC path execution
closed on 2026-09-04. Stage 23 Bar Magnifier fill wiring is planned but not
started. Its
[detailed execution plan](STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_EXECUTION_PLAN.md)
is the active step-by-step procedure.
Created on 2026-09-02 after Strategy Internal Stages 14-16 closed the
fixture-backed short, reversal, short-margin, and id-specific
`close_entries_rule="ANY"` subsets.

This plan defines the next strategy-runtime program while Pine v1-v4 strategy
compatibility and further source-version expansion are paused. It turns the
current collection of supported strategy paths into a unified broker model
before adding more order families. The work is split into Stages 17-22 so each
positive behavior can land as a small, independently verifiable slice.

This document is a plan, not a support claim. Executable support remains defined
by `tests/fixtures/conformance.tsv`, committed fixtures and snapshots, host
parity, audit records, and a passing `scripts/verify.sh` run.

## Primary References

Official behavior references:

- TradingView Pine Script strategy concepts:
  https://www.tradingview.com/pine-script-docs/concepts/strategies/
- TradingView Pine Script declaration statements:
  https://www.tradingview.com/pine-script-docs/language/declaration-statements/
- TradingView Pine Script execution model:
  https://www.tradingview.com/pine-script-docs/language/execution-model/
- TradingView Pine Script v6 reference:
  https://www.tradingview.com/pine-script-reference/v6/

Repository design references:

- `docs/STRATEGY_INTERNAL_GAP_AUDIT.md`
- `docs/PURE_INTERNAL_STRATEGY_ORDER_DESIGN.md`
- `docs/PURE_INTERNAL_STRATEGY_OCA_DESIGN.md`
- `docs/PURE_INTERNAL_STRATEGY_EXECUTION_TIMING_DESIGN.md`
- `docs/PURE_INTERNAL_STRATEGY_RISK_DESIGN.md`
- `docs/PURE_INTERNAL_STRATEGY_CLOSE_ENTRIES_RULE_DESIGN.md`
- `docs/STRATEGY_INTERNAL_MARGIN_ACCOUNT_MODEL_PLAN.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/CONFORMANCE.md`

Before starting a behavior-changing slice, recheck the relevant official page
and record the review date in that slice's audit. Do not copy proprietary source
code, private APIs, UI behavior, or error text. Fixtures must remain original or
carry the repository-required provenance metadata.

## Starting Point

The current fixture-backed strategy runtime already includes:

- long and short market, limit, stop, and stop-limit `strategy.entry()` subsets;
- market `strategy.entry()` reversal;
- selected same-side `strategy.order()` additions and reduce-only market-short
  behavior;
- `strategy.close()`, `strategy.close_all()`, cancellation, broad
  `strategy.exit()` triggers, brackets, trailing exits, partial quantities, and
  internal reservation behavior;
- a side-aware multi-entry `TradeLedger`, long and short trade fields, supported
  commission/slippage/limit-verification settings, long and short margin,
  affordability checks, forced liquidation, and liquidation-price reporting;
- id-specific long and short `close_entries_rule="ANY"` allocation;
- public CLI, Python, and WASM parity for the current strategy result shape.

The current internal model also has constraints that make direct feature
expansion risky:

- `OrderBook` owns separate pending-entry and pending-exit books, but does not
  represent a generic pending order family explicitly;
- `PendingEntry.enforce_pyramiding` currently carries part of the distinction
  between `strategy.entry()` and `strategy.order()` instead of storing command
  origin and fill policy explicitly;
- market, limit, stop, and stop-limit fills are dispatched through separate
  direction-specific runtime calls instead of one ordered broker scheduler;
- `BrokerState` keeps both ledger state and singleton aggregate mirrors, so
  every new transition must keep both representations synchronized;
- `strategy.close()` and `strategy.close_all()` execute through immediate
  current-bar close paths rather than the same market-order lifecycle as other
  broker commands;
- generic-order cross-zero netting, custom OCA behavior, recalculation after
  fills, realtime strategy tick scheduling, and `strategy.risk.*` remain
  unsupported.

The current strategy reporting surface is no longer the main blocker. Common
aggregate state variables and the documented open/closed-trade field families
already have fixture-backed subsets. New reporting helpers should be driven by
real strategy-corpus failures, not selected merely because they are small.

## Program Goal

Build a deterministic internal strategy broker with one order lifecycle and one
fill transition path, then use that foundation to implement the most
dependency-heavy strategy behaviors in this order:

1. Stage 17: unified order/fill kernel without public behavior changes;
2. Stage 18: correct historical market-order timing and close scheduling;
3. Stage 19: complete generic-order netting and price-based entry reversal;
4. Stage 20: OCA groups and unified cancellation/replacement behavior;
5. Stage 21: bounded recalculation and realtime/intrabar scheduling;
6. Stage 22: broker-enforced strategy risk rules.

The program is complete when each stage has its own closeout audit, all accepted
forms are represented in the conformance matrix, unsupported variants still
fail closed, and the canonical repository verification gate passes without
snapshot-update mode.

## How To Execute This Plan

Always execute the first slice whose status is not closed. For that slice:

1. Create a working branch or commit boundary dedicated to that slice; do not
   mix unrelated interpreter work into it.
2. Re-read the named repository design gate and current official reference.
3. Run the slice's baseline commands before editing and save the exact output
   in a new audit draft.
4. Add or confirm the negative/boundary test described by the slice.
5. Implement the internal model or positive behavior in the stated order.
6. Run owner-local tests after each coherent change; do not refresh snapshots
   while a direct test is failing.
7. Complete the per-slice file checklist, inspect every intentional snapshot
   change, and run `git diff --check` plus `scripts/verify.sh`.
8. Create an audit named
   `docs/STRATEGY_INTERNAL_STAGE<stage>_<SLICE_NAME>_AUDIT.md`, update the slice
   and stage status, and record remaining exclusions before starting the next
   slice.

Do not work ahead on a blocked stage merely because its semantic surface is
small. A later stage may begin early only where that stage explicitly permits
storage-only or boundary-test work and no runtime behavior is widened.

## Program Non-Goals

- No Pine v1-v4 strategy compatibility or new legacy strategy translation.
- No broad source-version project while this plan is active.
- No live broker connectivity, remote market-data fetching, chart UI, or
  Strategy Tester UI reproduction.
- No external alert delivery, webhook retry system, or authentication secret
  store.
- No public pending-order, reservation, OCA, or risk-state schema until a later
  schema plan proves that hosts need those fields.
- No currency conversion or symbol-specific contract precision inside the
  unified-order work unless a later account slice explicitly designs it.
- No inert acceptance of unsupported `strategy()` properties or order
  arguments.
- No snapshot refresh used to conceal an unexplained behavior change.

## Global Execution Rules

Apply these rules to every slice in Stages 17-22:

1. One slice owns one semantic change. Internal-only scaffolding and positive
   user-visible behavior must not be mixed unless the scaffolding cannot be
   tested independently.
2. Lock the old boundary before changing it. Add a test that fails for the
   intended reason under the current implementation.
3. Keep the public `StrategyResult` shape unchanged unless a separate schema
   design is approved before implementation.
4. Put behavior in the owning core crate. CLI, Python, and WASM remain thin
   projections of the same runtime result.
5. Route every fill through shared accounting, ledger allocation, commission,
   slippage, margin, alert-metadata, position, equity, and diagnostic helpers.
6. Unsupported forms must remain rejected by stable semantic diagnostics or
   explicitly documented runtime no-op rules. Do not accept a parameter merely
   because its value can be parsed.
7. Historical, incremental, and realtime behavior must either agree or have a
   documented rejection/boundary fixture.
8. Update conformance prose only for behavior proven in the same slice.
9. Create one closeout audit per positive slice. The audit records exact files,
   fixtures, snapshot names, commands, results, and remaining exclusions.
10. Run `scripts/verify.sh` before marking a slice closed.

## Required Internal Invariants

All stages must preserve these invariants:

- `TradeLedger` is the authoritative source for open-trade quantity, direction,
  entry identity, entry cost allocation, and aggregate net position.
- Aggregate `position_size` and `avg_price` values are derived from the ledger or
  checked against it after every state transition.
- A fill is atomic: validation failure leaves cash, orders, trades, ledger,
  reservations, alerts, position snapshots, and equity unchanged except for an
  explicitly documented diagnostic or rejected-order cleanup.
- Every pending order has stable creation order so same-tick eligibility can be
  resolved deterministically.
- Every fill records command origin separately from trade direction.
- Quantity used to close existing exposure and quantity used to open new
  exposure are distinguishable during a cross-zero transition.
- Entry commission is allocated proportionally on partial closes and exactly
  once over the lifetime of an open trade.
- Exit commission, slippage, limit verification, margin checks, and alert
  metadata are applied exactly once per actual fill.
- Cancellation clears every internal attachment owned by the cancelled order
  and does not clear unrelated reservations or deferred exits.
- Realtime rollback never leaves broker events, alerts, pending orders, or
  strategy series from an abandoned forming-bar execution.

## Target Internal Shape

Names are illustrative. A slice may choose different Rust names, but it must
preserve the responsibilities below.

```text
OrderIntent
  key
  id
  source: Entry | Order | Exit | Close | CloseAll | MarginCall
  direction: Long | Short
  kind: Market | Limit | Stop | StopLimit | ExitTrigger
  quantity
  quantity_policy
  target_entry
  created_bar_index
  creation_sequence
  pyramiding_policy
  reduction_policy
  oca_group
  metadata

FillRequest
  order_key
  bar_index
  time
  raw_price
  trigger_reason

FillTransition
  closed_allocations
  opened_trade
  filled_quantity
  close_quantity
  open_quantity
  fill_price
  cash_delta
  realized_profit
  commission

BrokerScheduler
  collect_eligible_orders(...)
  order_fill_candidates(...)
  execute_fill(...)
  request_recalculation(...)
  commit_or_rollback(...)
```

`OrderIntent` must not be added to public JSON in this program. It is an
internal model that makes later behavior testable without committing hosts to a
pending-order schema.

## Stage 17: Unified Order And Fill Kernel

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_UNIFIED_FILL_AUDIT.md`.

### Stage 17 Goal

Create explicit command-origin, order-intent, and fill-transition abstractions;
make `TradeLedger` the authoritative position source; and route the already
supported strategy subset through shared paths without changing any public
runtime output.

Stage 17 is a refactor stage. It must not widen semantic acceptance or update
conformance support claims.

### Stage 17 Slice Order

#### 17a. Baseline And Documentation Truth Lock

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_BASELINE_AUDIT.md`.

Goal:

- capture the exact pre-refactor strategy behavior and remove planning
  ambiguity before moving code.

Steps:

1. Run the current focused strategy gate and record counts/results in the Stage
   17a audit:

   ```text
   cargo test -p pine-runtime strategy -- --test-threads=1
   cargo test -p pine-sema strategy
   cargo test -p pine-cli runtime_outputs_match_golden_snapshots
   python3 scripts/check_host_parity.py
   ```

2. Record the current strategy runtime fixture, snapshot, and conformance-row
   counts:

   ```text
   rg --files tests/fixtures/runtime | rg '/strategy_.*\.pine$' | wc -l
   rg --files tests/snapshots | rg '/runtime_strategy_.*\.json$' | wc -l
   rg -c '^strategy' tests/fixtures/conformance.tsv
   ```

   The 2026-09-02 planning baseline is 259 runtime fixtures, 244 runtime
   snapshots, and 84 strategy-prefixed conformance rows. Treat later differences
   as drift to explain in the audit, not as an instruction to force these
   historical counts.
3. Add characterization tests for these currently distinct paths if a matching
   test does not already exist:
   - same-side market entry;
   - market entry reversal;
   - same-side market generic order;
   - reduce-only market generic order;
   - price-based entry and price-based generic order;
   - full close, partial close, exit fill, and margin-call fill.
4. Update stale strategy status prose that still describes the current runtime
   as long-only. Do not change executable claims.
5. Create `docs/STRATEGY_INTERNAL_STAGE17_BASELINE_AUDIT.md` with the exact
   starting contract and known documentation contradictions.

Acceptance:

- no semantic, HIR, runtime, snapshot, or public schema behavior changes;
- the characterization suite covers every fill-origin family that Stage 17 will
  route;
- all existing runtime snapshots remain byte-for-byte unchanged;
- the worktree contains only intentional documentation and test additions.

Stop condition:

- stop if an existing fixture contradicts the current conformance row; resolve
  the source-of-truth mismatch before beginning 17b.

#### 17b. Explicit Order Origin And Stable Internal Keys

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_ORIGIN_KEYS_AUDIT.md`.

Goal:

- stop using `enforce_pyramiding` as the only distinction between entry and
  generic-order behavior.

Steps:

1. Add an internal command-origin enum covering at least entry, generic order,
   exit, close, close-all, and margin call.
2. Add a stable internal order key or creation sequence that is independent of
   the Pine-visible string id.
3. Store entry versus generic-order origin on current pending market, limit,
   stop, and stop-limit records.
4. Derive pyramiding policy from order origin plus explicit placement policy;
   keep the existing boolean only temporarily if required for a safe migration.
5. Extend pending-entry replacement, cancellation, cloning, equality, and
   rollback tests for the new fields.
6. Do not route fill behavior through a new kernel yet.

Likely owners:

- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/types.rs`
- `crates/pine-runtime/src/strategy/broker/tests.rs`

Acceptance:

- every current pending entry/order records its origin;
- creation ordering survives replacement only according to an explicitly tested
  same-id rule;
- cancellation by public id still removes the same records as before;
- no runtime golden or conformance row changes.

Stop condition:

- stop if one public id cannot safely identify all currently cancellable
  families. Document the lookup policy before widening the order-book facade.

#### 17c. Fill Request And Transition Skeleton

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_FILL_TRANSITION_AUDIT.md`.

Goal:

- introduce one internal representation of an eligible fill and its state
  changes without routing production behavior through it.

Steps:

1. Add `FillRequest` data for order key, bar/time, raw price, and trigger reason.
2. Add `FillTransition` data for closed allocations, optional newly opened
   trade, filled quantity, close/open split, commission, realized PnL, and cash
   delta.
3. Add pure helpers that calculate same-side additions and reduce-only fills
   from an immutable position/ledger snapshot.
4. Add table-driven tests for flat, same-side, partial reduction, flatten, and
   cross-zero shapes. Cross-zero calculation may remain unrouted until Stage 19.
5. Prove invalid or non-finite quantities/prices return an error outcome without
   mutating broker state.

Acceptance:

- transition calculation can be tested without public result generation;
- the skeleton contains no host-specific types;
- no existing fill path has changed yet;
- no public snapshot or matrix update.

#### 17d. Ledger Authority And Aggregate Invariant Checks

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_LEDGER_INVARIANT_AUDIT.md`.

Goal:

- make ledger/aggregate divergence observable before consolidating fill paths.

Steps:

1. Add a private invariant helper that recomputes signed size and weighted
   average price from `TradeLedger`.
2. Assert after every existing entry, close, exit, partial fill, reversal, and
   margin-call mutation that aggregate state matches the ledger.
3. Cover zero, long, short, pyramided same-side, partial allocation, and full
   flatten cases.
4. Identify singleton fields that are historical compatibility mirrors rather
   than authoritative data.
5. Record the retained/deferred field decision in the 17d audit. Do not remove
   fields needed by existing reporting until their readers are migrated.

Acceptance:

- invariant checks pass across the focused strategy suite;
- any exception is documented with an owner and later removal slice;
- production code does not silently repair divergence in release builds without
  reporting or tests.

Stop condition:

- stop if an existing supported path produces ledger/aggregate divergence.
  Fix that defect as its own behavior-preserving regression slice before 17e.

#### 17e. Route Same-Side Entry And Order Fills

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_SAME_SIDE_APPLY_AUDIT.md`.

Goal:

- use the shared transition application path for the lowest-risk open/increase
  cases.

Steps:

1. Route flat and same-side market `strategy.entry()` fills through the new
   transition path.
2. Route flat and same-side market `strategy.order()` additions through it.
3. Preserve current pyramiding differences between the two command origins.
4. Preserve margin admission, commission, slippage, entry metadata, alert
   payloads, position snapshots, trade fields, and deferred-exit attachment.
5. Repeat for limit, stop, and stop-limit same-side fills only after market
   parity is proven.
6. Compare every affected runtime snapshot before and after; Stage 17 requires
   no intentional output differences.

Acceptance:

- existing same-side entry/order snapshots are unchanged;
- broker unit tests prove one accounting application per fill;
- rejected margin fills do not leave an opened trade or stale attachment;
- market and price-based paths share the transition applier rather than copying
  accounting logic.

#### 17f. Route Reduction, Close, Exit, And Margin-Call Fills

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_REDUCTION_APPLY_AUDIT.md`.

Goal:

- consolidate existing close-side mutations without changing their current
  timing yet.

Steps:

1. Route reduce-only market generic orders through shared ledger allocation and
   transition application.
2. Route `strategy.close()` and `strategy.close_all()` allocation results
   through the same transition applier while keeping their Stage 17 timing.
3. Route pending `strategy.exit()` fills through the same applier.
4. Route long and short margin-call liquidation through the same applier with an
   explicit margin-call origin.
5. Preserve FIFO/ANY allocation decisions outside the transition applier; the
   applier consumes allocations but does not choose policy.
6. Preserve current public order/trade/alert identity and quantity fields.

Acceptance:

- full and partial reductions allocate entry commission exactly once;
- close, exit, reduce-only order, and margin-call PnL remain snapshot-identical;
- pending-exit cleanup remains command-specific and fixture-backed;
- no behavior change is hidden in regenerated snapshots.

#### 17g. Remove Legacy Fill Forks And Close Stage 17

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE17_UNIFIED_FILL_AUDIT.md`.

Goal:

- remove obsolete per-command accounting branches after every current fill
  origin is routed through the shared kernel.

Steps:

1. Delete or reduce duplicate cash, PnL, commission, position-sync, trade-record,
   and alert-record helpers that no longer own behavior.
2. Rename misleading `*_long_*` facade methods that now dispatch both sides,
   without changing visibility unnecessarily.
3. Keep modules within the repository source-size guardrail; split by model,
   transition calculation, transition application, and scheduler ownership.
4. Run the complete canonical gate.
5. Create `docs/STRATEGY_INTERNAL_STAGE17_UNIFIED_FILL_AUDIT.md`.
6. Update this document and `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md` to mark
   Stage 17 closed.

Stage 17 completion gate:

- all pre-Stage-17 public strategy snapshots are unchanged;
- no semantic acceptance or conformance claim changed;
- all supported fills use the shared transition applier;
- ledger/aggregate invariant tests cover every fill-origin family;
- `scripts/verify.sh` passes without `UPDATE_SNAPSHOTS`.

## Stage 18: Historical Order Timing And Close Scheduling

Status: closed on 2026-09-04. Slices 18a-18e, 18f scheduler identity, and 18g
true OHLC path execution are complete. See
`docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`.

### Stage 18 Goal

Represent close commands as market orders and introduce an explicit historical
broker scheduler so default next-tick behavior, same-close overrides, pending
entries, pending exits, margin calls, and equity recording have one documented
order of operations.

Stage 18 intentionally changes supported strategy results. It must not begin
until Stage 17 proves the shared kernel can preserve the old outputs.

### Stage 18 Compatibility Decision

The project will follow one modern strategy timing profile while legacy
strategy-version work is paused. Do not add a hidden compatibility flag that
retains current close-at-current-bar behavior.

Before 18c changes default close timing, its audit must record:

- the official rule and review date;
- the current project behavior being replaced;
- every fixture and snapshot expected to change;
- why the change does not require a public JSON schema-version bump;
- the release-note migration statement for callers comparing historical output.

If the affected fixture set cannot be enumerated before implementation, stop.

### Stage 18 Slice Order

#### 18a. Scheduler Characterization And Phase Enum

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE18_SCHEDULER_AUDIT.md`.

1. Document the current historical order: eligible entry fills, trade-extreme
   update, margin call, builtin refresh, script statements, exit fills, equity,
   output commit.
2. Add an internal phase enum or equivalent traceable scheduler state.
3. Route the current calls through a scheduler facade without reordering them.
4. Add a test-only phase trace for one market entry, one price entry, one close,
   one exit, and one margin call.
5. Keep all snapshots unchanged.

Acceptance:

- one scheduler function owns phase ordering;
- runtime evaluation no longer invokes each order-kind fill method directly;
- test traces prove current ordering before it is changed.

#### 18b. Pending Market Close Intent

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE18_PENDING_CLOSE_AUDIT.md`.

1. Add internal pending market-close records for `strategy.close()` and
   `strategy.close_all()`.
2. Store id, resolved or deferred quantity policy, creation bar/tick, metadata,
   and close allocation policy inputs.
3. Keep production dispatch on the old immediate path while testing placement,
   replacement, cancellation interaction, and rollback storage.
4. Decide whether quantity is resolved at placement or fill for each close
   command and fixture-back the decision.
5. Keep pending closes private; do not add public pending-order JSON.

#### 18c. Default Next-Tick Close And Close-All

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE18_NEXT_TICK_CLOSE_AUDIT.md`.

1. Change default `strategy.close()` and `strategy.close_all()` to place market
   orders rather than mutate the broker during statement evaluation.
2. Fill eligible close orders at the next historical bar open.
3. Define same-id repeated placement, full versus partial quantity, missing
   position, position changes before fill, and pending-exit cleanup.
4. Apply entry/exit slippage and commission through the shared Stage 17 kernel.
5. Add paired old-signal/new-fill fixtures for long, short, partial, pyramided,
   metadata, alert, and FIFO/ANY cases.
6. Update affected runtime goldens only after broker/runtime assertions pass.
7. Add incremental parity and CLI/Python/WASM coverage for representative changed
   outputs.

Acceptance:

- default close commands do not fill on their creation bar;
- script-visible state on the signal bar still reflects the pre-fill position;
- next-bar script-visible state observes the filled close;
- no double fill occurs if another order flattens the position first;
- migration behavior is explicit in release notes and the Stage 18c audit.

#### 18d. `immediately` For Close Commands

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE18_IMMEDIATELY_AUDIT.md`.

1. Add builtin and semantic acceptance for const/simple bool `immediately` only
   on supported close/close-all forms.
2. Keep dynamic or invalid types rejected.
3. When true, execute the close through the scheduler's current-tick market
   phase after the command is placed.
4. When false or omitted, preserve 18c next-tick behavior unless declaration
   settings later override it.
5. Prove interaction with partial quantity, metadata, alerts, long/short,
   pyramiding, pending exits, and repeated calls.

#### 18e. Historical `process_orders_on_close`

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE18_PROCESS_ORDERS_ON_CLOSE_AUDIT.md`.

1. Add declaration/IR storage for const bool `process_orders_on_close` in the
   modern strategy profile.
2. Do not accept other execution-timing flags in the same slice.
3. Add a scheduler bar-close fill phase for market orders created during the
   closing script execution.
4. Apply it consistently to entry, generic order, close, and close-all market
   intents whose current supported forms are behavior-backed.
5. Define precedence: `immediately=true` is local to a close command;
   declaration-wide order-on-close affects eligible market orders generally.
6. Add negative combination fixtures for still-unsupported realtime or
   recalculation flags.

#### 18f. Deterministic Historical Bar Path

Status: partial on 2026-09-03 after closeout review. See
`docs/STRATEGY_INTERNAL_STAGE18_FILL_PATH_AUDIT.md`.

1. Replace order-kind call ordering with candidate collection over an explicit
   OHLC path model.
2. Define the path-selection rule, same-price ties, stop-limit activation,
   limit verification, margin-call checks, and downside/upside bracket priority.
3. Give every candidate a deterministic ordering key containing phase, path
   tick, creation sequence, and order key.
4. Add collision fixtures where multiple entry/order/exit candidates become
   eligible on the same bar.
5. Preserve documented price-based same-tick pyramiding exceptions only where
   official behavior and fixtures require them.

Implemented subset:

- broker fills dispatch through ordered `HistoricalFillStep` values;
- market-open, family-based price orders, and bar-close market orders have a
  stable scheduler order;
- a limit/stop collision fixture locks the current family order.

Unfinished from the original acceptance criteria:

- direction-selected OHLC walking rather than fixed order-family walking;
- candidate ordering across entry, generic order, exit, and margin events on
  the same path leg;
- creation-sequence and stable-order-key tie breaking at the same price;
- path-correct stop-limit activation and subsequent limit eligibility.

#### 18g. True OHLC Path And Cross-Family Candidate Ordering

Status: Slice 18g.1 authorized on 2026-09-03 under the B1 amendment in
`docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`. Official closer-to-high
and closer-to-low path rules are locked. Equal-distance is sample-locked OLHC.
B1 same-direction same-price entry/exit is `UNVERIFIED_INTERNAL_ORDER` (runtime
creation sequence/key only; no global entry/exit type rank; no atomicity
inference from missing callbacks). A/C/D stay sample-level ADAUSDT conclusions
and must not be re-run or generalized.

Use the
[detailed Stage 18g execution plan](STRATEGY_INTERNAL_STAGE18G_TRUE_OHLC_PATH_EXECUTION_PLAN.md)
for the step-by-step implementation and verification procedure.

Goal:

- finish the observable requirements originally assigned to 18f without
  changing the public strategy-result schema.

Steps:

1. Recheck the modern official broker-emulator path documentation and record
   the exact reviewed rule and date. If the rule is ambiguous, keep the current
   approximation documented and stop rather than invent behavior.
2. Add a pure historical path builder that emits open, first extreme, second
   extreme, and close legs according to the reviewed rule.
3. Replace family-only price dispatch with candidate collection at each path
   leg. Every candidate must include phase, leg index, trigger price, creation
   sequence, and stable internal order key.
4. Route supported entry, generic-order, exit, and margin candidates through
   the same ordering function. Preserve command-specific allocation and fill
   application outside candidate selection.
5. Model stop-limit activation and limit eligibility as separate events so an
   order cannot fill earlier than the selected path permits.
6. Define same-price ties explicitly, with creation sequence and stable order
   key as the final deterministic tie breakers.
7. Add paired high-first/low-first fixtures, same-price collisions,
   stop-limit activation/fill collisions, exit-versus-entry collisions, and
   margin-versus-exit collisions for long and short positions.
8. Prove historical/incremental parity and representative CLI/Python/WASM
   parity; name every intentional changed snapshot in a new 18g audit.
9. Update conformance, execution semantics, release notes, this plan, and
   `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md` only after the focused tests pass.
10. Run `scripts/verify.sh` without snapshot-update mode and create
    `docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`.

Acceptance:

- fixed order-family rank is no longer the sole price-path model;
- path direction changes observable collision outcomes in dedicated fixtures;
- entry, generic order, exit, and margin candidates share one deterministic
  ordering key;
- stop-limit activation and fill follow the selected path order;
- all changed outputs are intentional, audited, and cross-host consistent;
- `scripts/verify.sh` passes without update mode.

Stop conditions:

- stop if official behavior cannot select an observable path without guessing;
- stop if a candidate mutates broker state before it wins ordering;
- stop if the change requires a public schema expansion without a separate
  schema plan.

Stage 18 completion gate: closed on 2026-09-04 after Stage 18g. See
`docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`.

Prior acceptance criteria, retained as the closed record:

- Stage 18 is not complete until 18g meets the acceptance criteria above;
- the scheduler owns all historical broker phases;
- default close timing, `immediately`, and `process_orders_on_close` are
  fixture-backed;
- every intentional snapshot change is named in an audit;
- unchanged indicator and non-strategy snapshots remain unchanged;
- `scripts/verify.sh` passes without update mode.

## Stage 19: Generic Order Netting And Price-Based Entry Reversal

Status: closed on 2026-09-03. Slice 19a closed on 2026-09-02. Slice 19b closed
on 2026-09-03. Slice 19c closed on 2026-09-03. Slice 19d closed on 2026-09-03.
Slice 19e closed on 2026-09-03. Slice 19f closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19F_REPLACEMENT_CANCEL_CLOSE_RULE_AUDIT.md`.

### Stage 19 Goal

Implement `strategy.order()` as a signed net-position transition independent of
`pyramiding`, then reuse the same cross-zero mechanics for limit, stop, and
stop-limit `strategy.entry()` reversals.

### Stage 19 Netting Contract

For current signed position `P` and a filled generic-order signed quantity `D`:

```text
target_position = P + D
```

The transition must distinguish:

- flat to long/short;
- same-side increase;
- opposite-side partial reduction;
- exact flatten;
- cross-zero reduction plus new opposite exposure.

For `strategy.entry()` reversal, transaction quantity and resulting open
quantity follow entry-specific reversal rules rather than generic-order
addition. The Stage 19a audit must settle public order quantity, alert quantity,
closed-trade allocation, and new open-trade identity before positive routing.

### Stage 19 Slice Order

#### 19a. Transition Matrix And Boundary Fixtures

Status: closed on 2026-09-02. See
`docs/STRATEGY_INTERNAL_STAGE19_NETTING_MATRIX_AUDIT.md`.

1. Write table-driven broker tests for both directions and all five netting
   shapes.
2. Add runtime fixtures that remain unsupported/no-op under the pre-Stage-19
   boundary: long order against short, oversized short order against long,
   price-based opposite orders, and price-based entry reversal.
3. Record official quantity and identity decisions for generic order versus
   entry reversal.
4. Do not widen semantic acceptance or conformance.

#### 19b. Market Generic Order Full Netting

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19B_MARKET_NETTING_AUDIT.md`.

1. Route market `strategy.order(..., strategy.long)` and
   `strategy.order(..., strategy.short)` through signed netting in flat, long,
   and short states.
2. Allocate the reduction portion using the current close-allocation policy
   selected for generic reductions.
3. Open only the cross-zero remainder on the new side.
4. Record one public order fill for the generic order and the appropriate closed
   trade records for reduced ledger entries.
5. Preserve generic-order independence from `pyramiding`.
6. Cover commission, slippage, margin admission, max-held fields, trade fields,
   alerts, and pending-exit cleanup.

#### 19c. Limit Generic Order Netting

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19C_LIMIT_NETTING_AUDIT.md`.

1. Reuse 19b netting after limit trigger selection.
2. Add both-side partial reduction, flatten, and cross-zero fixtures.
3. Verify limit fill assumption and fill price before the transition is applied.
4. Prove cancellation before fill removes the intent without state mutation.

#### 19d. Stop And Stop-Limit Generic Order Netting

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19D_STOP_NETTING_AUDIT.md`.

1. Route stop fills through 19b netting.
2. Route stop-limit activation and later limit fill through the same intent.
3. Preserve activation state across bars and cancellation.
4. Add both-side reduction, flatten, cross-zero, margin rejection, and metadata
   fixtures.

#### 19e. Price-Based `strategy.entry()` Reversal

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19E_ENTRY_REVERSAL_AUDIT.md`.

1. Allow opposite-side limit entries to fill through entry-specific reversal.
2. Repeat for stop entries.
3. Repeat for stop-limit entries only after activation/fill ordering is stable.
4. Apply pyramiding to the resulting new entry side, not the quantity used to
   flatten the old side.
5. Resolve or clear active-entry exit attachments deterministically when the
   target entry reverses the position.
6. Add long-to-short and short-to-long fixtures for each supported order kind.

#### 19f. Generic Order Replacement, Cancellation, And Close-Rule Interaction

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE19F_REPLACEMENT_CANCEL_CLOSE_RULE_AUDIT.md`.

1. Define same-id replacement across market/limit/stop/stop-limit generic
   orders.
2. Define cancellation when an entry, exit, and generic order share a public id.
3. Define generic reduction allocation under FIFO and the currently supported
   id-specific ANY boundary.
4. Keep broader non-id-specific ANY behavior unchanged unless a separate slice
   proves an observable difference.
5. Synchronize conformance, matrix, runtime snapshots, host parity, docs, and
   release notes.

Stage 19 completion gate:

- every supported generic order kind can add, reduce, flatten, or cross zero in
  both directions;
- generic orders remain independent of pyramiding;
- every supported entry order kind can reverse both directions;
- accounting and trade-ledger invariants hold for multi-entry reductions;
- `scripts/verify.sh` passes.

## Stage 20: OCA Groups And Unified Cancellation

Status: closed on 2026-09-03. Slices 20a-20f closed on 2026-09-03.

### Stage 20 Goal

Implement OCA as side-effecting pending-order group state, not passive metadata,
and make cancellation/replacement operate consistently across entry, generic
order, exit, and close intents.

### Stage 20 Slice Order

#### 20a. OCA Storage And Group Identity

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20A_OCA_STORAGE_AUDIT.md`.

1. Add an internal OCA group key composed of name and type.
2. Store group membership on eligible pending intents.
3. Keep `oca_name`/`oca_type` semantic forms rejected while storage-only tests
   cover clone, replacement, cancel, and rollback.
4. Define deterministic behavior when the same name is used with different OCA
   types.

#### 20b. Explicit `strategy.oca.none`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20B_OCA_NONE_AUDIT.md`.

1. Accept const/simple string-compatible OCA names and the explicit `none` type
   for the smallest already-supported pending order family.
2. Prove grouped orders remain independent.
3. Keep cancel/reduce types rejected until their behavior slices land.

#### 20c. `strategy.oca.cancel`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20C_OCA_CANCEL_AUDIT.md`.

1. Start with two generic pending orders in one group.
2. After one fills, cancel every still-pending peer in deterministic order.
3. Clear peer attachments and activation state but preserve unrelated ids and
   groups.
4. Cover simultaneous eligibility, partial fills, replacement, explicit cancel,
   and margin-rejected fills.
5. Widen to supported entry orders only after generic-order behavior is stable.

#### 20d. `strategy.oca.reduce`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20D_OCA_REDUCE_AUDIT.md`.

1. After a fill, reduce each peer's remaining quantity by the filled quantity.
2. Remove peers reduced to zero.
3. Update reservations and deferred exits atomically with quantity reduction.
4. Cover partial fill sequences, multiple peers, over-reduction, same-bar
   eligibility, and cross-zero generic orders.

#### 20e. Exit OCA Naming And Existing Reservation Integration

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20E_EXIT_OCA_NAMING_AUDIT.md`.

1. Map custom exit `oca_name` onto the existing implicit exit-reduction model.
2. Keep exit OCA type behavior consistent with the supported public signature.
3. Define group membership for multiple exit calls targeting the same and
   different open-trade keys.
4. Cover brackets, trailing exits, fixed qty, percent qty, replacement, and
   full-position cleanup.

#### 20f. Unified Cancellation Closeout

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE20F_UNIFIED_CANCELLATION_AUDIT.md`.

1. Make `strategy.cancel(id)` search all supported pending families through one
   order-book lookup policy.
2. Make `strategy.cancel_all()` clear intents, OCA groups, reservations,
   deferred exits, and activation state exactly once.
3. Add collision fixtures for shared public ids across families.
4. Keep public pending-order/OCA schema private.

Stage 20 completion gate:

- none/cancel/reduce behavior is deterministic and fixture-backed;
- cancellation and replacement clear all owned internal state;
- no OCA argument is accepted as inert metadata;
- CLI/Python/WASM parity covers representative public fill changes;
- `scripts/verify.sh` passes.

## Stage 21: Recalculation And Intrabar Scheduling

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21E_BAR_MAGNIFIER_HOST_CONTRACT_AUDIT.md`.

### Stage 21 Goal

Allow bounded repeated strategy execution after fills and on realtime updates
without corrupting series history, `var`/`varip`, objects, alerts, pending
orders, broker state, or committed outputs.

### Stage 21 Slice Order

#### 21a. Execution-Pass Identity And Guardrails

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21A_PASS_IDENTITY_AUDIT.md`.

1. Add bar/tick/pass identity to strategy scheduler state.
2. Add a configurable internal maximum recalculation-pass guardrail.
3. Expose pass counts in internal runtime profiles before enabling extra passes.
4. Snapshot/rollback broker state together with existing runtime forming-bar
   state.
5. Add failure tests for bounded self-triggering order loops.

#### 21b. Historical `calc_on_order_fills`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21B_CALC_ON_ORDER_FILLS_AUDIT.md`.

1. Accept only const bool declaration values.
2. After a fill, schedule another script pass at the next available historical
   price tick according to the Stage 18 bar path.
3. Refresh `strategy.*` state before the extra pass.
4. Prevent the same order intent from filling twice.
5. Define series commit, alert emission, object mutation, and order placement
   between passes.
6. Cover entry/exit cycles, brackets placed from the post-entry average price,
   pyramiding, OCA, margin, and pass-limit diagnostics.

#### 21c. Realtime Broker Rollback Integration

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21C_REALTIME_BROKER_ROLLBACK_AUDIT.md`.

1. Include order book, OCA state, reservations, ledger, cash, alerts, and
   strategy snapshots in forming-bar rollback checkpoints.
2. Re-execute a forming update from the last confirmed checkpoint.
3. Commit only the confirmed update's final broker and output state.
4. Add forming/replacement/confirmed parity tests for orders placed, cancelled,
   activated, and filled intrabar.

#### 21d. `calc_on_every_tick`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21D_CALC_ON_EVERY_TICK_AUDIT.md`.

1. Accept the declaration setting only after 21c rollback tests pass.
2. Execute strategy code on each host-provided forming update.
3. Preserve `var` rollback and `varip` intrabar persistence according to the
   runtime's existing variable model.
4. Prevent abandoned update events from leaking into public orders, trades,
   alerts, plots, or drawings.
5. Add historical/realtime boundary documentation because historical bars do
   not contain arbitrary realtime ticks.

#### 21e. Bar Magnifier Host Contract

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE21E_BAR_MAGNIFIER_HOST_CONTRACT_AUDIT.md`.

1. Design a host-owned lower-timeframe bar input keyed by chart bar.
2. Define absence, gaps, duplicate ticks, timestamp order, and maximum input
   size diagnostics.
3. Reuse the scheduler's tick sequence rather than adding a second broker path.
4. Keep fallback behavior explicit when lower-timeframe data is unavailable.
5. Add CLI/Python/WASM input parity only after the host-neutral schema is
   approved.

Deferred beyond Stage 21 unless separately planned:

- external realtime alert delivery;
- arbitrary exchange tick feeds;
- deep-backtesting UI behavior;
- `calc_on_every_history_tick` or other newly documented execution settings not
  explicitly covered by this plan.

Stage 21 completion gate:

- repeated passes are bounded and profile-visible;
- historical fill recalculation and realtime tick execution have explicit,
  tested commit/rollback rules;
- incremental append agrees with equivalent historical execution where the
  selected mode promises parity;
- `scripts/verify.sh` passes.

## Stage 22: Strategy Risk Rules

Status: closed on 2026-09-03. Slices 22a-22g closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22G_CONS_LOSS_DAYS_AUDIT.md`.

### Stage 22 Goal

Implement risk directives as broker policy that can resize or reject orders,
cancel pending state, flatten exposure, and block later trade actions.

### Stage 22 Slice Order

#### 22a. Risk Configuration And Triggered-State Skeleton

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22A_RISK_SKELETON_AUDIT.md`.

1. Add separate internal `StrategyRiskRules` and `StrategyRiskState` models.
2. Keep every `strategy.risk.*` call rejected while storage and transition tests
   are added.
3. Add hooks before order admission, after fill, at session/day boundary, and
   before forced close.
4. Prove rollback and clone behavior for configured and tripped state.

#### 22b. `strategy.risk.allow_entry_in()`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22B_ALLOW_ENTRY_IN_AUDIT.md`.

1. Accept the documented direction constants only.
2. Permit allowed `strategy.entry()` directions.
3. Convert a disallowed opposite entry against an open position into the
   documented close-only behavior without opening prohibited exposure.
4. Define pending opposite entries when the rule changes or is called repeatedly.
5. Keep generic `strategy.order()` behavior outside this entry-specific rule
   unless official behavior requires otherwise.

#### 22c. `strategy.risk.max_position_size()`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22C_MAX_POSITION_SIZE_AUDIT.md`.

1. Check projected post-fill entry exposure.
2. Reduce entry quantity to the maximum allowed size when possible.
3. Reject/no-op orders whose minimum supported quantity cannot fit.
4. Cover pyramiding, price-based pending entries, reversal, margin, generic
   order distinction, and OCA peers.

#### 22d. `strategy.risk.max_drawdown()`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22D_MAX_DRAWDOWN_AUDIT.md`.

1. Define amount and percent threshold bases from existing equity metrics.
2. On trigger, cancel all pending orders, flatten exposure through a risk-owned
   market close, and block later trade actions.
3. Persist permanent stopped state according to the documented rule.
4. Cover open-profit drawdown, realized drawdown, margin calls, recalculation,
   and realtime rollback.

#### 22e. Intraday Boundary Foundation

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22E_INTRADAY_BOUNDARY_AUDIT.md`.

1. Define trading-day/session reset keys from host-neutral bar timestamps and
   chart/session data already available to the runtime.
2. Add per-window filled-order count and equity baseline state.
3. Prove reset behavior across ordinary sessions, missing bars, and higher-than-
   daily chart timeframes before accepting an intraday risk rule.

#### 22f. `max_intraday_loss` And `max_intraday_filled_orders`

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22F_MAX_INTRADAY_AUDIT.md`.

1. Implement intraday loss after 22e establishes the reset/equity basis.
2. Implement filled-order counting over the same window model.
3. On trigger, cancel pending orders, flatten as required, and block new trade
   actions until reset.
4. Count fills consistently across OCA, partial fills, margin calls, order-on-
   close, and calc-on-fill passes.

#### 22g. Consecutive-Loss-Day Rule And Risk Closeout

Status: closed on 2026-09-03. See
`docs/STRATEGY_INTERNAL_STAGE22G_CONS_LOSS_DAYS_AUDIT.md`.

1. Add daily realized-result aggregation.
2. Implement the documented consecutive-loss-day trigger and permanent stop
   behavior.
3. Add multi-day fixtures with gaps and no-trade days.
4. Synchronize risk diagnostics, conformance, matrix, docs, host parity, and
   release notes.

Stage 22 status: closed on 2026-09-03 after 22g.

Stage 22 completion gate:

- every accepted risk directive has observable broker effects;
- risk triggers atomically cancel, resize, close, or block according to their
  documented policy;
- no risk call is accepted as an inert no-op;
- rule state obeys historical, incremental, recalculation, and realtime
  lifecycle rules;
- `scripts/verify.sh` passes.

## Per-Slice File Checklist

Use this checklist for every positive behavior slice:

1. Builtin surface:
   - `crates/pine-builtins/src/namespaces/strategy.rs`
   - strategy constants when required.
2. Semantic boundary:
   - `crates/pine-sema/src/analyzer/strategy.rs`
   - accepted and rejected fixtures under `tests/fixtures/sema/`;
   - analyzer fixture registration/tests.
3. IR/settings only when the value must survive lowering:
   - `crates/pine-ir/src/strategy.rs`;
   - lowering assertions.
4. Runtime ownership:
   - `crates/pine-runtime/src/strategy/broker/` for broker state and transitions;
   - `crates/pine-runtime/src/builtins/strategy.rs` and submodules for call
     dispatch;
   - `crates/pine-runtime/src/runtime/` for scheduler, pass, and rollback logic.
5. Private helper tests:
   - broker module tests for transition tables, ordering, and atomicity.
6. Public behavior fixture:
   - original `.pine` fixture and focused bars CSV under
     `tests/fixtures/runtime/`;
   - one behavior per fixture where practical.
7. CLI golden:
   - register the fixture with the runtime snapshot suite;
   - refresh only the named expected snapshot.
8. Incremental/realtime coverage:
   - add when timing, retained state, rollback, or repeated execution matters;
   - otherwise record why it is not applicable in the audit.
9. Host parity:
   - Python and WASM assert the representative normalized JSON;
   - update `scripts/host_parity_required.txt` when the fixture becomes a public
     parity requirement.
10. Compatibility contract:
    - update the exact row in `tests/fixtures/conformance.tsv`;
    - refresh `tests/snapshots/matrix.json` only through the established test;
    - preserve the broad unsupported row for deferred variants.
11. Documentation:
    - `docs/EXECUTION_SEMANTICS.md`;
    - `docs/SEMANTIC_MODEL.md` or `docs/BUILTIN_SIGNATURES.md` when applicable;
    - `docs/CONFORMANCE.md`;
    - `docs/RELEASE_NOTES.md`;
    - stage plan status and one new closeout audit.

## Snapshot Update Procedure

Use snapshot-update mode only after direct semantic, broker, and runtime tests
prove the intended output.

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
```

Then:

1. inspect every changed snapshot;
2. reject unrelated changes;
3. rerun both tests without `UPDATE_SNAPSHOTS`;
4. run host-parity validation;
5. record the exact changed snapshots in the slice audit.

Stage 17 must not require intentional runtime snapshot changes. Stage 18 and
later may change behavior only when the owning audit names each changed golden.

## Verification Ladder

Run fast owner-local checks while implementing:

```text
cargo fmt --check
cargo test -p pine-sema strategy
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
python3 scripts/check_structure.py
python3 scripts/check_host_parity.py
```

When WASM or Python public behavior changes, run their focused tests before the
full gate. At slice close, always run:

```text
scripts/verify.sh
```

The audit must report the real commands and results. Do not write “all tests
pass” without naming the gate that was executed.

## Pull Request And Commit Boundaries

- One internal-only skeleton slice may be one pull request.
- One positive behavior slice should normally be one pull request.
- Do not combine Stage 17 refactoring with Stage 18 timing changes.
- Do not combine more than one order kind in a netting PR until the market-order
  slice is closed.
- Do not combine OCA cancel and reduce behavior.
- Do not combine historical calc-on-fill and realtime calc-on-every-tick.
- Do not combine more than one risk-rule family.
- Each behavior PR includes its fixture, conformance change, host parity when
  applicable, docs, release note, and audit; do not defer evidence to a later PR.

## Program Stop Conditions

Stop the active slice and write a design correction before proceeding if:

- official behavior cannot be expressed without guessing an observable rule;
- the current public schema cannot represent a required filled order/trade
  result without ambiguity;
- a refactor changes unrelated snapshots;
- a fill can mutate the ledger before margin/admission validation completes;
- order identity cannot distinguish replacement, cancellation, and OCA peers;
- repeated execution cannot be bounded or rolled back atomically;
- session/day boundaries cannot be derived deterministically from host-provided
  inputs;
- cross-host outputs disagree;
- the full verification gate fails for an unexplained reason.

A stop condition does not authorize accepting approximate semantics. Keep the
feature rejected, document the blocker, and split a smaller prerequisite slice.

## Definition Of Done For The Whole Program

The Stage 17-22 program is complete only when:

- the broker has one explicit order lifecycle and one fill transition applier;
- default historical market-order timing and supported timing overrides are
  scheduler-owned;
- generic orders net correctly in both directions for every supported order
  kind;
- price-based entries reverse through the shared transition model;
- OCA none/cancel/reduce and unified cancellation are deterministic;
- fill recalculation and realtime tick execution are bounded and rollback-safe;
- supported risk rules enforce broker behavior rather than act as metadata;
- unsupported variants still fail closed;
- public schema changes, if any later become necessary, have separate approved
  plans and host migrations;
- conformance, snapshots, docs, release notes, and stage audits agree;
- the final `scripts/verify.sh` run passes without snapshot-update mode.

Current closeout note: Stage 18g true OHLC path execution closed on
2026-09-04. Passing the repository gate proves the documented current subset.
Stage 23 Bar Magnifier fill wiring now has a dedicated execution plan but
remains unimplemented. The deferred general inter-bar gap rewrite remains
outside both closeouts.
