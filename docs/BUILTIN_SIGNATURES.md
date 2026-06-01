# Built-In Signatures

This document defines the first built-in surface as typed signatures. It is a
contract for semantic analysis and runtime implementation.

The syntax below is descriptive, not final Rust API syntax:

```text
name(arg_name: qualifier kind, ...) -> qualifier kind
```

`series<T>` means a series-qualified value of kind `T`.

## Phase C Qualifier Audit Notes

These signatures describe the currently implemented semantic surface, not the
full Pine surface. In this document:

- `series/simple numeric` means numeric values at any implemented qualifier up
  to `series`, including `const` and `input`.
- `simple int` means `const`, `input`, or `simple` integers, and rejects
  `series int`.
- `const` parameters require literal/named-constant style values after current
  semantic analysis.
- History offsets accept non-negative integer literals plus integer expressions
  at any implemented qualifier, including `series int`; non-integer offsets are
  rejected.

## Phase 1 Core

Phase 1 should be intentionally small:

- OHLCV and derived built-in series
- `indicator`
- `input`, `input.int`, `input.float`, `input.bool`, `input.source`,
  `input.color`, `input.string`, `input.price`, `input.time`, `input.symbol`,
  `input.timeframe`, `input.session`, `input.text_area`
- `plot`
- `hline`
- `fill`
- `na`
- `nz`
- `ta.sma`
- `ta.ema`

Other built-ins may be parsed and reported as unsupported until this set is
stable.

## Global Series

```text
open      -> series float
high      -> series float
low       -> series float
close     -> series float
volume    -> series float
time      -> series int
time_close -> series int
year      -> series int
month     -> series int
weekofyear -> series int
dayofmonth -> series int
dayofweek -> series int
hour      -> series int
minute    -> series int
second    -> series int
hl2       -> series float
hlc3      -> series float
hlcc4     -> series float
ohlc4     -> series float
bar_index -> series int
timeframe.period -> simple string
timeframe.isseconds -> simple bool
timeframe.isminutes -> simple bool
timeframe.isintraday -> simple bool
timeframe.isdaily -> simple bool
timeframe.isweekly -> simple bool
timeframe.ismonthly -> simple bool
timeframe.isdwm -> simple bool
timeframe.multiplier -> simple int
```

`year`, `month`, `weekofyear`, `dayofmonth`, `dayofweek`, `hour`, `minute`,
and `second` currently expose UTC calendar components derived from each bar's
`time`. Full exchange-timezone calendar semantics are not claimed until symbol
timezone metadata exists. `dayofweek.sunday` through `dayofweek.saturday`
evaluate to const ints `1` through `7`; `weekofyear` uses the UTC ISO week
number in the current subset. `time_close` uses the fixed default 1-minute
chart timeframe and returns `time + 60000`.

Bar state:

```text
barstate.isfirst -> series bool
barstate.islast -> series bool
barstate.isnew -> series bool
barstate.isconfirmed -> series bool
barstate.ishistory -> series bool
barstate.isrealtime -> series bool
```

`barstate.isfirst` is `true` only when `bar_index == 0`.
`barstate.islast` is `true` on the last known bar in finite historical batch
execution and on current realtime updates. Open-ended `append_bar` historical
updates treat the appended bar as the latest known bar.
`barstate.isnew` is `true` for historical bars and for the first realtime
update of a new bar. Subsequent forming updates for the same realtime bar and
the confirming update after a forming update return `false`.
`barstate.isconfirmed` is `true` for historical and confirmed updates, and
`false` for forming realtime updates.
`barstate.ishistory` is `true` for historical updates. `barstate.isrealtime`
is `true` for forming and confirmed realtime updates.

Session state:

```text
session.ismarket -> series bool
session.ispremarket -> series bool
session.ispostmarket -> series bool
```

The current subset assumes every runtime bar is in the regular session:
`session.ismarket` is `true`, while `session.ispremarket` and
`session.ispostmarket` are `false`.

Symbol info:

```text
syminfo.tickerid -> const string
syminfo.ticker -> const string
syminfo.prefix -> const string
syminfo.description -> const string
syminfo.type -> const string
syminfo.currency -> const string
syminfo.basecurrency -> const string
syminfo.session -> const string
syminfo.timezone -> const string
syminfo.root -> const string
syminfo.volumetype -> const string
syminfo.mintick -> const float
syminfo.pointvalue -> const float
syminfo.minmove -> const int
syminfo.pricescale -> const int
```

`syminfo.*` currently uses fixed default symbol metadata until runtime symbol
metadata is available: `NASDAQ:AAPL`, ticker `AAPL`, prefix `NASDAQ`, stock
type, `USD` currency/base currency, `regular` session, `Etc/UTC` timezone,
`base` volume type, `mintick = 0.01`, `pointvalue = 1.0`, `minmove = 1`, and
`pricescale = 100`.

The same names are also supported as functions over a timestamp:

```text
year(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
month(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
weekofyear(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
dayofmonth(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
dayofweek(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
hour(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
minute(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
second(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
timestamp(year: int-compatible, month: int-compatible, day: int-compatible, hour?: int-compatible, minute?: int-compatible, second?: int-compatible)
  -> int with strongest qualifier
```

For now, these function overloads use the same UTC-only timezone subset as
`str.format_time`; unsupported time zones are runtime errors. `timestamp`
currently supports only the numeric UTC subset; omitted hour/minute/second
default to 0, `na` inputs return `na`, and invalid UTC dates are runtime
errors.

Timeframe helpers:

```text
timeframe.in_seconds(timeframe?: simple string) -> simple int
timeframe.from_seconds(seconds: simple int) -> simple string
timeframe.change(timeframe: simple string) -> series bool
```

The current subset assumes a fixed default chart timeframe of `1` minute, so
`timeframe.period` returns `"1"`, `timeframe.multiplier` returns `1`,
`timeframe.isminutes` and `timeframe.isintraday` return `true`, and
`timeframe.isseconds`, `timeframe.isdaily`, `timeframe.isweekly`,
`timeframe.ismonthly`, and `timeframe.isdwm` return `false`.

Request helpers:

```text
request.security(symbol: simple string, timeframe: simple string, expression: any)
  -> series type matching expression
```

The current executable subset has two forms:

- `request.security(syminfo.tickerid, timeframe.period, expression)` evaluates a
  scalar side-effect-free expression in the chart context.
- `request.security("SYMBOL", timeframe, expression)` and
  `request.security(syminfo.tickerid, timeframe, expression)` evaluate scalar
  side-effect-free expressions over host-provided same-or-higher-timeframe bars.
  The supported provider expression subset includes direct OHLCV/time sources,
  pure arithmetic and ternaries, history references, `na`, `nz`, `ta.sma`, and
  `ta.ema`. Higher-timeframe alignment uses default `gaps_off` and
  `lookahead_off`: only confirmed requested bars are visible, and missing
  requested bars forward-fill the last confirmed value.

Lower timeframe requests, provider expression local variable aliases, UDF calls,
output/drawing side effects, input declarations, array mutation, optional
parameters, non-default barmerge behavior, explicit gaps, and lookahead remain
unsupported.
`request.security_lower_tf` is unsupported; it returns arrays in Pine and is not
claimed until typed array return semantics and host output shapes are designed.
`timeframe.in_seconds()` returns `60`.
Explicit timeframe strings support Pine-style seconds (`1S`, `5S`, `10S`,
`15S`, `30S`, `45S`), minutes (`1` through `1440`), days (`D`/`1D` through
`365D`), weeks (`W`/`1W` through `52W`), and months (`M`/`1M` through `12M`,
using 30-day month seconds). Tick and invalid timeframe strings are runtime
errors in this subset. `timeframe.from_seconds` supports the exact reverse
conversion for values representable in that subset, preferring canonical
strings such as `"1"`, `"D"`, `"W"`, and `"M"` over equivalent longer forms.
Non-positive or otherwise unrepresentable second counts are runtime errors.
`timeframe.change` uses the same supported timeframe string subset and returns
`true` on the first executed bar or when the UTC timeframe bucket changes from
the previous committed bar.

Type casts:

```text
int(x: int|float|bool|na) -> int with same qualifier
float(x: int|float|bool|na) -> float with same qualifier
bool(x: int|float|bool|na) -> bool with same qualifier
string(x: int|float|bool|string|na) -> string with same qualifier
color(x: color|na) -> color with same qualifier
```

`int` truncates finite floats toward zero and maps bools to `1`/`0`.
`float` maps ints and bools to numeric floats. `bool` maps zero and `na` to
`false`, and nonzero numeric values to `true`. `int(na)` and `float(na)`
return `na`. `string` maps scalar values using the default numeric text format
and returns `na` for `string(na)`. `color` preserves color values and returns
`na` for `color(na)`. Numeric-to-color and object casts are not part of the
current subset.

Derived values:

```text
hl2   = (high + low) / 2
hlc3  = (high + low + close) / 3
hlcc4 = (high + low + close + close) / 4
ohlc4 = (open + high + low + close) / 4
```

## Declarations

```text
indicator(title: const string, shorttitle?: const string, overlay?: const bool, max_bars_back?: const int, ...)
  -> void
strategy(title: const string, shorttitle?: const string, overlay?: const bool, max_bars_back?: const int, initial_capital?: const numeric, default_qty_type?: const string, default_qty_value?: const numeric)
  -> void
strategy.entry(id: simple string, direction: string-compatible, qty?: series/simple numeric)
  -> void
strategy.close(id: simple string) -> void
strategy.exit(id: simple string, from_entry: simple string, stop?: series/simple numeric, limit?: series/simple numeric, profit?: series/simple numeric, loss?: series/simple numeric, trail_price?: series/simple numeric, trail_points?: series/simple numeric, trail_offset?: series/simple numeric, qty?: series/simple numeric, qty_percent?: series/simple numeric)
  -> void
```

Only metadata arguments needed by the output and history-retention model should
be accepted in Phase 1. `max_bars_back` must be non-negative when provided.
Unsupported named arguments should produce compatibility diagnostics.
`strategy(...)` defaults `default_qty_type` to `strategy.fixed` and
`default_qty_value` to `1`, so `strategy.entry(..., qty=...)` may omit `qty` and
use the configured or default fixed quantity.
`strategy.exit` accepts `qty` or `qty_percent` on supported single-trigger,
one-downside/one-upside bracket, and trailing trigger shapes. `qty` and
`qty_percent` remain mutually exclusive. Richer strategy order options remain
unsupported.

## Inputs

```text
input(defval: const int/float/bool/string/color, title?: const string, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input defval kind
input.int(defval: const int, title?: const string, minval?: const int, maxval?: const int, step?: const int, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input int
input.float(defval: const float, title?: const string, minval?: const numeric, maxval?: const numeric, step?: const numeric, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input float
input.bool(defval: const bool, title?: const string, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input bool
input.color(defval: const color, title?: const string, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input color
input.string(defval: const string, title?: const string, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input string
input.price(defval: const float, title?: const string, minval?: const numeric, maxval?: const numeric, step?: const numeric, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input float
input.time(defval: const int, title?: const string, minval?: const int, maxval?: const int, step?: const int, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input int
input.symbol(defval: const string, title?: const string, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input string
input.timeframe(defval: const string, title?: const string, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input string
input.session(defval: const string, title?: const string, options?: tuple, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input string
input.text_area(defval: const string, title?: const string, tooltip?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> input string
input.source(defval: series float, title?: const string, tooltip?: const string, inline?: const string, group?: const string, confirm?: const bool, display?: string-compatible) -> series float
```

Rules:

- Input metadata is accepted and validated during analysis.
- Runtime execution currently uses each input's `defval`; host-provided input
  override APIs are not implemented yet.
- The supported metadata subset validates common option names and types, then
  ignores metadata at runtime; `defval` remains the executable value until
  host-side input override APIs are implemented.
- `input.session` and `input.text_area` currently execute their `defval`
  strings and accept metadata parameters without host-side override behavior.
- `input.source` returns the selected source series. Phase 1 may restrict this
  to known OHLCV-derived series.

## Plotting

```text
alertcondition(condition: bool-compatible, title: const string, message: const string)
  -> void

alert(message: const string)
  -> void

plot(series: series/simple numeric, title?: const string, color?: color-compatible, linewidth?: simple int, style?: const string, trackprice?: const bool, histbase?: numeric, offset?: simple int, join?: const bool, editable?: const bool, show_last?: simple int, display?: const string, format?: const string, precision?: simple int, force_overlay?: const bool)
  -> plot

plotchar(series: series/simple numeric-or-bool, title?: const string, char?: const string, color?: color-compatible, location?: const string, offset?: simple int, text?: const string, textcolor?: color-compatible, editable?: const bool, size?: const string, show_last?: simple int, display?: const string)
  -> void

plotshape(series: series/simple numeric-or-bool, title?: const string, style?: const string, location?: const string, color?: color-compatible, offset?: simple int, text?: const string, textcolor?: color-compatible, editable?: const bool, size?: const string, show_last?: simple int, display?: const string, force_overlay?: const bool)
  -> void

plotarrow(series: series/simple numeric, title?: const string, colorup?: color-compatible, colordown?: color-compatible, offset?: simple int, minheight?: simple int, maxheight?: simple int, editable?: const bool, show_last?: simple int, display?: const string, force_overlay?: const bool)
  -> void

plotbar(open: series/simple numeric, high: series/simple numeric, low: series/simple numeric, close: series/simple numeric, title?: const string, color?: color-compatible, editable?: const bool, show_last?: simple int, display?: const string)
  -> void

plotcandle(open: series/simple numeric, high: series/simple numeric, low: series/simple numeric, close: series/simple numeric, title?: const string, color?: color-compatible, wickcolor?: color-compatible, editable?: const bool, show_last?: simple int, bordercolor?: color-compatible, display?: const string)
  -> void

hline(price: const-or-input float, title?: const string, color?: color-compatible, linestyle?: const string, linewidth?: simple int, editable?: const bool, display?: const string)
  -> hline

fill(plot1: plot-or-hline, plot2: plot-or-hline, color?: color-compatible, title?: const string, editable?: const bool, show_last?: simple int, fillgaps?: const bool, display?: const string)
  -> void

bgcolor(color: color-compatible, title?: const string, offset?: simple int, editable?: const bool, show_last?: simple int, display?: const string) -> void
barcolor(color: color-compatible, title?: const string, offset?: simple int, editable?: const bool, show_last?: simple int, display?: const string) -> void
```

`alertcondition` emits a runtime alert event when its reached condition
evaluates to `true`. `title` is serialized as event `source`; `message` is
serialized as event `message`. `alert` emits whenever execution reaches the
call and serializes `source` as `alert`. Dynamic message/title strings,
TradingView-style `{{...}}` placeholder interpolation, optional alert frequency
parameters, and alert side effects inside UDF or requested-context expressions
are not part of the current subset.

`color-compatible` should initially accept:

- const color
- input color
- series color where the target built-in supports dynamic color
- `na` to mean no color for that bar when supported

The output collector retains plot ids, hline ids, and bar-aligned output series
so host integrations can adapt the normalized result without reinterpreting the
script. The supported output metadata subset accepts common style, visibility,
display, and editability parameters for compatibility, but accepted metadata is
not the same as emitted output schema.

Current normalized output fields are:

- `plot`: id and values only. Color, line style, width, trackprice, histbase,
  join, format, precision, and editability metadata are accepted but not
  emitted as fields.
- `hline`: id and price only. Color, line style, width, editability, and
  display metadata are accepted but not emitted as fields.
- `fill`: id plus first/second plot or hline ids only. Color, title,
  editability, show_last, fillgaps, and display metadata are accepted but not
  emitted as fields.
- `bgcolor`/`barcolor`: id and color values only.
- `plotchar`: value, char, and color series only. Location, text, textcolor,
  size, visibility, and editability metadata are accepted but not emitted as
  fields.
- `plotshape`: value, style, location, color, text, textcolor, and size
  series.
- `plotarrow`: value, up-color, down-color, min-height, and max-height series.
- `plotbar`: open, high, low, close, and color series.
- `plotcandle`: open, high, low, close, body color, wick color, and border
  color series.

Parameters such as `offset`, `show_last`, `display`, `force_overlay`, and
`editable` do not yet transform, filter, or annotate the runtime output series.
Supported direct display constants include `display.all`, `display.none`,
`display.pane`, `display.price_scale`, `display.status_line`, and
`display.data_window`. Display flag arithmetic is not implemented yet.

## Utility

```text
na(x: any) -> simple bool or series bool
nz(x: numeric-or-color-series) -> same kind and qualifier as x
nz(x: T, replacement: T) -> strongest qualifier of x and replacement, kind T
fixnan(source: series/simple int|float|color) -> same kind and qualifier as source
```

`na(x)` returns a series-qualified bool when `x` is series-qualified.

`nz` overloads must be explicit. Do not implement `nz` with a generic host
language null helper.
`fixnan` returns the current non-`na` source value and otherwise returns the
last non-`na` value observed at the same callsite. It returns `na` until the
callsite has observed a non-`na` value.

## Arrays

```text
array.new_float(size?: simple int, initial_value?: numeric) -> simple float-array
array.new_int(size?: simple int, initial_value?: int-compatible) -> simple int-array
array.new_bool(size?: simple int, initial_value?: bool-compatible) -> simple bool-array
array.new_string(size?: simple int, initial_value?: string-compatible) -> simple string-array
array.new_color(size?: simple int, initial_value?: color-compatible) -> simple color-array
array.from(value, ...) -> simple inferred-array
array.size(id: float-array|int-array|bool-array|string-array|color-array) -> simple int
array.push(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> void
array.get(id: float-array|int-array|bool-array|string-array|color-array, index: simple int) -> series element
array.set(id: float-array|int-array|bool-array|string-array|color-array, index: simple int, value: element-compatible) -> void
array.insert(id: float-array|int-array|bool-array|string-array|color-array, index: simple int, value: element-compatible) -> void
array.pop(id: float-array|int-array|bool-array|string-array|color-array) -> series element
array.remove(id: float-array|int-array|bool-array|string-array|color-array, index: simple int) -> series element
array.shift(id: float-array|int-array|bool-array|string-array|color-array) -> series element
array.unshift(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> void
array.fill(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible, index_from?: simple int, index_to?: simple int) -> void
array.first(id: float-array|int-array|bool-array|string-array|color-array) -> series element
array.last(id: float-array|int-array|bool-array|string-array|color-array) -> series element
array.copy(id: float-array|int-array|bool-array|string-array|color-array) -> same array kind
array.slice(id: float-array|int-array|bool-array|string-array|color-array, index_from: simple int, index_to: simple int) -> same array kind
array.concat(id: float-array|int-array|bool-array|string-array|color-array, id2: same array kind) -> same array kind
array.includes(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> series bool
array.every(id: float-array|int-array|bool-array) -> series bool
array.some(id: float-array|int-array|bool-array) -> series bool
array.indexof(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> simple int
array.lastindexof(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> simple int
array.binary_search(id: float-array|int-array, value: element-compatible) -> simple int
array.binary_search_leftmost(id: float-array|int-array, value: element-compatible) -> simple int
array.binary_search_rightmost(id: float-array|int-array, value: element-compatible) -> simple int
array.abs(id: float-array|int-array) -> same array kind
array.min(id: float-array|int-array) -> series element
array.max(id: float-array|int-array) -> series element
array.sum(id: float-array|int-array) -> series element
array.avg(id: float-array|int-array) -> series float
array.range(id: float-array|int-array) -> series element
array.median(id: float-array|int-array) -> series element
array.mode(id: float-array|int-array) -> series element
array.percentile_nearest_rank(id: float-array|int-array, percentage: numeric-compatible) -> series element
array.percentile_linear_interpolation(id: float-array|int-array, percentage: numeric-compatible) -> series float
array.percentrank(id: float-array|int-array, index: simple int) -> series float
array.covariance(id1: float-array|int-array, id2: float-array|int-array, biased?: bool-compatible) -> series float
array.standardize(id: float-array|int-array) -> float-array
array.variance(id: float-array|int-array, biased?: bool-compatible) -> series float
array.stdev(id: float-array|int-array, biased?: bool-compatible) -> series float
array.sort(id: float-array|int-array|string-array, order?: const string) -> void
array.sort_indices(id: float-array|int-array|string-array, order?: const string) -> int-array
array.reverse(id: float-array|int-array|bool-array|string-array|color-array) -> void
array.join(id: float-array|int-array|bool-array|string-array|color-array, separator?: string-compatible) -> series string
array.clear(id: float-array|int-array|bool-array|string-array|color-array) -> void
```

The supported typed-array subset covers float, int, bool, string, and color
arrays. Float arrays accept int or float values and store them as floats. Int
arrays accept int values. Bool arrays accept bool values. String arrays accept
string values. Color arrays accept color values. `array.from` infers the array
kind from its arguments, requires at least one non-`na` supported typed value,
allows `na` in otherwise typed arrays, and promotes mixed int/float arguments
to a float array.
`size/get/set/insert/push/pop/remove/shift/unshift/fill/first/last/copy/slice/concat/includes/indexof/lastindexof/clear`
may also be called with method syntax on a supported array receiver.
`array.get`, `array.set`, `array.insert`, and `array.remove` support negative
indexes from the array end. `array.insert` inserts a compatible value before
the requested index; greater-than-size or otherwise invalid indexes are no-ops.
`array.remove` removes and returns an element, or returns `na` for an invalid
index. `array.fill` fills the whole array by default or the half-open
`[index_from, index_to)` window when bounds are supplied; invalid ranges are
no-ops. `array.slice` allocates a same-kind array containing the half-open
`[index_from, index_to)` window; invalid bounds return `na` at runtime.
`array.concat` requires two arrays of the same kind,
appends `id2` values to `id` in place, and returns `id`. Numeric array
`binary_search/binary_search_leftmost/binary_search_rightmost/abs/min/max/sum/avg/range/median/mode/percentile_nearest_rank/percentile_linear_interpolation/percentrank/covariance/standardize/variance/stdev`
helpers may also be called with method syntax on float and int array receivers.
`every/some` may also be called with method syntax on float, int, and bool
array receivers.
`sort/sort_indices` may also be called with method syntax on float, int, and
string array receivers.
Binary search helpers expect the current array contents to be sorted ascending;
`array.binary_search` returns `-1` when not found, while leftmost/rightmost
return the nearest existing insertion-side index and return `-1` for empty
arrays. `array.every` and `array.some` are limited to float, int, and bool
arrays; false, zero, and `na` elements are falsey, other numeric values are
truthy, empty arrays return `true` for `every` and `false` for `some`.
`array.abs` allocates a new same-kind array containing the absolute
value of each source element, preserves `na`, and does not mutate the source.
`array.range` returns max minus min while ignoring `na` elements.
`array.median` returns the median of non-`na` values. `array.mode` returns the
smallest value among tied most-frequent values and returns `na` when all
remaining values occur only once.
Percentile helpers operate on non-`na` values sorted ascending. Percentages
outside `0..=100`, empty/all-`na` arrays, and invalid percentrank indexes
return `na`.
`array.covariance` requires two same-size numeric arrays, skips pairs where
either side is `na`, and uses a biased population estimate by default; pass
`false` for an unbiased sample estimate.
`array.standardize` allocates a new float array, uses non-`na` values to
calculate mean and population standard deviation, preserves `na` element
positions when at least one numeric value is present, and returns an empty
array for empty/all-`na` arrays.
`array.variance` and `array.stdev` ignore `na` elements and use a biased
population estimate by default; pass `false` for an unbiased sample estimate.
`array.sort` and `array.sort_indices` support float, int, and string arrays,
sort ascending by default, and accept `order.ascending` or `order.descending`.
`na` values and empty string elements sort last in ascending order and first in
descending order. `array.sort_indices` returns a new int array containing
original indexes in sorted order without modifying the source array.
`array.reverse` supports every supported typed array.
`array.join` supports every supported typed array, defaults the
separator to `,`, uses the default numeric string format, and renders colors as
their normalized integer color values. Array assignment passes the runtime array
id by reference; use `array.copy` to allocate an independent array with the same
current element values.

## TA Built-Ins

Initial signatures:

```text
ta.sma(source: series int/float, length: simple int) -> series float
ta.ema(source: series int/float, length: simple int) -> series float
```

Next signatures after Phase 1 is stable:

```text
ta.dema(source: series int/float, length: simple int) -> series float
ta.tema(source: series int/float, length: simple int) -> series float
ta.rma(source: series int/float, length: simple int) -> series float
ta.rsi(source: series int/float, length: simple int) -> series float
ta.macd(source: series int/float, fastlen: simple int, slowlen: simple int, siglen: simple int)
  -> tuple(series float, series float, series float)
ta.tsi(source: series int/float, short_length: simple int, long_length: simple int) -> series float
ta.cmo(source: series int/float, length: simple int) -> series float
ta.cci(source: series int/float, length: simple int) -> series float
ta.cog(source: series int/float, length: simple int) -> series float
ta.ao() -> series float
ta.bop() -> series float
ta.bb(source: series int/float, length: simple int, mult: numeric)
  -> tuple(series float, series float, series float)
ta.bbw(source: series int/float, length: simple int, mult: numeric) -> series float
ta.kc(source: series int/float, length: simple int, mult: simple numeric, useTrueRange?: bool-compatible)
  -> tuple(series float, series float, series float)
ta.kcw(source: series int/float, length: simple int, mult: simple numeric, useTrueRange?: bool-compatible) -> series float
ta.pivothigh(leftbars: simple int, rightbars: simple int) -> series float
ta.pivothigh(source: series int/float, leftbars: simple int, rightbars: simple int) -> series float
ta.pivotlow(leftbars: simple int, rightbars: simple int) -> series float
ta.pivotlow(source: series int/float, leftbars: simple int, rightbars: simple int) -> series float
ta.pivot_point_levels(type: series string, anchor: series bool, developing?: series bool) -> float[]
ta.stdev(source: series int/float, length: simple int, biased?: bool-compatible) -> series float
ta.variance(source: series int/float, length: simple int, biased?: bool-compatible) -> series float
ta.range(source: series int/float, length: simple int) -> series float
ta.dev(source: series int/float, length: simple int) -> series float
ta.vwma(source: series int/float, length: simple int) -> series float
ta.wma(source: series int/float, length: simple int) -> series float
ta.hma(source: series int/float, length: simple int) -> series float
ta.swma(source: series int/float) -> series float
ta.alma(series: series int/float, length: simple int, offset: simple numeric, sigma: simple numeric, floor?: simple bool) -> series float
ta.linreg(source: series int/float, length: simple int, offset: simple int) -> series float
ta.stoch(source: series int/float, high: series int/float, low: series int/float, length: simple int) -> series float
ta.wpr(length: simple int) -> series float
ta.cum(source: series/simple numeric) -> series float
ta.max(source: series/simple numeric) -> series float
ta.min(source: series/simple numeric) -> series float
ta.accdist -> series float
ta.iii -> series float
ta.nvi -> series float
ta.obv -> series float
ta.pvi -> series float
ta.pvt -> series float
ta.tr -> series float
ta.vwap -> series float
ta.vwap(source: series/simple numeric) -> series float
ta.vwap(source: series/simple numeric, anchor: bool-compatible) -> series float
ta.vwap(source: series/simple numeric, anchor: bool-compatible, stdev_mult: simple numeric) -> [series float, series float, series float]
ta.wad -> series float
ta.wvad -> series float
ta.mfi(source: series int/float, length: simple int) -> series float
ta.atr(length: simple int) -> series float
ta.tr(handle_na?: const bool) -> series float
ta.supertrend(factor: simple numeric, atrPeriod: simple int) -> [series float, series float]
ta.dmi(diLength: simple int, adxSmoothing: simple int) -> [series float, series float, series float]
ta.sar(start: simple numeric, inc: simple numeric, max: simple numeric) -> series float
ta.change(source: series int/float/bool, length?: simple int) -> series float/bool
ta.mom(source: series int/float, length: simple int) -> series float
ta.roc(source: series int/float, length: simple int) -> series float
ta.correlation(source1: series/simple numeric, source2: series/simple numeric, length: simple int) -> series float
ta.covariance(source1: series/simple numeric, source2: series/simple numeric, length: simple int) -> series float
ta.median(source: series/simple numeric, length: simple int) -> series float
ta.mode(source: series/simple numeric, length: simple int) -> series float
ta.percentile_nearest_rank(source: series/simple numeric, length: simple int, percentage: input/const numeric) -> series float
ta.percentile_linear_interpolation(source: series/simple numeric, length: simple int, percentage: input/const numeric) -> series float
ta.percentrank(source: series/simple numeric, length: simple int) -> series float
ta.rising(source: series int/float, length: simple int) -> series bool
ta.falling(source: series int/float, length: simple int) -> series bool
ta.barssince(condition: series bool) -> series int
ta.valuewhen(condition: series bool, source: series int/float/bool/color, occurrence: simple int) -> series source-kind
ta.cross(source1: series/simple numeric, source2: series/simple numeric) -> series bool
ta.crossover(source1: series/simple numeric, source2: series/simple numeric) -> series bool
ta.crossunder(source1: series/simple numeric, source2: series/simple numeric) -> series bool
ta.highest(length: simple int) -> series float
ta.highest(source: series int/float, length: simple int) -> series float
ta.lowest(length: simple int) -> series float
ta.lowest(source: series int/float, length: simple int) -> series float
ta.highestbars(length: simple int) -> series int
ta.highestbars(source: series int/float, length: simple int) -> series int
ta.lowestbars(length: simple int) -> series int
ta.lowestbars(source: series int/float, length: simple int) -> series int
```

Rules:

- `length` should initially be `simple int`; reject `series int` lengths.
- TA source parameters documented as `series int/float` accept numeric series
  sources and evaluate through the runtime's floating-point calculation path.
- `ta.bb` currently accepts any numeric qualifier for `mult`.
- `ta.bbw` uses the same basis/deviation window as `ta.bb` and returns
  `(upper - lower) / basis`; it returns `na` when the window is not ready or
  basis is zero.
- `ta.kc` returns Keltner Channels as `[ema(source, length), basis +
  ema(span, length) * mult, basis - ema(span, length) * mult]`, where `span`
  defaults to true range and uses `high - low` when `useTrueRange` is `false`.
- `ta.kcw` uses the same Keltner Channel basis/range EMA calculation and
  returns `(upper - lower) / basis`; it returns `na` when inputs are `na`,
  length is non-positive, or basis is zero.
- `ta.dema` returns `2 * ema(source, length) - ema(ema(source, length),
  length)` using independent callsite state.
- `ta.tema` returns `3 * ema1 - 3 * ema2 + ema3`, where each EMA is the next
  EMA of the previous EMA in the chain, using independent callsite state.
- `ta.pivothigh`/`ta.pivotlow` support the default-source two-argument forms
  and explicit-source three-argument forms. The default sources are `high` and
  `low` respectively. The current subset uses simple integer left/right bar
  counts and returns the confirmed pivot value `rightbars` bars after the pivot
  bar, otherwise `na`.
- `ta.pivot_point_levels` returns an 11-element float array ordered as
  `[P, R1, S1, R2, S2, R3, S3, R4, S4, R5, S5]`. The current subset supports
  `Traditional`, `Fibonacci`, `Woodie`, `Classic`, `DM`, and `Camarilla`
  formulas over runtime bars using the caller-provided `anchor` condition. With
  `developing = false`, levels update from the completed period when `anchor`
  is true; with `developing = true`, levels are recalculated from the current
  in-progress period. It does not request higher-timeframe/session data.
- `ta.stdev` defaults `biased` to `true`; `false` uses sample standard
  deviation and returns `na` for windows shorter than two values.
- `ta.variance` uses the same `biased` default and sample/population window
  rules as `ta.stdev`.
- `ta.range` returns highest minus lowest over the ready rolling window.
- `ta.dev` returns the average absolute deviation from the window mean.
- `ta.vwma` returns `sum(source * volume) / sum(volume)` over the ready rolling
  window and returns `na` when the volume sum is zero.
- `ta.wma` returns a weighted mean where the oldest ready-window value has
  weight `1` and the current value has weight `length`.
- `ta.hma` composes `ta.wma`-style windows as
  `wma(2 * wma(source, length / 2) - wma(source, length), round(sqrt(length)))`.
- `ta.swma` returns a fixed four-bar symmetric weighted average using weights
  `1, 2, 2, 1`; it returns `na` until the fixed window is ready or when the
  window contains `na`.
- `ta.alma` returns the Arnaud Legoux Moving Average using Gaussian weights
  over the ready source window. Optional `floor` floors the offset-derived
  center before weighting.
- `ta.linreg` fits a least-squares line over the ready source window and
  returns `intercept + slope * (length - 1 - offset)`.
- `ta.cum` returns the cumulative sum of numeric source values from the start of
  execution; a current `na` source returns `na` and resets the next cumulative
  step to the next available source value.
- `ta.max` and `ta.min` return the all-time maximum/minimum over executed
  non-`na` source values in their callsite state. A current `na` source leaves
  the previous extreme unchanged; if no non-`na` value has executed yet, they
  return `na`.
- `ta.accdist` is a built-in series variable equivalent to cumulative
  Accumulation/Distribution money flow volume:
  `(((close - low) - (high - close)) / (high - low)) * volume`. It returns
  `na` and resets the next cumulative step when `high == low`.
- `ta.iii` is a built-in series variable equivalent to
  `(2 * close - high - low) / ((high - low) * volume)`; it returns `na` when
  the price range or volume is zero.
- `ta.nvi` is a built-in series variable with an initial value of `1.0`; it
  updates by `((close - close[1]) / close[1]) * previous_nvi` only when
  `volume < volume[1]`, and carries the previous value when the current or
  previous close is zero.
- `ta.obv` is a built-in series variable equivalent to
  `ta.cum(math.sign(ta.change(close)) * volume)`.
- `ta.pvi` is a built-in series variable with an initial value of `1.0`; it
  updates by `((close - close[1]) / close[1]) * previous_pvi` only when
  `volume > volume[1]`, and carries the previous value when the current or
  previous close is zero.
- `ta.pvt` is a built-in series variable equivalent to
  `ta.cum((ta.change(close) / close[1]) * volume)`.
- `ta.tr` variable form is true range without first-bar `na` handling; it
  returns `na` until `close[1]` is available.
- `ta.vwap` variable form returns cumulative
  `sum(hlc3 * volume) / sum(volume)` over the runtime bars.
- `ta.vwap(source)` returns cumulative `sum(source * volume) / sum(volume)` in
  its own call-site state. `ta.vwap(source, anchor)` uses the same call-site
  cumulative state and resets it before the current bar when `anchor` is true.
  `ta.vwap(source, anchor, stdev_mult)` returns `[vwap, upper_band, lower_band]`
  using the call-site weighted standard deviation multiplied by `stdev_mult`.
  These forms return `na` while the cumulative volume is zero. Session-derived
  anchoring is not implemented yet.
- `ta.wad` is a built-in series variable equivalent to cumulative Williams
  Accumulation/Distribution gain using `trueHigh = max(high, close[1])` and
  `trueLow = min(low, close[1])`.
- `ta.wvad` is a built-in series variable equivalent to
  `(close - open) / (high - low) * volume`; it returns `na` when
  `high == low`.
- `ta.change` returns `source - source[length]` for numeric sources and whether
  the value changed for bool sources. It records the required source history
  depth.
- `ta.mom` returns `source - source[length]` and records the required source
  history depth.
- `ta.roc` returns `100 * (source - source[length]) / source[length]` and
  returns `na` when the historical denominator is zero.
- `ta.correlation` returns the Pearson correlation coefficient over the ready
  paired source window and returns `na` while the window is not ready, contains
  `na`, or either source has zero variance.
- `ta.covariance` returns population covariance over the ready paired source
  window and returns `na` while the window is not ready or contains `na`.
- `ta.median` sorts the ready source window ascending and returns the middle
  value, or the average of the two middle values for even windows.
- `ta.mode` returns the most frequent ready-window value. Ties, including
  windows where every value is unique, resolve to the smallest value.
- `ta.percentile_nearest_rank` sorts the ready source window ascending and
  returns the nearest-rank percentile member. It returns `na` while the window
  is not ready, contains `na`, or `percentage` is outside `0..=100`.
- `ta.percentile_linear_interpolation` uses the same ready sorted window and
  interpolates between adjacent ranks. It returns `na` under the same invalid
  window or percentage conditions as `ta.percentile_nearest_rank`.
- `ta.percentrank` returns the percentage of ready-window values less than or
  equal to the current source value. It returns `na` while the window is not
  ready or contains `na`.
- `ta.rising`/`ta.falling` compare the current source against the previous
  ready-window values and return `false` while that window is not ready.
- `ta.barssince` returns `0` on true conditions, increments after the last true
  condition, and returns `na` before the first true condition.
- `ta.valuewhen` returns the `source` value from the nth most recent true
  condition, where `occurrence = 0` is the most recent match.
- `ta.highest`/`ta.highestbars` length-only overloads use `high` as the source.
  `ta.lowest`/`ta.lowestbars` length-only overloads use `low` as the source.
- `ta.highestbars`/`ta.lowestbars` return the offset to the most recent
  matching extreme in the ready window.
- `ta.supertrend` returns `[line, direction]`, where direction is `-1` for the
  uptrend line and `1` for the downtrend line. The current subset follows the
  TradingView band update rules using the runtime's existing RMA-style ATR
  behavior.
- `ta.dmi` returns `[+DI, -DI, ADX]`. The current subset uses Wilder/RMA-style
  smoothing from the first executed bar, reuses `ta.tr(true)` true range
  semantics, and returns an all-`na` tuple for non-positive lengths.
- `ta.sar` supports the three-argument Parabolic SAR form. It initializes from
  the previous bar, clamps against the previous two highs/lows when available,
  and returns `na` until the callsite has enough prior OHLC data to initialize.
- `ta.mfi` supports the two-argument Money Flow Index form using the supplied
  source, current `volume`, and ready positive/negative money-flow windows. It
  returns `na` before the window is ready, when source/volume is `na`, for
  non-positive lengths, or when both flow sums are zero.
- `ta.tsi` returns the True Strength Index in the TradingView-style `[-1, 1]`
  range. The current subset double-smooths source momentum and absolute
  momentum with short then long EMA stages and returns `na` when prior source
  data is unavailable, lengths are non-positive, or the smoothed absolute
  momentum denominator is zero.
- `ta.cmo` returns the Chande Momentum Oscillator as `100 * (sum(up) -
  sum(down)) / (sum(up) + sum(down))` over ready rolling source-change windows.
  It returns `na` before the window is ready, for non-positive lengths, or when
  the denominator is zero.
- `ta.cci` returns the Commodity Channel Index as `(source - sma(source,
  length)) / (0.015 * ta.dev(source, length))`. It returns `na` before the
  rolling window is ready, for non-positive lengths, when source is `na`, or
  when mean absolute deviation is zero.
- `ta.cog` returns the Center of Gravity as `-sum(source[i] * (i + 1)) /
  math.sum(source, length)` over the ready source window. It returns `na` before
  the rolling window is ready, for non-positive lengths, when source is `na`, or
  when the source sum is zero.
- `ta.ao` returns the Awesome Oscillator as `sma(hl2, 5) - sma(hl2, 34)`.
  It returns `na` until both rolling windows are ready or when either window
  contains `na`.
- `ta.bop` returns the Balance of Power as `(close - open) / (high - low)`.
  It returns `na` when any OHLC input is `na` or the high-low range is zero.
- `ta.stoch` supports the four-argument stochastic oscillator form using ready
  rolling `high`/`low` windows. It returns `na` before the window is ready, when
  either window contains `na`, for non-positive lengths, or when the high-low
  range is zero.
- `ta.wpr` supports the single-argument Williams %R form over ready rolling
  `high`/`low` windows and current `close`. It returns `na` before the window is
  ready, for non-positive lengths, or when the high-low range is zero.
- Stateful TA functions require callsite ids.
- Tuple-returning functions require tuple lowering before execution.
- Numerical formulas must be fixture-tested with tolerance.

## Color

```text
color.new(color: color-compatible, transp?: simple int) -> same qualifier color
color.rgb(red: numeric, green: numeric, blue: numeric, transp?: numeric) -> color with strongest qualifier
color.r(color: color-compatible) -> float with same qualifier
color.g(color: color-compatible) -> float with same qualifier
color.b(color: color-compatible) -> float with same qualifier
color.t(color: color-compatible) -> float with same qualifier
color.from_gradient(value: numeric, bottom_value: numeric, top_value: numeric, bottom_color: color-compatible, top_color: color-compatible) -> color with strongest qualifier
```

Named colors include the common TradingView color constants used by fixtures.
Hex color literals in `#RRGGBB` and `#RRGGBBAA` form are accepted as const
colors.
`color.new` defaults `transp` to 0 when omitted.
`color.r`, `color.g`, `color.b`, and `color.t` return `na` for `na` colors;
`color.t` returns transparency on the 0-100 scale.
`color.from_gradient` linearly interpolates RGBA channels between the two
colors, clamps values outside the numeric range to the nearest endpoint, and
returns `na` when any required input is `na`. Equal bottom/top values return
the top color.

Hex color literals are parsed by the syntax layer and lowered to the runtime's
single normalized `Color` representation.

## String

```text
str.length(string: string-compatible) -> int with same qualifier
str.upper(string: string-compatible) -> string with same qualifier
str.lower(string: string-compatible) -> string with same qualifier
str.contains(source: string-compatible, str: string-compatible) -> bool with strongest qualifier
str.startswith(source: string-compatible, str: string-compatible) -> bool with strongest qualifier
str.endswith(source: string-compatible, str: string-compatible) -> bool with strongest qualifier
str.pos(source: string-compatible, str: string-compatible) -> int with strongest qualifier
str.substring(source: string-compatible, begin_pos: int-compatible, end_pos?: int-compatible)
  -> string with strongest qualifier
str.trim(string: string-compatible) -> string with same qualifier
str.repeat(source: string-compatible, repeat: int-compatible, separator?: string-compatible)
  -> string with strongest qualifier
str.replace(source: string-compatible, target: string-compatible, replacement: string-compatible, occurrence?: int-compatible)
  -> string with strongest qualifier
str.replace_all(source: string-compatible, target: string-compatible, replacement: string-compatible)
  -> string with strongest qualifier
str.tonumber(string: string-compatible) -> float with same qualifier
str.tostring(value: int|float|bool|string|non-color-supported-array|na, format?: string-compatible)
  -> string with strongest qualifier
str.format(formatString: string-compatible, arg0?: int|float|bool|string|non-color-supported-array|na, ...)
  -> string with strongest qualifier
str.match(source: string-compatible, regex: string-compatible)
  -> string with strongest qualifier
str.split(source: string-compatible, separator: string-compatible)
  -> simple string-array
str.format_time(time: int-compatible, format?: string-compatible, timezone?: string-compatible)
  -> string with strongest qualifier
```

Supported `str.*` helpers return `na` for `na` inputs.
`str.length` counts Unicode scalar values.
`str.contains`, `str.startswith`, and `str.endswith` return `true` for empty
substring arguments.
`str.pos` returns `na` when no match is found and returns 0 for `na` or empty
substring arguments. `str.substring` treats `na` `begin_pos` as 0 and omitted,
`na`, or too-large `end_pos` as the string length; invalid ranges are runtime
errors.
`str.trim` removes leading and trailing ASCII whitespace only. `str.repeat`
defaults `separator` to an empty string, returns an empty string for repeat 0,
and errors for negative counts or results over 40,960 characters.
`str.replace` replaces one non-overlapping occurrence, defaulting `occurrence`
to 0. `str.replace_all` replaces all non-overlapping occurrences. Empty
targets replace zero-width character boundaries. Replacement results over
40,960 characters are runtime errors.
`str.tonumber` accepts strings containing ASCII digits, an optional leading
sign, and at most one decimal point. It returns `na` for invalid formats,
`na` inputs, and non-finite parsed results.
`str.tostring` supports scalar int, float, bool, string, `na`, and
fixture-covered non-color array values. Numeric formatting supports the default
`#.########`, `format.mintick` and `format.price` as the default format,
`format.volume` as `#.##`, `format.percent` as `#.##%`, and fixture-covered
custom patterns using `#`, `0`, `.`, `,`, and trailing `%` tokens.
`str.format` supports indexed placeholders such as `{0}` and numeric
placeholders such as `{0,number,#.00}`. Missing placeholder indexes remain
literal text. Unmatched braces are runtime errors. Quote handling inside format
strings and non-numeric format modifiers outside the fixture-covered subset are
not yet claimed.
`str.match` uses Rust regex syntax for the fixture-covered subset. It returns
the first matched substring, an empty string when there is no match, `na` for
`na` inputs, and a runtime error for invalid regex patterns.
`str.split` splits by a literal separator and returns a string array. Empty
separators split the source into Unicode scalar values. It returns `na` for
`na` inputs and errors if the result would exceed 100,000 array elements.
`str.format_time` supports UNIX timestamps in milliseconds and a UTC-only
timezone subset (`UTC`, `Etc/UTC`, `GMT`, `Z`, `+0000`, `+00:00`). Omitted or
`na` `format` defaults to `yyyy-MM-dd'T'HH:mm:ssZ`. Supported tokens include
`y`/`Y`, `M`, `d`, `H`, `h`, `m`, `s`, `S`, `a`, `Z`, and single-quoted
literals. Other time zones are runtime errors until exchange/IANA timezone
support is designed.

## Math

Phase 1 may include only the math functions required by fixtures.

Supported constants:

```text
math.e -> const float
math.pi -> const float
math.phi -> const float
math.rphi -> const float
```

Recommended first set:

```text
math.abs(number: numeric) -> same numeric kind and qualifier
math.max(a: numeric, b: numeric, ...) -> promoted numeric kind and strongest qualifier
math.min(a: numeric, b: numeric, ...) -> promoted numeric kind and strongest qualifier
math.avg(number: numeric, ...) -> float with strongest qualifier
math.floor(number: numeric) -> same numeric kind and qualifier
math.ceil(number: numeric) -> same numeric kind and qualifier
math.trunc(number: numeric) -> same numeric kind and qualifier
math.sqrt(number: numeric) -> float with same qualifier
math.cbrt(number: numeric) -> float with same qualifier
math.log(number: numeric) -> float with same qualifier
math.log10(number: numeric) -> float with same qualifier
math.exp(number: numeric) -> float with same qualifier
math.acos(number: numeric) -> float with same qualifier
math.asin(number: numeric) -> float with same qualifier
math.atan(number: numeric) -> float with same qualifier
math.sign(number: numeric) -> float with same qualifier
math.todegrees(radians: numeric) -> float with same qualifier
math.toradians(degrees: numeric) -> float with same qualifier
math.sin(number: numeric) -> float with same qualifier
math.cos(number: numeric) -> float with same qualifier
math.tan(number: numeric) -> float with same qualifier
math.pow(base: numeric, exponent: numeric) -> float with strongest qualifier
math.hypot(number1: numeric, number2: numeric) -> float with strongest qualifier
math.round(number: numeric) -> numeric
math.round(number: numeric, precision: int) -> float with same qualifier
math.round_to_mintick(number: numeric) -> float with same qualifier
math.random(min?: numeric, max?: numeric, seed?: simple int) -> series float
math.sum(source: series/simple numeric, length: simple int) -> series float
```

Each added math function must declare its coercion and `na` behavior.

Current Phase 4 behavior:

- `math.e`, `math.pi`, `math.phi`, and `math.rphi` evaluate as const floats.
- `math.abs` preserves int/float kind and qualifier.
- `math.avg` accepts one or more numeric args and returns their average as a float.
- `math.floor`, `math.ceil`, and `math.trunc` preserve int/float kind and qualifier; float inputs return whole-number floats.
- `math.sqrt`, `math.cbrt`, `math.log`, `math.log10`, `math.exp`, `math.acos`, `math.asin`, `math.atan`, `math.sign`, `math.todegrees`, `math.toradians`, `math.sin`, `math.cos`, `math.tan`, `math.pow`, and `math.hypot` return float values and preserve or promote qualifiers from their arguments.
- `math.round` preserves int/float kind and qualifier when `precision` is omitted; with `precision`, it returns a float rounded to that many decimal places.
- `math.round_to_mintick` rounds to the nearest multiple of the current
  `syminfo.mintick` subset value, with ties rounding up.
- `math.random` returns a deterministic pseudorandom `series float` sequence
  per callsite. Omitted `min`/`max` default to `0` and `1`; seeded calls are
  reproducible for the same callsite and seed. Invalid or non-finite ranges
  return `na`.
- `math.sum` returns the rolling sum of `source` over a ready simple-int `length` window; it returns `na` for invalid lengths, until the window is ready, or when the window contains `na`.
- `math.max` and `math.min` require at least two numeric args and accept variadic numeric args.
- `math.max` and `math.min` return int only when all args are int; otherwise they return float.
- All selected math functions return `na` if any required numeric input is `na`.
