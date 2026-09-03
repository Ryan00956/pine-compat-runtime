# Strategy Internal Stage 22b `strategy.risk.allow_entry_in()` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`.
`strategy.risk.allow_entry_in` accepts documented `strategy.direction.*`
constants. Allowed `strategy.entry` directions keep current behavior. A
disallowed opposite `strategy.entry` against an open allowed position
flattens without opening prohibited exposure. A disallowed opposite entry
while flat is a no-op. Last call wins. Pending opposite `strategy.entry`
intents are cancelled while flat or converted to market close-only against
an open allowed position. `strategy.order` is not bound by this rule. Other
`strategy.risk.*` calls stay rejected. Public `StrategyResult` is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/
https://www.tradingview.com/pine-script-reference/v6/

## Behavior

- Const/simple `strategy.direction.all`, `strategy.direction.long`, and
  `strategy.direction.short` are accepted, including named const alias chains.
- Series values, `strategy.long`/`strategy.short`, and other strings are
  rejected (`E_CALL_ARG_TYPE` / `E_CALL_ARG_VALUE`).
- Indicator scripts still get `E_STRATEGY_MODE`.
- Allowed `strategy.entry` directions keep current open, add, and reversal
  fills.
- A disallowed opposite `strategy.entry` against an open allowed position is
  converted to a market close of that position. Requested limit/stop prices
  are not kept.
- A disallowed opposite `strategy.entry` while flat does not place or fill.
- Repeated calls use last-call-wins.
- When the rule is recorded, pending `strategy.entry` intents in the
  disallowed direction are cancelled if they would open prohibited exposure,
  or rewritten to market close-only if they would flatten an open allowed
  position.
- Generic `strategy.order` placement and fills are unchanged by this rule.
- Remaining `strategy.risk.*` calls stay semantic rejections.

## Named Runtime Goldens

- `runtime_strategy_risk_allow_entry_in_long.json`
- `runtime_strategy_risk_allow_entry_in_short.json`
- `runtime_strategy_risk_allow_entry_in_long_flat_noop.json`
- `runtime_strategy_risk_allow_entry_in_order_unaffected.json`
- `runtime_strategy_risk_allow_entry_in_repeated.json`
- `matrix.json` (conformance notes and fixtures)

## Incremental / Realtime

Not applicable as a dedicated forming-bar fixture. The rule is broker
configuration stored with `StrategyRiskRules`, so Stage 21c snapshot/restore
already rolls it back with the rest of broker state. Historical extra-pass
and forming execution do not add a second admission path.

## Files

- `crates/pine-builtins/src/namespaces/strategy.rs`
- `crates/pine-builtins/src/constants/strings.rs`
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
- `tests/fixtures/sema/supported_strategy_risk_allow_entry_in.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_unknown.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_series.pine`
- `tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_indicator.pine`
- `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`
- `tests/fixtures/runtime/strategy_risk_allow_entry_in_long.pine`
- `tests/fixtures/runtime/strategy_risk_allow_entry_in_short.pine`
- `tests/fixtures/runtime/strategy_risk_allow_entry_in_long_flat_noop.pine`
- `tests/fixtures/runtime/strategy_risk_allow_entry_in_order_unaffected.pine`
- `tests/fixtures/runtime/strategy_risk_allow_entry_in_repeated.pine`
- `tests/fixtures/runtime/strategy_constants.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 669/107 passed, saved as `{SCRATCH}/stage22b-baseline-1.log`,
`{SCRATCH}/stage22b-baseline-2.log`, `{SCRATCH}/stage22b-sema-baseline-1.log`,
and `{SCRATCH}/stage22b-sema-baseline-2.log`.

Fail-closed: unknown direction, series value, indicator-mode, remaining
`strategy.risk.*` rejections, and flat disallowed-entry no-op before
close-only conversion.

Owner-local: `cargo test -p pine-runtime --lib risk` 17 passed.
`cargo test -p pine-sema strategy` 111 passed.
`cargo test -p pine-runtime strategy` 679 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 604 passed. Host parity 528 required
runtime goldens. WASM 633 passed. Log: `{SCRATCH}/stage22b-verify.sh.log`.

## Remaining Exclusions

22c implements `strategy.risk.max_position_size()`. Other `strategy.risk.*`
calls stay rejected. Public risk-state schema stays private.
