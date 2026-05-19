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
  inputs, and plotting side effects.
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

## Current Package Layout

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

## Current Baseline

The current baseline is a Rust CLI and embeddable runtime that can parse,
analyze, and execute a small set of common indicator scripts over CSV OHLCV
data, then emit normalized JSON containing series, annotations, fills, inputs,
and diagnostics.

The project should not move into host-specific integration work until this
standalone loop is reliable:

```text
source.pine + bars.csv + inputs.json
  -> compile
  -> analyze
  -> run
  -> result.json
```

The supported executable subset includes indicator scripts, historical
bar-by-bar execution, constant history offsets, `if`/`else` blocks, `var`,
normal block-local declarations inside `if`, `na`, `nz`, `input.*`, `plot`,
`hline`, `fill`, common `ta.*` functions, selected `math.*` functions,
user-defined functions, named colors, `color.new`, tuple
returns, incremental append execution, realtime forming-bar rollback, Python
bindings, and a thin WASM binding.

The runtime intentionally rejects unsupported features such as `strategy.*`,
`request.*`, alerts, imports, arrays, drawing objects, dynamic history offsets,
recursive functions, function side effects, and `varip` intrabar persistence.
Stateful calls inside `if` blocks advance their callsite state only when the
branch executes; skipped bars commit `na` for series values that were not
evaluated on that bar.

## CLI

Run a script against CSV bars:

```text
cargo run -p pine-cli -- run tests/fixtures/runtime/macd.pine --bars tests/fixtures/runtime/bars.csv
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

Build locally in an active virtual environment with:

```text
maturin develop --manifest-path crates/pine-python/Cargo.toml
python -m pytest python/tests
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

## Development Verification

Run the core Rust and WASM checks before publishing changes:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p pine-wasm --target wasm32-unknown-unknown
```

Python binding tests require the extension module to be installed into an
active Python environment:

```text
python -m pip install --upgrade pip maturin pytest
maturin develop --manifest-path crates/pine-python/Cargo.toml
python -m pytest python/tests
```
