# Architecture

This document defines the long-term architecture for Pine Compat Runtime.

The project should be built as a Rust core with thin bindings for other
environments. The core owns parsing, semantic analysis, intermediate
representations, runtime execution, built-ins, diagnostics, and output
normalization. Host applications should not need to understand parser internals
or Pine-specific execution details.

## High-Level Pipeline

```text
Pine source
  -> lexer
  -> parser
  -> AST
  -> semantic analyzer
  -> HIR
  -> MIR / bytecode
  -> bar-by-bar VM
  -> normalized output
```

Each stage should expose diagnostics with source spans. A later stage must not
hide errors from an earlier stage; it should add context.

## Crates

### `pine-syntax`

Owns source handling, tokens, parser, AST, and syntax diagnostics.

Responsibilities:

- Preserve exact source spans for tokens and AST nodes.
- Parse version declarations, statements, expressions, blocks, function calls,
  named arguments, declarations, reassignment, history references, and comments.
- Provide recoverable diagnostics where possible.
- Avoid runtime or host dependencies.

Recommended approach:

- Hand-written lexer or `logos` lexer.
- Hand-written statement parser.
- Pratt parser for expressions.

### `pine-sema`

Owns semantic analysis.

Responsibilities:

- Resolve names and scopes.
- Infer value kind and qualifier.
- Validate declaration and reassignment rules.
- Validate supported language features.
- Emit compatibility reports for unsupported features.
- Lower AST into HIR.

The analyzer is the boundary where unsupported features should become explicit
diagnostics instead of runtime surprises.

### `pine-ir`

Owns host-independent intermediate representations.

Suggested layers:

- HIR: resolved names, explicit scopes, normalized declarations.
- MIR: runtime-friendly control flow and expressions.
- Bytecode: optional later target for the VM.

The first release can execute MIR directly. The design should leave room for a
bytecode VM once semantics stabilize.

Phase 6 deferred the bytecode VM. See
[`BYTECODE_VM_EVALUATION.md`](BYTECODE_VM_EVALUATION.md) for the decision and
re-evaluation triggers.

### `pine-runtime`

Owns execution.

Responsibilities:

- Execute a compiled program over OHLCV bars.
- Maintain current bar state.
- Maintain committed historical series buffers.
- Implement `var` and later `varip` storage.
- Collect plot, hline, fill, bgcolor, barcolor, and signal side effects.
- Collect drawing-object snapshots behind a host-neutral output contract.
- Enforce runtime limits.

The runtime should be deterministic for a fixed program, data set, and inputs.

Realtime execution uses explicit bar update kinds for historical, forming, and
confirmed bars. See [`REALTIME_MODEL.md`](REALTIME_MODEL.md).

### `pine-builtins`

Owns built-in namespaces and functions.

Initial namespaces:

- `ta`
- `input`
- `plot` functions
- `color`
- `math`
- basic time/bar state helpers

Built-ins must be implemented against runtime abstractions instead of directly
depending on host charting code.

### `pine-cli`

Owns command line usage.

Planned commands:

```text
pine-compat analyze script.pine
pine-compat run script.pine --bars bars.csv --out result.json
pine-compat fmt-ast script.pine
```

### `pine-python`

Owns Python bindings through PyO3 and maturin.

The binding should be thin. It should expose compile, analyze, and run APIs,
but should not duplicate runtime logic in Python.

### `pine-wasm`

Owns browser and plugin use cases.

WASM support should be added after the Rust CLI and Python package are stable.
The initial binding is intentionally thin and returns normalized JSON strings
from compile/analyze/run entry points.

## Core Data Model

### Types and Qualifiers

```rust
enum Qualifier {
    Const,
    Input,
    Simple,
    Series,
}

enum ValueKind {
    Int,
    Float,
    Bool,
    String,
    Color,
    Plot,
    HLine,
    Label,
    Void,
    Na,
}

struct PineType {
    kind: ValueKind,
    qualifier: Qualifier,
}
```

Qualifiers are not decorative metadata. They determine valid function
arguments, expression results, and runtime behavior. Expressions should promote
to the strongest qualifier involved.

### Runtime Values

```rust
enum PineValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(Color),
    Plot(PlotId),
    HLine(HLineId),
    Label(LabelId),
    Na,
    Void,
}
```

Series state should live in a dedicated store instead of inside arbitrary
values.

### Series Store

```rust
struct SeriesStore {
    current_bar: usize,
    buffers: Vec<Vec<PineValue>>,
}
```

`x[1]` means "the committed value of `x` one bar before the current bar." It is
not a normal array index. The runtime must make that distinction explicit.

The actual implementation should key buffers by stable series ids assigned
during lowering. Series ids may represent variables, built-in series, temporary
expressions, function callsites, or plot output. See
[`SERIES_MODEL.md`](SERIES_MODEL.md) for the detailed model.

### Built-In Registry

Built-ins should be declared through a registry shared by semantic analysis and
runtime execution. The registry should include:

- namespace and function name
- accepted positional and named arguments
- value kind and qualifier constraints
- return kind and qualifier behavior
- whether the call requires callsite-local state
- whether the call produces output side effects

See [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md) for the initial supported
surface.

## Public API Shape

Rust:

```rust
let program = pine_compat::compile(source)?;
let report = program.compatibility_report();
let result = program.run(&bars, &inputs)?;
```

Python:

```python
from pine_compat import compile_script

program = compile_script(source)
result = program.run(bars, inputs={"length": 20})
```

CLI:

```bash
pine-compat analyze script.pine
pine-compat run script.pine --bars bars.csv --out result.json
```

## Output Model

The core output must remain host-neutral:

```json
{
  "schemaVersion": 2,
  "plots": [],
  "plotChars": [],
  "plotShapes": [],
  "plotArrows": [],
  "plotBars": [],
  "plotCandles": [],
  "bgColors": [],
  "barColors": [],
  "hlines": [],
  "fills": [],
  "labels": [],
  "lines": [],
  "diagnostics": []
}
```

The `schemaVersion` field is owned by the shared runtime contract and is exposed
unchanged by CLI JSON, Python dictionaries, and WASM JSON. `schemaVersion: 2`
adds top-level drawing-object fields. Host integrations can adapt
this model into their charting or API format, but should preserve the schema
version when they forward machine-readable results.

Drawing-object outputs use sparse snapshot families. The initial drawing
contract reserves `labels` and `lines`, whose entries have an object `id` and a
`snapshots` array. Label snapshots use `barIndex`, `exists`, and, while
`exists` is true, the mutable label fields represented by normalized Pine
values. Phase E starts
with a `label.new` creation subset for `x`, `y`, `text`, `xloc.bar_index`,
`yloc.price`, colors, selected label styles, size, and tooltip metadata, plus
`label.set_*` mutation snapshots for x/y/text/color/style/size/tooltip fields.
`label.delete` appends an `exists: false` snapshot, deleting `na` or an already
deleted label is a no-op, and ids are not reused. The historical runtime caps
labels at 500 objects. Minimal `line.new` support emits line creation snapshots
with `x1`, `y1`, `x2`, and `y2`; line mutation, deletion, limits, realtime
rollback, and optional style fields are implemented in later Phase E slices.

The `pine-runtime` crate owns the shared runtime-result JSON helpers used by the
CLI and WASM bindings. Python keeps explicit dictionary conversion code because
it returns native Python objects, but its top-level runtime result keys are
tested against the same public contract. Analysis reports and compatibility
matrix JSON remain host-specific contracts until a later infrastructure slice
chooses to share them.
