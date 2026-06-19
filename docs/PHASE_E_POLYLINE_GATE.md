# Phase E Polyline Gate

Phase E keeps `polyline.*` unsupported. This gate is intentional: official
Pine polylines are not scalar drawing calls, they are object ids created from
an `array<chart.point>` input.

The supported drawing families now have runtime-owned scalar ids and
host-neutral snapshots:

- `label.*`
- `line.*`
- `box.*`
- `table.*`

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

This runtime currently has no `chart.point` value kind, no point snapshot
schema, and no typed point-array carrier that can safely carry point lists
through semantic analysis, historical execution, incremental append, and
realtime rollback.

Adding a narrow `polyline.new(na)` or ad hoc tuple-based point list would create
a different language surface from Pine and would bypass the array/type model
that conformance relies on. The implementation order is therefore:

1. `chart.point` design slice:
   add the point value model and constructor semantics for
   `chart.point.now`, `chart.point.from_index`, `chart.point.from_time`, and
   compatible field access/mutation for `index`, `time`, and `price`.
2. Point-list array slice:
   add `array.new<chart.point>()`, `array.from(chart.point, ...)`, and the
   existing array mutation/read subset needed to build fixture-backed point
   lists without widening unrelated numeric, sort, or join array behavior.
3. `polyline.new` slice:
   add runtime-owned polyline ids and host-neutral snapshots containing the
   copied point list, `curved`, `closed`, `xloc`, `line_color`, `fill_color`,
   `line_style`, `line_width`, and `force_overlay`.
4. Lifecycle slice:
   add `polyline.delete` and `polyline.all`, including deletion snapshots,
   active-object filtering, max-count behavior, and realtime rollback coverage.
5. Release-contract slice:
   expose the same snapshot shape through CLI, Python, and WASM hosts, then add
   conformance rows and matrix gates for the partial polyline claim.

The runtime must not mark `polyline.*` supported before the point value and
point-array slices are fixture-backed. `array.new_polyline` also remains out of
scope until polyline ids exist; point arrays are the prerequisite for
`polyline.new`.

Until then, `polyline.*` remains an explicit unsupported conformance row backed
by `tests/fixtures/sema/unsupported_polyline.pine`, with `polyline.all`
collection coverage backed by
`tests/fixtures/sema/unsupported_polyline_all.pine`.
