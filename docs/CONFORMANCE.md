# Conformance

This document defines how compatibility is measured.

The project should make compatibility claims by tested feature, not by broad
language name.

## Fixture Categories

Fixtures should be grouped by behavior:

```text
tests/fixtures/
  syntax/
  sema/
  runtime/
  builtins/
  unsupported/
  regressions/
```

Each fixture should include:

- source script
- expected diagnostics or expected output
- selected Pine version
- fixture ownership or license metadata when not original

## Snapshot Outputs

Runtime snapshots should be normalized JSON:

```json
{
  "schemaVersion": 3,
  "plots": [],
  "plotChars": [],
  "plotShapes": [],
  "plotArrows": [],
  "plotBars": [],
  "plotCandles": [],
  "bgColors": [],
  "barColors": [],
  "hlines": [],
  "fills": [],
  "labels": [],
  "lines": [],
  "boxes": [],
  "tables": [],
  "alerts": [],
  "diagnostics": []
}
```

The snapshot format should avoid host-specific charting details.

Every machine-readable public output must include top-level `schemaVersion`.
Runtime outputs use `PUBLIC_RUNTIME_SCHEMA_VERSION`; analysis outputs use
`PUBLIC_ANALYSIS_SCHEMA_VERSION`; matrix JSON uses
`PUBLIC_MATRIX_SCHEMA_VERSION`. Runtime output is currently `schemaVersion: 3`
because the top-level `alerts` array is reserved; analysis and matrix JSON
remain `schemaVersion: 2`. The contracts are separate so runtime-only fields do
not force analysis or matrix schema changes. The text-only CLI `analyze` output
is diagnostic console output and is not part of the machine-readable schema
until a JSON mode is added.

## Source Graph Host Contract

Phase J Slice 1 adds a host-neutral source graph scaffold without changing
conformance claims. Import statements remain unsupported, so
`tests/fixtures/conformance.tsv` must not gain supported import/library rows
from this slice alone.

Hosts may pass library source text into semantic analysis as future graph input:

- CLI accepts repeated `--library-source KEY=path.pine` options for `analyze`
  and `run`. The CLI owns filesystem reads and passes source text to core.
- Python accepts `library_sources={"KEY": "source text"}` on `compile_script`,
  `analyze_script`, and `run_script`.
- WASM remains a documented temporary gap for library source injection; it uses
  the shared root-source `AnalysisInput` path but exposes no library-source JSON
  argument yet.

Core crates must not perform filesystem, network, clock, or host registry I/O
for library resolution. Library source keys are deterministic host-provided
identifiers: empty keys, keys containing whitespace/control characters, and
duplicate keys are rejected before analysis. Cache keys include root source
name/text and every host-provided library key/name/text so future import graph
use cannot reuse stale analysis.

CLI and WASM runtime JSON must be generated through the shared runtime contract
helper so field names and nesting cannot drift. Python returns native
dictionaries, so its binding tests assert the same top-level runtime keys and
representative nested output families such as `plotShapes` and `plotCandles`.
The Phase E drawing-object scaffold adds `labels`, `lines`, `boxes`, and
`tables` as top-level runtime keys in `schemaVersion: 2`. The executable label
subset covers `label.new`, selected `label.set_*` mutators, and `label.delete`
with sparse snapshots and a 500-label runtime limit. The executable line subset
covers `line.new`, selected endpoint/color/width/style/extend mutators, and
`line.delete` with sparse snapshots and a 500-line runtime limit. The executable
box subset covers `box.new`, selected geometry/background/border mutators, and
`box.delete` with sparse snapshots and a 500-box runtime limit. The executable
table subset covers `table.new` plus `table.cell` text/background/text-color
cell writes with deterministic table dimensions, a 50-table runtime limit, and a
1000-cell per-table limit. Deleting `na`, mutating `na`, or mutating an already
deleted drawing object is a no-op where deletion exists; invalid non-`na` ids
are runtime errors; ids are stable and not reused. Supported drawing creation,
mutation, and cell writes are covered under realtime rollback, and drawing side
effects inside user-defined functions are rejected under the existing
side-effect policy. Keep unsupported coordinate modes and advanced object
methods out of the supported matrix until they have fixtures and public-output
coverage. `polyline.*` remains explicitly unsupported because it needs a
fixture-backed point-object and point-array design; see
`docs/PHASE_E_POLYLINE_GATE.md`.

Phase H reserves `alerts` as a top-level runtime key in `schemaVersion: 3`.
The first supported alert subsets are `alertcondition(condition, title,
message)` with bool-compatible conditions and const-string title/message, plus
`alert(message)` with const-string messages. Reached true alert conditions and
reached alert calls emit `{id, barIndex, time, message, source}` events in
program order; false and `na` alert conditions emit nothing. Forming realtime
events are visible in the forming result and roll back until a confirmed update
commits an event. Repeated forming updates recompute alert events from the
confirmed snapshot, so abandoned forming events are neither retained nor
duplicated, and a confirmed update matches the equivalent historical execution
where the same final bar data is available. Alert frequency modes remain
unsupported. TradingView-style `{{...}}` alert placeholder interpolation is
also unsupported; supported alert messages are serialized literally.

Checked-in golden JSON snapshots live in `tests/snapshots/`. Snapshot tests are
strict string comparisons against deterministic compact JSON; a public field
rename, omitted `schemaVersion`, or matrix shape change should fail tests. To
refresh snapshots after an intentional public-output change, run:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli runtime_outputs_match_golden_snapshots
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli matrix_output_matches_golden_snapshot
UPDATE_SNAPSHOTS=1 cargo test -p pine-wasm analysis_outputs_match_golden_snapshots
cargo test --workspace
```

Review the resulting JSON diff before committing. Do not update snapshots to
hide accidental public contract changes.

Snapshot maintenance rules:

- Treat checked-in snapshots as public contract evidence, not generated noise.
- Update snapshots only with the targeted `UPDATE_SNAPSHOTS=1` commands above.
- Include the source change, snapshot diff, and documentation update in the
  same commit when a public output change is intentional.
- Run `scripts/verify.sh` after any snapshot refresh so CLI, WASM, Python, and
  matrix contracts are checked together.

## Numeric Tolerance

Floating point outputs should be compared with an explicit tolerance:

```text
absolute tolerance: 1e-10
relative tolerance: 1e-9
```

Some built-ins may need per-function tolerances if their documented formulas
accumulate rounding differently. Any wider tolerance must be justified in the
fixture metadata.

## Test Data

OHLCV fixtures should be small and deterministic.

Rules:

- Include enough bars to test warmup behavior.
- Include gaps or flat sections where indicators often fail.
- Include first-bar and out-of-range history cases.
- Do not depend on external market data downloads in unit tests.

## Unsupported Features

Unsupported fixtures are first-class tests.

Examples:

- unsupported `request.security` variants outside the same-context identity and
  same-or-higher-timeframe scalar-expression provider subset
- `strategy.entry`
- unsupported collection families or unsupported array variants
- unsupported label and line methods
- imports
- unsupported `varip` forms such as drawing ids, tuples, and value families
  outside the scalar and scalar typed-array subset
- non-integer or negative history offsets
- unsupported function side effects, including drawing and alert side effects

Expected result:

- no panic
- stable diagnostic code
- source span
- machine-readable compatibility report entry

## Diagnostic Stability

Diagnostics should include:

```text
code
severity
message
span
feature id when applicable
help text when useful
```

Messages can improve over time, but codes should remain stable once published.

## Comparison Policy

Allowed comparison sources:

- public language documentation
- original mathematical formulas
- project-owned fixtures
- permissively licensed scripts with metadata
- user-provided scripts when the user has the right to use them

Disallowed:

- copied proprietary scripts
- private TradingView APIs
- scraped TradingView data
- copied official documentation text beyond short references
- TradingView UI or error text reproduction

## Release Compatibility Matrix

Every release should publish a generated or manually maintained matrix:

```text
feature              status       notes
indicator            supported
input.int            supported
ta.sma               supported
ta.ema               supported
ta.rsi               supported    fixture-derived executable subset
request.security     partial      same-context identity and same-or-higher-timeframe provider scalar-expression subset only
alertcondition       partial      bool-compatible condition plus const-string title/message runtime events
alert                partial      const-string message runtime events when execution reaches the call
strategy.*           unsupported  out of project scope for now
array.*              partial      float/int/bool/string/color creation and from inference, reference, copy, get/set/insert/remove with negative indexes, fill, slice/concat, search/binary search, float/int/bool truth helpers, numeric abs/statistics/range/median/mode/percentile/covariance/standardize/variance/stdev, numeric/string sort and sort_indices, join, mutation, and helper fixture subset only
request.security_lower_tf unsupported lower-timeframe array-returning request API is not implemented
request.*            unsupported  request families beyond the narrow request.security subsets
import               unsupported  out of Phase 1 scope
```

The matrix should be generated from conformance metadata once the test harness
exists.

Request support must cite request-specific fixtures. The conformance validator
rejects supported or partial `request.*` rows that only point at unrelated
runtime fixtures, so public request claims stay tied to request host-data,
semantic, or runtime coverage. The closed Phase F request boundary and
maintenance tails are summarized in `docs/PHASE_F_AUDIT.md`.

Current CLI output:

```text
pine-compat matrix
pine-compat matrix --format json
```

The generated matrix is derived from `tests/fixtures/conformance.tsv`. Each row
declares a feature, status, notes, and one or more fixture paths that back the
claim. CLI tests verify that every matrix entry references at least one existing
fixture. The text matrix includes the fixture paths, and the JSON matrix exposes
top-level matrix `schemaVersion` plus a `features` array whose entries expose
fixture paths as `fixtures`.

Conformance metadata is validated before matrix output is trusted:

- `feature` must be non-empty and unique.
- `status` must be `supported`, `partial`, or `unsupported`.
- `notes` must be non-empty.
- `fixtures` must contain at least one path and no empty `;` entries.
- Every fixture path must exist in the workspace.
- `supported` and `partial` entries must cite executable, realtime, syntax,
  positive semantic, or regression coverage.
- `unsupported` entries must cite unsupported semantic diagnostic fixtures.
- Every supported built-in registry entry and known unsupported platform family
  must remain represented.

Malformed rows, duplicate feature names, invalid statuses, missing fixture paths,
and status/fixture mismatches are first-class tests. The matrix command is
derived from the same validated metadata used by those tests.

Matrix maintenance rules:

- Edit `tests/fixtures/conformance.tsv` first; do not hand-edit generated
  matrix output.
- Add or update fixture paths in the same change as any new supported,
  partial, or unsupported claim.
- Keep unsupported platform families represented even when they remain outside
  the executable subset.
- If the JSON matrix shape changes, refresh `tests/snapshots/matrix.json` and
  document the public contract change in release notes.
- Use `pine-compat matrix --format json` to inspect the release matrix exposed
  to consumers.

The current scalar typed-array subset is summarized in
`docs/ARRAY_STAGE_AUDIT.md`. Keep `array.*` marked `partial` until the deferred
generic, object, UDT, map/matrix, history, and slice-aliasing semantics are
designed and fixture-backed.

The current `varip` subset is summarized in `docs/PHASE_I_AUDIT.md`. Keep
`varip` marked `partial` until drawing object ids, tuples, maps, matrices, UDTs,
imports, object arrays, generic arrays, and other value families have designed
rollback semantics and fixture coverage.
