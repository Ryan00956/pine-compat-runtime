# Strategy Internal Stage 18g True OHLC Path Audit

Status: draft after Slice 18g.0 on 2026-09-03. Stage 18g is **blocked** before
any path-builder or fill-order change. Official high-first and low-first path
rules are locked. Equal-distance selection, same-price entry-versus-exit rank,
same-price exit-versus-margin rank, and same-bar stop-limit post-activation
eligibility remain unresolved without a lawful TradingView reference export.

This document is a reference-lock and design-correction record. It does not
claim new strategy support. Support claims still come from
`tests/fixtures/conformance.tsv`, committed fixtures and snapshots, host
parity, and a passing `scripts/verify.sh` run.

Starting commit: `1e9ac6af6d585fb76c39674b627f68292878a542` (`main`).
Working branch: `codex/strategy-stage18g-ohlc-path`.
Ending commit for this slice: recorded in the 18g.0 commit that adds this
draft. Later slices must not append an ending Stage 18g closeout commit until
the blocking questions below are resolved.

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

## Design Correction

Stop Stage 18g here. Do not start Slice 18g.1 or any later behavior-changing
slice until a lawful source-free TradingView order/trade export answers the
blocking questions below.

Required external evidence, for original scripts and hand-authored OHLC, not
market-feed dependence:

1. Equal-distance path: which of open-high-low-close or open-low-high-close
   wins when `|open-high| == |open-low|`, including whether close direction
   matters, and what happens when the bar is a four-price doji.
2. Same-price user entry versus user exit on one path crossing: observed
   order-id sequence, bar index, fill price, direction, quantity, and
   resulting position.
3. Same-price user exit versus synthetic margin on one path mark: the same
   fields, plus whether the margin event is a public order and whether it
   impersonates a user id.
4. Stop-limit activation followed by a limit crossing on the **same**
   historical bar: whether the limit can fill on the activation bar, and at
   which path mark.

Until those rows exist, keep:

- the production `HistoricalFillStep` family-order dispatcher;
- the current later-bar stop-limit delay
  (`activated_bar_index < bar_index`);
- current whole-bar margin and exit phases;
- public strategy JSON schema versions unchanged.

Do not default equal-distance to high-first, low-first, close-direction, or
"else" of either official clause. Do not invent an entry-before-exit,
long-before-short, or margin-before-exit rank to preserve an old snapshot.

## Reference Matrix

| Case | Official statement | Lawful TV export | Expected sequence recorded before Rust change | Classification |
| --- | --- | --- | --- | --- |
| Open closer to high | open, high, low, close | not supplied | high-first long: stop then limit; high-first short: limit then stop, on the script/bars below | confirmed-in-scope |
| Open closer to low | open, low, high, close | not supplied | low-first long: limit then stop; low-first short: stop then limit | confirmed-in-scope |
| Open exactly equidistant | neither closer-than clause applies; page silent | not supplied | **blocked** | unresolved-blocking |
| Same-price entry vs user exit | path crossing only; no family rank | not supplied | **blocked** | unresolved-blocking |
| Same-price exit vs margin | margin formula exists; no same-price rank vs user exit | not supplied | **blocked** | unresolved-blocking |
| Same-bar stop-limit then limit | multi-bar example only | not supplied | **blocked** for widening; keep current later-bar delay fail-closed | unresolved-blocking |
| Inter-bar gap crossing | fill at next open | not supplied | keep current next-open fill; not part of this rewrite | confirmed-deferred |
| `calc_on_order_fills` | extra pass after a fill, not after non-fill bookkeeping | Stage 21b goldens | unchanged 18g.0 claim | confirmed-in-scope |
| In-range prices reachable | any price in the bar range | not supplied | use asymmetric bars in later fixtures | confirmed-in-scope |
| Bar Magnifier | lower-timeframe OHLC replaces inference | Stage 21e host contract | out of Stage 18g | confirmed-deferred |

## Oracle Scripts And Bars

These scripts are original and were **not** executed against a TradingView
reference environment in this worktree. They are the scripts that must be run
to lift the block. Do not commit them as passing runtime fixtures until
source-free order/trade rows exist.

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

Implemented path and comparator: not implemented.
Internal identity migration: not implemented.
Added fixtures: none.
Intentionally changed snapshots: none.
Schema versions: unchanged.
Stage 18g closed: no.
