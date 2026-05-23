# Realtime Model

This document defines the realtime bar model before implementation of rollback
semantics.

The historical runtime executes only closed bars. Realtime execution adds the
concept of a forming bar: the latest bar may be evaluated multiple times before
it is confirmed.

## Bar Update Kinds

The runtime model exposes three update kinds:

```rust
enum BarUpdateKind {
    Historical,
    Forming,
    Confirmed,
}
```

`Historical` means a closed bar loaded from history. It is equivalent to the
existing `HistoricalRuntime::append_bar` behavior.

`Forming` means an intrabar update for the current open bar. A forming update
must not commit series history. Each new forming update discards effects from
the previous forming update before re-executing the script.

`Confirmed` means the final update for a realtime bar. Confirmed updates execute
like forming updates, then commit current series values to historical buffers.

## State Partitions

Realtime execution needs explicit state partitions:

- committed series history
- current update values
- output side effects for committed bars
- temporary output side effects for the forming bar
- persistent `var` state
- future intrabar `varip` state
- callsite state for TA functions
- immutable request provider data
- deterministic request result cache

Rollback semantics must specify which partition is restored and which partition
survives repeated forming updates.

## Commit Rules

Only these updates commit series:

- `Historical`
- `Confirmed`

`Forming` updates are visible in the current evaluation but not visible through
positive history references on later evaluations until the bar is confirmed.

## Runtime API

Rollback execution is exposed through `RealtimeRuntime`:

```rust
let mut runtime = RealtimeRuntime::new(&hir);
runtime.update(BarUpdate::historical(bar))?;
runtime.update(BarUpdate::forming(partial_bar))?;
runtime.update(BarUpdate::forming(updated_partial_bar))?;
runtime.update(BarUpdate::confirmed(final_bar))?;
```

`RealtimeRuntime` internally keeps:

- a confirmed `HistoricalRuntime` snapshot
- an optional forming `HistoricalRuntime` snapshot

Each forming update starts from the confirmed snapshot. This rolls back:

- current update values
- uncommitted series values
- temporary output side effects
- `var` updates made during the previous forming execution
- callsite state changes made during the previous forming execution
- array storage mutations made during the previous forming execution
- label, line, box, and table creation, mutation, deletion, and cell snapshots
  made during the previous forming execution
- request cache entries and requested-context runtime state created during the
  previous forming execution

Request provider data is immutable and shared through the runtime request
environment. Repeated forming updates may reuse the same provider object, but
requested-context evaluation and cache population are part of the runtime state
that rolls back with the forming snapshot. This keeps provider-backed
`request.security` deterministic across historical, forming, and confirmed
updates.

Confirmed and historical updates replace the confirmed snapshot and clear the
forming snapshot.

## `var` and `varip`

`var` is supported under rollback. A `var` update made during a forming update
is temporary; the next forming update starts again from the last confirmed
snapshot. A confirmed update persists the new `var` value.

Scalar `varip` declarations are supported as a separate intrabar
persistence path. The first forming update for a bar starts from the confirmed
snapshot. Later forming updates for that same bar seed scalar `varip`
slots from the previous forming update while keeping ordinary `var`, arrays,
drawing objects, outputs, request caches, callsite state, and history reads on
the confirmed rollback path. A confirmed update also seeds from the latest
forming `varip` slots before executing, then stores the resulting values in the
confirmed snapshot for the next bar.

Historical execution treats the supported scalar `varip` subset like `var`
because historical bars have one committed evaluation. Local declaration sites
inside `if`, `for`, `while`, and UDF bodies initialize only when first reached;
each lowered UDF callsite has independent storage. Arrays held by `varip`,
drawing object ids, tuples, and other non-scalar value families remain rejected
with compatibility diagnostics instead of being approximated.

## Current Status

Phase 7 now defines the model and implements rollback for repeated forming
updates. Realtime fixtures cover temporary output rollback, drawing-object
lifecycle rollback for labels, lines, boxes, and tables, `var` rollback,
scalar `varip` intrabar persistence, stateful TA callsite rollback inside
conditional branches, array rollback, request provider immutability and cache
rollback, and dynamic history reads from confirmed history during forming
updates.

Next work:

- broaden realtime fixtures for more stateful built-ins and nested scopes
