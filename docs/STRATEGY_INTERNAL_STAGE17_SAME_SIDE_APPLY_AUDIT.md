# Strategy Internal Stage 17e Same-Side Open Apply Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
conformance status, snapshots, matrix output, or public strategy output.

Stage 17e routes flat and same-side `strategy.entry()` / `strategy.order()`
opens through `calculate_same_side_addition` plus one open applier. Cash is
applied from the transition `cash_delta` exactly once. Pyramiding
`EnforceLimit` versus `BypassLimit` / same-tick price exception is preserved.
Margin rejection still happens before the applier, so rejected fills do not
open a trade.

Price-based same-side limit/stop/stop-limit fills share `entry_long_internal` /
`entry_short_internal`, so they use the same applier after market
characterization tests passed. Opposite-side mixed opens that are not same-side
keep a fallback cash update and are not treated as routed netting.

## Files

- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/closed_trades.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/fill_origin_characterization_tests.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_SAME_SIDE_APPLY_AUDIT.md`
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

Results:

- characterization: 11 passed, 0 failed, including
  `characterization_same_side_open_applies_transition_cash_once`
- `pine-runtime` strategy: 534 passed plus the new cash-once test
- clippy: clean
- CLI runtime goldens: 1 passed, 0 failed; strategy goldens unchanged
- `git diff --check`: clean
- `scripts/verify.sh`: passed (exit 0). `cargo fmt --check`, clippy `-D warnings`,
  workspace tests, host parity, WASM Node smoke, and 544 Python binding tests
  succeeded.

## Remaining Exclusions

Reduce-only generic orders, `strategy.close` / `close_all`, pending exits, and
margin-call fills still use per-command close accounting until Stage 17f.
