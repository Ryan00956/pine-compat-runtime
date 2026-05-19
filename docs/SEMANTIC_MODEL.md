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

The first array subset supports float arrays only:

```pine
var values = array.new_float()
array.push(values, close)
values.push(close)
first = array.get(values, 0)
same = values.get(0)
count = array.size(values)
```

`array.new_float` returns a runtime-owned array id. Normal declarations allocate
a fresh array whenever the declaration executes. `var` declarations preserve the
array id and backing storage across bars. Supported operations are
`array.new_float`, `array.push`, `array.get`, `array.set`, `array.size`,
`array.pop`, and `array.clear`; `size/get/set/push/pop/clear` may also be
called with method syntax on a float array receiver. Non-float array
constructors and unsupported `array.*` functions are rejected. Out-of-range
`array.get` and empty `array.pop` return `na`; out-of-range `array.set` is a
no-op. Negative
`array.new_float` sizes fail at runtime. A float array can contain at most
100,000 elements; creation or push operations beyond that limit fail at runtime.

User-defined functions may receive float arrays and use read-only operations
such as `array.size` and `array.get`. Array mutation inside user-defined
functions is rejected until function side-effect semantics are broader.

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
