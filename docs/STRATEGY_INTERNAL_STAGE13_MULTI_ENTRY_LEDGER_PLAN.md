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

### Slice 2: Ledger Ownership Audit

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

### Slice 3: Long Market Pyramiding

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
