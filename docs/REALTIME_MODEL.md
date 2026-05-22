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
- label creation, mutation, and deletion snapshots made during the previous
  forming execution

Confirmed and historical updates replace the confirmed snapshot and clear the
forming snapshot.

## `var` and `varip`

`var` is supported under rollback. A `var` update made during a forming update
is temporary; the next forming update starts again from the last confirmed
snapshot. A confirmed update persists the new `var` value.

`varip` is rejected. It requires state that persists across repeated intrabar
updates while still interacting correctly with bar confirmation and historical
series commits. That is a separate state partition from `var`, and it is not
implemented yet.

The semantic analyzer rejects `varip` before HIR lowering with a compatibility
diagnostic. This avoids silently approximating `varip` with `var`, which would
produce incorrect realtime behavior.

## Current Status

Phase 7 now defines the model and implements rollback for repeated forming
updates. Realtime fixtures cover temporary output rollback, `var` rollback,
stateful TA callsite rollback inside conditional branches, array rollback, and
dynamic history reads from confirmed history during forming updates. `varip`
remains rejected until its intrabar persistence semantics are implemented
precisely.

Next work:

- broaden realtime fixtures for more stateful built-ins and nested scopes
