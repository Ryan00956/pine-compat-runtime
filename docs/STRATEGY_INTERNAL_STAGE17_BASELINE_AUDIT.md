# Strategy Internal Stage 17a Baseline Audit

Status: closed on 2026-09-02. This slice does not change syntax acceptance,
runtime fills, conformance status, snapshots, matrix output, or public
strategy output.

Stage 17a locks the pre-unified-broker contract before Stage 17b introduces
explicit command origin and stable internal keys.

## Baseline Commands

Both required runs were captured before any behavior edit and agree:

```text
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-sema strategy
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 scripts/check_host_parity.py
```

| Gate | Run 1 | Run 2 |
| --- | --- | --- |
| `pine-runtime` strategy | 502 passed, 0 failed | 502 passed, 0 failed |
| `pine-sema` strategy | 96 passed, 0 failed | 96 passed, 0 failed |
| CLI runtime goldens | 1 passed, 0 failed | 1 passed, 0 failed |
| host parity | passed: 764 registered CLI runtime snapshots; 468 required runtime and 5 required legacy-analysis Python/WASM assertions | same |

Count commands:

```text
rg --files tests/fixtures/runtime | rg '/strategy_.*\.pine$' | wc -l
rg --files tests/snapshots | rg '/runtime_strategy_.*\.json$' | wc -l
rg -c '^strategy' tests/fixtures/conformance.tsv
```

| Count | 2026-09-02 planning baseline | Observed 2026-09-02 |
| --- | ---: | ---: |
| strategy runtime fixtures | 259 | 259 |
| strategy runtime snapshots | 244 | 244 |
| strategy-prefixed conformance rows | 84 | 84 |

No count drift. Every fixture path listed on a `strategy*` conformance row
exists on disk.

## Characterization Coverage

New broker tests in
`crates/pine-runtime/src/strategy/broker/fill_origin_characterization_tests.rs`
drive the live `BrokerState` fill APIs from a real start state:

| Fill-origin family | Test |
| --- | --- |
| same-side market entry | `characterization_same_side_market_entry_adds_open_trade_under_pyramiding` |
| market entry reversal | `characterization_market_entry_reversal_flattens_then_opens_opposite` |
| same-side market generic order | `characterization_same_side_market_generic_order_bypasses_pyramiding` |
| reduce-only market generic order | `characterization_reduce_only_market_generic_order_does_not_cross_zero` |
| price-based entry | `characterization_price_based_entry_fills_at_limit_on_later_bar` |
| price-based generic order | `characterization_price_based_generic_order_adds_same_side_without_pyramiding` |
| full close | `characterization_full_close_flattens_matching_entry` |
| partial close | `characterization_partial_close_keeps_remaining_quantity_and_average` |
| exit fill | `characterization_exit_fill_closes_from_pending_stop` |
| margin-call fill | `characterization_margin_call_fill_partially_liquidates_long` |

Existing Stage 14-16 broker tests remain; these named tests are the Stage 17
routing contract.

## Starting Contract

The current fixture-backed strategy runtime already includes:

- long and short market, limit, stop, and stop-limit `strategy.entry()` subsets;
- market `strategy.entry()` reversal;
- selected same-side `strategy.order()` additions and reduce-only market-short
  behavior;
- `strategy.close()`, `strategy.close_all()`, cancellation, broad
  `strategy.exit()` triggers, brackets, trailing exits, partial quantities, and
  internal reservation behavior;
- a side-aware multi-entry `TradeLedger`, long and short trade fields, supported
  commission/slippage/limit-verification settings, long and short margin,
  affordability checks, forced liquidation, and liquidation-price reporting;
- id-specific long and short `close_entries_rule="ANY"` allocation;
- public CLI, Python, and WASM parity for the current strategy result shape.

Internal constraints that Stage 17 must not widen yet:

- pending entries use `enforce_pyramiding` as the entry-versus-generic-order
  distinction instead of an explicit command origin;
- pending records have no stable internal creation sequence independent of the
  public string id;
- market, limit, stop, and stop-limit fills still dispatch through separate
  direction-specific runtime calls;
- `strategy.close()` / `strategy.close_all()` still fill on the current bar;
- generic-order cross-zero netting, custom OCA, recalculation, realtime
  strategy ticks, and `strategy.risk.*` remain unsupported.

Public `StrategyResult` fields are unchanged.

## Documentation Truth Lock

Updated present-tense status prose that still described the runtime as
long-only:

- root `README.md` strategy subset and honest-compatibility bullet;
- `docs/CONFORMANCE.md` remaining-exclusion paragraph, signed `position_size`,
  `opentrades` broker wording, and current bracket/trailing broker wording;
- `docs/EXECUTION_SEMANTICS.md` explicit-margin subset wording;
- `docs/SEMANTIC_MODEL.md` current exit/trailing broker wording.

Executable claims were not widened: `tests/fixtures/conformance.tsv` and
`tests/snapshots/matrix.json` are unchanged.

## Known Documentation Contradictions

These are recorded, not rewritten, because they are historical notes or
row-level copies of older fixture-backed reporting subsets rather than
missing-fixture contradictions:

- Several `strategy.*` reporting rows in `tests/fixtures/conformance.tsv` still
  say "long-only" even though Stage 14-16 added short fills. The fixtures
  listed on those rows remain the original long reporting scripts; short
  behavior is claimed on the `strategy.entry` / `strategy.order` /
  `strategy.close` / margin rows instead. This is documentation drift, not a
  fixture-versus-row support mismatch.
- Historical phase audits, Stage 1-16 closeout docs, and older
  `docs/BUILTIN_SIGNATURES.md` helper notes still describe the then-current
  long-only subset. Those records stay historical.
- `docs/STRATEGY_INTERNAL_GAP_AUDIT.md` remains a historical inventory; the
  active slice order is `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`.

No listed strategy-row fixture is missing. No existing runtime fixture
contradicts its conformance support status. Stage 17b may proceed.

## Files

- `crates/pine-runtime/src/strategy/broker/fill_origin_characterization_tests.rs`
- `crates/pine-runtime/src/strategy/broker/mod.rs`
- `README.md`
- `docs/CONFORMANCE.md`
- `docs/EXECUTION_SEMANTICS.md`
- `docs/SEMANTIC_MODEL.md`
- `docs/RELEASE_NOTES.md`
- `docs/STRATEGY_INTERNAL_STAGE17_BASELINE_AUDIT.md`
- `docs/STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md`

Unrelated already-dirty planning docs were left untouched:
`docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`, `docs/PURE_INTERNAL_ROADMAP.md`,
`docs/README.md`, and `docs/STRATEGY_INTERNAL_EXECUTION_PLAN.md`.

## Tests

```text
cargo test -p pine-runtime characterization_ -- --test-threads=1
cargo test -p pine-runtime strategy -- --test-threads=1
cargo test -p pine-sema strategy
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
python3 scripts/check_host_parity.py
git diff --check
scripts/verify.sh
```

Post-change focused results:

- characterization: 10 passed, 0 failed
- `pine-runtime` strategy: 512 passed, 0 failed (502 baseline + 10 new tests)
- `pine-sema` strategy: 96 passed, 0 failed
- CLI runtime goldens: 1 passed, 0 failed; pre-Stage-17
  `tests/snapshots/runtime_strategy_*.json` unchanged
- host parity: passed
- `git diff --check`: clean
- `scripts/verify.sh`: passed (exit 0). `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, structure/host-parity/WASM Node smoke, and
  544 Python binding tests all succeeded.

## Remaining Exclusions

Stage 17b-17g internal kernel work, Stage 18 timing, generic-order netting,
OCA, recalculation, and `strategy.risk.*` remain unstarted. Unsupported
parameters stay fail-closed.
