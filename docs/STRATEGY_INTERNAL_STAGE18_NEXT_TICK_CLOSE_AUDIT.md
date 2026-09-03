# Strategy Internal Stage 18c Next-Tick Close Audit

Status: closed on 2026-09-02 after `scripts/verify.sh`. Default
`strategy.close()` / `strategy.close_all()` place pending market closes and
fill at the next historical bar open. Public JSON shape is unchanged (no
schemaVersion bump): only fill bar/time/price and script-visible same-bar
position change.

Official review date: 2026-09-02.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-docs/language/execution-model/

## Replaced Behavior

Previously, supported close/close-all filled during script evaluation at
`bar.close` on the signal bar. After 18c they fill at the next bar `open`.
Signal-bar `strategy.position_size` remains pre-fill. A close against a
missing position at fill time is a no-op (no double fill). Same-bar
`close_all` then `entry` stays a no-op for the pending close: market closes
fill before pending entries in the scheduler.

Quantity policy is stored at placement and resolved at fill (18b). Invalid
qty/qty_percent is still diagnosed immediately through the old close-qty
paths. `immediately` remains rejected (`E_CALL_ARG_NAME`).

Last-bar close needs a later bar to fill. Default `tests/fixtures/runtime/bars.csv`
stays 4 bars so indicator goldens are unchanged. Close-on-last-bar fixtures
use `tests/fixtures/runtime/strategy_next_tick_close_bars.csv` (5 bars).
`strategy_close_exit.pine` already closes before the last default bar, so it
keeps `bars.csv`.

## Named Runtime Goldens

CLI/Python/WASM goldens refreshed for next-tick close:

- `runtime_strategy_close.json`
- `runtime_strategy_close_all.json`
- `runtime_strategy_close_all_exit.json`
- `runtime_strategy_close_all_short.json`
- `runtime_strategy_close_entries_rule_any_close_short.json`
- `runtime_strategy_close_exit.json`
- `runtime_strategy_close_metadata.json`
- `runtime_strategy_close_noop.json`
- `runtime_strategy_close_qty_full_clamp.json`
- `runtime_strategy_close_qty_partial.json`
- `runtime_strategy_close_qty_percent_precedence.json`
- `runtime_strategy_close_short.json`
- `runtime_strategy_closedtrades_fields.json`
- `runtime_strategy_closedtrades_fields_pyramiding.json`
- `runtime_strategy_commission_cash_per_contract.json`
- `runtime_strategy_commission_cash_per_order.json`
- `runtime_strategy_commission_percent.json`
- `runtime_strategy_entry_metadata.json`
- `runtime_strategy_equity.json`
- `runtime_strategy_exit_qty_percent_state.json`
- `runtime_strategy_exit_qty_state.json`
- `runtime_strategy_exit_trailing_close_cancel.json`
- `runtime_strategy_margin_capital_held_long.json`
- `runtime_strategy_margin_capital_held_short.json`
- `runtime_strategy_opentrades_fields.json`
- `runtime_strategy_position_state.json`
- `runtime_strategy_profit_percent_state.json`
- `runtime_strategy_profit_state.json`
- `runtime_strategy_pyramiding_close.json`
- `runtime_strategy_pyramiding_close_all.json`
- `runtime_strategy_slippage.json`
- `runtime_strategy_trade_counts.json`
- `runtime_strategy_trade_outcome_counts.json`
- `runtime_strategy_variable_interactions.json`
- `matrix.json` (conformance notes only: `strategy.close`, `strategy.close_all`,
  `strategy.closedtrades`, `strategy.opentrades`, `strategy.position_size`,
  `strategy.position_avg_price`)

Runtime fixtures covered by pine-runtime assertions without a CLI golden
(`strategy_close_entries_rule_any_close.pine`,
`strategy_close_entries_rule_fifo.pine`,
`strategy_close_entries_rule_fifo_close_all.pine`) also fill next-tick.

Indicator and non-strategy goldens are unchanged.

## Files

- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/runtime/strategy_scheduler.rs`
- `crates/pine-runtime/src/runtime/historical.rs`
- `crates/pine-runtime/src/strategy/broker/pending_closes.rs`
- `crates/pine-cli/src/runtime_snapshots/bars.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `tests/fixtures/runtime/strategy_next_tick_close_bars.csv`
- `tests/fixtures/conformance.tsv`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`

## Commands

```text
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 -m pytest python/tests/test_bindings.py -q
git diff --check
scripts/verify.sh
```

Owner-local: `cargo test -p pine-runtime strategy` 547 passed; Python
bindings 544 passed after hardcoded plot updates. Full verify captured in
`{SCRATCH}/stage18c-verify.sh.log`.

## Migration

Callers comparing historical `orders`/`trades`/`equity` bar indexes or
same-bar `strategy.position_size` after `strategy.close` must treat the fill
as the next bar open. No public schema version bump: field names and types
are unchanged.

## Remaining Exclusions

`immediately` is Stage 18d. `process_orders_on_close` is 18e. Historical
OHLC path-candidate ordering is 18f.
