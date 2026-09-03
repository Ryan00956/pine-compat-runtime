# Strategy Internal Stage 17c Fill Transition Skeleton Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
runtime fills, conformance status, snapshots, matrix output, or public
strategy output.

Stage 17c introduces a host-neutral fill request/transition calculation that
can be tested without public `StrategyResult` generation. Production fill
paths are not routed through it yet.

## Skeleton

`crates/pine-runtime/src/strategy/broker/fill_transition.rs` owns:

- `FillRequest` (internal order key, bar/time, raw price, trigger reason)
- `FillTransition` (closed allocations, optional opened exposure, filled /
  close / open quantities, fill price, cash delta, realized PnL, commission,
  routable flag)
- `split_fill_quantities` for the five netting shapes
- `calculate_same_side_addition`
- `calculate_reduce_only` (oversized opposite quantity flattens and discards
  the remainder)
- `calculate_netting_transition` (computes cross-zero split, `routable=false`)

Cross-zero remains unrouted until Stage 19. Invalid or non-finite
quantity/price/position values return `FillCalcError` and do not mutate
`BrokerState`.

## Files

- `crates/pine-runtime/src/strategy/broker/fill_transition.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_FILL_TRANSITION_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Tests

```text
cargo test -p pine-runtime --lib fill_transition -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo clippy -p pine-runtime --all-targets -- -D warnings
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
git diff --check
scripts/verify.sh
```

Results:

- fill-transition tests: 10 passed, 0 failed
- `pine-runtime` strategy: 529 passed, 0 failed
- clippy: clean
- CLI runtime goldens: 1 passed, 0 failed; `tests/snapshots/runtime_strategy_*.json`
  unchanged
- `git diff --check`: clean
- `scripts/verify.sh`: passed (exit 0). `cargo fmt --check`, clippy `-D warnings`,
  workspace tests, host parity, WASM Node smoke, and 544 Python binding tests
  succeeded.

## Remaining Exclusions

- Production fills still use the existing per-command accounting paths.
- Ledger/aggregate invariant checks are Stage 17d.
- Routing same-side then reduction/close/exit/margin-call fills is Stage 17e-17f.
