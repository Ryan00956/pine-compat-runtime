# Strategy Internal Stage 23 Bar Magnifier Fill Wiring Execution Plan

Status: planned; not started.

Prepared: 2026-09-04.

Baseline: main at e5943848c, after Stage 18g true-OHLC path execution.

Release posture: this stage is internal development work. It does not publish a
release, change package versions, create release tags, or relax protected-main
requirements.

## 1. Outcome

Stage 23 makes the existing host-owned Bar Magnifier contract an actual input to
the unified historical strategy event loop.

When a v5 or v6 strategy opts in with the named declaration argument
"use_bar_magnifier = true", and the host supplies validated lower-timeframe bars
for a chart bar:

1. the scheduler executes those lower-timeframe bars in chronological order;
2. every lower-timeframe bar uses the existing Stage 18g true-OHLC path model;
3. all order families continue to compete through the existing single-candidate
   broker arbitration;
4. "calc_on_order_fills" resumes from the exact unconsumed lower-bar cursor;
5. missing lower-timeframe coverage falls back to the standard chart-bar path;
6. CLI, Python, WASM, batch, incremental, and historical realtime-seed modes
   agree on results;
7. invalid host input fails closed with stable diagnostics; and
8. the public strategy result schema remains unchanged.

The stage is complete only when the syntax, runtime behavior, host adapters,
fixtures, snapshots, parity evidence, and full repository gate close together.
Until then, "use_bar_magnifier" remains semantically rejected.

## 2. Why this is the next development target

Stage 18g closed the chart-bar OHLC path model. The repository also already has
a validated internal Bar Magnifier host contract in
"crates/pine-runtime/src/magnifier.rs". The missing work is the connection
between those two completed pieces.

That connection is the smallest dependency-complete strategy slice because it:

- reuses the current broker instead of introducing a second execution engine;
- exposes an already-reviewed host-owned capability;
- exercises the Stage 18g cursor and collision rules at a finer granularity;
- provides direct user-visible value before mixed-family OCA and session
  calendars; and
- can be closed without publishing a release or widening unrelated strategy
  semantics.

The next target after Stage 23 remains mixed-family OCA semantics, followed by
session-calendar support. The general chart-to-chart inter-bar gap rewrite stays
a separate later stage.

## 3. Authoritative behavior lock

The following public TradingView behavior was reviewed on 2026-09-04:

- "use_bar_magnifier = true" lets the broker emulator use lower-timeframe OHLC
  data for more granular historical fills.
- Bar Magnifier changes the historical price path used by the broker emulator;
  it does not by itself cause a script to execute on every lower-timeframe
  history tick.
- Additional script executions still depend on calculation settings such as
  "calc_on_order_fills".
- When lower-timeframe coverage is unavailable, the broker emulator falls back
  to its standard chart-bar assumptions.
- TradingView documents a 200,000 lower-timeframe bar limit.

References:

- https://www.tradingview.com/pine-script-docs/concepts/strategies/
- https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/
- https://www.tradingview.com/pine-script-docs/language/declaration-statements/

Stage 23 deliberately does not add the newer
"calc_on_every_history_tick" setting. The repository does not currently model
that setting, and Bar Magnifier is not permission to enable it indirectly.

## 4. Starting point

The implementation starts from these repository facts:

1. "MagnifierInput" already stores lower-timeframe bars grouped by zero-based
   chart-bar index.
2. "magnifier_host_ticks" already returns either the validated group or a
   standard-chart-bar fallback.
3. duplicate chart-bar keys, duplicate lower-bar timestamps, unsorted timestamps,
   and more than 200,000 lower bars already fail closed.
4. "HistoricalPath::from_validated_bar" already produces the Stage 18g OHLC or
   OLHC legs for one validated bar.
5. "StrategySchedulerState" currently tracks only the leg and mark inside one
   chart bar.
6. "HistoricalRuntime" and "RealtimeRuntime" do not own MagnifierInput yet.
7. the IR, builtins, and semantic analyzer do not expose
   "use_bar_magnifier".
8. the CLI, Python binding, and WASM binding do not accept magnifier input.
9. the public strategy result and runtime result schema are already stable.
10. the existing semantic fixture intentionally rejects
    "use_bar_magnifier".

No slice may treat the presence of the internal MagnifierInput type as evidence
that public Bar Magnifier support is already available.

## 5. Scope

### 5.1 In scope

- named v5 and v6 "strategy(..., use_bar_magnifier = <const bool>)"
- host-owned, pre-grouped lower-timeframe bars
- standard OHLC fallback when a chart bar has no usable lower-bar group
- one Stage 18g path per lower-timeframe bar
- one ordered event sequence across all lower bars belonging to a chart bar
- entry, order, exit, risk, margin, trailing, stop-limit, OCA, reservation, and
  recalculate-after-fill behavior already supported by the broker
- "process_orders_on_close" interaction
- existing "calc_on_order_fills" interaction
- batch, incremental, historical replay, and historical realtime seeding
- CLI, Python, and WASM input parity
- diagnostics, fixtures, snapshots, matrix metadata, documentation, and audit
  evidence

### 5.2 Explicitly out of scope

- publishing a release
- version bumps, tags, release notes, or release workflow changes
- automatic lower-timeframe selection
- fetching lower-timeframe bars from a provider or the network
- inferring chart-to-lower-timeframe membership inside the runtime
- Pine v1 through v4 Bar Magnifier
- positional support for "use_bar_magnifier"
- "calc_on_every_history_tick"
- changing realtime tick semantics
- using historical magnifier data for a live/forming realtime bar
- "fill_orders_on_standard_ohlc"
- general chart-to-chart inter-bar gap semantics
- mixed-family OCA expansion
- session calendars
- new order families or new risk families
- new public pending-order fields
- new public StrategyResult fields
- unrelated parser, request-provider, library, plugin, or release work

If an out-of-scope change becomes necessary, stop the stage and write a focused
prerequisite plan. Do not silently absorb it.

## 6. Non-negotiable invariants

### 6.1 One broker and one event loop

Bar Magnifier supplies a different sequence of historical host bars to the
existing event loop. It must not create a magnifier-only broker, alternate
candidate selector, alternate order store, or alternate fill-result builder.

Every lower bar must pass through:

1. the existing HistoricalPath construction;
2. the existing order-state preparation;
3. the existing all-family candidate collection;
4. the existing single-candidate selection;
5. the existing one-fill mutation boundary; and
6. the existing post-fill recalculation path.

### 6.2 Stable chart identity

The public "bar_index" of all fills, orders, trades, and alerts remains the
chart-bar index.

The lower-bar index is internal execution identity. It must be available to the
scheduler, no-progress detector, trace diagnostics, and tests, but it must not
replace the public chart-bar index.

### 6.3 Explicit time domains

Stage 23 must distinguish:

- chart execution time, which remains the script-visible historical bar time;
- lower-bar event time, which may be used internally to order magnifier events;
  and
- public order/trade event time, whose exact contract must be locked in Slice
  23.0 before implementation.

No implementation may accidentally expose lower-bar OHLC or lower-bar time as
the script's chart context during "calc_on_order_fills".

### 6.4 Monotonic cursor

Within one chart bar, the execution cursor is monotonic across:

1. host-bar index;
2. point or leg phase inside that host bar;
3. Stage 18g leg index; and
4. mark within the leg.

A recalculation after fill resumes at the current cursor. It must never restart
the chart bar, restart a lower bar, or make already-consumed price movement
available again.

### 6.5 Determinism

Given the same program, chart bars, magnifier input, request input, and execution
times, all supported hosts and execution modes must produce byte-equivalent
normalized results.

Input collection order must not affect the result after validation and
normalization.

### 6.6 Fail closed

The public syntax remains rejected until all required runtime and host paths are
implemented.

Invalid magnifier input must fail before strategy execution begins. It must not
partially execute some chart bars and then discover a structural input error.

### 6.7 No schema drift by accident

The RuntimeResult and StrategyResult schema versions remain unchanged unless a
separate explicit schema review proves that a version bump is required.

If the Python RealtimeSession lifecycle contract changes, its ABI schema version
must be reviewed explicitly in Slice 23.0. Do not assume that adding an optional
constructor input is automatically version-neutral.

## 7. Canonical host-input contract

Stage 23 uses one host-neutral schema. Adapters may offer an idiomatic wrapper,
but every wrapper must normalize to this exact structure before constructing
MagnifierInput:

~~~json
{
  "schemaVersion": 1,
  "chartBars": [
    {
      "chartBarIndex": 12,
      "bars": [
        {
          "time": 1700000000000,
          "open": 100.0,
          "high": 103.0,
          "low": 99.0,
          "close": 102.0,
          "volume": 1250.0
        }
      ]
    }
  ]
}
~~~

Contract rules:

1. "schemaVersion" is required and must equal 1.
2. "chartBarIndex" is zero-based.
3. chart-bar groups must have unique chart-bar indexes.
4. lower bars inside a group must have strictly increasing timestamps.
5. duplicate timestamps are rejected.
6. all bars must pass the repository's normal finite-number and OHLC validation.
7. the total lower-bar count across all groups must not exceed 200,000.
8. a group index outside the supplied chart-bar range is rejected before
   execution.
9. a missing group uses StandardOhlc fallback.
10. an empty group uses StandardOhlc fallback and the existing gap warning.
11. the host-provided grouping is authoritative.
12. the runtime does not infer lower timeframes or regroup by timestamp.
13. magnifier input is inert when the strategy setting is false.
14. an indicator may receive the host envelope, but it does not consume
    magnifier bars.

CLI contract:

~~~text
pine-compat run SCRIPT --bars CHART.csv --magnifier-bars MAGNIFIER.json
~~~

Python and WASM accept the same schema object. The WASM boundary accepts its JSON
serialization under the reserved "$magnifier" key in the existing request-host
JSON object:

~~~json
{
  "$magnifier": {
    "schemaVersion": 1,
    "chartBars": []
  }
}
~~~

Existing "runScriptCsvWithRequestBars" and "Program.runCsvWithRequestBars"
families carry that object, including their library/input-override variants.
Old calls that omit "$magnifier" remain compatible and mean no magnifier input.

## 8. Target internal execution shape

The target is a sequence adapter around the existing Stage 18g path walker:

~~~text
chart bar
  -> select magnifier group or standard fallback
  -> for each selected host bar in time order
       -> process host-bar open point and local gap semantics
       -> HistoricalPath::from_validated_bar(host bar)
       -> walk each path leg
       -> collect all eligible candidates
       -> choose exactly one candidate
       -> apply exactly one fill
       -> if calc_on_order_fills, recalculate
       -> resume at the exact cursor
  -> run chart-bar close phase
  -> finalize the chart bar
~~~

The scheduler state should be extended conceptually from:

~~~text
{ leg_index, mark }
~~~

to:

~~~text
{
  host_bar_index,
  path_phase,
  leg_index,
  mark
}
~~~

The exact Rust names may differ, but the state model and monotonicity may not.

The selected host sequence is:

- one chart bar when magnifier is disabled;
- one chart bar when magnifier is enabled but coverage is absent or empty; or
- the ordered lower bars for that chart bar when magnifier is enabled and
  coverage is valid.

Do not aggregate a group into one synthetic OHLC bar. That would discard the
ordering information Bar Magnifier exists to provide.

## 9. Mandatory pre-implementation decisions

Slice 23.0 must close the following decisions with an oracle fixture, a written
project rule, or both.

### 9.1 Public fill timestamp

Determine whether public order and trade event timestamps use:

- the chart-bar time;
- the lower-bar time; or
- different documented fields for different meanings.

The script-visible chart time remains the chart time regardless. If official
behavior cannot be observed precisely, choose and document a deterministic
project rule before code changes.

### 9.2 First lower-bar open

Determine whether the opening market phase for a magnified chart bar uses:

- the chart-bar open;
- the first lower-bar open; or
- a chart-open phase followed by a lower-open phase.

If host data contradicts the chart bar, behavior must be explicit. Silent
double-filling is forbidden.

### 9.3 Gaps between lower bars

A price move from one lower bar's close to the next lower bar's open is not a
tradable synthetic segment. Orders crossed in that gap must be evaluated at the
next lower-bar open under a locked better/worse fill-price rule.

This is a magnifier-local point-event rule. It must not claim to close the
separately deferred general chart-to-chart gap rewrite.

If this rule cannot be implemented without changing all chart-level gap
semantics, stop and split out a prerequisite.

### 9.4 Script context after a fill

Lock a fixture proving what the script sees during historical
"calc_on_order_fills":

- "bar_index" stays the chart bar;
- chart OHLC stays the chart bar;
- chart "time" stays the chart execution time;
- internal lower-bar cursor is not exposed as a new Pine execution context.

### 9.5 Realtime boundary

Lock the rule that:

- historical seed bars may consume magnifier input;
- historical replay/confirmed-history updates may consume magnifier input; and
- a live/forming realtime bar consumes actual realtime updates, never the
  historical magnifier manifest.

Define whether a magnifier group targeting the live/forming slot is rejected or
ignored. Prefer rejection because it prevents accidental double execution.

## 10. Branch, evidence, and worktree discipline

Run this stage on one feature branch and one PR. Use sliced commits inside the
PR; do not merge a partially public implementation.

### 10.1 Baseline checks

~~~bash
git status --short --branch
git rev-parse HEAD
git rev-parse origin/main
git log --oneline -8
git diff --check
~~~

Expected starting condition:

- current branch is main;
- HEAD and origin/main both resolve to e5943848c, or the plan is refreshed
  against a newer verified main;
- the worktree is clean.

If main has advanced, re-read the active roadmap, Stage 18g audit, and
Magnifier host-contract audit before continuing.

### 10.2 Create the branch

~~~bash
git switch -c codex/strategy-stage23-bar-magnifier
stage23_scratch=$(mktemp -d /tmp/pine-stage23.XXXXXX)
git status --short --branch > "$stage23_scratch/status.before.txt"
git rev-parse HEAD > "$stage23_scratch/head.before.txt"
~~~

Do not reuse a dirty branch. Do not reset or discard unrelated user changes.

### 10.3 Target-file allowlist

Expected implementation areas:

- "crates/pine-ir/src/strategy.rs"
- strategy builtin declarations under "crates/pine-builtins"
- strategy declaration analysis under "crates/pine-sema"
- "crates/pine-runtime/src/magnifier.rs"
- "crates/pine-runtime/src/runtime/historical.rs"
- "crates/pine-runtime/src/runtime/realtime.rs"
- "crates/pine-runtime/src/runtime/strategy_path.rs"
- "crates/pine-runtime/src/runtime/strategy_scheduler.rs"
- broker modules only where the unified candidate interface requires it
- CLI run options, input parsing, tests, and snapshots
- Python binding inputs, realtime session binding, and tests
- WASM input parsing, compatibility wrappers, and tests
- semantic, runtime, conformance, and host-parity fixtures
- the strategy matrix and relevant documentation

Unexpected changes require an explicit explanation before commit.

Protected areas for this stage:

- release workflows
- package and crate versions
- changelogs for a public release
- plugin artifacts
- unrelated request-provider behavior
- unrelated parser or lexer behavior
- public result field layout

## 11. Slice 23.0 - Behavior oracle and contract lock

Goal: eliminate the remaining semantic ambiguity before changing runtime code.

Expected files:

- a new Stage 23 behavior audit or an evidence section in this plan
- focused oracle scripts under the existing research/fixture convention
- no production runtime change

Steps:

1. Write minimal Pine v6 scripts for:
   - one lower bar allowing entry and exit within one chart bar;
   - a gap between two lower bars crossing a stop and a limit;
   - a fill followed by a "calc_on_order_fills" recalculation;
   - a mismatch between chart open and first lower-bar open;
   - a trailing stop that ratchets across lower bars.
2. Record chart bars, lower bars, strategy settings, order creation time, fill
   bar, fill price, public timestamp, and script-visible OHLC/time.
3. Separate facts observed from official behavior from project-defined fallback
   rules.
4. Decide the public event-time rule from Section 9.1.
5. Decide the first-open rule from Section 9.2.
6. Decide the magnifier-local gap rule from Section 9.3.
7. Decide the realtime-forming rejection rule from Section 9.5.
8. Decide whether the RealtimeSession ABI schema must increment.
9. Write all decisions into the Stage 23 audit before implementation begins.
10. Confirm that none of the decisions requires a public StrategyResult field.

Focused verification:

~~~bash
git diff --check
rg -n "timestamp|first lower|gap|calc_on_order_fills|forming|schema" docs
~~~

Gate:

- every Section 9 question has one explicit answer;
- unresolved official behavior has an explicit deterministic project rule;
- no question is deferred into the implementation.

Stop if:

- correct behavior requires exposing lower-bar OHLC as chart context;
- correct gap behavior requires an unplanned general gap rewrite;
- a new public result field is required; or
- event time cannot be represented without an explicit schema decision.

Suggested commit:

~~~text
Document Stage 23 bar magnifier behavior contract
~~~

## 12. Slice 23.1 - IR and semantic capability gate

Goal: represent the declaration setting internally while keeping public support
fail closed.

Expected files:

- "crates/pine-ir/src/strategy.rs"
- strategy builtin signature definitions
- strategy declaration analyzer
- semantic fixtures and tests

Steps:

1. Add "use_bar_magnifier: bool" to StrategySettings with default false.
2. Add the declaration name to the v5 and v6 strategy builtin signature as a
   const-bool named argument.
3. Preserve the official declaration ordering internally, but do not advertise
   positional support in this stage.
4. Parse and type-check literal or compile-time constant true/false values.
5. Add negative fixtures for:
   - series bool;
   - numeric, string, and other non-bool values;
   - use in unsupported Pine versions;
   - positional use if the repository cannot prove the complete official
     positional signature.
6. Add an internal capability gate after normal type checking:
   - false is accepted because it preserves existing behavior;
   - true emits the existing unsupported-capability diagnostic until Slice 23.7.
7. Add direct IR unit tests proving default false and explicit false.
8. Do not change matrix classification to supported yet.

Focused verification:

~~~bash
cargo test -p pine-ir strategy
cargo test -p pine-builtins strategy
cargo test -p pine-sema strategy
cargo test -p pine-sema unsupported_strategy_use_bar_magnifier
cargo fmt --check
git diff --check
~~~

Gate:

- the setting is represented end to end;
- false has no behavior change;
- true is still rejected;
- all old declaration diagnostics remain stable.

Suggested commit:

~~~text
Model gated strategy bar magnifier setting
~~~

## 13. Slice 23.2 - Canonical host input and runtime ownership

Goal: give the runtime validated, immutable access to one canonical magnifier
input without changing broker behavior yet.

Expected files:

- "crates/pine-runtime/src/magnifier.rs"
- "crates/pine-runtime/src/runtime/historical.rs"
- "crates/pine-runtime/src/runtime/realtime.rs"
- focused runtime tests

Steps:

1. Extend MagnifierInput validation to cover:
   - all normal Bar invariants;
   - chart-bar indexes outside the supplied chart dataset;
   - stable schema-version errors at host boundaries.
2. Preserve existing diagnostic codes for already-covered failures.
3. Add a distinct stable diagnostic for out-of-range chart-bar indexes.
4. Make validation eager: validate the complete manifest before executing bar
   zero.
5. Add an immutable MagnifierInput field to HistoricalRuntime.
6. Keep the default empty so all old constructors and free functions preserve
   exact behavior.
7. Add one builder or setter with clear ownership, such as
   "with_magnifier_input".
8. Avoid adding a new free-function variant for every existing combination of
   request environment, execution times, and input overrides.
9. Add the same builder/ownership path to RealtimeRuntime.
10. Ensure forming-runtime clones carry configuration safely but do not consume
    historical magnifier bars for live/forming updates.
11. Add direct tests for cloning, empty input, sparse input, eager errors, and
    unchanged default behavior.
12. Keep the input inert while the strategy setting is false.

Focused verification:

~~~bash
cargo test -p pine-runtime magnifier -- --test-threads=1
cargo test -p pine-runtime historical -- --test-threads=1
cargo test -p pine-runtime --test realtime -- --test-threads=1
cargo test -p pine-runtime --test owned_realtime -- --test-threads=1
cargo fmt --check
cargo clippy -p pine-runtime --all-targets -- -D warnings
git diff --check
~~~

Gate:

- all structural input errors occur before execution;
- old runtime entry points remain source compatible;
- empty input is behaviorally identical to the baseline;
- no broker or output change has landed.

Suggested commit:

~~~text
Install validated magnifier input in strategy runtimes
~~~

## 14. Slice 23.3 - Sequence cursor and path adapter

Goal: walk multiple validated host bars as one monotonic chart-bar event
sequence.

Expected files:

- "crates/pine-runtime/src/runtime/strategy_path.rs"
- "crates/pine-runtime/src/runtime/strategy_scheduler.rs"
- scheduler/path tests

Steps:

1. Introduce an internal host-sequence item containing:
   - chart-bar index;
   - host-bar index;
   - lower-bar event time;
   - source: StandardOhlc or Intrabars;
   - validated Bar.
2. Convert "magnifier_host_ticks" output into that sequence only when
   StrategySettings.use_bar_magnifier is true.
3. Preserve one standard-chart-bar item when the setting is false.
4. Preserve one standard-chart-bar fallback item when coverage is missing or
   empty.
5. Extend StrategySchedulerState with host-bar identity and point/leg phase.
6. Refactor the current one-bar path walker into a reusable one-host-bar walker.
7. For each host bar, build HistoricalPath with
   "HistoricalPath::from_validated_bar".
8. Execute host bars strictly in validated timestamp order.
9. Never synthesize an aggregate OHLC for a group.
10. Add the locked first-open behavior from Slice 23.0.
11. Add the locked lower-bar gap point-event behavior from Slice 23.0.
12. Do not represent a gap as a tradable close-to-open segment.
13. Include host-bar identity in the no-progress key and internal trace.
14. Ensure advancing a host-bar boundary cannot make an earlier path mark
    eligible again.
15. Add direct cursor tests without involving every broker family yet.

Required unit cases:

- disabled setting produces the existing single-bar cursor;
- enabled setting with no group produces the same cursor plus one warning;
- a three-lower-bar group walks three independent OHLC/OLHC paths;
- a fill at lower bar 0, leg 1 resumes from lower bar 0, leg 1;
- later lower bars remain eligible;
- consumed earlier lower bars never replay;
- doji lower bars terminate;
- empty groups fall back exactly once;
- sparse groups do not shift chart-bar identity;
- lower-bar gaps are point events, not segments.

Focused verification:

~~~bash
cargo test -p pine-runtime strategy_path -- --test-threads=1
cargo test -p pine-runtime strategy_scheduler -- --test-threads=1
cargo test -p pine-runtime magnifier -- --test-threads=1
cargo fmt --check
cargo clippy -p pine-runtime --all-targets -- -D warnings
git diff --check
~~~

Gate:

- cursor state is monotonic across lower bars and legs;
- standard OHLC behavior is unchanged when disabled or falling back;
- no duplicate fill is possible merely because a recalculation occurred.

Stop if:

- the sequence requires a second broker;
- the cursor has to rewind to implement a supported case; or
- the gap rule mutates general chart-to-chart behavior.

Suggested commit:

~~~text
Walk magnifier bars through the unified OHLC cursor
~~~

## 15. Slice 23.4 - Unified broker-family integration

Goal: make all already-supported broker families consume the host-bar sequence
through the existing arbitration rules.

Expected files:

- strategy scheduler
- broker candidate helpers only where they need host-tick identity
- broker and integration tests

Steps:

1. Pass chart-bar identity and lower-bar event identity separately through the
   existing path tick/candidate structures.
2. Preserve the Stage 18g rule: collect all eligible candidates before
   mutation.
3. Preserve the Stage 18g rule: select exactly one candidate at a time.
4. Preserve candidate precedence and tie-breaking.
5. Apply exactly one fill, then invalidate/rebuild candidates.
6. Preserve reservation and OCA reduction/cancellation.
7. Preserve risk and margin precedence.
8. Preserve trailing-stop ratchet monotonicity across lower-bar boundaries.
9. Preserve stop-limit activation memory across lower-bar boundaries.
10. Preserve "process_orders_on_close" as a chart-bar close phase, not a close
    phase after every lower bar.
11. Apply the locked public fill timestamp rule.
12. Keep public fill/order/trade "bar_index" equal to the chart bar.
13. Keep public result shape unchanged.
14. Add collision fixtures that differ from the standard chart-bar path and
    prove that magnifier order is decisive.

Required behavior cases:

- entry limit fills only in a later lower bar;
- entry and exit both fill within one chart bar;
- standard OHLC predicts a fill that the lower-bar sequence disproves;
- lower-bar sequence produces a fill standard OHLC misses;
- strategy.entry competes with strategy.order;
- strategy.exit competes with strategy.close/close_all;
- margin liquidation competes with an exit;
- risk close competes with another family;
- stop-limit activates in one lower bar and fills in a later one;
- trailing activation and ratchet span multiple lower bars;
- OCA cancellation prevents a later lower-bar fill;
- reserved quantity changes invalidate stale later candidates;
- lower-bar open gap uses the locked open price behavior;
- the same timestamp/tie rules remain deterministic.

Focused verification:

~~~bash
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-runtime broker -- --test-threads=1
cargo test -p pine-runtime magnifier -- --test-threads=1
cargo fmt --check
cargo clippy -p pine-runtime --all-targets -- -D warnings
git diff --check
~~~

Gate:

- every supported family uses the same candidate selector;
- every collision has one deterministic winner;
- no result field or schema version changes;
- setting false remains byte-for-byte baseline behavior.

Suggested commit:

~~~text
Route magnifier paths through unified broker arbitration
~~~

## 16. Slice 23.5 - Recalculation and execution-mode equivalence

Goal: prove that recalculation and runtime modes preserve the same cursor and
result.

Expected files:

- historical runtime
- realtime runtime
- incremental/realtime tests
- focused strategy fixtures

Steps:

1. On a magnifier fill, run the existing "calc_on_order_fills" recalculation.
2. Keep the Pine script context on the chart bar.
3. Preserve internal lower-bar cursor state across the recalculation.
4. Allow newly created orders to compete only on the unconsumed remainder of
   the sequence.
5. Preserve the existing pass limit and no-progress protection.
6. Include host-bar identity in pass traces and repeated-state detection.
7. Verify an exit created after an entry fill can fill in a later lower bar.
8. Verify it cannot fill using a price point already consumed before creation.
9. Verify "calc_on_order_fills = false" produces no extra script pass.
10. Verify "calc_on_every_tick" changes only existing realtime behavior and does
    not cause historical execution on every lower bar.
11. Run the same fixture in batch and incremental history.
12. Run the same fixture through historical realtime updates.
13. Seed a Python/Rust realtime session with magnified history and compare it to
    batch.
14. Update a forming realtime bar and prove historical magnifier input is not
    consumed.
15. Roll back/replace a forming bar and prove magnifier history remains
    immutable.
16. Confirm dataset-end and historical-end logic still refer to chart bars.

Required equivalence assertions:

~~~text
batch == incremental
batch == historical realtime replay
batch historical prefix == realtime session seed
forming update without magnifier == existing realtime semantics
~~~

Focused verification:

~~~bash
cargo test -p pine-runtime --test incremental -- --test-threads=1
cargo test -p pine-runtime --test realtime -- --test-threads=1
cargo test -p pine-runtime --test owned_realtime -- --test-threads=1
cargo test -p pine-runtime calc_on_order_fills -- --test-threads=1
cargo fmt --check
cargo clippy -p pine-runtime --all-targets -- -D warnings
git diff --check
~~~

Gate:

- all historical modes agree;
- post-fill orders see only the unconsumed cursor;
- realtime forming bars never double-consume historical lower bars;
- no-progress protection still terminates deterministically.

Suggested commit:

~~~text
Preserve magnifier cursors across strategy recalculation
~~~

## 17. Slice 23.6 - CLI, Python, and WASM host APIs

Goal: expose the same versioned host input through all required adapters without
breaking existing callers.

Expected files:

- CLI run options and parsing
- Python module and RealtimeSession binding
- WASM request-host parser, run module, and Program methods
- adapter tests and usage text

### 17.1 Shared parser behavior

Steps:

1. Define one MagnifierInputV1 contract matching Section 7.
2. Decode each host representation into MagnifierChartBarInput groups.
3. Validate the schema version at the host boundary.
4. Send all groups through the runtime's single
   "magnifier_input_from_groups" semantic validation path.
5. Convert to runtime MagnifierInput only after complete validation.
6. Map each error to a stable host-facing diagnostic.
7. Do not reimplement ordering, duplicate, OHLC, range, or cap validation in
   every host.

### 17.2 CLI

Steps:

1. Add optional "--magnifier-bars PATH" to the run command.
2. Read exactly one JSON manifest.
3. Report malformed JSON, wrong schema version, invalid bars, and runtime
   structural errors with nonzero exit status.
4. Pass the input to batch and incremental history.
5. Pass it to historical realtime replay.
6. For realtime-forming mode, apply it only to the historical seed/prefix under
   the Slice 23.0 rule.
7. Add usage text and focused command tests.
8. Preserve output JSON and trace schema.

### 17.3 Python

Steps:

1. Add optional "magnifier_bars=None" to run and compiled-program entry points.
2. Accept the canonical schema as a Python mapping/list structure.
3. Normalize it through the shared adapter.
4. Add the optional input to RealtimeSession construction or seed according to
   the Slice 23.0 ABI decision.
5. Use magnifier input for historical seed only.
6. Preserve all existing calls with the default None.
7. If the RealtimeSession ABI version changes, update it deliberately and add
   compatibility tests.

### 17.4 WASM

The WASM module already has many combinatorial run entry points and an existing
request-host JSON object with reserved "$chart" and "$executionTimes" keys.
Extend that envelope; do not add a magnifier variant for every combination.

Steps:

1. Extend the current request-host parse result to carry RequestEnvironment,
   optional execution times, and optional MagnifierInput.
2. Reserve "$magnifier" beside "$chart" and "$executionTimes".
3. Exclude "$magnifier" from request-stream key parsing.
4. Decode its value with the canonical schema and runtime validation.
5. Thread the parsed input through every existing "WithRequestBars" script and
   compiled-Program path, including library and input-override combinations.
6. Keep calls that omit "$magnifier" behaviorally identical.
7. Add no new public WASM entry point unless a focused ABI audit proves that the
   existing request-host envelope cannot carry the input.
8. Preserve RuntimeResult JSON schema.
9. Add Node/WASM tests for valid, fallback, invalid, and combined
   "$chart"/"$executionTimes"/request-stream input.

Focused verification:

~~~bash
cargo test -p pine-cli
cargo test -p pine-python
cargo test -p pine-wasm
python3 -m pytest python/tests -q
scripts/check_wasm_node.sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
~~~

Gate:

- one canonical schema reaches all hosts;
- old host calls remain compatible;
- malformed input fails before execution;
- the existing WASM request-host envelope is extended without combinatorial API
  growth;
- realtime history/forming boundaries are explicit and tested.

Suggested commit:

~~~text
Expose versioned magnifier input across runtime hosts
~~~

## 18. Slice 23.7 - Public semantic enablement and conformance

Goal: remove the fail-closed capability gate only after the full stack is
available.

Expected files:

- semantic analyzer and fixtures
- conformance fixtures and expected outputs
- CLI runtime snapshots
- matrix metadata
- Python/WASM parity tests

Steps:

1. Add a minimal accepted v5 fixture using the named argument.
2. Add a minimal accepted v6 fixture using the named argument.
3. Remove only the "true is unsupported" capability gate.
4. Retain every type, version, constness, and positional diagnostic from Slice
   23.1.
5. Convert the existing unsupported fixture into positive and negative focused
   fixtures as appropriate.
6. Add a standard-OHLC baseline fixture with the setting false.
7. Add a no-coverage fallback fixture with the setting true.
8. Add a decisive same-chart-bar entry/exit magnifier fixture.
9. Add a "calc_on_order_fills" cursor fixture.
10. Add a lower-bar gap fixture.
11. Add stop-limit, trailing, OCA, risk, and margin collision fixtures.
12. Run each public fixture through CLI, Python, and WASM.
13. Update runtime snapshots deliberately.
14. Update the matrix only after direct evidence exists.
15. Run host-parity registration checks and add every required fixture.
16. Inspect every changed golden file rather than accepting a bulk rewrite.

Snapshot workflow:

~~~bash
git diff -- crates/pine-cli/src/runtime_snapshots
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
python3 scripts/check_host_parity.py
git diff --check
~~~

Gate:

- syntax is enabled only after every required host consumes the input;
- all positive fixtures have direct expected outputs;
- all negative fixtures retain stable diagnostics;
- snapshots change only for named Stage 23 fixtures and intentional metadata;
- matrix claims match evidence.

Suggested commit:

~~~text
Enable strategy bar magnifier conformance
~~~

## 19. Slice 23.8 - Documentation, audit, and full closeout

Goal: make the implementation reviewable and leave an exact next-step handoff.

Expected files:

- "docs/EXECUTION_SEMANTICS.md"
- "docs/NEXT_INTERNAL_CAPABILITY_PLAN.md"
- "docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md"
- a Stage 23 closed audit
- "docs/README.md"

Steps:

1. Document the canonical host schema and zero-based grouping.
2. Document fallback behavior and every stable diagnostic.
3. Document chart-bar versus lower-bar identity.
4. Document the locked public event-time rule.
5. Document magnifier-local lower-bar gap handling.
6. Document "calc_on_order_fills" cursor behavior.
7. State explicitly that "calc_on_every_history_tick" remains unsupported.
8. State explicitly that live/forming realtime bars do not consume historical
   magnifier input.
9. Record the exact public schema versions and whether the RealtimeSession ABI
   changed.
10. Record focused test commands and results.
11. Record full-gate results.
12. Record snapshot and host-parity deltas.
13. Update the active roadmap to mark Stage 23 closed only after the full gate.
14. Set mixed-family OCA as the next strategy target.
15. Keep session calendars and the general inter-bar gap rewrite ordered after
    their prerequisites.
16. Review the final diff against the target-file allowlist.

Full verification:

~~~bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p pine-ir
cargo test -p pine-builtins
cargo test -p pine-sema
cargo test -p pine-runtime -- --test-threads=1
cargo test -p pine-cli
cargo test -p pine-python
cargo test -p pine-wasm
python3 -m pytest python/tests -q
python3 scripts/check_structure.py
python3 scripts/check_host_parity.py
scripts/check_wasm_node.sh
git diff --check
scripts/verify.sh
~~~

After verification:

~~~bash
git status --short --branch
git diff --stat origin/main...HEAD
git diff --name-only origin/main...HEAD
git log --oneline origin/main..HEAD
~~~

Gate:

- all focused and full verification passes;
- the audit records actual evidence, not planned evidence;
- no release artifact changed;
- no unrelated user change is included;
- roadmap and public support claims match the implementation.

Suggested commit:

~~~text
Close Stage 23 bar magnifier evidence
~~~

## 20. Required diagnostic matrix

The implementation must retain existing codes where already defined and add
stable codes only for newly validated boundaries.

| Condition | Required behavior |
| --- | --- |
| duplicate chart-bar group | fail before execution |
| duplicate lower-bar time | fail before execution |
| unsorted lower-bar time | fail before execution |
| total lower bars over 200,000 | fail before execution |
| invalid finite/OHLC values | fail before execution |
| chart-bar index out of range | fail before execution |
| unsupported schema version | fail at host boundary |
| missing group | standard-OHLC fallback warning |
| empty group | standard-OHLC gap warning |
| setting false with valid input | input inert; no behavior change |
| indicator with valid input | input inert; no strategy behavior |
| magnifier group for live/forming slot | follow locked Slice 23.0 fail-closed rule |

Warnings must be deterministic and de-duplicated to at most one fallback/gap
warning per affected chart bar.

## 21. Required regression matrix

### 21.1 Source selection

- setting omitted
- setting false
- setting true with full coverage
- setting true with sparse coverage
- setting true with empty group
- setting true with no manifest

### 21.2 Path order

- OHLC lower bar
- OLHC lower bar
- doji lower bar
- multiple lower bars with alternating path kinds
- gap-up and gap-down lower-bar opens

### 21.3 Broker state

- entry only
- order only
- exit only
- close and close_all
- stop
- limit
- stop-limit across lower bars
- trailing activation and ratchet
- risk close
- margin liquidation
- reservation and OCA invalidation

### 21.4 Calculation settings

- calc_on_order_fills false
- calc_on_order_fills true
- process_orders_on_close false
- process_orders_on_close true
- calc_on_every_tick remains realtime-only
- calc_on_every_history_tick remains rejected

### 21.5 Execution modes

- batch
- incremental
- historical realtime replay
- realtime session historical seed
- forming realtime update
- forming rollback/replacement

### 21.6 Hosts

- Rust runtime
- CLI
- Python direct run
- Python compiled program
- Python RealtimeSession seed
- WASM direct run
- WASM compiled Program
- Node host-parity gate

### 21.7 Compatibility

- old CLI invocation
- old Python calls without the new optional argument
- old WASM functions
- existing runtime snapshots
- existing Stage 17 through Stage 18g fixtures
- existing realtime ABI fixtures

## 22. Stop conditions

Stop the current slice and investigate before continuing when any of the
following occurs:

1. a public result schema change appears necessary;
2. lower-bar time leaks into script-visible chart context;
3. a recalculation restarts a consumed path segment;
4. more than one candidate mutates broker state in one selection cycle;
5. a magnifier-only broker or order store appears necessary;
6. lower-bar gaps are modeled as tradable synthetic segments;
7. realtime forming updates consume historical magnifier bars;
8. batch and incremental results diverge;
9. CLI, Python, and WASM normalize different host data;
10. a broad snapshot rewrite occurs;
11. existing non-magnifier results change;
12. an unsupported setting becomes accepted incidentally;
13. the pass limiter or no-progress detector no longer terminates;
14. unrelated worktree changes overlap the target files;
15. full verification fails outside an understood Stage 23 delta.

Do not hide a stop condition with a fixture update.

## 23. Pull-request sequence

Use one PR with reviewable commits in this order:

1. behavior contract;
2. gated IR and semantic representation;
3. runtime input ownership;
4. sequence cursor;
5. unified broker integration;
6. recalculation and execution-mode equivalence;
7. host adapters;
8. semantic enablement and conformance;
9. documentation and closeout evidence.

Before opening the PR:

~~~bash
git fetch origin
git rebase origin/main
scripts/verify.sh
git status --short --branch
~~~

If the rebase changes scheduler, magnifier, host adapter, or result-schema code,
rerun the focused gates as well as "scripts/verify.sh".

Do not publish a release after merge. Release work requires a separate explicit
request and a separate release-readiness review.

## 24. Definition of done

Stage 23 is done only when every item below is true:

- [ ] Section 9 behavior questions are locked in writing.
- [ ] "use_bar_magnifier" is a typed v5/v6 named const-bool setting.
- [ ] positional and unsupported-version behavior is explicit.
- [ ] true remains rejected until the final enablement slice.
- [ ] one canonical MagnifierInputV1 schema is shared by all hosts.
- [ ] complete input validation occurs before execution.
- [ ] HistoricalRuntime owns immutable magnifier input.
- [ ] RealtimeRuntime applies it only to historical execution.
- [ ] the scheduler cursor includes lower-bar identity.
- [ ] each lower bar uses the existing Stage 18g HistoricalPath.
- [ ] lower-bar gaps are point events under a locked rule.
- [ ] all broker families use the existing unified candidate selector.
- [ ] one fill occurs per arbitration cycle.
- [ ] "calc_on_order_fills" resumes from the exact remaining cursor.
- [ ] script-visible chart context remains chart-scoped.
- [ ] public event timestamp semantics are documented and tested.
- [ ] missing and empty coverage fall back deterministically.
- [ ] batch, incremental, and historical realtime results agree.
- [ ] live/forming realtime behavior is unchanged.
- [ ] CLI accepts the versioned manifest.
- [ ] Python accepts the canonical structure.
- [ ] Python RealtimeSession ABI handling is explicit.
- [ ] WASM extends the existing request-host envelope without combinatorial API
      growth.
- [ ] old host calls remain compatible.
- [ ] valid, fallback, invalid, and collision fixtures exist.
- [ ] host parity includes the required Stage 23 fixtures.
- [ ] matrix claims are evidence-backed.
- [ ] public RuntimeResult and StrategyResult schemas are unchanged, or an
      explicitly approved schema plan supersedes this invariant.
- [ ] all focused gates pass.
- [ ] "scripts/verify.sh" passes.
- [ ] the final diff contains no release or unrelated files.
- [ ] a closed Stage 23 audit records the actual evidence.
- [ ] the roadmap names mixed-family OCA as the next target.

## 25. Handoff after Stage 23

After Stage 23 closes:

1. do not release automatically;
2. retain the Stage 23 branch/PR evidence;
3. refresh the current main/protection/CI state;
4. start the mixed-family OCA behavior audit;
5. keep session calendars ordered after OCA unless new evidence changes the
   dependency;
6. keep the general chart-level inter-bar gap rewrite separate from the
   magnifier-local lower-bar-open rule; and
7. continue to keep unsupported strategy settings fail closed.

The completion claim should be:

> Bar Magnifier historical fill wiring is closed across Rust, CLI, Python, WASM,
> execution modes, and evidence gates.

It must not be:

> Strategy compatibility is complete.
