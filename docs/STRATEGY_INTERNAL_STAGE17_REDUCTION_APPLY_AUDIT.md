# Strategy Internal Stage 17f Reduction Apply Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
conformance status, snapshots, matrix output, or public strategy output.

Stage 17f routes reduce-only generic orders, `strategy.close` /
`strategy.close_all`, pending `strategy.exit` fills, and long/short margin-call
liquidations through `BrokerState::apply_reduction_cash_and_position`. FIFO/ANY
allocation and command-specific order/trade/alert identity stay in the callers.
Close timing is unchanged.

## Files

- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/close_orders.rs`
- `crates/pine-runtime/src/strategy/broker/fills.rs`
- `crates/pine-runtime/src/strategy/broker/fill_origin_characterization_tests.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_REDUCTION_APPLY_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Tests

```text
cargo test -p pine-runtime --lib characterization_ -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo clippy -p pine-runtime --all-targets -- -D warnings
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
git diff --check
scripts/verify.sh
```

Results recorded with Stage 17g closeout. Characterization includes
`characterization_reduce_only_applies_shared_reduction_cash`. Strategy goldens
are unchanged.

## Remaining Exclusions

Stage 17g removes leftover naming forks and closes Stage 17. Close timing stays
current-bar until Stage 18.
