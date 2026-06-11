# Strategy Realtime Alert Delivery Plan

Status: design gate closed on 2026-06-11.

This document defines the realtime host loop and delivery-sink boundary for
future running-alert delivery. It does not implement webhook, email, push,
broker, persistence, scheduling, UI, or network delivery.

## Background

The runtime now exposes deterministic strategy order-fill events under
`strategy.alerts`, and the host wrappers can render a running-alert message from
one immutable `RunningAlertConfig` and one public strategy fill event.

Official Pine behavior keeps these concerns separate:

- Pine code and broker fills create alert events.
- Users create running alerts through the host, using a snapshot of the script
  and chart context.
- Alerts trigger only on realtime bars.
- Strategy script alerts can include `alert()` events, order-fill events, or
  both.
- `{{strategy.order.alert_message}}` is substituted from the order-generating
  strategy call's `alert_message` argument in the host alert message.

Sources:

- TradingView Pine Script alerts documentation:
  <https://www.tradingview.com/pine-script-docs/concepts/alerts/>
- TradingView Pine Script strategies documentation:
  <https://www.tradingview.com/pine-script-docs/concepts/strategies/>

## Current Repository Boundary

Already supported:

- deterministic historical `alerts[]` events for the fixture-backed
  `alert()`/`alertcondition()` subset;
- deterministic historical `strategy.alerts[]` order-fill payloads for the
  supported strategy subset;
- host-side rendering of
  `{{strategy.order.alert_message}}` through Rust, Python, CLI, and WASM helper
  paths.

Still unsupported:

- external alert delivery;
- persistent running-alert management;
- Pine-source placeholder interpolation beyond the existing rejected subset;
- realtime strategy broker execution and tick-level strategy scheduling;
- combining indicator `alert()` events and strategy order-fill events into one
  host event envelope.

## Decision

Future delivery must be host-owned. The core runtime should keep producing
deterministic events and should not send messages, own network clients, persist
alert state, or know about user delivery settings.

The first implementation should introduce a host-facing realtime loop with this
conceptual shape:

```text
RunningAlertRuntime
  input: immutable script snapshot + RunningAlertConfig + realtime bar updates
  state: last delivered event keys, last confirmed bar cursor, host diagnostics
  output: DeliveryCandidate values passed to a host-provided sink
```

`RunningAlertRuntime` is a host adapter, not a new Pine language feature. It may
live beside CLI/Python/WASM host code or behind an explicit host API, but it
must not change default historical `run` output.

## Host Snapshot

The host must create a snapshot before starting delivery. The snapshot includes:

- script source and libraries;
- inputs and compile/analyze result;
- chart symbol and timeframe;
- request data bindings;
- runtime profile options that affect event generation;
- the immutable `RunningAlertConfig`.

Changing any of these requires a new snapshot and a new running alert. The
delivery loop must not read mutable editor state while evaluating an already
running alert.

## Realtime Loop

The loop accepts realtime bar updates from the host. A future API should make
the update policy explicit:

- historical backfill is analysis/test data, not delivery data;
- unconfirmed forming-bar updates may be evaluated for local preview/debug, but
  delivery must only happen through an explicitly realtime path;
- confirmed bar-close delivery must respect existing alert frequency and
  rollback rules;
- strategy order-fill delivery must wait until realtime strategy execution has a
  supported event source. Until then, strategy running-alert delivery remains a
  design boundary over public `strategy.alerts` events, not a live broker loop.

The first executable slice should prefer a bar-close-only realtime host loop
unless a separate design proves tick and `calc_on_every_tick` parity.

## Event Selection

Delivery candidates should be selected from existing event streams:

```text
indicatorAlertCalls -> top-level alerts[]
strategyOrderFills -> strategy.alerts[]
both                -> one shared host event envelope
```

The current runtime helper supports `strategyOrderFills` only. `both` must
remain design-only until indicator alert calls and strategy order fills share a
single host envelope with stable event kind, bar identity, message template, and
dedupe fields.

## Delivery Candidate

A future delivery candidate should contain only host-safe data:

```text
DeliveryCandidate
  runningAlertId: string
  scriptSnapshotId: string
  eventKind: indicatorAlertCall | strategyOrderFill
  barIndex: usize
  time: i64
  eventId: string
  renderedMessage: string
```

The candidate is not public runtime JSON. It is an explicit host delivery value.
Adding it must not bump `RuntimeResult` schema versions unless it is embedded in
default runtime output, which this design forbids.

## De-Duplication

The host loop must prevent repeated delivery for the same running alert and
event. The initial dedupe key should include:

```text
runningAlertId + scriptSnapshotId + eventKind + barIndex + time + eventId
```

The implementation must store delivered keys outside the core runtime. If the
host restarts without persisted delivery state, at-most-once delivery cannot be
claimed.

## Delivery Sink

The delivery sink is a host-provided interface:

```text
DeliverySink::deliver(candidate) -> DeliveryResult
```

The sink may write to a test collector, local log, webhook sender, or another
host system. The runtime and interpreter crates must not depend on any concrete
network delivery implementation.

Delivery failures are host diagnostics. They must not change Pine semantic
diagnostics or mutate runtime event output. Retrying, backoff, authentication,
rate limiting, and dead-letter queues are host-delivery concerns and need a
separate implementation plan before support is claimed.

## Non-Goals

- Do not add webhooks, email, push, SMS, broker orders, or HTTP clients.
- Do not add running-alert persistence or UI state.
- Do not replay historical backtests as delivered realtime alerts by default.
- Do not change `RuntimeResult`, `alerts[]`, `strategy.alerts[]`, CLI JSON,
  Python dictionaries, or WASM runtime JSON.
- Do not support `both` until a shared event envelope exists.
- Do not claim realtime strategy alert delivery until realtime strategy
  execution itself is fixture-backed.

## Implementation Slices

1. Closed on 2026-06-11: this design gate.
2. Closed on 2026-06-11: add a pure `DeliveryCandidate` model, dedupe key, and
   in-memory test sink without network delivery or public runtime JSON changes.
3. Closed on 2026-06-11: add a host-only strategy order-fill candidate builder
   over `RunningAlertConfig` plus public `strategy.alerts` events. This remains
   a test/debug replay helper unless wired to a realtime event source.
4. Design the shared event envelope for `indicatorAlertCalls` and
   `strategyOrderFills`; only then consider enabling `both`.
5. Design concrete external delivery adapters separately, including
   persistence, retries, authentication, and failure reporting.

## Completion Gate

This design gate is closed when:

- realtime delivery is explicitly host-owned;
- snapshot immutability is documented;
- historical replay is not treated as live delivery;
- delivery candidates and dedupe keys are separate from public runtime JSON;
- network delivery remains out of scope;
- later implementation slices are narrow and testable.
