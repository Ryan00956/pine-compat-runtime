# Strategy Internal Stage 19c Limit Generic-Order Netting Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. Limit
`strategy.order` long and short fills reuse Stage 19b signed netting after
limit trigger selection and fill-price verification. Public JSON shape is
unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- Eligible limit generic orders (`!enforce_pyramiding`) fill through
  `apply_generic_market_order_netting` after `take_all_eligible_limit_*`.
- Fill price is the order limit, then the existing long/short entry slippage
  offset. Limit verification still gates eligibility.
- Creation-bar and non-triggering prices do not net. Cancel before fill
  removes the intent without cash, trade, or position mutation.
- Short limit orders may now be placed while net long so opposite-side
  netting can fill later. Stop and stop-limit short placement guards stay.
- Price-based `strategy.entry()` reversal stays unrouted for 19e.

## Named Runtime Goldens

- `runtime_strategy_order_limit_long_against_short.json`
- `runtime_strategy_order_limit_short_against_long.json`
- `runtime_strategy_order_limit_long_flatten_short.json`
- `runtime_strategy_order_limit_short_flatten_long.json`
- `runtime_strategy_order_limit_long_reduce_short.json`
- `runtime_strategy_order_limit_short_reduce_long.json`
- `matrix.json` (conformance notes and fixtures)

## Files

- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/strategy/broker/netting_matrix_tests.rs`
- `crates/pine-runtime/src/strategy/broker/tests.rs`
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

Owner-local: `cargo test -p pine-runtime strategy -- --test-threads=1` (581
passed). Table-driven broker tests cover both-side reduce, flatten, and
cross-zero at the verified limit price. Stop opposite-side orders stay
unnetted.

## Remaining Exclusions

19d routes stop and stop-limit generic-order netting. 19e routes price-based
`strategy.entry()` reversal. Omitted `qty` for `strategy.short` stays
unsupported.
