# Pine Compat Runtime

Pine Compat Runtime is a planned clean-room, open-source runtime for a
Pine-compatible indicator scripting subset.

The project is intentionally designed as an embeddable language runtime, not as
an application-specific plugin. Hosts such as charting tools, research
notebooks, command line workflows, and CandleScope-style applications should be
able to integrate it through adapters.

## Goals

- Implement a clean-room Pine-compatible indicator runtime.
- Prioritize semantic correctness over early breadth.
- Support bar-by-bar time-series execution, historical references, `na`, `var`,
  inputs, and plotting side effects.
- Expose stable Rust, CLI, Python, and eventually WASM entry points.
- Produce a host-neutral output model that charting applications can adapt.
- Provide precise diagnostics and compatibility reports instead of silent
  partial execution.

## Non-Goals

- This is not affiliated with, endorsed by, or sponsored by TradingView.
- This is not a copy of TradingView's compiler, runtime, services, data, UI, or
  private APIs.
- The first releases will not attempt full Pine Script compatibility.
- Strategy backtesting, multi-timeframe data requests, object drawing systems,
  alerts, and libraries are out of scope for the initial runtime.

## Design Documents

- [Architecture](docs/ARCHITECTURE.md)
- [Language Scope](docs/LANGUAGE_SCOPE.md)
- [Execution Semantics](docs/EXECUTION_SEMANTICS.md)
- [Semantic Model](docs/SEMANTIC_MODEL.md)
- [Series Model](docs/SERIES_MODEL.md)
- [Built-In Signatures](docs/BUILTIN_SIGNATURES.md)
- [Conformance](docs/CONFORMANCE.md)
- [Diagnostic Codes](docs/DIAGNOSTIC_CODES.md)
- [Task Breakdown](docs/TASK_BREAKDOWN.md)
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Compatibility, Legal, and Branding Boundaries](docs/COMPATIBILITY_AND_LEGAL.md)

## Planned Package Layout

```text
pine-compat-runtime/
  crates/
    pine-syntax/       lexer, parser, AST, source spans, diagnostics
    pine-sema/         scope resolution, type and qualifier analysis
    pine-ir/           HIR, MIR, and bytecode definitions
    pine-runtime/      bar-by-bar VM, series store, state store
    pine-builtins/     ta, math, input, plot, color, time
    pine-cli/          command line runner and analyzer
    pine-python/       PyO3 and maturin Python bindings
    pine-wasm/         browser and host WASM bindings
  tests/
    fixtures/
    snapshots/
    conformance/
  docs/
```

## First Milestone

The first milestone is a Rust CLI that can parse, analyze, and execute a small
set of common indicator scripts over CSV OHLCV data, then emit normalized JSON
containing series, annotations, fills, inputs, and diagnostics.

The project should not move into host-specific integration work until this
standalone loop is reliable:

```text
source.pine + bars.csv + inputs.json
  -> compile
  -> analyze
  -> run
  -> result.json
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

Build locally with:

```text
maturin develop
```

## WASM Binding

The optional WASM crate exposes a thin `wasm-bindgen` API:

- `compileScript(source)`
- `analyzeScript(source)`
- `runScriptCsv(source, barsCsv)`
- `Program.runCsv(barsCsv)`

Build-check it with:

```text
rustup target add wasm32-unknown-unknown
cargo check -p pine-wasm --target wasm32-unknown-unknown
```
