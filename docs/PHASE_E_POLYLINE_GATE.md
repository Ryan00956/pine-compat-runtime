# Phase E Polyline Gate

Phase E keeps `polyline.*` unsupported.

The supported drawing families now have runtime-owned scalar ids and
host-neutral snapshots:

- `label.*`
- `line.*`
- `box.*`
- `table.*`

`polyline.new` needs a point-list input model. Pine expresses that through
point objects and arrays of points. This runtime currently has no `chart.point`
value kind, no point snapshot schema, and no object or generic array support
that can safely carry point lists through semantic analysis, historical
execution, incremental append, and realtime rollback.

Adding a narrow `polyline.new(na)` or ad hoc tuple-based point list would create
a different language surface from Pine and would bypass the array/type model
that conformance relies on. The correct implementation gate is therefore a
future point-object design:

1. Add `chart.point` value semantics.
2. Add typed point arrays or an equivalent fixture-backed point-list carrier.
3. Define polyline snapshot size limits and rollback behavior.
4. Then add `polyline.new` fixtures and matrix rows as partial support.

Until then, `polyline.*` remains an explicit unsupported conformance row backed
by `tests/fixtures/sema/unsupported_drawing.pine`.
