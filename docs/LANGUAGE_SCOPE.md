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
- expression-body user-defined functions with positional and named arguments
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
  tuple assignment to variables declared before the block
- normal declarations scoped to an `if`/`else` branch
- expression-body user-defined functions lowered by inlining
- arithmetic, comparison, logical, and ternary expressions
- constant history offsets
- `indicator`
- `input.*`
- `plot`, `hline`, and `fill`
- `na`, `nz`
- `ta.sma` and `ta.ema`

Stateful calls inside `if` blocks advance their callsite state only when the
branch executes. Series values not evaluated on a bar are committed as `na` to
keep history buffers bar-aligned.

Normal block-local declarations inside `if` blocks are scoped to the branch and
do not leak outside it. Tuple declarations that create new block-local symbols
are rejected for now; declare the tuple targets before the block when assigning
tuple-returning calls conditionally.

User-defined functions are limited to single expression bodies:

```pine
smooth(src, len) => ta.sma(src, len)
spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))
```

Named arguments are supported for user-defined functions. Multi-statement
function bodies, recursive functions, output side effects inside functions, and
stateful or side-effecting calls as UDF arguments are rejected in the current
executable subset.

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

- `input.int`
- `input.float`
- `input.bool`
- `input.source`
- `input.color`

TA namespace:

- `ta.sma`
- `ta.ema`
- `ta.rma`
- `ta.rsi`
- `ta.macd`
- `ta.bb`
- `ta.atr`
- `ta.tr`
- `ta.change`
- `ta.cross`
- `ta.crossover`
- `ta.crossunder`
- `ta.highest`
- `ta.lowest`

Plotting:

- `plot`
- `hline`
- `fill`
- `bgcolor`
- `barcolor`

Color namespace:

- common named colors
- `color.new`
- hex color parsing

Utility:

- `na`
- `nz`
- basic `math.*` functions

## Explicitly Unsupported in Phase 1

The analyzer should reject these with clear diagnostics:

- `strategy.*`
- `request.*`
- `alert` and `alertcondition`
- `library`, `import`, and `export`
- arrays, matrices, and maps
- user-defined types
- methods
- label, line, box, table, polyline objects
- multi-symbol or multi-timeframe data loading
- broker emulation and order execution
- realtime-only `varip` semantics

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
      "reason": "Multi-timeframe data requests are not supported in phase 1",
      "span": "..."
    }
  ]
}
```

The user experience should be "this part is unsupported" rather than "the
script crashed."
