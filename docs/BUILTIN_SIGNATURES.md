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
- `input.int`, `input.float`, `input.bool`, `input.source`, `input.color`
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
hl2       -> series float
hlc3      -> series float
ohlc4     -> series float
bar_index -> series int
```

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
input.int(defval: const int, title?: const string, ...) -> input int
input.float(defval: const float, title?: const string, ...) -> input float
input.bool(defval: const bool, title?: const string, ...) -> input bool
input.color(defval: const color, title?: const string, ...) -> input color
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

hline(price: const-or-input float, title?: const string, color?: color-compatible, ...)
  -> hline

fill(plot1: plot-or-hline, plot2: plot-or-hline, color?: color-compatible, ...)
  -> void

bgcolor(color: series color) -> void
barcolor(color: series color) -> void
```

`color-compatible` should initially accept:

- const color
- input color
- series color where the target built-in supports dynamic color
- `na` to mean no color for that bar when supported

The output collector should retain plot ids and hline ids so host integrations
can adapt the normalized result without reinterpreting the script.

## Utility

```text
na(x: any) -> simple bool or series bool
nz(x: numeric-or-color-series) -> same kind and qualifier as x
nz(x: T, replacement: T) -> strongest qualifier of x and replacement, kind T
```

`na(x)` returns a series-qualified bool when `x` is series-qualified.

`nz` overloads must be explicit. Do not implement `nz` with a generic host
language null helper.

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
color.new(color: color-compatible, transp: simple int) -> same qualifier color
```

Initial named colors should include only the common names needed by fixtures.
Unsupported named colors should be diagnostics until the registry is complete.

Hex color parsing should be implemented in the syntax or semantic layer with a
single normalized `Color` representation.

## Math

Phase 1 may include only the math functions required by fixtures.

Recommended first set:

```text
math.abs(number: numeric) -> same numeric kind and qualifier
math.max(a: numeric, b: numeric, ...) -> promoted numeric kind and strongest qualifier
math.min(a: numeric, b: numeric, ...) -> promoted numeric kind and strongest qualifier
math.round(number: numeric) -> numeric
```

Each added math function must declare its coercion and `na` behavior.

Current Phase 4 behavior:

- `math.abs` preserves int/float kind and qualifier.
- `math.round` preserves int/float kind and qualifier; float inputs round to the nearest whole float.
- `math.max` and `math.min` require at least two numeric args and accept variadic numeric args.
- `math.max` and `math.min` return int only when all args are int; otherwise they return float.
- All selected math functions return `na` if any required numeric input is `na`.
