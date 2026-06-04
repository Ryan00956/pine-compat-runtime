# Strategy Internal Active-Entry Exit Attachment Plan

Status: open. No compatibility claim widens until each slice closes with
fixtures, conformance metadata, host parity, docs, and verification evidence.

This plan defines the next narrow strategy-maintenance phase after the
long-only margin account subset closed through
`docs/STRATEGY_INTERNAL_MARGIN_ACCOUNT_MODEL_PLAN.md` Slice M5.

The target is not arbitrary future binding. The target is the Pine-compatible
case where a supported `strategy.exit(...)` call can attach to an active
matching `strategy.entry(...)` order that exists before it fills. TradingView's
strategy manual shows same-block `strategy.entry("buy", ...)` plus
`strategy.exit("exit", "buy", ...)` and states that a `from_entry` value with no
matching current-position entry creates no exit orders. This plan keeps that
boundary: unmatched missing-entry exits do not persist forever waiting for a
future unrelated entry.

Primary official reference:

- TradingView Pine Script strategies, `strategy.exit()` and invalid
  `from_entry` examples:
  https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Starting Point

The current repo baseline is:

- `strategy.entry` supports long market, limit, stop, and stop-limit entries.
- Pending long entries are internal-only and have no public pending-order
  output.
- The broker is still one-net-long, no short exposure, no reversals, and no
  pyramiding.
- `strategy.exit` supports the fixture-backed single-trigger, bracket,
  trailing, explicit-quantity reservation, omitted-quantity replacement, and
  supported quantity-percent subsets.
- Supported `strategy.exit` calls currently require an open matching long
  position at placement time; a matching active entry that has not filled yet is
  not part of the supported boundary.
- Runtime output remains `schemaVersion: 3` and exposes no pending-entry,
  pending-exit, reservation-ledger, or exit-reason records.

## Goal

Support the first active-entry exit attachment subset without changing the
public strategy output schema.

The positive subset is:

- one supported long active entry order;
- matching `from_entry` id;
- supported current `strategy.exit` trigger and quantity shapes;
- exit placement after the matching entry placement in the same historical
  calculation;
- fill ordering remains entry fill first, then attached exit can evaluate on
  later eligible bars according to the existing trigger rules;
- unmatched `from_entry` ids remain no-op or diagnostic behavior and do not
  create future persistent exits.

## Non-Goals

- No arbitrary future binding for missing-entry `from_entry` ids.
- No `strategy.exit` without `from_entry` persistence for later entries.
- No public pending-order or reservation output fields.
- No short exposure, reversals, pyramiding, or per-entry open-trade ledgers.
- No `strategy.order`, OCA APIs, realtime rollback, bar magnifier, or
  intrabar-path reconstruction.
- No broader order metadata such as comments, alert messages, or exit reasons.

## Slice A0: Boundary And Fixture Audit

Confirm the current behavior and lock the exact supported/unsupported boundary.

Acceptance:

- current negative behavior is captured with at least one runtime or semantic
  fixture if it is not already covered;
- conformance wording distinguishes active-entry attachment from arbitrary
  future binding;
- no runtime behavior changes.

Stop condition:

- stop if live code already persists missing-entry exits broadly, because that
  would need a compatibility correction plan before widening support.

## Slice A1: Matching Active Long Entry Attachment

Allow a supported `strategy.exit` call to attach to a matching active long
entry that was placed before the exit call and has not filled yet.

Contract:

- matching is by `from_entry` id against current open position first, then
  active long entry ids;
- missing ids still create no exit orders;
- exit trigger validation, quantity validation, replacement, and reservation
  semantics reuse the current pending-exit path;
- when the entry later fills, the attached exit becomes effective for that
  filled long position;
- if the active entry is canceled, rejected, or superseded before fill, attached
  exits for that entry id are cleared;
- public JSON shape remains unchanged.

Tests:

- runtime fixture with long limit or stop entry plus same-calculation
  `strategy.exit(..., from_entry=...)` that closes after the entry fills;
- unmatched `from_entry` fixture showing no future persistent exit;
- conformance row updates for `strategy.entry`, `strategy.exit`, and broad
  `strategy.*` unsupported boundary;
- CLI snapshot plus Python and WASM plot parity for the positive fixture.

Stop condition:

- stop if the current pending-entry model cannot identify active entries without
  adding public pending-order state or a multi-entry ledger.

## Slice A2: Replacement And Quantity Interaction

Extend the A1 attachment path to the already-supported replacement and explicit
quantity reservation cases for one matching active entry.

Contract:

- same `id + from_entry` replacement behaves the same before and after entry
  fill;
- explicit fixed `qty` and `qty_percent` reserve against the eventual supported
  entry quantity when the broker can determine it safely;
- omitted quantity continues to use the one-effective-pending replacement path;
- no multiple-entry or pyramiding behavior is claimed.

Tests:

- repeated same-id active-entry exit replacement;
- explicit fixed-quantity active-entry exit;
- explicit `qty_percent` active-entry exit;
- host parity for one representative fixture.

Stop condition:

- stop if reservation sizing cannot be made deterministic before entry fill for
  the current default-quantity and percent-of-equity entry subsets.

## Slice A3: Closeout

Close the phase after the smallest positive subset is implemented and verified.

Acceptance:

- `tests/fixtures/conformance.tsv`, `docs/CONFORMANCE.md`,
  `docs/EXECUTION_SEMANTICS.md`, `docs/BUILTIN_SIGNATURES.md`,
  `docs/STRATEGY_INTERNAL_GAP_AUDIT.md`, and `docs/RELEASE_NOTES.md` match the
  implemented subset;
- CLI, Python, and WASM host surfaces verify only serialized public outputs;
- `scripts/verify.sh` passes;
- a phase audit records supported behavior, unsupported variants, fixtures,
  commands, and residual risks.

Stop condition:

- stop instead of widening scope if exact Pine parity requires TradingView
  export data, realtime behavior, pyramiding, or a public schema redesign.
