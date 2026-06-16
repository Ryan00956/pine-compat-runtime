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

Phase J introduces a source graph scaffold and the first executable import
subset. Public semantic analysis can now be driven by `AnalysisInput`, which
contains a root `SourceFile` and an optional deterministic list of
host-provided library sources. `SourceGraph` assigns stable `SourceId` values
with root source id `0` and library source ids sorted by import key, while each
source unit keeps a diagnostic display name. Library keys are normalized by
trimming outer whitespace, reject empty or whitespace/control-containing keys,
and duplicate keys are rejected before analysis. This model is intentionally
host-neutral: core crates do not read files, fetch network data, consult
clocks, or resolve library names outside the host-provided map.

The executable subset accepts exact-key `import ... as alias` when the host
provides the matching library source. Exported const expressions are inlined,
and exported pure functions are lowered through the existing UDF path under
alias-qualified call targets. Runtime execution still receives a fully lowered
HIR program; it does not resolve imports or inspect source graphs.

The Phase J UDT/method subset is intentionally root-local. Semantic analysis
records local scalar-field type declarations, constructor calls, field reads,
and pure UDT methods before lowering. UDT values lower to immutable runtime
values, and local UDT methods lower through the same inlined body machinery as
ordinary UDF calls with the receiver passed as the first internal parameter.
Method-local UDT parameter identity is checked during semantic analysis and
carried into lowering for passthrough returns. Imported UDT identity and
imported method tables are not part of the current source-graph contract.

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
CLI uses repeated `--request-bars SYMBOL:TIMEFRAME=bars.csv` options, Python
accepts a `request_bars` dictionary with the same `SYMBOL:TIMEFRAME` keys, and
WASM accepts a deterministic request-bars JSON object through
`runScriptCsvWithRequestBars`,
`runScriptCsvWithLibrariesAndRequestBars`, and
`Program.runCsvWithRequestBars`. WASM request keys use the same
`SYMBOL:TIMEFRAME` format and split on the last colon, so exchange-prefixed
symbols such as `NYSE:IBM:1` are valid. The cross-host request fixture can be
exercised with:

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

Strategy execution is owned by `pine-runtime::strategy`. `BrokerState` remains
the runtime facade used by historical execution, runtime built-ins, strategy
variable reads, and public result projection. Broker internals are split under
`pine-runtime::strategy::broker`: pending-exit identity, trigger conversion,
single-trigger and bracket placement live in `exits`; pending-exit evaluation,
including stop/loss-first bracket both-hit selection, stays in the broker
facade; close/fill trade construction and position reset live in `fills`;
equity, profit, position, and trade-count accessors live in `accounting`;
broker-focused unit tests live in `tests`. Public strategy result structs
remain in `pine-runtime::output::strategy`, and host bindings continue to map
the shared runtime result without owning broker transitions. Phase R implements
the first one-downside/one-upside bracket subset inside these ownership
boundaries; it does not move broker behavior into built-in signatures, output
structs, Python, or WASM.

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

program = compile_script(
    source,
    library_sources={"user/lib/1": 'library("lib")\n'},
)
result = program.run(bars, request_bars={"NYSE:IBM:1": requested_bars})
```

CLI:

```bash
pine-compat analyze script.pine --library-source user/lib/1=lib.pine
pine-compat run script.pine --bars bars.csv \
  --library-source user/lib/1=lib.pine \
  --request-bars NYSE:IBM:1=ibm.csv
```

The WASM API exposes deterministic JSON library source injection through
`compileScriptWithLibraries`, `analyzeScriptWithLibraries`, and
`runScriptCsvWithLibraries`. The JSON value must be an object mapping import
keys to source text; malformed JSON is reported as a host-input diagnostic from
the binding layer before semantic analysis.
WASM request data injection is exposed through `runScriptCsvWithRequestBars`,
`runScriptCsvWithLibrariesAndRequestBars`, and
`Program.runCsvWithRequestBars`. The `requestBarsJson` value is an object
mapping `SYMBOL:TIMEFRAME` keys to arrays of `{time, open, high, low, close,
volume}` bar objects. This is explicit host-provided data; the WASM crate does
not fetch network data, read files, or discover symbols.

## Output Model

The core output must remain host-neutral:

```json
{
  "schemaVersion": 5,
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
message, source}` for the narrow `alertcondition` and `alert` subsets. For
`alertcondition`, `source` is the const title; for `alert`, `source` is
`alert`. `schemaVersion: 4` adds broker-owned strategy order-fill alert
payloads under `strategy.alerts` without changing the top-level `alerts[]`
callsite event shape. `schemaVersion: 5` adds host-neutral table cell
`textWrap` snapshots. Host integrations can adapt this model into their
charting or API format, but should preserve the runtime schema version when
they forward machine-readable runtime results.

Machine-readable analysis and matrix outputs use separate schema ownership:
`PUBLIC_ANALYSIS_SCHEMA_VERSION` for WASM/Python analysis reports and
`PUBLIC_MATRIX_SCHEMA_VERSION` for CLI matrix JSON. Runtime is currently `5`;
analysis and matrix remain `2`. These contracts can evolve independently when a
runtime-only output field does not affect analysis or matrix contracts.

Drawing-object outputs use sparse snapshot families. The Phase E drawing
contract reserves `labels`, `lines`, `boxes`, and `tables`, whose entries have
an object `id` and a `snapshots` array. Label snapshots use `barIndex`,
`exists`, and, while `exists` is true, the mutable label fields represented by
normalized Pine values, including `textAlign`, `textFontFamily`, and
`textFormatting` for host-side text layout. The label lifecycle covers
`label.new`, selected
`label.set_*` mutators including x-location and y-location snapshot mutation,
`label.copy` cloning, and `label.delete`. Line snapshots cover `x1`, `y1`,
`x2`, `y2`, `color`, `width`, `style`, and `extend`. `line.new` can initialize
those host-neutral style snapshot fields for the x1/y1/x2/y2 overload when
`xloc` is omitted or `xloc.bar_index`; chart-point overloads and
`xloc.bar_time` coordinate semantics remain outside the current runtime output
contract. Selected `line.set_*` mutators, including the `xloc.bar_index`
`line.set_xloc` subset that rewrites x1 and x2, `line.copy` cloning, and
`line.delete` reuse the same snapshot model; `line.get_x1`, `line.get_y1`,
`line.get_x2`, and `line.get_y2` read latest existing line snapshot values;
`line.get_price` derives a host-neutral bar-index price by interpolating or
extrapolating across the latest existing x1/y1/x2/y2 snapshot. Box snapshots
cover `left`, `top`,
`right`, `bottom`, `bgColor`, `borderColor`, `borderWidth`, `borderStyle`,
`extend`, `text`, `textColor`, `textSize`, `textHalign`, `textValign`,
`textWrap`, `textFontFamily`, and `textFormatting`. `box.new` can initialize
those host-neutral style and text snapshot fields for the left/top/right/bottom
overload when `xloc` is omitted or `xloc.bar_index`; chart-point overloads and
`xloc.bar_time` coordinate semantics remain outside the current runtime output
contract. Selected `box.set_*` mutators, including the `xloc.bar_index`
`box.set_xloc` subset that rewrites left and right, `box.copy` cloning, and
`box.delete` reuse the same snapshot model;
`box.get_left`, `box.get_right`, `box.get_top`, and `box.get_bottom` read latest
existing snapshot values.
Table entries
carry `position`, `bgColor`, `frameColor`, `frameWidth`, `borderColor`,
`borderWidth`, `columns`, `rows`, and sparse cell snapshots. Each table snapshot
carries `exists`; existing table snapshots store cells whose entries carry
`column`, `row`, `text`, `bgColor`, `textColor`, `width`, `height`, `textSize`,
`textHalign`, `textValign`, `textWrap`, `tooltip`, `textFontFamily`, and
`textFormatting`, avoiding host-specific table layout assumptions;
`table.new` may initialize the final background color, frame color, frame
width, border color, and border width through its optional `bgcolor`,
`frame_color`, `frame_width`, `border_color`, and `border_width` arguments;
`table.set_position` updates the table's final position, including when called
from ordinary control-flow blocks, `table.set_bgcolor` updates the table's final
background color, including when called from ordinary control-flow blocks,
`table.set_frame_color` updates the table's final frame color, including when
called from ordinary control-flow blocks,
`table.set_frame_width` updates the table's final frame width, including when
called from ordinary control-flow blocks,
`table.set_border_color` updates the table's final border color, including when
called from ordinary control-flow blocks,
`table.set_border_width` updates the table's final border width, including when
called from ordinary control-flow blocks, `table.delete` records an
`exists: false` snapshot, including when called from ordinary control-flow
blocks, while `table.cell_set_text`, `table.cell_set_bgcolor`,
`table.cell_set_text_color`, `table.cell_set_width`, and
`table.cell_set_height`, including when those setters are called from ordinary
control-flow blocks, plus `table.cell_set_text_size`/
`table.cell_set_text_halign`/`table.cell_set_text_valign`/
`table.cell_set_text_wrap`/`table.cell_set_tooltip`/
`table.cell_set_text_font_family`/`table.cell_set_text_formatting` mutate only
the stored text/background/text color/width/height/text size/text
alignment/text wrap/tooltip/font-family/text-formatting for cells already
populated by `table.cell`, `table.clear` removes populated cells in an
inclusive rectangular range, including from ordinary control-flow blocks, and
removes merged-cell records intersecting that range, `table.merge_cells`
records inclusive host-neutral merge rectangles, including from ordinary
control-flow blocks, and `table.delete` appends a deleted snapshot.
`table.all` reads currently existing table ids in creation order, including
from ordinary control-flow blocks after deletion.
Delete calls append an `exists: false`
snapshot for families with deletion; deleting `na` or an already deleted
drawing object is a no-op; ids are not reused. The historical runtime caps
labels, lines, and boxes at 500 objects, caps tables at 50 objects, and caps a
single table at 1000 cells.
Drawing-object method-call syntax is normalized before runtime. For supported
label, line, box, and table id-first functions, semantic analysis validates the
receiver type and lowering rewrites calls such as `id.set_text("x")` to the
same HIR callee and argument list as `label.set_text(id, "x")`. Runtime modules
therefore execute one canonical namespace-call path.

Alert events are flat runtime events rather than sparse snapshots. Historical
execution appends events in program order when an `alertcondition` call is
reached and true, or when an `alert` call is reached. Realtime forming events
live in the forming runtime snapshot and are discarded on rollback unless a
confirmed update emits the same event. Forming `RuntimeResult` values expose
the currently recomputed forming events, but `confirmed_result()` only exposes
events committed by historical or confirmed updates.

The `pine-runtime` crate owns the shared runtime-result JSON helpers used by the
CLI and WASM bindings. Python keeps explicit dictionary conversion code because
it returns native Python objects, but its top-level runtime result keys are
tested against the same public contract. Analysis reports and compatibility
matrix JSON keep distinct schema constants even where their current field shapes
remain host-owned.
