# Strategy Internal Stage 22c `strategy.risk.max_position_size()` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`.
`strategy.risk.max_position_size` accepts simple positive finite numeric
`contracts`. Later `strategy.entry` quantity is reduced so projected post-fill
exposure does not exceed the limit. Remaining room of zero is a no-op.
Reversal flattens then opens at most the limit on the new side. Pyramiding
may add until the size limit. Pending `strategy.entry` quantities are reduced
when the rule is recorded or when they fill. `strategy.order` is not bound by
this rule. Other `strategy.risk.*` calls stay rejected. Public
`StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- Simple positive finite numeric `contracts` are accepted, including named
  const alias chains.
- Zero, negative, non-finite, and series values are rejected
  (`E_CALL_ARG_VALUE` / simple-numeric type error).
- Indicator scripts still get `E_STRATEGY_MODE`.
- Same-side `strategy.entry` qty is `min(requested, max - |position|)`.
- If remaining room is zero, the entry is not placed and does not fill.
- Opposite `strategy.entry` reversal flattens first, then opens at most
  `contracts` on the new side.
- Price-based pending `strategy.entry` quantities are reduced at placement
  and again at fill against current remaining room.
- Recording the rule reduces or cancels already-pending `strategy.entry`
  intents. Generic `strategy.order` pending qty is unchanged.
- Margin affordability uses the clamped quantity. OCA peer reduction, if any,
  observes the filled (clamped) quantity. `strategy.entry` still has no
  entry-family OCA parameters in this subset.
- Remaining `strategy.risk.*` calls stay semantic rejections.

## Named Runtime Goldens

- `runtime_strategy_risk_max_position_size_reduces.json`
- `runtime_strategy_risk_max_position_size_full_noop.json`
- `runtime_strategy_risk_max_position_size_reversal.json`
- `runtime_strategy_risk_max_position_size_order_unaffected.json`
- `runtime_strategy_risk_max_position_size_pyramiding.json`
- `runtime_strategy_risk_max_position_size_limit.json`
- `matrix.json` (conformance notes and fixtures)

## Incremental / Realtime

Not applicable as a dedicated forming-bar fixture. The rule is broker
configuration stored with `StrategyRiskRules`, so Stage 21c snapshot/restore
already rolls it back with the rest of broker state.

## Files

- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-builtins/src/registry.rs`
- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-sema/src/analyzer/unsupported.rs`
- `crates/pine-sema/src/analyzer/calls/helpers.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/risk.rs`
- `crates/pine-runtime/src/strategy/broker/risk_storage_tests.rs`
- `crates/pine-runtime/src/strategy/broker/entries.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-runtime/src/tests/builtin_registry.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_risk_max_position_size.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_position_size_zero.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_position_size_negative.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_position_size_series.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_max_position_size_indicator.pine`
- `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_reduces.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_full_noop.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_reversal.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_order_unaffected.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_pyramiding.pine`
- `tests/fixtures/runtime/strategy_risk_max_position_size_limit.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 679/111 passed, saved as `{SCRATCH}/stage22c-baseline-1.log`,
`{SCRATCH}/stage22c-baseline-2.log`, `{SCRATCH}/stage22c-sema-baseline-1.log`,
and `{SCRATCH}/stage22c-sema-baseline-2.log`.

Fail-closed: zero, negative, series, indicator-mode, remaining `strategy.risk.*`
rejections, already-at-limit no-op, and generic-order exemption before
reduction/reversal.

Owner-local: `cargo test -p pine-runtime --lib risk` 25 passed.
`cargo test -p pine-sema strategy` 116 passed.
`cargo test -p pine-runtime strategy` 687 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 610 passed. Host parity 534 required
runtime goldens. WASM 639 passed. Log: `{SCRATCH}/stage22c-verify.sh.log`.

## Remaining Exclusions

22d implements `strategy.risk.max_drawdown()`. Other `strategy.risk.*` calls
stay rejected. Public risk-state schema stays private.
