# Strategy Internal Stage 18g True OHLC Path Audit

Status: Slice 18g.0 reference lock plus B1 contract amendment recorded on
2026-09-03. Slice 18g.1 path primitives are authorized under that amendment.
Family-order production fills remain active until a later slice. This is not
Stage 18g closeout.

This document is a reference-lock, evidence index, and contract-amendment
record. It does not claim new public strategy support. Support claims still
come from `tests/fixtures/conformance.tsv`, committed fixtures and snapshots,
host parity, and a passing `scripts/verify.sh` run.

Starting commit: `1e9ac6af6d585fb76c39674b627f68292878a542` (`main`).
Working branch: `codex/strategy-stage18g-ohlc-path`.
18g.0 docs commit: `f2f9338a06b72516aaedcba9e451e750c2fbcf75`.

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

## Current Runtime Boundary (Unchanged)

`HistoricalFillStep::pre_script_path()` still orders long limit, long stop,
long stop-limit, short limit, short stop, and short stop-limit. Pending exits
still fill after script statements. Margin still runs as a whole-bar phase
after entries. Long stop-limit activation sets `activated_bar_index` when
`high >= stop`; the limit is eligible only when
`activated_bar_index < bar_index`. Public JSON still omits internal order
keys.

Baseline recorded on 2026-09-03 from this worktree:

- `cargo fmt --check` exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` exit 0
- `cargo test -p pine-runtime strategy -- --test-threads=1` exit 0
- `cargo test -p pine-runtime magnifier -- --test-threads=1` exit 0
- `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` exit 0
- `cargo test -p pine-cli matrix_output_matches_golden_snapshot` exit 0
- `python3 scripts/check_host_parity.py` exit 0
- `scripts/verify.sh` exit 0 (623 Python tests in the release-gate venv)

Frozen pre-stage snapshot set: 319 `tests/snapshots/runtime_strategy_*.json`
files. Slice 18g.0 does not change any of them.

## Remaining Bar Magnifier And Gap Boundaries

Bar Magnifier remains out of Stage 18g. The Stage 21e host contract can later
feed a different OHLC tick sequence into the same event loop; it must not
create a second broker. Inter-bar gap fills stay confirmed-deferred: official
next-open filling is locked, but it is not part of the intrabar walk.

## Closeout Fields (Not Yet Applicable)

Implemented path builder: Slice 18g.1 pure `HistoricalPath` (not wired into
the production scheduler). Equal-distance uses the sample-locked OLHC rule.
Candidate comparator: not implemented. No B1 entry/exit type rank.
Internal identity: Slice 18g.2 broker-wide `OrderBook` sequence allocates
`InternalOrderKey` for pending entries, generic orders, closes, and exits.
Replacement keeps the original key; cancel+replace does not reuse keys;
expanded per-trade exits are ledger-ordered; snapshot/restore/forming
rollback continue from the saved next key. Public JSON still omits the key.
Added fixtures: none.
Intentionally changed snapshots: none.
Schema versions: unchanged.
Stage 18g closed: no.
