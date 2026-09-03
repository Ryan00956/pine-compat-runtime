# Strategy Internal Stage 18b Pending Close Storage Audit

Status: closed on 2026-09-02. Production `strategy.close()` /
`strategy.close_all()` still fill on the creation bar. Pending close records
are storage-only.

## Quantity Decision

Close quantity policy is stored at placement and resolved at fill:

- `Full` — close the matching position at fill time
- `Qty(n)` — finite positive quantity, clamped at fill
- `QtyPercent(n)` — percent of the matching fill-time position

This matches Stage 18c next-tick closes, where the position can change
between the signal bar and the fill bar. Same-id replacement keeps the
internal key and overwrites policy, bar, and metadata.

Pending closes stay private. `OrderBook::cancel_id` / `clear_all` include
them.

## Files

- `crates/pine-runtime/src/strategy/broker/pending_closes.rs`
- `crates/pine-runtime/src/strategy/broker/pending_close_tests.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/close_orders.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `docs/STRATEGY_INTERNAL_STAGE18_PENDING_CLOSE_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`
- `docs/RELEASE_NOTES.md`

## Tests

Placement, percent storage, same-id replacement, cancel, rollback sequence,
and production immediate close are covered. Strategy goldens unchanged.

## Remaining Exclusions

Stage 18c switches default close/close-all to next-tick market orders.
