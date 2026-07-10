# Pure Internal UDT Array Design Gate

Status: closed design gate, maintained as the current UDT array support
boundary.

This document defines the first internal path for future arrays of local
user-defined type values. It is scoped to interpreter internals only: parser,
semantic analysis, HIR lowering, runtime array storage, history, rollback, and
conformance. It does not cover host UI, rendering, remote data, imported UDT
identity, or public JSON/Python/WASM serialization of UDT array values.

## Current Boundary

UDT arrays are intentionally narrow today. The current fixture-backed local
subset is same-local scalar-tree `array.new<T>()` construction, optional same-UDT initial
values, and `array.from(...)` inference with `array.size`/`size()`,
`array.get`/`get()` field reads, same-UDT `array.set`/`set()` replacement,
same-UDT `array.push`/`push()` append, `array.pop`/`pop()` returns, and
`array.shift`/`shift()` returns, plus `array.first`/`first()` and
`array.last`/`last()` reads, `array.clear`/`clear()` reset/reuse, and
`array.copy`/`copy()` independence, `array.concat`/`concat()` same-UDT append,
`array.slice`/`slice()` parent-window read/write mirroring, and
`array.reverse`/`reverse()` reordering, plus `array.insert`/`insert()`
same-UDT insertion, `array.remove`/`remove()` returns,
`array.unshift`/`unshift()` same-UDT prepend, and `array.sort`/`sort()` by a
compile-time `int`, `float`, or `string` `sort_field`, and
`array.sort_indices`/`sort_indices()` by the same `sort_field` subset returning
original indexes without mutating the source array, plus `array.includes`,
`array.indexof`, and `array.lastindexof` by same-local scalar-tree UDT
structural equality, plus `array.fill`/`fill()` same-UDT replacement over valid
half-open ranges, plus `array.join`/`join()` positional UDT stringification
using `TypeName(field0, field1, ...)` with nested local UDT fields formatted
recursively, plus `array<T>` and `T[]` typed
declarations for the same local scalar-tree UDT array subset. UDT values read
from those arrays can be locally field-mutated without changing the source
array slot until an explicit same-UDT `array.set`/`set()` writeback, can be
passed to local pure UDFs, and when bound to local variables can call local pure
UDT methods. Direct chained slot mutation is supported for
`array.get(points, index).field := value` and `points.get(index).field := value`
when `points` is a same-local scalar-tree UDT array; it rewrites the selected
slot by value and mirrors through slices. Chained UDT array field mutation inside
UDFs remains rejected by the existing function side-effect policy. UDT-specific
numeric/statistical helpers, binary search, nested collections, and unresolved
or recursive UDT element families remain unsupported. Imported UDT arrays include
the fixture-backed same-imported scalar-tree `array.new<lib.Type>()`,
`array.from`, and typed-array subsets.
Ordinary `var` UDT arrays roll back to the
confirmed backing store during realtime forming updates, while same-local and
same-imported scalar-tree `varip` UDT arrays initialized through `array.from`
or `array.new<T>`/`array.new<lib.Type>` retain their backing stores intrabar.

Current evidence:

- `docs/CONFORMANCE.md` keeps UDT array helpers outside the explicitly
  fixture-backed UDT array subset except for the helpers listed here.
- `docs/ARRAY_STAGE_AUDIT.md` is historical for this area; newer conformance and
  this document are the authority for the now fixture-backed UDT `sort_field`
  subset.
- `docs/PHASE_J_AUDIT.md` is historical for this area; current conformance and
  this document keep nested UDT fields, recursive UDTs, non-scalar imported UDT
  arrays, and imported UDT array helpers outside the fixture-backed
  same-imported scalar-tree `array.from`/typed-array subset unsupported after
  the local scalar-tree UDT phase.
- `tests/fixtures/sema/supported_user_type_array_decl.pine` and
  `tests/fixtures/sema/supported_user_type_array_alias_decl.pine` accept typed
  UDT array declarations whose element UDT has local scalar-tree fields.
  `tests/fixtures/sema/unsupported_user_type_array_from_decl.pine` keeps
  mismatched UDT array initialization rejected,
  `tests/fixtures/sema/supported_user_type_array_varip_decl.pine` accepts
  same-local scalar-tree UDT array `varip`, and
  `tests/fixtures/runtime/user_type_array_varip.pine` and
  `tests/fixtures/realtime/user_type_array_varip.pine` cover same-local nested
  scalar-tree UDT array `varip` backing-store handoff, while
  `tests/fixtures/sema/supported_user_type_array_varip_nested_decl.pine` keeps
  the nested declaration boundary accepted.
  `tests/fixtures/runtime/import_udt_array_typed_declarations.pine` and
  `tests/fixtures/runtime/import_udt_array_scalar_tree.pine` cover
  same-imported scalar-tree UDT array template and alias declarations, while
  `tests/fixtures/sema/supported_imported_udt_array_decl.pine` and
  `tests/fixtures/sema/supported_imported_udt_array_alias_decl.pine` keep the
  semantic acceptance boundary fixture-backed.
  `tests/fixtures/runtime/import_udt_array_from.pine` covers same-imported
  scalar-field UDT `array.from` construction with size/get/first/last field
  reads, set replacement field reads, push append field reads, unshift prepend
  field reads, insert insertion field reads, fill replacement field reads,
  join positional stringification, includes/indexof/lastindexof structural
  equality search, sort/sort_indices by float root `sort_field`,
  pop/remove/shift return field reads, clear/clear() size reset, copy
  independent field reads, reverse reordered field reads, slice window field
  reads, concat appended field reads, and statement/expression/index-value
  for-in value-copy field reads, while
  `tests/fixtures/runtime/import_udt_array_scalar_tree.pine` covers the nested
  same-imported scalar-tree subset for typed declarations, `array.from`,
  `array.get`, `array.set`, `array.copy`, `array.push`, `array.unshift`,
  `array.insert`, `array.pop`, `array.remove`, `array.shift`, `array.first`,
  `array.last`, `array.clear`, `array.fill`, `array.reverse`, `array.slice`,
  `array.concat`, `array.join`, structural equality search,
  statement/index-value `for...in`, and `varip` backing-store handoff, while
  `tests/fixtures/runtime/import_udt_array_history.pine` covers committed
  imported UDT array history snapshots from both `array.from` and
  `array.new<lib.Type>()` construction, with first-bar and dynamic na-offset
  predicates.
  `crates/pine-sema/tests/fixtures.rs` and
  `crates/pine-sema/src/tests/compatibility.rs` assert those diagnostics.
- `tests/fixtures/runtime/array_new_udt.pine` covers the current
  `array.new<Point>()` expression subset, including direct `array.new<T>()`
  helper reads/mutations through size, get, set, push, insert, first, last,
  remove, shift, and pop, plus empty first/last/pop/shift `na` results.
  `tests/fixtures/sema/unsupported_array_new_unknown_udt.pine`,
  `tests/fixtures/sema/supported_array_new_nested_udt_field.pine`, and
  `tests/fixtures/sema/unsupported_array_new_mixed_udt_initial.pine` keep
  unknown UDTs and mismatched initial values rejected while nested local UDT
  fields are accepted.
- `tests/fixtures/syntax/unsupported_array_new_udt.pine` keeps deeply dotted
  `array.new<library.Type.Inner>()` templates rejected at the parser boundary, while
  `tests/fixtures/syntax/imported_udt_array_new.pine`,
  `tests/fixtures/sema/supported_imported_udt_array_new.pine`, and
  `tests/fixtures/runtime/import_udt_array_new.pine` cover same-imported
  scalar-tree UDT `array.new<lib.Type>()` construction plus post-construction
  mutation, returned-element, copy/window, search, join, sort, and clear helper
  smoke coverage.
- `tests/fixtures/sema/unsupported_array_concat_udt.pine` rejects concat between
  different local UDT array element identities, and
  `tests/fixtures/sema/unsupported_array_set_mixed_udt.pine`,
  `tests/fixtures/sema/unsupported_array_push_mixed_udt.pine`, and
  `tests/fixtures/sema/unsupported_array_insert_udt.pine` reject replacement,
  append, and insertion values from a different local UDT identity with
  fixture-backed expected and actual UDT names.
- `tests/fixtures/runtime/array_search_udt.pine` covers UDT
  `array.includes`, `array.indexof`, and `array.lastindexof` structural
  equality for the same local scalar-tree UDT, while
  `tests/fixtures/runtime/user_type_array_scalar_tree_helpers.pine` covers
  nested local UDT fields. `tests/fixtures/sema/unsupported_array_includes_udt.pine`,
  `tests/fixtures/sema/unsupported_array_indexof_udt.pine`, and
  `tests/fixtures/sema/unsupported_array_lastindexof_udt.pine` reject search
  values from a different local UDT identity.
- `tests/fixtures/runtime/array_fill_udt.pine` covers UDT `array.fill` and
  `fill()` replacement for the same local scalar-tree UDT, while
  `tests/fixtures/runtime/user_type_array_scalar_tree_helpers.pine` covers
  nested local UDT fields and
  `tests/fixtures/sema/unsupported_array_fill_udt.pine` rejects fill values
  from a different local UDT identity.
- `tests/fixtures/runtime/array_join_udt.pine` covers UDT `array.join` and
  `join()` using the local UDT name and field declaration order, while
  `tests/fixtures/runtime/user_type_array_scalar_tree_helpers.pine` covers
  recursive formatting of nested local UDT fields. General
  `str.tostring(UDT)` remains rejected by
  `tests/fixtures/sema/unsupported_str_tostring_udt.pine`.
- `tests/fixtures/runtime/user_type_array_method_values.pine` covers local pure
  UDT method calls on same-local scalar-tree UDT values read from UDT arrays
  into local variables.
- `tests/fixtures/runtime/user_type_array_udf_values.pine` covers local pure
  UDF calls that consume same-local scalar-tree UDT values read from UDT
  arrays and preserve UDT identity through passthrough or constructor returns.
- `tests/fixtures/runtime/array_sort_udt_field.pine` covers `array.sort` and
  `sort()` over same-local scalar-tree UDT arrays by root `int`, `float`, or
  `string` `sort_field`. `tests/fixtures/sema/unsupported_array_sort_udt.pine`,
  `tests/fixtures/sema/unsupported_array_sort_udt_unknown_field.pine`,
  `tests/fixtures/sema/unsupported_array_sort_udt_bool_field.pine`, and
  `tests/fixtures/sema/unsupported_array_sort_udt_dynamic_field.pine` keep
  missing, unknown, unsupported-type, and dynamic `sort_field` forms rejected
  with fixture-backed missing-field, scalar-tree field-family, and const-string
  qualifier diagnostics.
- `tests/fixtures/runtime/array_sort_indices_udt_field.pine` covers
  `array.sort_indices` and `sort_indices()` over same-local scalar-tree UDT
  arrays by root `int`, `float`, or `string` `sort_field`, returning original indexes
  without mutating the source array.
  `tests/fixtures/sema/unsupported_array_sort_indices_udt.pine`,
  `tests/fixtures/sema/unsupported_array_sort_indices_udt_unknown_field.pine`,
  `tests/fixtures/sema/unsupported_array_sort_indices_udt_bool_field.pine`, and
  `tests/fixtures/sema/unsupported_array_sort_indices_udt_dynamic_field.pine`
  keep missing, unknown, unsupported-type, and dynamic `sort_field` forms
  rejected with the same fixture-backed diagnostic families as `array.sort`.
- `tests/fixtures/runtime/import_udt_array_sort_field.pine` and
  `tests/fixtures/sema/supported_imported_udt_array_sort_field.pine` cover
  `array.sort`, `sort()`, `array.sort_indices`, and `sort_indices()` over
  same-imported scalar-tree UDT arrays by root `float`, `int`, and `string`
  `sort_field`, using imported type metadata to lower the compile-time field
  name to a stable field index.
  `tests/fixtures/sema/unsupported_imported_udt_array_sort_*field.pine` and
  `tests/fixtures/sema/unsupported_imported_udt_array_sort_indices_*field.pine`
  keep missing, unknown, unsupported bool, and dynamic imported `sort_field`
  forms on the same diagnostic boundary as local UDT arrays.
- `tests/fixtures/runtime/user_type_array_typed_declarations.pine` covers
  `array<T>` and `T[]` declarations for same-local scalar-tree UDT arrays,
  including `na` initialization, later same-UDT assignment, `array.new<T>()`,
  `array.from(...)`, `array.copy`, `array.push`, and `array.get` field reads.
  `tests/fixtures/sema/supported_user_type_array_decl.pine`,
  `tests/fixtures/sema/supported_user_type_array_alias_decl.pine`,
  `tests/fixtures/sema/unsupported_user_type_array_from_decl.pine`, and
  `tests/fixtures/sema/supported_user_type_array_varip_nested_decl.pine` keep
  mismatched UDT array assignment rejected and scalar-tree nested UDT array
  `varip` declarations accepted; `tests/fixtures/runtime/user_type_array_varip.pine`
  and `tests/fixtures/realtime/user_type_array_varip.pine` cover nested
  scalar-tree element backing-store handoff, while
  `tests/fixtures/sema/supported_user_type_array_varip_decl.pine` covers the
  same-local scalar-tree UDT array `varip` declaration subset.
- `tests/fixtures/sema/unsupported_array_unshift_udt.pine` rejects unshift
  values from a different local UDT identity.
- `tests/fixtures/runtime/user_type_array_writeback.pine` covers independent
  field mutation of UDT values read from arrays and explicit same-UDT
  `array.set`/`set()` writeback into the source slot, plus namespace-call,
  method-call, and slice-window chained field mutation writeback.
- `tests/fixtures/syntax/udt_array_chained_field_mutation.pine` parses direct
  chained UDT array slot field mutation syntax, while
  `tests/fixtures/sema/unsupported_udt_array_chained_field_mutation_udf.pine`
  keeps the UDF side-effect boundary rejected and
  `tests/fixtures/sema/unsupported_imported_udt_array_chained_field_mutation.pine`
  keeps imported UDT array slot field mutation rejected until imported writeback
  identity is designed.
- `crates/pine-sema/tests/fixtures.rs` also asserts the rejected UDT array helper
  diagnostics for the fixture-backed helper set above.

Do not widen UDT arrays until a runtime slice implements the behavior and
updates fixtures, conformance, snapshots, and docs together.

## Prerequisites

The first positive UDT array subset depends on already-stable local UDT identity.

Prerequisites before positive support:

- local scalar-tree UDT construction and field reads remain stable;
- local UDT typed variables, `var` persistence, UDF passthrough, and pure local
  methods remain stable for the existing fixture-backed subset;
- semantic analysis can name the concrete local UDT carried by an array element,
  not just `ValueKind::UserType`;
- lowering can preserve the UDT field layout needed by array helper calls;
- runtime array storage can clone, compare, and snapshot `PineValue::UserType`
  values without aliasing field vectors across array slots.

Imported UDT identity is not a prerequisite for local UDT arrays. It remains a
separate design gate because source-graph-wide type identity and method tables
change the meaning of assignment compatibility.

## Runtime Clone And Snapshot Audit

Current runtime value storage is close enough for a narrow local scalar-tree UDT
array slice, but the exact boundary must stay explicit:

- `crates/pine-runtime/src/value.rs` represents UDT values as
  `PineValue::UserType(Vec<PineValue>)` under a derived `Clone`. Cloning a UDT
  value clones the field vector and recursively clones nested `PineValue`
  fields.
- `crates/pine-runtime/src/runtime/historical.rs` stores array backing values in
  `array_store: HashMap<u32, Vec<PineValue>>`, with element families tracked
  separately in `array_kinds`.
- `crates/pine-runtime/src/builtins/arrays.rs::array_values_clone` clones full
  array backing vectors and slice windows before helpers such as copy, concat,
  statistics, and history snapshot creation consume them.
- `crates/pine-runtime/src/builtins/arrays/constructors.rs::new_array` seeds
  repeated initial values with `vec![initial_value; size]`, which clones each
  `PineValue` slot. That is sufficient for immutable scalar-field UDT values;
  it would be shallow for any future field that is itself an array id.
- `crates/pine-runtime/src/runtime/context.rs::commit_current_series` calls
  `clone_collection_history_value` before retaining array-valued series
  history, and
  `crates/pine-runtime/src/runtime/history.rs::clone_collection_history_value`
  creates a fresh runtime array id with cloned element values for both commit
  snapshots and positive-offset history reads.
- `crates/pine-runtime/src/runtime/realtime.rs` starts each forming update from a
  cloned confirmed `HistoricalRuntime`, and
  `crates/pine-runtime/src/runtime/persistence.rs::seed_intrabar_array_from`
  clones varip array backing values when carrying intrabar array state forward.

Implications for the first positive UDT array runtime slice:

- local scalar-tree UDT array elements can be stored as ordinary `PineValue`
  slots without sharing field vectors across array elements, `array.copy`, array
  history snapshots, or realtime rollback clones;
- the first positive subset must still add an explicit UDT array element kind or
  equivalent side table that preserves the concrete local UDT name for each
  array id;
- array fields, imported UDT fields, map fields, matrix fields, and other
  reference-like or source-crossing UDT field families must remain rejected for
  UDT array elements until their deep-copy, identity, and snapshot policy is
  designed;
- equality, ordering, `sort_field`, history mutation of copied snapshots, and
  `varip` UDT array handoff remain separate behavior slices even though the
  scalar-field value clone boundary is viable.

## Target Shape

The first positive UDT array subset should mirror existing array discipline:

- arrays remain runtime-owned ids;
- assignment passes array ids by reference;
- `array.copy()` is the explicit independent-copy boundary;
- array slots hold UDT values, not separate UDT object ids;
- writing a UDT value into an array slot stores an independent value snapshot;
- reading a UDT value from an array slot returns a value that can be field-read;
- mutating a field on a variable returned from `array.get` does not mutate the
  original array slot unless a later explicit write stores the changed value
  back with `array.set`;
- non-`var` declarations allocate when executed;
- `var` declarations preserve the array id and backing storage across bars;
- rollback restores UDT array backing storage to the confirmed snapshot;
- UDT array growth is bounded by the existing array limit and visible in runtime
  profiles.

The first runtime implementation should reuse the existing array id family, but
it must treat `PineValue::UserType(Vec<PineValue>)` as a structured value with a
known local UDT definition. Do not introduce a host-visible object identity for
UDT array elements in the first slice.

## Element Type Policy

First positive element family:

- one concrete local UDT whose fields are only `int`, `float`, `bool`, `string`,
  or `color`.

Deferred element families:

- nested UDT fields;
- recursive UDTs;
- imported UDTs;
- arrays, maps, or matrices inside UDT fields;
- tuples inside UDT fields;
- object ids and chart points inside UDT fields;
- strategy/order/trade records inside UDT fields;
- bare `array` declarations without a concrete UDT element type.

Rationale:

- local scalar-tree UDT values are already represented as runtime value vectors;
- one concrete local UDT element type keeps assignment diagnostics clear;
- nested and imported identity would otherwise force deep-copy, method-dispatch,
  and source-graph rules into the first array slice.

## Type Model

The semantic model must preserve the concrete UDT name attached to the array
element type.

Accepted forms currently stay narrow:

- `array.new<Point>()` / `array.new<Point>(size, Point.new(...))` for one local
  scalar-field UDT type
- `array.from(Point.new(...))` when every element is the same local UDT
- `array.from(lib.Point.new(...))` when every element is the same imported
  scalar-field UDT, currently fixture-backed for size/get/first/last plus
  pop/remove/shift return field reads, clear-size reset, and copy independent
  field reads plus reverse reordered field reads
- `array.get(points, index)` returning the same local UDT type
- `array.set(points, index, Point.new(...))`
- `array.push(points, Point.new(...))`
- `array.pop(points)` / `points.pop()` returning the same local UDT type
- `array.shift(points)` / `points.shift()` returning the same local UDT type
- `array.first(points)` / `points.first()` returning the same local UDT type
- `array.last(points)` / `points.last()` returning the same local UDT type
- `array.clear(points)` / `points.clear()` resetting the same local UDT array
  for later same-UDT reuse
- `array.copy(points)` / `points.copy()` returning an independent same local UDT
  array with field-readable copied elements
- `array.concat(points, morePoints)` / `points.concat(morePoints)` appending a
  same local UDT source array and returning the target array
- `array.slice(points, from, to)` / `points.slice(from, to)` returning a same
  local UDT parent-window slice with field-readable elements and read/write
  mirroring
- `array.reverse(points)` / `points.reverse()` reordering the same local UDT
  array while preserving field-readable elements
- `array.insert(points, index, Point.new(...))` / `points.insert(index,
  Point.new(...))` inserting a same local UDT value
- `array.remove(points, index)` / `points.remove(index)` removing and returning
  the same local UDT type
- `array.unshift(points, Point.new(...))` / `points.unshift(Point.new(...))`
  prepending a same local UDT value
- `array.fill(points, Point.new(...), from, to)` / `points.fill(Point.new(...))`
  replacing the whole same local UDT array or a valid half-open range with a
  same local UDT value
- `array.sort(points, order.ascending, "field")` / `points.sort(order.descending,
  "field")` sorting in place by a local scalar `int`, `float`, or `string`
  field
- `array.sort_indices(points, order.ascending, "field")` /
  `points.sort_indices(order.descending, "field")` returning original indexes
  by a local scalar `int`, `float`, or `string` field without mutating the
  source array
- `array.includes(points, Point.new(...))` / `points.includes(Point.new(...))`
  comparing every scalar field of the same local UDT value
- `array.indexof(points, Point.new(...))` / `points.indexof(Point.new(...))`
  returning the first structurally equal same local UDT value
- `array.lastindexof(points, Point.new(...))` /
  `points.lastindexof(Point.new(...))` returning the last structurally equal
  same local UDT value
- `array.join(points, separator)` / `points.join(separator)` returning each UDT
  element as `TypeName(field0, field1, ...)`, using existing scalar
  `array.join` formatting for field values and `NaN` for `na` elements
- `array<Point> points = na` / `Point[] points = array.from(Point.new(...))`
  declaring a same-local scalar-tree UDT array and preserving its UDT identity
  across later same-UDT assignment
- `var points = array.from(Point.new(...))` rolling back repeated realtime
  forming-bar mutations to the last confirmed UDT array backing store

Keep these forms unsupported in the first positive slice unless they are
fixture-backed together:

- bare `array` declarations initialized from UDT arrays;
- mixed local UDT element arrays;
- assignment between arrays of different UDT declarations with identical field
  shapes;
- imported UDT arrays beyond the same-imported scalar-tree `array.from` and
  typed-array fixture-backed helper subset;
- arrays of UDT values inferred across mixed source boundaries.

Shape equality is not type equality. Two UDT declarations with the same field
names and field types must remain incompatible unless a later design explicitly
adds structural typing.

## Runtime Operations

There is no remaining first-subset UDT array stringification helper after the
`array.join` slice. General `str.tostring(UDT)` and field-name-preserving UDT
formatting remain outside this subset.
`tests/fixtures/runtime/user_type_array_scalar_tree_helpers.pine` covers nested
local UDT formatting through `array.join` without adding a general
`str.tostring(UDT)` conversion.

Numeric helpers such as `array.sum`, `array.avg`, percentile helpers, and
boolean predicates do not become UDT helpers merely because UDT arrays exist.
They should continue to reject UDT arrays unless a later slice defines an
element projection.
`tests/fixtures/sema/unsupported_array_sum_udt.pine` covers this boundary for
`array.sum`, `tests/fixtures/sema/unsupported_array_abs_udt.pine` covers it for
`array.abs`, and `tests/fixtures/sema/unsupported_array_avg_udt.pine` covers it
for `array.avg`. `tests/fixtures/sema/unsupported_array_min_udt.pine` covers it
for `array.min`, and `tests/fixtures/sema/unsupported_array_max_udt.pine` covers
it for `array.max`. `tests/fixtures/sema/unsupported_array_range_udt.pine`
covers it for `array.range`, and
`tests/fixtures/sema/unsupported_array_median_udt.pine` covers it for
`array.median`. `tests/fixtures/sema/unsupported_array_mode_udt.pine` covers it
for `array.mode`.
`tests/fixtures/sema/unsupported_array_percentile_nearest_rank_udt.pine` covers
it for `array.percentile_nearest_rank`.
`tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_udt.pine`
covers it for `array.percentile_linear_interpolation`.
`tests/fixtures/sema/unsupported_array_percentrank_udt.pine` covers it for
`array.percentrank`.
`tests/fixtures/sema/unsupported_array_variance_udt.pine` covers it for
`array.variance`.
`tests/fixtures/sema/unsupported_array_stdev_udt.pine` covers it for
`array.stdev`.
`tests/fixtures/sema/unsupported_array_standardize_udt.pine` covers it for
`array.standardize`.
`tests/fixtures/sema/unsupported_array_covariance_udt.pine` covers it for
`array.covariance`. These fixtures now lock both the numeric-array receiver
expectation and the UDT array helper allow-list, so UDT array numeric and
statistical helpers cannot be accepted without an explicit element-projection
slice.
Search helpers use structural equality for same-local scalar-tree UDT values:
all fields must compare equal using the existing runtime value equality
relation, including int/float numeric equality and exact string/color/bool
equality. Different local UDT identities remain incompatible even when their
field shapes match. `tests/fixtures/runtime/array_search_udt.pine` covers
positive `array.includes`, `array.indexof`, and `array.lastindexof` behavior
for flat UDT values, while
`tests/fixtures/runtime/user_type_array_scalar_tree_helpers.pine` covers nested
local UDT values.
`tests/fixtures/sema/unsupported_array_includes_udt.pine`,
`tests/fixtures/sema/unsupported_array_indexof_udt.pine`, and
`tests/fixtures/sema/unsupported_array_lastindexof_udt.pine` cover mismatched
local UDT identities and lock the `value` argument diagnostic to the concrete
expected and actual UDT names, so same-shape UDT search values cannot be
accepted structurally.
Truthiness predicates also need an explicit UDT truthiness policy before
support. `tests/fixtures/sema/unsupported_array_every_udt.pine` covers this
boundary for `array.every`, and
`tests/fixtures/sema/unsupported_array_some_udt.pine` covers it for
`array.some`. These fixtures now lock both the numeric/bool-array receiver
expectation and the UDT array helper allow-list, so UDT array truthiness cannot
be accepted without an explicit truthiness-policy slice.
Binary-search helpers need an explicit UDT ordering policy before support.
`tests/fixtures/sema/unsupported_array_binary_search_udt.pine` covers this
boundary for `array.binary_search`, and
`tests/fixtures/sema/unsupported_array_binary_search_leftmost_udt.pine` covers
it for `array.binary_search_leftmost`.
`tests/fixtures/sema/unsupported_array_binary_search_rightmost_udt.pine` covers
it for `array.binary_search_rightmost`. These fixtures now lock both the
numeric-array receiver expectation and the UDT array helper allow-list, so UDT
array binary search cannot be accepted without an explicit ordering-policy
slice.

## Sorting And `sort_field`

`array.sort` and `array.sort_indices` are fixture-backed for UDT arrays only
when a compile-time `sort_field` names a sortable same-local or same-imported
scalar field.

Initial `sort_field` policy:

- the field name must be a compile-time string literal or equivalent constant;
- the field must exist on the concrete local or imported UDT element type;
- the field type must be `int`, `float`, or `string` for the first subset;
- `bool`, `color`, object, nested UDT, collection, and tuple fields are
  rejected;
- `na` values sort with the same placement policy as scalar array sorting;
- sorting is stable when compared field values are equal;
- `array.sort` mutates the array;
- `array.sort_indices` returns original indexes without mutating the array.

Positive UDT array storage support must not silently accept sorting without
`sort_field` coverage.

## History And Realtime

Supported history policy:

- `previous = points[1]` returns a fresh copy of the committed same-local
  scalar-tree UDT array snapshot, not an alias into past storage;
- each `PineValue::UserType` element in that snapshot is independently cloned;
- `array.get`/`get()` on the historical array id preserves the local UDT
  element identity so scalar fields remain readable;
- UDT values read from same-local scalar-tree UDT arrays preserve local UDT
  identity after binding to locals, so history references on those values are
  fixture-backed;
- same-local scalar-tree UDT array `varip` values retain the array id, backing
  contents, and element identity metadata across realtime forming updates;
- UDT arrays still roll back correctly for ordinary realtime forming updates
  when stored in `var` variables, with fixture-backed coverage in
  `tests/fixtures/realtime/user_type_array_var_rollback.pine`.

Later history policy:

- dynamic history over UDT array ids should use the same retention guardrails as
  other supported history values;
- collection fields require deeper copy policy before support.

## Function And Method Boundaries

Initial policy:

- UDT values read from arrays may be passed to local pure UDFs, including
  passthrough and constructor-return UDFs;
- passing UDT array ids to user-defined functions is allowed only after reference
  and mutation semantics are fixture-backed;
- mutating UDT arrays inside user-defined functions stays unsupported in the
  first positive slice;
- mutating fields inside methods stays unsupported, including fields on UDT
  values read from arrays;
- local pure methods on UDT values read from arrays are fixture-backed after the
  value is bound to a local variable;
- receiver-style scalar-tree imported UDT methods and alias-qualified imported
  method calls over same-imported scalar-tree UDT values read from arrays are
  supported, with mismatched local/imported identities rejected.

## Diagnostics

Before positive support lands, keep rejected UDT array forms diagnostic-only.

When support starts, unsupported variants should fail with precise diagnostics:

- unsupported UDT array declaration form;
- unsupported UDT element type;
- mixed UDT element types;
- incompatible same-shape but different-name UDT element type;
- unsupported imported UDT element type or imported UDT value inference;
- unsupported UDT field family inside an array element;
- UDT array helper unsupported for the element type;
- missing, non-constant, or unknown `sort_field`;
- unsupported `sort_field` field type;
- UDT array history or `varip` use outside the supported subset;
- UDT array mutation inside an unsupported side-effect context.

## Slice Order

Recommended future slices:

1. Semantic design lock: preserve concrete local UDT names in array element
   types and add missing negative fixtures while keeping UDT arrays unsupported.
2. Runtime clone/snapshot audit: prove `PineValue::UserType` values can be
   stored in array slots without slot-to-slot or past-state aliasing.
3. Copy and reference semantics: assignment and independent element
   mutation/writeback fixtures.
4. Method-call aliases after namespace calls are stable.
5. UDT array history snapshots now clone same-local and same-imported
   scalar-tree array ids and preserve element UDT identity.
6. UDT array search helpers use same-local scalar-tree structural equality.
7. UDT array fill replaces whole arrays or valid half-open ranges with a
   same-local scalar-tree UDT value.
8. UDT array join stringifies elements as `TypeName(field0, field1, ...)`
   without widening general UDT stringification.
9. UDT array `varip` for same-local scalar-tree elements retains array ids,
   backing contents, and element identity metadata across realtime forming
   updates. Done.
10. Chained UDT array slot field mutation supports namespace-call,
    method-call, and slice-window writeback for same-local scalar-tree UDT
    arrays. Done.

## Completion Gate For Future Positive Support

Any positive UDT array support must include:

- semantic fixtures for accepted and rejected UDT array declarations;
- helper-specific negative fixtures for unsupported UDT element families;
- runtime fixtures and golden snapshots for accepted helpers;
- assignment/reference and `array.copy` independence fixtures;
- realtime rollback tests when mutation is supported;
- incremental-vs-historical parity tests when history or state timing matters;
- profile or guardrail tests for array storage growth if new accounting is
  added;
- synchronized `tests/fixtures/conformance.tsv`, `docs/CONFORMANCE.md`, matrix
  snapshot, release notes, and this design document;
- `git diff --check`;
- `scripts/verify.sh`.

## Closed Slice Result

This design gate started as a planning prerequisite and is now kept synchronized
with incremental positive runtime slices. The current supported subset is the
same-local scalar-tree UDT `array.new<T>()` and `array.from` paths with the
helper set listed in Current Boundary and Accepted Forms, plus ordinary `var`
realtime rollback, same-local scalar-tree UDT array `varip`, and typed
`array<T>`/`T[]` declarations for that same UDT array subset. Broader UDT
element families, UDT value history references, and helpers not listed there
remain unsupported until later fixture-backed slices implement syntax, analysis,
runtime behavior, and conformance updates together.
