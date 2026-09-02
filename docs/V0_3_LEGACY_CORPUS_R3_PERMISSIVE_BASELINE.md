# v0.3 Legacy Corpus R3 Permissive Baseline

Status: deterministic local evidence baseline; source text and reports remain
ignored local artifacts.

## Outcome

The Pine v4 corpus now contains 51 pairwise-deduplicated, standalone eligible
indicators:

- 12 committed public seed indicators;
- 20 private user-authorized R2 indicators;
- 19 new permissively licensed R3 indicators.

This clears the provisional 50-script stable-profile floor. It does not by
itself promote the Pine v4 profile from preview: external reference-output and
profile-promotion gates remain independent. Batch, incremental,
realtime-history, forming rollback/confirmation, provider-cache, retained-value,
release-profile, and claimed-host parity are measured below.

## Permissive Intake

The intake searched public GitHub repositories whose repository-level license
was reported as MIT, pinned every checkout to a full commit, then inspected
candidate files for a more specific file-level license. Source text was copied
only into ignored `.local` intake directories.

| Upstream | Revision | Accepted | Intake disposition |
| --- | --- | ---: | --- |
| `agsosa/tv-pinescripts` | `07b46ecbc5115db5814ddbb53983a361a1d96d5c` | 1 | accepted |
| `blackcat1402/pinescript-1` | `127be076bfc14da9ea71e048bcf2a2902920f5d5` | 6 | accepted |
| `ClimberMel/Pinescript` | `fff34bb12ee228439b712c1f84a40afae9d63e92` | 1 | accepted |
| `girishsalaskar/pinescripts` | `eeb7967692df75a66ca0f5e4cd57f4467e094372` | 5 | accepted; two MPL-2.0 and one CC-BY-NC-SA-4.0 files excluded |
| `hirawatt/pineScripts` | `254d17ebb449d8ea3048f2cff60526375f2b2dff` | 0 | three file-level MPL-2.0 files excluded |
| `magic8bot/magic8bot` | `d292ef8012fcee197c099e697f72a8ee88250101` | 0 | one incomplete source excluded |
| `pacificbay/sar` | `7cd1ab8217d710abecc3d6059852f9b92c4e45e9` | 0 | three unexpanded preprocessor inputs excluded |
| `shyrwinsia/pine-scripts` | `4b0c9a15210da19c0274d6c66df237cc3983c4e4` | 3 | accepted |
| `sibvic/pinescript-templates` | `18b05daa2accd38ccaa08bef53442a4da058b7c6` | 0 | one customization template excluded |
| `andreperez/Stochastic-Oscilators-Collection` | `4dd95e2a0e2c3279726e978f103b030e6c2ed301` | 1 | accepted |
| `rKv4dr4t/SuperGuppy_SuperTrend_Screener` | `b804490d75b908279a4d1df3f72523616036f832` | 1 | accepted |
| `samgozman/vix-fix-double-pleasure` | `c1e448b4f9a572d522e6824f8bf2873c42bdb2fc` | 1 | accepted |

The initial scan found 30 explicit `//@version=4` study declarations. Six
files were excluded because their file-level license was outside this
permissive MIT intake. Five more were excluded as non-standalone inputs:

- `gh-v4-r3-magic8bot-c9cbd4173b7b3925` references four color variables that
  are never declared in the source;
- `gh-v4-r3-sar-294ed72c872bd864`,
  `gh-v4-r3-sar-6efda71d8e4f3d53`, and
  `gh-v4-r3-sar-ddd39c2ca473e8a8` contain bare repository-specific `import`
  placeholders which must be expanded before they are Pine programs;
- `gh-v4-r3-sibvic-bf8a58e7b9ced808` is a customization template whose four
  `TODO` identifiers must be replaced with user-owned signal expressions.

These exclusions are source-validity decisions, not compatibility-result
filtering. The merger's repeatable `--exclude-id` option fails on stale or
unknown ids so the decision remains explicit in the command. Difficult but
standalone sources stay in the denominator.

## Deduplication

The 19 accepted R3 sources are internally unique. Separate version-aware audits
found no exact-byte, normalized-text, or comment/trivia-free token match:

| Comparison | Baseline | R3 contribution | Cross matches |
| --- | ---: | ---: | ---: |
| committed public v4 seed vs R3 | 12 | 19 | 0 |
| private R2 v4 selection vs R3 | 20 | 19 | 0 |

The earlier R2 audit already proved that the public 12 and R2 20 do not match.
The three pairwise results therefore establish 51 unique v4 indicators.

## Deterministic Baselines

The merged manifest uses an explicit root for every input manifest because the
public seed is repository-root-relative while private intake manifests are
manifest-directory-relative. Its merger also rebases nested request CSV paths
into deterministic sidecar manifests. This corrected an intermediate audit
artifact that had falsely classified three existing provider-backed rows as
`missing_input`; no provider file was actually absent.

Corpus report/tool schema 5 adds a fourth execution stage. It replays the final
bar as a deliberately mutated forming update, replaces it with the original
forming value, then confirms the original bar. Output and retained resource
counts must equal batch execution.

| Stage | Release/forming audit |
| --- | ---: |
| Source read | 51/51 |
| Parse | 51/51 |
| Analyze/lower | 47/51 |
| Historical run | 47/51 |
| Incremental run | 47/51 |
| Realtime-history run | 47/51 |
| Realtime-forming rollback/confirm | 47/51 |
| Resource audit | 47/51 |

The latest report uses
`buildRevision=corpus-r3-v4-udf-line-setters-2`. Its two deterministic runs are
byte-identical with SHA-256
`4bd1d568ef87793b3a9fe40649585a0873e9fea755407b2f8c79e8281cd4dc83`.

The release/forming stable-baseline rates are:

- 100% parse, above the 95% threshold;
- 92.16% analyze/lower, above the 85% threshold;
- 92.16% historical execution, above the 80% threshold;
- zero unknown failure clusters affecting at least 2% of eligible scripts.

All 47 attempted batch, incremental, realtime-history, realtime-forming, and
resource executions pass. Missing-input counts are zero. Four provider-backed
rows populate request-cache evidence; the audit peaks at 152,220 retained
values, 2,880 series depth, 15 cache entries across four contexts, and 45,353
cached requested values, below the one-million retained-value ceiling. No
source-revision-paired reference output is supplied for the new R3 sources.

## First Corpus-Ranked Slice

One R3 script used the historical Pine v4 global `hma(source, length)` call.
The canonical `ta.hma` analyzer and runtime were already supported, so the
legacy catalog now applies an exact v4-only alias and increments the translator
revision to 32. Paired legacy/canonical fixtures prove HIR and historical value
equivalence; Pine v3, v5, and v6 remain negative controls. Its public runtime
golden is required across CLI, Python, and WASM, bringing the host-parity gate
to 435 required runtime snapshots. This moves exactly one R3 source through
analysis, lowering, all four execution modes, and the resource audit.

## Second Corpus-Ranked Slice

The next R3 source needed only `line.set_x2()` and `line.set_extend()` inside a
Pine v4 UDF. The legacy frontend now admits exactly those two drawing mutations
through its existing v4-only reference-side-effect inliner and increments the
translator revision to 33. It does not broaden Pine v3, v5, or v6 UDF side
effects, and other setters such as `line.set_x1()` remain explicit negative
controls.

Paired legacy/canonical fixtures prove historical value and drawing-state
equivalence. Runtime tests also cover incremental execution and mutated,
replaced, then confirmed forming-bar rollback. The fixture is the 37th release
profile and its public golden is required across CLI, Python, and WASM, raising
the host-parity gate to 436 runtime snapshots.

The selected R3 source also reads `timenow` and requests `TEST:D`, `TEST:W`,
`TEST:M`, and `TEST:12M`. Its ignored local measurement manifest therefore
supplies ten deterministic execution timestamps and four deterministic TEST
streams. This is internal execution and cache evidence, not a
source-revision-paired TradingView output claim. In every execution mode the
source retains 124 values and populates four request contexts, 12 callsite
entries, and 120 cached values.

## Next Selection

The stable-size and internal release/forming milestones are closed. The four
remaining analysis blockers are now separated rather than treated as one
feature request:

1. `gh-v4-r3-agsosa-08c89dfb88715414` can now request `barstate.*` flags and
   proven plot-style enums. Remaining execution still needs host `D`/`W`/`M`/
   `12M` streams for its lookahead UDF requests; that is host data, not a
   kernel whitelist gap.
2. `gh-v4-r3-climbermel-9d741d561f56d555` can now request `time("D")` /
   `time_close()` / named `time()` graphs and series `array.insert` indexes.
   Remaining execution still needs a host `timenow` clock and `60` minute
   streams.
3. `gh-v4-r3-superguppy-9307f2392b285cc9` makes 40 same-timeframe calls over
   20 configurable symbols. The two requested expressions include a large
   global EMA graph and a stateful pivot/ATR UDF, so it needs both a broader
   requested-expression contract and 20 symbol streams.
4. `gh-v4-r3-girish-419cb5a690ff5269` compares boolean signals with integer
   literals. Any v4 bool/numeric compatibility rule needs source-revision-paired
   output evidence before implementation because it would alter expression
   typing globally.

The bounded UDF line-setter slice is closed. The v4 bool/numeric comparison is
the smallest remaining local shape, but it should stay fail-closed until a
source-revision-paired TradingView export establishes the intended output. The
three legacy-security scripts should likewise remain fail-closed until their
requested-expression shapes are minimized; provider collection should then be
done only for the selected shape. A stable promotion still needs paired
TradingView output for a representative R3 subset.
