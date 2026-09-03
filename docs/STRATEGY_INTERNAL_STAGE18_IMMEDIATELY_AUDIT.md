# Strategy Internal Stage 18d Immediately Audit

Status: closed on 2026-09-02 after `scripts/verify.sh`. Const/simple bool
`immediately=true` on supported `strategy.close()` / `strategy.close_all()`
fills at the current bar close through the scheduler current-tick market
phase. Omitted or false keeps Stage 18c next-bar-open fills. Public JSON
shape is unchanged.

Official review date: 2026-09-02.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-docs/faq/strategies/

`immediately=true` is a per-command alternative to declaration-wide
`process_orders_on_close`: the broker emulator closes on the same tick that
creates the close order (historical bar close), not the next bar open. Series
and non-bool values stay rejected.

## Behavior

- Place a pending market close, then if `immediately` is true fill it at
  `bar.close` via `StrategyBarPhase::CurrentTickMarketFills`.
- Later statements on the signal bar observe the filled close.
- Quantity policy is still stored at placement and resolved at fill.
- Pending exits are cancelled when the immediate close flattens the entry.
- Repeated immediate closes against a missing position are no-ops.

## Named Runtime Goldens

- `runtime_strategy_close_immediately.json`
- `runtime_strategy_close_all_immediately.json`
- `runtime_strategy_close_immediately_false.json`
- `runtime_strategy_close_immediately_qty.json`
- `runtime_strategy_close_immediately_short.json`
- `matrix.json` (conformance notes only)

## Files

- `crates/pine-runtime/src/strategy/broker/pending_closes.rs`
- `crates/pine-runtime/src/strategy/broker/close_orders.rs`
- `crates/pine-runtime/src/strategy/broker/pending_close_tests.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/builtins/strategy/metadata.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
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

## Remaining Exclusions

`process_orders_on_close` is Stage 18e. Historical OHLC path-candidate
ordering is 18f.
