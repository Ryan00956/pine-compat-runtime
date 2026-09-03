# Strategy Internal Stage 19a Netting Matrix Audit

Status: closed on 2026-09-02 after `scripts/verify.sh`. Table-driven netting
splits cover both directions and the five generic-order shapes. Cross-zero
transitions are calculated and remain unrouted. Runtime fixtures lock the
pre-19b fail-closed/no-cross-zero boundary. Semantic acceptance and
conformance are unchanged.

Official review date: 2026-09-02.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Identity Decisions For Later Routing

Generic `strategy.order` signed fill `D` against position `P`:

- public order quantity is `|D|` (the filled delta, not only the close leg);
- close quantity is `min(|P|, |D|)` when `D` opposes `P`;
- open quantity is the remainder after flatten;
- closed trades allocate the close leg from the current ledger policy;
- a new open trade, when `open_quantity > 0`, uses the generic-order id;
- pyramiding does not cap generic-order netting.

`strategy.entry` reversal is not `P + D`: flatten the opposite side, then open
the requested entry quantity on the new side. Public order identity stays the
entry id. Price-based entry reversal stays unrouted until 19e.

## Current Production Boundary

- Oversized market `strategy.order` short against long is reduce-only flatten.
- Market/limit `strategy.order` long against short does not close the short
  (`closedtrades` stays 0).
- Limit `strategy.entry` against a short does not reverse in this slice.

## Fixtures

- `tests/fixtures/runtime/strategy_order_long_against_short.pine`
- `tests/fixtures/runtime/strategy_order_short_oversized_against_long.pine`
- `tests/fixtures/runtime/strategy_order_limit_long_against_short.pine`
- `tests/fixtures/runtime/strategy_entry_limit_reverses_short.pine`

No public goldens or conformance rows were added.

## Remaining Exclusions

19b routes market generic-order netting. Limit/stop/stop-limit netting and
price-based entry reversal stay later in Stage 19.
