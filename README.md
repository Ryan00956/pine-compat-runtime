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
- [Release Notes](docs/RELEASE_NOTES.md)
- [Phase K Execution Plan](docs/PHASE_K_EXECUTION_PLAN.md)
- [Phase F Request Platform Audit](docs/PHASE_F_AUDIT.md)
- [Next Language Expansion Playbook](docs/NEXT_LANGUAGE_EXPANSION_PLAYBOOK.md)
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
    pine-builtins/     ta, math, input, plot, color, time, request
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
functions, named colors, color helpers, tuple returns, incremental append
execution, realtime forming-bar rollback, Python bindings, and a thin WASM
binding.

The runtime intentionally rejects unsupported features such as `strategy.*`,
request variants outside the narrow `request.security` subset, alerts, imports,
advanced drawing families and methods, unsupported collection families and
element types, recursive functions, function side effects, and `varip` intrabar
persistence.
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
