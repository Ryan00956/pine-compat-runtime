# Release Notes

## Unreleased

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
- Added `barstate.isfirst`, `barstate.isconfirmed`, `barstate.ishistory`,
  and `barstate.isrealtime`.
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
- Added dedicated conformance coverage for global OHLCV, derived price, time,
  and `bar_index` series.
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
- Added `math.floor` and `math.ceil` support for numeric values.
- Added `math.sqrt`, `math.log`, and `math.pow` support for numeric values.
- Added `math.sin`, `math.cos`, and `math.tan` support for numeric values.
- Added `math.log10` and `math.exp` support for numeric values.
- Added `math.acos`, `math.asin`, and `math.atan` support for numeric values.
- Added `math.sign`, `math.todegrees`, and `math.toradians` support for numeric values.
- Added `math.avg` support for variadic numeric averages.
- Added `math.e`, `math.pi`, `math.phi`, and `math.rphi` constants.
- Added `precision` argument support for `math.round`.
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

### Runtime Surfaces

- Rust crates for syntax, semantic analysis, HIR, built-ins, runtime, CLI,
  Python bindings, and WASM bindings.
- CLI commands for analysis, AST formatting, historical execution, profiling,
  and compatibility matrix output.
- Python binding exposing compile, analyze, and run entry points.
- WASM binding exposing compile, analyze, and CSV execution entry points.

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

### Partial Support

- `for`: supports inclusive integer ranges, loop control, and loop results, but
  does not claim full Pine loop compatibility.
- `history references`: supports constant non-negative offsets and guarded
  dynamic integer offsets, including `series int`, loop-produced offsets, and
  user-defined function parameters.
- `max_bars_back`: supports indicator-level constant non-negative retention
  bounds for dynamic history.
- `color.*` named constants: supports the current common registry only.
- `realtime forming rollback`: covers output, `var`, callsite, array, and
  dynamic history rollback; `varip` remains unsupported.

### Explicitly Unsupported

The analyzer rejects these boundaries with diagnostics instead of approximating
them silently:

- `varip` intrabar persistence.
- `request.*` multi-symbol and multi-timeframe data requests.
- `strategy.*` broker emulation and backtesting.
- Non-float arrays, matrices, maps, and unsupported collection operations.
- Imports and external libraries.
- Alerts and alert conditions.
- Drawing object systems such as labels, lines, boxes, tables, and polylines.
- Per-variable `max_bars_back` declarations and inference.
- Recursive user-defined functions.
- User-defined function side effects, including output calls, input
  declarations, indicator declarations, array mutation, global reassignment,
  and passing side-effecting calls as UDF arguments.

### Verification

The release baseline is expected to pass:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p pine-wasm --target wasm32-unknown-unknown
maturin build --manifest-path crates/pine-python/Cargo.toml --out dist
python -m pip install --force-reinstall dist/*.whl
python -m pytest python/tests
```
