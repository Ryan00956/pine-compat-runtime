# Built-In Signatures

This document defines the first built-in surface as typed signatures. It is a
contract for semantic analysis and runtime implementation.

The syntax below is descriptive, not final Rust API syntax:

```text
name(arg_name: qualifier kind, ...) -> qualifier kind
```

`series<T>` means a series-qualified value of kind `T`.

## Phase 1 Core

Phase 1 should be intentionally small:

- OHLCV and derived built-in series
- `indicator`
- `input`, `input.int`, `input.float`, `input.bool`, `input.source`,
  `input.color`, `input.string`, `input.price`, `input.time`, `input.symbol`,
  `input.timeframe`
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
year      -> series int
month     -> series int
dayofmonth -> series int
hour      -> series int
minute    -> series int
second    -> series int
hl2       -> series float
hlc3      -> series float
ohlc4     -> series float
bar_index -> series int
```

`year`, `month`, `dayofmonth`, `hour`, `minute`, and `second` currently expose
UTC calendar components derived from each bar's `time`. Full exchange-timezone
calendar semantics are not claimed until symbol timezone metadata exists.

The same names are also supported as functions over a timestamp:

```text
year(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
month(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
dayofmonth(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
hour(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
minute(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
second(time: int-compatible, timezone?: string-compatible) -> int with strongest qualifier
```

For now, these function overloads use the same UTC-only timezone subset as
`str.format_time`; unsupported time zones are runtime errors.

Derived values:

```text
hl2   = (high + low) / 2
hlc3  = (high + low + close) / 3
ohlc4 = (open + high + low + close) / 4
```

## Declarations

```text
indicator(title: const string, shorttitle?: const string, overlay?: const bool, ...)
  -> void
```

Only metadata arguments needed by the output model should be accepted in Phase
1. Unsupported named arguments should produce compatibility diagnostics.

## Inputs

```text
input(defval: const int/float/bool/string/color, title?: const string, ...) -> input defval kind
input.int(defval: const int, title?: const string, ...) -> input int
input.float(defval: const float, title?: const string, ...) -> input float
input.bool(defval: const bool, title?: const string, ...) -> input bool
input.color(defval: const color, title?: const string, ...) -> input color
input.string(defval: const string, title?: const string, ...) -> input string
input.price(defval: const float, title?: const string, ...) -> input float
input.time(defval: const int, title?: const string, ...) -> input int
input.symbol(defval: const string, title?: const string, ...) -> input string
input.timeframe(defval: const string, title?: const string, ...) -> input string
input.source(defval: series float, title?: const string, ...) -> series float
```

Rules:

- Input metadata should be collected during analysis.
- Host-provided input values override `defval` at runtime.
- Unsupported options such as complex grouping or display flags should be
  diagnosed before runtime.
- `input.source` returns the selected source series. Phase 1 may restrict this
  to known OHLCV-derived series.

## Plotting

```text
plot(series: series float, title?: const string, color?: color-compatible, ...)
  -> plot

plotchar(series: series/simple numeric-or-bool, title?: const string, char?: const string, color?: color-compatible, ...)
  -> void

plotshape(series: series/simple numeric-or-bool, title?: const string, style?: const string, location?: const string, color?: color-compatible, text?: const string, textcolor?: color-compatible, size?: const string, ...)
  -> void

plotarrow(series: series/simple numeric, title?: const string, colorup?: color-compatible, colordown?: color-compatible, offset?: simple int, minheight?: simple int, maxheight?: simple int, ...)
  -> void

plotbar(open: series/simple numeric, high: series/simple numeric, low: series/simple numeric, close: series/simple numeric, title?: const string, color?: color-compatible, ...)
  -> void

plotcandle(open: series/simple numeric, high: series/simple numeric, low: series/simple numeric, close: series/simple numeric, title?: const string, color?: color-compatible, wickcolor?: color-compatible, bordercolor?: color-compatible, ...)
  -> void

hline(price: const-or-input float, title?: const string, color?: color-compatible, ...)
  -> hline

fill(plot1: plot-or-hline, plot2: plot-or-hline, color?: color-compatible, ...)
  -> void

bgcolor(color: color-compatible, title?: const string, ...) -> void
barcolor(color: color-compatible, title?: const string, ...) -> void
```

`color-compatible` should initially accept:

- const color
- input color
- series color where the target built-in supports dynamic color
- `na` to mean no color for that bar when supported

The output collector should retain plot ids, hline ids, and bar-aligned color
series so host integrations can adapt the normalized result without
reinterpreting the script.

## Utility

```text
na(x: any) -> simple bool or series bool
nz(x: numeric-or-color-series) -> same kind and qualifier as x
nz(x: T, replacement: T) -> strongest qualifier of x and replacement, kind T
```

`na(x)` returns a series-qualified bool when `x` is series-qualified.

`nz` overloads must be explicit. Do not implement `nz` with a generic host
language null helper.

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
array.indexof(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> simple int
array.lastindexof(id: float-array|int-array|bool-array|string-array|color-array, value: element-compatible) -> simple int
array.min(id: float-array|int-array) -> series element
array.max(id: float-array|int-array) -> series element
array.sum(id: float-array|int-array) -> series element
array.avg(id: float-array|int-array) -> series float
array.sort(id: float-array|int-array) -> void
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
`array.insert` inserts a compatible value before the requested index; negative
or greater-than-size indexes are no-ops. `array.remove` removes and returns an
element, or returns `na` for an invalid index. `array.fill` fills the whole
array by default or the half-open `[index_from, index_to)` window when bounds
are supplied; invalid ranges are no-ops. `array.slice` allocates a same-kind
array containing the half-open `[index_from, index_to)` window; invalid bounds
return `na` at runtime. `array.concat` requires two arrays of the same kind,
appends `id2` values to `id` in place, and returns `id`. Numeric array
`min/max/sum/avg` helpers may also be called with method syntax on float and int
array receivers. `array.sort` currently supports float and int arrays only and
sorts ascending with `na` values last. `array.reverse` supports every supported
typed array. `array.join` supports every supported typed array, defaults the
separator to `,`, uses the default numeric string format, and renders colors as
their normalized integer color values. Array assignment passes the runtime array
id by reference; use `array.copy` to allocate an independent array with the same
current element values.

## TA Built-Ins

Initial signatures:

```text
ta.sma(source: series float, length: simple int) -> series float
ta.ema(source: series float, length: simple int) -> series float
```

Next signatures after Phase 1 is stable:

```text
ta.rma(source: series float, length: simple int) -> series float
ta.rsi(source: series float, length: simple int) -> series float
ta.macd(source: series float, fastlen: simple int, slowlen: simple int, siglen: simple int)
  -> tuple(series float, series float, series float)
ta.bb(source: series float, length: simple int, mult: simple float)
  -> tuple(series float, series float, series float)
ta.atr(length: simple int) -> series float
ta.tr(handle_na?: simple bool) -> series float
ta.change(source: series float, length?: simple int) -> series float
ta.cross(source1: series float, source2: series float) -> series bool
ta.crossover(source1: series float, source2: series float) -> series bool
ta.crossunder(source1: series float, source2: series float) -> series bool
ta.highest(source: series float, length: simple int) -> series float
ta.lowest(source: series float, length: simple int) -> series float
```

Rules:

- `length` should initially be `simple int`; reject `series int` lengths.
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
```

Named colors include the common TradingView color constants used by fixtures.
`color.new` defaults `transp` to 0 when omitted.
`color.r`, `color.g`, `color.b`, and `color.t` return `na` for `na` colors;
`color.t` returns transparency on the 0-100 scale.

Hex color parsing should be implemented in the syntax or semantic layer with a
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
fixture-covered non-color array values. Numeric formatting supports the default `#.########`,
`format.mintick` as the default format, `format.percent` as `#.##%`, and
fixture-covered custom patterns using `#`, `0`, `.`, `,`, and trailing `%`
tokens.
`str.format` supports indexed placeholders such as `{0}` and numeric
placeholders such as `{0,number,#.00}`. Missing placeholder indexes remain
literal text. Unmatched braces are runtime errors. Quote handling inside format
strings and non-numeric format modifiers outside the fixture-covered subset are
not yet claimed.
`str.match` uses Rust regex syntax for the fixture-covered subset. It returns
the first matched substring, an empty string when there is no match, `na` for
`na` inputs, and a runtime error for invalid regex patterns.
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
math.sqrt(number: numeric) -> float with same qualifier
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
math.round(number: numeric) -> numeric
math.round(number: numeric, precision: int) -> float with same qualifier
```

Each added math function must declare its coercion and `na` behavior.

Current Phase 4 behavior:

- `math.e`, `math.pi`, `math.phi`, and `math.rphi` evaluate as const floats.
- `math.abs` preserves int/float kind and qualifier.
- `math.avg` accepts one or more numeric args and returns their average as a float.
- `math.floor` and `math.ceil` preserve int/float kind and qualifier; float inputs return whole-number floats.
- `math.sqrt`, `math.log`, `math.log10`, `math.exp`, `math.acos`, `math.asin`, `math.atan`, `math.sign`, `math.todegrees`, `math.toradians`, `math.sin`, `math.cos`, `math.tan`, and `math.pow` return float values and preserve or promote qualifiers from their arguments.
- `math.round` preserves int/float kind and qualifier when `precision` is omitted; with `precision`, it returns a float rounded to that many decimal places.
- `math.max` and `math.min` require at least two numeric args and accept variadic numeric args.
- `math.max` and `math.min` return int only when all args are int; otherwise they return float.
- All selected math functions return `na` if any required numeric input is `na`.
