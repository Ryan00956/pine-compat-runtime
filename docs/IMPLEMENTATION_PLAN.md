# Implementation Plan

This plan favors a serious long-term foundation over quick partial execution.

## Phase 0: Repository Foundation

Deliverables:

- Rust workspace.
- README and architecture docs.
- License and contribution guidelines.
- Formatting and linting configuration.
- Basic CI for format, clippy, and tests.

Exit criteria:

- `cargo fmt --check`, `cargo clippy`, and `cargo test` run in CI.

## Phase 1: Syntax

Deliverables:

- Source file abstraction.
- Tokenizer.
- AST.
- Parser for indicator declarations, variable declarations, assignments,
  expressions, function calls, named arguments, blocks, and history references.
- Syntax diagnostics with line and column spans.
- Golden parser tests.

Exit criteria:

- Parser handles a curated fixture set of common indicator scripts.
- Invalid syntax produces useful diagnostics without panics.
- Parser may accept syntax that later phases reject as unsupported.

## Phase 2: Semantic Analysis and Compatibility Gating

Deliverables:

- Scope resolver.
- Type and qualifier model.
- Name resolution for built-ins.
- Unsupported feature detection.
- AST to HIR lowering.
- Compatibility report.
- Built-in signature registry.
- Callsite and series id assignment for executable expressions.

Exit criteria:

- Analyzer can distinguish supported scripts from scripts requiring
  `strategy.*`, `request.*`, labels, arrays, imports, or realtime-only behavior.
- Analyzer accepts only the Phase 1 executable subset for runtime execution.
- Analyzer emits stable diagnostic codes for unsupported features.

## Phase 3: Minimal Historical Runtime

Deliverables:

- Bar data model.
- Runtime context.
- Series store.
- Persistent state store for `var`.
- Expression evaluator.
- Statement evaluator.
- `na` semantics.
- History reference semantics.
- Plot, hline, and fill output collection for the minimal subset.

Exit criteria:

- Runtime executes global-scope indicator scripts over historical OHLCV
  fixtures.
- Results are deterministic and snapshot tested.
- Constant history offsets, `var`, `na`, `nz`, `plot`, `hline`, `fill`,
  `ta.sma`, and `ta.ema` are covered by fixtures.

## Phase 4: Expanded Built-Ins and Output

Deliverables:

- `ta.*` built-ins for common indicators.
- Normalized JSON result model.
- CLI `analyze` and `run`.
- Tuple lowering for tuple-returning built-ins.

Exit criteria:

- CLI can run SMA, EMA, RSI, MACD, Bollinger Bands, and ATR fixtures.
- Output includes series, annotations, fills, inputs, diagnostics, and
  compatibility report.

## Phase 5: Python Binding

Deliverables:

- PyO3 binding.
- maturin build configuration.
- Python package API.
- Python tests.

Exit criteria:

- `pip install -e .` or `maturin develop` exposes `compile_script`,
  `analyze_script`, and `run_script`.

## Phase 6: Performance and Incremental Runtime

Deliverables:

- Compile cache.
- Runtime allocation profiling.
- Optimized rolling TA functions.
- Optional bytecode VM.
- Incremental append-bar execution.

Current compile cache API lives in `pine-sema` as `CompileCache`. It caches
complete analysis results by source name and source text so CLI, Python, and
future host integrations can avoid repeated parse and semantic passes for
unchanged scripts.

Runtime storage profiling is exposed by `pine-runtime::run_historical_profiled`
and by the CLI:

```text
pine-compat run script.pine --bars bars.csv --profile
```

The profile reports allocation-sensitive storage lengths and capacities for
series buffers, plot values, and runtime state maps. It is a portable storage
profile, not a process-wide allocator hook.

Rolling TA state is optimized for `ta.sma`, `ta.bb`, `ta.highest`, and
`ta.lowest`. These functions now maintain per-callsite rolling windows instead
of allocating a fresh window from committed history on every bar.

Append-bar execution is exposed by `pine-runtime::HistoricalRuntime`:

```rust
let mut runtime = HistoricalRuntime::new(&hir);
runtime.append_bar(bar)?;
let partial = runtime.result();
```

The existing `run_historical` helper is implemented on top of this stateful
runtime, so full historical execution and incremental append execution share the
same code path.

Runtime fixtures are also checked through an integration test that compares
full historical execution with incremental `append_bar` execution for every
script in `tests/fixtures/runtime`.

The optional bytecode VM was evaluated and deferred. Direct HIR execution
remains the reference runtime until MIR exists and profiling shows instruction
dispatch is a real bottleneck. See
[`BYTECODE_VM_EVALUATION.md`](BYTECODE_VM_EVALUATION.md).

Exit criteria:

- Re-running a compiled program over large historical data is acceptably fast.
- Incremental update path matches full recomputation for supported fixtures.

## Phase 7: Realtime and Wider Compatibility

Deliverables:

- Forming bar support.
- Rollback model.
- `varip` semantics.
- Expanded language features.
- Optional WASM binding.

Exit criteria:

- Realtime fixture tests define and verify repeated updates on the same bar.

## Development Rules

- Add fixtures before or alongside behavior.
- Unsupported features must produce diagnostics.
- Avoid host-specific assumptions in core crates.
- Do not use TradingView source, private APIs, or proprietary data.
- Keep public APIs small until semantics are stable.
