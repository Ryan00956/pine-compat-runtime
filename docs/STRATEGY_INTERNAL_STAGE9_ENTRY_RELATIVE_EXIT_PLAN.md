# Strategy Internal Stage 9 Entry-Relative Exit Plan

Status: Slice 0 design gate closed on 2026-06-04. Do not widen runtime
`strategy.exit` compatibility until a later slice adds fixture-backed behavior,
conformance metadata, host snapshots, and release verification.

Stage 9 targets the remaining same-calculation active-entry attachment gap for
entry-relative `strategy.exit` triggers:

- `profit`;
- `loss`;
- `trail_points` when paired with `trail_offset`.

This stage is not a generic missing-entry binding feature. It is only about an
exit call that targets an active pending `strategy.entry` order with the same
`from_entry` id before that entry fills.

Primary official reference:

- TradingView Pine Script strategies, `strategy.exit()` behavior:
  https://www.tradingview.com/pine-script-docs/concepts/strategies/

The official strategy manual establishes the relevant rules:

- `strategy.exit` can be called in the same block as `strategy.entry` for a
  matching entry id;
- `from_entry` limits exit orders to matching entries, and a non-matching id
  creates no exit orders;
- `profit` and `loss` are entry-relative tick distances that compete with
  absolute `limit` and `stop` levels respectively;
- `trail_points` is an entry-relative tick distance used to compute trailing
  activation, and `trail_offset` is required for trailing stops.

## Starting Point

The current repo baseline is:

- `strategy.entry` supports current long market, limit, stop, and stop-limit
  orders without pyramiding, shorts, or reversals.
- `strategy.exit` supports current single-trigger, one-downside/one-upside
  bracket, trailing, explicit `qty`, `qty_percent`, and reservation subsets for
  the active one-net-long position.
- Same-calculation active-entry attachment is already supported for absolute
  `stop`, `limit`, and `trail_price` against a matching active pending entry.
- Same-calculation active-entry attachment using entry-relative `profit`,
  `loss`, or `trail_points` remains unsupported and is documented in
  `tests/fixtures/conformance.tsv`.
- Missing-entry exits that do not target an active pending entry or current
  open position must remain unsupported/no-op according to the current
  documented boundary.
- Public output remains the existing `StrategyResult` shape with `orders`,
  `trades`, `position`, `equity`, and `diagnostics`.

## Compatibility Boundary

Stage 9 may support only this first subset:

- long-only strategy mode;
- one active pending entry id matching `from_entry`;
- current one-net-long broker with no pyramiding;
- `profit` active-entry attachment resolved from the eventual entry fill price;
- `loss` active-entry attachment resolved from the eventual entry fill price;
- `trail_points + trail_offset` active-entry attachment resolved from the
  eventual entry fill price;
- existing `qty` and `qty_percent` quantity resolution rules for matching
  active pending entries;
- existing public order, trade, position, equity, and diagnostic schema.

Stage 9 must not add:

- arbitrary future binding for unmatched missing-entry exits;
- short entries, reversals, or pyramiding;
- generic `strategy.order()`;
- public pending-order output;
- `trail_price + trail_points` precedence changes beyond the current supported
  trailing subset;
- lower-timeframe, tick-by-tick, bar magnifier, or recalculation-on-fill
  behavior.

## Design Requirement

Entry-relative pending-entry exits cannot use `strategy.position_avg_price` at
placement time because the active entry has not filled yet. The broker needs an
internal deferred trigger representation that stores the relative tick intent
and resolves it after the entry fill price is known.

For the current long-only subset:

- `profit` resolves to a take-profit limit price at
  `entry_fill_price + profit * syminfo.mintick`;
- `loss` resolves to a stop-loss price at
  `entry_fill_price - loss * syminfo.mintick`;
- `trail_points` resolves to a trailing activation price at
  `entry_fill_price + trail_points * syminfo.mintick`;
- `trail_offset` remains the trailing stop offset in ticks.

The resolved exit must keep the current same-bar rule: an active-entry
attachment created in the same calculation as the entry must not fill before
the entry itself fills.

## Slice Plan

### Slice 0: Design Gate

Status: closed on 2026-06-04 as this document. This slice does not add runtime
behavior, widen conformance, or change public output.

Goal:

- pick the next official Pine compatibility subset after Stage 8 and define a
  safe implementation boundary.

Acceptance:

- current repo boundary is documented;
- official behavior dependency is documented;
- implementation slices are ordered from smallest to highest risk;
- no conformance row changes;
- no public schema changes.

### Slice 1: Current Boundary Lock

Goal:

- add or refresh focused fixtures that prove `profit`, `loss`, and
  `trail_points` active-entry attachment remain unsupported before behavior is
  widened.

Implementation notes:

- prefer runtime or broker tests that target same-calculation pending-entry
  attachment directly;
- keep current public output unchanged;
- do not change analyzer acceptance unless the current fixture boundary is
  stale.

### Slice 2: Deferred Relative Trigger Skeleton

Goal:

- add internal deferred trigger data for active pending-entry exits without
  filling new exits yet.

Implementation notes:

- store relative trigger intent in broker/order-book structures;
- keep absolute `stop`, `limit`, and `trail_price` behavior unchanged;
- add broker unit tests for storing, replacing, and clearing deferred relative
  attachments.

### Slice 3: `profit` Active-Entry Attachment

Goal:

- support same-calculation `strategy.exit(..., profit=...)` for the current
  active pending long entry subset.

Implementation notes:

- resolve the take-profit limit from the actual entry fill price;
- preserve existing `qty` and `qty_percent` active-entry quantity resolution;
- add runtime, incremental, CLI golden, Python, WASM, conformance, docs, and
  release evidence in the same slice.

### Slice 4: `loss` Active-Entry Attachment

Goal:

- support same-calculation `strategy.exit(..., loss=...)` for the current
  active pending long entry subset.

Implementation notes:

- resolve the stop price from the actual entry fill price;
- include stop-only and `loss + profit` bracket evidence;
- keep same-side unsupported combinations unchanged.

### Slice 5: `trail_points` Active-Entry Attachment

Goal:

- support same-calculation
  `strategy.exit(..., trail_points=..., trail_offset=...)` for the current
  active pending long entry subset.

Implementation notes:

- resolve trailing activation from the actual entry fill price;
- preserve current trailing activation-bar behavior;
- do not add `trail_price + trail_points` precedence changes unless a later
  fixture-backed slice explicitly targets that official behavior.

### Slice 6: Host Parity, Conformance, And Audit

Goal:

- close Stage 9 with synchronized host parity, conformance, matrix snapshot,
  docs, release notes, and an audit.

Acceptance:

- CLI, Python, and WASM expose identical public strategy output for at least one
  representative active-entry relative-exit fixture;
- `tests/fixtures/conformance.tsv` precisely lists the supported and
  unsupported Stage 9 subset;
- broader missing-entry, pyramiding, short, reversal, and generic-order
  behavior remains unsupported.

## Verification Plan

Each behavior slice should run:

```text
cargo fmt
cargo test -p pine-runtime strategy --quiet
cargo test -p pine-runtime --test incremental --quiet
cargo test -p pine-sema strategy --quiet
cargo test -p pine-cli runtime_outputs_match_golden_snapshots --quiet
cargo test -p pine-cli matrix_output_matches_golden_snapshot --quiet
cargo test -p pine-cli conformance --quiet
python3 -m pytest python/tests -q
cargo test -p pine-wasm strategy --quiet
python3 scripts/check_structure.py
```

Before final closeout, run:

```text
scripts/verify.sh
```

Stop if official behavior requires multiple open trades, short exposure,
generic `strategy.order()`, public schema changes, or a host-data dependency
that is not available in the repo fixtures.
