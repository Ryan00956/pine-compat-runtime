# Strategy Internal Stage 18g True OHLC Path Audit

Status: Stage 18g closed on 2026-09-04 after slices 18g.0-18g.8. This document
is the reference lock, evidence index, implementation record, snapshot
allowlist, and closeout. Support claims still come from
`tests/fixtures/conformance.tsv`, committed fixtures and snapshots, host
parity, and a passing `scripts/verify.sh` run.

Starting commit: `1e9ac6af6d585fb76c39674b627f68292878a542` (`main`).
Working branch: `codex/strategy-stage18g-ohlc-path`.
18g.0 docs commit: `f2f9338a06b72516aaedcba9e451e750c2fbcf75`.
18g.7 last behavior commit: `b1902e5ec88f787367085cbb70d2a34adf2b155a`.

## Official Review

Re-opened on 2026-09-03:

- https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator
  Headings: Broker emulator; Adjusting historical bar detail
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior
  Headings: Altering calculation behavior; `calc_on_every_tick`;
  `calc_on_order_fills`; `calc_on_every_history_tick`; `process_orders_on_close`
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#margin-and-leverage
- https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/
- https://www.tradingview.com/support/solutions/43000786181/ (settings; no path
  tie rule)
- https://www.tradingview.com/pine-script-docs/v3/essential/strategies/#broker-emulator
  (same two exclusive closer-to-high / closer-to-low clauses)

The current official broker-emulator page states:

- when the open is closer to the high than to the low, the inferred path is
  open, high, low, close;
- when the open is closer to the low than to the high, the inferred path is
  open, low, high, close;
- every price inside one bar's range is reachable along that path;
- a price crossed only in the gap between the previous close and the next open
  fills at the next open, not at the requested price;
- Bar Magnifier replaces chart-bar inference with lower-timeframe OHLC when
  that data exists;
- `calc_on_order_fills` adds an execution after an order fill.

The official stop-limit example activates on a later bar, then fills after a
later reverse to the limit. It does not describe same-bar post-activation
eligibility.

The official broker-emulator diagram labels four qualitative bars as ticks
1-4. None of those bars is a measured equidistant case.

The v3 History SAW demo is consistent with four path points on a historical
bar when `calc_on_order_fills` is enabled. It does not identify the
equal-distance branch.

No comparator rank is taken from the current family-order scheduler, enum
order, or existing snapshots.

## Evidence Index (2026-09-03)

Chrome Strategy Tester pack (BINANCE:ADAUSDT.P 1m, Pine v6,
`use_bar_magnifier=false`, commission/slippage/limit-verify 0; emulator build
not exposed). Do not commit TradingView Pine sources from these packs as
runtime fixtures.

| Artifact | SHA-256 |
| --- | --- |
| `18g-b1-evidence-package.zip` | `506903bb331af76624d4402494babd3c4878905fd82ba438a08c94f4ad7d8725` |
| `18g-b1-source-free.json` | `5e037d5927d5eb108661b7b58d2f78c6f062449733c2dc64d0cc5440d5317fb2` |
| `18g-b1-contract-amendment.md` | `3696037b841aaebd0f82e398ba991a5f9d1ba85335c7babd454e8472d176968c` |
| earlier `stage18g-source-free-runs.json` (A/C/D) | `bb5ced6d188e52e667b112f6abfcfe9ad223ac84baf95d2ca029280f5773a49c` |

A, C, and D were not re-run in the B1 package. They stay the sample-level
rows in the 17-run JSON / Chrome audit report.

## B1 Contract Amendment

Confirmed by the user on 2026-09-03 after the B1 follow-up (both declaration
orders, capacity variants, NEXT-only and early-EX controls; 6 runs, 56 SNAP
lines). Target bar still only showed `BASE=1 → NEXT=1`. No intermediate
`position_size=0` or `position_size=2`. Capacity early-EX did not restore a
NEXT fill, so final flat cannot rank ENTRY vs EXIT. No margin call in B1.

Amendment:

- Do not lock a global entry-before-exit or exit-before-entry type rank.
- B1 is `UNVERIFIED_INTERNAL_ORDER` / an unverified boundary.
- For same-price events not locked by evidence, this runtime uses creation
  sequence and a stable internal key for determinism only. That is this
  runtime's contract. It is not a claim that TradingView's private internal
  order is reproduced.
- Missing intermediate script callbacks must not be read as proven emulator
  atomicity.
- A, C, and D remain sample-level ADAUSDT conclusions. They are not
  all-symbol laws and must not be re-run to "complete" 18g.0.

Creation-sequence / stable-key reproducibility must be testable. It must not
depend on wall-clock time, thread scheduling, random values, or unstable
collection order.

## Sample-Level Path And Event Locks

These are ADAUSDT 1m tick-aligned analogues, not the original hand-authored
`O=10 H=12 L=8 C=10` / 10.8/8.2 numeric oracles.

- Equal-distance analogue bar `O=0.1939 H=0.1941 L=0.1937 C=0.1939`: LIM@0.1938
  then STP@0.1940 on bar 23851. Sample-level path is open-low-high-close.
  Equal-distance doji and close-direction variants stay out of evidence.
- High-first stop-limit bar 23863 `O=0.1968 H=0.1971 L=0.1961 C=0.1969`: SL
  fills at 0.1962 on that bar. Sample-level same-bar post-activation fill.
  Short and low-first stop-limit stay out of evidence.
- User exit vs margin at mark 0.1937: USER_EXIT qty=1 then Margin call. Sample
  for this pre-placed long stop-exit. Not all margin scenes. The margin event
  is a public `Margin call` / `Close position order`, not a user id.

Inter-bar gap remains confirmed-deferred from official docs and is not in this
pack.

## Design Correction

The previous 18g.0 stop ("no Tester rows") is lifted for Slice 18g.1 path
primitives only. Later fill-order slices still must not invent a B1 type rank
or generalize A/C/D beyond the recorded samples.

Until later slices own fill-order changes, keep:

- the production `HistoricalFillStep` family-order dispatcher;
- current whole-bar margin and exit phases;
- public strategy JSON schema versions unchanged.

Stop-limit same-bar eligibility is sample-locked for the high-first long
analogue above; do not widen short/low-first without evidence. Do not infer
TradingView internal atomicity from missing intermediate callbacks.

## Reference Matrix

| Case | Official statement | Lawful TV export | Expected sequence recorded before Rust change | Classification |
| --- | --- | --- | --- | --- |
| Open closer to high | open, high, low, close | official pages | high-first long: stop then limit; high-first short: limit then stop | confirmed-in-scope |
| Open closer to low | open, low, high, close | official pages | low-first long: limit then stop; low-first short: stop then limit | confirmed-in-scope |
| Open exactly equidistant | page silent | ADAUSDT analogue LIM then STP (OLHC) | sample-level OLHC; not original 10/12/8/10; doji/close-direction unverified | sample-locked |
| Same-price entry vs user exit | path crossing only; no family rank | B1 pack: BASE=1→NEXT=1, no pos 0/2 | `UNVERIFIED_INTERNAL_ORDER`; runtime creation sequence/key only | unverified-boundary |
| Same-price exit vs margin | margin formula exists; no general rank | USER_EXIT then Margin call at 0.1937 | sample-level for that long stop-exit | sample-locked |
| Same-bar stop-limit then limit | multi-bar official example | high-first SL fill 0.1962 on bar 23863 | sample-level same-bar fill; short/low-first unverified | sample-locked |
| Inter-bar gap crossing | fill at next open | not supplied | keep current next-open fill; not part of this rewrite | confirmed-deferred |
| `calc_on_order_fills` | extra pass after a fill, not after non-fill bookkeeping | Stage 21b goldens | unchanged 18g.0 claim | confirmed-in-scope |
| In-range prices reachable | any price in the bar range | not supplied | use asymmetric bars in later fixtures | confirmed-in-scope |
| Bar Magnifier | lower-timeframe OHLC replaces inference | Stage 21e host contract | out of Stage 18g | confirmed-deferred |

## Oracle Scripts And Bars

These original synthetic scripts were the 18g.0 templates. TradingView
evidence used tick-aligned ADAUSDT analogues instead of these exact prices.
Do not commit TradingView Pine sources as passing runtime fixtures.

Platform context to record with any future export: Pine version, broker
emulator build/UI date, symbol, timeframe, `use_bar_magnifier=false`,
`calc_on_order_fills=false` unless the case requires it, commission 0,
slippage 0, limit verification 0.

### Shared bar set

Hand-authored prices. Distances are exact in decimal arithmetic.

| Bar | Open | High | Low | Close | Path if official closer-than clauses apply |
| --- | ---: | ---: | ---: | ---: | --- |
| 0 | 10 | 10 | 10 | 10 | degenerate; no fill intended |
| 1 high-first | 10 | 11 | 8 | 9 | `|o-h|=1 < \|o-l\|=2` → OHLC |
| 2 low-first | 10 | 12 | 9 | 11 | `|o-h|=2 > \|o-l\|=1` → OLHC |
| 3 equal | 10 | 12 | 8 | 10 | `|o-h|=2 = \|o-l\|=2` → **unknown** |
| 4 gap | 14 | 14.5 | 13.5 | 14 | previous close 10; 11 is in the gap |

### High-first / low-first long

Source sha256 `b1650da82890d271b35e522e4924f70037c14593943dcfa86f6f1777a668bfa9`.

```pine
//@version=5
strategy("18g0 high-low first long", overlay=false, pyramiding=2, initial_capital=100000)

if bar_index == 0
    strategy.entry("STP", strategy.long, qty=1, stop=10.8)
    strategy.entry("LIM", strategy.long, qty=1, limit=8.2)
```

On bar 1 (high-first) the expected **in-scope** sequence, if later slices
proceed after this lock, is STP then LIM: the stop is crossed on open→high,
the limit on high→low. On bar 2 (low-first) the expected in-scope sequence is
LIM then STP. Bar 3 must not be used to pick a default.

### High-first / low-first short

Source sha256 `5dadd5fcbf76770c8578201ab2613e3e9a5c597ea0c9736580d68b8faf23cb01`.

```pine
//@version=5
strategy("18g0 high-low first short", overlay=false, pyramiding=2, initial_capital=100000)

if bar_index == 0
    strategy.entry("STP", strategy.short, qty=1, stop=8.2)
    strategy.entry("LIM", strategy.short, qty=1, limit=10.8)
```

On bar 1 (OHLC) LIM then STP. On bar 2 (OLHC) STP then LIM.

### Equal-distance

Same long script as the high-first/low-first long case, observed only on
bar 3. Record which public order id fills first, fill prices, and whether both
fill. **No expected sequence is recorded.**

### Same-price entry versus exit

Source sha256 `633795aa79e836ec55f62e58bf93a28f279ca0cbc238991075faad9ddab0f3ad`.

```pine
//@version=5
strategy("18g0 same price entry vs exit", overlay=false, initial_capital=100000)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1)
if bar_index == 1
    strategy.exit("EX", "EN", limit=10.5)
    strategy.entry("NX", strategy.long, qty=1, limit=10.5)
```

Use a later bar whose path crosses 10.5 once (for example a high-first bar
with open 10, high 11, low 8, close 9). Record whether EX or NX fills first
and the resulting position. **No expected sequence is recorded.**

### Exit versus margin

Source sha256 `4e54f43ae1bd1df3b120f7864141ff32cfd2b1cfd1e25cdd1dc49086c1915fdf`.

```pine
//@version=5
strategy("18g0 exit vs margin", overlay=false, initial_capital=1000, margin_long=25)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=10)
if bar_index == 1
    strategy.exit("EX", "EN", stop=8.5)
```

Choose later bars so a user stop and a margin-call mark coincide at one
visited price. Record whether the public exit, a forced liquidation, or both
occur, and the remaining quantity. **No expected sequence is recorded.**

### Same-bar stop-limit

Source sha256 `d16dac2e23acf7b02d02c780c1e26251908549eb935d90102a653892f27302a7`.

```pine
//@version=5
strategy("18g0 same bar stop limit", overlay=false, initial_capital=100000)

if bar_index == 0
    strategy.entry("SL", strategy.long, qty=1, stop=10.8, limit=8.2)
```

On bar 1 (high-first) the stop is crossed on open→high and the limit on
high→low. Record whether SL fills on bar 1, on a later bar, or never, and the
fill price. Current runtime delays the limit until a later bar. **Do not
widen that delay from this script until a TV export exists.**

### Inter-bar gap

Source sha256 `11694b28f21057111099f2074460d639da2dd7560b7929812abf89d2d1b79e6f`.

```pine
//@version=5
strategy("18g0 gap fill", overlay=false, initial_capital=100000)

if bar_index == 0
    strategy.entry("GAP", strategy.long, qty=1, limit=11)
```

Bar 0 close is 10. Bar 4 opens at 14. Official rule: fill GAP at the next
open (14), not at 11. Deferred from the intrabar rewrite; keep the existing
runtime gap behavior unless a dedicated later slice re-audits it.

## Slice Commits

| Slice | Commit | Subject |
| --- | --- | --- |
| 18g.0 | `f2f9338a06b72516aaedcba9e451e750c2fbcf75` | docs(strategy): record Stage 18g.0 reference lock and unresolved-rule block |
| 18g.1 | `759038cbe37ebbc1cb366d95d0208670305632d8` | refactor(strategy): add historical OHLC path primitives |
| 18g.2 | `f6ac67a65559347913a051c5fa4e8b141c01ad73` | refactor(strategy): unify pending order creation identity |
| 18g.3 | `46ea057f5d67338ecac60ed2eb2a8ee8efd507d9` | refactor(strategy): collect broker events without mutation |
| 18g.4 | `3bcd5bae4c68b2db04d9eb63e5d549c7215e6cd7` | feat(strategy): order entry fills along historical OHLC paths |
| 18g.5 | `9f606e3133bf8ed40ea5d80688c65113db3fd2f1` | feat(strategy): integrate exits with historical OHLC paths |
| 18g.6 | `f55ce5a67fdd44ce52b0d8ff5b90dd26fa11c325` | feat(strategy): evaluate margin events on historical OHLC paths |
| 18g.7 | `b1902e5ec88f787367085cbb70d2a34adf2b155a` | test(strategy): close OHLC path recalculation and rollback gaps |
| 18g.8 goldens | `f0121c06bd90e0c39df3eeed639eb3624fc8694b` | test(strategy): publish Stage 18g OHLC path goldens |
| 18g.8 clippy | `d6f27e85a0c4592d2f7aae42720e59adbd2806e1` | refactor(strategy): satisfy clippy on OHLC path helpers |
| 18g.8 matrix | `e082b2a6c1458d9e6112306933e8a0b60b983b8e` | test(strategy): align CLI host-shape and matrix goldens |
| 18g.8 host parity | `72d3a8d85a9d69ba1ee36c4a35d3c5f9add21e73` | test(strategy): register fill-path host parity assertions |
| 18g.8 python | `d96039c87fde274647b9fd042da715628cf65692` | test(strategy): align Python plot contracts with path fills |

18g.8 documentation closeout follows these evidence commits.

## Implemented Path And Comparator

`HistoricalPath::from_ohlc` selects open-high-low-close when the open is
closer to the high than to the low, otherwise open-low-high-close, including
the sample-locked equal-distance ADAUSDT analogue. `from_validated_bar`
clamps an out-of-range open into high/low so invalid-but-accepted bars can
still walk; it does not expand the range to a close outside high/low.

Production pre-script ticks are market closes at open, market entries at
open, then `HistoricalFillStep::IntrabarPath`. The scheduler stores a path
cursor and resumes it after OCA effects and bounded `calc_on_order_fills`
passes.

Read-only `BrokerCandidate` collection ranks by phase, path leg, crossing
order on that leg, user fills before a same-mark margin call, then creation
sequence and stable internal key. There is no global entry-before-exit or
long-before-short type rank. Stop-limit activation and trailing
activation/ratchet are distinct from fills.

One broker-wide `OrderBook` sequence allocates `InternalOrderKey` for
pending entries, generic orders, closes, and exits. Same-id replacement
keeps the key; cancel+replace does not reuse it. Public JSON omits the key.

High-first long stop-limit orders may fill on the same bar after activation
(sample-locked). Short and low-first stop-limit fills stay fail-closed until
a later bar. Trailing activation uses the first visited mark already through
activation; ratchet uses only visited extremes. Exits and margin calls share
the same pre-script path. Successful margin fills clear pending entries.
Invalid historical paths fall back to whole-bar extremes/margin/risk.
Forming realtime rollback restores the scheduler path cursor with the
confirmed broker checkpoint.

## Added Fixtures

Path-direction, identity, collision, stop-limit, exit, trailing, margin,
recalculation, and realtime fixtures:

- `strategy_fill_path_high_first_long.pine`
- `strategy_fill_path_low_first_long.pine`
- `strategy_fill_path_high_first_short.pine`
- `strategy_fill_path_low_first_short.pine`
- `strategy_fill_path_same_price_creation_order.pine`
- `strategy_fill_path_order_oca_cancel.pine`
- `strategy_fill_path_stop_limit_long.pine`
- `strategy_fill_path_stop_limit_short.pine`
- `strategy_fill_path_entry_then_exit_same_bar.pine`
- `strategy_fill_path_exit_then_entry_same_bar.pine`
- `strategy_fill_path_bracket_high_first.pine`
- `strategy_fill_path_bracket_low_first.pine`
- `strategy_fill_path_bracket_short_high_first.pine`
- `strategy_fill_path_bracket_short_low_first.pine`
- `strategy_fill_path_trailing_activation_then_fill.pine`
- `strategy_fill_path_trailing_no_future_extreme.pine`
- `strategy_fill_path_exit_oca_reduce.pine`
- `strategy_fill_path_partial_exit_reservation.pine`
- `strategy_fill_path_exit_before_margin_long.pine`
- `strategy_fill_path_margin_before_exit_long.pine`
- `strategy_fill_path_exit_before_margin_short.pine`
- `strategy_fill_path_margin_before_exit_short.pine`
- `strategy_fill_path_drawdown_intrabar_ordering.pine`
- `strategy_fill_path_margin_invalidates_entry.pine`
- `strategy_fill_path_calc_on_order_fills_resume.pine`
- `strategy_fill_path_calc_on_order_fills_guard.pine`
- `strategy_fill_path_realtime_stop_limit_rollback.pine`
- `strategy_fill_path_realtime_trailing_rollback.pine`
- `strategy_fill_path_realtime_margin_rollback.pine`

Host-parity bars and goldens for the required 18g.8 subset:

- `strategy_fill_path_high_first_long_bars.csv`
- `strategy_fill_path_low_first_short_bars.csv`
- `strategy_fill_path_entry_then_exit_same_bar_bars.csv`
- `strategy_fill_path_stop_limit_long_bars.csv`
- `strategy_fill_path_exit_before_margin_long_bars.csv`

CLI, Python, and WASM assert those five public strategy JSON goldens. Public
`StrategyResult` remains `schemaVersion` 8 and `renderMetadataVersion` 1.

## Intentionally Changed Snapshots

Cause classes:

- script-visible plots and trade counts now observe pre-script path exits on
  the fill bar rather than the next bar;
- same-price OCA uses creation order rather than a downside-first family
  rank;
- high-first long stop-limit same-bar fill moves
  `runtime_strategy_pyramiding_limit_same_tick_stop_limit_entries.json`
  fills from bar 2 to bar 1.

New goldens:

- `runtime_strategy_fill_path_high_first_long.json`
- `runtime_strategy_fill_path_low_first_short.json`
- `runtime_strategy_fill_path_entry_then_exit_same_bar.json`
- `runtime_strategy_fill_path_stop_limit_long.json`
- `runtime_strategy_fill_path_exit_before_margin_long.json`

Updated existing goldens (155 files, all `runtime_strategy_*`):

`runtime_strategy_close_entries_rule_any_exit_from_entry_short.json`,
`runtime_strategy_close_entries_rule_any_exit_same_id_partial_short.json`,
`runtime_strategy_exit_active_entry_attachment.json`,
`runtime_strategy_exit_active_entry_loss_attachment.json`,
`runtime_strategy_exit_active_entry_loss_limit_bracket.json`,
`runtime_strategy_exit_active_entry_loss_profit_bracket.json`,
`runtime_strategy_exit_active_entry_profit_attachment.json`,
`runtime_strategy_exit_active_entry_stop_profit_bracket.json`,
`runtime_strategy_exit_active_entry_trail_points_attachment.json`,
`runtime_strategy_exit_bracket_both_hit.json`,
`runtime_strategy_exit_bracket_creation_bar.json`,
`runtime_strategy_exit_bracket_interactions.json`,
`runtime_strategy_exit_bracket_invalid_leg.json`,
`runtime_strategy_exit_bracket_loss_profit_loss_fill.json`,
`runtime_strategy_exit_bracket_loss_profit_profit_fill.json`,
`runtime_strategy_exit_bracket_mixed_pairs.json`,
`runtime_strategy_exit_bracket_repeated.json`,
`runtime_strategy_exit_bracket_replacement.json`,
`runtime_strategy_exit_bracket_state.json`,
`runtime_strategy_exit_bracket_stop_limit_limit_fill.json`,
`runtime_strategy_exit_bracket_stop_limit_limit_fill_short.json`,
`runtime_strategy_exit_bracket_stop_limit_stop_fill.json`,
`runtime_strategy_exit_bracket_stop_limit_stop_fill_short.json`,
`runtime_strategy_exit_interactions.json`,
`runtime_strategy_exit_limit.json`,
`runtime_strategy_exit_limit_short.json`,
`runtime_strategy_exit_loss.json`,
`runtime_strategy_exit_loss_short.json`,
`runtime_strategy_exit_metadata.json`,
`runtime_strategy_exit_oca_reduce.json`,
`runtime_strategy_exit_oca_reduce_bracket.json`,
`runtime_strategy_exit_omitted_bracket_replacement.json`,
`runtime_strategy_exit_omitted_replaces_reservations.json`,
`runtime_strategy_exit_omitted_single_replacement.json`,
`runtime_strategy_exit_omitted_trailing_replacement.json`,
`runtime_strategy_exit_profit.json`,
`runtime_strategy_exit_profit_loss_interactions.json`,
`runtime_strategy_exit_profit_short.json`,
`runtime_strategy_exit_qty_bracket_partial.json`,
`runtime_strategy_exit_qty_full_clamp.json`,
`runtime_strategy_exit_qty_limit_partial.json`,
`runtime_strategy_exit_qty_percent_bracket_partial.json`,
`runtime_strategy_exit_qty_percent_full.json`,
`runtime_strategy_exit_qty_percent_full_clamp.json`,
`runtime_strategy_exit_qty_percent_limit_partial.json`,
`runtime_strategy_exit_qty_percent_repeated.json`,
`runtime_strategy_exit_qty_percent_replacement.json`,
`runtime_strategy_exit_qty_percent_state.json`,
`runtime_strategy_exit_qty_percent_stop_partial.json`,
`runtime_strategy_exit_qty_percent_trailing_partial.json`,
`runtime_strategy_exit_qty_precedence_bracket.json`,
`runtime_strategy_exit_qty_precedence_stop.json`,
`runtime_strategy_exit_qty_precedence_trailing.json`,
`runtime_strategy_exit_qty_repeated.json`,
`runtime_strategy_exit_qty_replacement.json`,
`runtime_strategy_exit_qty_state.json`,
`runtime_strategy_exit_qty_stop_partial.json`,
`runtime_strategy_exit_qty_trailing_partial.json`,
`runtime_strategy_exit_reservation_bracket_host_parity.json`,
`runtime_strategy_exit_reservation_bracket_single_downside_precedence.json`,
`runtime_strategy_exit_reservation_bracket_single_replacement.json`,
`runtime_strategy_exit_reservation_bracket_single_upside_order.json`,
`runtime_strategy_exit_reservation_bracket_state.json`,
`runtime_strategy_exit_reservation_interactions.json`,
`runtime_strategy_exit_reservation_mixed_side_precedence.json`,
`runtime_strategy_exit_reservation_qty_bracket_clamp.json`,
`runtime_strategy_exit_reservation_qty_bracket_replacement.json`,
`runtime_strategy_exit_reservation_qty_bracket_stop_limit_downside_multi.json`,
`runtime_strategy_exit_reservation_qty_bracket_stop_limit_upside_multi.json`,
`runtime_strategy_exit_reservation_qty_clamp.json`,
`runtime_strategy_exit_reservation_qty_limit_multi.json`,
`runtime_strategy_exit_reservation_qty_mixed_bracket_multi.json`,
`runtime_strategy_exit_reservation_qty_mixed_stop_multi.json`,
`runtime_strategy_exit_reservation_qty_mixed_trailing_multi.json`,
`runtime_strategy_exit_reservation_qty_percent_bracket_clamp.json`,
`runtime_strategy_exit_reservation_qty_percent_bracket_multi.json`,
`runtime_strategy_exit_reservation_qty_percent_bracket_replacement.json`,
`runtime_strategy_exit_reservation_qty_percent_clamp.json`,
`runtime_strategy_exit_reservation_qty_percent_replacement.json`,
`runtime_strategy_exit_reservation_qty_percent_stop_multi.json`,
`runtime_strategy_exit_reservation_qty_percent_trailing_clamp.json`,
`runtime_strategy_exit_reservation_qty_percent_trailing_multi.json`,
`runtime_strategy_exit_reservation_qty_percent_trailing_replacement.json`,
`runtime_strategy_exit_reservation_qty_replacement.json`,
`runtime_strategy_exit_reservation_qty_stop_multi.json`,
`runtime_strategy_exit_reservation_qty_trailing_clamp.json`,
`runtime_strategy_exit_reservation_qty_trailing_points_multi.json`,
`runtime_strategy_exit_reservation_qty_trailing_price_multi.json`,
`runtime_strategy_exit_reservation_qty_trailing_replacement.json`,
`runtime_strategy_exit_reservation_state.json`,
`runtime_strategy_exit_reservation_trailing_activation_mixed_fill.json`,
`runtime_strategy_exit_reservation_trailing_bracket_downside_order.json`,
`runtime_strategy_exit_reservation_trailing_host_parity.json`,
`runtime_strategy_exit_reservation_trailing_mixed_side_precedence.json`,
`runtime_strategy_exit_reservation_trailing_mixed_state.json`,
`runtime_strategy_exit_reservation_trailing_replacement_mixed.json`,
`runtime_strategy_exit_reservation_trailing_single_downside_order.json`,
`runtime_strategy_exit_reservation_trailing_state.json`,
`runtime_strategy_exit_slippage.json`,
`runtime_strategy_exit_stop.json`,
`runtime_strategy_exit_stop_short.json`,
`runtime_strategy_exit_trade_counts.json`,
`runtime_strategy_exit_trail_points_fill.json`,
`runtime_strategy_exit_trail_points_fill_short.json`,
`runtime_strategy_exit_trail_price_fill.json`,
`runtime_strategy_exit_trail_price_fill_short.json`,
`runtime_strategy_exit_trailing_activation_bar.json`,
`runtime_strategy_exit_trailing_close_cancel.json`,
`runtime_strategy_exit_trailing_interactions.json`,
`runtime_strategy_exit_trailing_invalid.json`,
`runtime_strategy_exit_trailing_ratchet.json`,
`runtime_strategy_exit_trailing_repeated.json`,
`runtime_strategy_exit_trailing_replacement.json`,
`runtime_strategy_exit_trailing_state.json`,
`runtime_strategy_limit_verification_exit.json`,
`runtime_strategy_pyramiding_exit_bracket_from_entry.json`,
`runtime_strategy_pyramiding_exit_from_entry.json`,
`runtime_strategy_pyramiding_exit_omitted_from_entry_current.json`,
`runtime_strategy_pyramiding_exit_omitted_from_entry_persistent.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_loss_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_profit_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_profit_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_profit_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_profit_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_points_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_points_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_price_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.json`,
`runtime_strategy_pyramiding_exit_omitted_trail_price_same_id.json`,
`runtime_strategy_pyramiding_exit_profit_from_entry.json`,
`runtime_strategy_pyramiding_exit_same_id.json`,
`runtime_strategy_pyramiding_exit_trail_points_from_entry.json`,
`runtime_strategy_pyramiding_limit_same_tick_stop_limit_entries.json`.

`tests/snapshots/matrix.json` notes and fixtures changed only for
`strategy.entry` and `strategy.order`. No indicator, analysis, or other
non-strategy snapshot changed.

## Schema Versions

Unchanged: public runtime `schemaVersion` 8, `renderMetadataVersion` 1. No
public pending-order, reservation, remaining-quantity, or internal-key
fields.

## Remaining Bar Magnifier And Gap Boundaries

Bar Magnifier remains out of Stage 18g. The Stage 21e host contract can later
feed a different OHLC tick sequence into the same event loop; it must not
create a second broker. Inter-bar gap fills stay confirmed-deferred: official
next-open filling is locked, but it is not part of the intrabar walk.
Short/low-first same-bar stop-limit and a global entry-versus-exit type rank
stay unverified. Mixed-family OCA and instrument-session calendars stay out
of this stage.

## Final Verification

Recorded after the 18g.8 evidence commits. `scripts/verify.sh` ran without
`UPDATE_SNAPSHOTS`.

- `scripts/verify.sh` exit code: 0
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed
- `cargo test --workspace` passed, including pine-runtime lib 1669, pine-cli 221, and pine-wasm 653
- `python3 scripts/check_structure.py` passed (311 production Rust source files)
- `python3 scripts/check_host_parity.py` passed (844 registered CLI runtime snapshots; 548 required runtime and 5 required legacy-analysis Python/WASM golden assertions)
- `scripts/check_wasm_node.sh` passed
- release-gate venv pytest: 628 passed
- public runtime `schemaVersion` 8 / `renderMetadataVersion` 1 unchanged
- Stage 18g closed: yes
