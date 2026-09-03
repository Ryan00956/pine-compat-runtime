# Strategy Internal Stage 17d Ledger Invariant Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
runtime fills, conformance status, snapshots, matrix output, or public
strategy output.

Stage 17d makes ledger/aggregate divergence observable before Stage 17e
routes production fills through the shared transition applier.

## Invariant

`TradeLedger::computed_net_position()` recomputes signed size and weighted
average price from open trades. After every `sync_aggregate_position_from_ledger`
call, debug builds assert that:

- the ledger cache equals the recomputed net;
- `BrokerState.position_size` and `avg_price` equal that net.

Release builds do not silently repair divergence. Tests call
`BrokerState::assert_ledger_aggregates()` after each fill-origin family.

No existing supported path produced ledger/aggregate divergence.

Covered cases: flat, long, short, pyramided same-side, partial allocation,
full flatten, reversal, reduce-only generic order, price-based entry/order,
exit fill, and margin-call fill.

## Retained Singleton Mirrors

These `BrokerState` fields remain compatibility mirrors. They are still read
by reporting, accounting, and pending-exit helpers. They are not removed in
Stage 17.

| Field | Owner after 17d | Removal |
| --- | --- | --- |
| `position_size`, `avg_price` | synced from `TradeLedger` | keep until all readers use the ledger net |
| `entry_id`, `position_entry_name`, `entry_bar_index`, `entry_time` | first-open compatibility | later reporting migration |
| `open_entry_commission` | last-open commission mirror | later trade-field migration |
| `open_trade_max_high` / `min_low` / equity-on-entry mirrors | single-open reporting | keep while `open_trade_count()==1` helpers exist |
| `cash`, runup/drawdown, `max_contracts_held_*` | broker accounting | not ledger data |
| public `orders`, `trades`, `position`, `equity` | public result | unchanged schema |

## Files

- `crates/pine-runtime/src/strategy/broker/ledger.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/ledger_invariant_tests.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_LEDGER_INVARIANT_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Tests

```text
cargo test -p pine-runtime --lib ledger_aggregates -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo clippy -p pine-runtime --all-targets -- -D warnings
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
git diff --check
scripts/verify.sh
```

Results:

- ledger invariant tests: 5 passed, 0 failed
- `pine-runtime` strategy: 534 passed, 0 failed
- clippy: clean
- CLI runtime goldens: 1 passed, 0 failed; strategy goldens unchanged
- `git diff --check`: clean
- `scripts/verify.sh`: passed (exit 0). `cargo fmt --check`, clippy `-D warnings`,
  workspace tests, host parity, WASM Node smoke, and 544 Python binding tests
  succeeded.

## Remaining Exclusions

Routing production fills through the Stage 17c applier starts in Stage 17e.
