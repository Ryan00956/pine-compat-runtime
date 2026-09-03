# Strategy Internal Stage 17 Unified Fill Audit

Status: closed on 2026-09-02. Stage 17 is a refactor stage. Public
`StrategyResult` fields, semantic acceptance, conformance rows, and
pre-Stage-17 `tests/snapshots/runtime_strategy_*.json` goldens are unchanged.

## Closed Subset

- Pending entry/order records store `StrategyCommandOrigin` and a stable
  `InternalOrderKey` / creation sequence. Same-id replacement keeps the key.
- `FillRequest` / `FillTransition` calculate same-side opens, reduce-only
  closes, and unrouted cross-zero splits without host types.
- `TradeLedger::computed_net_position()` is the invariant source for signed
  size and average price. Debug builds assert after position sync.
- Flat/same-side opens apply cash from `calculate_same_side_addition`.
- Reduce-only, close, close-all, exit, and margin-call fills apply cash and
  ledger updates through `apply_reduction_cash_and_position`.
- `close_all_position` and `fill_pending_market_entries` replace misleading
  long-only facade names. Both sides use those paths.

## Slice Audits

- `docs/STRATEGY_INTERNAL_STAGE17_BASELINE_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE17_ORIGIN_KEYS_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE17_FILL_TRANSITION_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE17_LEDGER_INVARIANT_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE17_SAME_SIDE_APPLY_AUDIT.md`
- `docs/STRATEGY_INTERNAL_STAGE17_REDUCTION_APPLY_AUDIT.md`

## Files (17g)

- `crates/pine-runtime/src/strategy/broker/close_orders.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/tests.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `docs/STRATEGY_INTERNAL_STAGE17_UNIFIED_FILL_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`
- `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md`
- `docs/RELEASE_NOTES.md`

## Completion Gate

```text
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-sema strategy
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 scripts/check_host_parity.py
git diff --check
scripts/verify.sh
```

Results are recorded at Stage 17 close. No semantic acceptance or conformance
claim changed. Unsupported parameters remain fail-closed.

## Remaining Exclusions

Stage 18 owns historical close timing and scheduler order. Generic-order
cross-zero netting is Stage 19. OCA, recalculation, and `strategy.risk.*`
remain unsupported.
