# Phase E Polyline Gate

Phase E originally kept `polyline.*` unsupported. The first post-gate slice now
supports `polyline.new` only: official Pine polylines are object ids created
from an `array<chart.point>` input, so support starts with the point-list
creation path rather than ad hoc scalar overloads.

The supported drawing families now have runtime-owned scalar ids and
host-neutral snapshots:

- `label.*`
- `line.*`
- `box.*`
- `table.*`
- `polyline.new`

`polyline.new` needs a point-list input model. Pine expresses that through
`chart.point` objects and arrays of points. The official creation form is:

```pine
polyline.new(points, curved, closed, xloc, line_color, fill_color, line_style, line_width, force_overlay)
```

The `points` argument is an array of `chart.point` values. Each point carries
`index`, `time`, and `price` information, and `xloc` decides whether the
polyline uses the point `index` or `time` fields for x coordinates. The
creation call copies a point list into a runtime-owned drawing. Official
polylines also expose only creation, deletion, and the read-only `polyline.all`
collection; they do not have setter or getter methods, so redraws happen by
deleting and creating a new polyline.

This runtime now has fixture-backed `chart.point` constructor, field read,
top-level field mutation, and `array.new<chart.point>()` plus
`array.from(chart.point, ...)` point-array storage/read/mutation support.
`polyline.new` now copies those point arrays into host-neutral `polylines`
snapshots in runtime `schemaVersion: 7`. Deletion, `.all`, realtime rollback
fixtures, and polyline arrays remain outside this slice.

Adding a narrow `polyline.new(na)` or ad hoc tuple-based point list would create
a different language surface from Pine and would bypass the array/type model
that conformance relies on. The implementation order is therefore:

1. `polyline.new` slice: done.
   add runtime-owned polyline ids and host-neutral snapshots containing the
   copied point list, `curved`, `closed`, `xloc`, `line_color`, `fill_color`,
   `line_style`, `line_width`, and `force_overlay`.
2. Lifecycle slice:
   add `polyline.delete` and `polyline.all`, including deletion snapshots,
   active-object filtering, max-count behavior, and realtime rollback coverage.
3. Release-contract slice: done for creation snapshots.
   expose the same snapshot shape through CLI, Python, and WASM hosts, then add
   conformance rows and matrix gates for the partial polyline claim.

The runtime must not mark broader `polyline.*` supported before lifecycle
slices are fixture-backed. `array.new_polyline` also remains out of scope until
polyline ids have deletion, `.all`, and rollback behavior.

`polyline.new` is now a partial conformance row backed by
`tests/fixtures/runtime/polyline_new.pine`. Broader `polyline.*` remains an
explicit unsupported conformance row backed by
`tests/fixtures/sema/unsupported_polyline.pine`, with `polyline.all` collection
coverage backed by `tests/fixtures/sema/unsupported_polyline_all.pine`.
