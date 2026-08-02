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

The seventh and eighth slices extend only the legacy `security` boundary.
Immutable top-level requested aliases, capture-safe dependencies, and then
pure scalar UDF graphs plus legacy source inputs are recomputed in the requested
context. Analysis/lowering reaches 21/44 and historical execution 16/44 after
the seventh slice. The eighth slice removes four remaining blanket `security`
diagnostics without changing those whole-script totals because each affected
script still has an independent syntax or call-type failure.

The ninth slice implements only the Pine v1-v4 integer-call boundary exposed by
the corpus. A syntactic `int / int` expression is truncated when it is passed
to an integer-compatible built-in parameter, including through an untyped UDF
parameter whose body imposes that same constraint. Ordinary division remains
fractional, float operands remain rejected, and v5/v6 behavior is unchanged.
Three scripts become newly analyzable, lowerable, and executable, raising the
aggregate totals to 24/44 analysis/lowering and 19/44 historical execution.
The twenty-third slice later supersedes this temporary contextual
approximation with the documented whole-expression Pine v1-v4 rule; the ninth
slice measurements below remain as historical progression data.

The tenth slice closes the missing Pine v1-v5 numeric-to-bool built-in argument
path. Numeric or `na` arguments passed to bool-compatible parameters lower
through the canonical `bool(...)` cast while preserving their qualifier; v6
remains strict. Three legacy scripts improve, one becomes newly analyzable and
executable, and the aggregate totals rise to 25/44 analysis/lowering and 20/44
historical execution.

The eleventh slice aligns `array.get` and `array.set` with the official
integer-compatible index contract. Per-bar `series int` indexes are accepted
across legacy and modern Pine, while non-integer indexes and runtime bounds
checks remain strict. Four v4 scripts lose exactly twelve call-argument type
diagnostics; none changes whole-script stage because each retains an
independent blocker.

The twelfth slice separates supported dynamic drawing enums from arbitrary
strings. `label` style and `line` style/extend parameters accept a
string-compatible expression only when its complete value domain is statically
proven from immutable aliases, conditional branches, or explicit input
options. Pine v4's historical `label.style_labelup` and
`label.style_labeldown` spellings lower to current constants. Exactly four v4
scripts lose twelve type, fourteen value, and two unsupported-feature
diagnostics; three become newly analyzable, lowerable, and executable, raising
the aggregate totals to 28/44 analysis/lowering and 23/44 historical execution.

The thirteenth slice traces all ten remaining failure-derived `series na`
source/output arguments back to their producers instead of weakening consumer
types. Two come from Pine v1 bool-versus-numeric comparisons, three from an
independent implicit-v1 continuation parse failure, two from a valid
final-declaration UDF return outside the current function-return subset, and
three from a v4 UDF built-in call incorrectly hidden by a later global
declaration. Only the two independently proven producer rules change:
v1/v2 comparison bools lower through `float(...)`, and v3/v4 legacy call
shadowing respects declaration source order. Two scripts improve without any
added diagnostic; one implicit-v1 script becomes newly analyzable, lowerable,
and executable. Aggregate totals reach 29/44 analysis/lowering and 24/44
historical execution. The other five `na` argument records remain blocked at
their actual parse or function-return origin.

The fourteenth slice restores the pre-v6 output-offset behavior needed by one
v4 script. Pine v4/v5 `plot`, `plotchar`, `plotshape`, `plotarrow`, `bgcolor`,
and `barcolor` accept a `series int` offset and apply its final evaluated value
to the complete output; v3 and v6 retain simple-int-only typing. Exactly eight
`E_CALL_ARG_TYPE` records disappear from one v4 script. Whole-script stage
totals do not change because that script retains independent blockers, and all
60 modern controls remain item-identical.

The fifteenth slice implements UDF final-statement results across supported
Pine versions. A final local declaration or reassignment returns the bound
value, a final conditional recursively returns its branch-final statement, an
absent `else` supplies `na`, and a function-final side-effect loop may validly
return `void`. Five v4 scripts lose 58 diagnostics with no legacy diagnostic
added; one becomes newly analyzable, lowerable, and executable. Aggregate
totals reach 30/44 analysis/lowering and 25/44 historical execution.
Collection and drawing mutations inside UDFs remain independently rejected.

The sixteenth slice closes the two remaining parse failures with a deliberately
narrow source-origin rule. A no-directive v1 source at global scope may treat
exactly four ASCII spaces as layout-free only when the physical boundary is
adjacent to ternary `?` or `:` punctuation. Explicit v1-v6 sources, tabs, local
blocks, ordinary multiple-of-four indentation, and consumer typing remain
strict. Both affected indicators newly parse, analyze/lower, and execute.
Aggregate totals reach 44/44 parse, 32/44 analysis/lowering, and 27/44
historical execution while 29 diagnostics disappear. All 60 modern controls
remain item-identical.

The seventeenth slice closes the final implicit-v1 analysis failure without
relaxing unsafe graph nodes. An earlier `input()` used only as a source-order
inference prerequisite remains an ordinary declaration instead of being
predeclared or reordered with the self-history graph. Current-value forward
edges cannot cross such an unsafe-initializer declaration, and an `input()` that
is itself a graph target remains rejected. The same corpus item exposes the
official v1-v4 `rising` / `falling` aliases, which now lower exactly to
`ta.rising` / `ta.falling`; v5/v6 remain namespace-strict. Aggregate totals
reach 44/44 parse, 33/44 analysis/lowering, and 28/44 historical execution.
All 24 implicit-v1 samples now analyze, and 20/24 execute historically. All 60
modern controls remain item-identical.

The eighteenth slice classifies the remaining small-script time producer
without inventing runtime behavior. `timenow` is recognized as `series int`,
but remains known-unsupported because Pine defines it as the timestamp of each
script execution and the runtime has no host-provided execution-clock input.
The affected v4 script loses one unknown-symbol and two dependent
operator-type records, replacing them with one precise unsupported record.
Whole-script stages remain 44/44 parse, 33/44 analysis/lowering, and 28/44
historical execution. Two already-failing modern controls reclassify four
occurrences from unknown to known-unsupported; their total diagnostics and
stage outcomes are unchanged.

The nineteenth slice preserves concrete tuple destination types after a typed
producer is rejected exclusively by `E_UNSUPPORTED_FEATURE`. The original
unsupported diagnostics remain and HIR is still withheld; recursive or
otherwise erroneous producers remain outside recovery. Exactly one v4
indicator changes: its seven bounded legacy-`security` diagnostics remain,
while 78 unknown-symbol and eighteen dependent operator-type records disappear.
Whole-script stages remain 44/44 parse, 33/44 analysis/lowering, and 28/44
historical execution, while eligible diagnostics fall from 155 to 59. Four
already-failing modern controls lose 115 failure-derived diagnostics net with
no stage-outcome change.

The twentieth slice recognizes the corpus-proven `//@version = 4` spelling as
an explicit compiler annotation by allowing horizontal whitespace around its
equals sign. The `//@version` prefix remains exact, so `// @version=6` is still
an ordinary comment and its modern control remains unchanged. One v4 indicator
no longer falls back to implicit v1: fourteen false
`E_LEGACY_VERSION_FEATURE` records and one false `E_CALL_ARG_NAME` disappear,
and the script advances through analysis, lowering, and historical execution.
Aggregate totals reach 44/44 parse, 34/44 analysis/lowering, and 29/44
historical execution with 44 eligible diagnostics.

The twenty-first slice traces both remaining `E_BRANCH_RETURN` records to the
same valid control-flow shape: a value-producing outer `if` branch ends with a
complete nested `if`/`else-if`/`else` statement. Branch analysis, static type
queries, and lowering now recurse through that final conditional, while a
nested leaf ending in a non-value statement remains rejected. The affected v4
indicator advances through analysis and lowering, raising the aggregate to
35/44, but exposes a separate runtime/host failure and does not increase the
29/44 historical total. Eligible diagnostics fall from 44 to the 42
known-unsupported records only.

The twenty-second slice closes that independent runtime failure. Input metadata
and legacy lowering already identified the canonical `defval`, but historical
execution incorrectly evaluated the first raw HIR argument. A fully named v4
call with `title` before `defval` therefore returned its title string as the
input value. Runtime input evaluation now resolves the canonical `defval`
argument by name, with the existing positional fallback for ordinary calls.
The affected v4 indicator executes historically, raising the aggregate to
30/44 without changing parse, analysis/lowering, or diagnostics. All 60 modern
controls remain item-identical.

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

Of the 104 version selections, 78 use the standard exact directive, 24 are
implicit v1, one uses the spaced-equals compatibility spelling, and one uses a
comment-like noncanonical spelling that intentionally remains implicit v1.
The raw sources are not normalized; the private manifest records the intended
version while the runtime report exposes any expected/detected mismatch.

## Reproduction

Build and run the same manifest twice:

```text
cargo build -p pine-cli
python3 scripts/analyze_legacy_corpus.py \
  --manifest .local/legacy-corpus-r2/corpus.tsv \
  --root .local/legacy-corpus-r2 \
  --build-revision corpus-r2-input-default-slice-22 \
  --output .local/legacy-corpus-r2/report-input-default-22-final-a.json
python3 scripts/analyze_legacy_corpus.py \
  --manifest .local/legacy-corpus-r2/corpus.tsv \
  --root .local/legacy-corpus-r2 \
  --build-revision corpus-r2-input-default-slice-22 \
  --output .local/legacy-corpus-r2/report-input-default-22-final-b.json
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
| `report-security-8-final-a.json` | `2c1b61fa628c118e19c1fde3451a527c7253727e048de86a3e767d75fe1ebe3b` |
| `report-security-8-final-b.json` | `2c1b61fa628c118e19c1fde3451a527c7253727e048de86a3e767d75fe1ebe3b` |
| `report-length-9-final-a.json` | `b8c12abcf9d263abb34a9bea285f973e9ff19d5f746e9761eb7c9671769a3ce5` |
| `report-length-9-final-b.json` | `b8c12abcf9d263abb34a9bea285f973e9ff19d5f746e9761eb7c9671769a3ce5` |
| `report-bool-10-final-a.json` | `1c169fb676899f37b8e93e6b7a92f69693919a217fc39460f32153458467128b` |
| `report-bool-10-final-b.json` | `1c169fb676899f37b8e93e6b7a92f69693919a217fc39460f32153458467128b` |
| `report-array-index-11-final-a.json` | `b7fd8cd7e66d480ede4a687c4b7905da945002fabf2d6c7c68044e8f824b2270` |
| `report-array-index-11-final-b.json` | `b7fd8cd7e66d480ede4a687c4b7905da945002fabf2d6c7c68044e8f824b2270` |
| `report-drawing-enum-12-final-a.json` | `914457535fb95f60ee3b183ec4632e156ca3fb5cf1031449b2ddc6277ded7fa8` |
| `report-drawing-enum-12-final-b.json` | `914457535fb95f60ee3b183ec4632e156ca3fb5cf1031449b2ddc6277ded7fa8` |
| `report-na-origin-13-final-a.json` | `f61956f0b2d4be063a16fa72b1e21d791ac57e63b91507359d0156d494b8f901` |
| `report-na-origin-13-final-b.json` | `f61956f0b2d4be063a16fa72b1e21d791ac57e63b91507359d0156d494b8f901` |
| `report-output-offset-14-final-a.json` | `23cb5248fce14249b7c5e989a1b7d8592091822bd518023e5c3a913e0baec052` |
| `report-output-offset-14-final-b.json` | `23cb5248fce14249b7c5e989a1b7d8592091822bd518023e5c3a913e0baec052` |
| `report-function-return-15-final-a.json` | `758861781ae9d6d72ad4a80829c00f98eece59931ca95bf699fbdf405bd0613a` |
| `report-function-return-15-final-b.json` | `758861781ae9d6d72ad4a80829c00f98eece59931ca95bf699fbdf405bd0613a` |
| `report-continuation-16-final-a.json` | `6ce7996cae25765e109732ae87ea387b5e2ddc666b423dcb5df5f9a73ea4c734` |
| `report-continuation-16-final-b.json` | `6ce7996cae25765e109732ae87ea387b5e2ddc666b423dcb5df5f9a73ea4c734` |
| `report-graph-17-final-a.json` | `fdb910afee5cabaa455ad587eb3ea9598e1a6a521262809e5806f0a712ed1162` |
| `report-graph-17-final-b.json` | `fdb910afee5cabaa455ad587eb3ea9598e1a6a521262809e5806f0a712ed1162` |
| `report-timenow-18-final-a.json` | `b8b304806984e0e7b293e0e298894cb6e83c385511464b65a8765fe2ed4244fd` |
| `report-timenow-18-final-b.json` | `b8b304806984e0e7b293e0e298894cb6e83c385511464b65a8765fe2ed4244fd` |
| `report-tuple-19-final-a.json` | `1a10f14b4767d0d6cd932acb08e3b41c1ae9c292f579c1f5aff380e9b16f9e8d` |
| `report-tuple-19-final-b.json` | `1a10f14b4767d0d6cd932acb08e3b41c1ae9c292f579c1f5aff380e9b16f9e8d` |
| `report-version-20-final-a.json` | `daf09bcc9f8f422977a5293ef5759804ae8def343be5b46c07877edb50d959cc` |
| `report-version-20-final-b.json` | `daf09bcc9f8f422977a5293ef5759804ae8def343be5b46c07877edb50d959cc` |
| `report-branch-return-21-final-a.json` | `82a7cecb02e0c9c31743d19cdc9758ec134c9913c03500e1dd8058afb9073ada` |
| `report-branch-return-21-final-b.json` | `82a7cecb02e0c9c31743d19cdc9758ec134c9913c03500e1dd8058afb9073ada` |
| `report-input-default-22-final-a.json` | `4765800bd8aee74e178d97e3673fb028b98ca29658b37f1bf71b0f77bd00f756` |
| `report-input-default-22-final-b.json` | `4765800bd8aee74e178d97e3673fb028b98ca29658b37f1bf71b0f77bd00f756` |

The matching report hashes prove deterministic reporting for this fixed
manifest and build label. They do not prove TradingView output equivalence;
the corpus contains no reference-output bundles.

## Measured Profiles

All rates use every eligible script in that profile as the denominator.

| Profile | Eligible | Parse before | Parse after | Analyze/lower | Historical run | Stable result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| v4 | 20 | 60.0% | 100.0% | 55.0% | 50.0% | blocked by size and execution |
| implicit v1 | 24 | 54.2% | 100.0% | 100.0% | 83.3% | blocked by size |

The final aggregate report records:

- 44/44 parsing, 35/44 analysis/lowering, and 30/44 historical execution;
- 42 eligible diagnostic records, all at known-unsupported boundaries;
- zero strategy or scope mismatches and zero missing source/bar inputs;
- no supplied TradingView reference outputs;
- nine scripts with a known unsupported feature and no remaining unknown-symbol,
  operator-type, or branch-return diagnostics;
- one script still carries seven bounded legacy-`security` rejections.

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

### Eighth legacy-security slice

The unchanged manifest was measured with
`buildRevision=corpus-r2-security-slice-8`. The final reports are
`report-security-8-final-a.json` and `report-security-8-final-b.json`; their
hashes match. All 60 modern control items remain identical to the seventh slice
at the complete item level.

| Corpus metric | Seventh security slice | Eighth security slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 21/44 | 21/44 | 0 |
| Historical-run passes | 16/44 | 16/44 | 0 |
| Eligible diagnostic records | 348 | 344 | -4 |
| Known-unsupported records | 47 | 43 | -4 |
| Known-unsupported affected scripts | 13 | 10 | -3 |
| All unknown diagnostics | 121 | 121 | 0 |
| Call-argument type records | 58 | 58 | 0 |
| Scripts still carrying legacy-security rejection | 4 | 1 | -3 |

Three eligible scripts improve and none regress. Two lose UDF-related
`security` rejections and one loses the source-input rejection; none becomes a
whole-script pass because independent call-type or parse/name-resolution errors
remain.

The bounded extension accepts legacy provider expressions made from nested pure
scalar UDFs. Expression bodies and block bodies containing only normal
immutable declarations are allowed; UDF arguments, top-level immutable aliases,
legacy `input.source` defaults, and the legacy `int` cast are evaluated in the
requested context. Lowered UDF temporaries and immutable locals are treated as
lexically bound dependencies rather than mistaken for top-level captures.
Persistent declarations, reassignment, recursion, control-flow blocks,
side-effecting calls, mutable captures, and lower-timeframe requests remain
fail-closed. The modern `request.security` analyzer was not widened.

### Ninth contextual-integer slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-length-slice-9`. The final reports are
`report-length-9-final-a.json` and `report-length-9-final-b.json`; their hashes
match. All 60 modern control items remain identical to the eighth slice at the
complete item level.

| Corpus metric | Eighth security slice | Ninth integer slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 21/44 | 24/44 | +3 |
| Historical-run passes | 16/44 | 19/44 | +3 |
| Eligible diagnostic records | 344 | 334 | -10 |
| Known-unsupported records | 43 | 43 | 0 |
| All unknown diagnostics | 121 | 121 | 0 |
| Call-argument type records | 58 | 48 | -10 |
| Scripts with a call-argument type error | 12 | 9 | -3 |

Exactly three eligible scripts change and none regress. Each loses only the
expected integer-parameter diagnostics, then passes analysis, lowering, and
historical execution. The implicit-v1 profile rises from 18/24 to 20/24 at
analysis and from 14/24 to 16/24 at historical run; v4 rises from 3/20 to 4/20
and from 2/20 to 3/20 respectively.

The compatibility rule is deliberately contextual. For Pine v1-v4 only, a
syntactic division whose two operands are integers may be lowered through the
canonical `int(...)` conversion when the receiving built-in parameter requires
an integer. The same constraint can propagate through an untyped UDF parameter
when that parameter is used in an integer-compatible built-in position.
Division used as an ordinary value remains fractional; float operands,
unconstrained UDF arguments, and Pine v5/v6 calls do not gain a coercion.
This describes the ninth-slice implementation only. The twenty-third slice
below replaces it with version-wide Pine v1-v4 integer-division semantics.

### Tenth numeric-bool call slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-bool-call-slice-10`. The final reports are
`report-bool-10-final-a.json` and `report-bool-10-final-b.json`; their hashes
match.

| Corpus metric | Ninth integer slice | Tenth bool-call slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 24/44 | 25/44 | +1 |
| Historical-run passes | 19/44 | 20/44 | +1 |
| Eligible diagnostic records | 334 | 316 | -18 |
| Known-unsupported records | 43 | 43 | 0 |
| All unknown diagnostics | 121 | 109 | -12 |
| Call-argument type records | 48 | 42 | -6 |
| Scripts with a call-argument type error | 9 | 8 | -1 |

Exactly three eligible legacy scripts change and all improve. Six direct
numeric-to-bool argument errors disappear; resolving their return types removes
twelve dependent unknown-symbol diagnostics. One v4 script becomes a complete
analysis, lowering, and historical-run pass, taking that profile from 4/20 to
5/20 at analysis and from 3/20 to 4/20 at historical run. The implicit-v1
profile is unchanged.

The rule follows the pre-v6 language boundary rather than the private corpus
classification. Numeric and `na` arguments passed to an explicitly
bool-compatible built-in parameter lower through `bool(...)` in Pine v1-v5,
with the source qualifier retained so simple-only parameters remain bounded.
This also removes six expected diagnostics from two v5 control scripts without
changing either script's stage status. All seventeen v6 control items remain
identical at the complete report-item level.

### Eleventh array-index slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-array-index-slice-11`. The final reports are
`report-array-index-11-final-a.json` and
`report-array-index-11-final-b.json`; their hashes match.

| Corpus metric | Tenth bool-call slice | Eleventh array-index slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 25/44 | 25/44 | 0 |
| Historical-run passes | 20/44 | 20/44 | 0 |
| Eligible diagnostic records | 316 | 304 | -12 |
| Known-unsupported records | 43 | 43 | 0 |
| All unknown diagnostics | 109 | 109 | 0 |
| Call-argument type records | 42 | 30 | -12 |
| Scripts with a call-argument type error | 8 | 7 | -1 |

Exactly four eligible v4 scripts change and none regresses. They lose
respectively 2, 2, 6, and 2 call-argument type diagnostics, with no added
diagnostic and no stage transition because independent drawing or
function-side-effect blockers remain.

This is a general signature correction, not a legacy-only coercion.
`array.get` and `array.set` now accept integer-compatible indexes, including a
per-bar `series int`, in namespace and method forms. Their runtimes already
evaluated the index on each bar and retained strict bounds errors. Other
indexed array helpers keep their separately fixture-backed simple-index
contracts.

Eight modern controls also improve: four v5 and four v6 items lose 35
call-argument type records and twelve dependent unknown-symbol records. No
modern item changes stage, no diagnostic is added, and the other 52 controls
remain item-identical.

### Twelfth drawing-enum slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-drawing-enum-slice-12`. The final reports are
`report-drawing-enum-12-final-a.json` and
`report-drawing-enum-12-final-b.json`; their hashes match.

| Corpus metric | Eleventh array-index slice | Twelfth drawing-enum slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 25/44 | 28/44 | +3 |
| Historical-run passes | 20/44 | 23/44 | +3 |
| Eligible diagnostic records | 304 | 276 | -28 |
| Known-unsupported records | 43 | 41 | -2 |
| All unknown diagnostics | 109 | 109 | 0 |
| Call-argument type records | 30 | 18 | -12 |
| Call-argument value records | 14 | 0 | -14 |
| Scripts with a call-argument type error | 7 | 4 | -3 |

Exactly four eligible v4 scripts change and none regresses. Across them, the
slice removes twelve `E_CALL_ARG_TYPE`, fourteen `E_CALL_ARG_VALUE`, and two
`E_UNSUPPORTED_FEATURE` records without adding a diagnostic. Three scripts
advance from analysis failure through lowering and historical execution; the
fourth retains an independent blocker.

The current [`label.new`](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new)
and [`line.new`](https://www.tradingview.com/pine-script-reference/v6/#fun_line.new)
references type these enum-bearing roles as `series string`. TradingView's
[Pine v4 drawings documentation](https://www.tradingview.com/pine-script-docs/v4/essential/drawings/)
also demonstrates dynamic label styles and a conditional
`extend.right` / `extend.none` line argument. The compatibility surface remains
deliberately bounded: immutable ternary/if/switch branches and string inputs
with explicit `options` can prove a finite supported domain, while unbounded
input strings, reassigned aliases, unknown producers, missing switch defaults,
and any invalid branch fail closed.

Pine v4-only exact symbol translations map `label.style_labelup` and
`label.style_labeldown` to their current underscored equivalents. Those
spellings are evidenced by TradingView's
[Pine v4 launch example](https://www.tradingview.com/blog/en/introducing-pine-script-4-12626/)
and remain rejected in v5/v6. A public v4 fixture verifies canonical runtime
snapshot values, while modern positive and negative fixtures lock the bounded
domain distinction.

Thirteen modern controls also improve: eight v5 and five v6 items lose 75
call-argument type and 33 value records. One v5 control becomes newly
analyzable, lowerable, and executable. No modern diagnostic is added, no item
regresses, and the other 47 controls remain item-identical.

### Thirteenth `na`-origin slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-na-origin-slice-13`. The final reports are
`report-na-origin-13-final-a.json` and
`report-na-origin-13-final-b.json`; their hashes match.

| Corpus metric | Twelfth drawing-enum slice | Thirteenth `na`-origin slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 28/44 | 29/44 | +1 |
| Historical-run passes | 23/44 | 24/44 | +1 |
| Eligible diagnostic records | 276 | 253 | -23 |
| Known-unsupported records | 41 | 41 | 0 |
| All unknown diagnostics | 109 | 93 | -16 |
| Call-argument type records | 18 | 13 | -5 |
| Scripts with a call-argument type error | 4 | 2 | -2 |

The ten apparent `series na` source/output records were secondary diagnostics:

| Producer origin | Records | Disposition |
| --- | ---: | --- |
| Pine v1 bool result compared with a numeric literal | 2 | fixed at the comparison producer |
| implicit-v1 four-space ternary continuation parse failure | 3 | unchanged; parser decision remains separate |
| Pine v4 UDF whose last statement is a variable declaration | 2 | unchanged; belongs to the function-return slice |
| Pine v4 UDF built-in hidden by a later global declaration | 3 | fixed by source-ordered call shadowing |

Exactly two legacy scripts change. The implicit-v1 item loses two
`E_OPERATOR_TYPE` and two dependent `E_CALL_ARG_TYPE` records and advances
through analysis, lowering, and historical execution. The v4 item loses one
`E_UNKNOWN_FUNCTION`, fifteen dependent `E_UNKNOWN_SYMBOL`, and three
`E_CALL_ARG_TYPE` records while retaining independent blockers. Neither item
adds a diagnostic.

TradingView's official
[v3 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-3)
documents the v1/v2 implicit boolean-to-number rule that v3 removed. The
lowering therefore reuses the existing explicit `float(...)` compatibility
path only in v1/v2. TradingView's
[variable declaration rules](https://www.tradingview.com/pine-script-docs/language/variable-declarations/)
make a declaration visible from its declaration point onward, so a global
written after a v3/v4 UDF body cannot retroactively hide that body's
historical built-in call. An earlier lexical value still shadows the alias and
remains a non-callable-value error.

No `plot` or `ta.*` acceptor was broadened. The remaining five failure-derived
`na` records stay attached to their parse/function-return blockers, and a bare
or otherwise unresolved `na` argument remains rejected. Public legacy runtime
fixtures compare both producer corrections with explicit v6 rewrites; v3 and
earlier-shadow negative fixtures preserve the version and source-order
boundaries. All 60 modern controls are item-identical.

### Fourteenth series-output-offset slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-output-offset-slice-14`. The final reports are
`report-output-offset-14-final-a.json` and
`report-output-offset-14-final-b.json`; their hashes match.

| Corpus metric | Thirteenth `na`-origin slice | Fourteenth output-offset slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 29/44 | 29/44 | 0 |
| Historical-run passes | 24/44 | 24/44 | 0 |
| Eligible diagnostic records | 253 | 245 | -8 |
| Known-unsupported records | 41 | 41 | 0 |
| All unknown diagnostics | 93 | 93 | 0 |
| Call-argument type records | 13 | 5 | -8 |
| Scripts with a call-argument type error | 2 | 2 | 0 |

Exactly one v4 script changes: its eight `plot`/`plotshape` `offset`
arguments lose `E_CALL_ARG_TYPE`, while fifty unrelated diagnostics remain.
No stage changes and no diagnostic is added. The five remaining
call-argument-type records are all secondary to the already separated
implicit-v1 parse and v4 function-return failures.

TradingView's official
[v6 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/#no-series-offset-values)
states that Pine v5 accepted `series int` values for `plot()` and similar
`offset` parameters, warned about them, and applied only the last calculated
offset to the entire chart; v6 requires a simple-or-weaker integer. The corpus
supplies the matching v4 compatibility case. The implementation is therefore
deliberately limited to v4/v5 `plot`, `plotchar`, `plotshape`, `plotarrow`,
`bgcolor`, and `barcolor`. It retains each offset expression in HIR, and the
existing runtime metadata update naturally preserves the final evaluated
value for the whole output.

Public v4 and v5 runtime fixtures are identical to an explicit v6 rewrite that
uses the known final constant. V3 and v6 negative fixtures preserve the
simple-int boundary. Ordinary `expr[offset]` history indexing, `ta.*` offset
parameters, and every non-output simple-int consumer remain unchanged. All 60
modern controls are item-identical.

### Fifteenth UDF final-statement slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-function-return-slice-15`. The final reports are
`report-function-return-15-final-a.json` and
`report-function-return-15-final-b.json`; their hashes match.

| Corpus metric | Fourteenth output-offset slice | Fifteenth UDF-return slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 42/44 | 0 |
| Analyze/lower passes | 29/44 | 30/44 | +1 |
| Historical-run passes | 24/44 | 25/44 | +1 |
| Eligible diagnostic records | 245 | 187 | -58 |
| Known-unsupported records | 41 | 41 | 0 |
| All unknown diagnostics | 93 | 93 | 0 |
| Function-return records | 24 | 0 | -24 |
| Loop-return records | 1 | 0 | -1 |
| Call-argument type records | 5 | 3 | -2 |
| Operator-type records | 47 | 20 | -27 |
| Assignment-type records | 4 | 0 | -4 |

The 24 function-return records across five v4 scripts split into fifteen final
local declarations, three final local reassignments, and six final
conditionals without `else` used for side effects. One of those functions also
ended in a loop whose final mutation produced `void`. Supporting the actual
producer results removes all 25 return diagnostics, two failure-derived
call-argument diagnostics, and 31 dependent operator/assignment diagnostics.
Exactly one script crosses the whole-script boundary; the other four retain
independent unsupported or type/name blockers. No legacy diagnostic is added,
and the known-unsupported count is unchanged.

TradingView's official
[Pine v4 function declaration documentation](https://www.tradingview.com/pine-script-docs/v4/language/declaring-functions/)
states that the final expression or declared variable supplies a function's
result. The
[current user-defined-function documentation](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)
generalizes the same final-statement rule to variables, tuples, conditionals,
and loops. The implementation covers the corpus-proven single-variable
declaration/reassignment and conditional/loop shapes. It does not relax the
separate collection/drawing side-effect policy or claim broader tuple/complex
identity support.

The rule is language semantics rather than a legacy-only translation, so 19
modern controls also change: fifteen v5 and four v6 items. Their stage outcomes
are unchanged, while their aggregate diagnostic count falls by 101. Eight
deeper loop, tuple, or call diagnostics across seven controls become newly
reachable after the false outer function-return blockers are removed; these
are retained rather than suppressed.

### Sixteenth implicit-v1 ternary-continuation slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-continuation-slice-16`. The final reports are
`report-continuation-16-final-a.json` and
`report-continuation-16-final-b.json`; their hashes match.

| Corpus metric | Fifteenth UDF-return slice | Sixteenth continuation slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 42/44 | 44/44 | +2 |
| Analyze/lower passes | 30/44 | 32/44 | +2 |
| Historical-run passes | 25/44 | 27/44 | +2 |
| Eligible diagnostic records | 187 | 158 | -29 |
| Known-unsupported records | 41 | 41 | 0 |
| All unknown diagnostics | 93 | 79 | -14 |
| Parse-expression records | 12 | 0 | -12 |
| Call-argument type records | 3 | 0 | -3 |

Exactly two implicit-v1 indicators change. Both use top-level ternaries whose
continuation lines begin with four spaces: one places `:` at the preceding
line's end, and one places `?` / `:` at the following lines' starts. Both
advance through every measured stage and add no diagnostic. The twelve parser
records, three failure-derived output argument records, and fourteen recovery
name records disappear at their shared producer.

TradingView's current
[script-structure documentation](https://www.tradingview.com/pine-script-docs/language/script-structure/#line-wrapping)
states both that a missing directive selects v1 and that ordinary
non-parenthesized wraps generally avoid multiples of four. The archived
[v3 line-wrapping documentation](https://www.tradingview.com/pine-script-docs/v3/language/lines-wrapping)
and [v4 line-wrapping documentation](https://www.tradingview.com/pine-script-docs/v4/language/line-wrapping/)
preserve that general block-layout rule. The implementation therefore does not
claim a broad four-space Pine rule: it admits only the two corpus-proven
no-directive, global, ternary-punctuation shapes using exactly four ASCII
spaces. An explicit directive of any supported version, a tab, a local-block
continuation, or an ordinary arithmetic wrap remains structural.

The public implicit-v1 fixture is runtime-identical to a single-line v6 rewrite
across both punctuation placements. Syntax and semantic negative controls keep
ordinary and explicit-version multiple-of-four wrapping rejected. The release
registry now has 28 bounded profiles, translator revision 20 prevents reuse
across the front-end semantic change, and all 60 modern control report items
retain the same complete-item hash as the fifteenth slice.

### Seventeenth source-order declaration-prerequisite slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-graph-prerequisite-slice-17`. The final reports are
`report-graph-17-final-a.json` and `report-graph-17-final-b.json`; their hashes
match.

| Corpus metric | Sixteenth continuation slice | Seventeenth graph slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 32/44 | 33/44 | +1 |
| Historical-run passes | 27/44 | 28/44 | +1 |
| Eligible diagnostic records | 158 | 157 | -1 |
| Known-unsupported records | 41 | 41 | 0 |
| All unknown diagnostics | 79 | 79 | 0 |
| Unsafe-reference-graph records | 1 | 0 | -1 |

Exactly one implicit-v1 indicator changes. Its self-history declaration chain
uses a scalar `input()` declared earlier in source order. The old pass placed
every transitive inference dependency into the active graph and consequently
rejected that earlier input even though neither predeclaration nor topological
movement could affect it. The refined pass keeps separate sets for actual
self/forward graph nodes, predeclared targets, and the bounded inference
closure. Only actual graph nodes receive the unsafe-initializer restriction;
the earlier input stays on the ordinary analyzer path and in its original
execution position.

This is not a general call-in-graph relaxation. An input that is itself a
forward target still receives `E_LEGACY_REFERENCE_GRAPH_UNSAFE`, and a
current-value forward edge that would move across an input or another unsafe
initializer receives `E_LEGACY_FORWARD_REFERENCE_UNSAFE`. Public fixtures cover
both negatives plus runtime equality against an explicit v6 rewrite.

Removing the graph false positive exposes two calls in the same source:
`rising` and `falling`. TradingView's
[v3 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-3/)
documents v2 self/forward declarations and their explicit modern rewrites, and
the official
[v5 migration mapping](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-5/)
maps v4 `rising()` / `falling()` to `ta.rising()` / `ta.falling()`. The two
calls therefore use exact v1-v4 aliases rather than a shape-specific
workaround; v5/v6 unqualified negative controls remain errors.

The affected indicator advances through analysis, lowering, and historical
execution with no remaining diagnostic. The implicit-v1 profile reaches 24/24
analysis/lowering and 20/24 historical execution, meeting the provisional rate
thresholds but remaining blocked by its 24-script evidence count and the
separate full release audits. The 60 modern control items retain the exact
complete-item hash
`7308511fb4c3d94d780e994642e78e5433f8f2c97579f95a178a08cb00ba192c`.
The release registry now has 29 bounded profiles, and translator revision 21
prevents cache reuse across the graph and alias changes.

### Eighteenth typed execution-clock boundary slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-timenow-boundary-slice-18`. The final reports are
`report-timenow-18-final-a.json` and `report-timenow-18-final-b.json`; their
matching SHA-256 is
`b8b304806984e0e7b293e0e298894cb6e83c385511464b65a8765fe2ed4244fd`.

| Corpus metric | Seventeenth graph slice | Eighteenth clock-boundary slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 33/44 | 33/44 | 0 |
| Historical-run passes | 28/44 | 28/44 | 0 |
| Eligible diagnostic records | 157 | 155 | -2 |
| Known-unsupported records | 41 | 42 | +1 |
| All unknown diagnostics | 79 | 78 | -1 |
| Operator-type records | 20 | 18 | -2 |
| Scripts with unknown-symbol diagnostics | 2 | 1 | -1 |
| Scripts with operator-type diagnostics | 2 | 1 | -1 |

Exactly one v4 indicator changes. Its range calculation subtracts `time` from
`timenow` and then compares that result with integer durations. Treating the
unresolved producer as `na` previously emitted one `E_UNKNOWN_SYMBOL` and two
failure-derived `E_OPERATOR_TYPE` records. The analyzer now retains the
official `series int` type while emitting one `E_UNSUPPORTED_FEATURE`, so
downstream arithmetic and comparisons remain correctly typed without producing
HIR.

This is a diagnostic-boundary improvement, not execution support. TradingView's
[Pine v4 session/time documentation](https://www.tradingview.com/pine-script-docs/v4/essential/sessions-and-time-functions/)
describes `timenow` as current UNIX time in milliseconds, and the
[current time documentation](https://www.tradingview.com/pine-script-docs/concepts/time/#timenow)
clarifies that its values correspond to script executions rather than bar
times. The core has no per-execution host timestamp contract and deliberately
does not read a process wall clock. Substituting `time` or `last_bar_time` would
therefore be observably wrong on both loaded history and realtime updates.

Two already-failing modern controls contain four additional `timenow`
occurrences. Those records move from `E_UNKNOWN_SYMBOL` to
`E_UNSUPPORTED_FEATURE`; their combined diagnostic count remains 6,385 and no
stage outcome changes. The 60-control complete-item hash changes intentionally
from
`7308511fb4c3d94d780e994642e78e5433f8f2c97579f95a178a08cb00ba192c`
to
`94cb795414ae56f0a47e36afa738ad4bcf95c242c82a72f86642b392e5bcee3c`.
The bounded release registry remains at 29 executable profiles, and translator
revision 22 prevents semantic-cache reuse across the new classification.

### Nineteenth typed unsupported tuple-producer recovery slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-typed-tuple-recovery-slice-19`. The final reports are
`report-tuple-19-final-a.json` and `report-tuple-19-final-b.json`; their
matching SHA-256 is
`1a10f14b4767d0d6cd932acb08e3b41c1ae9c292f579c1f5aff380e9b16f9e8d`.

| Corpus metric | Eighteenth clock-boundary slice | Nineteenth tuple-recovery slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 33/44 | 33/44 | 0 |
| Historical-run passes | 28/44 | 28/44 | 0 |
| Eligible diagnostic records | 155 | 59 | -96 |
| Known-unsupported records | 42 | 42 | 0 |
| All unknown diagnostics | 78 | 0 | -78 |
| Operator-type records | 18 | 0 | -18 |
| Scripts with unknown-symbol diagnostics | 1 | 0 | -1 |
| Scripts with operator-type diagnostics | 1 | 0 | -1 |

Exactly one v4 indicator changes. Several UDFs return tuples whose elements
have concrete types even though their internal legacy `security` expressions
remain outside the supported provider-expression subset. The previous tuple
declaration path discarded those element types after seeing the producer
errors, leaving every destructured destination unbound. Downstream reads then
produced 78 `E_UNKNOWN_SYMBOL` and eighteen `E_OPERATOR_TYPE` records in
addition to the seven actual security boundaries.

The analyzer now establishes tuple destinations only when the initializer has
a concrete tuple type and every initializer error is
`E_UNSUPPORTED_FEATURE`. It retains the seven producer diagnostics and
withholds HIR, but downstream consumers can be type-checked without cascades.
Any recursive, arity, type, or other semantic error still stops binding; the
existing recursive-tuple fixture remains a four-diagnostic negative control
and does not re-enter static tuple queries.

This recovery matches Pine's tuple binding model without implementing the
rejected producer. TradingView's
[v4 type-system documentation](https://www.tradingview.com/pine-script-docs/v4/language/type-system/)
describes tuples as multi-result function returns, while the current
[variable-declaration documentation](https://www.tradingview.com/pine-script-docs/language/variable-declarations/)
and
[user-defined-function documentation](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)
describe tuple declarations binding each returned result. Those rules justify
retaining already-known element types; they do not make unsupported
`security` execution available.

Four already-failing modern controls change only in diagnostics. Across those
items, 60 unknown-symbol, 27 operator-type, 21 assignment-type, and ten
call-argument-type cascades disappear; three additional producer-level
unsupported records become reachable, for a net reduction of 115 diagnostics.
All 60 controls retain their stage outcomes, and their combined diagnostic
count falls from 6,385 to 6,270. The complete-item hash changes intentionally
from
`94cb795414ae56f0a47e36afa738ad4bcf95c242c82a72f86642b392e5bcee3c`
to
`8cebc1e4f0df98b20d7a10d4337a2033ab8712b85e63e7a4067bed83d0b13667`.
The bounded release registry remains at 29 executable profiles, and translator
revision 23 prevents semantic-cache reuse across the recovery change.

### Twentieth spaced-equals version-annotation slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-spaced-version-annotation-slice-20`. The final reports
are `report-version-20-final-a.json` and
`report-version-20-final-b.json`; their matching SHA-256 is
`daf09bcc9f8f422977a5293ef5759804ae8def343be5b46c07877edb50d959cc`.

| Corpus metric | Nineteenth tuple-recovery slice | Twentieth version-annotation slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 33/44 | 34/44 | +1 |
| Historical-run passes | 28/44 | 29/44 | +1 |
| Eligible diagnostic records | 59 | 44 | -15 |
| Known-unsupported records | 42 | 42 | 0 |
| Legacy-version-feature records | 14 | 0 | -14 |
| Named-call-shape records | 1 | 0 | -1 |
| Eligible version mismatches | 1 | 0 | -1 |

Exactly one v4 indicator changes. Its intended version annotation uses
horizontal whitespace before and after the equals sign. The previous lexer
treated that line as an ordinary comment, selected implicit v1, rejected a
v4-only declaration argument, and classified qualified color, size, and alert
spellings as later-version features. Recognizing the intended explicit v4
dialect removes all fifteen diagnostics. The indicator then analyzes, lowers,
and executes historically without another compatibility change.

TradingView's
[v4 version documentation](https://www.tradingview.com/pine-script-docs/v4/language/versions/)
and current
[script-structure documentation](https://www.tradingview.com/pine-script-docs/language/script-structure/)
show `//@version=N` as the standard compiler annotation. This slice adds only a
source-compatibility spelling: optional horizontal whitespace around `=`.
The `//@version` prefix remains exact, malformed values still use the existing
version diagnostics, duplicate and misplaced spaced annotations keep the
existing focused errors, and `// @version=6` remains an ordinary comment.

All 60 modern controls remain complete-item identical at
`8cebc1e4f0df98b20d7a10d4337a2033ab8712b85e63e7a4067bed83d0b13667`;
no modern diagnostic or stage changes. The paired public v4/v6 runtime fixture
passes historical equivalence plus release batch, incremental, realtime,
rollback, confirmation, and resource gates. The bounded release registry
reaches 30 executable profiles, and translator revision 24 prevents
semantic-cache reuse across version detection.

### Twenty-first nested-if branch-result slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-nested-if-return-slice-21`. The final reports are
`report-branch-return-21-final-a.json` and
`report-branch-return-21-final-b.json`; their matching SHA-256 is
`82a7cecb02e0c9c31743d19cdc9758ec134c9913c03500e1dd8058afb9073ada`.

| Corpus metric | Twentieth version-annotation slice | Twenty-first nested-if slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 34/44 | 35/44 | +1 |
| Historical-run passes | 29/44 | 29/44 | 0 |
| Eligible diagnostic records | 44 | 42 | -2 |
| Known-unsupported records | 42 | 42 | 0 |
| Branch-return records | 2 | 0 | -2 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

Both removed diagnostics belong to one v4 indicator and the same repeated
shape. An enclosing value-producing `if` has a branch whose final statement is
a complete nested `if`/`else-if`/`else`; every nested leaf is itself a value.
The analyzer previously treated the final statement form as non-value even
though it already supported final loop statements and function-return
conditionals. The corrected path recursively analyzes, statically types, and
lowers the nested conditional. A public negative fixture ends one nested leaf
with a reassignment and still receives `E_BRANCH_RETURN`, so this is not a
blanket statement-to-value conversion.

TradingView's current
[conditional-structure documentation](https://www.tradingview.com/pine-script-docs/language/conditional-structures/)
states that conditional structures can be embedded and that a value-producing
local block returns the value evaluated at its end. The public v4/v6 fixture
exercises all outer and nested selections and produces identical historical
outputs. It also passes the release batch, incremental, realtime, rollback,
confirmation, and resource gates.

The newly admitted corpus item lowers without diagnostics but fails historical
execution at an independent runtime/host boundary. It therefore increases only
analysis/lowering, not the execution count. The 60 modern controls remain
complete-item identical at
`8cebc1e4f0df98b20d7a10d4337a2033ab8712b85e63e7a4067bed83d0b13667`.
The bounded release registry reaches 31 executable profiles, and translator
revision 25 prevents semantic-cache reuse across the new branch-result
classification.

### Twenty-second named-input-default runtime slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-input-default-slice-22`. The final reports are
`report-input-default-22-final-a.json` and
`report-input-default-22-final-b.json`; their matching SHA-256 is
`4765800bd8aee74e178d97e3673fb028b98ca29658b37f1bf71b0f77bd00f756`.

| Corpus metric | Twenty-first nested-if slice | Twenty-second input-runtime slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 35/44 | 35/44 | 0 |
| Historical-run passes | 29/44 | 30/44 | +1 |
| Eligible diagnostic records | 42 | 42 | 0 |
| Known-unsupported records | 42 | 42 | 0 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

Exactly one already-lowered v4 indicator changes stage. Its generic
`input(...)` call uses only keyword arguments and places `title` before
`defval`. Legacy analysis correctly removes `type=input.string`, preserves
canonical argument names and metadata, and produces executable HIR. Historical
runtime previously ignored those names and evaluated raw argument zero, so the
title became the input value and a downstream timezone consumer failed.
Runtime input evaluation now selects `defval` through the shared argument
resolver, using a named argument or positional fallback. Input overrides still
win first, positional calls still use argument zero, and metadata arguments
are not evaluated as defaults.

TradingView's
[v4 script-input documentation](https://www.tradingview.com/pine-script-docs/v4/annotations/script-inputs/)
uses `title`, `type`, and `defval` as named parameters and includes examples
where `title` appears first. The current
[built-in function documentation](https://www.tradingview.com/pine-script-docs/language/built-ins/)
also states that keyword arguments may change position because parameter names,
not source order, identify them. The public v4/v6 pair preserves matching input
metadata and proves the default through an IANA-timezone calculation; it passes
historical, incremental, realtime, rollback, confirmation, and resource gates.

All 60 modern controls remain complete-item identical at
`8cebc1e4f0df98b20d7a10d4337a2033ab8712b85e63e7a4067bed83d0b13667`.
The bounded release registry reaches 32 executable profiles. Translator
revision remains 25 because this correction changes runtime interpretation of
already-canonical HIR and does not change parsing, analysis, or lowering.

### Twenty-third complete Pine v1-v4 integer-division slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-integer-division-slice-23`. The final reports are
`report-integer-division-23-final-a.json` and
`report-integer-division-23-final-b.json`; their matching SHA-256 is
`6a78f00eb5f27b89c48f4525d3aedb25e07fff8830ca521f054aad5b9521b10d`.

| Corpus metric | Twenty-second input-runtime slice | Twenty-third integer-division slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 35/44 | 36/44 | +1 |
| Historical-run passes | 30/44 | 31/44 | +1 |
| Eligible diagnostic records | 42 | 40 | -2 |
| Known-unsupported records | 42 | 40 | -2 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

Exactly two v4 indicators change. Both use an integer input to derive a
half-length alias and then use that alias as a history offset. Pine v1-v4 now
types every `int / int` expression as an integer and lowers it through an
explicit canonical `int(...)` conversion, so both dynamic-history-offset
diagnostics disappear. One indicator then analyzes, lowers, and runs
historically; the other keeps an independent declaration-level
`study(resolution=...)` boundary and remains fail-closed. The v4 profile rises
from 11/20 to 12/20 analysis/lowering and from 10/20 to 11/20 historical
execution.

TradingView's
[v4 operator documentation](https://www.tradingview.com/pine-script-docs/v4/language/operators/)
states that arithmetic over two integer operands produces an integer result.
The current
[v6 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/)
also records the later version boundary: v5 kept integer division for two
`const int` values but preserved fractions for input, simple, or series
integers, while v6 permits fractional results for all integer qualifiers. This
slice corrects only the corpus-backed Pine v1-v4 rule; the separate v5
constant-division case is not widened here.

All five lowered-but-not-executing indicators still fail for the same
`missing_provider_data` reason. There is no residual runtime/host failure in
that group, so analysis must not be widened to disguise absent request streams.
All 60 modern controls remain complete-item identical at
`8cebc1e4f0df98b20d7a10d4337a2033ab8712b85e63e7a4067bed83d0b13667`.
The bounded release registry remains at 32 executable profiles, and translator
revision 26 prevents semantic-cache reuse across the corrected type and
lowering contract.

### Twenty-fourth Pine v5 const-integer-division slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-v5-const-division-slice-24`. The final reports are
`report-v5-division-24-final-a.json` and
`report-v5-division-24-final-b.json`; their matching SHA-256 is
`27b36c02b74fc11c95cf12066bb7f3d429875a67570ee4df222918f3794e27e4`.

| Corpus metric | Twenty-third integer-division slice | Twenty-fourth v5 const slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 36/44 | 36/44 | 0 |
| Historical-run passes | 31/44 | 31/44 | 0 |
| Eligible diagnostic records | 40 | 40 | 0 |
| Known-unsupported records | 40 | 40 | 0 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

This slice closes the separate version boundary identified after the
Pine v1-v4 correction. In Pine v5, division produces an integer only when both
operands are `const int`; if either integer is input, simple, or series
qualified, the result remains fractional. Analysis and UDF type queries share
one qualifier-aware predicate, known constant history offsets fold through the
same boundary, and lowering emits an explicit canonical `int(...)` conversion
only for the truncating form. The v6 side of the public pair keeps fractional
division and uses explicit casts only where the v5 source truncates.

TradingView's
[v6 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/)
documents this exact v5 distinction and the v6 change to fractional division
for every integer qualifier. The public fixtures separately verify const
literal, const alias, input, series, UDF, and history-offset paths, including a
negative input-derived history offset that remains fail-closed.

All 44 eligible legacy item objects and all 60 modern-control item objects are
complete-item identical to the twenty-third slice. The change therefore makes
no private-corpus progress claim and introduces no control drift. All five
lowered historical failures remain missing-provider cases. The bounded legacy
release registry remains at 32 profiles because the new pair exercises a
modern v5/v6 language boundary, and translator revision 27 prevents
semantic-cache reuse across the qualifier-dependent result type.

### Twenty-fifth chart-inherited study-resolution slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-study-empty-resolution-slice-25`. The final reports
are `report-study-resolution-25-final-a.json` and
`report-study-resolution-25-final-b.json`; their matching SHA-256 is
`b5e73391c997ce81a8a4ea1025f7ba8a91e571878929a8bb3b35127ef8e8c2a2`.

| Corpus metric | Twenty-fourth v5 const slice | Twenty-fifth empty-resolution slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 36/44 | 39/44 | +3 |
| Historical-run passes | 31/44 | 34/44 | +3 |
| Eligible diagnostic records | 40 | 37 | -3 |
| Known-unsupported records | 40 | 37 | -3 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

All three affected indicators use the exact Pine v4
`study(resolution="")` declaration form. An empty declaration timeframe means
the script inherits the chart timeframe, so this slice does not synthesize a
request and does not claim arbitrary whole-program MTF execution. The binder
drops the empty selector and an omitted or literal-bool `resolution_gaps`
before canonical `indicator` lowering, while per-source-argument rewrites keep
later named metadata in the correct role. A public v4/v6 fixture proves the
host chart symbol and timeframe are visible inside the script and verifies
batch, incremental, and realtime historical handoff parity.

TradingView's [Pine v4 release notes](https://www.tradingview.com/pine-script-docs/v4/release-notes/)
record the addition of `resolution` to `study`, and the current
[declaration statement documentation](https://www.tradingview.com/pine-script-docs/language/declaration-statements/)
documents that a declaration timeframe selects the script's main execution
timeframe and that gap policy only controls mapping between execution and chart
contexts. The empty-string case leaves those contexts identical. Non-empty or
dynamic values still produce one `study.resolution` unsupported diagnostic
until a program-level coordinator owns provider identity, whole-script state,
output alignment, gaps, and realtime confirmation.

Exactly three legacy item objects change: each moves from failed/not-run to
passed/passed at analyze and historical-run stages and loses its sole
diagnostic. All 60 v5/v6 control item objects are complete-item identical. The
five lowered historical failures remain `missing_provider_data`; none is a
compiler or runtime semantic regression. The bounded release registry reaches
33 executable profiles, and translator revision 28 prevents stale semantic
cache reuse.

### Twenty-sixth Pine v4 UDF reference-side-effect slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-v4-udf-reference-side-effects-slice-26`. The final
reports are `report-udf-reference-effects-26-final-a.json` and
`report-udf-reference-effects-26-final-b.json`; their matching SHA-256 is
`a0bd2c904a3c1692ce84154cc60565526f46708569a064a443e781fbc3cac8de`.

| Corpus metric | Twenty-fifth empty-resolution slice | Twenty-sixth UDF reference slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 39/44 | 42/44 | +3 |
| Historical-run passes | 34/44 | 37/44 | +3 |
| Eligible diagnostic records | 37 | 8 | -29 |
| Known-unsupported records | 37 | 8 | -29 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

The remaining function-side-effect cluster was structurally concentrated, not
37 independent missing features. Pine v4 UDF bodies now admit only the exact
namespace-call subset observed in the corpus: `array.set`, `array.pop`,
`array.unshift`, `array.clear`, `label.new`, `label.delete`, `line.new`, and
`line.delete`. The existing inliner evaluates each reached body statement once
in source order, array parameters and globals carry the shared runtime id, and
drawing calls write the ordinary rollback-aware object stores. A v4 fixture and
an explicitly expanded v6 rewrite verify array values, drawing create/delete
snapshots, void final loop/conditional calls, historical batch execution,
incremental append, and realtime historical handoff.

TradingView's [Pine v4 array documentation](https://www.tradingview.com/pine-script-docs/v4/essential/arrays/)
states that globally assigned arrays can be modified from a function's local
scope. Its [v4 function execution documentation](https://www.tradingview.com/pine-script-docs/v4/language/functions-and-annotations/)
uses `label.new` as an example of a call whose local conditional execution must
not be forced, while the current [visuals overview](https://www.tradingview.com/pine-script-docs/visuals/overview/)
explicitly permits drawing calls in functions and other local scopes. Broader
array mutation, drawing setters/copies, v4 method syntax, global-only outputs,
alerts, strategy calls, global scalar reassignment, and side-effecting UDF
arguments remain fail-closed behind public negative fixtures.

Four legacy item objects change. Three move from failed/not-run to
passed/passed at analyze and historical-run stages. The fourth loses all
drawing-side-effect records but stays failed/not-run on its single `timenow`
diagnostic, so no execution claim is made for it. The only other remaining
analysis failure contains seven legacy-`security` records. All 60 v5/v6 control
item objects are complete-item identical. All five lowered historical failures
remain exactly `missing_provider_data`; none is a compiler/runtime semantic
regression. The release registry reaches 34 executable profiles, and translator
revision 29 prevents stale semantic-cache reuse.

### Twenty-seventh Pine v4 UDF-local security dependency slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-v4-udf-local-security-slice-27`. The final reports are
`report-security-udf-locals-27-final-a.json` and
`report-security-udf-locals-27-final-b.json`; their matching SHA-256 is
`74fa6a6230d1553291b540f5d0e88c4952064d39af0c7a11bb2f09f12cf23403`.

| Corpus metric | Twenty-sixth UDF reference slice | Twenty-seventh UDF-local security slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 42/44 | 43/44 | +1 |
| Historical-run passes | 37/44 | 37/44 | 0 |
| Historical missing-provider failures | 5 | 6 | +1 |
| Eligible diagnostic records | 8 | 1 | -7 |
| Known-unsupported records | 8 | 1 | -7 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

The seven remaining legacy-`security` records were one lexical dependency
family. Pine v4 `security` calls written directly in a UDF body may now use
scalar parameters and normal immutable scalar locals. The HIR keeps every
inlined declaration's unique symbol and initializer. Before provider execution,
the runtime builds an initializer index for the admitted program, captures
const/input/simple nodes from the reached outer call, and recomputes series
nodes in the isolated requested runtime. A prior legacy request may be one of
those nodes only in the three-positional-argument form when symbol, timeframe,
gaps, and lookahead match the enclosing request exactly. It consequently
becomes a same-context dependency inside the requested child runtime.

TradingView's [Pine v4 security documentation](https://www.tradingview.com/pine-script-docs/v4/essential/context-switching-the-security-function/)
defines the third argument as an expression evaluated on the selected series
and includes a UDF whose body calls `security`. Its
[Pine v4 function documentation](https://www.tradingview.com/pine-script-docs/v4/language/declaring-functions/)
defines function arguments and declarations as members of that function's
local scope. The implementation preserves both rules without admitting
different-selector nesting, control-flow-local requests, mutable or persistent
locals, recursion, side effects, lower-timeframe requests, or modern provider
local aliases.

Exactly one legacy item object changes. It moves from failed/not-run to
passed/passed at analyze/lower and then fails historical execution with
`missing_provider_data`; no execution success is claimed without its requested
streams. The other five historical failures retain the same error kind, so all
six lowered failures are now provider-input setup work. All 60 v5/v6 control
item objects are complete-item identical. The only remaining analysis failure
is the single `timenow` record. The release registry reaches 35 executable
profiles, and translator revision 30 prevents stale semantic-cache reuse.

### Twenty-eighth deterministic execution-clock slice

The unchanged manifest was measured twice with
`buildRevision=corpus-r2-timenow-execution-clock-slice-28`. The final reports
are `report-timenow-clock-28-final-a.json` and
`report-timenow-clock-28-final-b.json`; their matching SHA-256 is
`742fa06684acf5882e51a14899b9d879fa32d8758d3c765684db9a2dbb0a5e52`.

| Corpus metric | Twenty-seventh UDF-local security slice | Twenty-eighth execution-clock slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 43/44 | 44/44 | +1 |
| Historical-run passes | 37/44 | 37/44 | 0 |
| Historical missing-provider failures | 6 | 6 | 0 |
| Historical missing-execution-time failures | 0 | 1 | +1 |
| Eligible diagnostic records | 1 | 0 | -1 |
| Known-unsupported records | 1 | 0 | -1 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

`timenow` now lowers as its official `series int` value and reads a
host-provided UNIX millisecond timestamp for the current script execution.
Historical batch helpers require an exact per-bar timestamp slice when it is
supplied; incremental and realtime runtimes accept one timestamp with each
execution. CLI exposes `--execution-times`, Python exposes `execution_times`,
and WASM reserves `$executionTimes` in request-host JSON. A reached read without
a timestamp and a supplied batch whose counts differ both fail closed. Neither
the bar's `time` nor the process wall clock is used as a substitute.

TradingView's [time documentation](https://www.tradingview.com/pine-script-docs/concepts/time/#timenow)
defines `timenow` in UNIX milliseconds and distinguishes its script-execution
updates from bar opening times. Its
[execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/)
describes realtime rollback and repeated execution on an open bar. The runtime
therefore commits historical clock values as series history but rolls back and
recomputes a forming bar with each newly supplied execution timestamp.

Exactly one legacy item object changes. It now passes analysis and lowering,
then fails historical execution with `missing_execution_time` because the
private manifest has no execution-clock input; this is an honest host-input
failure, not an execution-success claim. The six existing
`missing_provider_data` failures are unchanged. Two already-failing v6 control
objects each lose two obsolete `timenow` diagnostics without changing any
stage status; the other 58 modern controls remain complete-item identical.
Eligible diagnostics and failure clusters both reach zero. The release
registry reaches 36 executable profiles, and translator revision 31 prevents
stale semantic-cache reuse.

### Twenty-ninth execution-clock corpus-input slice

The same 104-source intake was measured twice with corpus report schema 3 and
`buildRevision=corpus-r2-execution-clock-input-slice-29`. The final reports are
`report-execution-clock-input-29-final-a.json` and
`report-execution-clock-input-29-final-b.json`; their matching SHA-256 is
`d7e4024eae1a7cfce88b086e4d7b4ea7880bdfe682345a9151cb96a2da1d80c6`.

| Corpus metric | Twenty-eighth execution-clock slice | Twenty-ninth clock-input slice | Change |
| --- | ---: | ---: | ---: |
| Parse passes | 44/44 | 44/44 | 0 |
| Analyze/lower passes | 44/44 | 44/44 | 0 |
| Historical-run passes | 37/44 | 38/44 | +1 |
| Historical missing-provider failures | 6 | 6 | 0 |
| Historical missing-execution-time failures | 1 | 0 | -1 |
| Supplied execution-time inputs | 0 | 1 | +1 |
| Eligible diagnostic records | 0 | 0 | 0 |
| Known-unsupported records | 0 | 0 | 0 |
| Unknown-symbol records | 0 | 0 | 0 |
| Operator-type records | 0 | 0 | 0 |

Corpus schema 3 adds the optional `execution_times_path` manifest column and
the corresponding `executionTimes` availability count. A non-empty value must
resolve to a file before runtime starts and is forwarded to CLI
`--execution-times`. Reports record only `passed`, `not_supplied`, or
`missing_input`; the privacy contract continues to omit source paths,
execution-time paths, timestamp values, and source text. Invalid timestamp text
and batch/bar count mismatches remain explicit runtime/host failures instead of
being reclassified as compatibility diagnostics.

The private manifest supplies ten deterministic per-execution timestamps for
the sole v4 `timenow` item. That item moves from a
`missing_execution_time` historical failure to a successful run. The other six
historical failures remain exactly `missing_provider_data`, and no diagnostic
changes. Because schema 3 adds `executionTimes` availability to every report
item, complete-object equality with schema 2 is intentionally not meaningful;
after isolating that structural field, exactly one legacy stage map changes.
All 60 modern-control stage maps and diagnostic arrays are unchanged. This
slice changes measurement plumbing and private host input only, so translator
revision 31 and the 36-row release registry remain unchanged.

## Next Selection

The next implementation slice should still be measured over this unchanged
manifest and should not add strategy behavior. The ranked order is now:

1. treat all six lowered-but-not-executing scripts as request-data setup work;
   add only authorized symbol/timeframe streams matching each call and do not
   reuse chart bars or fabricate market data to raise the historical-run count;
2. add reference-output comparison only when authorized or independently
   generated oracles become available; the current manifest supplies none, so
   successful execution alone is not an external value-parity claim;
3. preserve the now-zero eligible diagnostic, legacy-security, branch-return,
   dynamic-history-offset, unknown-symbol, and operator-type clusters while
   resolving any next producer; do not weaken structured block layout, graph
   side-effect barriers, or modern call typing.

The v4 selection checkpoint originally required 30 total samples; this private
intake contributes 20, while the public seed contains 12. Counts must not be
blindly summed until duplicate content across the two manifests is checked.
Stable evidence still requires at least 50 eligible scripts per profile and the
separate incremental, realtime, provider, resource, cache, host-parity, and
release audits.
