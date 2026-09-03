# Strategy Internal Stage 19b Market Generic-Order Netting Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Market
`strategy.order(..., strategy.long)` and `strategy.order(..., strategy.short)`
fills apply signed netting `target = P + D` in flat, long, and short states.
Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

`strategy.order` is a signed position delta, not `strategy.entry` reversal.
Entry reversal flattens the opposite side and then opens the requested quantity.
Generic-order netting adds the signed fill to the current position.

## Behavior

- Pending market generic orders (`!enforce_pyramiding`) route through
  `apply_generic_market_order_netting`.
- Close quantity is `min(|P|, |D|)` when `D` opposes `P`; remainder opens on
  the order side using the generic-order id.
- Public order quantity is `|D|`. One public order fill is recorded. Alerts
  set `entry_id` when an open remainder exists and `exit_id` when a close leg
  exists.
- Close allocation uses the current FIFO / id-specific ANY close-entries rule.
- Long orders use long-entry slippage; short orders use short-entry slippage.
- Commission is calculated on `|D|` and scaled across close and open legs.
- An unaffordable open remainder rejects the whole fill (`E_STRATEGY_MARGIN`)
  with no cash, ledger, order, or trade mutation.
- Flatten and reverse clear pending exits. Pyramiding does not cap generic
  orders.
- Limit, stop, and stop-limit opposite-side generic-order netting and
  price-based `strategy.entry()` reversal stay unrouted for later Stage 19
  slices.

## Named Runtime Goldens

- `runtime_strategy_order_short_flat_noop.json`
- `runtime_strategy_order_market_short_increase.json`
- `runtime_strategy_order_long_flatten_short.json`
- `runtime_strategy_order_long_reduce_short.json`
- `runtime_strategy_order_short_flatten_long.json`
- `runtime_strategy_order_long_against_short.json`
- `runtime_strategy_order_short_oversized_against_long.json`
- `matrix.json` (conformance notes and fixtures)

Existing Stage 18c close-timing goldens are unchanged by this slice.

## Files

- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/netting_matrix_tests.rs`
- `crates/pine-runtime/src/strategy/broker/tests.rs`
- `crates/pine-runtime/src/strategy/broker/fill_origin_characterization_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`

## Tests

Owner-local: `cargo test -p pine-runtime strategy -- --test-threads=1` (575
passed). Table-driven broker tests cover both directions and the five netting
shapes. Additional tests cover atomic margin rejection, side slippage and
commission, max-held fields, pyramiding independence, pending-exit cleanup, and
cross-zero alert identity.

CLI/Python/WASM host parity covers the named goldens.

## Remaining Exclusions

19c routes limit generic-order netting. Stop/stop-limit netting is 19d.
Price-based `strategy.entry()` reversal is 19e. Omitted `qty` for
`strategy.short` stays unsupported.
