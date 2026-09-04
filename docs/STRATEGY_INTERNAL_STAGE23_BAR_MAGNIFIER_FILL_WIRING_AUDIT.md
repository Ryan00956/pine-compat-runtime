# Strategy Internal Stage 23 Bar Magnifier Fill Wiring Audit

Status: closed on 2026-09-04 after `scripts/verify.sh`. Named const bool
`use_bar_magnifier` is accepted for v5/v6 historical fill wiring. Host-owned
MagnifierInputV1 lower-timeframe groups walk the existing Stage 18g path and
unified broker selector. Public RuntimeResult schemaVersion remains 8.
Python `REALTIME_SESSION_SCHEMA_VERSION` remains 1. Mixed-family OCA is the
next strategy target.

Official review date: 2026-09-04.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/
https://www.tradingview.com/pine-script-docs/language/declaration-statements/

Working branch: `codex/strategy-stage23-bar-magnifier`.
Plan-baseline commit: `ad0a06fde`.
Enablement commit: `7b88b2432`.
Behavior lock: `docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_BEHAVIOR_AUDIT.md`.

## Behavior

- Named v5/v6 `strategy(..., use_bar_magnifier = <const bool>)` is accepted.
  Positional arguments, series/non-bool values, and Pine v1-v4 stay rejected.
- Host input is MagnifierInputV1: `schemaVersion` 1 and zero-based
  `chartBars` groups. CLI `--magnifier-bars`, Python `magnifier_bars`, and
  WASM `$magnifier` share `magnifier_input_from_json`.
- When `use_bar_magnifier=true` and a chart bar has a validated lower-bar
  group, the scheduler walks those bars in time order. Each lower bar uses
  `HistoricalPath::from_validated_bar`. All order families compete through
  the existing one-candidate broker.
- Public fill, order, trade, and alert `bar_index` is the chart-bar index.
  The public event timestamp is the chart-bar time. Lower-bar identity is
  internal cursor state only.
- The first tradable open of a covered chart bar is the first lower-bar open.
  A gap between one lower bar's close and the next lower bar's open is a
  point event at the next open, not a tradable close-to-open segment.
- `calc_on_order_fills` extra passes resume from the unconsumed
  `{host_bar_index, path_phase, leg_index, mark}` cursor and do not replay
  consumed marks. Script-visible OHLC, `time`, and `bar_index` stay
  chart-scoped.
- Missing groups emit `W_MAGNIFIER_FALLBACK`. Empty groups emit
  `W_MAGNIFIER_GAP`. Both fall back to that chart bar's standard OHLC path,
  at most once per affected chart bar.
- Invalid input fails closed before bar-zero execution:
  `E_MAGNIFIER_DUPLICATE_CHART_BAR`, `E_MAGNIFIER_DUPLICATE_TICK`,
  `E_MAGNIFIER_UNSORTED_TICKS`, `E_MAGNIFIER_MAX_INTRABARS`,
  `E_MAGNIFIER_INVALID_BAR`, `E_MAGNIFIER_CHART_BAR_RANGE`,
  `E_MAGNIFIER_SCHEMA_VERSION`, `E_MAGNIFIER_MALFORMED`,
  `E_MAGNIFIER_FORMING_BAR`.
- Setting false or omitted leaves supplied magnifier input inert. Indicator
  scripts ignore the input for strategy fills.
- Forming/live realtime bars never consume historical magnifier groups.
  `calc_on_every_history_tick` remains unimplemented and rejected.
- Batch, incremental, and historical realtime-seed results agree for the
  same magnified history.

## Named Runtime Goldens

- `runtime_strategy_use_bar_magnifier_fallback.json`
- `runtime_strategy_use_bar_magnifier_false.json`
- `matrix.json` (named const bool `use_bar_magnifier` accepted; fallback and
  false fixtures registered)

The fallback golden keeps standard-OHLC fills and records one
`W_MAGNIFIER_FALLBACK` diagnostic per chart bar. The false golden matches
those fills with empty diagnostics. CLI, Python, and WASM also prove a
lower-bar gap fill at the next open (`11.0`) against the standard-OHLC fill
at the stop (`10.5`), with public `barIndex` 1 and public `time` 2000.

## Incremental / Realtime

`magnifier_batch_matches_incremental_append` and
`magnifier_historical_realtime_replay_matches_batch` compare the same
entry/exit fixture. Forming updates do not consume historical magnifier
input (`E_MAGNIFIER_FORMING_BAR` for a forming-slot group;
`realtime_forming_does_not_consume_historical_magnifier_input`). Python
RealtimeSession keeps ABI version 1 and accepts optional seed-only
`magnifier_bars`.

## Files

- `crates/pine-ir/src/strategy.rs`
- `crates/pine-builtins/src/namespaces/core.rs`
- `crates/pine-sema/src/analyzer/strategy/declaration.rs`
- `crates/pine-runtime/src/magnifier.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/runtime/realtime.rs`
- `crates/pine-runtime/src/runtime/strategy_path.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-cli/src/commands/run.rs`
- `crates/pine-python/src/lib.rs`
- `crates/pine-python/src/realtime.rs`
- `crates/pine-wasm/src/request_bars.rs`
- `crates/pine-wasm/src/run.rs`
- `tests/fixtures/sema/supported_strategy_use_bar_magnifier.pine`
- `tests/fixtures/sema/supported_strategy_use_bar_magnifier_v6.pine`
- `tests/fixtures/runtime/strategy_use_bar_magnifier_fallback.pine`
- `tests/fixtures/runtime/strategy_use_bar_magnifier_false.pine`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/runtime_strategy_use_bar_magnifier_fallback.json`
- `tests/snapshots/runtime_strategy_use_bar_magnifier_false.json`
- `tests/snapshots/matrix.json`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/CONFORMANCE.md`
- `docs/DIAGNOSTIC_CODES.md`
- `docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`
- `docs/PURE_INTERNAL_ROADMAP.md`
- `docs/README.md`
- `docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_BEHAVIOR_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_EXECUTION_PLAN.md`

## Commands

Focused 23.7/23.8 evidence, saved under `{SCRATCH}` when present:

- `cargo test -p pine-sema use_bar_magnifier`: 11 passed
- `cargo test -p pine-sema accepts_strategy_process_orders_on_close_with_bar_magnifier`: 1 passed
- `cargo test -p pine-runtime magnifier -- --test-threads=1`: 28 passed
- `cargo test -p pine-runtime use_bar_magnifier_true_is_accepted -- --test-threads=1`: 1 passed
- `cargo test -p pine-runtime --test incremental magnifier_batch -- --test-threads=1`: 1 passed
- `cargo test -p pine-runtime --test realtime magnifier_historical -- --test-threads=1`: 1 passed
- `cargo test -p pine-cli magnifier`: 2 passed, including the lower-bar gap
  fill at public price `11.0`
- `cargo test -p pine-cli runtime_outputs_match_golden_snapshots`: 1 passed
- `cargo test -p pine-cli matrix_output_matches_golden_snapshot`: 1 passed
- `cargo test -p pine-wasm magnifier`: 4 passed
- `cargo fmt --check`: clean
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `python3 scripts/check_structure.py`: 311 production files
- `python3 scripts/check_host_parity.py`: 846 registered CLI runtime
  snapshots; 550 required runtime goldens
- `git diff --check`: clean

Close-out:

`scripts/verify.sh` EXIT:0. Workspace tests include pine-runtime lib 1693,
pine-sema 1229+2227, pine-cli 223, pine-wasm 657. Python 632 passed. Host
parity 846 registered CLI runtime snapshots and 550 required runtime
goldens. WASM Node smoke passed. Log: `{SCRATCH}/stage23-verify.sh.log`.

## Remaining Exclusions

- `fill_orders_on_standard_ohlc` stays rejected
- `calc_on_every_history_tick` stays unimplemented and rejected
- positional `use_bar_magnifier` and Pine v1-v4 stay rejected
- automatic lower-timeframe selection and network fetch stay out of scope
- forming/live realtime bars do not consume historical magnifier groups
- mixed-family OCA is the next strategy target
- session calendars and the general chart-to-chart inter-bar gap rewrite stay
  later, separate stages
- public RuntimeResult / StrategyResult fields are unchanged
- no release, version bump, tag, or package publish
