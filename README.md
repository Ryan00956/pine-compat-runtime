# Pine Compat Runtime

Pine Compat Runtime is a clean-room, open-source runtime for a Pine-compatible
indicator scripting subset.

The project is intentionally designed as an embeddable language runtime, not as
an application-specific plugin. Hosts such as charting tools, research
notebooks, command line workflows, and CandleScope-style applications should be
able to integrate it through adapters.

## Goals

- Implement a clean-room Pine-compatible indicator runtime.
- Prioritize semantic correctness over early breadth.
- Support bar-by-bar time-series execution, historical references, `na`, `var`,
  inputs, plotting and selected drawing side effects, and fixture-backed
  request data.
- Expose stable Rust, CLI, Python, and WASM entry points for the supported
  subset.
- Produce a host-neutral output model that charting applications can adapt.
- Provide precise diagnostics and compatibility reports instead of silent
  partial execution.

## Non-Goals

- This is not affiliated with, endorsed by, or sponsored by TradingView.
- This is not a copy of TradingView's compiler, runtime, services, data, UI, or
  private APIs.
- The first releases will not attempt full Pine Script compatibility.
- Strategy backtesting, broad request families, advanced drawing systems,
  host-delivered alert services, and full remote library registry behavior are
  out of scope for the initial runtime.

## Design Documents

- [Architecture](docs/ARCHITECTURE.md)
- [Language Scope](docs/LANGUAGE_SCOPE.md)
- [Execution Semantics](docs/EXECUTION_SEMANTICS.md)
- [Semantic Model](docs/SEMANTIC_MODEL.md)
- [Series Model](docs/SERIES_MODEL.md)
- [Built-In Signatures](docs/BUILTIN_SIGNATURES.md)
- [Conformance](docs/CONFORMANCE.md)
- [Diagnostic Codes](docs/DIAGNOSTIC_CODES.md)
- [Release Notes](docs/RELEASE_NOTES.md)
- [Phase I Execution Plan](docs/PHASE_I_EXECUTION_PLAN.md)
- [Phase J Execution Plan](docs/PHASE_J_EXECUTION_PLAN.md)
- [Phase J Libraries/User Types Audit](docs/PHASE_J_AUDIT.md)
- [Phase K Execution Plan](docs/PHASE_K_EXECUTION_PLAN.md)
- [Phase L Strategy Usability Execution Plan](docs/PHASE_L_EXECUTION_PLAN.md)
- [Phase L Strategy Usability Audit](docs/PHASE_L_AUDIT.md)
- [Phase M Strategy Exit Execution Plan](docs/PHASE_M_EXECUTION_PLAN.md)
- [Phase M Strategy Exit Audit](docs/PHASE_M_AUDIT.md)
- [Phase R Strategy Exit Bracket Execution Plan](docs/PHASE_R_EXECUTION_PLAN.md)
- [Phase R Strategy Exit Bracket Audit](docs/PHASE_R_AUDIT.md)
- [Phase S Strategy Exit Trailing Stop Execution Plan](docs/PHASE_S_EXECUTION_PLAN.md)
- [Phase T WASM Request Provider Execution Plan](docs/PHASE_T_EXECUTION_PLAN.md)
- [Phase T WASM Request Provider Audit](docs/PHASE_T_AUDIT.md)
- [Phase F Request Platform Audit](docs/PHASE_F_AUDIT.md)
- [Phase H Alert Audit](docs/PHASE_H_AUDIT.md)
- [Next Language Expansion Playbook](docs/NEXT_LANGUAGE_EXPANSION_PLAYBOOK.md)
- [Next Internal Capability Plan](docs/NEXT_INTERNAL_CAPABILITY_PLAN.md)
- [Strategy Internal Gap Audit](docs/STRATEGY_INTERNAL_GAP_AUDIT.md)
- [Task Breakdown](docs/TASK_BREAKDOWN.md)
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Compatibility, Legal, and Branding Boundaries](docs/COMPATIBILITY_AND_LEGAL.md)

## Current Package Layout

```text
pine-compat-runtime/
  crates/
    pine-syntax/       lexer, parser, AST, source spans, diagnostics
    pine-sema/         scope resolution, type and qualifier analysis
    pine-ir/           HIR, MIR, and bytecode definitions
    pine-runtime/      bar-by-bar VM, series store, state store, request data
    pine-builtins/     ta, math, input, plot, color, time, request, alert
    pine-cli/          command line runner and analyzer
    pine-python/       PyO3 and maturin Python bindings
    pine-wasm/         browser and host WASM bindings
  tests/
    fixtures/         runtime, sema, syntax, request, profile, realtime, etc.
      conformance.tsv executable subset inventory
    snapshots/        generated matrix snapshots
  docs/
```

## Current Baseline

The current baseline is a Rust CLI and embeddable runtime that can parse,
analyze, and execute a small set of common indicator scripts over CSV OHLCV
data, then emit normalized JSON containing series, annotations, fills,
diagnostics, and compatibility reports.

The project should not move into host-specific integration work until this
standalone loop is reliable:

```text
source.pine + bars.csv
  -> compile
  -> analyze
  -> run
  -> result.json
```

The supported executable subset includes indicator scripts, historical
bar-by-bar execution, constant and guarded dynamic integer history offsets,
`if`/`else` blocks, `switch`, partial `for`/`while` loops, `var`, block-local
declarations, `na`, `nz`, `input.*` defval execution, output calls, selected
drawing objects, partial typed arrays, common `ta.*` functions, selected
`math.*` and `str.*` functions, partial `request.security`, user-defined
functions, local scalar-field user-defined types, pure local UDT methods, named
colors, color helpers, tuple returns, scalar and scalar typed-array `varip`,
partial `alertcondition`/`alert` runtime events, host-provided exact-key
imports for exported const expressions and pure exported functions,
incremental append execution, realtime forming-bar rollback, partial
strategy-mode long entries, closes, stop/limit/profit/loss exits, the first
one-downside/one-upside `strategy.exit` bracket subset, the first trailing-stop
`strategy.exit` subset, optional fixed-quantity and percent-quantity partial
exits on those supported exit shapes, explicit fixed-quantity or
percent-quantity single-trigger, bracket, and trailing multiple-exit
reservations, Python bindings, and a thin WASM binding.

The runtime intentionally rejects unsupported features such as strategy order
families beyond the current `strategy.entry`/`strategy.close`/
`strategy.close_all`/`strategy.cancel`/`strategy.cancel_all`/`strategy.exit`
subset, same-side, 3+ trigger, or invalid trailing strategy exits, request
variants outside the narrow `request.security` subset, multiple pending exits
outside explicit fixed-quantity or percent-quantity single-trigger, bracket,
and trailing `strategy.exit` reservations, including omitted-quantity multiple
reservations, reservation behavior outside that subset, missing-entry future
binding beyond the supported active-entry attachment subset, alert frequency
modes and placeholder interpolation, remote library lookup, re-exports,
imported UDTs,
imported methods, side-effecting exported library functions, advanced drawing
families and methods, unsupported collection families and element types, recursive
functions, function side effects, and unsupported `varip` value families such
as drawing ids and tuples.
Stateful calls inside `if` blocks advance their callsite state only when the
branch executes; skipped bars commit `na` for series values that were not
evaluated on that bar.

## CLI

Run a script against CSV bars:

```text
cargo run -p pine-cli -- run tests/fixtures/runtime/macd.pine --bars tests/fixtures/runtime/bars.csv
```

Library source text can be passed as Phase J graph input for the exact-key
import subset:

```text
cargo run -p pine-cli -- run script.pine --bars bars.csv --library-source user/lib/1=lib.pine
```

Print the compatibility matrix:

```text
cargo run -p pine-cli -- matrix
cargo run -p pine-cli -- matrix --format json
```

## Python Binding

The Python module is exposed as `pine_compat` through PyO3 and maturin:

```python
import pine_compat

program = pine_compat.compile_script('indicator("demo")\nplot(close)\n')
result = program.run([
    {"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0},
])
```

Python can also pass Phase J library source text:

```python
report = pine_compat.analyze_script(
    'indicator("demo")\nplot(close)\n',
    library_sources={"user/lib/1": 'library("lib")\n'},
)
```

Build locally in an active virtual environment with:

```text
maturin develop --manifest-path crates/pine-python/Cargo.toml
python -m pytest python/tests
```

## WASM Binding

The optional WASM crate exposes a thin `wasm-bindgen` API:

- `compileScript(source)`
- `compileScriptWithLibraries(source, librarySourcesJson)`
- `analyzeScript(source)`
- `analyzeScriptWithLibraries(source, librarySourcesJson)`
- `runScriptCsv(source, barsCsv)`
- `runScriptCsvWithRequestBars(source, barsCsv, requestBarsJson)`
- `runScriptCsvWithLibraries(source, barsCsv, librarySourcesJson)`
- `runScriptCsvWithLibrariesAndRequestBars(source, barsCsv, librarySourcesJson, requestBarsJson)`
- `Program.runCsv(barsCsv)`
- `Program.runCsvWithRequestBars(barsCsv, requestBarsJson)`

The WASM library source argument is a deterministic JSON object mapping import
keys to source text, for example `{"user/lib/1":"library(\"lib\")\n"}`.
The request-bars argument is explicit host data injection, not network fetching
or symbol discovery. It must be a JSON object keyed by `SYMBOL:TIMEFRAME`, with
symbols split on the last colon so exchange-prefixed keys such as
`NYSE:IBM:1` are valid:

```json
{
  "NYSE:IBM:1": [
    {"time": 0, "open": 10, "high": 11, "low": 9, "close": 30, "volume": 100}
  ],
  "NYSE:IBM:5": [
    {"time": 300000, "open": 100, "high": 101, "low": 99, "close": 100, "volume": 500}
  ]
}
```

Build-check it with:

```text
rustup target add wasm32-unknown-unknown
cargo check -p pine-wasm --target wasm32-unknown-unknown
```

## Development Verification

Run the release verification entry point before publishing changes:

```text
scripts/verify.sh
```

This is the same canonical command list used by CI: Rust formatting, clippy,
workspace tests, the `wasm32-unknown-unknown` target check, Python wheel build,
wheel reinstall, and Python binding tests.

Prerequisites for the full gate:

```text
python3 -m pip install --upgrade pip maturin pytest
rustup target add wasm32-unknown-unknown
```

## Performance Profile Fixtures

Run the deterministic runtime profile fixtures with:

```text
cargo test -p pine-runtime --test profile_fixtures
```

These tests use existing `RuntimeProfile` metrics rather than wall-clock timing.
Hard failures cover severe growth in plot capacity, rolling-window storage,
array storage, and `max_bars_back` history retention. Runtime speed remains an
informational concern until a stable benchmark harness is added.
