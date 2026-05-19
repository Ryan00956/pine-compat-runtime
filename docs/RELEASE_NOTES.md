# Release Notes

## Unreleased

- Added partial `switch` expression support for condition arms, selector/case
  arms, expression results, default arms, and conditional stateful-call
  execution.

## v0.1 Baseline

This release establishes the first executable Pine-compatible indicator subset.
Compatibility claims are backed by `tests/fixtures/conformance.tsv`; run
`pine-compat matrix` or `pine-compat matrix --format json` to inspect the
feature-level matrix and its fixture paths.

### Runtime Surfaces

- Rust crates for syntax, semantic analysis, HIR, built-ins, runtime, CLI,
  Python bindings, and WASM bindings.
- CLI commands for analysis, AST formatting, historical execution, profiling,
  and compatibility matrix output.
- Python binding exposing compile, analyze, and run entry points.
- WASM binding exposing compile, analyze, and CSV execution entry points.

### Supported Executable Subset

- Indicator scripts over OHLCV bar input.
- Historical bar-by-bar execution and incremental append execution.
- Realtime forming-bar rollback for output, `var`, and stateful callsite state.
- Constant non-negative history references.
- Normal declarations, reassignment, tuple declarations, tuple-returning
  built-ins, and tuple `for` expression results.
- `if`/`else` blocks, nested blocks, and conditional stateful calls that advance
  only when their branch executes.
- `for` loops over inclusive integer ranges, explicit non-zero `by` steps,
  `break`, `continue`, scalar loop results, and tuple loop results.
- Local scopes for block declarations, tuple declarations, shadowing, and local
  `var` declaration-site storage.
- User-defined functions with expression bodies and multi-statement block
  bodies, positional and named arguments, single evaluation of arguments,
  function-local declarations, local `var`, loops inside functions, and
  independent state per syntactic callsite.
- `na`, `nz`, `indicator`, `input.*`, `plot`, `hline`, `fill`, `color.new`,
  selected named colors, selected `math.*` functions, and the fixture-covered
  `ta.*` built-ins listed in the compatibility matrix.

### Partial Support

- `for`: supports inclusive integer ranges, loop control, and loop results, but
  does not claim full Pine loop compatibility.
- `history references`: supports constant non-negative offsets only.
- `color.*` named constants: supports the current common registry only.
- `realtime forming rollback`: covers output, `var`, and callsite rollback;
  `varip` remains unsupported.

### Explicitly Unsupported

The analyzer rejects these boundaries with diagnostics instead of approximating
them silently:

- `varip` intrabar persistence.
- `request.*` multi-symbol and multi-timeframe data requests.
- `strategy.*` broker emulation and backtesting.
- Arrays and mutable collection storage.
- Imports and external libraries.
- Alerts and alert conditions.
- Drawing object systems such as labels, lines, boxes, tables, and polylines.
- Dynamic history offsets.
- Recursive user-defined functions.
- User-defined function side effects, including output calls, input
  declarations, indicator declarations, global reassignment, and passing
  side-effecting calls as UDF arguments.

### Verification

The release baseline is expected to pass:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p pine-wasm --target wasm32-unknown-unknown
maturin build --manifest-path crates/pine-python/Cargo.toml --out dist
python -m pip install --force-reinstall dist/*.whl
python -m pytest python/tests
```
