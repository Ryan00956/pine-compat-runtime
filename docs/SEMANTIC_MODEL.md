# Semantic Model

This document defines the language semantics that must be stable before the
runtime grows broad compatibility.

The first implementation should prefer a small, explicit semantic model over a
large set of partially compatible behaviors.

## Value Kinds

Initial value kinds:

- `int`
- `float`
- `bool`
- `string`
- `color`
- `plot`
- `hline`
- `na`
- `void`

`na` is a real runtime value, but it should not become a universal static type
that erases useful diagnostics. The analyzer may use an internal "unknown due to
na" marker while it waits for contextual type information.

## Qualifiers

Qualifiers describe how a value changes over execution:

```text
const < input < simple < series
```

Promotion rule:

```text
The result qualifier of an expression is the strongest qualifier among the
operands and the called function's declared behavior.
```

Examples:

```pine
1 + 2              // const int
input(20)          // input int
input.int(20) + 1  // input int
input.string("S")  // input string
input.time(0)      // input int
close + 1          // series float
ta.sma(close, 20)  // series float
```

The analyzer must validate function arguments against declared qualifier
constraints. A `series` argument must not be accepted where a `simple` or
`input` value is required unless that specific built-in signature allows it.

## Numeric Coercion

Initial coercion rules:

- `int` can promote to `float`.
- `float` must not silently narrow to `int`.
- Arithmetic on `int` and `float` returns `float` when either side is `float`.
- Division returns `float`.
- Modulo requires numeric operands and should follow the selected Pine version's
  documented behavior.

The analyzer should emit diagnostics for unsupported or ambiguous coercions
instead of deferring them to runtime.

## Loops

`while` statements are statement-only in the current executable subset:

```pine
while condition
    statement
```

The condition must be `bool`. The loop body has its own local scope, and
`break`/`continue` use the same nearest-loop control-flow rules as `for`.
Runtime execution enforces an iteration guard; the semantic analyzer does not
try to prove termination.

## Operators

Arithmetic operators:

```text
+ unary plus
- unary minus
+ addition
- subtraction
* multiplication
/ division
% modulo
```

Comparison operators:

```text
== != > >= < <=
```

Logical operators:

```text
and or not
```

Conditional operator:

```text
condition ? true_expr : false_expr
```

The ternary result kind should be the least common compatible kind of both
branches. Its qualifier should be the strongest qualifier among the condition
and both branch expressions.

Switch expressions:

```pine
value = switch
    close > open => high
    close < open => low
    => close

value = switch direction
    1 => high
    -1 => low
    => close
```

The current executable subset supports expression arms only. Selector-less arm
conditions must be `bool`. Selector-form cases are compared with equality in
source order. Arm result kinds must have a common compatible kind, following the
same branch merge rules as ternary expressions. The result qualifier is the
strongest qualifier among the selector or conditions and the selected result
expressions.

## Arrays

The current array subset supports float, int, bool, string, and color arrays:

```pine
var values = array.new_float()
array.push(values, close)
values.push(close)
first = array.get(values, 0)
same = values.get(0)
count = array.size(values)

var counts = array.new_int()
counts.push(bar_index)

var flags = array.new_bool()
flags.push(close > open)

var names = array.new_string()
names.push("seed")

var shades = array.new_color()
shades.push(color.red)
```

`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`, and
`array.new_color` return runtime-owned array ids. `array.from` allocates a
runtime-owned array id with an element kind inferred from its arguments; at
least one non-`na` supported typed value is required, `na` may be mixed into an
otherwise typed array, and mixed int/float arguments produce a float array.
Normal declarations allocate a fresh array whenever the declaration executes.
`var` declarations preserve the array id and backing storage across bars.
Supported operations are
`array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`,
`array.new_color`, `array.from`, `array.push`, `array.get`, `array.set`, `array.size`,
`array.insert`, `array.pop`, `array.remove`, `array.shift`, `array.unshift`,
`array.fill`, `array.first`, `array.last`, and `array.copy`, `array.slice`,
`array.concat`, `array.includes`, `array.indexof`, `array.lastindexof`,
`array.binary_search`, `array.binary_search_leftmost`,
`array.binary_search_rightmost`, `array.abs`, `array.min`, `array.max`,
`array.sum`, `array.avg`, `array.range`, `array.median`, `array.mode`,
`array.variance`, `array.stdev`, `array.percentile_nearest_rank`,
`array.percentile_linear_interpolation`, `array.percentrank`,
`array.covariance`, `array.standardize`, `array.sort`, `array.reverse`,
`array.join`, and `array.clear`;
`size/get/set/insert/push/pop/remove/shift/unshift/fill/first/last/copy/slice/concat/includes/indexof/lastindexof/reverse/join/clear`
may also be called with method syntax on a supported array receiver. Numeric
`binary_search/binary_search_leftmost/binary_search_rightmost/abs/min/max/sum/avg/range/median/mode/percentile_nearest_rank/percentile_linear_interpolation/percentrank/covariance/standardize/variance/stdev/sort`
helpers may also be called with method syntax on float and int arrays. Float
arrays accept int or float values and store them as
floats. Int arrays accept int values. Bool arrays accept bool values. String
arrays accept string values. Color arrays accept color values. Other array
constructors and unsupported `array.*` functions are rejected. Array assignment
and UDF argument binding pass the runtime array id by reference. `array.copy`
allocates a new array id with the same current element values, so later
mutations do not affect the source. `array.slice` allocates a same-kind array
containing the half-open `[index_from, index_to)` window; invalid bounds return
`na` at runtime. `array.concat` requires two arrays of the same kind, appends
the second array's current values to the first array in place, and returns the
first array id. `array.insert` inserts before a valid index and is a no-op for
negative or greater-than-size indexes. `array.remove` removes and returns a
valid indexed element, or returns `na` when the index is invalid. `array.fill`
replaces all elements by default or a half-open `[index_from, index_to)` window
when bounds are supplied; invalid ranges are no-ops.
`array.indexof` and `array.lastindexof` return `-1` when the value is not
present. Numeric binary search helpers are limited to float and int arrays and
expect the current array contents to be sorted ascending. `array.binary_search`
returns `-1` when the value is not found; leftmost/rightmost return the nearest
existing insertion-side index and return `-1` for empty arrays. Numeric helpers
`array.abs`, `array.min`, `array.max`, `array.sum`, `array.avg`, `array.range`,
`array.median`, `array.mode`, `array.covariance`, `array.standardize`,
`array.variance`, and `array.stdev` are limited to float and int arrays.
`array.abs` allocates a new same-kind array, preserves `na`, and leaves the
source array unchanged. `array.covariance` requires two same-size numeric
arrays, skips pairs where either side is `na`, defaults to a biased population
estimate, and returns `na` when no numeric pair remains or an unbiased sample
has fewer than two numeric pairs.
`array.standardize` allocates a new float array, uses non-`na` values to
calculate mean and population standard deviation, preserves `na` element
positions when at least one numeric value exists, and returns an empty array
for empty/all-`na` arrays. The remaining helpers ignore `na` elements and
return `na` when no numeric element is present. `array.range` returns max minus
min. `array.mode`
returns the smallest value among tied most-frequent values and returns `na`
when all remaining values occur only once. Percentile helpers operate on
non-`na` values sorted ascending. Percentages outside `0..=100`,
empty/all-`na` arrays, and invalid `array.percentrank` indexes return `na`.
`array.variance` and `array.stdev`
accept an optional `biased` bool argument that defaults to `true`; passing
`false` uses the sample denominator and returns `na` when fewer than two
numeric values remain.
`array.sort` is currently limited to float and int arrays, sorts ascending in
place, and leaves `na` values at the end.
`array.reverse` reverses any supported typed array in place. `array.join`
converts supported array elements to string with the default numeric format,
uses `,` as the default separator, and returns an empty string for empty arrays.
Color elements render as normalized integer color values. Out-of-range
`array.get`, empty `array.pop`, empty `array.shift`, and `array.first`/`array.last` on empty arrays
return `na`; out-of-range `array.remove` returns `na`; out-of-range
`array.set`, `array.insert`, and `array.fill` are no-ops. Negative array sizes fail at
runtime. Each array can contain at most 100,000 elements; creation, push,
unshift, insert, or concat operations beyond that limit fail at runtime.

User-defined functions may receive supported arrays and use read-only
operations such as `array.size` and `array.get`. Array mutation inside
user-defined functions is rejected until function side-effect semantics are
broader.

## `na`

`na` behavior must be implemented deliberately. It must not be represented as
Rust `None` in the evaluator because `na` participates in Pine expressions.

Initial rules:

- Arithmetic with `na` returns `na`.
- Historical references outside available history return `na`.
- `na(x)` returns `true` when `x` is `na`, otherwise `false`.
- `nz(x)` returns `x` when `x` is not `na`; otherwise it returns the default
  replacement for the overload or the explicit replacement argument.
- `nz(x, replacement)` returns `replacement` when `x` is `na`.
- A condition that evaluates to `na` should follow the selected Pine version's
  documented behavior. Phase 1 should target one version policy and test it
  explicitly.

The analyzer should track Pine version because boolean `na` behavior differs
across versions. If a version-specific behavior is not implemented, it must be
reported as unsupported instead of approximated silently.

## Declarations

Normal declarations are evaluated whenever their containing scope executes:

```pine
x = close + open
```

The declaration creates or updates the current bar value for that symbol. If the
symbol is series-qualified, the current value is committed after bar execution.

Normal and tuple block-local declarations inside `if` blocks are executable and
scoped to their branch. The analyzer records resolved symbol bindings before
lowering so locals do not leak into outer scopes. Tuple declarations in local
scopes always create local symbols, even when an outer scope already contains the
same names.

`for i = start to end` is executable for integer ranges with an optional
explicit `by step`. The loop counter is a block-local integer symbol scoped to
the loop body. Range and step expressions must be int-typed, and literal zero
steps are rejected; series-qualified bounds are evaluated for the current bar
when the loop statement executes.
`break` and `continue` are valid only inside a loop and target the nearest
enclosing loop.
Loop expressions are supported for scalar and tuple declaration values when the
body ends with an expression. The expression determines the loop result type;
bodies that do not end with an expression are rejected with `E_LOOP_RETURN`.

## Reassignment

Reassignment updates an already resolved symbol:

```pine
x := x + 1
```

Rules:

- Reassignment to an unknown name is a semantic error.
- Reassignment must respect the symbol's declared value kind.
- The assigned expression may strengthen the qualifier only when the language
  rules allow the target to become series-qualified.
- Reassignment inside a local scope must resolve according to Pine scoping
  rules, not ordinary block-local shadowing assumptions.

## `var`

`var` declarations initialize once at the first execution of their declaration
site:

```pine
var x = 0
```

Rules:

- The declaration site owns persistent storage.
- The initializer runs only when the declaration site is first reached.
- Later executions read the persistent value.
- Local `var` declarations in blocks and inlined user-defined functions use
  declaration-site storage, with independent storage per syntactic UDF callsite.
- Reassignment writes the persistent value for subsequent bars.

`var` storage is separate from ordinary per-bar local values and from committed
series history.

## Function Calls

Each built-in and user-defined function call must have a stable callsite id.

Callsite ids are needed because many functions have stateful series behavior.
For example, two separate calls to `ta.ema(close, 20)` must not accidentally
share rolling state unless the language semantics require sharing.

The lowering stage should assign callsite ids before runtime execution.

## User-Defined Functions

User-defined functions are supported by lowering each callsite inline.
Expression bodies lower directly as expressions. Multi-statement block bodies
lower as block expressions that execute local statements and return the final
expression. Positional and named arguments are resolved to the declared
parameter list before semantic analysis of the function body.

Current rules:

- Resolve parameters and local symbols.
- Infer return kind and qualifier.
- Reject recursion.
- Allocate independent callsite state for every syntactic callsite.
- Functions called conditionally follow the same conditional callsite rules as
  built-ins.
- Resolve local declarations and reassignments inside function block bodies to
  callsite-local symbols.
- Preserve parameter shadowing when a function-local declaration or loop counter
  uses the same name as a parameter.
- Reject duplicate or unknown named UDF arguments and positional arguments after
  named arguments.
- Evaluate arguments once into callsite-local temporaries before evaluating the
  inlined function body.
- Reject output side effects inside functions.
- Reject global reassignment inside functions.
- Reject side-effecting calls as UDF arguments.

Function block bodies must end with an expression.
