# Array Stage Audit

This audit closes the current scalar typed-array expansion pass. It records what
the runtime now claims, what remains intentionally out of scope, and what should
happen before moving to another language phase.

Primary references:

- TradingView Pine Script arrays documentation:
  <https://www.tradingview.com/pine-script-docs/language/arrays/>
- TradingView Pine Script methods documentation:
  <https://www.tradingview.com/pine-script-docs/language/methods/>
- Local conformance matrix source:
  `tests/fixtures/conformance.tsv`

## Stage Verdict

Stage 3 arrays are complete for the current fixture-backed scalar subset. Later
compatibility slices added fixture-backed `array.new_label` label-id arrays,
`array.new_line` line-id arrays, and `array.new_box` box-id arrays on top of
that scalar baseline without opening other drawing array families.

The project should keep `array.*` marked `partial`, not `supported`, because the
current implementation deliberately excludes generic arrays, object arrays, UDT
arrays, maps, matrices, `varip`, Pine's shallow slice/window semantics, and
several advanced sorting forms.

The next implementation work should not continue adding random array helpers.
Future array work should be chosen from the explicit gap list below and should
usually be paired with a larger language phase, such as object systems, user
types, or history/series semantics.

## Implemented Subset

Runtime model:

- Array values are runtime-owned ids stored in `PineValue::Array`.
- Assignment and UDF argument binding pass array ids by reference.
- `array.copy` is the explicit boundary for creating an independent array id.
- Non-`var` declarations allocate when they execute.
- `var` declarations preserve array ids and backing storage across bars.
- Realtime forming-bar rollback covers array state.
- Runtime array growth is guarded by the 100,000 element limit.

Element kinds:

- `float`
- `int`
- `bool`
- `string`
- `color`
- `label` ids
- `line` ids
- `box` ids

Creation and inference:

- `array.new_float`
- `array.new_int`
- `array.new_bool`
- `array.new_string`
- `array.new_color`
- `array.new_label`
- `array.new_line`
- `array.new_box`
- `array.from`

General operations:

- `array.size`
- `array.get`
- `array.set`
- `array.insert`
- `array.push`
- `array.pop`
- `array.remove`
- `array.shift`
- `array.unshift`
- `array.fill`
- `array.first`
- `array.last`
- `array.copy`
- `array.slice`
- `array.concat`
- `array.clear`

Search, predicate, and ordering helpers:

- `array.includes`
- `array.indexof`
- `array.lastindexof`
- `array.every`
- `array.some`
- `array.binary_search`
- `array.binary_search_leftmost`
- `array.binary_search_rightmost`
- `array.sort`
- `array.sort_indices`
- `array.reverse`

Numeric helpers:

- `array.abs`
- `array.min`
- `array.max`
- `array.sum`
- `array.avg`
- `array.range`
- `array.median`
- `array.mode`
- `array.percentile_nearest_rank`
- `array.percentile_linear_interpolation`
- `array.percentrank`
- `array.covariance`
- `array.standardize`
- `array.variance`
- `array.stdev`

String conversion:

- `array.join`

Method syntax:

- Supported array functions lower to the same `array.*` runtime calls where
  listed in `tests/fixtures/conformance.tsv`.
- Method syntax is supported for the scalar typed-array subset and line-id
  arrays where listed in `tests/fixtures/conformance.tsv`.

## Known Gaps

These gaps are intentional. Do not mark `array.*` broadly supported until they
are designed and fixture-backed.

Generic arrays:

- `array.new<type>()` is not supported.
- Type-template array declarations such as `array<float>` are not a general
  parser or semantic feature in the current subset.
- `array.from` only infers the scalar element kinds and drawing ids listed
  above.

Reference and object arrays:

- Arrays of `linefill`, `table`, `polyline`, and other drawing/object ids are
  not supported. Label-id, line-id, and box-id arrays are the only
  fixture-backed drawing-object array families.
- Additional drawing-object arrays should wait for explicit object id lifetime,
  rollback, and host-output semantics.

User-defined type arrays:

- UDT declarations and object field access are not supported.
- Sorting UDT arrays by `sort_field` is not supported.
- UDT arrays should wait for the user-defined type and method-dispatch phase.

Maps and matrices:

- `matrix.*` and `map.*` are out of scope for this array stage.
- They need separate storage models, type rules, and conformance fixtures.

`varip`:

- `varip` arrays are not supported because intrabar persistence is still
  rejected.
- This belongs to the dedicated `varip` phase.

History and snapshots:

- Scalar dynamic integer history offsets are supported, but array history
  behavior and historical array snapshots have not been designed.
- Any future support must define storage retention and aliasing rules.

Slice semantics:

- The current `array.slice` implementation returns a same-kind copied array for
  the requested window.
- Pine documents slice as shallow window-like behavior over the parent array.
- Treat this as a known compatibility limitation. A future fix needs a design
  for shared backing storage, mutation mirroring, bounds invalidation, rollback,
  and incremental execution.

Loops over arrays:

- `for...in` array iteration is not part of the current loop subset.
- This should be handled in a loop hardening or language syntax phase.

Advanced sorting:

- `array.sort` and `array.sort_indices` support scalar `float`, `int`, and
  `string` arrays with `order.ascending` and `order.descending`.
- `sort_field` for UDT arrays is not supported.
- Sorting object arrays is not supported.

Unsupported helpers and variants:

- Any `array.*` function absent from `tests/fixtures/conformance.tsv` remains
  unsupported.
- Any supported helper called on unsupported element kinds should remain a
  semantic error, not a runtime approximation.

## Recommended Next Phase

The best next step is to leave Stage 3 arrays and choose one of these tracks:

1. Phase C, history and series semantics.
   This is the most foundational path. It would address dynamic history
   offsets, first-bar behavior, qualifier propagation, and array history
   boundaries.

2. Phase D, built-in coverage expansion.
   This is the highest user-visible compatibility path. Start with pure
   built-ins, then stateful `ta.*`, then output options that affect public
   result schemas.

3. Phase A residual hardening.
   If stability is preferred over coverage, add more real-script loop and
   branch interaction fixtures before broadening the language.

Do not start matrices, maps, drawing objects, `request.*`, or strategy runtime
without a dedicated design document. Each of those introduces a new runtime
storage or host integration model.

## Array Follow-Up Backlog

Only take these when they are explicitly selected as the next work item:

- Design Pine-compatible `array.slice` shallow window semantics.
- Design generic `array.new<type>()` parsing and type checking.
- Design array history snapshots and aliasing behavior.
- Add `for...in` array iteration syntax and runtime behavior.
- Add object arrays after drawing object ids exist.
- Add UDT arrays and `sort_field` after user-defined types exist.
- Expand diagnostics for unsupported generic/object/UDT array syntax once those
  syntaxes are parsed precisely.

## Exit Criteria Met For Current Subset

- Syntax, semantic analysis, runtime behavior, conformance metadata, and docs
  agree for every claimed array feature.
- Historical and incremental fixture execution match for runtime fixtures.
- Realtime rollback covers array state.
- UDF side-effect boundaries reject array mutation inside functions.
- Unsupported collection families remain diagnostic-only.
- `array.*` remains `partial` in conformance metadata.
