# Strategy Internal Stage 18g True OHLC Path Execution Plan

Status: blocked after Slice 18g.0 on 2026-09-03. Stage 18a-18e are closed.
Stage 18f established a deterministic family-ordered scheduler but did not
implement a true historical OHLC walk. Official high-first and low-first path
rules are locked in
`docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`. Equal-distance
selection, same-price entry-versus-exit rank, same-price exit-versus-margin
rank, and same-bar stop-limit post-activation eligibility remain unresolved
without a lawful TradingView reference export. Do not start Slice 18g.1 or
later behavior-changing work until that audit's design-correction evidence
exists. This document remains the step-by-step plan for closing Stage 18 after
the block lifts.

This plan is subordinate to executable evidence. The compatibility authority
remains `tests/fixtures/conformance.tsv`, its referenced fixtures, committed
snapshots, host-parity assertions, and a passing `scripts/verify.sh` run.

## Outcome

Replace the current fixed price-family order with one deterministic historical
broker path that:

1. selects open-high-low-close or open-low-high-close from the chart bar;
2. visits price crossings in path order rather than order-family order;
3. collects entry, generic-order, exit, and margin events without mutating
   broker state;
4. chooses one deterministic winning event at a time;
5. applies the winner through the existing shared fill-transition machinery;
6. resumes at the same path position after OCA effects or a bounded
   `calc_on_order_fills` execution;
7. models stop-limit and trailing state transitions without allowing a fill
   before its activation point;
8. preserves realtime rollback and historical/incremental/CLI/Python/WASM
   parity; and
9. leaves the public strategy-result schema unchanged.

Stage 18g is complete only when path direction changes observable outcomes in
dedicated fixtures and the full repository gate passes without snapshot-update
mode.

## Current Starting Point

The existing implementation has a sound foundation:

- `StrategySchedulerState` owns bar, phase, fill-step, and recalculation-pass
  identity.
- `HistoricalFillStep` gives market-open, price-family, and optional bar-close
  fills a deterministic order.
- pending entry and generic-order records have an explicit command origin and
  stable internal key.
- pending closes have an internal key.
- `FillRequest` and `FillTransition` provide one shared position-transition
  calculation path.
- the trade ledger and aggregate position mirrors have invariant checks.
- OCA none/cancel/reduce, bounded fill recalculation, realtime broker rollback,
  and broker-enforced risk rules are fixture-backed.
- `MagnifierInput` and `MagnifierHostTicks` already define a host-neutral future
  lower-timeframe input contract.

The remaining observable defect is structural:

- `HistoricalFillStep::pre_script_path()` orders long limit, long stop, long
  stop-limit, short limit, short stop, and short stop-limit families rather than
  walking the bar's price path.
- entry/order fills are processed before whole-bar trade extremes and margin
  checks.
- exit fills are processed after script statements in a separate whole-bar
  high/low pass.
- stop-limit activation currently receives both the bar high and low at once.
- pending exits do not yet carry the same general-purpose internal order key as
  pending entries and closes.
- the current collision fixture proves only the family-order approximation.

Stage 18g must remove those ordering forks without replacing them with a second
broker implementation.

## Authoritative Behavior Lock

Official references reviewed when this plan was written on 2026-09-03:

- https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior
- https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/

The current official broker-emulator documentation states the following
historical-bar assumptions:

- when the open is closer to the high, the inferred path is open, high, low,
  close;
- when the open is closer to the low, the inferred path is open, low, high,
  close;
- every price inside one bar's range is considered reachable along the inferred
  path;
- a price level crossed only in the gap between the previous close and the next
  open fills at the next open rather than at the requested price;
- Bar Magnifier replaces the chart-bar inference with lower-timeframe OHLC
  information when it is available; and
- `calc_on_order_fills` adds an execution after an order fill, not after a
  non-fill bookkeeping transition.

Do not copy TradingView source, private APIs, UI text, or proprietary fixtures.
Use original minimal scripts and user-owned exported results for behavioral
oracles.

### Unresolved Reference Questions

Resolve these questions before the first behavior-changing slice:

1. The current official page does not state which path wins when the open is
   exactly equidistant from the high and low.
2. The exact same-price precedence between a synthetic margin event and a
   user-created exit is not fully specified by the general path description.
3. The existing runtime deliberately delays a stop-limit's limit eligibility;
   same-bar post-activation eligibility must not be widened without exact
   reference-output evidence.
4. Inter-bar gap fills are adjacent to, but not automatically part of, the
   intrabar Stage 18g rewrite.

For each question, create an original minimal strategy, run it against an
explicit OHLC dataset in the reference environment, and save only lawful
source-free order/trade output when source retention is not authorized. Record
the source hash, bar data, platform/version context, observed order sequence,
and review date in the Stage 18g audit.

If an observable rule remains ambiguous, stop before changing that behavior.
Do not infer a compatibility rule from enum order, current snapshots, or a
single accidental result.

## Scope

In scope:

- historical standard-OHLC path selection;
- market-open and optional process-on-close ticks on the shared scheduler;
- price crossing order for the currently supported long and short entry and
  generic-order families;
- fixed stop, limit, bracket, and currently supported trailing exit events;
- stop-limit activation and later limit eligibility;
- margin-call and broker risk state evaluated at the correct path mark;
- OCA and reservation effects caused by a selected fill;
- bounded `calc_on_order_fills` resumption from the selected path position;
- historical, incremental, and realtime rollback behavior;
- CLI/Python/WASM parity for representative changed outputs; and
- documentation, conformance, snapshots, and release-note synchronization.

Out of scope:

- enabling `use_bar_magnifier=true`;
- adding public CLI, Python, or WASM Bar Magnifier inputs;
- mixed entry/order/exit OCA groups beyond the current supported subsets;
- series `oca_name`;
- exchange-session calendars;
- lower-timeframe `request.*` behavior;
- new strategy order families or new risk rules;
- currency conversion, symbol precision, or richer account constraints;
- public pending-order, reservation, candidate, path-trace, or exit-reason
  output;
- changing strategy output schema versions; and
- Pine v1-v4 strategy compatibility.

If Stage 18g appears to require one of these items, split and document a new
design prerequisite rather than silently expanding this stage.

## Required Invariants

Every slice must preserve the following rules.

### Path Invariants

- A valid standard bar produces exactly four ordered path points: open, first
  extreme, second extreme, close.
- The three path legs are monotonic. Candidate crossing order follows the leg's
  direction.
- Degenerate zero-length legs are retained as explicit points or normalized in
  one documented place; they must not cause duplicate fills.
- Non-finite or invalid OHLC input continues to fail at the existing bar-input
  validation boundary.
- Path selection is a pure function of the bar and the locked tie rule.
- The standard-OHLC and future magnifier path sources feed the same broker
  event loop.

### Candidate Invariants

- Candidate collection is read-only. Collecting candidates must leave the
  complete `BrokerState` byte-for-byte or value-for-value equivalent.
- A candidate describes why an event is eligible; it does not remove an order,
  activate a stop-limit, ratchet a trailing stop, alter reservations, apply OCA
  effects, update trade extremes, or change account state.
- The scheduler selects one winner, applies it, then re-collects. It must not
  apply a stale candidate after another event has cancelled, reduced, replaced,
  or invalidated it.
- Every applied event either advances the path cursor, changes one tracked
  broker generation, or terminates the current event loop. A no-progress cycle
  is a runtime error covered by a test.
- A selected candidate is revalidated immediately before application.

### Identity And Ordering Invariants

- One broker-wide monotonic creation sequence orders pending entries, generic
  orders, closes, and exits across families.
- Replacing the same logical pending order keeps its original stable key and
  creation sequence while updating its last-update bar and payload.
- A distinct order receives a new sequence even when its public id matches an
  order from another supported family.
- Per-trade exits expanded from one command receive deterministic keys in trade
  ledger order.
- Synthetic margin/risk events use an explicit stable identity and an
  evidence-backed precedence; they must not impersonate a user order.
- Ordering never depends on `HashMap` iteration, allocation address, test
  execution order, or public string id alone.

### Fill And State Invariants

- Trigger price, crossing position, and actual fill price are separate values.
- Limit-verification offsets affect eligibility without silently changing the
  existing requested fill price.
- Slippage and commission remain owned by the existing fill-application path.
- Admission and margin checks complete before ledger mutation.
- The trade ledger remains authoritative; aggregate position, average price,
  cash, open-trade counts, and public events match it after each selected fill.
- OCA cancellation/reduction and reservation release happen exactly once.
- A stop-limit activation is an internal state transition, not an order fill.
- A trailing activation or ratchet is an internal state transition, not an
  order fill.
- `calc_on_order_fills` runs after a successful fill only. It does not run after
  a rejected fill, stop-limit activation, trailing ratchet, or read-only
  candidate collection.
- Existing `process_orders_on_close` and `immediately` precedence stays intact.

### Rollback And Schema Invariants

- Historical batch and incremental append execution remain item-identical.
- Repeated realtime forming updates restore the confirmed broker, scheduler,
  candidate cursor, stop-limit activation, trailing state, OCA state, and risk
  state before replay.
- Confirmation commits exactly the final forming outcome once.
- `StrategyResult`, Python dictionaries, CLI JSON, and WASM JSON retain their
  current schema.
- Indicator and non-strategy snapshots do not change.

## Target Internal Shape

Names may change during implementation, but ownership must remain equivalent to
the following design.

### Path Model

Create a focused module such as
`crates/pine-runtime/src/runtime/strategy_path.rs` instead of growing
`strategy_scheduler.rs` into another large dispatcher.

The path layer should represent at least:

```text
HistoricalPath
  kind: OpenHighLowClose | OpenLowHighClose
  points: [PathPoint; 4]

PathPoint
  index: 0..3
  kind: Open | High | Low | Close
  price: finite f64

PathLeg
  index: 0..2
  from: PathPoint
  to: PathPoint
  direction: Rising | Falling | Flat
```

Implement path comparison with validated finite prices and `f64::total_cmp` or
an equally explicit comparator. Do not derive `Ord` directly on raw `f64`.

### Broker Order Identity

Move creation-sequence allocation to one broker/order-book owner. The target
shape is:

```text
BrokerOrderKey(u64)
OrderBook
  next_order_sequence: u64
  entries
  exits
  closes
```

`PendingEntry`, `PendingExit`, and `PendingClose` should all retain a stable key.
Remove family-local counters only after replacement, snapshot/restore, and
rollback tests prove the shared allocator.

This is an internal migration. Do not serialize the key or expose it through a
host ABI.

### Candidate Model

Use one broker candidate type for the selected Stage 18g event families:

```text
BrokerCandidate
  identity
  event_kind
  path_leg
  crossing_price
  fill_price_or_mark
  creation_sequence
  stable_order_key
  observed_generation

BrokerCandidateEvent
  EntryOrOrderFill
  ExitFill
  StopLimitActivation
  TrailingActivation
  TrailingRatchet
  MarginCall
  RiskFlatten
```

Keep the public command origin on the underlying pending record. Do not use the
candidate enum as a second source of order semantics.

Candidate ordering on one monotonic leg must be explicit:

1. scheduler phase or path-point class;
2. path leg index;
3. first crossing along the leg: lower prices first on a rising leg and higher
   prices first on a falling leg;
4. evidence-backed same-price event precedence, if required;
5. broker-wide creation sequence; and
6. stable internal key as the final tie breaker.

Do not add an entry-before-exit or long-before-short rank merely to preserve an
old snapshot. When same-price behavior is not specified, obtain reference
evidence or stop.

### Event Loop

The scheduler should converge on one event loop:

```text
for each path point or leg:
    set the unconsumed segment from the current mark to the next path point
    loop:
        collect read-only candidates over the unconsumed segment
        if none are eligible:
            advance the broker mark to the segment endpoint
            update path-local trade extremes and account observations
            break
        select the nearest candidate by the documented comparator
        advance the broker mark to the candidate crossing price
        update path-local trade extremes and account observations
        re-collect same-price candidates and select the evidence-backed winner
        revalidate the winner against the current broker generation
        apply exactly one state transition or fill
        if an order filled:
            apply OCA/reservation effects
            enforce ledger invariants
            run bounded calc_on_order_fills when configured
        prove progress, then collect again from the current mark
```

Advancing the broker mark may update observations such as reached trade
extremes, but it must not itself fill an order. A margin or risk trade action is
still represented by a revalidated candidate at that mark.

Market-open and process-on-close events may use point events rather than price
legs, but they must share the same selection/application machinery where their
ordering can interact with other candidates.

## Worktree And Evidence Setup

Execute from the repository root with a clean `main` synchronized to its
configured upstream.

### Step 1: Record The Starting State

```bash
git status --short --branch
git rev-parse HEAD
git log -1 --oneline --decorate
```

Expected before branching:

- no modified, staged, or untracked project files;
- `main` points at the intended integration commit; and
- the upstream relationship is understood.

If unrelated changes exist, stop and preserve them. Do not mix them into Stage
18g, discard them, or hide them inside a snapshot refresh.

### Step 2: Create The Stage Branch

```bash
git switch -c codex/strategy-stage18g-ohlc-path
```

If that branch already exists, inspect its worktree and history before deciding
whether to reuse it. Do not overwrite it.

### Step 3: Create A Disposable Evidence Directory

```bash
stage18g_scratch=$(mktemp -d "${TMPDIR:-/tmp}/pine-stage18g.XXXXXX")
printf '%s\n' "$stage18g_scratch"
```

The directory is not a release artifact. Copy only summarized command results
into the eventual audit; do not commit compiler output or temporary logs.

### Step 4: Run And Save The Baseline

Use Bash pipe failure propagation when saving logs:

```bash
set -o pipefail
cargo fmt --check 2>&1 | tee "$stage18g_scratch/fmt-baseline.log"
cargo clippy --workspace --all-targets -- -D warnings \
  2>&1 | tee "$stage18g_scratch/clippy-baseline.log"
cargo test -p pine-runtime strategy -- --test-threads=1 \
  2>&1 | tee "$stage18g_scratch/runtime-strategy-baseline.log"
cargo test -p pine-runtime magnifier -- --test-threads=1 \
  2>&1 | tee "$stage18g_scratch/runtime-magnifier-baseline.log"
cargo test -p pine-cli runtime_outputs_match_golden_snapshots \
  2>&1 | tee "$stage18g_scratch/runtime-goldens-baseline.log"
cargo test -p pine-cli matrix_output_matches_golden_snapshot \
  2>&1 | tee "$stage18g_scratch/matrix-baseline.log"
python3 scripts/check_host_parity.py \
  2>&1 | tee "$stage18g_scratch/host-parity-baseline.log"
scripts/verify.sh 2>&1 | tee "$stage18g_scratch/verify-baseline.log"
```

Do not start implementation if the baseline fails. Classify the failure first
as an environment problem, pre-existing regression, or stale generated file.
Stage 18g must not absorb an unrelated repair without an explicit scope change.

### Step 5: Freeze The Initial Snapshot Set

```bash
find tests/snapshots -maxdepth 1 -type f -name 'runtime_strategy_*.json' \
  -print0 | sort -z | xargs -0 sha256sum \
  > "$stage18g_scratch/strategy-snapshots-before.sha256"
```

This list is the evidence boundary for later snapshot review.

## Slice 18g.0: Reference Oracle And Behavior Matrix

Goal: settle observable rules before designing comparator ranks.

### Steps

1. Re-open the official strategy/broker-emulator pages and record the review
   date and relevant headings.
2. Create original minimal reference scripts for:
   - open closer to high;
   - open closer to low;
   - open exactly equidistant;
   - same-price entry versus exit;
   - exit versus margin pressure;
   - stop-limit activation followed by a limit crossing on the same bar; and
   - a requested price crossed only in the inter-bar gap.
3. Use explicit, hand-authored OHLC bars. Avoid market-feed dependence.
4. Export the order/trade results from the reference environment.
5. Record source hashes and source-free result rows when the source itself
   cannot be committed.
6. Write the expected path and event sequence for every case before changing
   Rust code.
7. Classify each case as:
   - confirmed and in scope;
   - confirmed but deferred to a separately named slice; or
   - unresolved and blocking.

### Gate

- high-first and low-first rules have current primary-source support;
- equality, same-price synthetic precedence, and stop-limit timing have an
  explicit disposition;
- no comparator rank is justified only by current runtime behavior; and
- the Stage 18g audit draft contains the reference table.

If equality or a required cross-family rule is still unresolved, stop Stage
18g here and write a design-correction note.

## Slice 18g.1: Pure Historical Path Primitives

Goal: add the path builder without changing broker outputs.

### Expected Files

- `crates/pine-runtime/src/runtime/strategy_path.rs` (new)
- `crates/pine-runtime/src/runtime/mod.rs`
- focused unit tests in the new module
- this plan or the audit draft if the locked rule needs clarification

### Steps

1. Add `HistoricalPathKind`, `PathPoint`, `PathLeg`, and `HistoricalPath`.
2. Implement distance comparison using validated finite values.
3. Implement the evidence-backed equal-distance rule.
4. Emit the four ordered points and three monotonic legs.
5. Define flat-leg behavior explicitly.
6. Add tests for:
   - high-first selection;
   - low-first selection;
   - equal-distance selection;
   - bullish and bearish closes independent of path choice;
   - `open == high`;
   - `open == low`;
   - `high == low == open == close`; and
   - negative and fractional prices, if valid at the shared bar boundary.
7. Add comparator tests that prove crossing order on rising, falling, and flat
   legs.
8. Do not call the new builder from production scheduling yet.

### Verification

```bash
cargo fmt --check
cargo test -p pine-runtime strategy_path -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
git diff --check
```

### Gate

- path construction is pure and fully unit-tested;
- no runtime, matrix, Python, or WASM snapshot changes; and
- the existing family-order production path remains active.

Suggested commit boundary:

```text
refactor(strategy): add historical OHLC path primitives
```

## Slice 18g.2: Broker-Wide Stable Order Identity

Goal: make cross-family tie breaking possible without public ids or collection
iteration order.

### Expected Files

- `crates/pine-runtime/src/strategy/broker/types.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/pending_exits.rs`
- `crates/pine-runtime/src/strategy/broker/pending_closes.rs`
- placement modules that construct `PendingExit`
- broker identity, replacement, snapshot, and rollback tests

### Steps

1. Add one monotonic order-key allocator to the `OrderBook` or equivalent
   broker-owned structure.
2. Route new pending entry, generic order, close, close-all, and exit records
   through that allocator.
3. Add a stable key to `PendingExit`.
4. Preserve keys when the same logical pending record is replaced.
5. Allocate deterministic keys when one exit command expands over multiple
   open trades; use ledger order and test it.
6. Preserve the allocator in `BrokerState` clone/snapshot/restore paths.
7. Remove family-local counters only after all tests pass with the shared
   allocator.
8. Keep OCA membership behavior unchanged; do not broaden mixed-family OCA.
9. Add tests proving:
   - alternating entry/exit/close placement receives one increasing sequence;
   - same-id replacement retains the original key;
   - cancellation followed by a new placement does not reuse a key;
   - snapshot/restore continues from the saved next key;
   - forming rollback discards abandoned keys and replays deterministically;
   - expanded exits have stable ledger-ordered keys; and
   - no public JSON contains the key.

### Verification

```bash
cargo fmt --check
cargo test -p pine-runtime pending_entry_origin -- --test-threads=1
cargo test -p pine-runtime pending_close -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime realtime -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 scripts/check_structure.py
git diff --check
```

### Gate

- stable identity spans every in-scope user-created pending family;
- replacement and rollback preserve identity;
- no observable fill ordering changes yet; and
- runtime snapshots remain unchanged.

Suggested commit boundary:

```text
refactor(strategy): unify pending order creation identity
```

## Slice 18g.3: Read-Only Candidate Collection

Goal: enumerate eligible events without taking records or mutating broker
state.

### Expected Files

- `crates/pine-runtime/src/strategy/broker/candidates.rs` (new, preferred)
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- pending entry/exit books for read-only iteration helpers
- focused candidate unit tests

### Steps

1. Define candidate identity, event kind, trigger price, fill/mark price,
   creation sequence, stable key, and observed broker generation.
2. Add read-only collectors for eligible market-open entries/orders and closes.
3. Add read-only collectors for long and short limit/stop entries and generic
   orders on one path leg.
4. Represent stop-limit activation separately from its limit fill.
5. Add read-only collectors for fixed and bracket exit legs.
6. Represent trailing activation, ratchet, and fill as distinct state events.
7. Add read-only margin/risk candidate calculation at one mark price.
8. Keep limit-verification threshold separate from requested fill price.
9. Implement the evidence-backed candidate comparator.
10. Add candidate-generation tests for every supported direction and origin.
11. Add a hard non-mutation test: clone the broker, collect candidates, and
    assert complete equality with the clone.
12. Add stale-generation tests without applying production behavior yet.

### Verification

```bash
cargo fmt --check
cargo test -p pine-runtime candidate -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 scripts/check_structure.py
git diff --check
```

### Gate

- candidate collection is provably non-mutating;
- comparator ordering is covered for both leg directions and same-price ties;
- every candidate can be revalidated by stable identity; and
- the production family-order scheduler remains active.

Suggested commit boundary:

```text
refactor(strategy): collect broker events without mutation
```

## Slice 18g.4: Entry And Generic-Order Path Execution

Goal: replace family-only price dispatch for pending entries and generic orders
with the true OHLC event loop.

### Steps

1. Add a path cursor to the historical strategy scheduler.
2. Process eligible next-tick market closes and market entries at the open
   point using the shared selection/application boundary.
3. Walk each path leg and collect entry/order candidates.
4. Select and revalidate one winner.
5. Remove/take the selected record only inside the apply step.
6. Route the selected fill through the existing `FillRequest` /
   `FillTransition` path.
7. Apply OCA effects and re-collect at the same path position.
8. Preserve the supported same-tick pyramiding exception only where existing
   fixtures and current official evidence require it.
9. Implement stop-limit activation as a path event.
10. Implement the locked post-activation eligibility rule.
11. Prove that a cancelled or reduced peer cannot fill from a stale candidate.
12. Remove the long-before-short family rank from production price ordering.
13. Retain `HistoricalFillStep` only if it still provides useful point-phase
    identity; it must no longer be the sole price-path model.

### Required New Runtime Fixtures

- `strategy_fill_path_high_first_long.pine`
- `strategy_fill_path_low_first_long.pine`
- `strategy_fill_path_high_first_short.pine`
- `strategy_fill_path_low_first_short.pine`
- `strategy_fill_path_same_price_creation_order.pine`
- `strategy_fill_path_order_oca_cancel.pine`
- `strategy_fill_path_stop_limit_long.pine`
- `strategy_fill_path_stop_limit_short.pine`

Use asymmetric bars so high-first/low-first selection is unambiguous. Do not use
an equidistant bar in a test that intends to prove one of the two primary path
branches.

### Focused Verification

```bash
cargo fmt --check
cargo test -p pine-runtime fill_path -- --test-threads=1
cargo test -p pine-runtime strategy_order -- --test-threads=1
cargo test -p pine-runtime strategy_entry -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime --test incremental -- --test-threads=1
git diff --check
```

### Gate

- high-first and low-first fixtures produce different, expected order
  sequences;
- entry and generic-order candidates use the same comparator;
- stop-limit events cannot fill before activation;
- OCA-invalidated candidates do not fill; and
- no exit or margin behavior has been accidentally changed before its owning
  slice.

Suggested commit boundary:

```text
feat(strategy): order entry fills along historical OHLC paths
```

## Slice 18g.5: Exit And Stateful Price-Event Integration

Goal: move fixed, bracket, and trailing exits into the same path event loop.

### Steps

1. Collect fixed stop and limit exit candidates on each leg.
2. Expand bracket legs into independent candidates tied to one pending exit
   identity; the first selected leg invalidates its peer according to current
   reservation/OCA rules.
3. Replace whole-bar downside-first behavior with actual path crossing order.
4. Integrate long and short trailing activation.
5. Apply trailing ratchets only after the path reaches the new favorable mark.
6. Prevent a trailing stop from using a future extreme that the path has not
   visited.
7. Preserve current creation-bar and replacement eligibility.
8. Re-collect candidates after every exit fill or trailing state transition.
9. Preserve partial quantities, percent quantities, reservations, omitted
   `from_entry`, repeated entry ids, and FIFO/ANY allocation behavior.
10. Add ledger and reservation invariant assertions after every selected exit.
11. Confirm that an exit placed by a fill recalculation can participate only in
    the unconsumed path remainder allowed by the locked timing contract.

### Required New Runtime Fixtures

- `strategy_fill_path_entry_then_exit_same_bar.pine`
- `strategy_fill_path_exit_then_entry_same_bar.pine`
- `strategy_fill_path_bracket_high_first.pine`
- `strategy_fill_path_bracket_low_first.pine`
- `strategy_fill_path_bracket_short_high_first.pine`
- `strategy_fill_path_bracket_short_low_first.pine`
- `strategy_fill_path_trailing_activation_then_fill.pine`
- `strategy_fill_path_trailing_no_future_extreme.pine`
- `strategy_fill_path_exit_oca_reduce.pine`
- `strategy_fill_path_partial_exit_reservation.pine`

### Focused Verification

```bash
cargo fmt --check
cargo test -p pine-runtime fill_path -- --test-threads=1
cargo test -p pine-runtime strategy_exit -- --test-threads=1
cargo test -p pine-runtime oca -- --test-threads=1
cargo test -p pine-runtime reservation -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime --test incremental -- --test-threads=1
git diff --check
```

### Gate

- entry, generic-order, and exit candidates share one ordering function;
- brackets follow path direction rather than unconditional downside priority;
- trailing state never observes a future path extreme;
- OCA and reservations update once; and
- changed legacy strategy snapshots are intentional and enumerated.

Suggested commit boundary:

```text
feat(strategy): integrate exits with historical OHLC paths
```

## Slice 18g.6: Margin, Risk, And Path-Local Account State

Goal: evaluate account transitions at the path price where they become
eligible, not in a detached whole-bar phase.

### Steps

1. Change open-trade high/low extremes from one whole-bar update to path-local
   updates as points are reached.
2. Evaluate equity-dependent risk rules at the current path mark.
3. Collect long and short margin-call candidates without mutating account state.
4. Resolve margin-versus-exit and margin-versus-entry same-price precedence only
   from the Slice 18g.0 evidence.
5. Apply a selected margin call through the existing shared fill-transition
   and allocation code.
6. Recompute equity, held capital, liquidation price, and risk blocking after
   each selected fill.
7. Ensure risk flattening cannot apply twice at one path point.
8. Ensure a margin or risk action invalidates stale entry/exit candidates before
   the next selection.
9. Preserve the current intraday-window reset boundary; exchange calendars are
   not part of this slice.
10. Cover long and short partial liquidation plus a full flatten boundary.

### Required New Runtime Fixtures

- `strategy_fill_path_margin_before_exit_long.pine`
- `strategy_fill_path_exit_before_margin_long.pine`
- `strategy_fill_path_margin_before_exit_short.pine`
- `strategy_fill_path_exit_before_margin_short.pine`
- `strategy_fill_path_drawdown_intrabar_ordering.pine`
- `strategy_fill_path_margin_invalidates_entry.pine`

### Focused Verification

```bash
cargo fmt --check
cargo test -p pine-runtime margin -- --test-threads=1
cargo test -p pine-runtime risk -- --test-threads=1
cargo test -p pine-runtime fill_path -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime --test incremental -- --test-threads=1
git diff --check
```

### Gate

- trade extremes and account risk state advance only with the visited path;
- margin is a deterministic shared candidate, not a detached high/low pass;
- margin fills preserve ledger invariants and recalculate once; and
- existing intraday reset fixtures remain unchanged unless the path order
  provides explicit, audited evidence for a change.

Suggested commit boundary:

```text
feat(strategy): evaluate margin events on historical OHLC paths
```

## Slice 18g.7: Recalculation, Incremental, And Realtime Rollback

Goal: prove that the new event loop composes with Stage 21 rather than bypassing
it.

### Steps

1. Include the current path point/leg and event-loop generation in scheduler
   state where needed for deterministic diagnostics and rollback.
2. After every successful fill, run `recalculate_after_fill` exactly once when
   `calc_on_order_fills` is enabled.
3. Resume from the same path cursor after recalculation.
4. Make newly placed orders obey the locked same-tick and creation-bar
   eligibility rules.
5. Keep the existing maximum recalculation-pass guard effective across all
   path events.
6. Add a no-progress guard for repeated activation/replacement loops.
7. Snapshot and restore every new mutable scheduler field in realtime sessions.
8. Add repeated forming-update coverage that abandons:
   - a stop-limit activation;
   - a trailing activation/ratchet;
   - an OCA peer cancellation;
   - a margin call; and
   - a fill-triggered recalculation order.
9. Confirm the replacement forming update does not inherit abandoned state.
10. Confirm the matching confirmed update commits the outcome once.
11. Run all new fixtures through batch and incremental execution.

### Required New Runtime Fixtures Or Tests

- `strategy_fill_path_calc_on_order_fills_resume.pine`
- `strategy_fill_path_calc_on_order_fills_guard.pine`
- `strategy_fill_path_realtime_stop_limit_rollback.pine`
- `strategy_fill_path_realtime_trailing_rollback.pine`
- `strategy_fill_path_realtime_margin_rollback.pine`
- owned realtime-session coverage for one path-changing strategy

### Focused Verification

```bash
cargo fmt --check
cargo test -p pine-runtime calc_on_order_fills -- --test-threads=1
cargo test -p pine-runtime realtime -- --test-threads=1
cargo test -p pine-runtime --test owned_realtime -- --test-threads=1
cargo test -p pine-runtime --test incremental -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
git diff --check
```

### Gate

- one fill causes at most one immediate recalculation pass;
- path execution resumes rather than restarts from open;
- the pass limit still fails closed;
- forming rollback restores every new internal state component; and
- batch, incremental, forming replacement, and confirmation agree.

Suggested commit boundary:

```text
test(strategy): close OHLC path recalculation and rollback gaps
```

## Slice 18g.8: Public Evidence, Documentation, And Closeout

Goal: make every new compatibility statement inspectable and close Stage 18.

### Steps

1. Add the new runtime fixtures to the narrow existing strategy conformance
   rows; do not create a broad “full broker parity” row.
2. Generate runtime snapshots only after direct broker/runtime tests pass.
3. Add every changed or new required runtime snapshot to
   `scripts/host_parity_required.txt`.
4. Add direct Python and WASM assertions for at least:
   - one high-first long case;
   - one low-first short case;
   - one entry/exit collision;
   - one stop-limit state sequence; and
   - one margin/exit collision.
5. Confirm the CLI owns the canonical JSON golden and hosts only assert parity.
6. Update:
   - `docs/EXECUTION_SEMANTICS.md`;
   - `docs/CONFORMANCE.md`;
   - `docs/LANGUAGE_SCOPE.md` only if its boundary wording is affected;
   - `docs/RELEASE_NOTES.md`;
   - `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`;
   - `docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`; and
   - `docs/README.md`.
7. Create `docs/STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md`.
8. In the audit, record:
   - official review date and unresolved-rule disposition;
   - starting and ending commit ids;
   - implemented path and comparator;
   - internal identity migration;
   - all added fixtures;
   - every intentionally changed snapshot;
   - unchanged schema versions;
   - focused and full verification commands/results; and
   - remaining Bar Magnifier and gap boundaries.
9. Mark Stage 18g closed only after the non-update gate succeeds.

### Snapshot Update Procedure

```bash
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
git status --short
git diff -- tests/snapshots
```

Review every changed snapshot. Then run without update mode:

```bash
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
python3 scripts/check_host_parity.py
```

Reject any unrelated indicator, analysis, or non-strategy snapshot change.

### Final Verification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p pine-sema strategy
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime --test incremental -- --test-threads=1
cargo test -p pine-runtime --test owned_realtime -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
cargo test -p pine-wasm strategy
python3 -m pytest python/tests -q
python3 scripts/check_structure.py
python3 scripts/check_host_parity.py
git diff --check
scripts/verify.sh
```

Run `scripts/verify.sh` without `UPDATE_SNAPSHOTS`. The closeout audit must
record its exit code and the meaningful test/parity counts from the actual run.

Suggested final documentation commit boundary:

```text
docs(strategy): close Stage 18g true OHLC path execution
```

## Fixture Matrix

The exact file grouping may be adjusted, but the following behavior matrix must
be represented by original fixtures before closeout.

| Dimension | Required cases |
| --- | --- |
| Path selection | closer-to-high, closer-to-low, equal-distance disposition, flat/degenerate legs |
| Entry direction | long and short |
| Command origin | `strategy.entry`, `strategy.order` |
| Price order | limit, stop, stop-limit activation and limit eligibility |
| Exit order | fixed stop, fixed limit, both bracket directions, trailing activation/ratchet/fill |
| Collision | entry-entry, order-order, entry-order, entry-exit, exit-margin, same-price |
| Identity | placement/replacement order, key reuse, per-trade expansion |
| OCA/reservation | cancel, reduce, partial exit reservation, peer invalidation |
| Timing | next-bar open, intrabar leg, `process_orders_on_close`, `immediately` regression |
| Recalculation | disabled, enabled, newly placed order, pass-limit failure |
| Account | long margin, short margin, risk flatten, drawdown path mark |
| Execution mode | batch, incremental, realtime forming replacement, confirmation |
| Host | CLI golden, Python parity, WASM parity |
| Gap boundary | reference disposition and either a focused fixture or explicit deferral |

For path-direction fixtures, use bars whose open-to-high and open-to-low
distances are unequal and whose relevant levels are crossed on different legs.
Assert the order id sequence, bar index, fill price, direction, quantity,
position, trades, and equity effect rather than checking only the final net
position.

## Expected Change Allowlist

The implementation will likely touch only these areas:

- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`;
- a new focused strategy-path runtime module;
- `crates/pine-runtime/src/strategy/broker/` identity, candidate, pending-order,
  fill, exit, margin, and risk modules;
- `crates/pine-runtime/src/tests/strategy.rs` and focused integration tests;
- original strategy runtime fixtures and their CLI-owned snapshots;
- the narrow strategy conformance rows;
- Python/WASM host-parity assertions and required-snapshot registry; and
- the documentation files named in Slice 18g.8.

These areas are outside the expected allowlist unless a blocker is documented:

- lexer, parser, or general semantic-analysis behavior;
- indicator built-ins and output collectors;
- request/provider semantics;
- public runtime JSON model structs;
- Python ABI or WASM API signatures;
- release workflow files; and
- Bar Magnifier public acceptance.

Before every commit, inspect the actual scope:

```bash
git status --short
git diff --stat
git diff --check
git diff --cached --stat
git diff --cached --check
```

Stage only the owning slice's explicit file list. Do not use a repository-wide
staging command when unrelated worktree changes exist.

## Stop Conditions

Stop the active slice and write a design correction before proceeding if:

- the equal-distance or required same-price rule cannot be established without
  guessing;
- a candidate collector must mutate pending records to determine eligibility;
- applying one candidate can leave another stale candidate executable;
- no-progress event cycles cannot be bounded and diagnosed;
- stop-limit or trailing behavior requires future path information;
- fill recalculation restarts the bar from open or skips unconsumed path events;
- margin admission occurs after ledger mutation;
- broker/order identity cannot survive replacement and realtime rollback;
- an unrelated snapshot changes;
- CLI, Python, and WASM results disagree;
- a public schema expansion appears necessary;
- Bar Magnifier wiring becomes a prerequisite for standard-OHLC correctness;
  or
- the full gate fails for an unexplained reason.

A stop condition is not permission to preserve an approximation as supported
behavior. Keep the boundary explicit and split a smaller prerequisite.

## Definition Of Done

Stage 18g and Stage 18 are closed only when every item is true:

- [ ] Current official path behavior and all ambiguity dispositions are
      recorded.
- [ ] Standard historical bars select a tested high-first or low-first path.
- [ ] Equal-distance behavior is evidence-backed or explicitly blocks closure.
- [ ] Entry, generic-order, exit, and margin events share one deterministic
      path comparator.
- [ ] Candidate collection is read-only and covered by non-mutation tests.
- [ ] One broker-wide creation sequence provides cross-family stable identity.
- [ ] Same-id replacement retains identity and later new orders do not reuse
      keys.
- [ ] Same-price ties use documented creation-sequence/key ordering.
- [ ] Stop-limit activation and fill follow the selected path and locked timing
      rule.
- [ ] Trailing activation, ratchet, and fill never observe an unvisited extreme.
- [ ] OCA and reservation effects invalidate peers before re-collection.
- [ ] Margin/risk state advances at path-local marks.
- [ ] `calc_on_order_fills` resumes from the current path point and remains
      bounded.
- [ ] Historical and incremental outputs are item-identical.
- [ ] Realtime forming replacement rolls back all new broker/path state.
- [ ] Representative changed outputs match through CLI, Python, and WASM.
- [ ] Public strategy JSON and host API schema versions are unchanged.
- [ ] Every intentional snapshot change is named in the closeout audit.
- [ ] Indicator and unrelated snapshots are unchanged.
- [ ] Conformance, execution semantics, release notes, active plans, and audit
      agree.
- [ ] `git diff --check` passes.
- [ ] `scripts/verify.sh` passes without snapshot-update mode.

## Post-Stage Handoff

After Stage 18g closes:

1. freeze the Stage 18 behavior and prepare the next release boundary;
2. do not combine release/version changes with the final behavior slice;
3. wire the existing Bar Magnifier host contract to this same event loop in a
   new dedicated stage;
4. keep `use_bar_magnifier=true` rejected until host inputs, fallback behavior,
   CLI/Python/WASM parity, and the full gate close together; and
5. defer mixed-family OCA and instrument-session semantics to their own
   evidence-backed slices.

Bar Magnifier must supply a different sequence of input OHLC ticks to this
scheduler. It must not create a parallel broker or restore the old family-order
dispatcher.
