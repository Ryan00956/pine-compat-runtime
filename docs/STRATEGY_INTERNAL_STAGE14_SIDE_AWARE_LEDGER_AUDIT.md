# Strategy Internal Stage 14b Side-Aware Ledger Audit

Status: closed as a behavior-preserving internal model slice. Runtime fills,
conformance status, snapshots, matrix output, and public strategy output are
unchanged for the current long-only subset.

Stage 14b makes the broker direction-aware so later short/reversal slices can
store signed position state without first rewriting the ledger.

## Internal Model

`TradeDirection` now has `Long` and `Short`.

`TradeLedger` responsibilities after this slice:

- every `OpenTrade` carries an explicit side;
- net position `signed_size` is the signed sum of open quantities;
- average price is computed only across the current net-position side;
- current `strategy.close` / `strategy.exit` allocation helpers stay long-only
  and skip short trades;
- `TradeAllocation` copies the source trade direction.

`BrokerState` aggregate mirrors:

- `position_size` continues to come from `TradeLedger::net_position()`;
- `max_contracts_held_long` updates from positive net size;
- `max_contracts_held_short` is stored and currently stays `0.0`;
- `max_contracts_held_all` is `max(long, short)`, which equals the long
  maximum while shorts are unsupported.

Pending books:

- pending entries already store `PendingEntryDirection::{Long, Short}`;
  `Short` remains the reduce-only market-short order path, not short exposure;
- pending exits expose `trade_direction()` as `Long` for the current supported
  exit subset. A stored short-exit field is deferred until a short-exit slice.

## Behavior Preservation

Long-only open, pyramiding, FIFO/`ANY` close allocation, pending exits, and
reduce-only market-short orders keep the same public orders, trades, position,
equity, and `strategy.max_contracts_held_*` values.

The mixed long/short ledger math is covered by broker-only tests. No Pine
source can currently construct a short `OpenTrade`.

## Evidence

- `crates/pine-runtime/src/strategy/broker/ledger.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/accounting.rs`
- Broker tests:
  - `trade_ledger_mirrors_current_single_long_entry`
  - `stage14_side_aware_ledger_uses_signed_net_and_side_specific_average`
  - `stage14_side_aware_ledger_short_only_net_is_negative`
  - `stage14_pending_exits_report_long_exposure`

## Next Slice

14c may accept one explicit-quantity market short entry without reversal. It
must not treat existing long-exit/close helpers as implicitly valid for shorts.
