# Strategy Internal Stage 21c Realtime Broker Rollback Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Forming-bar updates
re-execute from the last confirmed checkpoint. After `varip` seeding, the
runtime restores the confirmed broker and alert checkpoint so abandoned
intrabar placements, cancellations, activations, fills, and alerts do not
leak. `calc_on_every_tick` stays rejected. Public `StrategyResult` is
unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/language/execution-model/
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Forming and confirmed updates clone the last confirmed runtime, seed `varip`
  from the previous forming update, then restore the confirmed broker
  checkpoint (order book, OCA, reservations, ledger, cash, fill alerts) and
  script alerts.
- Only a confirmed update commits that replay's broker and output state.
- Replacement forming updates therefore discard abandoned placements,
  cancellations, stop-limit activations, fills, and alerts.
- Confirmed output matches an equivalent historical batch for the covered
  limit-fill path.
- `calc_on_every_tick` remains semantically rejected.

## Files

- `crates/pine-runtime/src/runtime/realtime.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/strategy/broker/state.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `tests/fixtures/conformance.tsv`
- `tests/snapshots/matrix.json`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 652/104 passed, saved as
`{SCRATCH}/stage21c-baseline-1.log`, `{SCRATCH}/stage21c-baseline-2.log`,
`{SCRATCH}/stage21c-sema-baseline-1.log`, and
`{SCRATCH}/stage21c-sema-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime strategy` 657 passed.

Close-out:
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 598 passed. Host parity 522 required
runtime goldens. WASM 627 passed. Log: `{SCRATCH}/stage21c-verify.sh.log`.

## Remaining Exclusions

21d accepts `calc_on_every_tick` only after this rollback path. 21e is the bar
magnifier host contract. External alert delivery stays out of scope. Public
`StrategyResult` stays unchanged.
