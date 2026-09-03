# Strategy Internal Stage 21a Execution-Pass Identity And Guardrails Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. The strategy scheduler
now tracks bar, fill-path tick, and pass identity, counts script passes on
internal runtime profiles, and rejects extra passes above a configurable
internal recalculation-pass limit. `calc_on_order_fills` and
`calc_on_every_tick` stay rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Each strategy bar starts a scheduler identity of bar index, current phase or
  fill-path tick, and pass `0`.
- Historical execution still runs one script pass per bar. Extra fill-triggered
  passes are not scheduled.
- Internal runtime profiles report `strategyScriptPasses`,
  `strategyRecalculationPasses`, `strategyMaxPassesOnBar`, and
  `strategyMaxRecalculationPasses`. Indicator runs report zeros.
- The default extra-pass cap is `1000` per bar. A simulated self-triggering
  loop that exceeds the configured cap fails with
  `strategy recalculation pass limit exceeded`.
- `BrokerState::snapshot` / `restore` clone broker state. Forming-bar realtime
  updates continue to clone the confirmed runtime, including the broker, so an
  abandoned forming fill does not leak into the confirmed result.
- `strategy(..., calc_on_order_fills=true)` remains a semantic rejection.

## Files

- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/profile.rs`
- `crates/pine-runtime/src/output/json/profile.rs`
- `crates/pine-runtime/src/strategy/broker/state.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/snapshot_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/main_tests.rs`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 632/102 passed, saved as
`{SCRATCH}/stage21a-baseline-1.log`, `{SCRATCH}/stage21a-baseline-2.log`,
`{SCRATCH}/stage21a-sema-baseline-1.log`, and
`{SCRATCH}/stage21a-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime strategy` 647 passed.
`cargo test -p pine-sema strategy` 102 passed.
`cargo test -p pine-cli formats_profiled_result_json` passed.

Close-out:
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 595 passed. Host parity 519 required
runtime goldens. WASM 624 passed. Log: `{SCRATCH}/stage21a-verify.sh.log`.

## Remaining Exclusions

21b accepts historical `calc_on_order_fills` and schedules extra script passes
after fills. 21c expands realtime broker rollback checkpoints. 21d accepts
`calc_on_every_tick`. 21e is the bar-magnifier host contract. Public
`StrategyResult` stays unchanged; extra-pass execution is still off.
