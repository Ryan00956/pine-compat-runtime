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
  "meta": {
    "languageVersion": 5,
    "bars": 100
  },
  "plots": [],
  "hline": [],
  "fills": [],
  "inputs": [],
  "diagnostics": [],
  "compatibility": {
    "supported": [],
    "unsupported": []
  }
}
```

The snapshot format should avoid host-specific charting details.

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
- arrays
- labels and lines
- imports
- `varip` in historical-only mode
- dynamic history offsets if disabled
- stateful TA calls inside unsupported conditional contexts

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
ta.rsi               partial      requires rma warmup tests
request.security     unsupported  out of Phase 1 scope
strategy.*           unsupported  out of project scope for now
array.*              partial      float/int/bool/string/color creation and from inference, reference, copy, get/set/insert/remove with negative indexes, fill, slice/concat, search/binary search, numeric abs/statistics/range/median/mode/percentile/covariance/standardize/variance/stdev, ordering including sort_indices, join, mutation, and helper fixture subset only
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
them as a `fixtures` array.
