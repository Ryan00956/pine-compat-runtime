# Strategy Internal Stage 20c `strategy.oca.cancel` Audit

Status: closed on 2026-09-03 after `scripts/verify.sh`. After a generic
`strategy.order` fill, still-pending peers in the same `strategy.oca.cancel`
group are cancelled in internal creation order. Unrelated groups stay.
`strategy.oca.reduce` remains rejected. Public JSON shape is unchanged.

Official review date: 2026-09-03.
https://www.tradingview.com/pine-script-docs/concepts/strategies/

## Behavior

- `strategy.order` accepts const `oca_type=strategy.oca.cancel` with
  const/simple `oca_name`.
- After a successful generic fill, same-group pending generic orders are
  removed in creation-key order. Same-tick remaining candidates that were
  already taken are skipped.
- Unrelated `oca_name` groups and `strategy.oca.none` peers are not cancelled.
- Margin-rejected fills do not cancel peers.
- `strategy.oca.reduce` and `strategy.exit` `oca_name` stay rejected.
- Entry-family OCA cancel stays later (slice item 5).

## Named Runtime Goldens

- `runtime_strategy_order_oca_cancel.json` (A fills, B cancelled, unrelated C
  fills; size `2`)
- `matrix.json`

## Files

- `crates/pine-sema/src/analyzer/strategy.rs`
- `crates/pine-runtime/src/builtins/strategy.rs`
- `crates/pine-runtime/src/strategy/broker/fill_apply.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entry_fills.rs`
- `crates/pine-runtime/src/strategy/broker/order_book.rs`
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs`
- `crates/pine-runtime/src/strategy/broker/oca.rs`
- `crates/pine-runtime/src/strategy/broker/oca_storage_tests.rs`
- `crates/pine-runtime/src/tests/strategy.rs`
- `crates/pine-sema/tests/fixtures.rs`
- `crates/pine-cli/src/conformance/guards/strategy.rs`
- `crates/pine-cli/src/runtime_snapshots/fixtures/strategy_orders.rs`
- `crates/pine-wasm/src/tests/mod.rs`
- `python/tests/test_bindings.py`
- `scripts/host_parity_required.txt`
- `tests/fixtures/conformance.tsv`
- `tests/fixtures/sema/supported_strategy_order_oca_cancel.pine`
- `tests/fixtures/sema/unsupported_strategy_orders.pine`
- `tests/fixtures/runtime/strategy_order_oca_cancel.pine`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/LANGUAGE_SCOPE.md`
- `docs/BUILTIN_SIGNATURES.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

## Commands

Baseline: `cargo test -p pine-runtime strategy` and `cargo test -p pine-sema
strategy` twice, 609/126 passed.

Close-out:
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots`
`UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot`
`git diff --check` (clean)
`scripts/verify.sh` EXIT:0. Python 588 passed. Host parity 512 required
runtime goldens. Log: `{SCRATCH}/stage20c-verify.sh.log`.

## Remaining Exclusions

20d implements `strategy.oca.reduce`. 20e maps `strategy.exit` `oca_name`.
Entry-family OCA cancel is still later.
