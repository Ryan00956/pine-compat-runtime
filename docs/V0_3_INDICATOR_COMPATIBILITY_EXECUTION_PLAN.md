# v0.3 Indicator Compatibility Execution Plan

Status: active after the `v0.2.0` release.

## Outcome

`v0.3.0` is an evidence-driven indicator compatibility release. Its primary
goal is to increase the number of representative legacy indicators that can be
copied into the runtime and executed directly, while preserving the explicit
versioned semantics and fail-closed behavior established in `v0.2.0`.

The release is not defined by a fixed list of new built-ins. Work is selected
from measured failures in an expanded authorized corpus. Pine v1-v4 strategies
remain excluded from this plan.

## Starting Point

`v0.2.0` released the following legacy indicator profiles:

| Profile | Maturity | Eligible seed scripts | Historical result |
| --- | --- | ---: | --- |
| Pine v4 indicators | preview | 12 | 12/12 pass |
| Pine v3 indicators | preview | 7 | 7/7 pass |
| Pine v2 indicators | experimental | 2 | 2/2 pass |
| implicit Pine v1 indicators | experimental | 1 | 1/1 pass |

The existing seed is deterministic and fully passing, but it is too small and
has no external reference-output oracle. The main post-`v0.2.0` risk is
therefore evidence breadth rather than a known failing release feature.

## Scope

In scope:

- direct execution of authorized Pine v1-v4 indicators;
- v4 as the first corpus and implementation priority;
- v3 as the second priority after the first v4 failure ranking exists;
- version-aware parser, binder, lowering, runtime, request, and output fixes
  selected from corpus failure clusters;
- source-free reference outputs where they can be supplied lawfully;
- modern v5/v6 negative and regression controls for every compatibility slice;
- CLI, Python, and WASM parity for every newly claimed runtime behavior.

Out of scope:

- every Pine v1-v4 strategy and every `strategy.*` call in legacy sources;
- feature work selected only because a broad matrix row is partial;
- unmeasured built-in additions with no representative source demand;
- an automatic source migration or rewriting command;
- host rendering, chart layout, remote data lookup, and broker behavior;
- lower-timeframe `security` or whole-program `study(resolution=...)` execution
  without a separate design gate and corpus evidence that justifies it.

## Release Policy

- Use `v0.2.1` only for a confirmed regression or packaging defect in the
  published `v0.2.0` contract.
- Accumulate corpus-ranked compatibility improvements for `v0.3.0`.
- Do not promise a maturity promotion in advance. A profile remains preview or
  experimental unless every applicable release gate passes.
- Do not change public analysis or runtime schemas merely to record corpus
  planning data. The corpus report has its own independent schema.

## Track 0: Consumer Release Smoke

Before changing interpreter behavior, install the published wheel in each real
consumer environment and exercise the public release contract. For the current
CandleScope consumer this work belongs in the host repository; semantic fixes
found by the smoke belong back in this interpreter repository.

The smoke must assert at least:

- analysis schema 5 and runtime schema 8;
- `renderMetadataVersion: 1`;
- explicit low-valued RGBA round trips;
- input defaults, constraints, options, and callsite overrides;
- one modern indicator, one v4 indicator, and one request-backed v4 indicator;
- rollback to the previous installed wheel if activation fails.

A failure here blocks new feature work and is a `v0.2.1` candidate.

## Track 1: Corpus R2 Intake

### Intake Milestones

Use two distinct milestones so useful failures can be selected before the final
stable-size corpus exists:

1. R2 selection baseline:
   - at least 30 total eligible v4 indicators;
   - at least 15 total eligible v3 indicators when authorized samples are
     available;
   - ordinary inputs, plots, colors/styles, history, stateful TA, and
     multi-timeframe examples;
   - at least five invalid or modern controls.
2. Stable-size evidence baseline:
   - at least 50 eligible scripts for each profile considered for stable;
   - difficult eligible scripts remain in the denominator;
   - available reference-output bundles remain attached to their manifest rows.

The first R2 intake therefore needs 18 additional v4 indicators to reach the
selection baseline. The final v4 stable-size corpus needs 38 additional
indicators relative to `v0.2.0`.

### Allowed Sources

- original project indicators;
- user-owned or explicitly user-authorized indicators;
- permissively licensed indicators with recorded provenance;
- minimized original reproductions that do not retain protected formulas;
- source-free reference-output bundles.

Do not scrape protected sources to fill a count. Private manifests may use
absolute paths, but reports must continue to expose only opaque ids, hashes,
structured diagnostics, and aggregate metrics.

### Manifest And Baseline Command

Use the existing manifest columns documented in
`tests/fixtures/legacy/README.md`. Keep rows sorted by opaque id and run:

```text
cargo build -p pine-cli
python3 scripts/analyze_legacy_corpus.py \
  --manifest /absolute/path/to/corpus-r2.tsv \
  --root /absolute/path/to/corpus-root \
  --build-revision corpus-r2-pre-code \
  --output /absolute/path/to/corpus-r2-pre-code.json
```

Run the same command twice and require byte-identical JSON before using the
report for prioritization.

## Track 2: Baseline And Failure Ranking

Corpus report schema 2 provides two planning views:

- `eligibleSuccessRate` uses every eligible script as the denominator, so an
  upstream failure or missing required input cannot disappear from a later
  stage rate;
- each failure cluster reports diagnostic occurrences, affected script count,
  total/profile share, and `requiresDisposition` when it reaches 2% of a source
  profile.

Each profile also receives a `stableBaseline` assessment for the provisional
50-script, 95% parse, 85% analyze/lower, and 80% historical-run thresholds.
Available reference outputs must all compare successfully, and any unknown
diagnostic cluster affecting at least 2% of a profile must be implemented or
explicitly treated as a release blocker.

`stableBaseline.thresholdsMet` is not a stable compatibility claim. Incremental,
realtime, request-provider, resource, cache, host-parity, and release audits are
still mandatory afterward.

## Track 3: Behavior Slices

Select slices from the R2 report in this order when their affected-script impact
is comparable:

1. exact legacy aliases whose canonical implementation already runs;
2. historical call signatures, parameter names, inputs, and output metadata;
3. versioned expression, type, qualifier, and declaration-graph behavior;
4. deterministic same-context and higher-timeframe request expressions;
5. separately designed whole-program timeframe or lower-timeframe behavior.

Each slice must contain:

- a minimized original or authorized failing fixture;
- a paired canonical fixture where semantic equivalence is claimed;
- an original-span translation, emulation, or focused unsupported diagnostic;
- modern v5/v6 collision and negative controls;
- historical runtime evidence and incremental/realtime evidence when claimed;
- CLI-owned goldens plus Python/WASM parity for public behavior;
- updated conformance notes without widening adjacent untested forms.

Do not combine unrelated failure clusters merely to increase a release count.

## Track 4: Release Audit

A profile can be considered for stable only when:

- its frozen authorized corpus contains at least 50 eligible scripts;
- parse succeeds for at least 95% of all eligible scripts;
- analyze/lower succeeds for at least 85% of all eligible scripts;
- historical execution succeeds for at least 80% of all eligible scripts;
- every executed fixture passes incremental parity;
- every claimed realtime/request profile passes its specific gate;
- every supplied reference output passes;
- no crash, panic, hang, scope mismatch, or silent unsupported execution occurs;
- every unknown failure cluster affecting at least 2% of eligible scripts is
  implemented or named as a blocker.

Passing these thresholds does not promote v2/v1 automatically and never admits
legacy strategies.

## Verification Per Slice

When only the corpus analyzer or its documentation changes:

```text
python3 -m unittest scripts/tests/test_analyze_legacy_corpus.py
git diff --check
scripts/verify.sh
```

When conformance rows change, refresh `tests/snapshots/matrix.json` first with:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
scripts/verify.sh
```

## Immediate Next Decision

The repository does not yet contain authorized R2 sources, so no new behavior
slice is selected by this plan. The next input is an intake batch that brings the
v4 corpus toward 30 scripts. Once its deterministic pre-code report exists, the
largest copy-paste-blocking failure cluster becomes the first `v0.3.0` behavior
slice.
