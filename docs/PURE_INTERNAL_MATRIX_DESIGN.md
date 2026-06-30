# Pure Internal Matrix Design Gate

Status: closed design gate with the first positive float subset and copy
semantics implemented. The runtime store skeleton is implemented internally,
ordinary `var` matrix ids roll back across realtime forming updates, and
`matrix.new<float>`, `matrix.get`, `matrix.set`, `matrix.fill`,
`values.fill(value)`, `values.get(row, column)`,
`values.set(row, column, value)`, `matrix.copy`, `values.copy()`,
`matrix.reshape`, `values.reshape(rows, columns)`, `matrix.rows`,
`values.rows()`, `matrix.columns`, `values.columns()`, `matrix.row`,
`values.row(row)`, `matrix.col`, `values.col(column)`, `matrix.add_row`,
`values.add_row(row, array_id)`, `matrix.add_col`, and
`values.add_col(column, array_id)`, `matrix.remove_row`, and
`values.remove_row(row)`, `matrix.remove_col`, and `values.remove_col(column)`
plus `matrix.sum`, `values.sum()`, `matrix.avg`, and `values.avg()` are
fixture-backed for runtime-owned float matrices.

This document defines the first internal path for future `matrix.*` support. It
is scoped to interpreter internals only: parser, semantic analysis, runtime
storage, history, rollback, and conformance. It does not cover chart rendering,
host UI, external services, or public JSON/Python/WASM matrix serialization.

## Current Boundary

`matrix.*` is partially supported today for runtime-owned float matrices only.

Current evidence:

- `tests/fixtures/conformance.tsv` records `matrix.*` as `partial`.
- `tests/fixtures/runtime/matrix_float.pine` covers `matrix.new<float>`,
  `matrix.get`, `matrix.set`, `matrix.fill`, `values.fill(value)`,
  `values.get(row, column)`, `values.set(row, column, value)`, `matrix.rows`,
  `values.rows()`, `matrix.columns`, and `values.columns()` with numeric and
  `na` cells. `tests/fixtures/runtime/matrix_row.pine` covers `matrix.row`
  returning an independent `array<float>` snapshot of a matrix row, and
  `tests/fixtures/runtime/matrix_col.pine` covers `matrix.col` returning an
  independent `array<float>` snapshot of a matrix column.
  `tests/fixtures/runtime/matrix_add_row.pine` covers namespace and
  method-alias row insertion from `array<float>` row snapshots, including array
  copy-at-call-time behavior.
  `tests/fixtures/runtime/matrix_add_col.pine` covers namespace and
  method-alias column insertion from `array<float>` column snapshots, including
  array copy-at-call-time behavior.
  `tests/fixtures/runtime/matrix_remove_row.pine` covers namespace and
  method-alias row deletion with shape and remaining-cell readback.
  `tests/fixtures/runtime/matrix_remove_col.pine` covers namespace and
  method-alias column deletion with shape and remaining-cell readback.
  `tests/fixtures/runtime/matrix_sum.pine` covers namespace and method-alias
  matrix sum reads that ignore `na` cells and return `na` for empty or all-`na`
  matrices.
  `tests/fixtures/runtime/matrix_zero_dimensions.pine` covers non-negative
  zero row/column constructor dimensions and shape reads.
  `tests/fixtures/runtime/matrix_shape_loop_read.pine` covers namespace and
  method-alias shape reads through ordinary loop bodies.
  `tests/fixtures/runtime/matrix_shape_while_read.pine` covers namespace and
  method-alias shape reads after reshape calls inside `while` loop bodies.
  `tests/fixtures/runtime/matrix_control_flow.pine` covers branch and loop
  matrix mutation/readback ordering through namespace and method calls.
  `tests/fixtures/runtime/matrix_set_method_control_flow.pine` covers
  method-alias set mutation ordering through loops.
  `tests/fixtures/runtime/matrix_set_while_control_flow.pine` covers namespace
  and method-alias set mutation ordering through `while` loop bodies.
  `tests/fixtures/runtime/matrix_fill_control_flow.pine` covers namespace and
  method-alias fill mutation ordering through branches and loops.
  `tests/fixtures/runtime/matrix_fill_while_control_flow.pine` covers namespace
  and method-alias fill mutation ordering through `while` loop bodies.
  `tests/fixtures/sema/unsupported_matrix_rows.pine` and
  `tests/fixtures/sema/unsupported_matrix_columns.pine` keep matrix shape
  readers rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_rows_method_receiver.pine` and
  `tests/fixtures/sema/unsupported_matrix_columns_method_receiver.pine` keep
  the `values.rows()`/`values.columns()` method aliases rejected for non-matrix
  receivers.
  `tests/fixtures/sema/unsupported_matrix_sum.pine` and
  `tests/fixtures/sema/unsupported_matrix_sum_method_receiver.pine` keep
  namespace and method-alias sum readers rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_row.pine` and
  `tests/fixtures/sema/unsupported_matrix_col.pine` keep row/column extraction
  rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_row_method_receiver.pine` and
  `tests/fixtures/sema/unsupported_matrix_col_method_receiver.pine` keep the
  `values.row(row)`/`values.col(column)` method aliases rejected for non-matrix
  receivers.
  `tests/fixtures/sema/unsupported_matrix_row_index_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_col_index_type.pine` keep non-int
  namespace `matrix.row`/`matrix.col` row/column indexes rejected at semantic
  analysis time.
  `tests/fixtures/sema/unsupported_matrix_row_method_index_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_col_method_index_type.pine` keep
  non-int method-alias `values.row(row)`/`values.col(column)` row/column indexes
  rejected at semantic analysis time.
  `tests/fixtures/sema/unsupported_matrix_get.pine` and
  `tests/fixtures/sema/unsupported_matrix_copy.pine` keep core matrix helpers
  rejected for non-matrix receivers, and
  `tests/fixtures/sema/unsupported_matrix_copy_method_receiver.pine` keeps the
  `values.copy()` method alias rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_get_method_receiver.pine` keeps the
  `values.get(row, column)` method alias rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_get_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_get_column_type.pine` keep non-int
  `matrix.get` row/column indexes rejected at semantic analysis time.
  `tests/fixtures/sema/unsupported_matrix_get_method_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_get_method_column_type.pine` keep the
  `values.get(row, column)` method alias on the same row/column index type
  boundary.
  `tests/fixtures/sema/unsupported_matrix_set.pine`,
  `tests/fixtures/sema/unsupported_matrix_fill.pine`, and
  `tests/fixtures/sema/unsupported_matrix_reshape.pine` keep mutating matrix
  helpers rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_set_method_receiver.pine` keeps the
  `values.set(row, column, value)` method alias rejected for non-matrix
  receivers.
  `tests/fixtures/sema/unsupported_matrix_fill_method_receiver.pine` keeps the
  `values.fill(value)` method alias rejected for non-matrix receivers.
  `tests/fixtures/sema/unsupported_matrix_reshape_method_receiver.pine` keeps
  the `values.reshape(rows, columns)` method alias rejected for non-matrix
  receivers.
  `tests/fixtures/sema/unsupported_matrix_set_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_set_column_type.pine` keep non-int
  namespace `matrix.set` row/column indexes rejected at semantic analysis time.
  `tests/fixtures/sema/unsupported_matrix_set_value.pine` keeps non-numeric
  `matrix.set` values rejected for `matrix<float>` storage.
  `tests/fixtures/sema/unsupported_matrix_set_method_value.pine` keeps the
  `values.set(row, column, value)` method alias on the same value-type boundary.
  `tests/fixtures/sema/unsupported_matrix_set_method_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_set_method_column_type.pine` keep the
  method alias on the same row/column index type boundary.
  `tests/fixtures/sema/unsupported_matrix_fill_value.pine` keeps non-numeric
  `matrix.fill` values rejected for `matrix<float>` storage.
  `tests/fixtures/sema/unsupported_matrix_fill_method_value.pine` keeps the
  `values.fill(value)` method alias on the same value-type boundary.
  `tests/fixtures/runtime/matrix_udf_read.pine` covers read-only UDF matrix
  cell and shape reads through namespace and method calls.
  `tests/fixtures/runtime/matrix_udf_row_col.pine` covers read-only UDF
  row/column extraction through namespace and method calls.
  `tests/fixtures/runtime/matrix_row_col_loop_read.pine` covers row/column
  extraction reads through ordinary loop bodies.
  `tests/fixtures/runtime/matrix_udf_copy.pine` covers UDF-returned independent
  `matrix.copy` / `values.copy()` storage.
  `tests/fixtures/runtime/matrix_copy_loop.pine` covers namespace and
  method-alias copy independence through ordinary loop bodies.
  `tests/fixtures/runtime/matrix_copy_while.pine` covers namespace and
  method-alias copy independence through `while` loop bodies.
  `tests/fixtures/runtime/matrix_copy.pine` covers assignment aliasing and
  explicit independent `matrix.copy` / `values.copy()` storage.
  `tests/fixtures/realtime/matrix_rollback.pine` covers ordinary `var` matrix
  persistence and forming-bar rollback for matrix mutation.
  `tests/fixtures/realtime/matrix_reshape_rollback.pine` covers forming-bar
  rollback for matrix shape changes after namespace and method reshape calls.
  `tests/fixtures/runtime/matrix_history_shape.pine` covers committed matrix
  history shape snapshots returning fresh copies and first-bar `na` predicates
  for missing prior shape snapshots, including dynamic `na` offset predicates.
  `tests/fixtures/runtime/matrix_dynamic_history.pine` covers dynamic-offset
  matrix history snapshots returning fresh copies for cell and shape reads plus
  the `na` offset predicate boundary.
  `tests/fixtures/runtime/matrix_row_col_branch_read.pine` covers row/column
  extraction reads and independent array snapshots through `if`/`else`
  branches.
  `tests/fixtures/runtime/matrix_row_col_while_read.pine` covers the same
  snapshot boundary through `while` loop bodies.
  `crates/pine-runtime/tests/incremental.rs` covers incremental append parity
  for committed, shape, and dynamic-offset matrix history fixtures.
  `tests/fixtures/profile/matrix_heavy.pine` and
  `crates/pine-runtime/tests/profile_fixtures.rs` cover runtime profile
  slot/cell counters and bounded slot/cell capacity growth.
- `tests/fixtures/regressions/matrix_get_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_get_column_bounds.pine` cover runtime
  row/column bounds errors through the public fixture path.
  `tests/fixtures/regressions/matrix_get_method_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_get_method_column_bounds.pine` cover
  runtime `values.get(row, column)` method-alias row/column-bounds errors
  through the same path.
  `tests/fixtures/regressions/matrix_get_negative_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_get_negative_column_bounds.pine` cover
  negative row/column index rejection through the same runtime path.
  `tests/fixtures/regressions/matrix_get_method_negative_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_get_method_negative_column_bounds.pine`
  cover runtime `values.get(row, column)` method-alias negative row/column-index
  rejection through the same path.
  `tests/fixtures/regressions/matrix_set_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_set_column_bounds.pine` cover mutating
  `matrix.set` row/column bounds through that same guard.
  `tests/fixtures/regressions/matrix_set_method_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_set_method_column_bounds.pine` cover
  runtime `values.set(row, column, value)` method-alias row/column-bounds
  errors through the same guard.
  `tests/fixtures/regressions/matrix_set_negative_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_set_negative_column_bounds.pine` cover
  mutating negative row/column index rejection.
  `tests/fixtures/regressions/matrix_set_method_negative_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_set_method_negative_column_bounds.pine`
  cover runtime `values.set(row, column, value)` method-alias negative
  row/column-index rejection through the same guard.
  `tests/fixtures/regressions/matrix_get_na_row_index.pine`,
  `tests/fixtures/regressions/matrix_get_na_column_index.pine`,
  `tests/fixtures/regressions/matrix_set_na_row_index.pine`, and
  `tests/fixtures/regressions/matrix_set_na_column_index.pine` cover `na`
  row/column index rejection for matrix cell reads and writes.
  `tests/fixtures/regressions/matrix_get_method_na_row_index.pine` and
  `tests/fixtures/regressions/matrix_get_method_na_column_index.pine` cover
  runtime `values.get(row, column)` method-alias `na` row/column-index
  rejection through the same path.
  `tests/fixtures/regressions/matrix_set_method_na_row_index.pine` and
  `tests/fixtures/regressions/matrix_set_method_na_column_index.pine` cover
  runtime `values.set(row, column, value)` method-alias `na` row/column-index
  rejection through the same path.
  `tests/fixtures/regressions/matrix_row_bounds.pine` and
  `tests/fixtures/regressions/matrix_col_bounds.pine` cover runtime
  row/column extraction bounds errors through the public fixture path.
  `tests/fixtures/regressions/matrix_row_method_bounds.pine` and
  `tests/fixtures/regressions/matrix_col_method_bounds.pine` cover runtime
  `values.row(row)`/`values.col(column)` method-alias row/column-bounds errors
  through the same path.
  `tests/fixtures/regressions/matrix_row_negative_bounds.pine` and
  `tests/fixtures/regressions/matrix_col_negative_bounds.pine` cover runtime
  row/column extraction negative index rejection through the same path.
  `tests/fixtures/regressions/matrix_row_method_negative_bounds.pine` and
  `tests/fixtures/regressions/matrix_col_method_negative_bounds.pine` cover
  runtime `values.row(row)`/`values.col(column)` method-alias negative
  row/column-index rejection through the same path.
  `tests/fixtures/regressions/matrix_row_na_index.pine` and
  `tests/fixtures/regressions/matrix_col_na_index.pine` cover runtime
  row/column extraction `na` index rejection through the same path.
  `tests/fixtures/regressions/matrix_row_method_na_index.pine` and
  `tests/fixtures/regressions/matrix_col_method_na_index.pine` cover runtime
  `values.row(row)`/`values.col(column)` method-alias `na` row/column-index
  rejection through the same path.
  `tests/fixtures/regressions/matrix_new_negative_row_count.pine` and
  `tests/fixtures/regressions/matrix_new_negative_column_count.pine` cover
  negative constructor dimensions.
  `tests/fixtures/regressions/matrix_new_na_row_count.pine` and
  `tests/fixtures/regressions/matrix_new_na_column_count.pine` cover `na`
  constructor dimensions.
  `tests/fixtures/regressions/matrix_cell_limit.pine` covers the runtime matrix
  cell-budget guard before allocating unbounded storage.
- `tests/fixtures/sema/unsupported_matrix.pine` calls an unsupported matrix
  namespace function.
  `tests/fixtures/sema/unsupported_matrix_add_row.pine` keeps
  non-`array<float>` row data rejected for namespace row insertion.
  `tests/fixtures/sema/unsupported_matrix_add_col.pine` keeps
  non-`array<float>` column data rejected for namespace column insertion.
  `tests/fixtures/sema/unsupported_matrix_remove_row.pine` keeps non-int row
  indexes rejected for namespace row deletion.
  `tests/fixtures/sema/unsupported_matrix_remove_col.pine` keeps non-int
  column indexes rejected for namespace column deletion.
  `tests/fixtures/sema/unsupported_matrix_new_template.pine` calls
  `matrix.new<int>(...)`, and
  `tests/fixtures/sema/unsupported_matrix_new_deferred_template.pine` calls
  `matrix.new<label>(...)` to keep non-float and deferred element templates on
  the unsupported boundary. `tests/fixtures/sema/unsupported_matrix_new_initial_value.pine`
  keeps non-numeric `matrix.new<float>` initial values rejected at semantic
  analysis time.
- `tests/fixtures/sema/unsupported_matrix_set_udf.pine`,
  `tests/fixtures/sema/unsupported_matrix_set_method_udf.pine`,
  `tests/fixtures/sema/unsupported_matrix_fill_udf.pine`, and
  `tests/fixtures/sema/unsupported_matrix_fill_method_udf.pine` keep
  `matrix.set` and `matrix.fill`, including the `values.set(...)` and
  `values.fill(...)` method aliases, rejected inside user-defined functions
  through the collection side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_add_row_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_add_row_method_udf.pine` keep
  namespace and method-alias `matrix.add_row` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_add_col_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_add_col_method_udf.pine` keep
  namespace and method-alias `matrix.add_col` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_remove_row_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_remove_row_method_udf.pine` keep
  namespace and method-alias `matrix.remove_row` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_remove_col_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_remove_col_method_udf.pine` keep
  namespace and method-alias `matrix.remove_col` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_add_row_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_add_col_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_remove_row_method.pine`, and
  `tests/fixtures/sema/unsupported_matrix_remove_col_method.pine` keep
  matrix method-call argument diagnostics while keeping non-`array<float>`
  `values.add_row(row, array_id)` row data and
  `values.add_col(column, array_id)` column data plus non-int
  `values.remove_row(row)` row indexes and non-int
  `values.remove_col(column)` column indexes rejected;
  `values.fill(value)`, `values.get(row, column)`,
  `values.set(row, column, value)`, `values.copy()`,
  `values.reshape(rows, columns)`, `values.rows()`, `values.columns()`,
  `values.row(row)`, `values.col(column)`, and
  `values.add_row(row, array_id)`, `values.add_col(column, array_id)`, and
  `values.remove_row(row)`, and `values.remove_col(column)` are the only
  fixture-backed matrix method aliases.
  `tests/fixtures/runtime/matrix_typed_declarations.pine` covers
  `matrix<float>` declarations with compatible matrix values or `na`.
  `tests/fixtures/sema/unsupported_matrix_typed_decl.pine` keeps bare matrix
  typed declarations outside the current subset, and
  `tests/fixtures/sema/unsupported_matrix_int_typed_decl.pine` and
  `tests/fixtures/sema/unsupported_matrix_label_typed_decl.pine` keep non-float
  and deferred element matrix typed declarations outside the current subset.
  `tests/fixtures/sema/unsupported_matrix_for_in.pine` keeps matrix values out
  of `for...in` iteration.
  `tests/fixtures/runtime/matrix_history.pine` covers committed matrix history
  snapshots returning fresh matrix copies and first-bar `na` predicates for
  missing prior matrix snapshots.
  `tests/fixtures/runtime/matrix_dynamic_history.pine` covers dynamic-offset
  matrix history snapshots returning fresh matrix copies plus the `na` offset
  predicate boundary.
  `tests/fixtures/sema/unsupported_matrix_varip.pine` keeps matrix `varip`
  declarations outside the current realtime handoff subset.
  `tests/fixtures/runtime/matrix_reshape.pine` covers namespace-call reshape
  preserving element order and element count.
  `tests/fixtures/runtime/matrix_reshape_method.pine` covers the matching
  `values.reshape(rows, columns)` method alias.
  `tests/fixtures/runtime/matrix_reshape_control_flow.pine` covers namespace
  and method-alias reshape mutation ordering through branches and loops.
  `tests/fixtures/runtime/matrix_reshape_while_control_flow.pine` covers
  namespace and method-alias reshape mutation ordering through `while` loop
  bodies.
  `tests/fixtures/regressions/matrix_reshape_mismatch.pine` and
  `tests/fixtures/regressions/matrix_reshape_method_mismatch.pine` cover
  namespace and method-alias reshape dimension products that do not match the
  current element count.
  `tests/fixtures/regressions/matrix_reshape_negative_row_count.pine` and
  `tests/fixtures/regressions/matrix_reshape_negative_column_count.pine` cover
  namespace `matrix.reshape(id, rows, columns)` negative row/column-count
  rejection.
  `tests/fixtures/regressions/matrix_reshape_method_negative_row_count.pine`
  and
  `tests/fixtures/regressions/matrix_reshape_method_negative_column_count.pine`
  cover method-alias `values.reshape(rows, columns)` negative row/column-count
  rejection through the same path.
  `tests/fixtures/regressions/matrix_reshape_na_row_count.pine` and
  `tests/fixtures/regressions/matrix_reshape_na_column_count.pine` cover
  namespace `matrix.reshape(id, rows, columns)` `na` row/column-count
  rejection.
  `tests/fixtures/regressions/matrix_reshape_method_na_row_count.pine` and
  `tests/fixtures/regressions/matrix_reshape_method_na_column_count.pine`
  cover method-alias `values.reshape(rows, columns)` `na` row/column-count
  rejection through the same path.
  `tests/fixtures/sema/unsupported_matrix_reshape_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_reshape_column_type.pine` keep
  non-int namespace `matrix.reshape` row/column counts rejected at semantic
  analysis time.
  `tests/fixtures/sema/unsupported_matrix_reshape_method_row_type.pine` and
  `tests/fixtures/sema/unsupported_matrix_reshape_method_column_type.pine`
  keep non-int method-alias `values.reshape(rows, columns)` row/column counts
  rejected at semantic analysis time.
  `tests/fixtures/sema/unsupported_matrix_reshape_udf.pine` keeps reshape
  rejected inside user-defined functions through the collection side-effect
  gate.
  `tests/fixtures/sema/unsupported_matrix_reshape_method_udf.pine` keeps the
  method alias on the same side-effect boundary.
- `crates/pine-sema/src/analyzer/unsupported.rs` reports
  `matrix collections are not implemented; matrix.* requires a dedicated
  two-dimensional storage model` for matrix namespace members that are still
  outside the registered float subset.
- `crates/pine-runtime/src/builtins/matrices.rs` provides an internal
  runtime-owned rectangular matrix store skeleton, independent copy behavior,
  cell bounds checks, runtime profile slot/cell counters, and the first
  `matrix<float>` namespace-call runtime subset.
- `crates/pine-sema/tests/fixtures.rs` asserts the remaining unsupported
  diagnostics.

Do not widen `matrix.*` further until a runtime slice implements the behavior
and updates fixtures, conformance, snapshots, and docs together.

## Target Shape

The first positive matrix subset should mirror the existing collection discipline:

- matrices are runtime-owned ids, not host-visible structures;
- assignment passes matrix ids by reference;
- `matrix.copy()` is the explicit independent-copy boundary;
- non-`var` declarations allocate when executed;
- `var` declarations preserve the matrix id and backing storage across bars;
- rollback restores matrix backing storage to the confirmed snapshot;
- matrix growth is bounded by a runtime limit and visible in runtime profiles.

The first runtime value should add a new internal id family such as
`PineValue::Matrix(u32)` with a dedicated runtime store. Do not model matrices as
arrays of arrays in the first implementation. Matrix operations need stable row
and column dimensions, rectangular storage, and two-dimensional index validation.

## Element Policy

First positive element families:

- `int`
- `float`
- `bool`
- `string`
- `color`
- `na`

Deferred element families:

- arrays;
- maps;
- matrices;
- user-defined types;
- tuples;
- object ids;
- chart points;
- strategy/order/trade records.

Rationale:

- scalar elements avoid nested collection aliasing in the first slice;
- same-kind matrix storage keeps assignment and mutation diagnostics clear;
- object, UDT, and nested collection elements need separate lifetime and history
  rules before support.

## Size And Index Policy

Initial policy:

- row and column counts are non-negative integers;
- dimensions are fixed for a matrix id unless a later slice implements explicit
  row/column insertion or removal;
- row and column indexes are zero-based integers;
- negative indexes are rejected for the first subset;
- out-of-bounds indexes are runtime errors;
- `na` indexes are runtime errors;
- construction with too many cells fails before allocating unbounded storage.

The runtime uses one matrix cell budget, not separate row and column caps that
still allow excessive multiplication. Runtime profiles expose matrix slot count,
slot capacity, total matrix cell count, and matrix cell capacity.

## Type Model

The semantic model should represent matrix types explicitly, not as arrays.

Future type candidates:

- `matrix<float>`
- `matrix<int>`
- `matrix<bool>`
- `matrix<string>`
- `matrix<color>`

The first slice avoids broad generic inference. The current positive subset is
`matrix<float>` through `matrix.new<float>(rows, columns, initial)`, fixture-
backed `matrix<float>` typed declarations, and the matching get/set/fill/copy
and shape helpers for that exact element kind.

Bare `matrix` declarations, non-float matrix declarations, and mixed element
matrices should stay unsupported until type identity, `na` element behavior, and
assignment compatibility are designed for those families.

## Runtime Operations

Candidate first operation set:

- `matrix.new<T>(rows, columns, initial_value?)`
- `matrix.get(id, row, column)`
- `matrix.set(id, row, column, value)`
- `matrix.fill(id, value)`
- `matrix.rows(id)`
- `matrix.columns(id)`
- `matrix.copy(id)`

The implemented runtime rejects negative row/column counts, limits a single
matrix allocation to 100,000 cells, and reports deterministic runtime errors
before reserving matrix storage.

Candidate later operation set:

- row and column extraction helpers if they can return fixture-backed arrays;
- row and column insertion/removal only after mutation and history policy is
  explicit.

Keep method-call aliases out of the first positive slice unless the namespace
calls are already fixture-backed. Method calls should lower to the same built-in
operations only after receiver typing is stable.

The first matrix method alias is `values.fill(value)`, which lowers to
`matrix.fill(values, value)` for the existing `matrix<float>` subset and remains
rejected inside user-defined functions through the collection side-effect gate.
Shape method aliases `values.rows()` and `values.columns()` lower to their
matching namespace calls. The read-only `values.get(row, column)` method alias
lowers to `matrix.get(values, row, column)`. The mutating
`values.set(row, column, value)` method alias lowers to
`matrix.set(values, row, column, value)` and remains rejected inside
user-defined functions through the existing collection side-effect gate.
`values.copy()` lowers to `matrix.copy(values)` and allocates an independent
store snapshot. `matrix.reshape(values, rows, columns)` is supported as a
namespace call and preserves element order while requiring `rows * columns` to
match the current element count. `values.reshape(rows, columns)` lowers to that
same namespace operation. Both reshape forms remain rejected inside user-defined
functions through the collection side-effect gate. `matrix.row(values, row)` and
`matrix.col(values, column)` return independent `array<float>` snapshots of the
selected row or column. `values.row(row)` lowers to `matrix.row(values, row)`
and returns the same independent row snapshot. `values.col(column)` lowers to
`matrix.col(values, column)` and returns the same independent column snapshot.
`matrix.add_row(values, row, array_id)` inserts a copied `array<float>` row at
an index in `0..=matrix.rows(values)`, requires the row array length to match
the current column count, preserves existing row order around the insertion,
and remains guarded by the 100,000-cell matrix budget. `values.add_row(row,
array_id)` lowers to the same namespace operation.
`matrix.add_col(values, column, array_id)` inserts a copied `array<float>`
column at an index in `0..=matrix.columns(values)`, requires the column array
length to match the current row count, preserves existing column order around
the insertion, and remains guarded by the same cell budget.
`values.add_col(column, array_id)` lowers to the same namespace operation.
`matrix.remove_row(values, row)` removes an existing row using the same
`0..rows-1` row-index bounds as row reads, and `values.remove_row(row)` lowers
to the same namespace operation.
`matrix.remove_col(values, column)` removes an existing column using the same
`0..columns-1` column-index bounds as column reads, and
`values.remove_col(column)` lowers to the same namespace operation.
`matrix.sum(values)` sums numeric cells in row-major storage order, ignores
`na` cells, returns `na` for empty or all-`na` matrices, and `values.sum()`
lowers to the same read-only namespace helper. `matrix.avg(values)` averages
the same non-`na` numeric cells and returns `na` when that set is empty;
`values.avg()` lowers to `matrix.avg(values)`.
Other matrix method aliases remain unsupported until each namespace operation
has a matching fixture-backed method slice.

## History And Realtime

Current history policy:

- `previous = m[1]` should return a fresh copy of the committed matrix snapshot,
  not an alias into past storage, and first-bar missing matrix history should
  remain observable through `na(previous)`;
- dynamic history over matrix ids should use the same retention guardrails as
  other supported history values;
- committed, shape, and dynamic-offset matrix history fixtures, including
  dynamic `na` offset predicates, should match full recomputation under
  incremental append execution;
- no `varip` matrix values in the current runtime slice;
- matrix state and shape roll back correctly for ordinary realtime forming
  updates.

Later history policy:

- nested matrices or collection elements require a deeper copy policy before
  support;
- broader interaction fixtures should expand only after the committed snapshot
  contract remains stable.

## Function And Method Boundaries

Initial policy:

- passing matrix ids to user-defined functions is fixture-backed for read-only
  cell/shape reads and independent matrix copies;
- matrix mutation inside user-defined functions stays unsupported in the first
  positive slice, matching the current conservative array mutation boundary;
- matrix method calls remain limited to the fixture-backed aliases listed in the
  current boundary.

## Diagnostics

For matrix namespace members still outside the supported subset, keep the
current unsupported diagnostic:

```text
matrix collections are not implemented; matrix.* requires a dedicated two-dimensional storage model
```

When support starts, unsupported variants should fail with precise diagnostics:

- unsupported element type;
- invalid row or column count;
- invalid row or column index;
- out-of-bounds row or column access;
- unknown matrix method;
- matrix mutation inside an unsupported side-effect context;
- `varip` use outside the supported subset;
- unsupported row/column extraction return type.

## Slice Order

Recommended future slices:

1. Semantic design lock: add type names, signatures, and negative fixtures while
   keeping `matrix.*` unsupported.
2. Runtime store skeleton: add an internal matrix store and public runtime
   profile counters with no accepted Pine syntax. Done.
3. First positive scalar subset: done.
   `matrix.new<float>`, `matrix.get`, `matrix.set`, `matrix.fill`,
   `matrix.rows`, and `matrix.columns`, including runtime row/column bounds,
   negative-dimension, and cell-budget fixture coverage.
4. Copy semantics: done.
   `matrix.copy` and assignment/reference fixtures.
5. Additional scalar element kinds after the float subset is stable.
6. Realtime rollback fixtures for matrix mutation and shape changes. Done.
7. Optional method-call aliases after namespace calls are stable.
8. Row/column extraction: done for namespace `matrix.row` and `matrix.col`
   returning independent `array<float>` snapshots.
9. Matrix history snapshots only after copy/deep-copy policy is explicit. Done
   for committed and dynamic-offset `matrix<float>` ids returning fresh copies.
10. Row removal: done for namespace `matrix.remove_row` and method alias
    `values.remove_row(row)`.
11. Column removal: done for namespace `matrix.remove_col` and method alias
    `values.remove_col(column)`.
12. Matrix sum: done for namespace `matrix.sum` and method alias
    `values.sum()`.
13. Matrix average: done for namespace `matrix.avg` and method alias
    `values.avg()`.

## Completion Gate For Future Positive Support

Any positive matrix support must include:

- semantic fixtures for accepted and rejected element/index forms;
- runtime fixtures and golden snapshots;
- realtime rollback tests when mutation is supported;
- incremental-vs-historical parity tests when history or state timing matters;
- profile or guardrail tests for matrix storage growth;
- synchronized `tests/fixtures/conformance.tsv`, `docs/CONFORMANCE.md`, matrix
  snapshot, release notes, and this design document;
- `git diff --check`;
- `scripts/verify.sh`.

## Closed Slice Result

This design gate closes the planning prerequisite, the runtime-owned matrix
store skeleton prerequisite, the first positive `matrix<float>` namespace
subset, and the explicit copy/reference semantics slice. Remaining matrix
support is intentionally narrow: additional element kinds, method syntax beyond
`values.fill(value)`, `values.get(row, column)`,
`values.set(row, column, value)`, `values.copy()`,
`values.reshape(rows, columns)`, `values.row(row)`, `values.col(column)`,
`values.add_row(row, array_id)`, `values.add_col(column, array_id)`,
`values.remove_row(row)`, `values.remove_col(column)`, `values.rows()`, and
`values.columns()`, bare or non-float matrix typed declarations, `varip`, and
for-in iteration remain future slices.
`matrix<float>` typed declarations, namespace and method-call reshape,
namespace and method-call row/column extraction, namespace and method-call
row/column insertion, namespace and method-call row/column removal, and
committed plus dynamic-offset matrix history snapshots, including dynamic `na`
offset predicates, are fixture-backed.
