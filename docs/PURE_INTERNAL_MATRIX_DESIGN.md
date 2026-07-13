# Pure Internal Matrix Design Gate

Status: closed design gate with fixture-backed float, int, bool, string, and
color matrix subsets. The runtime store is implemented internally,
ordinary `var` matrix ids roll back across realtime forming updates, and
`matrix.new<float>`, `matrix.new<int>`, `matrix.new<bool>`, `matrix.get`, `matrix.set`, `matrix.fill`,
`values.fill(value)`, `values.get(row, column)`,
`values.set(row, column, value)`, `matrix.copy`, `values.copy()`,
`matrix.transpose`, `values.transpose()`, `matrix.reverse`,
`values.reverse()`, `matrix.reshape`, `values.reshape(rows, columns)`,
`matrix.kron`, `values.kron(other)`, matrix-or-scalar namespace `matrix.mult`,
`values.mult(other)`, matrix-or-scalar namespace `matrix.diff`, `values.diff(other)`,
`matrix.pow`, `values.pow(power)`,
`matrix.rows`, `values.rows()`,
`matrix.columns`, `values.columns()`,
`matrix.elements_count`, `values.elements_count()`, `matrix.is_square`,
`values.is_square()`, `matrix.is_binary`, `values.is_binary()`,
`matrix.is_diagonal`, `values.is_diagonal()`, `matrix.is_identity`,
`values.is_identity()`, `matrix.is_symmetric`, `values.is_symmetric()`,
`matrix.is_antisymmetric`, `values.is_antisymmetric()`,
`matrix.is_stochastic`, `values.is_stochastic()`, `matrix.is_zero`,
`values.is_zero()`, `matrix.row`, `values.row(row)`, `matrix.col`,
`values.col(column)`, `matrix.add_row`,
`values.add_row(row, array_id)`, `matrix.add_col`, and
`values.add_col(column, array_id)`, `matrix.remove_row`, and
`values.remove_row(row)`, `matrix.remove_col`, `values.remove_col(column)`,
`matrix.swap_rows`, `values.swap_rows(row1, row2)`, `matrix.swap_columns`,
`values.swap_columns(column1, column2)`, `matrix.sort`,
`values.sort(column?, order?)`, `matrix.submatrix`, and
`values.submatrix(from_row?, to_row?, from_column?, to_column?)` plus
`matrix.sum`,
`values.sum()`, `matrix.avg`, `values.avg()`, `matrix.min`,
`values.min()`, `matrix.max`, `values.max()`, `matrix.mode`,
`values.mode()`, `matrix.trace`, `values.trace()`, `matrix.det`,
`values.det()`, `matrix.eigenvalues`, `values.eigenvalues()`,
`matrix.eigenvectors`, `values.eigenvectors()`, `matrix.inv`, `values.inv()`,
`matrix.pinv`, `values.pinv()`, `matrix.rank`, and `values.rank()` are
fixture-backed for runtime-owned float matrices, while `matrix.new<int>` is
fixture-backed for runtime-owned int matrices through get/set/fill/copy/
transpose/reverse/reshape/submatrix/row/column extraction/row insertion/column
insertion/row deletion/column deletion/row swaps/column swaps/row sorting/shape
reads/value predicates/numeric readers/float-result matrix arithmetic including scalar namespace mult/diff and matrix-array multiplication/linear
algebra readers and the corresponding supported method aliases, while
`matrix.new<bool>` is fixture-backed for runtime-owned bool matrices through
structural get/set/fill/copy/transpose/reverse/reshape/submatrix/row/column
extraction/row insertion/column insertion/row deletion/column deletion/row
swaps/column swaps/shape reads and `is_square`.

This document defines the first internal path for future `matrix.*` support. It
is scoped to interpreter internals only: parser, semantic analysis, runtime
storage, history, rollback, and conformance. It does not cover chart rendering,
host UI, external services, or public JSON/Python/WASM matrix serialization.

## Current Boundary

`matrix.*` is partially supported today for runtime-owned float, int, bool,
string, and color matrix subsets.

Current evidence:

- `tests/fixtures/conformance.tsv` records `matrix.*` as `partial`.
- `tests/fixtures/runtime/bound_matrix_copy_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.copy().rows()`/`columns()`/`elements_count()`/`get()`/`copy()` for
  float/int/bool/string/color matrices, shape and independent-storage
  preservation, nested copies, UDF-contained reads, wrong index/helper and
  non-matrix receiver diagnostics, and retained gates for other bound matrix
  producers.
- `tests/fixtures/runtime/bound_matrix_transpose_call_result_reads.pine` plus
  the matching supported/unsupported semantic fixtures cover direct
  `values.transpose().rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for float/int/bool/string/color matrices, row/column swapping, independent
  storage, nested copies, UDF-contained reads, wrong index/helper and non-matrix
  receiver diagnostics, and the retained bound-submatrix gate.
- `tests/fixtures/runtime/bound_matrix_submatrix_call_result_reads.pine` plus
  the matching supported/unsupported semantic fixtures cover direct
  `values.submatrix(...).rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for float/int/bool/string/color matrices, half-open selected/default/empty
  ranges, independent storage, nested copies, UDF-contained reads, wrong
  range/index/helper and non-matrix receiver diagnostics, and the retained
  bound-kron gate.
- `tests/fixtures/runtime/bound_matrix_kron_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.kron(other).rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for numeric float/int matrices, expanded shape, fixed float-matrix results,
  independent storage, nested copies, UDF-contained reads, wrong operand/
  index/helper and non-numeric/non-matrix receiver diagnostics, and the retained
  bound-diff gate.
- `tests/fixtures/runtime/bound_matrix_diff_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.diff(other).rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for numeric matrix and scalar operands, operand direction, selected matrix
  shape, fixed float-matrix results, independent storage, nested copies,
  UDF-contained reads, wrong operand/index/helper and non-numeric/non-matrix
  receiver diagnostics, and the retained bound-pow gate.
- `tests/fixtures/runtime/bound_matrix_pow_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.pow(power).rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for numeric square float/int matrices, identity/copy/positive powers, fixed
  float-matrix results, independent storage, nested copies, UDF-contained
  reads, wrong power/index/helper and non-numeric/non-matrix receiver
  diagnostics, and the retained bound-inverse gate.
- `tests/fixtures/runtime/bound_matrix_inv_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.inv().rows()`/`columns()`/`elements_count()`/`get()`/`copy()` for
  numeric square float/int matrices, invertible, singular, invalid-cell, and
  empty inputs, fixed float-matrix results, independent storage, nested copies,
  UDF-contained reads, wrong index/helper and non-numeric/non-matrix receiver
  diagnostics, and the retained bound-pseudo-inverse gate.
- `tests/fixtures/runtime/bound_matrix_pinv_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct
  `values.pinv().rows()`/`columns()`/`elements_count()`/`get()`/`copy()` for
  numeric rectangular float/int matrices, swapped shape, singular results,
  swapped zero-cell shapes, invalid-cell `na`, fixed float-matrix results,
  independent storage, nested copies, UDF-contained reads, wrong index/helper
  and non-numeric/non-matrix receiver diagnostics, and the retained bound
  `values.eigenvectors()` gate.
- `tests/fixtures/runtime/bound_matrix_eigenvectors_call_result_reads.pine`
  plus the matching supported/unsupported semantic fixtures cover direct
  `values.eigenvectors().rows()`/`columns()`/`elements_count()`/`get()`/
  `copy()` for numeric square float/int matrices, real eigenvector results,
  empty `0 x 0`, invalid-cell/non-real `na`, fixed float-matrix results,
  independent storage, nested copies, UDF-contained reads, wrong index/helper
  and non-numeric/non-matrix receiver diagnostics, and the retained
  matrix-valued bound `values.mult(other)` gate.
- `tests/fixtures/runtime/bound_matrix_mult_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover direct matrix-result
  `values.mult(other).rows()`/`columns()`/`elements_count()`/`get()`/`copy()`
  for numeric matrix or scalar operands, multiplied and scalar-selected shape,
  fixed float-matrix results, `na` and zero-inner-dimension behavior,
  independent storage, nested copies, UDF-contained reads, and wrong result
  helper/index/non-numeric/non-matrix diagnostics. Matrix-array overloads keep
  array-helper dispatch; the following local-UDF slice is covered separately.
- `tests/fixtures/runtime/local_udf_matrix_call_result_reads.pine` plus the
  matching supported/unsupported semantic fixtures cover unqualified local-UDF
  matrix results through rows/columns/elements_count/get/copy for parameter
  passthrough, block aliases, nested calls, same-kind control flow,
  matrix-operation and constructor returns, named/reordered arguments,
  float/int/bool/string/color call-specific kinds, zero dimensions, independent
  copies, and copy-only continuation. Unknown/`na`, scalar, array, map,
  unregistered or unresolved user-function results, broader helpers, mutation,
  and terminal-read continuation remain gated.
- `tests/fixtures/runtime/user_method_matrix_call_result_reads.pine` and
  `tests/fixtures/runtime/import_user_method_matrix_call_result_reads.pine`
  plus their supported/unsupported semantic fixtures cover local and imported
  user-method matrix results through rows/columns/elements_count/get/copy.
  Receiver-style, local-type-qualified or alias-qualified, direct-constructor-
  receiver, block/nested/same-kind-control-flow, float/int/bool/string/color,
  zero-dimension, same-library dual-alias, independent-copy, and copy-only-
  continuation paths are fixture-backed. Unknown/`na`, non-matrix or
  unresolved method results, unregistered or unresolved user-function matrix
  results, broader helpers, mutation, and terminal-read continuation remain
  gated.
- `tests/fixtures/runtime/import_function_matrix_call_result_reads.pine` plus
  its supported/unsupported semantic fixtures covers registered imported pure-
  function matrix results through rows/columns/elements_count/get/copy. Alias-
  qualified, block/nested/same-kind-control-flow, float/int/bool/string/color,
  zero-dimension, same-library dual-alias, independent-copy, and copy-only-
  continuation paths are fixture-backed. Unknown/`na`, non-matrix,
  unregistered or unresolved function results, broader helpers, mutation, and
  terminal-read continuation remain gated.
- `tests/fixtures/runtime/matrix_float.pine` covers `matrix.new<float>`,
  `matrix.get`, `matrix.set`, `matrix.fill`, `values.fill(value)`,
  `values.get(row, column)`, `values.set(row, column, value)`, `matrix.rows`,
  `values.rows()`, `matrix.columns`, and `values.columns()` with numeric and
  `na` cells. `tests/fixtures/runtime/matrix_row.pine` covers `matrix.row`
  returning an independent `array<float>` snapshot of a matrix row, and
  `tests/fixtures/runtime/matrix_col.pine` covers `matrix.col` returning an
  independent `array<float>` snapshot of a matrix column.
  `tests/fixtures/runtime/matrix_int.pine` covers `matrix.new<int>` with int
  and `na` cells plus get/set/fill/copy/transpose/reverse/reshape/submatrix/
  row/column extraction, row/column insertion, row/column deletion,
  row/column swaps, row sorting, shape helpers, value predicates, and numeric
  readers, plus float-result matrix arithmetic including scalar namespace mult
  and scalar namespace diff and linear algebra readers.
  `tests/fixtures/runtime/matrix_bool.pine` covers `matrix.new<bool>` with
  bool and `na` cells plus get/set/fill/copy/transpose/reverse/reshape/
  submatrix/row/column extraction, row/column insertion, row/column deletion,
  row/column swaps, shape helpers, and `is_square`.
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
  `tests/fixtures/runtime/matrix_swap_rows.pine` covers namespace and
  method-alias in-place row swaps, including same-row and zero-column no-op
  behavior.
  `tests/fixtures/runtime/matrix_swap_columns.pine` covers namespace and
  method-alias in-place column swaps, including same-column and zero-row no-op
  behavior.
  `tests/fixtures/runtime/matrix_sort.pine` covers namespace and method-alias
  in-place row sorting by column, including default column `0`, descending
  order, stable tie order, and `na` placement.
  `tests/fixtures/runtime/matrix_submatrix.pine` covers namespace and
  method-alias independent matrix slice copies, including default full-range
  copies, empty row and column slices, UDF reads, and source mutation
  independence.
  `tests/fixtures/runtime/matrix_transpose.pine` covers namespace and
  method-alias transposes returning independent matrix copies with swapped
  row/column counts.
  `tests/fixtures/runtime/matrix_kron.pine` covers namespace and method-alias
  Kronecker products returning independent matrix copies with expanded shape,
  `na` cell propagation, and zero-dimension results.
  `tests/fixtures/runtime/matrix_mult.pine` covers namespace and method-alias
  matrix-by-matrix, scalar namespace, and matrix-array/array-matrix multiplication returning independent matrix or array copies with
  multiplied or same shape, `na` cell propagation, and zero-dimension results.
  `tests/fixtures/runtime/matrix_diff.pine` covers namespace and method-alias
  matrix-by-matrix and scalar namespace subtraction returning independent matrix copies with
  same shape, `na` cell propagation, and zero-dimension results.
  `tests/fixtures/runtime/matrix_pow.pine` covers namespace and method-alias
  matrix powers returning independent identity, copy, and powered matrix
  results, including `na` propagation and zero-dimension results.
  `tests/fixtures/runtime/matrix_reverse.pine` covers namespace and
  method-alias in-place matrix reversal, including zero-dimension no-op shape
  preservation.
  `tests/fixtures/runtime/matrix_is_square.pine` covers namespace and
  method-alias square-shape predicates, including zero-dimension matrices.
  `tests/fixtures/runtime/matrix_is_binary.pine` covers namespace and
  method-alias binary-value predicates, including `na` cells and zero-dimension
  matrices.
  `tests/fixtures/runtime/matrix_is_diagonal.pine` covers namespace and
  method-alias diagonal-value predicates, including rectangular matrices, `na`
  cells, and zero-dimension matrices.
  `tests/fixtures/runtime/matrix_is_identity.pine` covers namespace and
  method-alias identity-value predicates, including non-square matrices, `na`
  cells, and empty `0 x 0` matrices.
  `tests/fixtures/runtime/matrix_is_symmetric.pine` covers namespace and
  method-alias symmetric-value predicates, including non-square matrices, `na`
  cells, and empty `0 x 0` matrices.
  `tests/fixtures/runtime/matrix_is_antisymmetric.pine` covers namespace and
  method-alias antisymmetric-value predicates, including non-square matrices,
  non-zero diagonal cells, `na` cells, and empty `0 x 0` matrices.
  `tests/fixtures/runtime/matrix_is_stochastic.pine` covers namespace and
  method-alias stochastic-value predicates, including row-sum and column-sum
  forms, negative values, `na` cells, and zero-element matrices.
  `tests/fixtures/runtime/matrix_is_zero.pine` covers namespace and
  method-alias zero-value predicates, including `na` cells and zero-dimension
  matrices.
  `tests/fixtures/runtime/matrix_sum.pine` covers namespace and method-alias
  matrix sum reads that ignore `na` cells and return `na` for empty or all-`na`
  matrices. `tests/fixtures/runtime/matrix_min_max.pine` covers namespace and
  method-alias matrix min/max reads under the same `na` policy.
  `tests/fixtures/runtime/matrix_trace.pine` covers namespace and method-alias
  matrix trace reads over the main diagonal under the same `na` policy,
  including rectangular matrices.
  `tests/fixtures/runtime/matrix_det.pine` covers namespace and method-alias
  determinant reads for runtime-owned float square matrices, including
  row-swap pivoting, empty `0 x 0` matrices, and `na` matrices.
  `tests/fixtures/runtime/matrix_eigenvalues.pine` covers namespace and
  method-alias eigenvalue reads for runtime-owned float square matrices,
  including symmetric, 2x2 real non-symmetric, empty, `na`, and non-real
  eigenvalue boundaries.
  `tests/fixtures/runtime/matrix_eigenvectors.pine` covers namespace and
  method-alias eigenvector reads for runtime-owned float square matrices,
  including independent result matrices, 2x2 real non-symmetric, empty, `na`,
  and non-real eigenvector boundaries.
  `tests/fixtures/runtime/matrix_inv.pine` covers namespace and method-alias
  inverse-matrix reads for runtime-owned float square matrices, including
  independent result matrices, singular matrices, empty `0 x 0` matrices, and
  `na` matrices.
  `tests/fixtures/runtime/matrix_pinv.pine` covers namespace and method-alias
  pseudo-inverse reads for runtime-owned float matrices, including invertible
  square, singular square, rectangular, zero-dimension, and `na` matrices.
  `tests/fixtures/runtime/matrix_rank.pine` covers namespace and method-alias
  rank reads for runtime-owned float rectangular matrices, including dependent
  rows, zero-dimension matrices, and `na` matrices.
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
  `tests/fixtures/sema/supported_matrix_new_int.pine` keeps the int matrix
  starter subset accepted. `tests/fixtures/sema/unsupported_matrix_new_template.pine`
  calls `matrix.new<line>(...)`, and
  `tests/fixtures/sema/unsupported_matrix_new_deferred_template.pine` calls
  `matrix.new<label>(...)` to keep other scalar and deferred element templates on
  the unsupported boundary. `tests/fixtures/sema/unsupported_matrix_new_initial_value.pine`
  keeps non-numeric `matrix.new<float>` initial values rejected at semantic
  analysis time. `tests/fixtures/sema/unsupported_matrix_new_int_initial_value.pine`
  keeps non-int `matrix.new<int>` initial values rejected, and
  `tests/fixtures/sema/unsupported_matrix_int_set_float.pine` and
  `tests/fixtures/sema/unsupported_matrix_int_fill_float.pine` keep non-int
  writes rejected for int matrix cells,
  `tests/fixtures/sema/unsupported_matrix_int_add_row_float_array.pine` and
  `tests/fixtures/sema/unsupported_matrix_int_add_col_float_array.pine` keep
  float arrays rejected for int matrix row/column insertion.
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
  `tests/fixtures/sema/unsupported_matrix_swap_rows_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_swap_rows_method_udf.pine` keep
  namespace and method-alias `matrix.swap_rows` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_swap_columns_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_swap_columns_method_udf.pine` keep
  namespace and method-alias `matrix.swap_columns` rejected inside user-defined
  functions through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_sort_udf.pine` and
  `tests/fixtures/sema/unsupported_matrix_sort_method_udf.pine` keep namespace
  and method-alias `matrix.sort` rejected inside user-defined functions
  through the same side-effect gate.
  `tests/fixtures/sema/unsupported_matrix_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_add_row_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_add_col_method.pine`,
  `tests/fixtures/sema/unsupported_matrix_remove_row_method.pine`, and
  `tests/fixtures/sema/unsupported_matrix_remove_col_method.pine` keep
  matrix method-call argument diagnostics, while
  `tests/fixtures/sema/unsupported_matrix_swap_rows_method_row1.pine` and
  `tests/fixtures/sema/unsupported_matrix_swap_rows_method_row2.pine` keep
  non-int method row-swap indexes rejected, and
  `tests/fixtures/sema/unsupported_matrix_swap_columns_method_column1.pine`
  and
  `tests/fixtures/sema/unsupported_matrix_swap_columns_method_column2.pine`
  keep non-int method column-swap indexes rejected.
  `tests/fixtures/sema/unsupported_matrix_sort_method_column.pine` and
  `tests/fixtures/sema/unsupported_matrix_sort_method_order.pine` keep non-int
  sort columns and non-const-string sort orders rejected.
  `tests/fixtures/sema/unsupported_matrix_submatrix_method_from_row.pine`,
  `tests/fixtures/sema/unsupported_matrix_submatrix_method_to_row.pine`,
  `tests/fixtures/sema/unsupported_matrix_submatrix_method_from_column.pine`,
  and `tests/fixtures/sema/unsupported_matrix_submatrix_method_to_column.pine`
  keep non-int submatrix range indexes rejected. They also keep
  non-`array<float>`
  `values.add_row(row, array_id)` row data and
  `values.add_col(column, array_id)` column data plus non-int
  `values.remove_row(row)` row indexes and non-int
  `values.remove_col(column)` column indexes rejected;
  `values.fill(value)`, `values.get(row, column)`,
  `values.set(row, column, value)`, `values.copy()`,
  `values.reshape(rows, columns)`, `values.rows()`, `values.columns()`,
  `values.row(row)`, `values.col(column)`, and
  `values.add_row(row, array_id)`, `values.add_col(column, array_id)`, and
  `values.remove_row(row)`, `values.remove_col(column)`, and
  `values.swap_rows(row1, row2)`, and
  `values.swap_columns(column1, column2)`, and `values.sort(column?, order?)`
  and `values.submatrix(from_row?, to_row?, from_column?, to_column?)` are the only
  fixture-backed matrix method aliases.
  `tests/fixtures/runtime/matrix_typed_declarations.pine` covers
  `matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and `matrix<color>` declarations with compatible matrix values
  or `na`.
  `tests/fixtures/sema/unsupported_matrix_typed_decl.pine` keeps bare matrix
  typed declarations outside the current subset, and
  `tests/fixtures/sema/unsupported_matrix_int_typed_decl.pine` and
  `tests/fixtures/sema/unsupported_matrix_label_typed_decl.pine` keep
  cross-element and deferred element matrix typed declarations outside the
  current subset.
  `tests/fixtures/runtime/matrix_for_in.pine` covers statement-form
  `for...in` iteration over runtime-owned matrix rows, including index/value
  row numbers, independent row snapshot arrays, empty matrices, loop control,
  and shape mutation after loop-entry snapshots.
  `tests/fixtures/runtime/matrix_history.pine` covers committed matrix history
  snapshots returning fresh matrix copies and first-bar `na` predicates for
  missing prior matrix snapshots.
  `tests/fixtures/runtime/matrix_dynamic_history.pine` covers dynamic-offset
  matrix history snapshots returning fresh matrix copies plus the `na` offset
  predicate boundary.
  `tests/fixtures/runtime/matrix_varip.pine`,
  `tests/fixtures/realtime/matrix_varip.pine`, and
  `tests/fixtures/sema/supported_matrix_varip.pine` cover matrix `varip`
  declarations and realtime backing-store handoff.
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
backed `matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and `matrix<color>`
typed declarations, and the matching get/set/fill/copy and shape helpers for
float matrices, plus `matrix<int>`
inference through `matrix.new<int>(rows, columns, initial)` with basic get/set/fill/copy/
transpose/reverse/reshape/submatrix/row/column extraction/row/column
insertion/row deletion/column deletion/row swaps/column swaps/row sorting/shape
helpers/value predicates/numeric readers/float-result matrix arithmetic including scalar namespace mult/diff and matrix-array multiplication/linear
algebra readers only, plus `matrix<bool>` inference through
`matrix.new<bool>(rows, columns, initial)` with structural helpers and bool
row/column arrays only, plus `matrix<string>` inference through
`matrix.new<string>(rows, columns, initial)` with structural helpers and string
row/column arrays only, plus `matrix<color>` inference through
`matrix.new<color>(rows, columns, initial)` with structural helpers and color
row/column arrays only.

Bare `matrix` declarations, matrix declarations beyond float/int/bool/string/color, additional matrix
templates, and mixed element matrices should stay unsupported until type
identity, `na` element behavior, and assignment compatibility are designed for
those families.

## Runtime Operations

Candidate first operation set:

- `matrix.new<T>(rows, columns, initial_value?)`
- `matrix.get(id, row, column)`
- `matrix.set(id, row, column, value)`
- `matrix.fill(id, value)`
- `matrix.rows(id)`
- `matrix.columns(id)`
- `matrix.elements_count(id)`
- `matrix.copy(id)`
- `matrix.transpose(id)`

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
`matrix.fill(values, value)` for supported matrix element kinds and remains
rejected inside user-defined functions through the collection side-effect gate.
Shape method aliases `values.rows()` and `values.columns()` lower to their
matching namespace calls. The read-only `values.get(row, column)` method alias
lowers to `matrix.get(values, row, column)`. The mutating
`values.set(row, column, value)` method alias lowers to
`matrix.set(values, row, column, value)` and remains rejected inside
user-defined functions through the existing collection side-effect gate.
`values.copy()` lowers to `matrix.copy(values)` and allocates an independent
store snapshot. `matrix.transpose(values)` allocates an independent matrix
store whose row count is the source column count and whose column count is the
source row count; `values.transpose()` lowers to the same read-only transform.
`matrix.reverse(values)` reverses cells in place so source cell `(row, column)`
moves to `(rows - 1 - row, columns - 1 - column)` without changing shape, and
`values.reverse()` lowers to the same mutating namespace operation. Both
reverse forms remain rejected inside user-defined functions through the
collection side-effect gate.
`matrix.reshape(values, rows, columns)` is supported as a namespace call and
preserves element order while requiring `rows * columns` to match the current
element count. `values.reshape(rows, columns)` lowers to that same namespace
operation. Both reshape forms remain rejected inside user-defined functions
through the collection side-effect gate. `matrix.kron(left, right)` returns an
independent `matrix<float>` Kronecker product whose shape is
`left.rows() * right.rows()` by `left.columns() * right.columns()`, propagates
`na` to result cells when either source cell is `na` or non-finite, preserves
zero-dimension results, and rejects outputs over the matrix cell budget.
`values.kron(other)` lowers to the same read-only namespace helper.
Matrix-by-matrix `matrix.mult(left, right)` returns an independent
`matrix<float>` product whose shape is `left.rows()` by `right.columns()`,
requires `left.columns() == right.rows()`, propagates `na` to a result cell
when any contributing source cell is `na` or non-finite, preserves
zero-dimension results, and rejects outputs over the matrix cell budget.
`values.mult(other)` lowers to the same read-only namespace helper for matrix
right-hand operands, and scalar namespace `matrix.mult(values, scalar)` and
`matrix.mult(scalar, values)` plus method alias `values.mult(scalar)` return
independent `matrix<float>` copies with every numeric cell multiplied by the
numeric scalar while `na` or non-finite cells/scalars propagate to `na`.
`matrix.mult(values, vector)` and `values.mult(vector)` accept right-hand
`array<float>` or `array<int>` operands as column vectors, require the array
size to match the matrix column count, and return independent `array<float>`
dot-product results with one element per matrix row. Namespace
`matrix.mult(vector, values)` accepts left-hand `array<float>` or `array<int>`
operands as row vectors, requires the array size to match the matrix row count,
and returns independent `array<float>` dot-product results with one element per
matrix column. Namespace `matrix.mult(left_vector, right_vector)` accepts
numeric array pairs with equal length, treats them as a row vector and column
vector, and returns an independent single-element `array<float>` dot-product
result. Non-numeric-array `matrix.mult` overloads remain outside this slice.
Matrix-by-matrix `matrix.diff(left, right)` returns an independent
`matrix<float>` element-wise difference whose shape matches both operands,
requires identical row and column counts, propagates `na` to a result cell when
either source cell is `na` or non-finite, and preserves zero-dimension results.
`values.diff(other)` lowers to the same read-only namespace helper for matrix
right-hand operands, and scalar namespace `matrix.diff(values, scalar)` and
`matrix.diff(scalar, values)` plus method alias `values.diff(scalar)` return
independent `matrix<float>` copies using operand order while `na` or non-finite
cells/scalars propagate to `na`.
`matrix.pow(values, power)` returns an independent `matrix<float>` power for
runtime-owned square float matrices. Power `0` returns an identity matrix,
power `1` returns an independent copy, larger powers use matrix multiplication
semantics with `na` or non-finite contributing cells propagating to `na` result
cells, and zero-dimension `0 x 0` matrices remain zero-dimensional. Negative
powers and non-square matrices raise runtime errors. `values.pow(power)` lowers
to the same read-only namespace helper.
`matrix.elements_count(values)`
returns the current row-count by column-count element count, including zero for
zero-dimension matrices, and `values.elements_count()` lowers to the same shape
reader. `matrix.is_square(values)` returns whether row and column counts are
equal, including true for `0 x 0` matrices, and `values.is_square()` lowers to
the same read-only shape predicate. `matrix.is_zero(values)` returns true when
every stored numeric cell is zero, false for any non-zero or `na` cell, and true
for zero-element matrices. `values.is_zero()` lowers to the same read-only
value predicate. `matrix.is_binary(values)` returns true when every stored
numeric cell is exactly zero or one, false for any other numeric value or `na`
cell, and true for zero-element matrices. `values.is_binary()` lowers to the
same read-only value predicate. `matrix.is_diagonal(values)` returns true when
every cell outside the main diagonal is zero, false for any non-zero or `na`
off-diagonal cell, allows any main-diagonal value, does not require a square
shape, and returns true for zero-element matrices. `values.is_diagonal()`
lowers to the same read-only value predicate. `matrix.is_identity(values)`
returns true only for square matrices whose main diagonal cells are exactly one
and whose off-diagonal cells are exactly zero, false for any `na` cell, false
for non-square matrices, and true for empty `0 x 0` matrices.
`values.is_identity()` lowers to the same read-only value predicate.
`matrix.is_symmetric(values)` returns true only for square matrices whose
stored numeric cells match their transposed counterparts, false for any `na`
cell, false for non-square matrices, and true for empty `0 x 0` matrices.
`values.is_symmetric()` lowers to the same read-only value predicate.
`matrix.is_antisymmetric(values)` returns true only for square matrices whose
main diagonal cells are exactly zero and whose off-diagonal cells are the
negatives of their transposed counterparts, false for any `na` cell, false for
non-square matrices, and true for empty `0 x 0` matrices.
`values.is_antisymmetric()` lowers to the same read-only value predicate.
`matrix.is_stochastic(values)` returns true when every cell is a finite
non-negative number and either every row sums exactly to one or every column
sums exactly to one, returns false for any `na` or negative cell, and returns
false for zero-element matrices. `values.is_stochastic()` lowers to the same
read-only value predicate.
`matrix.row(values, row)` and
`matrix.col(values, column)` return independent row/column snapshots:
`array<float>` for float matrices and `array<int>` for int matrices.
`values.row(row)` lowers to `matrix.row(values, row)`
and returns the same independent row snapshot. `values.col(column)` lowers to
`matrix.col(values, column)` and returns the same independent column snapshot.
`matrix.add_row(values, row, array_id)` inserts a copied row array at an index
in `0..=matrix.rows(values)`, requires the row array element kind to match the
matrix element kind and its length to match the current column count, preserves
existing row order around the insertion, and remains guarded by the 100,000-cell
matrix budget. `values.add_row(row, array_id)` lowers to the same namespace
operation. Float matrices require `array<float>` row data and int matrices
require `array<int>` row data.
`matrix.add_col(values, column, array_id)` inserts a copied column array at an
index in `0..=matrix.columns(values)`, requires the column array element kind to
match the matrix element kind and its length to match the current row count,
preserves existing column order around the insertion, and remains guarded by
the same cell budget.
`values.add_col(column, array_id)` lowers to the same namespace operation.
Float matrices require `array<float>` column data and int matrices require
`array<int>` column data.
`matrix.remove_row(values, row)` removes an existing row using the same
`0..rows-1` row-index bounds as row reads, and `values.remove_row(row)` lowers
to the same namespace operation.
`matrix.remove_col(values, column)` removes an existing column using the same
`0..columns-1` column-index bounds as column reads, and
`values.remove_col(column)` lowers to the same namespace operation.
`matrix.swap_rows(values, row1, row2)` swaps two existing rows in place using
the same `0..rows-1` row-index bounds as row reads, preserves shape, leaves a
same-row swap unchanged, and leaves zero-column matrices unchanged after row
validation. `values.swap_rows(row1, row2)` lowers to the same namespace
operation. Both forms remain rejected inside user-defined functions through
the collection side-effect gate.
`matrix.swap_columns(values, column1, column2)` swaps two existing columns in
place using the same `0..columns-1` column-index bounds as column reads,
preserves shape, leaves a same-column swap unchanged, and leaves zero-row
matrices unchanged after column validation.
`values.swap_columns(column1, column2)` lowers to the same namespace operation.
Both forms remain rejected inside user-defined functions through the collection
side-effect gate.
`matrix.sort(values, column?, order?)` sorts complete row ranges in place by
the selected column, defaults to column `0`, accepts `order.ascending` and
`order.descending`, preserves original row order for equal sort keys, places
`na` keys last ascending and first descending, and validates the selected
column using the same column-index bounds as column reads. `values.sort()`,
`values.sort(column)`, and `values.sort(column, order)` lower to the same
namespace operation. Both forms remain rejected inside user-defined functions
through the collection side-effect gate.
`matrix.submatrix(values, from_row?, to_row?, from_column?, to_column?)`
returns an independent matrix copy of the selected half-open row/column range,
defaulting omitted bounds to the full source matrix. Range indexes accept
`0..=rows` and `0..=columns`, allowing empty row or column slices, and reject
`na`, out-of-bounds, or reversed ranges at runtime.
`values.submatrix(...)` lowers to the same namespace operation.
`matrix.sum(values)` sums numeric cells in row-major storage order, ignores
`na` cells, returns `na` for empty or all-`na` matrices, and `values.sum()`
lowers to the same read-only namespace helper. `matrix.avg(values)` averages
the same non-`na` numeric cells and returns `na` when that set is empty;
`values.avg()` lowers to `matrix.avg(values)`. `matrix.min(values)` and
`matrix.max(values)` scan the same non-`na` numeric cells, return `na` when the
set is empty, and `values.min()`/`values.max()` lower to the corresponding
namespace helpers. `matrix.mode(values)` returns the smallest most-frequent
non-`na` numeric cell only when a value repeats, returns `na` for empty,
all-`na`, or no-repeated-value matrices, and `values.mode()` lowers to the same
namespace helper. `matrix.trace(values)` sums non-`na` numeric cells on the
main diagonal over `min(rows, columns)` positions, returns `na` for empty or
all-`na` diagonals, and `values.trace()` lowers to the same read-only namespace
helper. `matrix.det(values)` computes the determinant of square float matrices
without mutating the source matrix, returns `1.0` for empty `0 x 0` matrices,
returns `na` for any `na` or non-finite cell, and raises a runtime error for
non-square matrices. `values.det()` lowers to the same read-only namespace
helper. `matrix.eigenvalues(values)` returns an independent `array<float>` of
real eigenvalues for square float matrices, returns an empty array for empty
`0 x 0` matrices, returns `na` for any `na` or non-finite cell and for
non-real eigenvalue results, and raises a runtime error for non-square
matrices. `values.eigenvalues()` lowers to the same read-only namespace helper.
`matrix.eigenvectors(values)` returns an independent `matrix<float>` whose
columns are real eigenvectors for square float matrices, returns an independent
empty `0 x 0` matrix for empty `0 x 0` input, returns `na` for any `na` or
non-finite cell and for non-real or incomplete eigenvector results, and raises
a runtime error for non-square matrices. `values.eigenvectors()` lowers to the
same read-only namespace helper.
`matrix.inv(values)` computes an independent inverse matrix for
non-singular square float matrices, returns an independent empty `0 x 0` matrix
for empty `0 x 0` input, returns `na` for any `na` or non-finite cell and for
singular matrices, and raises a runtime error for non-square matrices.
`values.inv()` lowers to the same read-only namespace helper.
`matrix.pinv(values)` computes an independent Moore-Penrose pseudo-inverse
matrix with row/column counts swapped from the source, returns an independent
zero-cell matrix for zero-row or zero-column input, returns `na` for any `na`
or non-finite cell, and supports singular and rectangular matrices.
`values.pinv()` lowers to the same read-only namespace helper.
`matrix.rank(values)` computes the rank of rectangular float matrices
without mutating the source matrix, returns `0` for zero-element matrices,
returns `na` for any `na` or non-finite cell, and `values.rank()` lowers to
the same read-only namespace helper.
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
- `matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and
  `matrix<color>` `varip` declarations retain matrix ids and backing stores
  across repeated realtime forming updates;
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
14. Matrix min/max: done for namespace `matrix.min`/`matrix.max` and method
    aliases `values.min()`/`values.max()`.
15. Matrix mode: done for namespace `matrix.mode` and method alias
    `values.mode()`.
16. Matrix trace: done for namespace `matrix.trace` and method alias
    `values.trace()`.
17. Matrix determinant: done for namespace `matrix.det` and method alias
    `values.det()`.
18. Matrix eigenvalues: done for namespace `matrix.eigenvalues` and method
    alias `values.eigenvalues()`.
19. Matrix eigenvectors: done for namespace `matrix.eigenvectors` and method
    alias `values.eigenvectors()`.
20. Matrix Kronecker product: done for namespace `matrix.kron` and method alias
    `values.kron(other)`.
21. Matrix multiplication: done for matrix-by-matrix, scalar, matrix-array,
    array-matrix, and numeric array-pair namespace `matrix.mult`, plus the
    matrix-receiver method alias `values.mult(other)`.
22. Matrix subtraction: done for matrix-by-matrix and scalar namespace
    `matrix.diff` plus method alias `values.diff(other)`.
23. Matrix power: done for namespace `matrix.pow` and method alias
    `values.pow(power)`.
24. Matrix inverse: done for namespace `matrix.inv` and method alias
    `values.inv()`.
25. Matrix pseudo-inverse: done for namespace `matrix.pinv` and method alias
    `values.pinv()`.
26. Matrix rank: done for namespace `matrix.rank` and method alias
    `values.rank()`.
27. Matrix elements count: done for namespace `matrix.elements_count` and
    method alias `values.elements_count()`.
28. Matrix square predicate: done for namespace `matrix.is_square` and method
    alias `values.is_square()`.
29. Matrix transpose: done for namespace `matrix.transpose` and method alias
    `values.transpose()`.
30. Matrix reverse: done for namespace `matrix.reverse` and method alias
    `values.reverse()`.
31. Matrix zero predicate: done for namespace `matrix.is_zero` and method alias
    `values.is_zero()`.
32. Matrix binary predicate: done for namespace `matrix.is_binary` and method
    alias `values.is_binary()`.
33. Matrix diagonal predicate: done for namespace `matrix.is_diagonal` and
    method alias `values.is_diagonal()`.
34. Matrix identity predicate: done for namespace `matrix.is_identity` and
    method alias `values.is_identity()`.
35. Matrix symmetric predicate: done for namespace `matrix.is_symmetric` and
    method alias `values.is_symmetric()`.
36. Matrix antisymmetric predicate: done for namespace `matrix.is_antisymmetric`
    and method alias `values.is_antisymmetric()`.
37. Matrix stochastic predicate: done for namespace `matrix.is_stochastic` and
    method alias `values.is_stochastic()`.
38. Matrix row swap: done for namespace `matrix.swap_rows` and method alias
    `values.swap_rows(row1, row2)`.
39. Matrix column swap: done for namespace `matrix.swap_columns` and method
    alias `values.swap_columns(column1, column2)`.
40. Matrix row sort: done for namespace `matrix.sort` and method alias
    `values.sort(column?, order?)`.
41. Matrix submatrix copy: done for namespace `matrix.submatrix` and method
    alias `values.submatrix(from_row?, to_row?, from_column?, to_column?)`.
42. Matrix `varip`: done for `matrix<float>`, `matrix<int>`,
    `matrix<bool>`, `matrix<string>`, and `matrix<color>` ids with realtime
    backing-store handoff across forming updates.
43. Bound matrix-copy call results: done for exact supported matrix receivers
    using `values.copy()` followed by rows/columns/elements_count/get/copy,
    with concrete element-kind checks, shape preservation, independent backing
    storage, copy-only continuation, and retained gates for other bound
    producers, broader helpers, mutation, and non-matrix receivers.
44. Bound matrix-transpose call results: done for exact supported matrix
    receivers using `values.transpose()` followed by
    rows/columns/elements_count/get/copy, with concrete element-kind checks,
    swapped shape, independent backing storage, copy-only continuation, and
    retained gates for other bound producers, broader helpers, mutation, and
    non-matrix receivers.
45. Bound matrix-submatrix call results: done for exact supported matrix
    receivers using `values.submatrix(...)` followed by
    rows/columns/elements_count/get/copy, with concrete element-kind checks,
    selected/default/empty half-open ranges, independent backing storage,
    copy-only continuation, and retained gates for other bound producers,
    broader helpers, mutation, and non-matrix receivers.
46. Bound matrix-Kronecker call results: done for exact numeric matrix receivers
    using `values.kron(other)` followed by
    rows/columns/elements_count/get/copy, with numeric operand checks, expanded
    shape, fixed float-matrix results, independent backing storage, copy-only
    continuation, and retained gates for other bound producers, broader
    helpers, mutation, and non-matrix receivers.
47. Bound matrix-difference call results: done for exact numeric matrix
    receivers using `values.diff(other)` with matrix or scalar operands followed
    by rows/columns/elements_count/get/copy, preserving operand direction and
    selected matrix shape with fixed float-matrix results, independent backing
    storage, copy-only continuation, and retained gates for other bound
    producers, broader helpers, mutation, and non-matrix receivers.
48. Bound matrix-power call results: done for exact numeric square matrix
    receivers using `values.pow(power)` followed by
    rows/columns/elements_count/get/copy, preserving square shape across
    identity, copy, and positive powers with fixed float-matrix results,
    independent backing storage, copy-only continuation, and retained gates
    for other bound producers, broader helpers, mutation, and non-matrix
    receivers.
49. Bound matrix-inverse call results: done for exact numeric square matrix
    receivers using `values.inv()` followed by
    rows/columns/elements_count/get/copy, preserving invertible square shape,
    empty `0 x 0` results, and `na` singular/invalid-cell results with fixed
    float-matrix metadata, independent backing storage, copy-only continuation,
    and retained gates for other bound producers, broader helpers, mutation,
    and non-matrix receivers.
50. Bound matrix-pseudo-inverse call results: done for exact numeric matrix
    receivers using `values.pinv()` followed by
    rows/columns/elements_count/get/copy, swapping rectangular shape,
    preserving singular matrix results and swapped zero-cell shapes, yielding
    `na` for invalid cells, and using fixed float-matrix metadata, independent
    backing storage, copy-only continuation, and retained gates for other bound
    producers, broader helpers, mutation, and non-matrix receivers.
51. Bound matrix-eigenvector call results: done for exact numeric square matrix
    receivers using `values.eigenvectors()` followed by
    rows/columns/elements_count/get/copy, preserving real square shape, empty
    `0 x 0` results, and `na` invalid-cell/non-real/incomplete results with
    fixed float-matrix metadata, independent backing storage, copy-only
    continuation, and retained gates for other bound producers, broader
    helpers, mutation, and non-matrix receivers.
52. Bound matrix-multiplication call results: done for exact numeric matrix
    receivers using matrix-valued `values.mult(other)` with matrix or scalar
    operands followed by rows/columns/elements_count/get/copy, preserving
    multiplied or scalar-selected shape, fixed float-matrix results, `na` and
    zero-inner-dimension behavior, independent backing storage, and copy-only
    continuation. Array-result overloads retain array-helper dispatch; UDF
    matrix results, broader helpers, mutation, and non-matrix receivers stay
    gated.
53. Local-UDF matrix call results: done for unqualified local functions whose
    inferred per-call result is one concrete supported matrix kind. Parameter
    passthrough, block aliases, nested calls, same-kind control flow,
    matrix-operation and constructor returns, named/reordered arguments, zero
    dimensions, float/int/bool/string/color interleaving, independent copies,
    and rows/columns/elements_count/get/copy with copy-only continuation are
    fixture-backed. Unknown/`na`, scalar, array, map, remaining user-function
    results, broader helpers, mutation, and terminal-read continuation remain
    fail closed.
54. User-method matrix call results: done for local and imported methods whose
    per-call result is one concrete supported matrix kind. Receiver-style,
    local-type-qualified or alias-qualified, direct-constructor-receiver,
    block/nested/same-kind-control-flow, float/int/bool/string/color, zero-
    dimension, same-library dual-alias, independent-copy, and copy-only-
    continuation paths expose rows/columns/elements_count/get/copy. Unknown/
    `na`, non-matrix or unresolved method results, remaining user-function
    matrix results, broader helpers, mutation, and terminal-read continuation
    remain fail closed.
55. Imported pure-function matrix call results: done for registered imported
    functions whose per-call result is one concrete supported matrix kind.
    Alias-qualified, block/nested/same-kind-control-flow, float/int/bool/string/
    color, zero-dimension, same-library dual-alias, independent-copy, and copy-
    only-continuation paths expose rows/columns/elements_count/get/copy.
    Unknown/`na`, non-matrix, unregistered or unresolved function results,
    broader helpers, mutation, and terminal-read continuation remain fail
    closed.
56. Scalar-map call-result key arrays: done for every existing concrete
    scalar-map producer. `.keys()` returns a fresh key-kind-preserving array
    and switches to size/get/first/last/copy with copy-only array continuation.
    Built-in constructor/copy, local/imported pure-function, local/imported
    user-method, five scalar key kinds, dual-alias, and source-independence paths
    are fixture-backed. Direct `.values()`, map or call-result-array mutation,
    unsupported templates, broader helpers, and terminal key-reader
    continuation remain fail closed.
57. Scalar-map call-result value arrays: done for the same producer set.
    `.values()` returns a fresh value-kind-preserving array and switches to
    size/get/first/last/copy with copy-only array continuation. Built-in
    constructor/copy, local/imported pure-function, local/imported user-method,
    five scalar value kinds, dual-alias, and source-independence paths are
    fixture-backed. Map or call-result-array mutation, unsupported templates,
    broader helpers, and terminal key/value-reader continuation remain fail
    closed.
58. Matrix call-result row arrays: done for every existing concrete matrix
    producer. `.row(index)` returns a fresh element-kind-preserving scalar array
    and switches to size/get/first/last/copy with copy-only array continuation.
    Namespace and bound matrix operations, exact five-scalar `matrix.new<T>`
    templates, local UDFs, local/imported user methods, and imported pure
    functions are fixture-backed across direct binding, copy independence, and
    dual aliases. Bad indexes use the ordinary `matrix.row` checks; `.col()`,
    matrix or call-result-array mutation, broader helpers, and terminal row-
    reader continuation remain fail closed.
59. Matrix call-result column arrays: done for the same concrete matrix producer
    set. `.col(index)` returns a fresh element-kind-preserving scalar array and
    switches to size/get/first/last/copy with copy-only array continuation.
    Namespace and bound matrix operations, exact five-scalar `matrix.new<T>`
    templates, local UDFs, local/imported user methods, imported pure functions,
    direct binding, copy independence, and dual aliases are fixture-backed.
    Bad indexes use the ordinary `matrix.col` checks; matrix or call-result-
    array mutation, broader matrix helpers, and terminal column-reader
    continuation remain fail closed.

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
support boundaries are intentional: element kinds beyond
float/int/bool/string/color, method syntax beyond
`values.fill(value)`, `values.get(row, column)`,
`values.set(row, column, value)`, `values.copy()`,
`values.transpose()`, `values.reverse()`, `values.reshape(rows, columns)`,
`values.row(row)`, `values.col(column)`, `values.add_row(row, array_id)`,
`values.add_col(column, array_id)`, `values.remove_row(row)`,
`values.remove_col(column)`, `values.swap_rows(row1, row2)`,
`values.swap_columns(column1, column2)`, `values.sort(column?, order?)`,
`values.submatrix(from_row?, to_row?, from_column?, to_column?)`,
`values.rows()`, `values.columns()`,
`values.elements_count()`,
`values.sum()`, `values.avg()`, `values.min()`, `values.max()`,
`values.mode()`, `values.trace()`, `values.det()`, `values.rank()`,
`values.is_zero()`, and
`values.is_binary()`, `values.is_diagonal()`, `values.is_identity()`, and
`values.is_symmetric()`, `values.is_antisymmetric()`, and
`values.is_stochastic()`, bare matrix or matrix templates beyond
float/int/bool/string/color typed declarations remain future slices.
`matrix<float>`, `matrix<int>`, `matrix<bool>`, `matrix<string>`, and
`matrix<color>` typed declarations, matching `varip` declarations, namespace
and method-call reshape,
namespace and method-call row/column extraction, namespace and method-call
row/column insertion, namespace and method-call row/column removal, and
committed plus dynamic-offset matrix history snapshots, including dynamic `na`
offset predicates, and statement-form matrix row `for...in` iteration are
fixture-backed.
