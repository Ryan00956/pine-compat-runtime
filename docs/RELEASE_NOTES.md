# Release Notes

## Unreleased

- Added fixture-backed UDT passthrough through UDF parameters, UDF returns, and
  pure receiver methods. Local UDT values can now be passed to a pure UDF that
  directly returns the same parameter, or a block-local alias chain that starts
  from that parameter, or a nested passthrough UDF call that maps back to that
  parameter, assigned at the callsite through positional or named arguments,
  and field-read there. Pure local UDT methods may also return the receiver
  itself or another local UDT parameter and allow the caller to assign and
  field-read that returned value; field mutation, UDF construction from untyped
  scalar parameters, imported UDT identity, UDT history, `varip`, nested UDT
  fields, and UDT arrays remain outside the supported subset.
- Closed Strategy Internal Stage 13 release-note coverage through Slice 101. The
  Stage 13 multi-entry ledger contract now records omitted-`from_entry`
  trailing future-entry persistence, CLI/WASM/Python host parity for omitted
  future-entry and same-entry-id fixtures, internal per-open-trade exit key
  scoping, current and persistent same-entry-id omitted exits, and same-tick
  price-based entry pyramiding-limit exceptions. The supported surface remains
  limited to `tests/fixtures/conformance.tsv` and does not claim shorts,
  reversals, `strategy.order()`, `close_entries_rule`, public pending-order or
  reservation records, or broader multi-entry reporting.
- Added fixture-backed `alert()` frequency support for the const-string
  `alert.freq_once_per_bar` default, `alert.freq_all`, and
  `alert.freq_once_per_bar_close` subset. The runtime now suppresses repeated
  same-callsite default/once-per-bar alerts within a bar, preserves every
  reached call for `alert.freq_all`, and emits close-frequency alerts only on
  historical or confirmed realtime bar-close execution.
- Added a representative loop/state interaction fixture covering `if`,
  `switch`, `for`, `while`, `break`/`continue`, UDF block bodies, and stateful
  TA callsites in one runtime snapshot without widening accepted syntax.
- Added fixture-backed `array.new_table()` and table-id array support for the
  existing generic array operations, including `array.from` inference and
  shallow `array.copy`. String conversion, `array.join`, `varip` table arrays,
  and linefill/polyline arrays remain unsupported.
- Added fixture-backed `array.new_box()` and box-id array support for the
  existing generic array operations, including `array.from` inference and
  shallow `array.copy`. String conversion, `array.join`, `varip` box arrays,
  and linefill/polyline arrays remain unsupported.
- Added fixture-backed `array.new_label()` and label-id array support for the
  existing generic array operations, including `array.from` inference and
  shallow `array.copy`. String conversion, `array.join`, `varip` label arrays,
  and linefill/polyline arrays remain unsupported.
- Added fixture-backed `array.new_line()` and line-id array support for the
  existing generic array operations, including `array.from` inference and
  shallow `array.copy`. String conversion, `array.join`, `varip` line arrays,
  and linefill/polyline arrays remain unsupported.
- Added fixture-backed drawing object method-call syntax for supported
  label/line/box/table id-first functions. Method calls lower to the existing
  namespace-call runtime paths, so this does not widen unsupported drawing
  methods, chart-point overloads, or unsupported xloc/time variants.
- Added fixture-backed `box.set_xloc()` support for the `xloc.bar_index`
  subset. It updates the latest existing box snapshot's left and right values;
  `na` and deleted boxes remain no-ops, and `xloc.bar_time` stays unsupported.
- Added fixture-backed `line.set_xloc()` support for the `xloc.bar_index`
  subset. It updates the latest existing line snapshot's x1 and x2 values;
  `na` and deleted lines remain no-ops, and `xloc.bar_time` stays unsupported.
- Added fixture-backed `line.get_price()` support over the latest existing
  bar-index line snapshot. It uses x1/y1/x2/y2 interpolation or extrapolation
  and returns `na` for `na`, deleted, vertical, or nonnumeric lines.
- Added fixture-backed `line.get_x1()`, `line.get_y1()`, `line.get_x2()`, and
  `line.get_y2()` support over the latest existing line snapshot. `na` and
  deleted lines return `na`.
- Added `line.new()` initialization support for existing host-neutral line
  snapshot style fields: extend, color, style, and width. The supported
  creation subset remains the x1/y1/x2/y2 overload with `xloc` omitted or
  `xloc.bar_index`; chart-point overloads and time-coordinate lines are still
  unsupported.
- Added `box.new()` initialization support for existing host-neutral box
  snapshot style and text fields, including border/background/extend/text,
  alignment, wrapping, font family, and text-formatting masks. The supported
  creation subset remains the left/top/right/bottom overload with `xloc`
  omitted or `xloc.bar_index`; chart-point overloads and time-coordinate boxes
  are still unsupported.
- Added host-neutral `box.set_text_formatting()` support. Box snapshots now
  carry `textFormatting` masks for none/bold/italic combinations while leaving
  glyph styling to hosts.
- Added host-neutral label text-formatting support. `label.new()` can now
  initialize `textalign`, `text_font_family`, and `text_formatting` snapshot
  fields, and `label.set_text_formatting()` records none/bold/italic formatting
  masks while leaving glyph styling to hosts.
- Closed Strategy Internal Stage 13 Slice 34 omitted-`from_entry`
  `strategy.exit` stop+limit bracket future-entry persistence. A full omitted-
  `from_entry` stop+limit bracket now persists for later pyramided long entries
  with the shared absolute stop and limit prices.
- Closed Strategy Internal Stage 13 Slice 33 omitted-`from_entry`
  `strategy.exit` loss+limit bracket future-entry persistence for unique entry
  ids. A full omitted-`from_entry` loss+limit bracket now persists for later
  pyramided long entries with each later entry's own loss stop and the shared
  absolute limit.
- Closed Strategy Internal Stage 13 Slice 32 omitted-`from_entry`
  `strategy.exit` stop+profit bracket future-entry persistence for unique entry
  ids. A full omitted-`from_entry` stop+profit bracket now persists for later
  pyramided long entries with the shared absolute stop and each later entry's
  own profit target.
- Closed Strategy Internal Stage 13 Slice 31 omitted-`from_entry`
  `strategy.exit` loss+profit bracket future-entry persistence for unique entry
  ids. A full omitted-`from_entry` loss+profit bracket now persists for later
  pyramided long entries and derives both bracket legs from each later entry's
  own fill price.
- Closed Strategy Internal Stage 13 Slice 30 omitted-`from_entry`
  `strategy.exit` loss-tick future-entry persistence for unique entry ids. A
  full omitted-`from_entry` loss exit now persists for later pyramided long
  entries and derives each later entry's stop from that entry's own fill price.
- Closed Strategy Internal Stage 13 Slice 29 omitted-`from_entry`
  `strategy.exit` profit-tick future-entry persistence for unique entry ids. A
  full omitted-`from_entry` profit exit now persists for later pyramided long
  entries and derives each later entry's limit from that entry's own fill price.
- Closed Strategy Internal Stage 13 Slice 28 omitted-`from_entry`
  `strategy.exit` `trail_points+trail_offset` trailing all-entry support for
  current unique entry ids. A full omitted-`from_entry` trail-points exit now
  creates entry-specific trailing exits with each open entry's entry-price-based
  activation.
- Closed Strategy Internal Stage 13 Slice 27 omitted-`from_entry`
  `strategy.exit` `trail_price+trail_offset` trailing all-entry support for
  current open entries. A full omitted-`from_entry` trail-price trailing exit now
  uses the existing all-entry FIFO allocation path to close currently open
  pyramided long entries after trailing activation and stop touch.
- Closed Strategy Internal Stage 13 Slice 26 omitted-`from_entry`
  `strategy.exit` `stop+limit` bracket all-entry support for current open
  entries. A full omitted-`from_entry` stop+limit bracket now uses the existing
  all-entry FIFO allocation path to close currently open pyramided long entries.
- Closed Strategy Internal Stage 13 Slice 25 omitted-`from_entry`
  `strategy.exit` `loss+limit` bracket all-entry support for current unique
  entry ids. A full omitted-`from_entry` loss+limit bracket now creates
  entry-specific bracket exits with entry-specific loss stops and a shared
  absolute limit for currently open pyramided long entries with distinct ids.
- Closed Strategy Internal Stage 13 Slice 24 omitted-`from_entry`
  `strategy.exit` `stop+profit` bracket all-entry support for current unique
  entry ids. A full omitted-`from_entry` stop+profit bracket now creates
  entry-specific bracket exits with a shared absolute stop and entry-specific
  profit targets for currently open pyramided long entries with distinct ids.
- Closed Strategy Internal Stage 13 Slice 23 omitted-`from_entry`
  `strategy.exit` `loss+profit` bracket all-entry support for current unique
  entry ids. A full omitted-`from_entry` loss+profit bracket now creates
  entry-specific bracket exits for currently open pyramided long entries with
  distinct ids.
- Closed Strategy Internal Stage 13 Slice 22 omitted-`from_entry`
  `strategy.exit` loss-tick all-entry support for current unique entry ids. A
  full omitted-`from_entry` loss exit now creates entry-specific stop exits for
  currently open pyramided long entries with distinct ids.
- Closed Strategy Internal Stage 13 Slice 21 omitted-`from_entry`
  `strategy.exit` profit-tick all-entry support for current unique entry ids. A
  full omitted-`from_entry` profit exit now creates entry-specific limit exits
  for currently open pyramided long entries with distinct ids.
- Closed Strategy Internal Stage 13 Slice 20 omitted-`from_entry`
  `strategy.exit` persistent future-entry support for the current absolute
  stop/limit subset. A full omitted-`from_entry` stop/limit exit now expands to
  later pyramided long entries until the position closes.
- Closed Strategy Internal Stage 13 Slice 19 omitted-`from_entry`
  `strategy.exit` current all-entry absolute exit support. Supported stop-only
  and limit-only exits without `from_entry` now close all currently open
  pyramided long entries through the ledger, while persistent future-entry
  behavior remains out of scope.
- Closed Strategy Internal Stage 13 Slice 18 trailing `strategy.exit`
  `trail_points` price basis for pyramided entries. Supported trailing
  activation now converts from the matched open entry price instead of aggregate
  average price.
- Closed Strategy Internal Stage 13 Slice 17 bracket `strategy.exit` tick price
  basis for pyramided entries. Supported bracket `profit`/`loss` relative legs now
  convert from the matched open entry price instead of aggregate average price.
- Closed Strategy Internal Stage 13 Slice 16 same-entry-id `strategy.exit`
  allocation fan-out. A supported exit matching multiple open trades with the
  same entry id now records one public exit order and one closed trade per
  matched ledger allocation.
- Closed Strategy Internal Stage 13 Slice 15 relative `strategy.exit` tick price
  basis for pyramided entries. Supported single-trigger `profit`/`loss` exits now
  convert from the matched open entry price instead of aggregate average price.
- Closed Strategy Internal Stage 13 Slice 14 absolute `strategy.exit` matching
  for pyramided entries. Supported absolute stop/limit exits can now target an
  open long ledger entry by `from_entry`, closing that entry while other
  pyramided entries remain open.
- Closed Strategy Internal Stage 13 Slice 12 multi-entry `strategy.close_all()`.
  Close-all now allocates across all open long ledger entries, records one
  closed trade per matched entry, and flattens aggregate position state.
- Closed Strategy Internal Stage 13 Slice 11 multi-entry `strategy.close(id)`
  matching. Close calls now match and clamp against the requested ledger entry
  id, so one pyramided long entry can close while another remains open.
- Closed Strategy Internal Stage 13 Slice 10 long market pyramiding entry
  foundation. `strategy(..., pyramiding=N)` now accepts positive integer const
  values for same-direction long market entries, appends open trades up to the
  configured limit, and keeps default `pyramiding=1` behavior unchanged.
- Closed Strategy Internal Stage 13 Slice 9 open-trade field ledger reads.
  `strategy.opentrades.*` field helpers now read the requested open-trade index
  from `TradeLedger`, with an internal two-entry test, while accepted scripts
  and public output remain unchanged.
- Closed Strategy Internal Stage 13 Slice 8 open-trade count ledger read.
  `BrokerState::open_trade_count()` now reads `TradeLedger::open_count()` and is
  test-backed for an internal two-entry ledger state, while accepted scripts and
  public output remain unchanged.
- Closed Strategy Internal Stage 13 Slice 7 default pyramiding gate helper.
  `BrokerState` now stores an internal `pyramiding_limit` defaulting to `1`, and
  current long-entry placement/fill paths route through `can_open_long_entry()`
  while preserving no-pyramiding behavior and public output.
- Closed Strategy Internal Stage 13 Slice 6 allocation sync helper. Existing
  long `strategy.close`, supported `strategy.exit`, and long margin-call
  reduction paths now sync aggregate `position_size` and `avg_price` from
  `TradeLedger` after allocation updates, with unchanged public behavior.
- Closed Strategy Internal Stage 13 Slice 5 aggregate position sync helper.
  Long entry fills now sync aggregate `position_size` and `avg_price` from
  `TradeLedger::net_position()` after the ledger update, preserving current
  one-open-trade behavior and public output.
- Closed Strategy Internal Stage 13 Slice 4 `TradeLedger` append helper.
  `open_long()` still preserves the current one-open-trade runtime behavior, and
  the new internal append path is covered by a weighted net-position unit test
  without widening conformance, matrix output, or public JSON.
- Closed Strategy Internal Stage 13 Slice 3 entry fill ownership helper.
  `BrokerState::entry_long()` now routes the existing one-open-long fill
  handoff through a private helper that updates legacy singleton mirrors and
  `TradeLedger` together, with no runtime behavior, conformance, matrix, or
  public JSON change.
- Closed Strategy Internal Stage 13 Slice 2 with a ledger ownership audit. The
  audit records current `TradeLedger` responsibilities, legacy singleton
  `BrokerState` mirrors, aggregate accounting owners, existing unit-test
  evidence, and the migration order needed before any positive `pyramiding` or
  multi-entry behavior.
- Closed Strategy Internal Stage 13 Slice 1 boundary lock. Sema fixture tests
  now assert unsupported `pyramiding` and short-entry diagnostics by message,
  while the repeated-entry runtime test verifies the current no-pyramiding
  one-position behavior without widening conformance, matrix output, or public
  JSON.
- Opened Strategy Internal Stage 13 as a multi-entry ledger and pyramiding design
  gate. The plan records official strategy-entry, pyramiding, close-all, FIFO,
  and generic-order dependencies, documents the current one-net-long broker and
  internal `TradeLedger` baseline, and keeps runtime behavior, conformance,
  matrix output, and public JSON unchanged.
- Closed Strategy Internal Stage 12 with a declaration-property audit. The
  closeout records the fixture-backed unsupported declaration-property boundary,
  the supported `strategy.cash` default quantity subset, unchanged public
  strategy JSON shape across CLI/Python/WASM, and the remaining broker-model
  dependencies for timing, recalculation, currency/precision, shorts, pyramiding,
  OCA, and public order-event behavior.
- Closed Strategy Internal Stage 12 Slice 3 `strategy.cash` default quantity
  support. `strategy(default_qty_type=strategy.cash, default_qty_value=N)` now
  resolves omitted supported `strategy.entry` quantities once at placement time
  as `N / close`, covers market, limit, and explicit-`qty` override fixtures
  across CLI/Python/WASM, and keeps currency conversion, precision rounding,
  lot-step constraints, `currency`, shorts, and pyramiding unsupported.
- Closed Strategy Internal Stage 12 Slice 2 property selection review. The next
  runtime target is `default_qty_type=strategy.cash`, scoped to cash divided by
  current close for omitted supported entry quantities, with explicit `qty`
  precedence preserved and currency conversion, precision rounding, `currency`,
  `strategy.order`, shorts, and pyramiding still out of scope.
- Closed Strategy Internal Stage 12 Slice 1 declaration-property boundary lock.
  The unsupported declaration-property fixture now covers only truly unsupported
  `strategy()` properties, sema tests assert each target property diagnostic by
  name, and the unsupported conformance row registers the declaration-property
  rejection fixtures without widening runtime behavior or public output.
- Opened Strategy Internal Stage 12 as a declaration-property design gate. The
  gap audit now reflects the current supported `strategy()` declaration subset,
  removes already-closed `strategy.close_all()` and `strategy.exit` `qty`
  precedence work from next-step recommendations, and keeps runtime behavior,
  conformance claims, and public output unchanged for this design slice.
- Closed Strategy Internal Stage 11 with a partial `strategy.close` audit. The
  closeout records the fixture-backed full close, fixed-`qty` partial close,
  `qty_percent` partial close, and `qty` precedence subset, confirms unchanged
  public strategy JSON shape across CLI, Python, and WASM, and keeps close
  metadata, `immediately`, partial `strategy.close_all()`, multi-entry
  allocation, and public order-event output unsupported.
- Closed Strategy Internal Stage 11 Slice 3 `qty_percent` partial
  `strategy.close`. `strategy.close(id, qty_percent=...)` now resolves finite
  positive percentages against the current matching long position, clamps
  over-100 percentages to the current position size, keeps invalid percentages
  from mutating broker state, and preserves `qty` precedence when both quantity
  forms are supplied.
- Closed Strategy Internal Stage 11 Slice 2 fixed-`qty` partial
  `strategy.close`. `strategy.close(id, qty=...)` now supports finite positive
  fixed quantities for the current one-net-long broker, clamps oversize closes
  to the matching position, keeps remaining position state open, preserves the
  existing public strategy JSON shape without close order events, and cancels
  matching pending exits only on full flatten.
- Closed Strategy Internal Stage 11 Slice 1 boundary lock. Semantic fixtures
  now prove `strategy.close` partial quantity forms and close metadata options
  remain outside the supported subset before fixed-quantity runtime support is
  introduced.
- Opened Strategy Internal Stage 11 as a partial `strategy.close` design gate.
  The plan targets fixture-backed support for fixed `qty`, `qty_percent`, and
  `qty` over `qty_percent` precedence in the current one-net-long broker while
  keeping runtime behavior, conformance claims, and public output unchanged for
  this design slice.
- Closed Strategy Internal Stage 10 with an active-entry relative bracket
  audit. The closeout records the fixture-backed `stop + profit`,
  `loss + limit`, and `loss + profit` pending-entry bracket subset, confirms
  unchanged public strategy JSON shape across CLI, Python, and WASM, and keeps
  same-side pairs, 3+ triggers, trailing-plus-bracket forms, missing-entry
  future binding, broader broker families, and public schema expansion
  unsupported.
- Closed Strategy Internal Stage 10 Slice 5 `loss + profit` active-entry
  bracket attachment. Same-calculation exits targeting a matching active
  pending long entry can now defer both bracket legs until the actual entry
  fill price is known, then place the existing bracket trigger with unchanged
  public strategy JSON shape across CLI, Python, and WASM.
- Closed Strategy Internal Stage 10 Slice 4 `loss + limit` active-entry
  bracket attachment. Same-calculation exits targeting a matching active
  pending long entry can now defer the loss leg until the actual entry fill
  price is known, then place the existing bracket trigger with unchanged public
  strategy JSON shape across CLI, Python, and WASM.
- Closed Strategy Internal Stage 10 Slice 3 `stop + profit` active-entry
  bracket attachment. Same-calculation exits targeting a matching active
  pending long entry can now defer the profit leg until the actual entry fill
  price is known, then place the existing bracket trigger with unchanged public
  strategy JSON shape across CLI, Python, and WASM.
- Closed Strategy Internal Stage 10 Slice 2 deferred bracket storage. The
  broker can now store, replace, take, cancel, and clear internal active-entry
  relative bracket intent without routing runtime `strategy.exit` calls into
  that storage or widening public behavior.
- Closed Strategy Internal Stage 10 Slice 1 boundary lock. Runtime tests now
  captured the pre-routing boundary where active-entry relative bracket forms
  filled only the matching pending entry and created no public exit orders or
  trades before deferred bracket storage was implemented.
- Opened Strategy Internal Stage 10 as an active-entry relative bracket design
  gate. The plan covers future fixture-backed support for `stop + profit`,
  `loss + limit`, and `loss + profit` against matching active pending long
  entries while keeping runtime behavior, conformance claims, and public output
  unchanged for this design slice.
- Closed Strategy Internal Stage 9 with an entry-relative active-entry exit
  audit. The closeout records the supported single-trigger `profit`, `loss`,
  and `trail_points + trail_offset` pending-entry subset, keeps the public
  strategy result schema unchanged, and leaves active-entry relative brackets
  for a separate bracket-specific design slice.
- Closed Strategy Internal Stage 9 Slice 5 `trail_points + trail_offset`
  active-entry attachment. Same-calculation trailing exits can now attach to a
  matching active pending long entry, resolve activation from the actual entry
  fill price, and preserve activation-bar behavior with CLI, Python, WASM,
  conformance, and matrix evidence while relative-leg active-entry brackets
  remain unsupported.
- Closed Strategy Internal Stage 9 Slice 4 `loss` active-entry attachment.
  Same-calculation `strategy.exit(..., loss=...)` now attaches to a matching
  active pending long entry, resolves the stop price from the actual entry fill
  price, and has CLI, Python, WASM, conformance, and matrix evidence while
  `trail_points` and relative-leg active-entry brackets remain unsupported.
- Closed Strategy Internal Stage 9 Slice 3 `profit` active-entry attachment.
  Same-calculation `strategy.exit(..., profit=...)` now attaches to a matching
  active pending long entry, resolves the take-profit limit from the actual
  entry fill price, and has CLI, Python, WASM, conformance, and matrix evidence
  while `loss` and `trail_points` active-entry attachment remain unsupported.
- Closed Strategy Internal Stage 9 Slice 2 deferred relative trigger skeleton.
  The broker can now store, replace, and clear internal `profit`, `loss`, and
  `trail_points + trail_offset` active-entry exit intent without routing
  runtime calls into that storage or widening public behavior.
- Closed Strategy Internal Stage 9 Slice 1 current boundary lock. Broker tests
  now prove `profit`, `loss`, and `trail_points + trail_offset` active-entry
  attachment remains rejected for current market, limit, stop, and stop-limit
  pending entries, without widening conformance or public output.
- Added Strategy Internal Stage 9 Slice 0 entry-relative active-entry exit
  design gate. The plan targets fixture-backed `strategy.exit` attachment for
  `profit`, `loss`, and `trail_points` against matching active pending entries
  while keeping broader missing-entry, pyramiding, short, reversal, and
  `strategy.order()` behavior unsupported.
- Closed Strategy Internal Stage 8 with a broker expansion audit. The audit
  records the completed behavior-preserving internal order, ledger, allocation,
  and fill-routing skeleton and leaves broader Pine strategy compatibility
  widening to a new staged direction.
- Closed Strategy Internal Stage 8 Slice 16 open long legacy state recorder.
  Current supported long entry fills now use one `OpenTrade` metadata object to
  update both the legacy one-position fields and internal ledger while
  preserving public output and conformance.
- Closed Strategy Internal Stage 8 Slice 15 entry position snapshot routing.
  Current supported long entry fills now write public net-position snapshots
  through the shared internal snapshot recorder while preserving output and
  conformance.
- Closed Strategy Internal Stage 8 Slice 14 entry order event routing. Current
  supported long entry fills now use the shared internal order-event recorder
  while preserving public order output, conformance, Python, and WASM behavior.
- Closed Strategy Internal Stage 8 Slice 13 order event recorder. Long
  margin-call and supported pending `strategy.exit` fills now write existing
  public order events through one internal helper, while `strategy.close`,
  public output, and conformance remain unchanged.
- Closed Strategy Internal Stage 8 Slice 12 position snapshot recorder. Full
  and partial long margin liquidation, `strategy.close`, and supported pending
  `strategy.exit` fills now write existing public net-position snapshots
  through one internal helper while preserving public output and conformance.
- Closed Strategy Internal Stage 8 Slice 11 flat long legacy state cleanup.
  Full long margin liquidation, `strategy.close`, and full supported pending
  `strategy.exit` cleanup now share one internal legacy-state reset helper,
  while ledger allocation application, position snapshots, public trades,
  closed-trade metrics, and conformance remain unchanged.
- Closed Strategy Internal Stage 8 Slice 10 closed trade fill recorder. The
  current long margin-call, `strategy.close`, and supported pending
  `strategy.exit` paths now record existing `StrategyTrade` and
  `ClosedTradeMetrics` outputs through one internal `ClosedTradeFill` helper
  while preserving public output and conformance.
- Closed Strategy Internal Stage 8 Slice 9 allocated entry fill summary. The
  current long margin-call, `strategy.close`, and supported pending
  `strategy.exit` trade-emission paths now share an internal
  `AllocatedEntryFill` summary for allocation metadata and commission fallback
  handling while preserving existing public output and conformance.
- Closed Strategy Internal Stage 8 Slice 8 allocation entry metadata. Internal
  `TradeAllocation` slices now carry entry price, entry bar index, and entry
  time from `OpenTrade`, and current margin-call, `strategy.close`, and
  supported pending `strategy.exit` trade emission reads that metadata while
  preserving existing public output and conformance.
- Closed Strategy Internal Stage 8 Slice 7 single-position exit allocation
  routing. The current long margin-call, `strategy.close`, and supported
  pending `strategy.exit` fill paths now synchronize `TradeLedger` through
  FIFO allocation helpers while preserving existing public output,
  conformance, Python, and WASM behavior.
- Added Strategy Internal Stage 8 Slice 6 internal FIFO allocation helpers.
  `TradeLedger` can now plan omitted-entry and entry-id FIFO allocation slices
  and apply them to internal open trades while rebuilding net position, without
  wiring multiple open trades into runtime behavior or changing conformance,
  public strategy output, Python, or WASM behavior.
- Closed the Strategy Internal Stage 8 Slice 5 first widening candidate by
  choosing an internal-only multiple-open-trade skeleton. `TradeLedger` now
  stores open trades as an internal list and rebuilds net position from that
  list, while current runtime behavior still permits only one supported long
  open trade and keeps public output and conformance unchanged.
- Closed the Strategy Internal Stage 8 Slice 4 multiple-open-trade allocation
  design gate. The plan now fixes future `from_entry`, omitted-`from_entry`,
  FIFO close ordering, partial-exit commission/run-up/drawdown allocation,
  margin-liquidation allocation, public-output boundaries, and concrete fixture
  gates before any pyramiding or multi-entry runtime widening.
- Added the Strategy Internal Stage 8 Slice 3 order-book skeleton. `BrokerState`
  now owns pending entries and exits through an internal `OrderBook` facade
  that delegates to the existing `PendingEntryBook` and `PendingExitBook`,
  preserving cancellation, entry fill, exit reservation, conformance, and
  public output behavior without adding generic orders or OCA support.
- Added the Strategy Internal Stage 8 Slice 2 broker ledger skeleton. The
  runtime now mirrors the existing single long open trade and net position into
  internal `TradeLedger` state across entry, open-trade extremes, partial
  reductions, margin-call reductions, and final flat transitions while keeping
  public strategy output, conformance status, and current metric behavior
  unchanged.
- Closed Strategy Internal Stage 8 Slice 1 boundary lock for the first semantic
  guardrail subset. Added dedicated negative fixtures for unsupported
  `pyramiding=2` and `strategy.exit(..., oca_name=...)` while keeping runtime
  behavior, conformance status, and public output unchanged.
- Added `docs/STRATEGY_INTERNAL_STAGE8_BROKER_EXPANSION_PLAN.md` as the
  design gate for future broker expansion. The plan keeps runtime behavior,
  conformance, and public CLI/Python/WASM strategy output unchanged while
  documenting the intended ledger, order-book, same-bar precedence, OCA, and
  slice sequence before any short, reversal, pyramiding, or generic-order work.
- Closed the current Strategy Internal Stage 7 planning boundary in docs. The
  Stage 7 audit and execution plan now mark the fixture-backed long-only
  trade-record, cost, reporting, default-sizing, and active-margin account
  subset closed, and point the next strategy step at a Stage 8 broker-expansion
  design gate rather than another runtime patch.
- Closed the Strategy Internal active-entry exit attachment evidence slice.
  A same-calculation absolute `strategy.exit` can target a matching active
  pending long entry id and later fill through the existing public
  order/trade/position/equity schema after the entry fills. The new fixture
  covers a supported long limit entry plus attached stop exit across CLI,
  Python, and WASM while keeping unmatched future binding and entry-relative
  pending-entry exits unsupported.
- Implemented Strategy Internal Margin Slice M5. Explicit active `margin_long`
  now supports the first long-only forced-liquidation subset: historical checks
  use `bar.low`, apply TradingView's documented available-funds and
  four-times-cover algorithm with temporary whole-unit truncation, emit existing
  order/trade/position/equity output only, and update
  `strategy.opentrades.capital_held` for the remaining long position.
- Closed Strategy Internal Margin Slice M4 with
  `docs/STRATEGY_INTERNAL_MARGIN_CALL_DESIGN.md`, mapping TradingView's
  documented long margin-call algorithm onto the current long-only broker and
  defining the no-schema-expansion output, timing, and whole-unit truncation
  boundary for the later liquidation implementation.
- Implemented Strategy Internal Margin Slice M3. With explicit active
  `margin_long`, supported long market, limit, stop, and stop-limit entry fills
  now check required margin at the actual fill price, reject overleveraged fills
  with a strategy diagnostic, and keep public strategy output shape unchanged;
  short margin behavior and margin liquidation price remain unsupported.
- Implemented Strategy Internal Margin Slice M2. With explicit active
  `margin_long`, `strategy.opentrades.capital_held` now returns current open
  long market value times `margin_long / 100`, returns `0.0` while flat, and
  preserves `na` in the no-margin subset.
- Implemented Strategy Internal Margin Slice M1. `strategy(..., margin_long=N,
  margin_short=N)` now accepts finite non-negative const numeric declaration
  values and stores their explicit presence in IR.
- Added `docs/STRATEGY_INTERNAL_MARGIN_ACCOUNT_MODEL_PLAN.md` as the design
  gate for future margin/account-model work. The document keeps current
  runtime behavior unchanged while defining the official semantics, non-goals,
  slice order, and stop conditions for any later `margin_long`,
  `margin_short`, `strategy.opentrades.capital_held`, and forced-liquidation
  implementation.
- Added Strategy Internal Stage 7 Slice 35
  `strategy.opentrades.capital_held` as a read-only strategy-mode variable. In
  the current no-margin subset it returns `na`, matching Pine's behavior when a
  strategy does not simulate funding trades with `margin_long` or
  `margin_short`, while public JSON, Python, and WASM strategy schemas remain
  unchanged.
- Added Strategy Internal Stage 7 Slice 34
  `strategy.max_contracts_held_all`, `strategy.max_contracts_held_long`, and
  `strategy.max_contracts_held_short`. Strategy-mode scripts can read maximum
  held quantity metrics for the current long-only subset while public JSON,
  Python, and WASM strategy schemas remain unchanged.
- Added Strategy Internal Stage 7 Slice 33
  `strategy.avg_trade_percent`, `strategy.avg_winning_trade_percent`, and
  `strategy.avg_losing_trade_percent`. Strategy-mode scripts can read average
  per-trade percentage profit/loss values while public JSON, Python, and WASM
  strategy schemas remain unchanged.
- Added Strategy Internal Stage 7 Slice 32
  `strategy.netprofit_percent`, `strategy.grossprofit_percent`, and
  `strategy.grossloss_percent`. Strategy-mode scripts can read realized
  profit/loss percentages relative to `initial_capital`, while public JSON,
  Python, and WASM strategy schemas remain unchanged.
- Added Strategy Internal Stage 7 Slice 31
  `default_qty_type=strategy.percent_of_equity`.
  Supported long entries without explicit `qty` now resolve their default
  quantity from current supported equity and current close at placement time,
  while cash sizing, margin behavior beyond the later explicit-margin subset,
  and currency conversion remain unsupported.
- Added Strategy Internal Stage 7 Slice 30
  `strategy.max_runup_percent` and `strategy.max_drawdown_percent`.
  Strategy-mode scripts can read maximum intrabar equity run-up/drawdown
  percentages over the current supported long-only trading interval, while
  keeping public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 28 `strategy.max_runup`.
  Strategy-mode scripts can read maximum intrabar equity run-up amount over the
  current supported long-only trading interval, while keeping public JSON,
  Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 27 `strategy.max_drawdown`.
  Strategy-mode scripts can read maximum intrabar equity drawdown amount over
  the current supported trading interval, while keeping public JSON, Python,
  and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 26 `strategy.avg_losing_trade`.
  Strategy-mode scripts can read average realized loss among losing closed
  trades only as a positive value, with `na` before the first losing closed
  trade, while keeping public JSON, Python, and WASM strategy schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 25 `strategy.avg_winning_trade`.
  Strategy-mode scripts can read average realized profit among winning closed
  trades only, with `na` before the first winning closed trade, while keeping
  public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 24 `strategy.avg_trade`. Strategy-mode
  scripts can read average realized profit/loss per closed trade, with `na`
  before the first closed trade, while keeping public JSON, Python, and WASM
  strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 23 `strategy.grossloss`. Strategy-mode
  scripts can read cumulative realized closed-trade loss as a positive series
  that excludes winning, flat, and current open trades while keeping public
  JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 22 `strategy.grossprofit`. Strategy-mode
  scripts can read a cumulative positive realized closed-trade profit series
  that excludes losing, flat, and current open trades while keeping public JSON,
  Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 21 percent commission accounting.
  `strategy(..., commission_type=strategy.commission.percent,
  commission_value=N)` now debits `qty * fill_price * N / 100` on supported
  entry and exit fills, updates cash, equity, closed trade profit,
  `strategy.netprofit`, and closed/open trade `commission()` field functions,
  and keeps public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 20 fixed-tick limit verification.
  `strategy(..., backtest_fill_limits_assumption=N)` now accepts finite
  non-negative integer const ticks, requires supported long limit entry and
  supported long limit/profit exit fills to move that many fixed
  `syminfo.mintick` ticks beyond the limit price, preserves the original limit
  fill price, and keeps public JSON, Python, and WASM strategy schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 19 fixed-tick slippage. `strategy(...,
  slippage=N)` now accepts finite non-negative integer const ticks, converts
  them through the fixed `syminfo.mintick` subset, worsens supported long entry
  fill prices upward and supported long close/exit fill prices downward after
  trigger selection, and keeps public JSON, Python, and WASM strategy schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 18 cash-per-order commission
  accounting. `strategy(...,
  commission_type=strategy.commission.cash_per_order, commission_value=N)` now
  applies one fixed commission per supported entry and exit fill, allocates
  entry commission across partial closes, updates cash, equity, closed trade
  profit, `strategy.netprofit`, and the closed/open trade `commission()` field
  functions, and keeps public JSON, Python, and WASM strategy schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 17 cash-per-contract commission
  accounting. `strategy(...,
  commission_type=strategy.commission.cash_per_contract, commission_value=N)`
  now applies entry and exit commission to cash, equity, closed trade profit,
  `strategy.netprofit`, and the closed/open trade `commission()` field
  functions while leaving public JSON, Python, and WASM strategy schemas
  unchanged; unsupported commission modes beyond the current listed subset plus
  richer fill models remain unsupported.
- Added Strategy Internal Stage 7 Slice 16 closed-trade `max_drawdown()` field
  function. It exposes the largest low-based adverse excursion retained for the
  closed trade quantity, follows the same zero-based `trade_num` contract,
  returns `na` for invalid indexes, and keeps public JSON, Python, and WASM
  strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 15 closed-trade `max_runup()` field
  function. It exposes the largest high-based favorable excursion retained for
  the closed trade quantity, follows the same zero-based `trade_num` contract,
  returns `na` for invalid indexes, and keeps public JSON, Python, and WASM
  strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 14 open-trade `max_drawdown()` field
  function. It exposes the largest low-based adverse excursion seen so far for
  the current supported long position when `trade_num == 0`, returns `na` when
  flat or for invalid indexes, and keeps public JSON, Python, and WASM strategy
  schemas unchanged.
- Added Strategy Internal Stage 7 Slice 13 open-trade `max_runup()` field
  function. It exposes the largest high-based favorable excursion seen so far
  for the current supported long position when `trade_num == 0`, returns `na`
  when flat or for invalid indexes, and keeps public JSON, Python, and WASM
  strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 12 open-trade `commission()` field
  function. It exposes `0.0` without configured commission, later reports the
  supported cash-per-contract entry commission when configured, returns `na`
  when flat or for invalid indexes, and keeps public JSON, Python, and WASM
  strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 11 open-trade `entry_id()` field
  function. It exposes the current supported long position entry id when
  `trade_num == 0`, returns `na` when flat or for invalid indexes, and keeps
  public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 10 open-trade `profit()` field
  function. It exposes the current close-based floating profit for the current
  supported long position when `trade_num == 0`, returns `na` when flat or for
  invalid indexes, and keeps public JSON, Python, and WASM strategy schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 9 open-trade `size()` field function.
  It exposes the current supported long position size for `trade_num == 0`,
  returns `na` when flat or for invalid indexes, and keeps public JSON, Python,
  and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 8 open-trade `entry_time()` field
  function. It exposes the current supported long position's entry fill
  timestamp for `trade_num == 0`, returns `na` when flat or for invalid
  indexes, and keeps public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 7 open-trade `entry_bar_index()` field
  function. It exposes the current supported long position's entry fill bar for
  `trade_num == 0`, returns `na` when flat or for invalid indexes, and keeps
  public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 6 open-trade `entry_price()` field
  function. It exposes the current supported long position's entry price for
  `trade_num == 0`, returns `na` when flat or for invalid indexes, and keeps
  public JSON, Python, and WASM strategy schemas unchanged.
- Added Strategy Internal Stage 7 Slice 5 closed-trade `exit_id()` field
  function. It returns the retained close id for `strategy.close` /
  `strategy.close_all` fills and the pending exit id for `strategy.exit` fills,
  follows the same zero-based `trade_num` contract, and keeps public JSON,
  Python, and WASM strategy trade schemas unchanged.
- Added Strategy Internal Stage 7 Slice 4 closed-trade `entry_id()` field
  function. It returns the entry id already retained on closed trade records,
  follows the same zero-based `trade_num` contract, returns `na` for invalid
  indexes, and keeps public JSON, Python, and WASM strategy trade schemas
  unchanged.
- Added Strategy Internal Stage 7 Slice 3 closed-trade `commission()` field
  function. It follows the same zero-based `trade_num` contract as the existing
  script-visible closed-trade fields and returns `0.0` without configured
  commission, later reports supported entry-plus-exit cash-per-contract
  commission when configured, and keeps public JSON, Python, and WASM strategy
  trade schemas unchanged.
- Added Strategy Internal Stage 7 Slice 2 closed-trade `entry_time` and
  `exit_time` field functions. They expose the timestamps already retained on
  closed trade records, use the same zero-based `trade_num` contract, and keep
  the public runtime output shape unchanged.
- Added Strategy Internal Stage 7 Slice 1 closed-trade `size` and `profit`
  field functions. They use the same zero-based `trade_num` contract as the
  existing closed-trade field subset, return `na` for invalid or out-of-range
  indexes, and keep the public runtime output shape unchanged. Open-trade
  namespaces, runup/drawdown, ids, times, and richer reporting metrics remain
  unsupported.
- Added Strategy Internal Stage 7 Slice 0 closed-trade field functions:
  `strategy.closedtrades.entry_price`, `strategy.closedtrades.exit_price`,
  `strategy.closedtrades.entry_bar_index`, and
  `strategy.closedtrades.exit_bar_index`. These read the existing closed-trade
  list in strategy-mode scripts with zero-based integer `trade_num` indexes,
  return `na` for invalid or out-of-range indexes, and keep the public runtime
  output shape unchanged. Open-trade namespaces, additional closed-trade fields,
  and richer reporting metrics remain unsupported.
- Added Strategy Internal Stage 6 Slice 1 `strategy.cancel_all()` support for
  the current supported pending-entry and pending-exit subset. The call clears
  all supported internal pending orders, is a no-op when none exist, and keeps
  the public strategy output shape unchanged. Generic order APIs, OCA groups,
  pyramiding, shorts, and reversals remain unsupported.
- Added Strategy Internal Stage 6 Slice 0 `strategy.cancel(id)` support for
  the current supported pending-entry and pending-exit subset. Matching pending
  ids are cancelled internally, filled or unknown ids are no-op, and the public
  strategy output shape remains unchanged. Generic order APIs, OCA groups,
  pyramiding, shorts, and reversals remain unsupported.
- Added Strategy Internal Stage 5 Slice 2 long `strategy.entry(..., stop=...,
  limit=...)` stop-limit support. Supported long stop-limit entries use the
  existing internal pending-entry model, activate on a later historical bar when
  `high >= stop`, do not fill on that activation bar, and fill at the limit
  price on a later historical bar when `low <= limit`, with no public
  pending-order output. Shorts, pyramiding, and generic order APIs remain
  unsupported.
- Added Strategy Internal Stage 5 Slice 1 long `strategy.entry(..., stop=...)`
  support. Supported long stop entries use the existing internal pending-entry
  model, never fill on their creation bar, and fill at the stop price on a later
  historical bar when `high >= stop`, with no public pending-order output.
  Shorts, pyramiding, and generic order APIs remain unsupported.
- Added Strategy Internal Stage 5 Slice 0 long `strategy.entry(..., limit=...)`
  support. Supported long limit entries use the existing internal pending-entry
  model, never fill on their creation bar, and fill at the limit price on a
  later historical bar when `low <= limit`, with no public pending-order output.
  Shorts, pyramiding, and generic order APIs remain unsupported.
- Added Strategy Internal Stage 4 `strategy.exit` quantity precedence. For
  supported single-trigger, bracket, and trailing exit shapes, calls that supply
  both `qty` and `qty_percent` now follow Pine-compatible precedence where fixed
  `qty` determines the reserved or filled quantity and `qty_percent` is ignored.
  Unsupported trigger combinations remain rejected, and the public strategy
  output shape is unchanged.
- Added Strategy Internal Stage 3 Slice 1 trade outcome count variables.
  `strategy.wintrades`, `strategy.losstrades`, and `strategy.eventrades` are
  read-only strategy-mode series int counts derived from the current closed
  trade list by positive, negative, and zero realized profit. Rich trade
  namespace functions and broader reporting metrics remain unsupported, and the
  public strategy output shape is unchanged.
- Added Strategy Internal Stage 3 Slice 0 `strategy.close_all()` support for
  the current one-net-long broker. Strategy-mode scripts can close the current
  supported long position at the current bar close without naming the entry id;
  flat or already-closed calls are no-op, pending exits for the closed entry are
  cancelled, and the existing public strategy output shape is unchanged.
- Added Strategy Internal Stage 2 pending-entry timing. Supported market-long
  `strategy.entry` calls now create an internal pending entry and fill at the
  next historical bar open, with no public pending-order output. Same-calculation
  absolute `strategy.exit` attachment for the active pending entry id is
  supported, including fixed `qty` or `qty_percent` reservation against the
  pending entry quantity; same-calculation entry-relative `profit`, `loss`, and
  `trail_points` attachment remains unsupported until deferred price resolution
  is designed. Public strategy output shape remains unchanged.
- Closed Strategy Internal Stage 1 boundary lock documentation. The Stage 1
  audit records the current fixture-backed strategy support and unsupported
  boundary, aligns current reservation wording for single-trigger, bracket, and
  trailing `strategy.exit` reservations, and adds negative semantic fixtures
  for unsupported strategy declaration properties plus unsupported
  order/trade/risk namespaces. Runtime behavior and public strategy output are
  unchanged; conformance and matrix snapshots are synchronized to the added
  semantic guard coverage.
- Closed Phase Z for the omitted-quantity `strategy.exit` boundary. Omitted
  `qty` and omitted `qty_percent` exits keep full-position
  one-effective-pending behavior across supported single-trigger, bracket, and
  trailing forms, and a later omitted full-position exit clears earlier explicit
  reservations for the current matching long entry. Runtime fixtures and
  CLI/Python/WASM host tests cover the boundary. Omitted-quantity multiple
  reservations, missing-entry pre-placement, public pending/reservation fields,
  pyramiding, shorts, and richer broker behavior remain unsupported.
- Added Phase Y `strategy.exit` trailing reservations for explicit fixed `qty`
  or `qty_percent` trailing exits on the current matching long entry. Supported
  trailing reservation forms remain `trail_price + trail_offset` and
  `trail_points + trail_offset`. Different `id + from_entry` identities can keep
  multiple internal pending trailing reservations, and trailing reservations can
  share the reservation pool with Phase W single-trigger and Phase X bracket
  reservations. Inactive trailing reservations activate on a later eligible bar
  without filling on that bar; active trailing reservations fill as downside
  candidates before same-bar ratchets, and otherwise ratchet upward only. Public
  runtime output remains `schemaVersion: 3` and continues to expose only
  `orders`, `trades`, `position`, `equity`, and `diagnostics` under `strategy`.
  Omitted-quantity multiple reservations, missing-entry pre-placement, public
  pending/trailing-state records, pyramiding, shorts, and richer broker behavior
  remain unsupported.
- Added Phase X `strategy.exit` bracket reservations for explicit fixed `qty`
  or `qty_percent` one-downside/one-upside brackets on the current matching
  long entry. Different `id + from_entry` identities can keep multiple internal
  pending bracket reservations, and bracket reservations can share the
  reservation pool with Phase W single-trigger reservations. Same identities
  replace the previous reservation after releasing it; new reservations resolve
  at placement time, clamp to remaining unreserved position quantity, and are
  rejected with strategy diagnostics when no quantity remains. Same-side
  touched candidates fill in placement order, mixed downside/upside same-bar
  touches process downside candidates only, and a both-leg bracket touch
  contributes the bracket's downside candidate. Public runtime output remains
  `schemaVersion: 3` and continues to expose only `orders`, `trades`,
  `position`, `equity`, and `diagnostics` under `strategy`. Omitted-quantity
  multiple reservations, missing-entry pre-placement, public pending-order
  records, pyramiding, shorts, and richer broker behavior remain unsupported.
- Added Phase W `strategy.exit` quantity reservations for explicit fixed
  `qty` or `qty_percent` single-trigger exits on the current matching long
  entry. Different `id + from_entry` identities can keep multiple internal
  pending reservations; same identities replace the previous reservation after
  releasing it. New reservations resolve at placement time, clamp to remaining
  unreserved position quantity, and are rejected with strategy diagnostics when
  no quantity remains. Same-side touched exits fill in placement order, while
  mixed downside/upside same-bar touches process downside candidates only.
  Public runtime output remains `schemaVersion: 3` and continues to expose only
  `orders`, `trades`, `position`, `equity`, and `diagnostics` under
  `strategy`. Omitted-quantity multiple reservations, missing-entry
  pre-placement, public pending-order records, pyramiding, shorts, and richer
  broker behavior remain unsupported.
- Closed Phase V for the current fixture-backed `strategy.exit(...,
  qty_percent=...)` subset. The audit records supported single-trigger,
  one-downside/one-upside bracket, and trailing percent exits; placement-time
  percent-to-absolute resolution; fill-time clamping; unchanged public runtime
  schema; host coverage; and the full release verification gate.
- Added Phase V `strategy.exit(..., qty_percent=...)` support on the existing
  supported single-trigger, one-downside/one-upside bracket, and trailing exit
  subsets. `qty_percent` is evaluated at placement time, must be finite and
  positive, resolves to an absolute requested close quantity against the current
  position size, and fills no more than the current position. Public strategy
  outputs continue to expose absolute order/trade `qty` values with no schema
  bump. Quantity reservation, multiple pending exits, missing-entry
  pre-placement, pyramiding, shorts, and richer broker behavior remain
  unsupported.
- Added Phase U fixed `strategy.exit(..., qty=...)` support for the existing
  single-trigger, one-downside/one-upside bracket, and trailing exit subsets.
  Fixed `qty` is evaluated at placement time, must be finite and positive,
  fills `min(qty, position_size)`, leaves any remaining long position open at
  the same average price, and keeps the public strategy result shape and
  runtime `schemaVersion: 3`. Phase U did not add `qty_percent`, quantity
  reservation, multiple pending exits, missing-entry pre-placement, pyramiding,
  shorts, or richer broker behavior.
- Added Phase T WASM request-bars host injection for the existing
  provider-backed `request.security` subset. WASM hosts can now pass explicit
  `requestBarsJson` data through `runScriptCsvWithRequestBars`,
  `runScriptCsvWithLibrariesAndRequestBars`, and
  `Program.runCsvWithRequestBars`, while request semantics, conformance status,
  and runtime `schemaVersion: 3` remain unchanged.
- Added the Phase S `strategy.exit` trailing-stop subset. Supported forms are
  exactly `trail_price + trail_offset` and `trail_points + trail_offset` for
  the current long-only, no-pyramiding broker. Trailing exits activate on a
  later eligible historical bar, do not fill on the activation bar, ratchet the
  active stop upward only, emit one existing `strategy.exit` order event and one
  closed trade when filled, keep runtime `schemaVersion: 3`, and leave invalid
  trailing combinations, partial exits, missing-entry pre-placement, and richer
  broker behavior unsupported.
- Closed Phase R for the first positive `strategy.exit` bracket subset.
  Supported brackets are exactly `stop + limit`, `stop + profit`,
  `loss + limit`, and `loss + profit` for the current long-only,
  no-pyramiding broker. A bracket is one pending full-position exit, uses
  stop/loss-first precedence when both legs are touched on the same eligible
  historical bar, emits one `strategy.exit` order event and one closed trade,
  keeps runtime `schemaVersion: 3`, and leaves same-side pairs, 3+ triggers,
  trailing stops, partial exits, missing-entry pre-placement, and richer broker
  behavior unsupported.
- Closed Phase Q as a `strategy.exit` bracket design gate. At Phase Q close,
  combined trigger exits remained unsupported, and the audit recorded the
  future one-downside plus one-upside bracket subset, stop/loss-first same-bar
  precedence, identity/replacement rules, invalid-leg behavior, fixture plan,
  and module implementation blueprint, with the full release verification gate
  passing on the closeout workspace.
- Hardened `strategy.exit` unsupported diagnostics to use phase-neutral
  current-subset wording and added a diagnostic-only four-trigger
  `stop + limit + profit + loss` fixture, with conformance and matrix metadata
  updated without widening support.
- Implemented Phase P broker-structure maintenance. Strategy broker internals
  are split into facade, pending-exit, fill, accounting, and broker-test
  modules while preserving the existing strategy compatibility surface, public
  runtime `schemaVersion: 3`, CLI/Python/WASM output shapes, and runtime
  snapshots.
- Recorded Phase Q as the next strategy maintenance target after Phase P:
  a bracket design gate to specify same-bar precedence, bracket identity, and
  interaction with the current one-pending-exit model before any later support
  claim.
- Closed Phase O for the current fixture-backed strategy reporting count
  subset. The audit records supported `strategy.closedtrades` and
  `strategy.opentrades` count variables, explicit unsupported reporting
  namespaces and rich metrics, unchanged public runtime schema, host coverage,
  and the full release verification gate.
- Added Phase O strategy reporting count compatibility metadata and docs for
  `strategy.closedtrades` and `strategy.opentrades`. The supported subset is
  strategy-mode historical series int counts for the current long-only broker;
  rich trade namespaces, public open-trade records, and broader reporting
  metrics remain unsupported, with no runtime schema bump.
- Closed Phase N for the current fixture-backed `strategy.exit` profit/loss
  subset. The audit records supported profit-only and loss-only tick-distance
  exits, unsupported bracket/trailing/partial forms, unchanged public runtime
  schema, host coverage, and the full release verification gate.
- Closed Phase N Slice 7 as a bracket design gate. Combined trigger exits stay
  unsupported, including stop/limit, profit/loss, mixed price/tick, and
  three-trigger calls, because OHLC-only same-bar precedence remains a future
  broker design task.
- Hardened Phase N Slice 6 profit/loss exit interactions. Runtime fixtures now
  cover profit placement and later loss replacement through branch, switch, for,
  and while contexts, plus strategy state/history reads around the delayed loss
  fill.
- Added Phase N Slice 4 runtime fixtures and compatibility metadata for
  `strategy.exit(id, from_entry, profit=ticks)` and
  `strategy.exit(id, from_entry, loss=ticks)`. Profit/loss exits convert
  positive tick distances from the current long average entry price using the
  fixed default `syminfo.mintick`, reuse the Phase M pending-exit lifecycle,
  and keep combined trigger forms unsupported.
- Closed Phase M for the current fixture-backed `strategy.exit` subset. The
  audit records supported stop-only and limit-only full-position exits,
  unchanged public runtime `schemaVersion: 3` behavior, host coverage,
  maintenance tails, and release-gate verification.
- Completed Phase M Slice 7 public contract hardening without a runtime schema
  bump. Existing strategy `orders`, `trades`, `position`, `equity`, and
  `diagnostics` fields fully represent filled stop/limit exits across CLI,
  Python, and WASM, with no pending-order or exit-reason public fields added.
- Hardened Phase M Slice 6 strategy-exit interactions. Runtime fixtures now
  cover exit placement through branch, switch, for, and while contexts plus
  strategy state/history reads around an exit fill, and incremental append
  execution checks the new fixture.
- Closed Phase M Slice 5 with combined stop/limit exits intentionally
  unsupported. Stop-only and limit-only exits remain the supported deterministic
  subsets; combined brackets need an explicit same-bar high/low precedence
  policy before compatibility can be claimed.
- Added Phase M Slice 4 limit-exit fills. The supported `strategy.exit` subset
  now includes `strategy.exit(id, from_entry, limit=price)` for full-position
  long exits, triggering on later historical bars when `high >= limit` and
  sharing the same order, trade, position, equity, CLI, Python, and WASM
  contract as stop exits.
- Added Phase M Slice 3 stop-exit fills. The supported
  `strategy.exit(id, from_entry, stop=price)` subset now creates or replaces a
  full-position pending stop for the matching current long entry, fills on a
  later historical bar when `low <= stop`, records a `strategy.exit` order event
  plus closed trade, and is covered through CLI snapshots, Python bindings, and
  WASM JSON.
- Added Phase M Slice 2 broker-owned pending state for stop-only
  `strategy.exit`. Accepted calls now place or replace one internal pending
  stop for the matching current long entry, `strategy.close(id)` cancels that
  pending exit, and missing or mismatched entries produce a stable strategy
  diagnostic without changing the public runtime output shape.
- Added Phase M Slice 1 semantic staging for stop-only `strategy.exit`. The
  analyzer accepts `strategy.exit(id, from_entry, stop=price)` in strategy-mode
  scripts and keeps unsupported exit variants diagnostic-only before executable
  fills are claimed.
- Locked Phase M Slice 0 strategy-exit boundaries. The decision record selects
  stop-only `strategy.exit` as the first executable target, keeps combined,
  requested-context, and function-side-effect exit forms fixture-backed
  unsupported, and avoids public strategy schema changes before runtime support
  lands.
- Closed Phase L for the current strategy usability subset. The audit records
  supported strategy state variables, fixed default quantity behavior, public
  host coverage, `strategy.exit` design boundaries, remaining maintenance tails,
  and release-gate verification.
- Completed Phase L Slice 5 as a `strategy.exit` design gate. Stop, limit,
  profit/loss, trailing, partial quantity, and missing-entry exit forms now have
  explicit unsupported fixtures, and no pending-order or exit schema fields are
  added.
- Added Phase L Slice 4 fixed default quantity support. Strategy declarations
  now accept `default_qty_type=strategy.fixed` with positive const numeric
  `default_qty_value`; `strategy.entry(id, strategy.long)` uses that default,
  while explicit `qty` continues to override it.
- Hardened Phase L Slice 3 strategy variable interactions. The supported
  position/profit/equity state variables now have fixture-backed behavior in
  branches, switches, loops, pure UDF arguments, constant history references,
  incremental append execution, profile retention, and public host smoke tests;
  mutation and requested-context usage remain rejected.
- Added Phase L Slice 2 profit and equity state variables. Strategy-mode
  historical scripts can read and plot `strategy.openprofit`,
  `strategy.netprofit`, and `strategy.equity` for the current long-only broker
  subset. Expression-time `strategy.netprofit` is realized closed-trade profit
  only; the existing public strategy snapshot field `netProfit` remains
  `equity - initial_capital` and can include open profit while a position is
  open.
- Added Phase L Slice 1 position state variables. Strategy-mode historical
  scripts can read and plot `strategy.position_size` and
  `strategy.position_avg_price`; values follow the current long-only broker
  state and update immediately after supported entry/close calls.
- Locked Phase L Slice 0 strategy state-variable boundaries. Known Phase L
  strategy variables now have fixture-backed pre-implementation diagnostics, and
  broad `strategy.*` remains unsupported for unimplemented state/reporting
  helpers.
- Closed Phase G for the first fixture-backed strategy runtime subset. The
  audit records the supported declaration, long entry, full close, trade,
  position, and equity surface plus explicit maintenance tails for richer order
  types, broker settings, strategy variables, alerts, and realtime broker
  rollback.
- Added Phase G Slice 5 strategy equity snapshots and basic
  `initial_capital` handling. Strategy declarations accept positive const
  numeric `initial_capital`, long entry/close accounting updates cash, and the
  public strategy result now includes per-bar `cash`, `marketValue`, `equity`,
  and `netProfit` snapshots.
- Added Phase G Slice 4 minimal `strategy.close` support. Strategy scripts can
  fully close an existing long entry at the current bar close and receive a
  deterministic closed-trade record with entry/exit bars, prices, quantity, and
  profit; missing or repeated closes are no-ops.
- Added Phase G Slice 3 minimal `strategy.entry` support. Strategy scripts can
  open one long market position with `strategy.entry(id, strategy.long, qty=...)`
  filled at the current bar close; repeated entries are ignored under the
  current no-pyramiding rule, and short/stop/limit/indicator-mode variants
  remain rejected by semantic diagnostics.
- Added Phase G Slice 2 strategy runtime/output scaffolding. Strategy-mode
  scripts now return an empty `strategy` result contract with `orders`,
  `trades`, `position`, `equity`, and `diagnostics` arrays across CLI, Python,
  and WASM, while indicator output keys remain unchanged and order functions
  remain unsupported.
- Added Phase G Slice 1 strategy declaration scaffolding. `strategy(...)` is
  accepted as a declaration-only partial feature with strategy HIR mode
  metadata, while strategy order functions remain unsupported.
- Locked Phase G Slice 0 unsupported diagnostics for the reserved strategy
  surface. `strategy(...)`, `strategy.entry`, `strategy.exit`, and
  `strategy.close` now use fixture-backed `E_UNSUPPORTED_FEATURE` diagnostics
  while `strategy.*` remains unsupported in the conformance matrix.
- Closed Phase J for the fixture-backed libraries/imports/user-types/methods
  subset. The closeout audit records supported host-provided import behavior,
  local scalar UDTs, pure local UDT methods, and explicit maintenance tails.
- Locked Phase J Slice 9 imported UDT/method boundaries. Imported UDT identity
  and imported methods remain unsupported maintenance tails while source-graph
  imports continue to support exported constants and pure functions only.
- Added Phase J Slice 8 user-defined methods for pure methods on local UDT
  receivers with scalar parameters. Calls lower through the existing inlined
  function-body path with the receiver as the first internal parameter; side
  effects, recursion, imported methods, unknown receivers, and unsupported
  parameter families remain rejected.
- Clarified Phase J Slice 7 UDT storage semantics. Local UDT values are
  immutable, may be stored in ordinary variables and `var`, and roll back with
  confirmed `var` state during realtime forming updates. UDT `varip`, history
  references, and field mutation remain diagnostic-only unsupported forms.
- Added Phase J Slice 6 local user-defined type support for top-level scalar
  field declarations, `Type.new(...)` constructors, and field reads. UDT
  history references, field mutation, nested UDT fields, arrays of UDTs,
  imported UDTs, and advanced method forms remain unsupported.
- Added Phase J Slice 5 host parity for imported functions. CLI integration
  fixtures now run the import subset through `--library-source`, Python binding
  tests cover imported function execution through `library_sources`, and WASM
  exposes deterministic JSON library source maps via
  `compileScriptWithLibraries`, `analyzeScriptWithLibraries`, and
  `runScriptCsvWithLibraries`.
- Added Phase J Slice 4 executable import subset. Host-provided exact-key
  imports with aliases can now use exported const expressions and pure exported
  functions through `alias.name`; imported functions reuse existing UDF
  lowering/runtime behavior, including independent callsite state. The import
  conformance row is now `partial`; unaliased imports, missing host sources,
  private or unknown exports, non-const exported constants, side-effecting
  exported functions, re-exports, imported UDTs, and imported methods remain
  rejected.
- Added Phase J Slice 3 module graph validation while keeping imports
  non-executable. Analysis now validates host-provided library sources for
  missing import keys, duplicate root aliases, invalid library declarations,
  duplicate exports, dependency cycles, unknown exports, and private symbol
  access.
- Added Phase J Slice 2 parser structure for imports, library declarations,
  export declarations, user-defined type declarations, and user-defined method
  declarations. These nodes now preserve import keys, aliases, declaration
  names, fields, method parameters, bodies, and spans for future source-graph
  analysis.
- Added Phase J Slice 1 source graph scaffolding: `AnalysisInput`,
  deterministic source ids, normalized library source keys, duplicate/invalid
  key rejection, and compile-cache keys that include host-provided library
  source text. CLI now accepts repeated `--library-source KEY=path.pine`
  options for `analyze` and `run`; Python accepts `library_sources` dictionaries
  on `compile_script`, `analyze_script`, and `run_script`; WASM parity is
  covered by the later Slice 5 host contract.
- Started Phase J Slice 0 by locking the diagnostic-only boundary for
  `library`, `export`, user-defined type declarations, and user-defined method
  declarations with unsupported sema fixtures and conformance rows. Method
  calls outside later fixture-backed array and local UDT method subsets remain
  ordinary receiver/type diagnostics.
- Closed Phase I with `docs/PHASE_I_AUDIT.md`, fixture-backed scalar and scalar
  typed-array `varip` conformance rows, host-surface review for CLI/Python/WASM
  historical paths, and explicit maintenance tails for drawing ids, tuples,
  maps, matrices, UDTs, imports, and unimplemented value families.
- Added the scalar and scalar typed-array `varip` executable subset: global and
  local int/float/bool/string/color/`na` declarations now behave like `var`
  during historical execution and preserve intrabar state across repeated
  realtime forming updates. Supported float/int/bool/string/color array ids also
  retain their backing contents across repeated forming updates, including
  branch-local declaration sites and `array.copy` boundaries. Array mutation
  inside UDFs remains rejected by existing function side-effect rules. Drawing
  ids are rejected with a dedicated diagnostic because object stores still roll
  back; tuples and other value families remain unsupported.
- Added the first `request.security` executable subset:
  `request.security(syminfo.tickerid, timeframe.period, expression)` returns the
  scalar side-effect-free expression in the current chart context.
- Added same-or-higher-timeframe host dataset injection for
  `request.security("SYMBOL", timeframe, expression)` lookups in Rust
  runtime, CLI `--request-bars SYMBOL:TIMEFRAME=bars.csv`, and Python
  `request_bars` dictionaries. WASM request dataset injection was a documented
  temporary Phase F gap and is now closed for this subset by Phase T; optional
  parameters, explicit gaps/lookahead, and lower timeframe requests remain
  unsupported.
- Widened provider-backed `request.security` to evaluate scalar requested
  expressions in an isolated requested context with deterministic callsite
  caching and default higher-timeframe `gaps_off`/`lookahead_off` alignment.
  The supported provider expression subset now includes direct OHLCV/time
  sources, pure arithmetic and ternaries, history references, `na`, `nz`,
  `ta.sma`, and `ta.ema`; provider local aliases, side effects, and unsupported
  calls are rejected during semantic analysis where possible.
- Documented the lower-timeframe request boundary: lower-timeframe
  `request.security` remains runtime-rejected with a stable error, and
  `request.security_lower_tf` remains unsupported until typed array return
  semantics and host output shapes are designed.
- Added cross-host request contract fixtures for CLI and Python request dataset
  injection, plus conformance validation that prevents partial `request.*`
  claims without request-specific fixtures. WASM request dataset injection was
  the remaining temporary host gap until the Phase T JSON host shape.
- Closed Phase F with `docs/PHASE_F_AUDIT.md`, fixture-backed request matrix
  rows, same-or-higher-timeframe request contract coverage across Rust, CLI,
  and Python, plus a documented WASM provider-data gap later closed by Phase T.
- Started Phase E drawing-object infrastructure by bumping the public
  machine-readable contract to `schemaVersion: 2` and adding the
  top-level `labels` output across CLI JSON, Python dictionaries, and WASM JSON.
  It is empty for scripts that do not create supported drawing objects.
- Added the first drawing behavior: a `label.new` creation subset that returns
  deterministic label ids and emits sparse creation snapshots with bar-index
  coordinates, price y-values, text, colors, selected label styles, size, and
  tooltip metadata.
- Added sparse mutation snapshots for the initial `label.set_*` subset covering
  x/y/text/color/style/size/tooltip fields.
- Added `label.delete` lifecycle snapshots and a deterministic 500-label
  runtime limit. Label creation, mutation, and deletion now have fixture-backed
  realtime rollback, and drawing side effects inside user-defined functions are
  rejected under the existing side-effect policy. Unsupported coordinate modes
  and advanced label methods remain unsupported.
- Added the initial `line.*` lifecycle: deterministic line ids, sparse public
  `lines` snapshots for creation/mutation/deletion, selected endpoint/color/
  width/style/extend mutators, realtime rollback coverage, and a deterministic
  500-line runtime limit. Advanced line methods remain unsupported.
- Added the initial `box.*` lifecycle: deterministic box ids, sparse public
  `boxes` snapshots for creation/mutation/deletion, selected geometry/
  background/border mutators, realtime rollback coverage, and a deterministic
  500-box runtime limit. Advanced box methods remain unsupported.
- Added the initial `table.*` lifecycle: deterministic table ids, sparse public
  `tables` snapshots for fixed-dimension table creation and `table.cell`
  text/background/text-color writes, realtime rollback coverage, a
  deterministic 50-table runtime limit, and a 1000-cell per-table limit.
  Advanced table methods plus polyline drawing families remain unsupported.
- Kept `polyline.*` explicitly unsupported for Phase E because it depends on a
  future `chart.point` value and point-array design; the decision is captured in
  `docs/PHASE_E_POLYLINE_GATE.md`.
- Closed Phase E with `docs/PHASE_E_AUDIT.md`, fixture-backed drawing matrix
  rows, schemaVersion 2 drawing output coverage across CLI/Python/WASM, and
  family-split runtime drawing built-ins.
- Closed Phase K release infrastructure with public `schemaVersion: 1` output
  contracts for CLI, Python, and WASM public machine-readable outputs.
- Moved CLI and WASM runtime JSON onto shared runtime serialization helpers,
  with Python binding tests asserting the same public runtime key contract.
- Added golden JSON snapshots for representative CLI runtime output, CLI matrix
  JSON, and WASM analysis JSON.
- Hardened `tests/fixtures/conformance.tsv` validation so compatibility matrix
  claims require unique features, valid statuses, notes, existing fixtures, and
  status-appropriate fixture coverage.
- Added `scripts/verify.sh` as the canonical local and CI release verification
  entry point.
- Added deterministic runtime profile fixture gates for long TA histories,
  many stateful callsites, array-heavy scripts, and dynamic history retention.
- Added partial `switch` expression support for condition arms, selector/case
  arms, expression results, default arms, and conditional stateful-call
  execution.
- Added partial `while` statement support with bool conditions, `break`,
  `continue`, scoped loop bodies, and a runtime iteration guard.
- Added coverage for stateful callsite advancement inside `for` and `while`
  loop bodies.
- Added `ta.supertrend` line/direction tuple support for the fixture-covered
  ATR-based subset.
- Added `ta.dmi` `+DI`/`-DI`/`ADX` tuple support using the runtime's existing
  Wilder/RMA-style smoothing behavior.
- Added `ta.sar` Parabolic SAR support with callsite state and prior-bar
  high/low clamping.
- Added `ta.mfi` Money Flow Index support over ready positive/negative
  money-flow windows using source and volume.
- Added `ta.tsi` True Strength Index support using short/long EMA smoothing of
  source momentum and absolute momentum.
- Added `ta.cmo` Chande Momentum Oscillator support over ready rolling
  positive/negative source-change windows.
- Added `ta.cci` Commodity Channel Index support over ready source mean
  deviation windows.
- Added `ta.cog` Center of Gravity support over ready source windows.
- Added `ta.ao` Awesome Oscillator support as the fast/slow SMA spread of
  median price.
- Added `ta.bop` Balance of Power support over current OHLC values.
- Added `ta.kc` and `ta.kcw` Keltner Channel support using source/range EMA
  state.
- Added `ta.pivothigh` and `ta.pivotlow` support for confirmed left/right pivot
  windows.
- Added `ta.pivot_point_levels` support for runtime-bar anchored pivot level
  arrays across Traditional, Fibonacci, Woodie, Classic, DM, and Camarilla
  formulas.
- Added `ta.wpr` Williams %R support over ready rolling high/low windows and
  current close.
- Added `ta.stoch` four-argument stochastic oscillator support over ready
  rolling high/low windows.
- Added partial float array support with runtime-owned array ids,
  `array.new_float`, `array.push`, `array.get`, `array.set`, `array.size`,
  `array.pop`, and `array.clear`.
- Added partial int array support through `array.new_int` and the existing
  size/get/set/push/pop/clear operations.
- Added partial bool array support through `array.new_bool` and the existing
  size/get/set/push/pop/clear operations.
- Added partial string array support through `array.new_string` and the
  existing size/get/set/push/pop/clear operations.
- Added partial color array support through `array.new_color` and the existing
  size/get/set/push/pop/clear operations.
- Added `array.from` support for inferred float/int/bool/string/color typed
  arrays.
- Added array helper support for `array.first`, `array.last`, `array.shift`,
  and `array.unshift` across supported typed arrays.
- Added `array.insert` and `array.remove` support for supported typed arrays,
  including method-call syntax and array element limit checks.
- Added negative indexing support for `array.get`, `array.set`,
  `array.insert`, and `array.remove`.
- Added `array.fill` support for supported typed arrays, including optional
  half-open range bounds and method-call syntax.
- Added `array.copy` support for explicitly creating independent typed-array
  snapshots while plain array assignment remains id/reference-based.
- Added array search helpers `array.includes`, `array.indexof`,
  `array.lastindexof`, and numeric `array.binary_search*` variants.
- Added array truth helpers `array.every` and `array.some` for float, int, and
  bool arrays.
- Added numeric array statistics helpers `array.min`, `array.max`,
  `array.sum`, `array.avg`, `array.range`, `array.median`, `array.mode`,
  `array.percentile_nearest_rank`, `array.percentile_linear_interpolation`,
  `array.percentrank`, `array.covariance`, `array.standardize`,
  `array.variance`, and `array.stdev`, plus same-kind `array.abs`, for float
  and int arrays.
- Added array ordering helpers: `array.sort` for numeric/string arrays with
  optional order direction, `array.sort_indices` for numeric/string arrays with
  optional order direction, and `array.reverse` for all supported typed arrays.
- Added `array.join` support for supported typed arrays with optional string
  separators.
- Added `array.slice` and `array.concat` support for supported typed arrays,
  including method-call syntax and array element limit checks.
- Added partial array method-call syntax for supported array `size`, `get`,
  `set`, `insert`, `push`, `pop`, `remove`, `shift`, `unshift`, `first`,
  `last`, `fill`, `copy`, `slice`, `concat`, `includes`, `indexof`,
  `lastindexof`, `every`, `some`, numeric `binary_search*`, `min`, `max`,
  `sum`, `avg`, `range`, `median`, `mode`, `percentile_nearest_rank`,
  `percentile_linear_interpolation`, `percentrank`, `variance`, `stdev`,
  `sort`, `reverse`, `join`, and `clear`.
- Added `input.string` support for the executable `defval`/`title` subset.
- Added `input.price`, `input.time`, `input.symbol`, and `input.timeframe`
  support for the executable `defval`/`title` subset.
- Added generic `input` support for const int, float, bool, string, and color
  defaults.
- Added common `input.*` metadata parameters, including min/max/step,
  `options`, `tooltip`, `inline`, `group`, `confirm`, and `display` where they
  fit the supported input kinds.
- Added UTC-derived `year`, `month`, `dayofmonth`, `hour`, `minute`, and
  `second` bar time component variables.
- Added UTC-only function overloads for `year`, `month`, `dayofmonth`, `hour`,
  `minute`, and `second`.
- Added a numeric UTC subset of `timestamp`.
- Added `barstate.isfirst`, `barstate.islast`, `barstate.isnew`,
  `barstate.isconfirmed`, `barstate.ishistory`, and `barstate.isrealtime`.
- Added fixed-default regular-session `session.ismarket`,
  `session.ispremarket`, and `session.ispostmarket`.
- Added `bgcolor` and `barcolor` support with bar-aligned color output series.
- Added common `plot`, `hline`, `fill`, `bgcolor`, and `barcolor` metadata
  parameters for style/display compatibility; runtime output series remain
  unshifted by display metadata in this subset.
- Added basic `plotchar` support with bar-aligned values, chars, and colors.
- Expanded `plotchar` compatibility with common marker metadata parameters;
  runtime output remains normalized to value, char, and color series.
- Expanded `plotshape` and `plotarrow` compatibility with common marker
  metadata parameters while preserving the existing normalized output schemas.
- Expanded `plotbar` and `plotcandle` compatibility with common display
  metadata parameters while preserving existing OHLC output schemas.
- Added `color.from_gradient` with fixture-covered linear RGBA interpolation.
- Added conformance coverage for hex color literals as const colors.
- Added `input.session` and `input.text_area` defval execution with metadata
  validation.
- Added direct `display.pane`, `display.price_scale`, `display.status_line`,
  and `display.data_window` metadata constants.
- Accepted `confirm` metadata on `input.source`.
- Added explicit `str.tostring` handling for `format.price` and
  `format.volume`.
- Added dedicated conformance coverage for global OHLCV, derived price
  (`hl2`, `hlc3`, `hlcc4`, `ohlc4`), time, and `bar_index` series.
- Added basic `plotshape` support with bar-aligned values, style, location,
  color, text, text color, and size marker output.
- Added basic `plotarrow` support with bar-aligned numeric values, up/down
  colors, and height bounds.
- Added basic `plotbar` support with bar-aligned OHLC values and optional
  colors.
- Added basic `plotcandle` support with bar-aligned OHLC values plus body,
  wick, and border colors.
- Added `color.rgb` support for numeric RGB channels and optional transparency.
- Added optional transparency defaulting for `color.new`.
- Added `color.r`, `color.g`, `color.b`, and `color.t` channel extraction.
- Added `str.length`, `str.upper`, and `str.lower` string helpers.
- Added `str.contains`, `str.startswith`, and `str.endswith` string predicates.
- Added `str.pos` and `str.substring` string extraction helpers.
- Added `str.trim` and `str.repeat` string modification helpers.
- Added `str.replace` and `str.replace_all` string replacement helpers.
- Added `str.tonumber` numeric string parsing.
- Added `str.tostring` scalar and float-array string conversion.
- Added `str.format` indexed placeholder string formatting.
- Added `str.match` regex substring matching.
- Added `str.split` support for literal separators and empty-separator
  character splitting.
- Added a UTC subset of `str.format_time` timestamp formatting.
- Added UTC `weekofyear` and `dayofweek` calendar variables/functions plus
  `dayofweek.*` constants.
- Added fixed-default `time_close` using the current 1-minute chart timeframe
  subset.
- Added fixed-default `timeframe.period` and `timeframe.in_seconds` support for
  common seconds/minutes/days/weeks/months timeframe strings, plus
  `timeframe.from_seconds` for the exact reverse conversion subset.
- Added `timeframe.change` UTC bucket detection for the supported timeframe
  string subset.
- Added fixed-default `timeframe.is*` and `timeframe.multiplier` chart
  timeframe metadata.
- Added `int`, `float`, `bool`, `string`, and `color` scalar type casts for
  numeric, bool, string, color, and `na` values.
- Added `fixnan` support for carrying forward the last non-`na` numeric or
  color value at each callsite.
- Added `math.floor` and `math.ceil` support for numeric values.
- Added `math.sqrt`, `math.log`, and `math.pow` support for numeric values.
- Added `math.trunc`, `math.cbrt`, and `math.hypot` support for numeric
  values.
- Added `math.sin`, `math.cos`, and `math.tan` support for numeric values.
- Added `math.log10` and `math.exp` support for numeric values.
- Added `math.acos`, `math.asin`, and `math.atan` support for numeric values.
- Added `math.sign`, `math.todegrees`, and `math.toradians` support for numeric values.
- Added `math.avg` support for variadic numeric averages.
- Added `math.e`, `math.pi`, `math.phi`, and `math.rphi` constants.
- Added `precision` argument support for `math.round`.
- Added `math.round_to_mintick` support using the current default
  `syminfo.mintick` subset value.
- Added fixed-default `syminfo.*` metadata for common ticker, exchange,
  currency, session, timezone, tick-size, and price-scale fields.
- Added deterministic callsite-backed `math.random` support with optional
  `min`, `max`, and `seed` arguments.
- Added `math.sum` support for rolling source sums with simple-int lengths.
- Added `ta.stdev` support with default biased and optional sample standard
  deviation modes.
- Added `ta.variance` support with the same biased/sample window modes as
  `ta.stdev`.
- Added `ta.range` support for rolling highest-minus-lowest values.
- Added `ta.dev` support for rolling average absolute deviation.
- Added `ta.vwma` support for rolling volume-weighted moving averages.
- Added `ta.wma` support for linearly weighted moving averages.
- Added `ta.hma` support for Hull moving averages composed from internal WMA
  windows.
- Added `ta.swma` support for fixed four-bar symmetric weighted moving
  averages.
- Added `ta.alma` support for Arnaud Legoux moving averages with optional
  floored offset centers.
- Added `ta.dema` and `ta.tema` support for double and triple EMA-chain
  smoothing.
- Added `ta.linreg` support for rolling least-squares linear regression values.
- Added `ta.bbw` support for Bollinger Bands Width values.
- Added `ta.cum` support for cumulative numeric source sums.
- Added `ta.max` and `ta.min` support for all-time source extremes.
- Added `ta.tr` support as a built-in true range series variable.
- Added `ta.accdist` support as the built-in Accumulation/Distribution index
  series variable.
- Added `ta.iii` support as the built-in Intraday Intensity Index series
  variable.
- Added `ta.nvi` and `ta.pvi` support as built-in Negative/Positive Volume
  Index series variables.
- Added `ta.obv` support as the built-in On Balance Volume series variable.
- Added `ta.pvt` support as the built-in Price Volume Trend series variable.
- Added partial `ta.vwap` support for the variable form, one-argument source
  call, source/anchor call, and source/anchor/bands tuple call as runtime-bar
  cumulative VWAP; session-derived anchoring remains future work.
- Added `ta.wad` support as the built-in Williams Accumulation/Distribution
  series variable.
- Added `ta.wvad` support as the built-in Williams Variable
  Accumulation/Distribution series variable.
- Added `ta.mom` support for source momentum over explicit history lengths.
- Added `ta.roc` support for rate-of-change percentages over explicit history
  lengths.
- Expanded `ta.change` to support series int and bool sources.
- Expanded core TA source signatures to accept series int sources where the
  runtime already evaluates numeric windows through floating-point values.
- Added explicit fixture coverage for simple numeric sources in rolling
  correlation, covariance, median, mode, percentile, and percent-rank helpers.
- Added `ta.correlation` support for rolling Pearson correlation coefficients.
- Added `ta.covariance` support for rolling population covariance values.
- Added `ta.median` and `ta.mode` support for rolling sorted-window statistics.
- Added `ta.percentile_nearest_rank` support for rolling nearest-rank
  percentile values.
- Added `ta.percentile_linear_interpolation` support for rolling interpolated
  percentile values.
- Added `ta.percentrank` support for rolling percent-rank values.
- Added `ta.rising` and `ta.falling` support for current-vs-previous-window
  trend checks.
- Added two-argument `ta.highestbars` and `ta.lowestbars` support for rolling
  extreme offsets.
- Added `ta.barssince` support for tracking bars elapsed since the last true
  condition.
- Added `ta.valuewhen` support for retrieving source values from prior true
  condition occurrences.
- Added length-only overloads for `ta.highest`, `ta.lowest`,
  `ta.highestbars`, and `ta.lowestbars`.
- Tightened float array UDF boundaries: read-only array operations are allowed,
  while array mutation inside UDFs is rejected as a side effect.
- Added a 100,000-element runtime guard for each float array.

## v0.1 Baseline

This release establishes the first executable Pine-compatible indicator subset.
Compatibility claims are backed by `tests/fixtures/conformance.tsv`; run
`pine-compat matrix` or `pine-compat matrix --format json` to inspect the
feature-level matrix and its fixture paths.

Machine-readable public outputs use top-level `schemaVersion`. Runtime,
analysis, and matrix outputs now have separate schema constants:
`PUBLIC_RUNTIME_SCHEMA_VERSION`, `PUBLIC_ANALYSIS_SCHEMA_VERSION`, and
`PUBLIC_MATRIX_SCHEMA_VERSION`. Runtime output is now `schemaVersion: 3` with a
reserved top-level `alerts` array. Analysis and matrix outputs remain
`schemaVersion: 2`; increment only the affected contract when an intentional
consumer-visible output change is documented with snapshot updates.

### Runtime Surfaces

- Rust crates for syntax, semantic analysis, HIR, built-ins, runtime, CLI,
  Python bindings, and WASM bindings.
- CLI commands for analysis, AST formatting, historical execution, profiling,
  and compatibility matrix output.
- Python binding exposing compile, analyze, and run entry points.
- WASM binding exposing compile, analyze, and CSV execution entry points.
- Public JSON/dictionary outputs for CLI, Python, and WASM expose
  `schemaVersion`; CLI and WASM runtime JSON share the same runtime contract
  helper.
- Runtime outputs include an `alerts` array for Phase H alert events.
  `alertcondition(condition, title, message)` is partially supported for
  bool-compatible conditions and const-string title/message, and
  `alert(message, freq?)` is partially supported for const-string messages and
  the fixture-backed `alert.freq_once_per_bar`/`alert.freq_all`/
  `alert.freq_once_per_bar_close` frequency subset. Reached true conditions and
  reached alert calls emit deterministic `{id, barIndex, time, message,
  source}` events in program order; forming realtime events roll back until
  confirmed, and close-frequency alert calls emit only on historical or
  confirmed realtime bar-close execution. TradingView-style placeholder
  interpolation remains unsupported until deterministic semantics are designed.
- The compatibility matrix source of truth is
  `tests/fixtures/conformance.tsv`; generated text and JSON matrix output must
  remain fixture-backed.

### Supported Executable Subset

- Indicator scripts over OHLCV bar input.
- Historical bar-by-bar execution and incremental append execution.
- Realtime forming-bar rollback for output, `var`, and stateful callsite state.
- Constant non-negative history references.
- Normal declarations, reassignment, tuple declarations, tuple-returning
  built-ins, and tuple `for` expression results.
- `if`/`else` blocks, nested blocks, and conditional stateful calls that advance
  only when their branch executes.
- `for` loops over inclusive integer ranges, explicit non-zero `by` steps,
  `break`, `continue`, scalar loop results, and tuple loop results.
- Local scopes for block declarations, tuple declarations, shadowing, and local
  `var` declaration-site storage.
- User-defined functions with expression bodies and multi-statement block
  bodies, positional and named arguments, single evaluation of arguments,
  function-local declarations, local `var`, loops inside functions, and
  independent state per syntactic callsite.
- `na`, `nz`, `indicator`, `input.*`, `plot`, `hline`, `fill`, `color.new`,
  selected named colors, selected `math.*` functions, and the fixture-covered
  `ta.*` built-ins listed in the compatibility matrix.
- Typed scalar arrays for float, int, bool, string, and color through the
  fixture-covered `array.*` subset documented as partial in the matrix.

### Partial Support

- `for`: supports inclusive integer ranges, loop control, and loop results, but
  does not claim full Pine loop compatibility.
- `history references`: supports constant non-negative offsets and guarded
  dynamic integer offsets, including `series int`, loop-produced offsets, and
  user-defined function parameters.
- `max_bars_back`: supports indicator-level constant non-negative retention
  bounds for dynamic history.
- `color.*` named constants: supports the current common registry only.
- `realtime forming rollback`: covers output, alert events,
  supported drawing objects, `var`, scalar and scalar typed-array `varip`,
  callsite, array, and dynamic history rollback.

### Explicitly Unsupported

The analyzer rejects these boundaries with diagnostics instead of approximating
them silently:

- `varip` drawing ids, tuple `varip`, and `varip` value families outside the
  scalar and scalar typed-array subset.
- `request.*` multi-symbol and multi-timeframe data requests.
- `strategy.*` broker emulation and backtesting.
- Generic arrays, object arrays, user-defined type arrays, matrices, maps, and
  deferred collection semantics that are not fixture-backed in the current
  `array.*` partial subset.
- Imports and external libraries.
- Alert frequency controls.
- Advanced drawing object methods and unsupported `polyline.*` point-list
  object systems.
- Per-variable `max_bars_back` declarations and inference.
- Recursive user-defined functions.
- User-defined function side effects, including output calls, alerts,
  input declarations, indicator declarations, array mutation, global
  reassignment, and passing side-effecting calls as UDF arguments.

### Verification

The release baseline is expected to pass:

```text
scripts/verify.sh
```

Snapshot updates are intentional public-contract changes. Refresh them with the
commands in `docs/CONFORMANCE.md`, review the JSON diff, then run
`scripts/verify.sh`.
