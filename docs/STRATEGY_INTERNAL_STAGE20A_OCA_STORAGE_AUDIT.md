# Strategy Internal Stage 20a OCA Storage And Group Identity Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Internal OCA group keys
and pending-intent membership are stored on the order book. `oca_name` and
`oca_type` remain semantically rejected. Public JSON shape and conformance are
unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- An OCA group key is `(name, type)` where type is `none`, `cancel`, or
  `reduce`. The same name with different types is two groups.
- Membership is stored on `OrderBook` for pending entries/orders (by internal
  order key) and pending exits (by exit identity).
- Same-id same-direction replacement preserves the internal key, so membership
  remains until reassigned.
- `strategy.cancel(id)` removes matching pending intents and their OCA
  membership. Clone/rollback keeps membership with the cloned broker state.
- Pine `oca_name` / `oca_type` arguments stay rejected. No public pending-order
  or OCA schema.

## Files

- `crates/pine-runtime/src/strategy/broker/types.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/oca.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` twice, 603 passed, saved as
`{SCRATCH}/stage20a-baseline-1.log` and `{SCRATCH}/stage20a-baseline-2.log`.

Owner-local: `cargo test -p pine-runtime strategy` 607 passed.
`cargo test -p pine-sema strategy` passed (oca_name/oca_type still rejected).

Close-out:
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 586 passed. Host parity 510 required
runtime goldens. Log: `{SCRATCH}/stage20a-verify.sh.log`.

## Remaining Exclusions

20b accepts explicit `strategy.oca.none` for the smallest supported pending
family. Cancel/reduce types stay rejected until 20c/20d. Custom `oca_name`
syntax stays rejected in 20a.
