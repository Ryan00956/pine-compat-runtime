# Strategy Internal Stage 17b Origin And Internal Keys Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
runtime fills, conformance status, snapshots, matrix output, or public
strategy output.

Stage 17b stores explicit command origin and a stable internal creation
sequence on pending entry/order records so later slices do not use
`enforce_pyramiding` as the only entry-versus-generic-order distinction.

## Same-Id Replacement Rule

Same public id replacement keeps the original `InternalOrderKey` /
creation sequence and overwrites quantity, kind, origin, metadata, and
`created_bar_index`. A later distinct id receives the next sequence. This
matches current eligibility (new `created_bar_index`) while preserving
creation order among still-pending peers.

## Cancellation Lookup Policy

`OrderBook::cancel_id` still searches pending entries and pending exits by
the public string id. Internal keys are not part of cancellation. One public
id can currently match both an entry/order and an exit; that is the existing
facade and is unchanged. Stage 20 owns unified cross-family lookup widening.

## Files

- `crates/pine-runtime/src/strategy/broker/types.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/tests.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_origin_tests.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_ORIGIN_KEYS_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

Fill dispatch in `pending_entry_fills.rs` is unchanged. The temporary
`enforce_pyramiding` boolean remains as a Copy migration field derived from
`StrategyCommandOrigin::Entry` at placement.

## Tests

```text
cargo test -p pine-runtime --lib pending_entry_origin -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-sema strategy
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
git diff --check
scripts/verify.sh
```

Results:

- origin tests: 7 passed, 0 failed
- `pine-runtime` strategy: 519 passed, 0 failed (512 after 17a plus 7 origin tests)
- `pine-sema` strategy: 96 passed, 0 failed
- CLI runtime goldens: 1 passed, 0 failed; `tests/snapshots/runtime_strategy_*.json`
  unchanged
- `git diff --check`: clean
- `scripts/verify.sh`: passed (exit 0). `cargo fmt --check`, clippy `-D warnings`,
  workspace tests, host parity, WASM Node smoke, and 544 Python binding tests
  succeeded.

## Remaining Exclusions

- Fill request/transition skeleton is Stage 17c.
- Ledger invariant checks are Stage 17d.
- Routing production fills through a shared applier is Stage 17e-17f.
- Pending exits, closes, and margin calls do not yet store origin or internal
  keys.
- Public pending-order JSON remains private.
