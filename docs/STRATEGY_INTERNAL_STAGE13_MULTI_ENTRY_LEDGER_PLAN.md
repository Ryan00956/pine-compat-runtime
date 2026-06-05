# Strategy Internal Stage 13 Multi-Entry Ledger Plan

Status: design gate opened on 2026-06-05. Runtime behavior, conformance claims,
fixtures, matrix output, and public JSON shape are unchanged in this slice.

Stage 12 closed the declaration-property boundary and explicitly warned against
accepting another `strategy()` property as a no-op. Stage 13 designs the next
broker-model foundation before any `pyramiding`, short, reversal,
`close_entries_rule`, or generic order work is accepted.

Primary official reference:

- TradingView Pine Script strategies:
  https://www.tradingview.com/pine-script-docs/concepts/strategies/

Relevant official rules:

- `strategy.entry()` can reverse an open opposite-direction position by adding
  the open position size to the new transaction size.
- `pyramiding` limits the number of open trades from `strategy.entry()` in one
  position; the default is `1`.
- Multiple price-based `strategy.entry()` orders triggered on the same tick can
  exceed the pyramiding setting.
- `strategy.order()` ignores most strategy properties such as `pyramiding` and
  changes the net market position directly.
- `strategy.close_all()` exits the open position without linking to a specific
  entry id, which matters once a position contains multiple open trades.
- By default, exits close oldest trades first; `close_entries_rule="ANY"` changes
  id-specific close behavior.

## Starting Point

The live repo currently supports a long-only one-net-position strategy broker:

- `strategy.entry()` accepts only `strategy.long` and supported market, limit,
  stop, and stop-limit entry forms.
- Repeated long entries while long are ignored by the current no-pyramiding
  rule.
- `strategy.close()`, `strategy.close_all()`, `strategy.cancel()`,
  `strategy.cancel_all()`, and the supported `strategy.exit()` subset operate
  against the current long-only position and internal pending-entry/exit books.
- `strategy()` still rejects `pyramiding` and `close_entries_rule`; sema also
  rejects `strategy.short` entries and generic `strategy.order()`.
- `TradeLedger` already stores a vector of `OpenTrade` values and FIFO
  allocation helpers, but normal entry opening still clears the ledger before
  pushing one long trade.
- `BrokerState` still carries singleton mirrors such as `position_size`,
  `avg_price`, `entry_id`, entry timing, and open-trade runup/drawdown fields.
- Public strategy output exposes filled orders, closed trades, aggregate
  position/equity snapshots, and diagnostics. It does not expose pending orders,
  open-trade ledgers, OCA groups, or declaration settings.

## Goal

Design a staged path from the current one-net-long broker to a Pine-compatible
multi-entry foundation without widening compatibility claims prematurely.

The design must answer these internal questions before runtime behavior changes:

- Which broker fields become derived from `TradeLedger`, and which singleton
  fields can remain as public aggregate mirrors?
- How `pyramiding` counts open and pending `strategy.entry()` trades in the
  current historical timing model.
- How repeated same-id entries, same-direction entries, and price-based pending
  entries interact with the pyramiding limit.
- How FIFO close allocation works for `strategy.close()`,
  `strategy.close_all()`, and `strategy.exit()` before
  `close_entries_rule="ANY"` is accepted.
- Which unsupported pieces stay locked: shorts, reversal, `strategy.order()`,
  OCA across order families, public pending-order output, realtime recalculation,
  and strategy order-fill alerts.

## Non-Goals

- No runtime widening in Slice 0.
- No acceptance of `pyramiding`, `close_entries_rule`, short entries,
  reversals, `strategy.order()`, OCA settings, or risk APIs in the design slice.
- No public `StrategyResult` schema expansion.
- No public pending-order or open-trade ledger output.
- No realtime/tick recalculation changes.
- No change to existing no-pyramiding behavior until a behavior slice is
  fixture-backed.

## Compatibility Boundary

The first behavior stage after design should be long-only and aggregate-output
only. It may use `TradeLedger` internally, but public output must remain the
existing aggregate strategy JSON unless a separate schema stage is designed.

Allowed first behavior target after the design and boundary-lock slices:

- `strategy(..., pyramiding=N)` with positive const integer `N`;
- long-only `strategy.entry()` market entries first;
- existing supported explicit/default quantity paths;
- multiple open long trades with aggregate `strategy.position_size`,
  `strategy.position_avg_price`, equity, and profit variables derived from the
  ledger;
- `strategy.close_all()` flattening all open long trades;
- default FIFO allocation for closes and exits.

Still out of scope for the first behavior target:

- short exposure and automatic reversal;
- `strategy.order()` netting behavior;
- `close_entries_rule="ANY"`;
- price-based same-tick pyramiding-limit exceptions;
- multiple pending entry fills that exceed the limit;
- OCA across generic orders;
- public open-trade or pending-order schema expansion;
- strategy order-fill alert delivery.

## Slice Plan

### Slice 0: Design Gate

Status: this document. This slice does not change runtime behavior,
conformance, fixtures, snapshots, or public output.

Goal:

- define the Stage 13 multi-entry and pyramiding boundary before code changes.

Acceptance:

- official strategy-entry, pyramiding, close-all, FIFO, and generic-order
  dependencies are recorded;
- the live one-net-long baseline is documented;
- the first behavior target is long-only and public-schema preserving;
- unsupported short/reversal/generic-order/OCA/realtime boundaries are explicit.

### Slice 1: Boundary Lock

Status: closed on 2026-06-05. This slice added test assertions only and did not
change runtime behavior, conformance, fixtures, snapshots, matrix output, or
public JSON.

Goal:

- add or refresh tests proving the current repo still rejects or no-ops the
  pieces Stage 13 has not accepted.

Targets:

- sema diagnostics for `pyramiding`, `close_entries_rule`, `strategy.short`, and
  `strategy.order()` remain stable;
- runtime repeated-long-entry no-pyramiding behavior remains stable;
- conformance stays unchanged except for fixture registration if needed.

Acceptance:

- unsupported boundaries are fixture-backed before any positive behavior route
  changes;
- no public output or matrix support claim widens.

Closed evidence:

- `crates/pine-sema/tests/fixtures.rs` now asserts that unsupported
  `pyramiding` and short-entry diagnostics name the relevant unsupported
  boundary.
- `unsupported_strategy_declaration_properties.pine` still covers
  `close_entries_rule`, and `unsupported_strategy_orders.pine` still covers
  `strategy.order()`.
- `crates/pine-runtime/src/tests/strategy.rs` now verifies repeated long entries
  without accepted pyramiding still emit only the first entry order, keep a
  one-unit position at the first fill price, produce no closed trades, and emit
  no runtime diagnostics.

### Slice 2: Ledger Ownership Audit

Status: closed on 2026-06-05. See
`docs/STRATEGY_INTERNAL_STAGE13_LEDGER_OWNERSHIP_AUDIT.md`. This slice did not
change runtime behavior, conformance, fixtures, snapshots, matrix output, or
public JSON.

Goal:

- document and, if needed, test the internal split between `TradeLedger` and
  singleton aggregate broker fields.

Targets:

- identify every `BrokerState` field that duplicates ledger state;
- define derived aggregate behavior for position size, average price, max held
  contracts, open profit, runup/drawdown, commission, and margin inputs;
- keep current one-open-trade runtime behavior unchanged.

Acceptance:

- internal ledger invariants are covered by unit tests or audit notes;
- no accepted script produces different output.

Closed evidence:

- The ownership audit records current `TradeLedger` responsibilities, legacy
  singleton `BrokerState` mirrors, aggregate accounting owners, and the existing
  unit-test evidence for single-open-trade mirroring, reductions, and FIFO
  allocation helpers.
- The next migration order is locked before any positive `pyramiding` behavior:
  centralize ledger mutation helpers, keep aggregate public mirrors stable,
  convert open-trade namespace reads later, and preserve the current public
  `StrategyResult` schema.

### Slice 3: Entry Fill Ownership Helper

Status: closed on 2026-06-05. This slice introduced a private helper for the
current long-entry fill ownership handoff and did not change runtime behavior,
conformance, fixtures, snapshots, matrix output, or public JSON.

Goal:

- route the existing one-open-long entry fill through one helper that updates
  both legacy singleton mirrors and `TradeLedger`.

Closed evidence:

- `BrokerState::entry_long()` now calls `record_open_long_trade()` after
  constructing the supported `OpenTrade`.
- `record_open_long_trade()` owns the current handoff to
  `record_open_long_legacy_state()` and `trade_ledger.open_long()`.
- Existing single-entry, pending-entry, ledger mirror, and full release gates
  continue to pass with unchanged behavior.

### Slice 4: TradeLedger Append Helper

Status: closed on 2026-06-05. This slice added an internal ledger append helper
and unit test only. Runtime behavior, conformance, fixtures, snapshots, matrix
output, and public JSON are unchanged.

Goal:

- prepare the ledger for later multi-entry work without routing accepted scripts
  into multi-open-trade behavior.

Closed evidence:

- `TradeLedger::open_long()` still clears existing open trades, preserving the
  current one-net-long runtime behavior.
- `TradeLedger::append_long()` appends an open long trade and rebuilds the
  weighted aggregate `NetPosition`.
- `trade_ledger_append_long_rebuilds_weighted_net_position` covers the internal
  append invariant before any public `pyramiding` route uses it.

### Slice 5: Aggregate Position Sync Helper

Status: closed on 2026-06-05. This slice added a private aggregate sync helper
and routed the current one-open-long entry handoff through it. Runtime behavior,
conformance, fixtures, snapshots, matrix output, and public JSON are unchanged.

Goal:

- prepare aggregate `position_size` and `avg_price` ownership for later
  multi-entry work by syncing them from `TradeLedger::net_position()`.

Closed evidence:

- `BrokerState::record_open_long_trade()` now calls
  `sync_aggregate_position_from_ledger()` after updating `TradeLedger`.
- The helper currently updates only aggregate position size and average price;
  entry id, open-trade namespace mirrors, runup/drawdown state, and public
  output remain under the current one-open-trade legacy contract.
- Existing entry, pending-entry, ledger mirror, and full release gates continue
  to pass with unchanged behavior.

### Slice 6: Allocation Sync Helper

Status: closed on 2026-06-05. This slice routed existing long close, exit, and
margin-call allocation updates through a private aggregate sync helper. Runtime
behavior, conformance, fixtures, snapshots, matrix output, and public JSON are
unchanged.

Goal:

- keep aggregate `position_size` and `avg_price` derived from `TradeLedger`
  after existing allocation updates.

Closed evidence:

- Long `strategy.close`, supported `strategy.exit` fills, and long margin-call
  reductions now call `apply_trade_allocations_and_sync_position()` after
  recording the same closed-trade and cash effects.
- Full-close fallback for empty allocation lists still clears the ledger and
  syncs aggregate position to flat.
- Existing partial close, partial exit, margin, strategy entry, and full release
  gates continue to pass with unchanged public output.

### Slice 7: Default Pyramiding Gate Helper

Status: closed on 2026-06-05. This slice added an internal default
`pyramiding_limit` of `1` and routed current long-entry admission checks through
`can_open_long_entry()`. Runtime behavior, conformance, fixtures, snapshots,
matrix output, and public JSON are unchanged.

Goal:

- isolate the current no-pyramiding long-entry gate behind one helper before
  accepting any public `pyramiding` setting.

Closed evidence:

- `BrokerState` now stores `pyramiding_limit: 1` by default.
- Market, limit, stop, and stop-limit long-entry placement/fill paths use
  `can_open_long_entry()` instead of direct aggregate-position checks.
- `default_pyramiding_limit_allows_only_one_long_entry` proves the internal
  default still rejects a second long entry and leaves a single open trade.

### Slice 8: Open-Trade Count Ledger Read

Status: closed on 2026-06-05. This slice changed the internal
`open_trade_count()` source from singleton mirrors to `TradeLedger::open_count()`
and added an internal multi-ledger count test. Runtime behavior, conformance,
fixtures, snapshots, matrix output, and public JSON are unchanged.

Goal:

- make the strategy open-trade count read path ledger-backed before accepting
  public multi-entry behavior.

Closed evidence:

- `BrokerState::open_trade_count()` now returns the ledger open count with
  overflow clamped to `i64::MAX`.
- Existing no-pyramiding runtime behavior still keeps the ledger count at `0` or
  `1`.
- `open_trade_count_reads_trade_ledger_count` proves the internal count path can
  observe two ledger entries before any accepted script can create them.

### Slice 9: Open-Trade Field Ledger Reads

Status: closed on 2026-06-05. This slice changed the internal
`strategy.opentrades.*` field read helpers from singleton mirrors to
ledger-indexed reads and added an internal two-entry field test. Runtime
behavior, conformance, fixtures, snapshots, matrix output, and public JSON are
unchanged.

Goal:

- make open-trade field reads index the ledger directly before accepting public
  multi-entry behavior.

Closed evidence:

- `TradeLedger::open_at()` exposes a bounded internal accessor for open trade
  field reads.
- `BrokerState::open_trade_entry_price()`, `entry_id()`, `entry_bar_index()`,
  `entry_time()`, `size()`, `profit()`, `commission()`, `max_runup()`, and
  `max_drawdown()` now read from the requested ledger entry index.
- `open_trade_fields_read_trade_ledger_entries` proves fields can be read from
  ledger indexes `0` and `1`, with out-of-range and negative indexes returning
  `None`.

### Slice 10: Long Market Pyramiding

Goal:

- implement the first positive multi-entry behavior for long market entries.

Target subset:

- `strategy(..., pyramiding=N)` with positive const integer `N`;
- same-direction long market entries only;
- existing explicit/default quantity paths;
- no short/reversal/price-based entry exception behavior;
- aggregate public strategy JSON only.

Acceptance:

- runtime fixtures cover multiple long entries, average price, position size,
  equity/profit variables, `strategy.opentrades`, `strategy.close_all()`, and a
  FIFO `strategy.close()` case;
- Python and WASM parity tests cover the public JSON contract;
- conformance, matrix, docs, release notes, and `scripts/verify.sh` are
  synchronized.

## Compatibility Contract

Until a later behavior slice closes, the supported strategy subset remains the
one recorded in `tests/fixtures/conformance.tsv`. Stage 13 Slice 0 is only a
design gate and must not be used to claim `pyramiding`, multi-entry ledgers,
short entries, reversals, `strategy.order()`, or `close_entries_rule` support.
