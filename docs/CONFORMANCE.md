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
  "schemaVersion": 2,
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
  "diagnostics": []
}
```

The snapshot format should avoid host-specific charting details.

Every machine-readable public output must include top-level `schemaVersion`.
The version value comes from the shared runtime contract constant and must match
across CLI JSON, Python dictionaries, and WASM JSON. The text-only CLI
`analyze` output is diagnostic console output and is not part of the
machine-readable schema until a JSON mode is added.

CLI and WASM runtime JSON must be generated through the shared runtime contract
helper so field names and nesting cannot drift. Python returns native
dictionaries, so its binding tests assert the same top-level runtime keys and
representative nested output families such as `plotShapes` and `plotCandles`.
The Phase E drawing-object scaffold adds `labels` as a top-level runtime key in
`schemaVersion: 2`. The first executable drawing subset is `label.new` creation
with bar-index coordinates, price y-values, text, `xloc.bar_index`,
`yloc.price`, colors, selected label styles, size, and tooltip metadata. Keep
mutation, deletion, limits, realtime-specific behavior, unsupported coordinate
modes, and other drawing families out of the supported matrix until they have
fixtures and public-output coverage.

Checked-in golden JSON snapshots live in `tests/snapshots/`. Snapshot tests are
strict string comparisons against deterministic compact JSON; a public field
rename, omitted `schemaVersion`, or matrix shape change should fail tests. To
refresh snapshots after an intentional public-output change, run:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli golden_snapshot
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

- `request.security`
- `strategy.entry`
- unsupported collection families or unsupported array variants
- labels and lines
- imports
- `varip` in historical-only mode
- non-integer or negative history offsets
- unsupported function side effects

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
request.security     unsupported  out of Phase 1 scope
strategy.*           unsupported  out of project scope for now
array.*              partial      float/int/bool/string/color creation and from inference, reference, copy, get/set/insert/remove with negative indexes, fill, slice/concat, search/binary search, float/int/bool truth helpers, numeric abs/statistics/range/median/mode/percentile/covariance/standardize/variance/stdev, numeric/string sort and sort_indices, join, mutation, and helper fixture subset only
import               unsupported  out of Phase 1 scope
```

The matrix should be generated from conformance metadata once the test harness
exists.

Current CLI output:

```text
pine-compat matrix
pine-compat matrix --format json
```

The generated matrix is derived from `tests/fixtures/conformance.tsv`. Each row
declares a feature, status, notes, and one or more fixture paths that back the
claim. CLI tests verify that every matrix entry references at least one existing
fixture. The text matrix includes the fixture paths, and the JSON matrix exposes
top-level `schemaVersion` plus a `features` array whose entries expose fixture
paths as `fixtures`.

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
