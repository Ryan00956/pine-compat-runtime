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

The committed, redacted Phase 0 result is recorded in
`docs/LEGACY_INDICATOR_PHASE0_BASELINE.md`.

Versioned `v*/syntax`, `v*/sema`, `v*/runtime`, and `v*/unsupported`
directories own paired compatibility fixtures added after the baseline. Phase 3
adds v4 declaration and exact-alias pairs under `v4/sema` and `v4/runtime`;
these files are original project fixtures and are also referenced by
`conformance.tsv`.

Phase 4 adds the paired `v4/runtime/inputs_*` fixtures for all supported Pine v4
input type constants, metadata, callsites, default values, and scalar host
overrides. `v4/sema/input_constant_alias.pine` owns the local const-alias case.
