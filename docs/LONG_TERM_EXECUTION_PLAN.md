# Long-Term Execution Plan

This document tracks the remaining compatibility work after the current
indicator-focused baseline. It is intentionally broader than the next-stage
playbook: use it to choose future phases, but continue to land changes through
small, fixture-backed increments.

The current rule remains unchanged: a feature is only claimed in
`tests/fixtures/conformance.tsv` when syntax, semantic analysis, runtime
behavior, public outputs, docs, and fixtures all agree.

## Current Baseline

Implemented or partially implemented:

- Historical indicator execution over OHLCV bars.
- Incremental append-bar execution.
- Realtime forming-bar rollback for outputs, `var`, callsite state, and float
  arrays.
- `if`/`else`, local scopes, tuple declarations, reassignment, and local `var`
  declaration-site storage.
- User-defined functions with expression bodies, block bodies, positional and
  named arguments, local declarations, loops inside functions, and independent
  callsite state.
- Partial `switch` expressions.
- Partial `for` and `while` loops.
- Constant non-negative history offsets and guarded dynamic integer offsets,
  including `series int`.
- Partial float/int/bool/string/color arrays, reference assignment,
  `array.from`, `array.copy`, negative indexes for
  `array.get`/`array.set`/`array.insert`/`array.remove`, `array.slice`,
  `array.concat`, `array.fill`, search/binary search helpers, truth helpers,
  numeric abs/statistics/range/median/mode/percentile/covariance/standardize/variance/stdev
  helpers, queue/end helpers, numeric/string ordering helpers including
  `array.sort_indices`, `array.join`, and supported array method calls. The
  current scalar array pass is closed in `docs/ARRAY_STAGE_AUDIT.md`; `array.*`
  remains partial because generic, object, UDT, matrix/map, history, and Pine
  shallow-slice semantics are still out of scope.
- A fixture-covered set of `input.*`, output calls, color helpers, math
  helpers, and `ta.*` functions. The main built-in coverage pass is closed in
  `docs/PHASE_D_AUDIT.md`; further built-in work should be maintenance unless
  a later phase needs it.
- Partial alert events for `alertcondition()` and `alert()` with a versioned
  public `alerts` runtime output. The Phase H alert subset is closed in
  `docs/PHASE_H_AUDIT.md`; frequency modes, placeholders, host delivery, and
  strategy alerts remain out of scope.
- Phase J libraries/user types are closed for the fixture-backed claimed
  subset in `docs/PHASE_J_AUDIT.md`: host-provided exact-key imports expose
  exported const expressions and pure exported functions, local scalar-field
  UDTs support construction, field reads, ordinary variables, and `var`
  persistence, and pure local UDT methods are supported.
- CLI, Python, and WASM surfaces for the supported runtime result model.

Remaining work falls into the phases below.

## Execution Rules

- Add or update fixtures before or alongside implementation.
- Keep unsupported variants diagnostic-only until their behavior is designed.
- Prefer one feature through the full stack over several parser-only changes.
- Preserve deterministic runtime guards for loops and mutable storage.
- Keep public JSON, Python, and WASM output schemas synchronized.
- Update `tests/fixtures/conformance.tsv` only after fixture coverage exists.
- Run the full verification set before calling a phase complete.

Recommended verification:

```text
git diff --check
scripts/verify.sh
```

## Phase A: Loop and Branch Hardening

Status: complete for the current executable subset. See
`docs/LOOP_BRANCH_AUDIT.md`.

Goal: turn the current partial loop support into a more reliable Pine subset
before adding larger runtime systems.

Scope:

- Broaden `for` fixtures for zero-iteration behavior, `na` loop bounds, step
  direction edge cases, nested loops, loop counter shadowing, and loop results.
- Broaden `while` fixtures for `na` conditions, nested loop control, local
  declarations, local `var`, and stateful calls inside loop bodies. Initial
  coverage exists; keep adding real-script cases as gaps appear.
- Add more branch interaction fixtures: `if` inside loops, loops inside `if`,
  `switch` inside loops, and loops inside UDFs. Initial coverage exists; keep
  adding real-script cases as gaps appear.
- Keep `while` expression results rejected until a dedicated expression-result
  design exists.
- Keep statement-block `switch` arms rejected until block-arm scoping and
  result semantics are designed.

Out of scope:

- Per-variable `max_bars_back` declarations and inference.
- Object systems.
- Multi-timeframe data.

Acceptance criteria:

- Runtime fixtures cover nested loop control and stateful calls inside loops.
- Incremental append execution matches full historical execution for all new
  loop fixtures.
- Diagnostics remain stable for unsupported loop forms.
- `for`, `while`, and `switch` conformance notes describe the exact supported
  subset.

Suggested commits:

1. `Harden for loop edge cases`
2. `Harden while loop edge cases`
3. `Cover loop branch interactions`
4. `Document loop compatibility boundaries`

## Phase B: Collections Beyond Float Arrays

Goal: expand mutable collection support without breaking the runtime-owned
storage model.

Status: the scalar typed-array subset is fixture-backed and should be treated
as closed for now. Use `docs/ARRAY_STAGE_AUDIT.md` before selecting any further
array work.

Scope:

- Add typed arrays beyond the current float/int/bool/string/color subset:
  source where practical.
- Add common array constructors and helpers after element typing is stable.
- Define copy/reference behavior for array values across assignments,
  function calls, `var`, rollback, and incremental execution.
- Expand method syntax for supported array functions.
- Add precise diagnostics for still-unsupported array operations.

Later candidates:

- Matrices.
- Maps.
- Array sorting/searching/statistical helpers.
- Generic collection behavior if the type system can support it cleanly.

Acceptance criteria:

- Each supported element type has creation, mutation, read, persistence,
  rollback, and UDF-boundary fixtures.
- Unsupported collection variants are rejected during semantic analysis.
- Runtime profiles include enough collection storage information to catch
  uncontrolled growth.

Suggested commits:

1. `Document collection semantics`
2. `Design map collection support`

## Phase C: History and Series Semantics

Goal: make series behavior closer to Pine while keeping static guarantees where
possible.

Status: substantially complete for the current executable subset. The static
and integer dynamic history boundary is recorded in
`docs/HISTORY_SERIES_AUDIT.md`; qualifier findings are recorded in
`docs/QUALIFIER_AUDIT.md`; built-in signature notes were tightened in
`docs/BUILTIN_SIGNATURES.md`. Series-qualified integer history offsets are
supported with runtime guards. Runtime profiles expose max series depth and
history retention mode, and committed series history has a hard runtime cap. HIR
lowering records program-wide and per-series history requirements, runtime
retention trims static-only scripts to those requirements, and
`indicator(..., max_bars_back=N)` bounds dynamic retention.

Remaining follow-up:

- Add clearer diagnostics/profile fields or per-variable `max_bars_back`
  handling for scripts that depend on dynamic history.
- Continue expanding tests around `na`, first-bar behavior, and interactions
  with more built-ins as needed.
- Revisit scalar `simple` inference and qualifier-bound helper APIs if Phase D
  built-ins need stricter qualifier semantics.

Residual risks:

- Dynamic offsets can require deeper history retention and new runtime bounds.
- Future qualifier changes may affect many built-in signatures at once.

Acceptance criteria:

- Any supported dynamic-offset subset has explicit retention limits/profile
  fields and runtime errors for unsafe offsets.
- Built-in signature docs match semantic checks.
- Existing fixture results remain stable unless a deliberate compatibility fix
  is documented.

Suggested commits:

1. `Document history series audit`
2. `Audit qualifier propagation`
3. `Design dynamic history offset support`
4. `Implement guarded dynamic history offsets`
5. `Infer history retention requirements`
6. `Trim static history retention`
7. `Support series history offsets`
8. `Expose history retention profile`
9. `Support indicator max bars back`
10. `Cover dynamic history scopes`
11. `Close Phase C audit`

## Phase D: Built-In Coverage Expansion

Status: closed for the current executable indicator subset. See
`docs/PHASE_D_AUDIT.md`.

Goal: grow useful indicator compatibility through high-value built-ins before
large platform features.

Closed coverage:

- Additional `ta.*` functions. Initial Phase D coverage includes `ta.stdev`
  and `ta.variance` with default biased and optional sample window modes, plus
  `ta.range`, `ta.dev`, `ta.vwma`, `ta.wma`, `ta.hma`, `ta.swma`, `ta.alma`,
  `ta.linreg`, `ta.bbw`, `ta.correlation`, `ta.covariance`, `ta.median`,
  `ta.mode`, `ta.kc`, `ta.kcw`,
  `ta.pivothigh`, `ta.pivotlow`,
  `ta.percentile_nearest_rank`, `ta.percentile_linear_interpolation`, and
  `ta.percentrank` over ready rolling windows, `ta.cum` cumulative sums,
  `ta.cmo` Chande Momentum Oscillator rolling change sums,
  `ta.cci` Commodity Channel Index mean-deviation oscillator,
  `ta.cog` Center of Gravity rolling weighted-ratio oscillator,
  `ta.ao` Awesome Oscillator fast/slow median-price SMA spread,
  `ta.bop` Balance of Power OHLC range oscillator,
  `ta.tsi` True Strength Index double-EMA momentum support,
  `ta.accdist`/`ta.iii`/`ta.nvi`/
  `ta.obv`/`ta.pvi`/`ta.pvt`, partial `ta.vwap` variable/source/anchor/bands
  support,
  `ta.mfi` Money Flow Index support,
  `ta.pivot_point_levels` runtime-bar anchored pivot arrays,
  `ta.max`/`ta.min` all-time source extremes, `ta.tr` as a true range series
  variable, `ta.wad`/`ta.wvad` flow variables, `ta.supertrend`
  line/direction tuple support, `ta.dmi` directional movement tuple support,
  `ta.sar` Parabolic SAR support, and `ta.stoch` stochastic oscillator support,
  `ta.wpr` Williams %R support, `ta.dema`/`ta.tema` EMA-chain smoothing,
  numeric `series int`/`series float` sources across core TA windows,
  `ta.change` over numeric and bool series sources,
  `ta.mom`/`ta.roc` over explicit source history, and
  `ta.rising`/`ta.falling` trend-window checks. It also includes
  source/length and length-only rolling extreme helpers, `ta.barssince`
  condition counters, plus `ta.valuewhen` condition occurrence lookups.
- Additional `math.*` and `str.*` helpers. Initial post-baseline math coverage
  includes `math.floor`, `math.ceil`, `math.sqrt`, `math.log`, `math.log10`,
  `math.exp`, `math.acos`, `math.asin`, `math.atan`, `math.sign`,
  `math.todegrees`, `math.toradians`, `math.sin`, `math.cos`, `math.tan`,
  `math.trunc`, `math.cbrt`, `math.hypot`, `math.avg`, `math.e`, `math.pi`,
  `math.phi`, `math.rphi`, `math.pow`, `math.round`,
  `math.round_to_mintick`, `math.random`, and `math.sum`;
  string coverage includes `str.split`; time
  helper coverage includes the numeric UTC `timestamp`, `weekofyear`,
  `dayofweek`, and fixed-default `time_close` subsets, plus a fixed-default
  `timeframe.period` and `timeframe.in_seconds`/`timeframe.from_seconds`
  conversion subset, `timeframe.change` UTC bucket detection, plus
  fixed-default `timeframe.is*` and `timeframe.multiplier` metadata.
- Initial utility helper coverage includes `fixnan` for the current numeric
  and color last-non-`na` callsite subset.
- Initial type-cast coverage includes the pure scalar `int`, `float`, `bool`,
  `string`, and `color` numeric/bool/string/color/`na` subset.
- Initial color helper coverage includes `color.from_gradient` linear RGBA
  interpolation and `#RRGGBB`/`#RRGGBBAA` hex color literals.
- Initial symbol metadata coverage includes a fixed-default `syminfo.*` subset
  for common string, tick-size, and price-scale fields.
- Initial `barstate.*` coverage includes `barstate.isfirst`,
  `barstate.islast`, `barstate.isnew`, `barstate.isconfirmed`,
  `barstate.ishistory`, and `barstate.isrealtime`.
- Initial session-state coverage includes fixed-default `session.ismarket`,
  `session.ispremarket`, and `session.ispostmarket`.
- Initial global derived source coverage includes `hl2`, `hlc3`, `hlcc4`, and
  `ohlc4` as bar-local OHLC price series.
- Initial `input.*` metadata coverage accepts common min/max/step, `options`,
  `tooltip`, `inline`, `group`, `confirm`, and `display` parameters while
  continuing to execute the `defval` value. Additional string-like input
  coverage includes `input.session` and `input.text_area`.
- Initial output metadata coverage accepts common style/display/editability
  parameters on `plot`, `hline`, `fill`, `bgcolor`, and `barcolor` without
  changing the normalized runtime output schemas. Direct `display.*` constant
  coverage includes all/none plus pane, price scale, status line, and data
  window values.
- Initial `plotchar` metadata coverage accepts common marker display/style
  parameters while keeping the normalized value/char/color output schema.
- Initial `plotshape`/`plotarrow` metadata coverage accepts common marker
  display/style parameters while preserving existing normalized output schemas.
- Initial `plotbar`/`plotcandle` metadata coverage accepts common display
  parameters while preserving existing OHLC output schemas.

Maintenance tails:

- `ta.vwap` session-derived anchoring remains future work.
- `color.*` named constants are a common registry, not an exhaustive claim.
- More complete host-side input override APIs remain future work.
- More plot options, visibility controls, styles, and display-driven renderer
  behavior remain future work.

Maintenance order:

1. Prefer small compatibility fixes inside already supported built-in families.
2. Keep new stateful helpers explicit about callsite state and rollback.
3. Move public output schema expansion into Phase K or the relevant platform
   phase.

Acceptance criteria:

- Every maintained built-in change has semantic signature tests and runtime
  fixtures.
- Stateful built-ins behave correctly inside `if`, `switch`, loops, and UDF
  callsites when those combinations are claimed.
- CLI, Python, and WASM expose any new result fields consistently.

## Phase E: Drawing Object Systems

Goal: support Pine drawing objects as first-class runtime outputs.

Execution playbook: `docs/PHASE_E_EXECUTION_PLAN.md`.
Closeout audit: `docs/PHASE_E_AUDIT.md`.

Status: closed for the current fixture-backed partial drawing platform. Labels,
lines, boxes, and tables have public sparse snapshots, deterministic ids,
limits, incremental append coverage, and realtime rollback coverage for their
claimed method subsets. `polyline.*` remains unsupported pending `chart.point`
and point-list array design.

Object families:

- `label.*`
- `line.*`
- `box.*`
- `table.*`
- `polyline.*`

Required design:

- Runtime object ids and lifetime rules.
- Per-bar creation, mutation, and deletion semantics.
- Rollback behavior for forming bars.
- Output schema for object snapshots or event streams.
- Limits for object counts and memory use.
- UDF side-effect policy for object creation and mutation.

Acceptance criteria:

- Object creation, update, delete, rollback, and limit fixtures exist for each
  supported family.
- Public outputs are stable enough for downstream renderers.
- Unsupported object families or methods produce precise diagnostics.

Suggested first slice:

1. Implement `label.new` plus a minimal immutable snapshot output.
2. Add `label.set_*` mutation methods.
3. Add deletion and object limits.
4. Repeat the pattern for `line` after label semantics settle.

## Phase F: `request.*` and Multi-Timeframe Data

Goal: support external data requests only after the runtime has a clear data
provider abstraction.

Execution playbook: `docs/PHASE_F_EXECUTION_PLAN.md`.

Scope:

- Design a host data-provider API for symbols and timeframes.
- Define bar alignment and gap behavior.
- Define caching and deterministic replay semantics.
- Implement a narrow `request.security` subset first.
- Preserve diagnostics for unsupported request variants.

Risks:

- Multi-timeframe alignment can change stateful-call behavior.
- Host APIs differ across CLI, Python, WASM, and future embeddings.

Acceptance criteria:

- Fixtures cover higher-timeframe and lower-timeframe alignment.
- Historical, incremental, and realtime paths agree.
- Missing data and provider errors produce stable diagnostics or runtime
  errors.

## Phase G: Strategy Runtime

Goal: add `strategy.*` only as a separate runtime mode, not as a small built-in
extension.

Execution playbook: `docs/PHASE_G_EXECUTION_PLAN.md`.

Scope:

- Strategy declaration and settings.
- Order placement functions.
- Broker emulator state.
- Position, trade, equity, commission, slippage, and pyramiding semantics.
- Strategy output schema.

Dependencies:

- Stable historical execution.
- Clear result schema versioning.
- Dedicated strategy fixtures separate from indicator fixtures.

Acceptance criteria:

- Indicator and strategy runtime modes are clearly separated.
- Order execution is deterministic and fixture-backed.
- Public APIs expose strategy results without weakening indicator results.

## Phase H: Alerts

Status: closed for the fixture-backed claimed subset. See
`docs/PHASE_H_AUDIT.md`.

Goal: support alert surfaces after series and condition evaluation semantics
are stable.

Execution playbook: `docs/PHASE_H_EXECUTION_PLAN.md`.

Delivered scope:

- `alertcondition(condition, title, message)` with bool-compatible conditions
  and const-string title/message values.
- `alert(message)` with const-string messages.
- Deterministic public runtime alert events in `schemaVersion: 3`.
- Historical, incremental, and realtime rollback fixture coverage.
- Stable diagnostics for frequency modes, placeholders, side-effect contexts,
  and requested-context alert side effects.

Maintenance tails:

- Alert frequency modes and placeholder interpolation need a dedicated design
  before support is claimed.
- Host-side alert delivery APIs are not part of the current runtime host
  surfaces.
- Strategy alerts remain blocked on Phase G strategy runtime semantics.

Acceptance criteria:

- Alert outputs are represented as deterministic events.
- Realtime forming-bar behavior is explicitly documented.
- Unsupported alert options remain diagnostic-only.

## Phase I: `varip` and Intrabar Persistence

Status: closed for the fixture-backed claimed subset. See
`docs/PHASE_I_AUDIT.md`.

Goal: implement intrabar persistence only after realtime update semantics are
fully specified.

Execution playbook: `docs/PHASE_I_EXECUTION_PLAN.md`.

Delivered scope:

- `varip` declaration analysis.
- Intrabar storage distinct from confirmed-bar `var` storage.
- Interaction with forming-bar rollback.
- Interaction with scalar typed-array ids and backing contents.

Maintenance tails:

- Drawing object ids remain rejected for `varip` until object-store handoff rules
  are designed.
- Tuple `varip`, maps, matrices, UDTs, imports, object arrays, generic arrays,
  and other value families remain outside the Phase I claim.
- Realtime behavior is covered in Rust runtime fixtures; CLI, Python, and WASM
  continue to expose historical compile/analyze/run host surfaces.

## Phase J: Libraries, Imports, User Types, and Methods

Status: closed for the fixture-backed claimed subset. See
`docs/PHASE_J_AUDIT.md`.

Goal: support larger Pine programs after the core runtime model has matured.

Execution playbook: `docs/PHASE_J_EXECUTION_PLAN.md`.

Delivered scope:

- Host-neutral `AnalysisInput` and deterministic `SourceGraph`.
- CLI, Python, and WASM host-supplied library source maps.
- Exact-key `import ... as alias` for exported const expressions and pure
  exported functions.
- Local scalar-field user-defined types with `Type.new(...)`, field reads,
  ordinary variables, and `var` persistence.
- Pure methods on local UDT receivers with scalar parameters.

Maintenance tails:

- Remote registry lookup, version resolution, re-exports, wildcard/unaliased
  imports, imported UDT identity, imported methods, private exported UDTs, and
  advanced library visibility remain unsupported.
- UDT field mutation, UDT history references, `varip` UDTs, nested UDT fields,
  UDT arrays, recursive UDTs, method side effects, recursive methods, generic
  methods, and strategy-library interactions remain unsupported.

Risks:

- This touches semantic analysis, name resolution, caching, packaging, and
  security boundaries.

Acceptance criteria:

- Compile cache keys include all source dependencies.
- Diagnostics identify the originating file and span.
- Imported code follows the same side-effect and compatibility rules as local
  code.

## Phase K: Release and Compatibility Infrastructure

Goal: keep the growing subset maintainable.

Execution playbook: `docs/PHASE_K_EXECUTION_PLAN.md`.
Closure audit: `docs/PHASE_K_AUDIT.md`.

Scope:

- Result schema versioning for CLI, Python, and WASM.
- Fixture-backed conformance metadata gates.
- Golden JSON snapshots for public output shapes.
- Deterministic runtime profile gates for long histories and many callsites.
- A shared CI/local release verification entry point covering Rust, Python
  wheel, and WASM target checks.
- Compatibility matrix reporting by feature, status, fixture, and known gaps.

Acceptance criteria:

- New feature work cannot accidentally widen compatibility claims without
  conformance metadata.
- Public output changes are intentional and documented.
- Performance regressions are visible before release.

## Backlog Priority

Recommended order from the current state:

1. Phase G: strategy runtime.
2. Phase J maintenance only when a small, fixture-backed change widens the
   already claimed import, UDT, or method subsets.
3. Phase E/F/H/I maintenance only when a small, fixture-backed change widens an
   already claimed drawing, request, alert, or `varip` subset.
4. Phase K maintenance only when release contracts, snapshots, or matrix gates
   need tightening.
5. Phase B/C maintenance when new work exposes collection, history, or
   qualifier gaps.
6. Phase D maintenance for small fixture-backed built-in compatibility fixes.

This order keeps the project useful for indicator execution while delaying
features that require new host APIs, object lifetimes, or broker simulation.
