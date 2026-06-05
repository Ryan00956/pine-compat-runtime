# Strategy Internal Stage 13 Ledger Ownership Audit

Status: closed on 2026-06-05 as a documentation-only ownership audit. Runtime
behavior, conformance, fixtures, snapshots, matrix output, and public JSON are
unchanged.

Stage 13 Slice 2 reviewed the current broker state split before any positive
multi-entry or `pyramiding` behavior is accepted. The live code already contains
a private `TradeLedger`, but the executable strategy subset remains one
net-long position with legacy singleton broker mirrors.

## Current Owners

`TradeLedger` currently owns an internal vector of `OpenTrade` records and a
cached aggregate `NetPosition`.

Current ledger responsibilities:

- store the one supported open long trade after a successful entry fill;
- mirror entry id, entry price, quantity, entry bar/time, entry commission, and
  current open-trade extremes for the supported single open trade;
- calculate FIFO allocations for future multi-trade exits;
- apply allocations and rebuild aggregate net position in internal tests.

Current ledger limitations:

- `open_long()` clears existing open trades before pushing the new long trade;
- normal runtime paths do not append multiple open trades;
- `update_extremes()` only updates the first open trade;
- allocation helpers are not a public compatibility claim and are not routed
  through accepted multi-entry behavior.

`BrokerState` still owns the executable one-net-long compatibility surface.

Legacy singleton mirrors:

- `position_size`;
- `avg_price`;
- `entry_id`;
- `entry_bar_index`;
- `entry_time`;
- `open_entry_commission`;
- `open_trade_max_high`;
- `open_trade_min_low`;
- `open_trade_equity_on_entry`;
- `open_trade_min_equity_before_entry`;
- `open_trade_max_equity_before_entry`.

Aggregate/accounting state still owned directly by `BrokerState`:

- `cash`;
- `min_equity_before_open_trade`;
- `max_equity_before_open_trade`;
- `max_runup`, `max_runup_percent`;
- `max_drawdown`, `max_drawdown_percent`;
- `max_contracts_held_long`;
- public `orders`, `trades`, `position`, `equity`, and `diagnostics`;
- `closed_trade_metrics`;
- the internal pending-entry and pending-exit `OrderBook`.

## Live Code Evidence

- `crates/pine-runtime/src/strategy/broker/mod.rs` stores both legacy singleton
  mirrors and `trade_ledger`.
- Successful entries call the legacy mirror recorder and then
  `trade_ledger.open_long()`, preserving one-open-trade behavior.
- `crates/pine-runtime/src/strategy/broker/fills.rs` updates both singleton
  mirrors and `trade_ledger` during close, exit, and margin-call reductions.
- `crates/pine-runtime/src/strategy/broker/accounting.rs` still reads singleton
  mirrors for open-trade namespace functions, mark-to-market values,
  runup/drawdown, capital held, and `strategy.position_avg_price`.
- `crates/pine-runtime/src/strategy/broker/exits.rs` and
  `active_entry_brackets.rs` still route matching through singleton
  `position_size`, `avg_price`, and `entry_id` checks.
- `crates/pine-runtime/src/strategy/broker/tests.rs` already covers the current
  ownership invariants:
  - `trade_ledger_mirrors_current_single_long_entry`;
  - `trade_ledger_tracks_partial_and_final_long_reductions`;
  - `trade_ledger_allocates_omitted_entry_by_global_fifo`;
  - `trade_ledger_allocates_matching_entry_by_fifo`;
  - `trade_ledger_applies_allocations_and_rebuilds_net_position`.

## Migration Order

Stage 13 should keep behavior unchanged until each migration point has tests.

1. Keep legacy singleton mirrors as the executable public aggregate source while
   introducing explicit ledger mutation helpers for any future append behavior.
2. Add positive multi-entry behavior only after the broker can update ledger and
   aggregate mirrors from one helper instead of parallel ad hoc assignments.
3. Convert `strategy.opentrades.*()` field functions from singleton
   `trade_num == 0` assumptions to ledger-indexed reads before claiming multiple
   open-trade namespace support.
4. Convert close and exit fills to allocate through `TradeLedger` first, then
   derive aggregate `position_size` and `avg_price` from `NetPosition`.
5. Keep public `StrategyResult` aggregate-only until a separate schema stage
   decides whether open-trade or pending-order output is exposed.

## Boundaries

Slice 2 does not support:

- `pyramiding`;
- multiple runtime open trades;
- short exposure or reversal;
- `strategy.order()`;
- `close_entries_rule`;
- price-based same-tick pyramiding exceptions;
- public pending-order or open-trade ledger output.

The supported strategy subset remains the one recorded in
`tests/fixtures/conformance.tsv`.

## Slice 3 Follow-Up

Stage 13 Slice 3 added `BrokerState::record_open_long_trade()` as the first
private helper for the current long-entry fill ownership handoff. It keeps the
same one-open-trade behavior by updating legacy singleton mirrors and
`TradeLedger` together, and it does not append multiple open trades.

## Slice 4 Follow-Up

Stage 13 Slice 4 added `TradeLedger::append_long()` as an internal append helper
and test-backed weighted-net-position invariant. Runtime `open_long()` still
clears existing open trades before appending, so accepted scripts keep the same
one-net-long behavior.

## Slice 5 Follow-Up

Stage 13 Slice 5 added
`BrokerState::sync_aggregate_position_from_ledger()` and routes the current
long-entry fill handoff through it after `TradeLedger` updates. The helper only
syncs aggregate `position_size` and `avg_price`; entry-id-specific mirrors,
open-trade namespace reads, runup/drawdown state, and public output stay on the
current one-open-trade path.

## Slice 6 Follow-Up

Stage 13 Slice 6 added
`BrokerState::apply_trade_allocations_and_sync_position()` for the existing
long close, supported exit, and long margin-call reduction paths. It keeps
aggregate `position_size` and `avg_price` derived from `TradeLedger` after
allocation updates while preserving legacy entry-id and open-trade mirrors.

## Slice 7 Follow-Up

Stage 13 Slice 7 added the internal `pyramiding_limit` field with a default of
`1` and routes current long-entry admission through `can_open_long_entry()`.
This is still the current no-pyramiding behavior; no public `strategy()`
`pyramiding` declaration is accepted yet.

## Slice 8 Follow-Up

Stage 13 Slice 8 changed `BrokerState::open_trade_count()` to read
`TradeLedger::open_count()` instead of singleton mirrors. Accepted scripts still
produce only `0` or `1` open trades, but the internal count path is ready for
future multi-entry behavior.

## Slice 9 Follow-Up

Stage 13 Slice 9 changed `strategy.opentrades.*` field helpers to read the
requested open-trade index from `TradeLedger::open_at()`. Accepted scripts still
produce only index `0`, but internal tests now cover indexes `0` and `1` before
public multi-entry behavior is accepted.

## Slice 10 Follow-Up

Stage 13 Slice 10 accepts the first public `strategy(..., pyramiding=N)` subset
for positive integer const values and same-direction long market entries.
`StrategySettings::pyramiding_limit` now initializes `BrokerState`, open long
entries append to `TradeLedger` while the limit allows them, and aggregate
`position_size`, `avg_price`, and `max_contracts_held_long` synchronize from
the ledger. Price-based same-tick entry exceptions, shorts, reversals, and
broader multi-entry exit/reporting semantics remain outside this slice.
