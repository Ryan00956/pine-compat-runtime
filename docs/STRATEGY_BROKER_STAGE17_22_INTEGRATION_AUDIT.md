# Strategy Broker Stage 17-22 Integration Audit

Status: integrated on branch `codex/strategy-broker-stages-17-22` on
2026-09-03. The executable implementation and evidence are committed as
`a4001e666`. Documentation and audit records are committed separately.

This audit records the recovery and integration of the previously uncommitted
Stage 17-22 strategy worktree. It does not claim Stage 18g support or external
TradingView output parity.

## Starting Worktree

Before integration, `main` matched `origin/main` at `5b05ad3a3` and contained:

- 102 modified tracked files;
- 256 untracked files;
- 0 staged files;
- 358 changed paths in total;
- no deleted or renamed paths;
- no untracked logs, temporary files, Python caches, or `target/` artifacts.

The worktree was moved intact onto
`codex/strategy-broker-stages-17-22` before staging or committing anything.

## Integrated Executable Scope

Commit `a4001e666` contains 308 executable/evidence paths under `crates/`,
`python/`, `scripts/`, and `tests/`. It includes:

- the Stage 17 shared fill-transition and ledger-invariant foundation;
- Stage 18 scheduler phases, pending closes, next-tick close behavior,
  `immediately`, `process_orders_on_close`, and family-ordered fill steps;
- Stage 19 generic-order signed netting and price-based entry reversal;
- Stage 20 OCA none/cancel/reduce subsets and unified cancellation;
- Stage 21 recalculation guardrails, `calc_on_order_fills`, realtime rollback,
  `calc_on_every_tick`, and the private Bar Magnifier host-input model;
- Stage 22 supported entry-direction, position-size, drawdown, intraday, and
  consecutive-loss-day risk rules;
- semantic boundary fixtures, runtime fixtures, snapshots, conformance rows,
  CLI guards, Python parity, and WASM parity.

The resulting strategy evidence baseline is:

- 333 `strategy_*.pine` runtime fixtures;
- 319 `runtime_strategy_*.json` snapshots;
- 90 strategy-prefixed conformance rows;
- 543 required runtime goldens in the host-parity guard.

## Closeout Corrections

### Stage 18f scope

Closeout review found that Stage 18f implemented stable order-family steps but
did not implement the original plan's direction-selected OHLC walk,
cross-family entry/order/exit/margin candidates, same-price stable-key ties, or
path-correct stop-limit sequencing.

The integration therefore:

- changes Stage 18 and 18f from `closed` to `partial`;
- preserves the passing family-ordered scheduler subset;
- adds Stage 18g as the next executable slice with explicit steps, acceptance
  criteria, and stop conditions;
- does not change current runtime behavior or regenerate runtime snapshots for
  an unimplemented path model.

### Conformance risk summary

The broad `strategy` conformance row still described all `strategy.risk.*`
directives as rejected after the six specific risk rows had become supported.
The broad unsupported row also listed only three of the six supported rules.
Both summaries now name all six fixture-backed rules and retain rejection for
undocumented names. The generated matrix snapshot was refreshed and rerun
without update mode.

### Roadmap state

`docs/NEXT_INTERNAL_CAPABILITY_PLAN.md` and
`docs/PURE_INTERNAL_ROADMAP.md` no longer recommend Stage 17 or list completed
Stage 19-22 work as pending. Both now route the next behavior slice to Stage
18g, followed by Bar Magnifier fill wiring, mixed-family OCA, and
instrument-session semantics.

## Verification

The final pre-commit worktree ran:

```text
git diff --check
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
cargo test -p pine-cli matrix_output_matches_golden_snapshot
scripts/verify.sh
```

Results:

- `git diff --check`: passed;
- matrix snapshot update and non-update rerun: 1 passed each;
- `scripts/verify.sh`: exit 0;
- workspace formatting, Clippy with warnings denied, Rust tests, structural
  guardrails, host-parity guard, wasm32 build, and Node smoke: passed;
- Python: 619 passed;
- WASM: 648 passed;
- host parity: 543 required runtime goldens and 5 required legacy-analysis
  assertions passed.

## Remaining Boundaries

- Stage 18g true OHLC-path and cross-family candidate ordering is not started.
- `use_bar_magnifier=true` remains rejected; fill wiring and public host APIs
  wait on Stage 18g.
- Mixed entry/order/exit OCA groups and series `oca_name` remain unsupported.
- Omitted quantity for unsupported short generic-order forms remains deferred.
- Risk windows do not use an instrument session calendar.
- Public pending-order, reservation, OCA, and risk-state schema remains private.
- This closeout proves repository-internal consistency, not independent
  comparison against TradingView outputs.
