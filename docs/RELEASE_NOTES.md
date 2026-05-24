# Release Notes

## Unreleased

- Closed Phase I with `docs/PHASE_I_AUDIT.md`, fixture-backed scalar and scalar
  typed-array `varip` conformance rows, host-surface review for CLI/Python/WASM
  historical paths, and explicit maintenance tails for drawing ids, tuples,
  maps, matrices, UDTs, imports, and unimplemented value families.
- Added the scalar and scalar typed-array `varip` executable subset: global and
  local int/float/bool/string/color/`na` declarations now behave like `var`
  during historical execution and preserve intrabar state across repeated
  realtime forming updates. Supported float/int/bool/string/color array ids also
  retain their backing contents across repeated forming updates, including
  branch-local declaration sites and `array.copy` boundaries. Array mutation
  inside UDFs remains rejected by existing function side-effect rules. Drawing
  ids are rejected with a dedicated diagnostic because object stores still roll
  back; tuples and other value families remain unsupported.
- Added the first `request.security` executable subset:
  `request.security(syminfo.tickerid, timeframe.period, expression)` returns the
  scalar side-effect-free expression in the current chart context.
- Added same-or-higher-timeframe host dataset injection for
  `request.security("SYMBOL", timeframe, expression)` lookups in Rust
  runtime, CLI `--request-bars SYMBOL:TIMEFRAME=bars.csv`, and Python
  `request_bars` dictionaries. WASM request dataset injection remains a
  documented temporary gap; optional parameters, explicit gaps/lookahead, and
  lower timeframe requests remain unsupported.
- Widened provider-backed `request.security` to evaluate scalar requested
  expressions in an isolated requested context with deterministic callsite
  caching and default higher-timeframe `gaps_off`/`lookahead_off` alignment.
  The supported provider expression subset now includes direct OHLCV/time
  sources, pure arithmetic and ternaries, history references, `na`, `nz`,
  `ta.sma`, and `ta.ema`; provider local aliases, side effects, and unsupported
  calls are rejected during semantic analysis where possible.
- Documented the lower-timeframe request boundary: lower-timeframe
  `request.security` remains runtime-rejected with a stable error, and
  `request.security_lower_tf` remains unsupported until typed array return
  semantics and host output shapes are designed.
- Added cross-host request contract fixtures for CLI and Python request dataset
  injection, plus conformance validation that prevents partial `request.*`
  claims without request-specific fixtures. WASM request dataset injection
  remains a documented temporary gap.
- Closed Phase F with `docs/PHASE_F_AUDIT.md`, fixture-backed request matrix
  rows, same-or-higher-timeframe request contract coverage across Rust, CLI,
  and Python, and a documented WASM provider-data gap.
- Started Phase E drawing-object infrastructure by bumping the public
  machine-readable contract to `schemaVersion: 2` and adding the
  top-level `labels` output across CLI JSON, Python dictionaries, and WASM JSON.
  It is empty for scripts that do not create supported drawing objects.
- Added the first drawing behavior: a `label.new` creation subset that returns
  deterministic label ids and emits sparse creation snapshots with bar-index
  coordinates, price y-values, text, colors, selected label styles, size, and
  tooltip metadata.
- Added sparse mutation snapshots for the initial `label.set_*` subset covering
  x/y/text/color/style/size/tooltip fields.
- Added `label.delete` lifecycle snapshots and a deterministic 500-label
  runtime limit. Label creation, mutation, and deletion now have fixture-backed
  realtime rollback, and drawing side effects inside user-defined functions are
  rejected under the existing side-effect policy. Unsupported coordinate modes
  and advanced label methods remain unsupported.
- Added the initial `line.*` lifecycle: deterministic line ids, sparse public
  `lines` snapshots for creation/mutation/deletion, selected endpoint/color/
  width/style/extend mutators, realtime rollback coverage, and a deterministic
  500-line runtime limit. Advanced line methods remain unsupported.
- Added the initial `box.*` lifecycle: deterministic box ids, sparse public
  `boxes` snapshots for creation/mutation/deletion, selected geometry/
  background/border mutators, realtime rollback coverage, and a deterministic
  500-box runtime limit. Advanced box methods remain unsupported.
- Added the initial `table.*` lifecycle: deterministic table ids, sparse public
  `tables` snapshots for fixed-dimension table creation and `table.cell`
  text/background/text-color writes, realtime rollback coverage, a
  deterministic 50-table runtime limit, and a 1000-cell per-table limit.
  Advanced table methods plus polyline drawing families remain unsupported.
- Kept `polyline.*` explicitly unsupported for Phase E because it depends on a
  future `chart.point` value and point-array design; the decision is captured in
  `docs/PHASE_E_POLYLINE_GATE.md`.
- Closed Phase E with `docs/PHASE_E_AUDIT.md`, fixture-backed drawing matrix
  rows, schemaVersion 2 drawing output coverage across CLI/Python/WASM, and
  family-split runtime drawing built-ins.
- Closed Phase K release infrastructure with public `schemaVersion: 1` output
  contracts for CLI, Python, and WASM public machine-readable outputs.
- Moved CLI and WASM runtime JSON onto shared runtime serialization helpers,
  with Python binding tests asserting the same public runtime key contract.
- Added golden JSON snapshots for representative CLI runtime output, CLI matrix
  JSON, and WASM analysis JSON.
- Hardened `tests/fixtures/conformance.tsv` validation so compatibility matrix
  claims require unique features, valid statuses, notes, existing fixtures, and
  status-appropriate fixture coverage.
- Added `scripts/verify.sh` as the canonical local and CI release verification
  entry point.
- Added deterministic runtime profile fixture gates for long TA histories,
  many stateful callsites, array-heavy scripts, and dynamic history retention.
- Added partial `switch` expression support for condition arms, selector/case
  arms, expression results, default arms, and conditional stateful-call
  execution.
- Added partial `while` statement support with bool conditions, `break`,
  `continue`, scoped loop bodies, and a runtime iteration guard.
- Added coverage for stateful callsite advancement inside `for` and `while`
  loop bodies.
- Added `ta.supertrend` line/direction tuple support for the fixture-covered
  ATR-based subset.
- Added `ta.dmi` `+DI`/`-DI`/`ADX` tuple support using the runtime's existing
  Wilder/RMA-style smoothing behavior.
- Added `ta.sar` Parabolic SAR support with callsite state and prior-bar
  high/low clamping.
- Added `ta.mfi` Money Flow Index support over ready positive/negative
  money-flow windows using source and volume.
- Added `ta.tsi` True Strength Index support using short/long EMA smoothing of
  source momentum and absolute momentum.
- Added `ta.cmo` Chande Momentum Oscillator support over ready rolling
  positive/negative source-change windows.
- Added `ta.cci` Commodity Channel Index support over ready source mean
  deviation windows.
- Added `ta.cog` Center of Gravity support over ready source windows.
- Added `ta.ao` Awesome Oscillator support as the fast/slow SMA spread of
  median price.
- Added `ta.bop` Balance of Power support over current OHLC values.
- Added `ta.kc` and `ta.kcw` Keltner Channel support using source/range EMA
  state.
- Added `ta.pivothigh` and `ta.pivotlow` support for confirmed left/right pivot
  windows.
- Added `ta.pivot_point_levels` support for runtime-bar anchored pivot level
  arrays across Traditional, Fibonacci, Woodie, Classic, DM, and Camarilla
  formulas.
- Added `ta.wpr` Williams %R support over ready rolling high/low windows and
  current close.
- Added `ta.stoch` four-argument stochastic oscillator support over ready
  rolling high/low windows.
- Added partial float array support with runtime-owned array ids,
  `array.new_float`, `array.push`, `array.get`, `array.set`, `array.size`,
  `array.pop`, and `array.clear`.
- Added partial int array support through `array.new_int` and the existing
  size/get/set/push/pop/clear operations.
- Added partial bool array support through `array.new_bool` and the existing
  size/get/set/push/pop/clear operations.
- Added partial string array support through `array.new_string` and the
  existing size/get/set/push/pop/clear operations.
- Added partial color array support through `array.new_color` and the existing
  size/get/set/push/pop/clear operations.
- Added `array.from` support for inferred float/int/bool/string/color typed
  arrays.
- Added array helper support for `array.first`, `array.last`, `array.shift`,
  and `array.unshift` across supported typed arrays.
- Added `array.insert` and `array.remove` support for supported typed arrays,
  including method-call syntax and array element limit checks.
- Added negative indexing support for `array.get`, `array.set`,
  `array.insert`, and `array.remove`.
- Added `array.fill` support for supported typed arrays, including optional
  half-open range bounds and method-call syntax.
- Added `array.copy` support for explicitly creating independent typed-array
  snapshots while plain array assignment remains id/reference-based.
- Added array search helpers `array.includes`, `array.indexof`,
  `array.lastindexof`, and numeric `array.binary_search*` variants.
- Added array truth helpers `array.every` and `array.some` for float, int, and
  bool arrays.
- Added numeric array statistics helpers `array.min`, `array.max`,
  `array.sum`, `array.avg`, `array.range`, `array.median`, `array.mode`,
  `array.percentile_nearest_rank`, `array.percentile_linear_interpolation`,
  `array.percentrank`, `array.covariance`, `array.standardize`,
  `array.variance`, and `array.stdev`, plus same-kind `array.abs`, for float
  and int arrays.
- Added array ordering helpers: `array.sort` for numeric/string arrays with
  optional order direction, `array.sort_indices` for numeric/string arrays with
  optional order direction, and `array.reverse` for all supported typed arrays.
- Added `array.join` support for supported typed arrays with optional string
  separators.
- Added `array.slice` and `array.concat` support for supported typed arrays,
  including method-call syntax and array element limit checks.
- Added partial array method-call syntax for supported array `size`, `get`,
  `set`, `insert`, `push`, `pop`, `remove`, `shift`, `unshift`, `first`,
  `last`, `fill`, `copy`, `slice`, `concat`, `includes`, `indexof`,
  `lastindexof`, `every`, `some`, numeric `binary_search*`, `min`, `max`,
  `sum`, `avg`, `range`, `median`, `mode`, `percentile_nearest_rank`,
  `percentile_linear_interpolation`, `percentrank`, `variance`, `stdev`,
  `sort`, `reverse`, `join`, and `clear`.
- Added `input.string` support for the executable `defval`/`title` subset.
- Added `input.price`, `input.time`, `input.symbol`, and `input.timeframe`
  support for the executable `defval`/`title` subset.
- Added generic `input` support for const int, float, bool, string, and color
  defaults.
- Added common `input.*` metadata parameters, including min/max/step,
  `options`, `tooltip`, `inline`, `group`, `confirm`, and `display` where they
  fit the supported input kinds.
- Added UTC-derived `year`, `month`, `dayofmonth`, `hour`, `minute`, and
  `second` bar time component variables.
- Added UTC-only function overloads for `year`, `month`, `dayofmonth`, `hour`,
  `minute`, and `second`.
- Added a numeric UTC subset of `timestamp`.
- Added `barstate.isfirst`, `barstate.islast`, `barstate.isnew`,
  `barstate.isconfirmed`, `barstate.ishistory`, and `barstate.isrealtime`.
- Added fixed-default regular-session `session.ismarket`,
  `session.ispremarket`, and `session.ispostmarket`.
- Added `bgcolor` and `barcolor` support with bar-aligned color output series.
- Added common `plot`, `hline`, `fill`, `bgcolor`, and `barcolor` metadata
  parameters for style/display compatibility; runtime output series remain
  unshifted by display metadata in this subset.
- Added basic `plotchar` support with bar-aligned values, chars, and colors.
- Expanded `plotchar` compatibility with common marker metadata parameters;
  runtime output remains normalized to value, char, and color series.
- Expanded `plotshape` and `plotarrow` compatibility with common marker
  metadata parameters while preserving the existing normalized output schemas.
- Expanded `plotbar` and `plotcandle` compatibility with common display
  metadata parameters while preserving existing OHLC output schemas.
- Added `color.from_gradient` with fixture-covered linear RGBA interpolation.
- Added conformance coverage for hex color literals as const colors.
- Added `input.session` and `input.text_area` defval execution with metadata
  validation.
- Added direct `display.pane`, `display.price_scale`, `display.status_line`,
  and `display.data_window` metadata constants.
- Accepted `confirm` metadata on `input.source`.
- Added explicit `str.tostring` handling for `format.price` and
  `format.volume`.
- Added dedicated conformance coverage for global OHLCV, derived price
  (`hl2`, `hlc3`, `hlcc4`, `ohlc4`), time, and `bar_index` series.
- Added basic `plotshape` support with bar-aligned values, style, location,
  color, text, text color, and size marker output.
- Added basic `plotarrow` support with bar-aligned numeric values, up/down
  colors, and height bounds.
- Added basic `plotbar` support with bar-aligned OHLC values and optional
  colors.
- Added basic `plotcandle` support with bar-aligned OHLC values plus body,
  wick, and border colors.
- Added `color.rgb` support for numeric RGB channels and optional transparency.
- Added optional transparency defaulting for `color.new`.
- Added `color.r`, `color.g`, `color.b`, and `color.t` channel extraction.
- Added `str.length`, `str.upper`, and `str.lower` string helpers.
- Added `str.contains`, `str.startswith`, and `str.endswith` string predicates.
- Added `str.pos` and `str.substring` string extraction helpers.
- Added `str.trim` and `str.repeat` string modification helpers.
- Added `str.replace` and `str.replace_all` string replacement helpers.
- Added `str.tonumber` numeric string parsing.
- Added `str.tostring` scalar and float-array string conversion.
- Added `str.format` indexed placeholder string formatting.
- Added `str.match` regex substring matching.
- Added `str.split` support for literal separators and empty-separator
  character splitting.
- Added a UTC subset of `str.format_time` timestamp formatting.
- Added UTC `weekofyear` and `dayofweek` calendar variables/functions plus
  `dayofweek.*` constants.
- Added fixed-default `time_close` using the current 1-minute chart timeframe
  subset.
- Added fixed-default `timeframe.period` and `timeframe.in_seconds` support for
  common seconds/minutes/days/weeks/months timeframe strings, plus
  `timeframe.from_seconds` for the exact reverse conversion subset.
- Added `timeframe.change` UTC bucket detection for the supported timeframe
  string subset.
- Added fixed-default `timeframe.is*` and `timeframe.multiplier` chart
  timeframe metadata.
- Added `int`, `float`, `bool`, `string`, and `color` scalar type casts for
  numeric, bool, string, color, and `na` values.
- Added `fixnan` support for carrying forward the last non-`na` numeric or
  color value at each callsite.
- Added `math.floor` and `math.ceil` support for numeric values.
- Added `math.sqrt`, `math.log`, and `math.pow` support for numeric values.
- Added `math.trunc`, `math.cbrt`, and `math.hypot` support for numeric
  values.
- Added `math.sin`, `math.cos`, and `math.tan` support for numeric values.
- Added `math.log10` and `math.exp` support for numeric values.
- Added `math.acos`, `math.asin`, and `math.atan` support for numeric values.
- Added `math.sign`, `math.todegrees`, and `math.toradians` support for numeric values.
- Added `math.avg` support for variadic numeric averages.
- Added `math.e`, `math.pi`, `math.phi`, and `math.rphi` constants.
- Added `precision` argument support for `math.round`.
- Added `math.round_to_mintick` support using the current default
  `syminfo.mintick` subset value.
- Added fixed-default `syminfo.*` metadata for common ticker, exchange,
  currency, session, timezone, tick-size, and price-scale fields.
- Added deterministic callsite-backed `math.random` support with optional
  `min`, `max`, and `seed` arguments.
- Added `math.sum` support for rolling source sums with simple-int lengths.
- Added `ta.stdev` support with default biased and optional sample standard
  deviation modes.
- Added `ta.variance` support with the same biased/sample window modes as
  `ta.stdev`.
- Added `ta.range` support for rolling highest-minus-lowest values.
- Added `ta.dev` support for rolling average absolute deviation.
- Added `ta.vwma` support for rolling volume-weighted moving averages.
- Added `ta.wma` support for linearly weighted moving averages.
- Added `ta.hma` support for Hull moving averages composed from internal WMA
  windows.
- Added `ta.swma` support for fixed four-bar symmetric weighted moving
  averages.
- Added `ta.alma` support for Arnaud Legoux moving averages with optional
  floored offset centers.
- Added `ta.dema` and `ta.tema` support for double and triple EMA-chain
  smoothing.
- Added `ta.linreg` support for rolling least-squares linear regression values.
- Added `ta.bbw` support for Bollinger Bands Width values.
- Added `ta.cum` support for cumulative numeric source sums.
- Added `ta.max` and `ta.min` support for all-time source extremes.
- Added `ta.tr` support as a built-in true range series variable.
- Added `ta.accdist` support as the built-in Accumulation/Distribution index
  series variable.
- Added `ta.iii` support as the built-in Intraday Intensity Index series
  variable.
- Added `ta.nvi` and `ta.pvi` support as built-in Negative/Positive Volume
  Index series variables.
- Added `ta.obv` support as the built-in On Balance Volume series variable.
- Added `ta.pvt` support as the built-in Price Volume Trend series variable.
- Added partial `ta.vwap` support for the variable form, one-argument source
  call, source/anchor call, and source/anchor/bands tuple call as runtime-bar
  cumulative VWAP; session-derived anchoring remains future work.
- Added `ta.wad` support as the built-in Williams Accumulation/Distribution
  series variable.
- Added `ta.wvad` support as the built-in Williams Variable
  Accumulation/Distribution series variable.
- Added `ta.mom` support for source momentum over explicit history lengths.
- Added `ta.roc` support for rate-of-change percentages over explicit history
  lengths.
- Expanded `ta.change` to support series int and bool sources.
- Expanded core TA source signatures to accept series int sources where the
  runtime already evaluates numeric windows through floating-point values.
- Added explicit fixture coverage for simple numeric sources in rolling
  correlation, covariance, median, mode, percentile, and percent-rank helpers.
- Added `ta.correlation` support for rolling Pearson correlation coefficients.
- Added `ta.covariance` support for rolling population covariance values.
- Added `ta.median` and `ta.mode` support for rolling sorted-window statistics.
- Added `ta.percentile_nearest_rank` support for rolling nearest-rank
  percentile values.
- Added `ta.percentile_linear_interpolation` support for rolling interpolated
  percentile values.
- Added `ta.percentrank` support for rolling percent-rank values.
- Added `ta.rising` and `ta.falling` support for current-vs-previous-window
  trend checks.
- Added two-argument `ta.highestbars` and `ta.lowestbars` support for rolling
  extreme offsets.
- Added `ta.barssince` support for tracking bars elapsed since the last true
  condition.
- Added `ta.valuewhen` support for retrieving source values from prior true
  condition occurrences.
- Added length-only overloads for `ta.highest`, `ta.lowest`,
  `ta.highestbars`, and `ta.lowestbars`.
- Tightened float array UDF boundaries: read-only array operations are allowed,
  while array mutation inside UDFs is rejected as a side effect.
- Added a 100,000-element runtime guard for each float array.

## v0.1 Baseline

This release establishes the first executable Pine-compatible indicator subset.
Compatibility claims are backed by `tests/fixtures/conformance.tsv`; run
`pine-compat matrix` or `pine-compat matrix --format json` to inspect the
feature-level matrix and its fixture paths.

Machine-readable public outputs use top-level `schemaVersion`. Runtime,
analysis, and matrix outputs now have separate schema constants:
`PUBLIC_RUNTIME_SCHEMA_VERSION`, `PUBLIC_ANALYSIS_SCHEMA_VERSION`, and
`PUBLIC_MATRIX_SCHEMA_VERSION`. Runtime output is now `schemaVersion: 3` with a
reserved top-level `alerts` array. Analysis and matrix outputs remain
`schemaVersion: 2`; increment only the affected contract when an intentional
consumer-visible output change is documented with snapshot updates.

### Runtime Surfaces

- Rust crates for syntax, semantic analysis, HIR, built-ins, runtime, CLI,
  Python bindings, and WASM bindings.
- CLI commands for analysis, AST formatting, historical execution, profiling,
  and compatibility matrix output.
- Python binding exposing compile, analyze, and run entry points.
- WASM binding exposing compile, analyze, and CSV execution entry points.
- Public JSON/dictionary outputs for CLI, Python, and WASM expose
  `schemaVersion`; CLI and WASM runtime JSON share the same runtime contract
  helper.
- Runtime outputs include an `alerts` array for Phase H alert events.
  `alertcondition(condition, title, message)` is partially supported for
  bool-compatible conditions and const-string title/message, and
  `alert(message)` is partially supported for const-string messages. Reached
  true conditions and reached alert calls emit deterministic `{id, barIndex,
  time, message, source}` events in program order; forming realtime events roll
  back until confirmed. Alert frequency modes remain unsupported until
  deterministic frequency semantics are designed.
- The compatibility matrix source of truth is
  `tests/fixtures/conformance.tsv`; generated text and JSON matrix output must
  remain fixture-backed.

### Supported Executable Subset

- Indicator scripts over OHLCV bar input.
- Historical bar-by-bar execution and incremental append execution.
- Realtime forming-bar rollback for output, `var`, and stateful callsite state.
- Constant non-negative history references.
- Normal declarations, reassignment, tuple declarations, tuple-returning
  built-ins, and tuple `for` expression results.
- `if`/`else` blocks, nested blocks, and conditional stateful calls that advance
  only when their branch executes.
- `for` loops over inclusive integer ranges, explicit non-zero `by` steps,
  `break`, `continue`, scalar loop results, and tuple loop results.
- Local scopes for block declarations, tuple declarations, shadowing, and local
  `var` declaration-site storage.
- User-defined functions with expression bodies and multi-statement block
  bodies, positional and named arguments, single evaluation of arguments,
  function-local declarations, local `var`, loops inside functions, and
  independent state per syntactic callsite.
- `na`, `nz`, `indicator`, `input.*`, `plot`, `hline`, `fill`, `color.new`,
  selected named colors, selected `math.*` functions, and the fixture-covered
  `ta.*` built-ins listed in the compatibility matrix.
- Typed scalar arrays for float, int, bool, string, and color through the
  fixture-covered `array.*` subset documented as partial in the matrix.

### Partial Support

- `for`: supports inclusive integer ranges, loop control, and loop results, but
  does not claim full Pine loop compatibility.
- `history references`: supports constant non-negative offsets and guarded
  dynamic integer offsets, including `series int`, loop-produced offsets, and
  user-defined function parameters.
- `max_bars_back`: supports indicator-level constant non-negative retention
  bounds for dynamic history.
- `color.*` named constants: supports the current common registry only.
- `realtime forming rollback`: covers output, alert events,
  supported drawing objects, `var`, scalar and scalar typed-array `varip`,
  callsite, array, and dynamic history rollback.

### Explicitly Unsupported

The analyzer rejects these boundaries with diagnostics instead of approximating
them silently:

- `varip` drawing ids, tuple `varip`, and `varip` value families outside the
  scalar and scalar typed-array subset.
- `request.*` multi-symbol and multi-timeframe data requests.
- `strategy.*` broker emulation and backtesting.
- Generic arrays, object arrays, user-defined type arrays, matrices, maps, and
  deferred collection semantics that are not fixture-backed in the current
  `array.*` partial subset.
- Imports and external libraries.
- Alert frequency controls.
- Advanced drawing object methods and unsupported `polyline.*` point-list
  object systems.
- Per-variable `max_bars_back` declarations and inference.
- Recursive user-defined functions.
- User-defined function side effects, including output calls, alerts,
  input declarations, indicator declarations, array mutation, global
  reassignment, and passing side-effecting calls as UDF arguments.

### Verification

The release baseline is expected to pass:

```text
scripts/verify.sh
```

Snapshot updates are intentional public-contract changes. Refresh them with the
commands in `docs/CONFORMANCE.md`, review the JSON diff, then run
`scripts/verify.sh`.
