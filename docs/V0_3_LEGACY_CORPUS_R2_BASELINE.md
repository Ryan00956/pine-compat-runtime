# v0.3 Legacy Corpus R2 Baseline

## Outcome

A private, user-authorized TradingView community corpus is now materialized at
`.local/legacy-corpus-r2`. The source directory remains unchanged, all 104
copies are byte-identical to their inputs, and `.local/` is excluded through
the repository-local Git exclude file. Source text, original names, and the
private source map are not tracked.

The intake contains 44 eligible legacy indicators and 60 modern indicator
controls. It contains no strategy declarations. The legacy profile counts are
24 implicit-v1 and 20 v4 scripts; this intake contains no v2 or v3 samples and
therefore cannot improve those evidence profiles.

The first corpus-selected syntax slice is implemented. Legacy comma-separated
statements, leading/trailing decimal literals, and comment/blank lines before
the first statement in an indented block now parse without weakening the v5/v6
comma policy. Across the unchanged 44 eligible scripts, parse success increased
from 25 to 42 scripts (56.8% to 95.5%). v4 now parses 20/20; the two remaining
parse failures are implicit-v1 scripts using four-space multiline ternary
continuations. Analyze/lower and historical-run success remain 2/44, so no
runtime compatibility claim follows from the parser improvement.

The subsequent exact-alias slice adds `cross`, `round`, `rma`, and `wma` for
Pine v1-v4 and extends `highest` and `lowest` through the same range. It does
not change the 2/44 whole-script analysis or execution count, but it removes 92
unknown-function diagnostics and 35 dependent type/call diagnostics across 24
eligible scripts. That is a measured reduction in semantic noise, not a new
whole-script execution claim.

The second exact-alias slice adds `sqrt`, `stdev`, and `vwma` and extends
`change`, `abs`, `max`, `min`, and `crossover` through Pine v1-v4. This slice
does cross the whole-script boundary: analysis/lowering and historical
execution improve from 2/44 to 5/44, with all three new passes coming from the
implicit-v1 profile.

The third and fourth corpus-ranked call slices then add 16 more verified
legacy mappings. The fourth slice covers `barssince`, `crossunder`,
`heikinashi`, `log10`, `macd`, `sign`, and `valuewhen` across Pine v1-v4 plus
the Pine v4 `tostring(x, y)` signature reshape. It improves all 14 affected
scripts without changing the 5/44 whole-script execution count, because each
still has an independent blocker.

The fifth corpus-ranked slice adds `cci`, `ceil`, `log`, `mfi`, `mom`, `pow`,
`tr`, `obv`, and `vwap` across Pine v1-v4. `tr` and `vwap` retain their
historical variable/call distinctions, and `vwap(x)` is deliberately limited
to the historical one-source signature. All 12 affected scripts improve, two
implicit-v1 scripts become newly analyzable and executable, and the aggregate
whole-script result rises from 5/44 to 7/44.

The sixth slice completes the bounded Pine v1-v3 indicator-output family using
the documented pre-v4 parameter tables. Fifteen scripts advance through the
output binder, seven implicit-v1 scripts become newly analyzable and
executable, and the aggregate whole-script result doubles from 7/44 to 14/44.
Later `display`/`fillgaps` arguments remain rejected, and v5/v6 controls remain
isolated.

## Private Intake

The deterministic importer copies exact bytes to opaque content-derived ids,
writes a public-report-safe manifest, and keeps the original filename mapping
inside the ignored corpus directory:

```text
python3 scripts/import_legacy_corpus.py \
  --source-dir /path/to/private/pine \
  --output-dir .local/legacy-corpus-r2
```

The importer refuses to overwrite an existing output directory. Its private
license scan is only an intake hint: 34 MPL-2.0 headers, 26 CC BY-NC-SA-4.0
headers, one GPL header, two copyright-only headers, and 41 unspecified cases.
Every manifest row is therefore classified `private_user_authorized`; none of
these sources may be promoted to public fixtures solely from that hint.

Version and mode composition:

| Version selection | Items | Corpus role |
| --- | ---: | --- |
| implicit v1 | 24 | eligible legacy indicator |
| v4 | 20 | eligible legacy indicator |
| v5 | 43 | modern indicator control |
| v6 | 17 | modern indicator control |

Of the 104 version selections, 78 use an exact directive, 24 are implicit v1,
and two use a noncanonical but recognizable version comment. The raw sources
are not normalized; the private manifest records the intended version while
the runtime report exposes the expected/detected mismatch.

## Reproduction

Build and run the same manifest twice:

```text
cargo build -p pine-cli
python3 scripts/analyze_legacy_corpus.py \
  --manifest .local/legacy-corpus-r2/corpus.tsv \
  --root .local/legacy-corpus-r2 \
  --build-revision corpus-r2-syntax-slice \
  --output .local/legacy-corpus-r2/report-final-a.json
python3 scripts/analyze_legacy_corpus.py \
  --manifest .local/legacy-corpus-r2/corpus.tsv \
  --root .local/legacy-corpus-r2 \
  --build-revision corpus-r2-syntax-slice \
  --output .local/legacy-corpus-r2/report-final-b.json
```

Evidence hashes:

| Private/local asset | SHA-256 |
| --- | --- |
| `intake-summary.json` | `a313043e6038dc777983af1ea5d706124d164d961ef5768179bfdf95cd657e3f` |
| `corpus.tsv` | `c65c6e6a0ffb6297109372a1c358f199a599996ad255364ef04763468867b2f7` |
| `report-final-a.json` | `f51bbf2b46f0bf893bc9079bcbb308fc576e05d12716d8981e65dc03ae15104b` |
| `report-final-b.json` | `f51bbf2b46f0bf893bc9079bcbb308fc576e05d12716d8981e65dc03ae15104b` |
| `report-alias-final-a.json` | `cb6e3252b688662855a02fccd1706391e2f236bac8c3aa838be7ec9870946ffa` |
| `report-alias-final-b.json` | `cb6e3252b688662855a02fccd1706391e2f236bac8c3aa838be7ec9870946ffa` |
| `report-alias-2-final-a.json` | `cd2416f0b9a627f3ff26ba742fea7eb0ccff0c2e9c2026514cb39dcb166cbc9d` |
| `report-alias-2-final-b.json` | `cd2416f0b9a627f3ff26ba742fea7eb0ccff0c2e9c2026514cb39dcb166cbc9d` |
| `report-alias-3-final-a.json` | `eaa7e825540d602505acd666b5f725f05d8c40a47048a64128ab065362a2923f` |
| `report-alias-3-final-b.json` | `eaa7e825540d602505acd666b5f725f05d8c40a47048a64128ab065362a2923f` |
| `report-alias-4-final-a.json` | `6ed5fd6503787461772a90527c82ac07da2e7382ef852ca8d02d80a9c00fa3ec` |
| `report-alias-4-final-b.json` | `6ed5fd6503787461772a90527c82ac07da2e7382ef852ca8d02d80a9c00fa3ec` |
| `report-alias-5-final-a.json` | `f21000ac43fc74c9ddae95fdb864617f67d5dbbebb4ae2728061573e29c50e0e` |
| `report-alias-5-final-b.json` | `f21000ac43fc74c9ddae95fdb864617f67d5dbbebb4ae2728061573e29c50e0e` |
| `report-output-6-final-a.json` | `0c616f323dfb87f14f44f517b75eeaf4aa1a0ac2162e0e787c9adf718c8b8487` |
| `report-output-6-final-b.json` | `0c616f323dfb87f14f44f517b75eeaf4aa1a0ac2162e0e787c9adf718c8b8487` |
| `report-security-7-final-a.json` | `afd22ba81547cf9ebe8a1f6cfe0a6820488ea608400b7be4d1afddc19719705b` |
| `report-security-7-final-b.json` | `afd22ba81547cf9ebe8a1f6cfe0a6820488ea608400b7be4d1afddc19719705b` |

The matching report hashes prove deterministic reporting for this fixed
manifest and build label. They do not prove TradingView output equivalence;
the corpus contains no reference-output bundles.

## Measured Profiles

All rates use every eligible script in that profile as the denominator.

| Profile | Eligible | Parse before | Parse after | Analyze/lower | Historical run | Stable result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| v4 | 20 | 60.0% | 100.0% | 5.0% | 5.0% | blocked by size and execution |
| implicit v1 | 24 | 54.2% | 91.7% | 54.2% | 54.2% | blocked by parse, size, and execution |

The final aggregate report records:

- 42/44 parsing, 14/44 analysis/lowering, and 14/44 historical execution;
- zero strategy or scope mismatches and zero missing source/bar inputs;
- no supplied TradingView reference outputs;
- 21 scripts with a known unsupported feature, 12 with a call-argument type
  failure, five with a function-return failure, and three with an
  operator-type failure;
- 12 scripts blocked by the bounded legacy `security` subset, now the highest
  reach remaining unsupported family.

## Exact-Alias Remeasurement

After rebuilding the CLI, the unchanged manifest was measured twice with
`buildRevision=corpus-r2-alias-slice`:

```text
python3 scripts/analyze_legacy_corpus.py \
  --manifest .local/legacy-corpus-r2/corpus.tsv \
  --root .local/legacy-corpus-r2 \
  --build-revision corpus-r2-alias-slice \
  --output .local/legacy-corpus-r2/report-alias-final-a.json
```

The second run differs only in its `report-alias-final-b.json` output path. The
matching hashes prove deterministic reporting for the fixed build and corpus.
The 60 modern controls are byte-for-byte identical at the report-item level.

| Corpus metric | Syntax baseline | Alias slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 2/44 | 2/44 | 0 |
| Historical-run passes | 2/44 | 2/44 | 0 |
| Eligible diagnostic records | 1548 | 1421 | -127 |
| All unknown diagnostics | 693 | 601 | -92 |
| Unclassified unknown-function cluster | 36 scripts / 442 records | 28 / 366 | -8 scripts / -76 records |
| Operator-type cluster | 25 scripts | 20 scripts | -5 scripts |
| Unresolved `plot.series` cluster | 17 scripts | 11 scripts | -6 scripts |

The six aliases account for all 92 removed unknown diagnostics: `cross` 17,
`round` 13, `rma` 19, `wma` 27, `highest` 9, and `lowest` 7. Remaining failures
still fail closed; no permissive unknown return type or coercion was added.

### Second exact-alias slice

The unchanged manifest was then measured twice with
`buildRevision=corpus-r2-alias-slice-2` and the output paths
`report-alias-2-final-a.json` and `report-alias-2-final-b.json`. The hashes
match, and the 60 modern control report items remain identical to the preceding
slice.

| Corpus metric | First alias slice | Second alias slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 2/44 | 5/44 | +3 |
| Historical-run passes | 2/44 | 5/44 | +3 |
| Eligible diagnostic records | 1421 | 1307 | -114 |
| All unknown diagnostics | 601 | 549 | -52 |
| Unclassified unknown-function cluster | 28 scripts / 366 records | 22 / 344 | -6 scripts / -22 records |
| Operator-type cluster | 20 scripts | 15 scripts | -5 scripts |
| Unresolved `plot.series` cluster | 11 scripts | 7 scripts | -4 scripts |

Seventeen eligible scripts improve. The eight exact aliases account for all 52
removed unknown diagnostics: `change` 14, `abs` 2, `max` 6, `min` 6,
`crossover` 2, `sqrt` 8, `stdev` 5, and `vwma` 9. Another 62 dependent
diagnostics disappear after their return types become known. The three newly
executable scripts are all implicit-v1; v4 remains 1/20 executable in this
corpus and still needs broader function/output coverage.

### Third exact-alias slice

The unchanged manifest was measured twice again with
`buildRevision=corpus-r2-alias-slice-3` and the output paths
`report-alias-3-final-a.json` and `report-alias-3-final-b.json`. Their hashes
match. All 60 modern controls preserve identical stage outcomes and diagnostic
code/count behavior; two control report items only gain privacy-safe candidate
classification metadata because the analyzer now recognizes the newly audited
legacy names.

| Corpus metric | Second alias slice | Third alias slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 5/44 | 5/44 | 0 |
| Historical-run passes | 5/44 | 5/44 | 0 |
| Eligible diagnostic records | 1307 | 1069 | -238 |
| All unknown diagnostics | 549 | 394 | -155 |
| Unclassified unknown-function cluster | 22 scripts / 344 records | 15 / 189 | -7 scripts / -155 records |
| Operator-type cluster | 15 scripts | 8 scripts | -7 scripts |
| Unresolved `plot.series` cluster | 7 scripts | 2 scripts | -5 scripts |

Eighteen eligible scripts change and seventeen improve. The eight exact aliases
account for all 155 removed unknown-function diagnostics: `floor` 111,
`linreg` 9, `pivothigh` 7, `pivotlow` 7, `sum` 6, `atr` 5, `avg` 5, and
`stoch` 5. Resolving those callees also removes 86 dependent operator,
call-argument, and arity diagnostics. Three more precise call-argument errors
appear after one formerly unknown call becomes typed, so that script has one
additional diagnostic while still failing closed. No whole script becomes
newly executable in this slice because every improved failure still has an
independent unsupported or unresolved blocker.

### Fourth legacy-call slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-alias-slice-4` and the output paths
`report-alias-4-final-a.json` and `report-alias-4-final-b.json`. Their hashes
match, and all 60 modern control items retain identical stages and diagnostics.

| Corpus metric | Third alias slice | Fourth call slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 5/44 | 5/44 | 0 |
| Historical-run passes | 5/44 | 5/44 | 0 |
| Eligible diagnostic records | 1069 | 587 | -482 |
| All unknown diagnostics | 394 | 195 | -199 |
| Unclassified unknown-function cluster | 15 scripts / 189 records | 8 / 13 | -7 scripts / -176 records |
| Call-argument type cluster | 14 scripts / 102 records | 13 / 52 | -1 script / -50 records |
| Operator-type cluster | 8 scripts / 317 records | 5 / 89 | -3 scripts / -228 records |

Fourteen eligible scripts change and all fourteen improve. The seven exact
aliases plus the focused `tostring` reshape account for all 176 removed
unknown-function diagnostics: `barssince` 17, `crossunder` 12, `heikinashi` 9,
`log10` 102, `macd` 4, `sign` 8, `tostring` 12, and `valuewhen` 12. Resolving
their return types removes another 306 dependent unknown-symbol, operator,
call-argument, and arity diagnostics. No permissive fallback was added, and no
whole script becomes newly executable because the improved scripts retain
independent fail-closed blockers.

### Fifth legacy-call slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-alias-slice-5` and the output paths
`report-alias-5-final-a.json` and `report-alias-5-final-b.json`. Their hashes
match, and all 60 modern control items retain identical stages, diagnostic
codes, and complete diagnostic records.

| Corpus metric | Fourth call slice | Fifth call slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 5/44 | 7/44 | +2 |
| Historical-run passes | 5/44 | 7/44 | +2 |
| Eligible diagnostic records | 587 | 478 | -109 |
| All unknown diagnostics | 195 | 125 | -70 |
| Call-argument type cluster | 13 scripts / 52 records | 10 / 53 | -3 scripts / +1 record |
| Operator-type cluster | 5 scripts / 89 records | 3 / 49 | -2 scripts / -40 records |

Twelve eligible scripts change and all twelve improve. The verified mappings
remove 24 direct unknown records: `tr` 7, `obv` 4, `log` 3, `cci` 2, `ceil` 2,
`mfi` 2, `mom` 2, `pow` 1, and `vwap` 1. Resolving those names removes another
86 dependent unknown-symbol and operator errors; one formerly unresolved call
now reports a more precise call-argument error, for a net reduction of 109
diagnostics. The two newly executable scripts are both implicit-v1, bringing
that profile to 6/24 and the whole corpus to 7/44. No permissive fallback or
modern-dialect rewrite was added.

### Sixth legacy-output slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-output-slice-6` and the output paths
`report-output-6-final-a.json` and `report-output-6-final-b.json`. Their hashes
match. All 60 modern control items are identical to the fifth-slice report at
the complete item level.

| Corpus metric | Fifth call slice | Sixth output slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 7/44 | 14/44 | +7 |
| Historical-run passes | 7/44 | 14/44 | +7 |
| Eligible diagnostic records | 478 | 416 | -62 |
| Known-unsupported records | 173 | 111 | -62 |
| Known-unsupported affected scripts | 30 | 21 | -9 |
| All unknown diagnostics | 125 | 125 | 0 |
| Call-argument type records | 54 | 58 | +4 precise records |
| Legacy-output argument records | 4 | 0 | -4 |

Fifteen eligible scripts change. Fourteen have fewer diagnostics; the remaining
script replaces two blanket output rejections with two precise series-type
errors. Seven implicit-v1 scripts become newly executable, bringing that
profile from 6/24 to 13/24 and the whole corpus from 7/44 to 14/44. The slice
removes 62 known-unsupported output records, exposes four precise argument-type
errors, and resolves four pre-existing style/fill errors by preserving the old
ordinal meaning of primitive style constants in their receiving output family.
No private source or output reference was promoted into the repository.

### Seventh legacy-security slice

The unchanged manifest was measured with
`buildRevision=corpus-r2-security-slice-7`. The final reports are
`report-security-7-final-a.json` and `report-security-7-final-b.json`; their
hashes must match. All 60 modern control items remain identical to the sixth
slice at the complete item level.

| Corpus metric | Sixth output slice | Seventh security slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 14/44 | 21/44 | +7 |
| Historical-run passes | 14/44 | 16/44 | +2 |
| Eligible diagnostic records | 416 | 348 | -68 |
| Known-unsupported records | 111 | 47 | -64 |
| Known-unsupported affected scripts | 21 | 13 | -8 |
| All unknown diagnostics | 125 | 121 | -4 |
| Call-argument type records | 58 | 58 | 0 |
| Scripts still carrying legacy-security rejection | 12 | 4 | -8 |

Nine eligible scripts change and none regress. Seven newly analyze and lower;
two also execute with the inputs already present in the private corpus. Of the
other five newly lowered scripts, three correctly require provider streams not
present in the private manifest, while two expose unrelated legacy-input values
that are not valid request timeframe strings. The implicit-v1 profile rises
from 13/24 to 18/24 at analysis and from 13/24 to 14/24 at historical run.

The implemented boundary follows historical `security` expression behavior:
immutable top-level scalar aliases are recomputed in the requested context,
while const/input/simple dependencies are captured. Mutable or persistent
aliases, local blocks, cycles, UDF requested expressions, side effects, lower
timeframes, and missing host data stay explicit failures. No private source,
original filename, or request stream was committed.

## Next Selection

The next implementation slice should still be measured over this unchanged
manifest and should not add strategy behavior. The ranked order is now:

1. audit the four remaining legacy `security` scripts by precise boundary:
   UDF requested expressions, unresolved/mutable alias graphs, and independent
   input/call-shape failures; do not widen to arbitrary UDF or mutable state;
2. audit the remaining call-shape, function-return, unknown-symbol, and
   operator clusters without masking them with permissive return types or
   coercions;
3. rank the smaller unsupported groups (`study.resolution`, function side
   effects, and array mutation) by unique-script reach before selecting any
   implementation slice;
4. decide the two implicit-v1 four-space continuation cases explicitly; do not
   silently weaken structured block layout for modern Pine.

The v4 selection checkpoint originally required 30 total samples; this private
intake contributes 20, while the public seed contains 12. Counts must not be
blindly summed until duplicate content across the two manifests is checked.
Stable evidence still requires at least 50 eligible scripts per profile and the
separate incremental, realtime, provider, resource, cache, host-parity, and
release audits.
