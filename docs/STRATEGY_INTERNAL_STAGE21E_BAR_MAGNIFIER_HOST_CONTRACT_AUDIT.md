# Strategy Internal Stage 21e Bar Magnifier Host Contract Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Host-owned
lower-timeframe bars are keyed by chart bar index. Absence and gaps fall back
to the chart bar's standard OHLC path. Duplicate ticks, unsorted timestamps,
duplicate chart-bar keys, and more than 200000 intrabars fail closed. The
existing Stage 18 fill-step path is the tick consumer; no second broker was
added. `use_bar_magnifier` stays rejected. CLI/Python/WASM input parity is
deferred until this host-neutral schema is used by a fill-wiring slice.
Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- `MagnifierInput` stores `Vec<Bar>` per chart bar index.
- Missing or empty data for a chart bar selects `MagnifierFallback::StandardOhlc`
  and emits `W_MAGNIFIER_FALLBACK` or `W_MAGNIFIER_GAP`.
- Invalid host data emits `E_MAGNIFIER_DUPLICATE_CHART_BAR`,
  `E_MAGNIFIER_DUPLICATE_TICK`, `E_MAGNIFIER_UNSORTED_TICKS`, or
  `E_MAGNIFIER_MAX_INTRABARS`.
- `magnifier_host_ticks` returns either validated intrabars or the chart bar.
  Later fill wiring must run `HistoricalFillStep` against each returned tick.
- `strategy(..., use_bar_magnifier=true)` remains a semantic rejection.

## Files

- `crates/pine-runtime/src/magnifier.rs`
- `crates/pine-runtime/src/lib.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `tests/fixtures/sema/unsupported_strategy_use_bar_magnifier.pine`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/DIAGNOSTIC_CODES.md`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 661/106 passed, saved as
`{SCRATCH}/stage21e-baseline-1.log`, `{SCRATCH}/stage21e-baseline-2.log`,
`{SCRATCH}/stage21e-sema-baseline-1.log`, and
`{SCRATCH}/stage21e-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime magnifier` 9 passed.
`cargo test -p pine-runtime strategy` 662 passed.
`cargo test -p pine-sema strategy` 107 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 599 passed. Host parity 523 required
runtime goldens. WASM 628 passed. Log: `{SCRATCH}/stage21e-verify.sh.log`.

## Remaining Exclusions

Fill wiring for `use_bar_magnifier=true` is later work. CLI/Python/WASM
magnifier input APIs wait on that wiring. `fill_orders_on_standard_ohlc`
stays rejected. Stage 22 strategy risk rules may begin after this Stage 21
closeout. Public `StrategyResult` stays unchanged.
