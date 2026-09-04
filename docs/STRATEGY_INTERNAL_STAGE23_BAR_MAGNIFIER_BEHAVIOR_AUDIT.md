# Strategy Internal Stage 23 Bar Magnifier Behavior Audit

Status: Slice 23.0 locked on 2026-09-04. Production runtime, IR, hosts, and
fixtures are unchanged. `use_bar_magnifier=true` remains semantically
rejected until Slice 23.7.

Working branch: `codex/strategy-stage23-bar-magnifier`.
Plan-baseline commit: `ad0a06fdefa118461d039e9815ca21eb47d717d1`.
Execution source of truth:
`docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_EXECUTION_PLAN.md`.

This document is the Slice 23.0 behavior contract. Later slices must implement
these rules. They must not invent a second answer for any Section 9 question.

## Official Review

Reviewed on 2026-09-04:

- https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#adjusting-historical-bar-detail
- https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior
- https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/
- https://www.tradingview.com/pine-script-docs/language/declaration-statements/

Official statements used as facts:

1. `use_bar_magnifier=true` lets the broker emulator use lower-timeframe OHLC
   for more granular historical fills. It replaces chart-bar path inference
   when that lower-timeframe coverage exists.
2. When lower-timeframe coverage is unavailable, the emulator uses its default
   chart-bar OHLC assumptions. Official docs also state a 200,000 lower-bar
   request cap.
3. Bar Magnifier does not by itself execute the script on every lower-timeframe
   history tick. Extra historical executions still depend on calculation
   settings such as `calc_on_order_fills`.
4. A price-based order crossed only in the gap between one bar's close and the
   next bar's open fills at the next open, not at the requested price.
5. Official same-chart-bar examples show that lower-timeframe OHLC can allow
   an entry and a later exit on one chart bar when the chart-bar inference
   would have delayed the exit.

Official pages do not publish:

- the public List-of-Trades timestamp granularity under Bar Magnifier;
- whether `strategy.opentrades.entry_time` becomes a lower-bar time;
- the exact first-open rule when the first lower-bar open differs from the
  chart-bar open;
- the exact lower-bar gap fill-price rule beyond the general next-open rule;
- whether a magnifier group for a live/forming realtime slot is rejected or
  ignored.

## Oracle Access

This environment cannot drive the official TradingView Strategy Tester, export
List-of-Trades rows, or inspect `strategy.*` values on a live TV chart. That
gap is recorded. Slice 23.0 therefore locks deterministic **project rules**
for every unresolved official question. These rules are this runtime's
contract. They are not a claim that TradingView's private emulator is
reproduced.

Do not copy TradingView Pine sources, private APIs, UI text, or proprietary
fixtures. The scripts below are original.

## Locked Answers

Copy-paste lock for Slice 23.0 gate:

1. Public fill timestamp: chart-bar time in existing `time` / `entry_time` / `exit_time` fields. Lower-bar event time is internal only.
2. First lower-bar open: first host-bar open is the opening market phase; no extra chart-open fill phase.
3. Magnifier-local gap: point event at the next lower-bar open; fill price is that open, not the requested stop/limit; no tradable close-to-open segment.
4. Script context after fill: chart `bar_index`, chart OHLC, chart `time`; cursor is not a Pine context; resume from the unconsumed remainder.
5. Forming realtime: historical seed/replay may consume magnifier; a live/forming bar never does; a forming-slot or out-of-range group is rejected before execution.
6. RealtimeSession ABI: schema version remains 1; optional constructor `magnifier_bars=None` is version-neutral. Public RuntimeResult / StrategyResult schemas are unchanged.

Every execution-plan Section 9 question has one explicit answer.

### 9.1 Public fill timestamp

**Rule:** public order, trade, and order-fill-alert timestamps use the
**chart-bar time**.

- `StrategyOrderEvent.time`, `StrategyTrade.entry_time`,
  `StrategyTrade.exit_time`, and `StrategyOrderFillAlertOutput.time` store the
  chart bar's `time`.
- `strategy.opentrades.entry_time`, `strategy.closedtrades.entry_time`, and
  `strategy.closedtrades.exit_time` store that same chart-bar time.
- Public `bar_index` / `entry_bar_index` / `exit_bar_index` remain the chart
  bar index.
- Lower-bar event time is internal scheduler, no-progress, trace, and test
  identity only. It is not a new public field.

Classification: **project rule**. Official List-of-Trades date/time
granularity under Bar Magnifier was not observed. Reusing the existing
time fields at chart-bar granularity needs no `StrategyResult` or
`RuntimeResult` schema change.

### 9.2 First lower-bar open

**Rule:** when magnifier coverage exists for a chart bar, the opening market
phase and the first path open are the **first lower-bar open**. There is no
separate chart-open fill phase.

- `MarketClosesAtOpen` and `MarketEntriesAtOpen` use the first host-bar open
  price.
- The first host bar then walks `HistoricalPath::from_validated_bar` starting
  at that same open.
- Script-visible `open` remains the chart-bar open even when it differs from
  the first lower-bar open.
- If host data contradicts the chart bar, fill prices follow the host sequence.
  Silent double-filling of a chart-open phase plus a lower-open phase is
  forbidden.

Classification: **project rule**, consistent with the official statement that
Bar Magnifier **replaces** chart-bar inference when lower-timeframe coverage
exists.

### 9.3 Gaps between lower bars

**Rule:** a move from one lower bar's close to the next lower bar's open is a
**point event at the next lower-bar open**, not a tradable close-to-open
segment.

- Do not synthesize an OHLC or OLHC leg across that gap.
- A price-based order whose trigger is crossed only in that gap becomes
  eligible at the next lower-bar open.
- The fill price is that next lower-bar open, not the requested stop or limit
  price. This is the official next-open gap rule applied locally between
  consecutive host bars of one chart bar.
- Better/worse relative to the requested price is irrelevant: the fill is the
  next open.
- This is magnifier-local. It does not rewrite general chart-to-chart gap
  handling, which already fills at the next host-bar open and stays deferred
  as a separate later stage for any broader rewrite.

Classification: **official next-open fact** + **project-local application** to
consecutive lower bars of one chart bar.

### 9.4 Script context after a fill

**Rule:** historical `calc_on_order_fills` keeps the Pine script on the chart
bar.

- `bar_index` is the chart bar.
- `open`, `high`, `low`, `close`, `volume`, and `time` are the chart bar.
- The extra pass may read updated `strategy.*` fill state.
- The internal host-bar / path cursor is not a new Pine execution context and
  is not exposed as `bar_index`, `time`, or OHLC.
- Newly created orders compete only on the unconsumed remainder of the host
  sequence. Already-consumed lower bars and already-consumed path marks never
  replay.

Classification: **official fact** that Bar Magnifier does not by itself run
the script on every lower-timeframe history tick, plus **project rule** that
chart context stays chart-scoped. Official pages do not publish a
lower-bar `time`/`ohlc` overlay during extra passes.

### 9.5 Realtime boundary

**Rule:** historical execution may consume validated magnifier input. A
live/forming realtime bar never does.

- Historical seed bars may consume magnifier groups.
- Historical replay and confirmed-history updates may consume magnifier groups
  for those confirmed chart indexes.
- A live/forming realtime bar consumes actual realtime updates only.
- A magnifier group whose `chartBarIndex` is outside the supplied historical
  chart-bar range, or that targets the live/forming slot, is **rejected
  before execution**. Ignoring the group is forbidden because it can hide
  accidental double execution.
- Forming rollback/replacement must not consult the historical magnifier
  manifest.

Classification: **project rule**. Official docs describe Bar Magnifier as a
historical-bar-detail setting. This repository already separates confirmed
history from forming updates; Stage 23 keeps that split.

### RealtimeSession ABI

**Rule:** `REALTIME_SESSION_SCHEMA_VERSION` remains **1**.

Adding optional `magnifier_bars=None` to session construction (the same
configuration site as `request_bars` / `input_overrides`) does not add a
lifecycle phase, does not change `seed` / `update_forming` /
`update_confirmed` / `result` required arguments, and does not change the
public result schema. Old callers that omit the argument remain compatible
and mean no magnifier input. Magnifier data is used for historical seed
only.

If a later slice is forced to change the lifecycle state machine, stop and
bump the ABI version explicitly. Do not silently treat a lifecycle change as
version-neutral.

### Public result schema

**Rule:** no new `RuntimeResult` or `StrategyResult` field is required.

Chart-bar timestamps reuse existing `time` / `entry_time` / `exit_time`.
Chart-bar identity reuses existing `bar_index` fields. Lower-bar identity
stays internal. If a later slice appears to need a public lower-bar time or
index field, stop Stage 23 and write a schema plan.

## Facts Versus Project Rules

| Topic | Official fact | Project rule |
| --- | --- | --- |
| Setting meaning | named `use_bar_magnifier=true` uses lower-TF OHLC when coverage exists | v5/v6 named const bool only; positional and v1–v4 stay rejected |
| Missing coverage | default chart-bar assumptions | `StandardOhlc` fallback + existing fallback/gap warnings |
| Extra script passes | depend on calculation settings, not on Bar Magnifier alone | `calc_on_every_history_tick` stays unimplemented and rejected |
| Gap fill | next open, not requested price | same rule at each lower-bar open; no synthetic gap segment |
| Public timestamp | unpublished under magnifier | chart-bar time in existing fields |
| First open | unpublished when host open ≠ chart open | first lower-bar open; no extra chart-open phase |
| Script OHLC/time | script is not a lower-TF history tick engine | chart-bar context on extra passes |
| Forming realtime | unpublished | reject forming-slot groups; never consume historical magnifier |
| RealtimeSession ABI | n/a | schema version stays 1; optional constructor input |

## Oracle Scripts And Bars

These original synthetic scripts are Slice 23.0 templates. They are not
passing runtime fixtures while `use_bar_magnifier=true` remains rejected.
Do not commit them under `tests/fixtures` until Slice 23.7 enablement.

Platform context to record with any future lawful export: Pine v6, commission
0, slippage 0, limit verification 0, `process_orders_on_close=false` unless a
case requires it.

### Shared chart bars

Hand-authored prices. Distances are exact in decimal arithmetic. Times are
milliseconds.

| Chart bar | Time | Open | High | Low | Close | Volume |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 1_000_000 | 10 | 10 | 10 | 10 | 1 |
| 1 | 2_000_000 | 10 | 12 | 8 | 11 | 1 |
| 2 | 3_000_000 | 11 | 11 | 11 | 11 | 1 |

Chart bar 1 is equal-distance (`|10-12|=2` and `|10-8|=2`). Stage 18g
sample-locks that case to open-low-high-close. The magnifier cases below
are designed so the lower-bar sequence is decisive independently of that
tie.

### 23.0-A Same-chart-bar entry and exit

Source sha256 `b9959fb963d4e7f1877cf30f59141e4906679def709e2a566047e4c09ac87e74`.

```pine
//@version=6
strategy("23.0 same-bar entry exit", overlay=false, initial_capital=100000, use_bar_magnifier=true)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
    strategy.exit("EX", "EN", limit=11.5)
```

Lower bars for chart bar 1:

| Lower | Time | Open | High | Low | Close |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 2_000_000 | 10.0 | 10.4 | 9.8 | 10.2 |
| 1 | 2_300_000 | 10.2 | 10.8 | 10.1 | 10.6 |
| 2 | 2_600_000 | 10.6 | 11.8 | 10.5 | 11.0 |

Locked expectation under the project rules:

- without magnifier, chart bar 1 is one inferred path and may fill both or
  delay the exit depending on OHLC vs OLHC;
- with magnifier, EN fills on lower bar 1 when 10.5 is crossed, EX fills on
  lower bar 2 when 11.5 is crossed, both with public `bar_index=1` and public
  `time=2_000_000`;
- script-visible `open/high/low/close/time` on any extra pass stay the chart
  bar 1 values.

### 23.0-B Lower-bar gap crossing a stop and a limit

Source sha256 `05dac1cf491828cf9984f9b83f2243165c83a7e8a69eb3e34817eb87dd11209e`.

```pine
//@version=6
strategy("23.0 lower-bar gap", overlay=false, initial_capital=100000, pyramiding=2, use_bar_magnifier=true)

if bar_index == 0
    strategy.entry("STP", strategy.long, qty=1, stop=10.5)
    strategy.entry("LIM", strategy.long, qty=1, limit=9.5)
```

Lower bars for chart bar 1:

| Lower | Time | Open | High | Low | Close |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 2_000_000 | 10.0 | 10.2 | 9.9 | 10.1 |
| 1 | 2_300_000 | 11.0 | 11.2 | 10.8 | 11.1 |

The move 10.1 → 11.0 is a gap. 10.5 is crossed only in that gap. STP fills at
the next lower-bar open **11.0**, not at 10.5. LIM at 9.5 is not crossed by
the gap or either lower-bar range, so it does not fill on this chart bar.
There is no synthetic 10.1→11.0 tradable segment that could fill LIM.

### 23.0-C calc_on_order_fills resume

Source sha256 `2eb539a7d1f17402bbc1c0c4465414b6969c4d00fa0b8fd10299eb00ad167dcd`.

```pine
//@version=6
strategy("23.0 calc on fills", overlay=false, initial_capital=100000, calc_on_order_fills=true, use_bar_magnifier=true)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
if strategy.position_size > 0 and strategy.opentrades == 1
    strategy.exit("EX", "EN", limit=11.5)
```

Use the 23.0-A lower bars. Locked expectation:

- EN fills on lower bar 1;
- the extra pass still sees chart `bar_index=1`, chart OHLC, and chart `time`;
- `strategy.position_size` is updated;
- EX is created after the fill and may fill only on unconsumed later host
  bars or unconsumed marks of the current host bar;
- EX must not fill using lower bar 0, which was already consumed.

### 23.0-D Chart open versus first lower-bar open

Source sha256 `3a621fca5b698e17bea36f2bee21f27afae636554ec10431b1f1b5054ace6671`.

```pine
//@version=6
strategy("23.0 first open mismatch", overlay=false, initial_capital=100000, use_bar_magnifier=true)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1)
```

Chart bar 1: time 2_000_000, O=10, H=12, L=8, C=11.
First lower bar: time 2_000_000, O=10.8, H=11.0, L=10.6, C=10.9.

Locked expectation:

- the pending market entry fills at **10.8**, not at the chart open 10;
- there is no second market-open fill at 10;
- script-visible `open` on bar 1 remains 10;
- public fill `time` remains 2_000_000 and `bar_index` remains 1.

### 23.0-E Trailing ratchet across lower bars

Source sha256 `0da8d716d0d37d736aaf9d3129a5f3300788677fe478e90e4c1f7d5767054a19`.

```pine
//@version=6
strategy("23.0 trailing ratchet", overlay=false, initial_capital=100000, use_bar_magnifier=true)

if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.2)
    strategy.exit("TR", "EN", trail_price=10.4, trail_offset=2)
```

Lower bars for chart bar 1, tick size 0.1 so `trail_offset=2` is 0.2:

| Lower | Time | Open | High | Low | Close |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 2_000_000 | 10.0 | 10.3 | 9.9 | 10.2 |
| 1 | 2_300_000 | 10.2 | 10.6 | 10.1 | 10.5 |
| 2 | 2_600_000 | 10.5 | 10.5 | 10.1 | 10.2 |

Locked expectation:

- EN activates on lower bar 0 after 10.2 is reached;
- trailing activation and ratchet follow existing Stage 14/18g trailing
  rules, but the observed highs come from successive lower bars;
- the ratchet is monotonic across host-bar boundaries;
- a later lower low does not unwind an earlier ratchet;
- public fill identity stays the chart bar.

### 23.0-F Forming realtime host contract

This case is a host-input rule, not a Pine script observation.

- Seed confirmed bars 0..N-1 with a validated magnifier manifest covering
  only those indexes.
- A forming update at chart index N must ignore the historical manifest and
  use the supplied forming bar.
- A manifest group with `chartBarIndex >= N` at seed time is a fail-closed
  structural error before execution.
- Replacing the forming bar must not resurrect historical lower bars.

## Implementation Constraints Carried Forward

1. One broker, one candidate selector, one fill per arbitration cycle.
2. Each host bar uses `HistoricalPath::from_validated_bar`.
3. Cursor is monotonic in host-bar index, point/leg phase, leg index, and
   mark.
4. `process_orders_on_close` is a chart-bar close phase, not a close after
   every lower bar.
5. Invalid magnifier input fails before bar zero.
6. Setting false or omitted remains byte-identical to the current baseline.
7. CLI, Python, and WASM share MagnifierInputV1 (`schemaVersion` 1,
   zero-based `chartBars`).
8. `calc_on_every_history_tick` stays rejected.

## Stop Conditions Rechecked At Lock Time

- Lower-bar OHLC as chart context is **not** required.
- Magnifier-local next-open gap handling does **not** require a general
  chart-to-chart gap rewrite.
- No new public result field is required.
- Event time is representable in existing fields under the chart-bar-time
  project rule.

Slice 23.1 may begin. It must keep `use_bar_magnifier=true` fail-closed.
