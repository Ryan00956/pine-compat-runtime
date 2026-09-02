# Strategy Internal Stage 14h Short Exit Bracket Audit

Status: closed. This slice adds fixture-backed one-downside/one-upside
`strategy.exit` brackets against the current market short-entry subset.

## Closed Subset

- `stop+limit`, `stop+profit`, `loss+limit`, and `loss+profit` brackets cover a
  matching open short entry.
- The stop/loss leg is the higher cover price and fills when `high >= stop`.
- The limit/profit leg is the lower cover price and fills when
  `low <= limit - verification_offset`.
- Relative `profit`/`loss` ticks reuse Stage 14g conversion from the matching
  short entry price.
- When both legs touch on the same eligible bar, the stop/loss leg fills.
- Cover fills reuse the Stage 14f short stop/limit path.
- Pending-short relative bracket attachment and omitted-`from_entry` all-entry
  fan-out remain no-op on shorts. Trailing stops landed in Stage 14i.

## Evidence

- `tests/fixtures/runtime/strategy_exit_bracket_stop_limit_stop_fill_short.pine`
- `tests/fixtures/runtime/strategy_exit_bracket_stop_limit_limit_fill_short.pine`
- `tests/snapshots/runtime_strategy_exit_bracket_stop_limit_stop_fill_short.json`
- `tests/snapshots/runtime_strategy_exit_bracket_stop_limit_limit_fill_short.json`
- CLI/Python/WASM host parity for those snapshots
- Broker tests `stage14h_*`
- Runtime test `strategy_exit_bracket_stop_limit_short_stop_covers_on_later_high`

## Unchanged Claims

Short limit/stop/stop-limit entries, generic `strategy.order()` netting, and
`margin_short` runtime behavior remain unsupported. Short trailing landed in
Stage 14i.
