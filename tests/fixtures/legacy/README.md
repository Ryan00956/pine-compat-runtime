# Legacy Indicator Corpus Seeds

These sources are original minimal indicators written for the legacy
compatibility corpus. They are not copied from third-party or protected
scripts.

`corpus.tsv` is the deterministic Phase 0 manifest consumed by
`scripts/analyze_legacy_corpus.py`. The public seed corpus intentionally covers
common legacy failure families rather than claiming to represent the user's
private indicator library. Authorized whole-script samples can be added to a
private manifest without changing the analyzer.

`legacy_strategy_excluded` rows prove that strategy sources stay outside the
indicator denominator and are not sent through the corpus compiler path. The
analyzer also derives strategy mode from source so an incorrect manifest scope
cannot bypass that exclusion.

Five `invalid_control` rows exercise lexer, parser, unknown-name, call-shape,
and type failures. `control_modern_v6` must still analyze and run successfully.
Together these controls distinguish legacy compatibility failures from a
broken analyzer or CLI.

After building the CLI, generate the deterministic report with:

```text
cargo build -p pine-cli
python3 scripts/analyze_legacy_corpus.py --output /tmp/legacy-corpus.json
```

Corpus report schema 4 retains schema 3's privacy-preserving `executionTimes`
input availability and adds three-mode resource and provider-cache evidence.
The optional manifest
`execution_times_path` is forwarded to CLI `--execution-times`; reports expose
only whether the file was supplied, passed preflight, or was missing, never its
path or timestamp values. `eligibleSuccessRate` always uses every eligible
script in the profile as the denominator; later-stage `not_run` and
`missing_input` rows therefore cannot inflate the promotion measurements.
Failure clusters expose both diagnostic occurrence counts and affected-script
shares. `requiresDisposition` becomes true at the provisional 2% per-profile
threshold, but only qualifying unknown clusters block the automated baseline
assessment.

Passing `stableBaseline.thresholdsMet` is not a stable release claim. The
release registry must still prove incremental, realtime, provider, resource,
cache, and host-parity behavior.

Before combining a private v4 selection with the 12 committed public v4 seeds,
run the version-aware dedup audit. Its JSON contains one-way fingerprints and
opaque private ids, not source text, paths, or titles:

```text
python3 scripts/audit_legacy_corpus_dedup.py \
  --candidate-manifest /absolute/path/to/corpus-r2.tsv \
  --build-revision corpus-r2-dedup \
  --output /absolute/path/to/corpus-r2-dedup.json
```

Exact, normalized-text, and token-equivalent matches are reported separately.
The token fingerprint removes comments and trivia but remains version-bound;
it does not claim that differently written programs are semantically equal.

For a private or user-authorized R2 corpus, keep sources outside the repository
and point the analyzer at an external root:

```text
python3 scripts/analyze_legacy_corpus.py \
  --manifest /absolute/path/to/corpus-r2.tsv \
  --root /absolute/path/to/corpus-root \
  --build-revision corpus-r2-pre-code \
  --output /absolute/path/to/corpus-r2-pre-code.json
```

The committed, redacted Phase 0 result is recorded in
`docs/LEGACY_INDICATOR_PHASE0_BASELINE.md`.

Versioned `v*/syntax`, `v*/sema`, `v*/runtime`, and `v*/unsupported`
directories own paired compatibility fixtures added after the baseline. Phase 3
adds v4 declaration and exact-alias pairs under `v4/sema` and `v4/runtime`;
these files are original project fixtures and are also referenced by
`conformance.tsv`.

The chart-context declaration slice adds the paired
`v4/runtime/study_empty_resolution_*` fixtures. The legacy source proves that
the exact `study(resolution="")` form inherits the host chart symbol and
timeframe without requesting provider data; non-empty and dynamic resolution
forms remain unsupported until a whole-program execution coordinator exists.

Phase 4 adds the paired `v4/runtime/inputs_*` fixtures for all supported Pine v4
input type constants, metadata, callsites, default values, and scalar host
overrides. `v4/sema/input_constant_alias.pine` owns the local const-alias case.

Phase 5 adds paired `v4/runtime/outputs_*` fixtures for all ten initial output
families, primitive plot/hline styles, transparency defaults and alpha
precedence, visual metadata, normalized colors, and historical execution.
`v4/unsupported/output_arguments.pine` keeps later-only output arguments behind
an analysis-time diagnostic.

The corpus-ranked pre-v4 output slice adds paired `v1/runtime/outputs_*`
fixtures for the same ten output families using the documented v3 parameter
tables, including historical transparency defaults and context-specific style
ordinals. `v1/unsupported/output_arguments.pine` keeps later `display` and
`fillgaps` roles outside the v1-v3 surface.

The next corpus-ranked request slice adds
`v1/runtime/security_aliases_legacy.pine` for dynamic input resolution,
immutable requested-series alias expansion, const/input capture, provider
alignment, and v1 historical lookahead. The release profile verifies batch,
incremental, realtime, provider, and resource behavior;
`v1/unsupported/security_mutable_alias.pine` preserves the historical mutable
variable rejection. The follow-up pure-function slice adds
`v4/runtime/security_pure_udf_legacy.pine` for nested, immutable UDF
recomputation and legacy `input.source` default-source selection in the
requested context;
`v4/unsupported/security_mutable_udf.pine` keeps reassigned UDF-local state
outside that bounded subset.

The next Pine v4 request fixture pair,
`v4/runtime/security_udf_local_dependencies_legacy.pine` and its explicit
canonical rewrite, covers a `security` call placed directly in a UDF body. Its
scalar parameters and normal immutable scalar locals are dependency nodes:
series nodes recompute in the requested context, while const/input/simple nodes
are captured. An earlier three-positional-argument legacy request may feed a
later request only when symbol, timeframe, gaps, and lookahead are identical.
`v4/unsupported/security_udf_local_dependency_mismatch.pine` keeps a different
requested symbol fail-closed, while
`v4/unsupported/security_udf_control_flow_local.pine` covers a request nested
under a local `if`; reassignment, persistence, recursion, and the modern
provider-local boundary are unchanged.

The following call-shape slice adds
`v4/runtime/contextual_integer_division_legacy.pine` and its explicit-v6
canonical rewrite. They prove that every Pine v1-v4 `int / int` expression
produces an integer by discarding its fractional remainder, including named
aliases, history offsets, integer-compatible calls, and untyped UDF arguments.
Float operands are unaffected. The distinct v5 qualifier-dependent rule and v6
fractional rule remain outside this Pine v1-v4 compatibility feature. A later
version-boundary fixture pair,
`../runtime/v5_const_integer_division.pine` and
`../runtime/v6_fractional_integer_division.pine`, proves that v5 truncates only
two `const int` operands; input and series integers retain fractional results,
and v6 requires an explicit `int(...)` cast when truncation is intended.

The next call-shape slice adds
`v4/runtime/numeric_bool_call_arguments_legacy.pine`. It extends the existing
Pine v1-v5 numeric-to-bool rule to bool-compatible built-in parameters using an
explicit canonical `bool(...)` lowering. The source qualifier is preserved,
zero and `na` are false, nonzero numerics are true, and v6 remains strict.

The following array-index slice adds
`v4/runtime/array_series_index_legacy.pine`. It proves that `array.get()` and
`array.set()` accept a per-bar `series int` index in Pine v4, matching the
general array contract. Modern namespace and method forms share the same
integer-compatible signature, while float and string indexes remain rejected.

The subsequent drawing-enum slice adds
`v4/runtime/dynamic_drawing_enums_legacy.pine`. It proves that supported
`line` style/extend and `label` style values can vary by bar, including the
historical v4 `label.style_labelup` / `label.style_labeldown` spellings.
Static enum-domain checks keep unbounded strings and invalid branches rejected.

The following `na`-origin slice adds
`v2/runtime/bool_numeric_comparisons_legacy.pine` and
`v4/runtime/udf_source_order_builtin_aliases_legacy.pine`. The first proves
that Pine v1/v2 bool-versus-numeric comparisons use the same explicit
boolean-to-float lowering as their arithmetic profile, while v3 stays strict.
The second proves that, from v3 onward, a global declared after a UDF body does
not retroactively hide an unqualified historical built-in call in that body.
`v4/unsupported/udf_earlier_legacy_alias_shadow.pine` keeps an earlier lexical
collision rejected. Bare or failure-derived `na` call arguments are not
contextually accepted by this slice.

The following output-offset slice adds
`v4/runtime/series_output_offset_legacy.pine` and its explicit constant-offset
v6 rewrite. Pine v4/v5 `plot`, `plotchar`, `plotshape`, `plotarrow`,
`bgcolor`, and `barcolor` accept `series int` offsets and apply the final
evaluated value to the complete output. The v3 and v6 negative fixtures keep
their simple-int boundary, and ordinary `expr[offset]` history access is not
changed.

The next UDF-return slice adds
`v4/runtime/udf_final_statements_legacy.pine` and an explicit-expression v6
rewrite. A final local declaration or reassignment returns its bound value,
branch-final declarations participate in conditional results, and a final
conditional without `else` returns `na` on the missing path.
The following reference-side-effect slice adds
`v4/runtime/udf_reference_side_effects_legacy.pine` and an explicitly expanded
v6 rewrite. Pine v4 UDFs may use the corpus-backed namespace calls
`array.set/pop/unshift/clear`, `label.new/delete`, and `line.new/delete`,
including final side-effect-only conditionals and loops. The paired runtime
fixture covers historical, incremental, and realtime historical handoff;
`v4/unsupported/udf_other_reference_side_effects.pine` keeps all other
collection and drawing mutations fail-closed.

The following parser slice adds
`v1/runtime/ternary_continuation_legacy.pine` and a single-line v6 rewrite.
Only a no-directive v1 source at global scope may treat exactly four ASCII
spaces as a continuation when the line boundary is adjacent to ternary `?` or
`:` punctuation. The paired fixture covers both operator-at-end and
operator-at-start forms. Explicit versions, tabs, local blocks, ordinary
multiple-of-four indentation, and consumer typing remain unchanged.

The next declaration-graph slice adds
`v1/runtime/graph_source_order_prerequisite_legacy.pine` and its explicit v6
rewrite. An earlier scalar `input()` needed only to infer a later self-history
chain remains in ordinary source order instead of becoming a predeclared graph
node. The same pair proves the exact v1-v4 `rising` / `falling` mappings to
`ta.rising` / `ta.falling`. The v2 unsafe-initializer fixture still rejects an
input that is itself a graph target, and
`v2/unsupported/forward_reference_unsafe_initializer_barrier.pine` prevents a
current-value forward edge from moving across an input declaration.

The following version-annotation slice adds
`v4/runtime/spaced_version_annotation_legacy.pine` and its canonical v6
rewrite. Horizontal whitespace around the equals sign in `//@version = 4`
selects an explicit v4 dialect instead of falling back to implicit v1. The
prefix remains exact: `// @version=6` is still an ordinary comment. The paired
fixture exercises v4-only qualified colors, size constants, `study`
`max_bars_back`, output metadata, and alert declarations through historical,
incremental, realtime, and resource release gates.

The next conditional-result slice adds
`v4/runtime/nested_if_expression_legacy.pine` and its canonical v6 pair. A
complete nested `if`/`else-if`/`else` statement at the end of an enclosing
value-producing `if` block recursively supplies that branch's result.
`sema/unsupported_nested_if_branch_return.pine` keeps a nested leaf ending in
a reassignment behind `E_BRANCH_RETURN`.

The following input-runtime slice adds
`v4/runtime/named_input_default_legacy.pine` and its canonical v6 pair. Input
defaults are selected by the canonical `defval` parameter instead of raw source
position, so a named `title` or other metadata argument may precede `defval`
without becoming the runtime value. Callsite allocation, metadata, host
overrides, and the positional-input path remain unchanged.

Phase 8 adds paired `v3/runtime/core_*` fixtures for the executable v3 name,
constant, declaration, input, output, chart-metadata, and untyped-`na` slice.
`v3/sema/shadowing.pine` proves that source declarations retain precedence over
fallback aliases, while `v3/unsupported` owns stable fixtures for ambiguous
`na` inference and later-only call parameters.

Phase 9 adds the implicit-v1/explicit-v2 shared pair and the paired
`v2/runtime/core_*` fixtures for self-history, current and historical forward
references, bool arithmetic, and numeric conditions. `v2/unsupported` owns the
stable graph cycle, statement-barrier, and unsafe-initializer diagnostics;
v3/v6 negative controls prove the conversion version boundaries. The
host-neutral `runtime_legacy_v2_core.json` golden is generated by the CLI and
required by Python and WASM parity tests.

Phase 10 adds the dedicated implicit-v1 and v4-input runtime goldens plus five
complete `analysis_legacy_*` reports spanning v1-v4 and a v2 graph failure.
`scripts/host_parity_required.txt` and
`scripts/legacy_analysis_parity_required.txt` are the explicit two-host policy
manifests; `scripts/check_host_parity.py` verifies the CLI registrations and
both Python/WASM assertions.

The execution-clock slice adds
`v4/runtime/timenow_execution_clock_legacy.pine` and its canonical v6 pair.
`timenow_execution_times.txt` provides the deterministic per-execution UNIX
millisecond inputs used by the `deterministic_clock` release profile. The gate
proves historical, incremental, forming replacement/rollback, and confirmation
behavior without substituting bar time or a process wall clock.
