# Language Scope

The project should start with an indicator-focused subset. Scope discipline is
more important than broad, incomplete support.

## Version Policy

The parser should recognize version declarations:

```pine
//@version=4
//@version=5
//@version=6
```

The runtime can initially support a shared v5/v6-style subset. Unsupported
version-specific behavior must be reported in diagnostics.

## Initial Supported Syntax

The first executable subset should be smaller than the first parseable subset.
The parser may recognize more syntax than the runtime accepts, but unsupported
runtime behavior must become diagnostics during semantic analysis.

Statements:

- version declaration comments
- `indicator(...)`
- variable declarations
- `var` declarations
- reassignment with `:=`
- `if` statements
- local blocks
- user-defined functions with positional and named arguments
- simple `for` loops only after the expression runtime and callsite state are
  stable

Expressions:

- literals: int, float, bool, string, color literals where applicable
- identifiers
- function calls
- named arguments
- tuple expressions and tuple assignment
- arithmetic operators: `+`, `-`, `*`, `/`, `%`
- comparison operators: `==`, `!=`, `>`, `>=`, `<`, `<=`
- logical operators: `and`, `or`, `not`
- ternary operator: `condition ? a : b`
- history operator: `expr[offset]`

Phase 1 executable subset:

- global declarations
- `if`/`else` blocks for expression statements, plot calls, reassignment, and
  block-local tuple declarations
- `for i = start to end` loops over inclusive integer ranges with optional
  `by step`, `break`, `continue`, and loop result assignment
- partial `while condition` statement loops with bool/`na` conditions, `break`,
  `continue`, local `var`, nested loops, and a runtime iteration guard
- partial `switch` expressions with condition arms, selector/case arms, and
  expression results
- branch/loop interactions for `if` inside loops, loops inside `if`, `switch`
  inside loops, and loops inside UDF block bodies
- normal and tuple declarations scoped to an `if`/`else` branch
- user-defined functions lowered by inlining
- arithmetic, comparison, logical, and ternary expressions
- constant history offsets and guarded dynamic integer history offsets
- `indicator`
- `input.*`
- `plot`, `plotchar`, `plotshape`, `plotarrow`, `plotbar`, `plotcandle`,
  `bgcolor`, `barcolor`, `hline`, and `fill`
- `na`, `nz`
- common `ta.*` helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md), including moving averages,
  rolling statistics, momentum/history helpers, crosses, extremes, trend
  checks, value lookups, true range, volume flow helpers, and partial VWAP
- partial float, int, bool, string, and color arrays with `array.new_float`,
  `array.new_int`, `array.new_bool`, `array.new_string`, `array.new_color`,
  `array.from`, `array.push`, `array.get`, `array.set`, `array.size`, `array.pop`,
  `array.insert`, `array.remove`, `array.shift`, `array.unshift`,
  `array.fill`, `array.first`, `array.last`, `array.copy`, `array.slice`,
  `array.concat`, `array.includes`, `array.every`, `array.some`,
  `array.indexof`, `array.lastindexof`, numeric `array.binary_search*`,
  `array.clear`, numeric `array.abs`, `array.min`, `array.max`, `array.sum`,
  `array.avg`, `array.range`, `array.median`, `array.mode`,
  `array.percentile_nearest_rank`, `array.percentile_linear_interpolation`,
  `array.percentrank`, `array.covariance`, `array.standardize`,
  `array.variance`, `array.stdev`, numeric/string `array.sort`,
  numeric/string `array.sort_indices`, `array.reverse`, `array.join`, and
  equivalent method-call syntax such as
  `values.push(close)` and `values.get(0)`

Stateful calls inside `if` blocks advance their callsite state only when the
branch executes. Series values not evaluated on a bar are committed as `na` to
keep history buffers bar-aligned.

Stateful calls inside `switch` arms follow the same conditional callsite rule:
only the selected arm result executes. Selector-form switches evaluate the
selector once per bar, compare cases in source order, and return `na` when no
arm matches and no default arm is present.

Normal and tuple block-local declarations inside `if` blocks are scoped to the
branch and do not leak outside it. A tuple declaration in a local scope shadows
outer variables with the same names; use reassignment syntax for scalar updates
to existing variables. `var` declarations in local blocks initialize the first
time their declaration site is reached, then preserve state across later
executions.

`for` loops support inclusive integer ranges with an optional explicit `by`
step. The runtime increments when `from <= to` and decrements when `from > to`;
the absolute step magnitude is used, so signed step values do not override the
range direction. If a runtime range bound or step evaluates to `na`, the loop
body is skipped and a loop expression returns `na`. The loop counter is scoped
to the loop body. Step values must be non-zero ints.
`break` exits the nearest enclosing loop and `continue` skips to its next
iteration.
`x = for ...` and tuple assignment from `for` results are supported when the
loop body ends with an expression. The assigned value is the latest iteration
result that reached that expression, or `na` if no iteration reaches it.

`while` loops are statement-only in the current executable subset. Conditions
must type-check as bool; a runtime `na` condition exits the loop like false.
`break` and `continue` target the nearest enclosing loop. A deterministic
iteration guard prevents runaway loops. `while` expressions are rejected.

`switch` support is expression-only. Each arm must return a single expression;
statement-block switch arms are rejected.

User-defined functions support single-expression and multi-statement block
bodies:

```pine
smooth(src, len) => ta.sma(src, len)
spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))

range2(hi, lo) =>
    value = hi - lo
    value * 2
```

Named arguments are supported for user-defined functions. Block bodies must end
with an expression. Recursive functions, output side effects inside functions,
global reassignment inside functions, and side-effecting calls as UDF arguments
are rejected in the current executable subset. UDF arguments are evaluated once
into callsite-local temporaries.

## Initial Built-Ins

This is the first broad target set, not the first executable milestone. The
minimal executable Phase 1 built-ins are defined in
[`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md).

Global values:

- `open`
- `high`
- `low`
- `close`
- `volume`
- `time`
- `hl2`
- `hlc3`
- `ohlc4`
- `bar_index`
- `na`

Input namespace:

- `input`
- `input.int`
- `input.float`
- `input.bool`
- `input.source`
- `input.color`
- `input.string`
- `input.price`
- `input.time`
- `input.symbol`
- `input.timeframe`
- `input.session`
- `input.text_area`

TA namespace:

- common indicator helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md)

Plotting:

- `plot`
- `plotchar`
- `plotshape`
- `plotarrow`
- `plotbar`
- `plotcandle`
- `hline`
- `fill`
- `bgcolor`
- `barcolor`

Color namespace:

- common named colors
- `color.new`
- `color.rgb`
- color component helpers
- `color.from_gradient`
- hex color parsing

Utility:

- `na`
- `nz`
- selected `math.*`, `str.*`, and UTC time helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md)

Request data:

- `request.security(syminfo.tickerid, timeframe.period, expression)` for the
  current chart context only. The requested expression must be scalar and
  side-effect-free.
- `request.security("SYMBOL", timeframe.period, expression)` for host-provided
  same-timeframe bars. The provider expression subset includes direct
  OHLCV/time sources, pure arithmetic and ternaries, history references, `na`,
  `nz`, `ta.sma`, and `ta.ema`; local variable aliases inside provider
  expressions are not part of this subset. CLI hosts pass these bars with
  `--request-bars SYMBOL:TIMEFRAME=bars.csv`; Python hosts pass
  `request_bars={"SYMBOL:TIMEFRAME": bars}`. WASM request dataset injection is a
  documented temporary gap.

## Explicitly Unsupported in Phase 1

The analyzer should reject these with clear diagnostics:

- `strategy.*`
- `request.*` variants outside the narrow same-context and same-timeframe
  provider-backed `request.security` subsets
- `alert` and `alertcondition`
- `library`, `import`, and `export`
- unsupported array element types, matrices, and maps
- user-defined types
- non-array methods
- label, line, box, table, polyline objects
- multi-symbol or multi-timeframe data loading
- broker emulation and order execution
- realtime-only `varip` semantics

Longer-term work for these unsupported areas is tracked in
[`LONG_TERM_EXECUTION_PLAN.md`](LONG_TERM_EXECUTION_PLAN.md).

## Compatibility Report

The analyzer should return a machine-readable report:

```json
{
  "languageVersion": 5,
  "supported": [
    {"feature": "ta.ema", "span": "..."}
  ],
  "unsupported": [
    {
      "feature": "request.security",
      "reason": "Only same-context identity and same-timeframe scalar provider requests are supported in phase 1",
      "span": "..."
    }
  ]
}
```

The user experience should be "this part is unsupported" rather than "the
script crashed."
