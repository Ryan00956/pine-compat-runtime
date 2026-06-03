# Strategy Internal Execution Plan

Status: planning document.

This document turns `docs/STRATEGY_INTERNAL_GAP_AUDIT.md` into an executable
eight-stage strategy roadmap. It does not claim new support. A stage becomes
supported only after syntax, semantic analysis, runtime behavior, fixtures,
conformance metadata, snapshots, documentation, and release verification are
complete.

The stages below are intentionally ordered for dependency management. Do not
execute the gap-audit inventory numbers mechanically; that inventory is a
coverage checklist, not the implementation order.

## Execution Rules

- Work one small slice at a time.
- Start every slice by rechecking `tests/fixtures/conformance.tsv`, matrix
  output, current phase audits, and the relevant strategy modules.
- Keep unsupported variants explicitly rejected with diagnostics or documented
  no-op behavior.
- Preserve the public runtime output shape unless the stage explicitly designs a
  new host contract.
- Do not widen conformance claims before fixture-backed runtime and host parity
  evidence exists.
- If repo evidence and plan wording diverge, stop and reconcile the document or
  report the blocker before implementation.

## Stage 1: Boundary Lock

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE1_BOUNDARY_AUDIT.md`.

Goal: freeze the current strategy boundary before adding new behavior.

Scope:

- Reconcile known documentation drift between current conformance evidence and
  older phase-audit wording.
- Confirm the current fixture-backed strategy subset:
  - long-only one-net-position broker;
  - fixed default quantity and explicit positive `qty`;
  - `strategy.entry`, `strategy.close`, supported `strategy.exit` shapes;
  - `qty` and `qty_percent` partial exits;
  - explicit fixed-`qty` or `qty_percent` reservations for supported
    single-trigger, bracket, and trailing exits;
  - supported position, profit, equity, and trade-count variables.
- Add or update negative semantic fixtures only where unsupported strategy
  forms are not clearly guarded.

Out of scope:

- Runtime compatibility widening.
- Public output schema changes.
- Reinterpreting unsupported Pine behavior as supported.

Acceptance:

- Current support and unsupported boundaries agree across conformance, matrix,
  docs, and strategy audit wording.
- Unsupported declaration properties, order APIs, risk APIs, trade namespace
  functions, and unsupported `strategy.exit` combinations remain guarded.
- `scripts/verify.sh` passes before this stage is considered closed.

## Stage 2: Pending Entry And Order Timing Foundation

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE2_PENDING_ENTRY_AUDIT.md`.

Goal: introduce the internal pending-entry model needed by Pine-compatible order
timing.

Scope:

- Add active entry orders that can exist before fill.
- Preserve public output unless an entry actually fills.
- Fixture the selected default historical fill policy; Stage 2 selected
  next-historical-bar-open behavior for market entries.
- Support same-calculation `strategy.entry` plus `strategy.exit` attachment when
  the exit refers to the active entry id.

Out of scope:

- Arbitrary future binding for unmatched `from_entry` ids.
- Entry limit, stop, or stop-limit orders.
- Pyramiding, shorts, reversals, or generic pending-order cancellation.
- Realtime strategy handoff and bar magnifier fills.

Acceptance:

- Pending entries are internally represented and tested without exposing an
  unstable public pending-order shape.
- Same-calculation entry/exit attachment has semantic, broker, runtime,
  incremental, and host parity coverage.
- Missing-entry exits keep the documented unsupported or no-op boundary.

## Stage 3: Small Independent Strategy Utilities

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE3_UTILITIES_AUDIT.md`.

Goal: add narrow, high-value helpers that do not require the full broker model.

Scope:

- Implement `strategy.close_all()` for the current one-net-long model.
- Add win/loss/even trade count variables for the current closed-trade list.
- Keep these as small slices; either item can be skipped if Stage 2 exposes a
  blocker that should be handled first.

Out of scope:

- Partial `strategy.close`.
- Close ordering across multiple entries.
- Rich trade namespace functions.
- Commission, slippage, margin, or account-model changes.

Acceptance:

- Each helper has accepted and rejected semantic fixtures.
- Runtime fixtures prove behavior in flat, open-position, and already-closed
  cases.
- Public output shape is unchanged unless a separate contract is explicitly
  designed.

## Stage 4: Pine-Compatible `qty + qty_percent`

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE4_QTY_PRECEDENCE_AUDIT.md`.

Goal: align `strategy.exit` quantity precedence with Pine where `qty` wins over
`qty_percent`.

Scope:

- Replace the current diagnostic-only rejection for `qty + qty_percent` with a
  fixture-backed rule where `qty` determines the reserved or filled quantity.
- Cover supported single-trigger, bracket, and trailing exit shapes.
- Preserve existing placement-time quantity evaluation and fill-time clamping
  rules.

Out of scope:

- New trigger families.
- Omitted-quantity multiple reservations.
- Multiple-entry or pyramiding quantity allocation.

Acceptance:

- Analyzer and runtime agree on the same accepted forms.
- Negative fixtures still reject unsupported trigger combinations even when
  `qty_percent` is present.
- Public output continues to expose absolute filled quantities only.

## Stage 5: Entry Limit, Stop, And Stop-Limit Orders

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE5_ENTRY_ORDERS_AUDIT.md`.

Goal: extend `strategy.entry` beyond market-long entries after pending-entry
timing is stable.

Scope:

- Add long entry `limit`, `stop`, and stop-limit order forms in the current
  one-net-long model.
- Reuse the Stage 2 pending-entry timing foundation.
- Define deterministic historical trigger and fill behavior for each order form.

Out of scope:

- Short entries and automatic reversals.
- Pyramiding.
- Generic `strategy.order`.
- Cancellation APIs unless Stage 6 is already complete.
- Bar magnifier or lower-timeframe reconstruction.

Acceptance:

- Entry order forms have semantic fixtures, broker tests, runtime snapshots, and
  incremental parity coverage.
- Repeated entries under no-pyramiding keep the documented behavior.
- Supported entry orders interact correctly with same-calculation exit
  attachment.

## Stage 6: General Pending-Order Book And Cancellation

Status: closed on 2026-06-02. See
`docs/STRATEGY_INTERNAL_STAGE6_CANCELLATION_AUDIT.md`.

Goal: add cancellation semantics after pending orders are no longer
exit-specific.

Scope:

- Introduce a general internal pending-order book spanning supported pending
  entries and exits.
- Implement `strategy.cancel(id)` for supported pending order ids.
- Implement `strategy.cancel_all()` for all supported pending orders.

Out of scope:

- Generic `strategy.order`.
- Custom OCA groups.
- Pyramiding, shorts, reversals, and multi-entry ledgers.
- Public pending-order records unless separately designed.

Acceptance:

- Cancel behavior is deterministic for pending entries, pending exits, filled
  orders, flat positions, and unknown ids.
- Runtime and incremental paths agree.
- CLI, Python, and WASM parity tests cover at least one cancellation fixture if
  host-visible output changes.

## Stage 7: Trade Records, Costs, And Account Model

Status: in progress. Slices 0, 1, 2, and 3 closed on 2026-06-02; Slices 4,
5, 6, 7, 8, 9, 10, and 11 closed on 2026-06-03; see
`docs/STRATEGY_INTERNAL_STAGE7_TRADE_RECORDS_AUDIT.md`.

Goal: enrich strategy reporting and accounting without jumping directly to a
multi-position broker.

Scope:

- Add a small individual trade namespace subset, starting with closed-trade
  `entry_price`, `exit_price`, `entry_bar_index`, and `exit_bar_index` if the
  script-variable-only contract is acceptable.
- Add selected cost or account-model slices only after the output contract is
  explicit.
- Candidate cost/account slices include commission, slippage, cash sizing, and
  percent-of-equity sizing.

Out of scope:

- Full trade namespace coverage.
- Runup and drawdown until per-trade lifecycle state is retained.
- Margin and forced liquidation until account constraints are designed.
- Public JSON expansion without an explicit host contract.

Acceptance:

- Script-variable support and public-output decisions are documented before
  implementation.
- Closed-trade field slices keep their functions script-visible only and do not
  expand public CLI/Python/WASM runtime schema.
- Open-trade field slices keep their functions script-visible only and limited
  to the documented one-net-long broker subset unless a later slice designs a
  broader ledger.
- Profit, equity, and trade-count behavior remain internally consistent when
  costs or account sizing are introduced.
- Existing strategy fixtures either remain unchanged or are intentionally
  updated with clear audit notes.

## Stage 8: Full Broker Expansion

Goal: move from the current one-net-long model toward Pine's broader broker
semantics.

Scope:

- Multiple entries and per-entry trade ledgers.
- Pyramiding.
- Short positions and automatic reversals.
- Generic `strategy.order()`.
- Full OCA behavior, including custom OCA names and OCA behavior across mixed
  order families.

Out of scope:

- Treating any of these as small compatibility patches.
- Widening public claims before the broker model, same-bar precedence, account
  interactions, and host contracts are designed together.

Acceptance:

- The internal broker can represent separate open trades, net position, and
  order-family state without losing current one-net-long behavior.
- Same-bar precedence and order allocation are fixture-backed.
- Public output and host bindings are updated only through an explicit schema
  and release plan.

## Shared Completion Gates

Every stage or slice must close with:

- semantic fixtures for accepted and rejected forms;
- broker unit tests for state transitions and accounting;
- runtime fixtures and golden snapshots;
- incremental/runtime interaction tests where state timing matters;
- CLI, Python, and WASM parity tests when public output or host behavior
  changes;
- synchronized `tests/fixtures/conformance.tsv`, matrix snapshot, docs, and
  release notes;
- a phase audit recording the supported and unsupported boundary;
- `scripts/verify.sh`.
