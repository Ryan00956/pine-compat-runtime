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

Phase F introduces a host-neutral request data boundary before enabling
`request.*` execution. Core runtime code owns chart metadata, request keys,
timeframe parsing, requested-bar validation, and provider error shapes, but it
must not fetch network data or read host files. Hosts supply immutable requested
bar streams through the shared request provider contract. The default runtime
environment keeps the existing fixed chart metadata and no-request provider so
current `HistoricalRuntime::new`, `RealtimeRuntime::new`, and `run_historical`
call sites keep their behavior until request execution is explicitly enabled.
CLI uses repeated `--request-bars SYMBOL:TIMEFRAME=bars.csv` options and Python
accepts a `request_bars` dictionary with the same `SYMBOL:TIMEFRAME` keys. WASM
does not yet expose request dataset injection; scripts requiring provider data
fail with the shared missing-request-data runtime error until a JSON host shape
is added. The cross-host request fixture can be exercised with:

```text
cargo run -p pine-cli -- run tests/fixtures/request/request_security_host.pine \
  --bars tests/fixtures/request/chart_1m.csv \
  --request-bars NYSE:IBM:1=tests/fixtures/request/ibm_1m.csv \
  --request-bars NYSE:IBM:5=tests/fixtures/request/ibm_5m.csv
```

Provider-backed `request.security` expressions are evaluated in a separate
requested-context `HistoricalRuntime` over the immutable provider bars, then
cached by callsite, requested symbol, requested timeframe, and HIR expression
identity. That keeps requested history, `ta.*` callsite state, `var` storage,
arrays, and drawing state isolated from the chart runtime. Slice 4 intentionally
uses the lowered HIR expression debug identity as the cache expression marker;
future widening that rewrites request expressions should replace it with an
explicit request-expression id.

For higher-timeframe provider requests, alignment uses the default
`lookahead_off`/`gaps_off` subset: same-timeframe requests still require an
exact requested-bar timestamp match, while coarser requested bars are visible
only after their requested bar close is not later than the current chart bar
close. Missing higher-timeframe bars forward-fill the last confirmed requested
value; chart bars before the first confirmed requested bar return `na`.
Lower-timeframe `request.security` alignment is intentionally not implemented
in Phase F because it needs a separate rule for selecting intrabars inside each
chart bar and bounded storage for multiple requested bars per chart bar. The
array-returning `request.security_lower_tf` API remains unsupported until typed
array return shapes and host JSON bindings are designed together. Phase F's
closed request boundary and maintenance tails are recorded in
[`PHASE_F_AUDIT.md`](PHASE_F_AUDIT.md).

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
result = program.run(bars, request_bars={"NYSE:IBM:1": requested_bars})
```

CLI:

```bash
pine-compat analyze script.pine
pine-compat run script.pine --bars bars.csv --request-bars NYSE:IBM:1=ibm.csv
```

## Output Model

The core output must remain host-neutral:

```json
{
  "schemaVersion": 3,
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
  "boxes": [],
  "tables": [],
  "alerts": [],
  "diagnostics": []
}
```

The runtime `schemaVersion` field is owned by
`PUBLIC_RUNTIME_SCHEMA_VERSION` and is exposed unchanged by CLI runtime JSON,
Python runtime dictionaries, and WASM runtime JSON. `schemaVersion: 2` added
top-level drawing-object fields, and `schemaVersion: 3` reserves the top-level
`alerts` event array. Phase H's initial event shape is `{id, barIndex, time,
message, source}` for the narrow `alertcondition` subset, where `source` is the
const title and `message` is the const message. Host integrations can adapt
this model into their charting or API format, but should preserve the runtime
schema version when they forward machine-readable runtime results.

Machine-readable analysis and matrix outputs use separate schema ownership:
`PUBLIC_ANALYSIS_SCHEMA_VERSION` for WASM/Python analysis reports and
`PUBLIC_MATRIX_SCHEMA_VERSION` for CLI matrix JSON. Runtime is currently `3`;
analysis and matrix remain `2`. These contracts can evolve independently when a
runtime-only output field does not affect analysis or matrix contracts.

Drawing-object outputs use sparse snapshot families. The Phase E drawing
contract reserves `labels`, `lines`, `boxes`, and `tables`, whose entries have
an object `id` and a `snapshots` array. Label snapshots use `barIndex`,
`exists`, and, while `exists` is true, the mutable label fields represented by
normalized Pine values. The label lifecycle covers `label.new`, selected
`label.set_*` mutators, and `label.delete`. Line snapshots cover `x1`, `y1`,
`x2`, `y2`, `color`, `width`, `style`, and `extend` for `line.new`, selected
`line.set_*` mutators, and `line.delete`. Box snapshots cover `left`, `top`,
`right`, `bottom`, `bgColor`, `borderColor`, `borderWidth`, and `borderStyle`
for `box.new`, selected `box.set_*` mutators, and `box.delete`. Table entries
carry `position`, `columns`, `rows`, and sparse cell snapshots. Each table cell
snapshot stores `column`, `row`, `text`, `bgColor`, and `textColor`, avoiding
host-specific table layout assumptions. Delete calls append an `exists: false`
snapshot for families with deletion; deleting `na` or an already deleted
drawing object is a no-op; ids are not reused. The historical runtime caps
labels, lines, and boxes at 500 objects, caps tables at 50 objects, and caps a
single table at 1000 cells.

Alert events are flat runtime events rather than sparse snapshots. Historical
execution appends events in program order when an `alertcondition` call is
reached and its condition is true. Realtime forming events live in the forming
runtime snapshot and are discarded on rollback unless a confirmed update emits
the same event.

The `pine-runtime` crate owns the shared runtime-result JSON helpers used by the
CLI and WASM bindings. Python keeps explicit dictionary conversion code because
it returns native Python objects, but its top-level runtime result keys are
tested against the same public contract. Analysis reports and compatibility
matrix JSON keep distinct schema constants even where their current field shapes
remain host-owned.
