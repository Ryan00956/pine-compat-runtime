# Pure Internal UDT Array Design Gate

Status: closed design gate, maintained as the current UDT array support
boundary.

This document defines the internal path for arrays of local or imported
user-defined type values. It is scoped to interpreter internals only: parser,
semantic analysis, HIR lowering, runtime array storage, history, rollback, and
conformance. It does not cover host UI, rendering, remote library lookup, or
public JSON/Python/WASM serialization of UDT array values.

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
Same-local and same-imported scalar-tree UDT array identities are preserved
through ternary, `if`, `switch`, `for`, `for...in`, and `while` results,
including array/`na` branches, block-local aliases, typed or inferred
declarations, and caller-side helper or iteration consumers. Mixed element
identities remain rejected. Local pure UDFs and local user methods may return
same-local scalar-tree UDT arrays, and imported pure exported UDFs and imported
user methods may return same-imported scalar-tree UDT arrays, through direct
parameters, block-local aliases, `array.copy`,
`array.new<T>`/`array.new<alias.Type>`, `array.from`, private nested calls, and
final control-flow results. Direct and alias returns preserve the source array
id, while copy/new/from returns allocate independently. Imported type positions
are rewritten for the active alias, and source-aware return metadata isolates
call sites when the same physical library is imported under multiple aliases.
Interleaved calls over distinct identities therefore retain the correct
A-to-B-to-A element layout. Tuple literals and local/imported UDF or method
tuple returns now preserve a concrete UDT-array identity independently for each
destructured slot through direct, block, nested, final-flow, typed-`na`, typed
destination, A-to-B-to-A, and dual-alias paths. Different identities may occupy
different slots. Tuple-valued ordinary declarations preserve type and identity
metadata through direct and self aliases, ternary/`switch`, assigned-`if`,
fresh shadowing, and later-destructuring paths. A declaration fixes each
UDT-array slot identity: same-identity or `na` reassignment is accepted, while
cross-identity direct/control-flow reassignment and unresolved nested consumers
are rejected at the root span.
Qualified user-defined UDF/method results and unqualified plain local UDF
results returning any currently supported array kind support direct `.size()`,
`.get(index)`, `.first()`, `.last()`, and `.copy()`, including nested copy/read
chains. The unqualified form uses the impossible parser-only `$call_result`
prefix and is admitted only for a plain lexical callee; qualified user-defined
forms retain their alias/type prefix. The completed built-in producer slice in
historical item 19 adds `$builtin_array_result` for its exact `array.*`
producer allowlist and exposes only the same five postfix helpers. Only
`.copy()` may return another array receiver for a nested allowed read/copy;
`.size()`, `.get()`, `.first()`, and `.last()` are terminal and cannot continue
into a user method or other call-result method, including a method on a UDT
element. UDT-array results require one concrete same-local or same-imported
scalar-tree identity. Named/`na`/negative `get` indexes, precise bounds errors,
empty and typed-`na` results, A-to-B-to-A calls, and imported dual aliases are
fixture-backed. Unqualified local UDF results carrying a concrete local or
imported scalar UDT identity may also invoke existing pure user methods; the
built-in producer path does not gain that scalar-method composition. The
lexical prefix `array` is reserved for built-in recognition, so a user/import
qualifier named `array` is not a supported qualified call-result receiver.
At the historical item 19 boundary, other namespaces and templates remained
gated. Historical item 20 later admits only `str.split`,
`ta.pivot_point_levels`, `matrix.row`, `matrix.col`,
`matrix.eigenvalues`, `map.keys`, and `map.values` on the same synthetic path,
with the same five helpers and only `.copy()` nestable. Those producers return
only scalar arrays: row/column snapshots follow the five supported scalar
matrix element kinds, eigenvalues keep the existing numeric-matrix
`array<float>` result, and map key/value snapshots follow the corresponding
five-scalar template kind in insertion order. They add no local/imported UDT
identity. Item 21 additionally admitted namespace-qualified
`matrix.mult(...)` for its matrix-by-array, array-by-matrix, and array-by-array
`array<float>` results. Item 22 routes that namespace-only dynamic candidate
through `$builtin_matrix_result`, retains those five array helpers, and admits
matrix-by-matrix, matrix-by-scalar, and scalar-by-matrix `matrix<float>` results
through only `.rows()`, `.columns()`, `.elements_count()`,
`.get(row, column)`, and `.copy()`. Int inputs still produce float collections,
and only `.copy()` may continue another allowed read/copy chain for either
result kind. Bound or UDF matrix-result helpers, wrong-result or broader
helpers, invalid helper arguments, and mutation remain fail-closed.
Item 23 adds exact namespace `matrix.copy(values)` to the same matrix-result
path. Its `SameAsArg` result preserves the source float/int/bool/string/color
matrix kind and admits the same five matrix helpers with copy-only continuation;
bound `values.copy()` results remain gated. Item 24 adds exact namespace
`matrix.transpose(values)` with the same five element kinds and helpers,
row/column shape swapping, independent storage, and a retained bound
`values.transpose()` gate. Item 25 adds exact namespace
`matrix.submatrix(values, ...)` with preserved element kind, independent
half-open/default-full/empty range copies, the same helpers, and a retained
bound `values.submatrix()` gate. Item 26 adds exact namespace
`matrix.kron(left, right)` with a fixed float-matrix result, expanded shape,
independent storage, `na`/zero-dimension behavior, the same helpers, and a
retained bound `values.kron(other)` gate. Item 27 adds exact namespace
`matrix.diff(left, right)` with a fixed float-matrix result for matrix-matrix
and scalar/matrix operand pairs, selected-matrix shape and left-to-right
direction, the same helpers, and a retained bound `values.diff(other)` gate.
Item 28 adds exact namespace `matrix.pow(values, power)` with a fixed
float-matrix result for numeric square matrices and simple-int powers,
identity/copy/positive-power behavior, the same helpers, and a retained bound
`values.pow(power)` gate.
Item 29 adds exact namespace `matrix.inv(values)` with a fixed float-matrix
result that preserves invertible square shape, returns empty `0 x 0` or `na`
for the established boundaries, exposes the same helpers, and retains the
bound `values.inv()` gate.
Item 30 adds exact namespace `matrix.pinv(values)` with a fixed float-matrix
result that swaps rectangular row/column counts, preserves singular
matrix-valued results, returns swapped zero-cell shapes, exposes the same
helpers, yields `na` for invalid-cell inputs, and retains the bound
`values.pinv()` gate.
Item 31 adds exact namespace `matrix.eigenvectors(values)` with a fixed
float-matrix result that preserves square shape for real complete eigenvector
columns, returns empty `0 x 0`, yields `na` for invalid-cell/non-real/incomplete
results, exposes the same helpers, and retains the bound
`values.eigenvectors()` gate plus non-square runtime error.
Item 32 adds exact `matrix.new<float|int|bool|string|color>` template results
with preserved element kind, requested rectangular shape, type-compatible
initial or default `na` cells, fresh allocation, the same helpers, and retained
unsupported-template and mutation gates.
Item 33 adds exact supported scalar `map.new<K,V>` template results through a
separate `$builtin_map_result` path with known key/value kinds, fresh empty
allocation, direct size/get/contains/copy, copy-only continuation, and retained
mutation, keys/values, unsupported-template, and other map-result gates.
Item 34 adds exact namespace `map.copy(existing)` results through the same path,
retaining the source scalar template and entries in independent backing storage
while preserving the same helper, continuation, non-map-input, mutation, and
keys/values gates.
Item 35 adds exact bound matrix-receiver `values.copy()` results with preserved
float/int/bool/string/color element kind, shape, independent backing storage,
the five direct matrix read/copy helpers, copy-only continuation, and retained
other-bound-producer/non-matrix/mutation gates.
Item 36 adds exact bound matrix-receiver `values.transpose()` results with the
same five helpers, preserved element kind, swapped shape, independent backing
storage, copy-only continuation, and retained other-bound-producer/non-matrix/
mutation gates.
Item 37 adds exact bound matrix-receiver `values.submatrix(...)` results with
the same five helpers, preserved element kind, selected/default/empty half-open
ranges, independent backing storage, copy-only continuation, and retained
other-bound-producer/non-matrix/mutation gates.
Item 38 adds exact bound numeric-matrix-receiver `values.kron(other)` results
with the same five helpers, expanded shape, fixed float-matrix result kind,
independent backing storage, copy-only continuation, and retained operand/
other-bound-producer/non-matrix/mutation gates.
Item 39 adds exact bound numeric-matrix-receiver `values.diff(other)` results
for matrix or scalar operands with the same five helpers, selected matrix
shape, operand direction, fixed float-matrix result kind, independent backing
storage, copy-only continuation, and retained operand/other-bound-producer/
non-matrix/mutation gates.
Item 40 adds exact bound numeric-square-matrix-receiver `values.pow(power)`
results with the same five helpers, identity/copy/positive-power behavior,
fixed float-matrix result kind, independent backing storage, copy-only
continuation, and retained power/other-bound-producer/non-matrix/mutation
gates.
Item 41 adds exact bound numeric-square-matrix-receiver `values.inv()` results
with the same five helpers, preserved invertible square shape, empty `0 x 0`
and `na` singular/invalid-cell boundaries, fixed float-matrix result kind,
independent backing storage, copy-only continuation, and retained
other-bound-producer/non-matrix/mutation gates.
Item 42 adds exact bound numeric-matrix-receiver `values.pinv()` results with
the same five helpers, swapped rectangular shape, singular matrix results,
swapped zero-cell shapes, `na` invalid-cell boundaries, fixed float-matrix
result kind, independent backing storage, copy-only continuation, and retained
other-bound-producer/non-matrix/mutation gates.
Item 43 adds exact bound numeric-square-matrix-receiver
`values.eigenvectors()` results with the same five helpers, preserved real
square shape, empty `0 x 0` and `na` invalid/non-real/incomplete boundaries,
fixed float-matrix result kind, independent backing storage, copy-only
continuation, and retained other-bound-producer/non-matrix/mutation gates.
Item 44 adds exact bound numeric-matrix-receiver matrix-valued
`values.mult(other)` results for matrix or scalar operands with the same five
helpers, multiplied or scalar-selected shape, fixed float-matrix result kind,
`na`/zero-inner-dimension behavior, independent backing storage, copy-only
continuation, and retained array-result/UDF/non-matrix/mutation gates.
Outside the exact static producer sets and these namespace-only paths,
unsupported `array.new<T>` element families, non-producer calls, map/matrix
unsupported matrix templates and map templates, and other matrix-returning
calls remain fail-closed. `array.slice`
remains a live parent view, while a
postfix `.copy()` independently captures its current values. `array.concat`
still mutates and returns its first array id; a following reader is itself
non-mutating, but concat remains rejected inside UDFs.
Local UDFs and typed local user methods may also iterate a generic same-local
scalar-tree UDT-array parameter. Value-only and index/value statement loops,
block-local array aliases, and final expression-form loops preserve the
call-local element identity for field/scalar results, a returned UDT element,
or a same-identity UDT array rebuilt from that element, including named method
arguments and interleaved A-to-B-to-A calls.

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
- `tests/fixtures/runtime/user_type_array_scalar_tree.pine` and
  `tests/fixtures/runtime/import_udt_array_scalar_tree.pine` cover local and
  imported scalar-tree UDT array identities returned by control-flow
  expressions. The matching supported and mixed-identity semantic fixtures
  cover ternary, `if`, `switch`, `for`, `for...in`, and `while`, array/`na`
  branches, aliases, typed declarations, helper calls, and iteration. The local
  runtime fixture also interleaves generic UDF calls over UDTs with different
  field orders to lock per-call lowering identity for namespace/method element
  helpers and `array.from` reconstruction.
- `tests/fixtures/runtime/user_type_array_scalar_tree.pine` also covers local
  UDF and user-method UDT array returns through direct parameters, block-local
  aliases, copies, fresh construction, nested calls, named arguments, and final
  control flow. It interleaves `First`, `Second`, then `First` calls whose
  scalar fields use different declaration orders, locking call-specific return
  identity as well as alias-versus-copy/fresh backing-store behavior.
  `tests/fixtures/sema/supported_user_type_array_udf_method_returns.pine` keeps
  the accepted return shapes fixture-backed, while
  `tests/fixtures/sema/unsupported_user_type_array_udf_method_return_identities.pine`
  rejects mixed return branches and incompatible typed destinations.
- `tests/fixtures/runtime/user_type_array_scalar_tree.pine` and
  `tests/fixtures/sema/supported_user_type_array_udf_method_returns.pine` cover
  qualified same-local user-method and unqualified local UDF result
  `.size()`/`.get(index)`/`.first()`/`.last()`/`.copy()`, separate A-to-B-to-A
  sequences, generic wrappers over UDT and scalar arrays, receiver/type-qualified
  and named-index forms, empty/typed-`na` reads, negative indexes, explicit
  same-named local helpers, a local UDF named `array`, scalar-UDT result method
  dispatch, bounds errors, and copy independence;
- `tests/fixtures/runtime/import_udt_array_udf_method_returns.pine` and
  `tests/fixtures/sema/supported_imported_user_type_array_udf_method_returns.pine`
  cover imported same-scalar-tree UDF and user-method array returns through
  direct and block-alias returns, copy/new/from allocation, private nested calls,
  final control flow, typed method arguments, imported type-position rewrites,
  and same-library dual-alias call-site isolation. The shared definitions live
  in `tests/fixtures/libraries/import_udt_array_return_lib.pine`.
  `tests/fixtures/sema/unsupported_imported_user_type_array_udf_method_return_identities.pine`
  preserves the scalar-return mixed-identity boundary.
- `tests/fixtures/runtime/user_type_array_tuple_returns.pine`,
  `tests/fixtures/sema/supported_user_type_array_tuple_returns.pine`,
  `tests/fixtures/runtime/import_udt_array_tuple_returns.pine`, and
  `tests/fixtures/sema/supported_imported_user_type_array_tuple_returns.pine`
  cover per-slot tuple-return identity, distinct identities in different slots,
  function-local tuple destructuring, tuple-valued declaration aliases through
  ternary/`switch`/assigned-`if` and later destructuring, typed-`na`, A-to-B-to-A calls, and
  same-library dual aliases. The matching local/imported identity fixtures
  reject conflicts within one slot;
  `tests/fixtures/sema/unsupported_user_type_array_tuple_alias_mutation.pine`
  and
  `tests/fixtures/sema/unsupported_imported_user_type_array_tuple_alias_mutation.pine`
  lock the stable-slot reassignment and root-span boundaries.
- `tests/fixtures/runtime/import_udt_array_udf_method_returns.pine` and
  `tests/fixtures/sema/supported_imported_user_type_array_udf_method_returns.pine`
  cover qualified imported UDF/method and unqualified root-local wrapper result
  `.size()`/`.get(index)`/`.first()`/`.last()`/`.copy()`, nested copy/read
  chains, A-to-B-to-A, dual aliases, named and negative indexes,
  empty/typed-`na` reads, explicit same-named export or scalar-UDT method
  dispatch, private library-local UDF postfix normalization, and copy
  independence. The imported call-result negative fixture retains the
  broader-helper boundary, while
  `tests/fixtures/sema/unsupported_local_user_type_array_call_result_chaining.pine`
  retains mixed/unknown/non-array/other-helper boundaries and prevents silent
  dispatch to a same-named local user method.
- `tests/fixtures/runtime/user_type_array_scalar_tree.pine` and
  `tests/fixtures/sema/supported_user_type_array_param_for_in.pine` cover
  value-only and index/value statement `for...in`, block-local array aliases,
  final expression-form scalar/field results, final UDT-element results, and
  same-identity UDT arrays rebuilt from the loop element inside local UDFs,
  plus typed methods and named method arguments. Repeated `First`, `Second`,
  then `First` calls lock the fresh loop-value identity and returned element or
  array layout to each call.
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
- passing same-local scalar-tree UDT array ids to local pure UDFs and local user
  methods, or same-imported scalar-tree UDT array ids to imported pure exported
  UDFs and imported user methods, is fixture-backed, including direct, alias,
  copy, constructor, private nested call, and final control-flow return paths;
- direct and alias returns preserve the source array id, while `array.copy`,
  `array.new<T>`, and `array.from` return independently allocated array ids;
- returned identities are resolved from the current call arguments rather than
  shared function-body spans, including interleaved A-to-B-to-A calls across
  same-shaped UDTs with different field order and same-library dual-alias calls;
- imported type positions are rewritten to the active alias, while source-aware
  expression metadata keeps separate import instances isolated;
- local UDFs and typed local user methods may use value-only or index/value
  `for...in` over same-local scalar-tree UDT-array parameters, including
  block-local aliases and final expression results that return a field/scalar
  value, the UDT element itself, or a same-identity UDT array rebuilt from that
  element; loop-value identity remains call-local;
- mutating UDT arrays inside user-defined functions stays unsupported in the
  first positive slice;
- mutating fields inside methods stays unsupported, including fields on UDT
  values read from arrays;
- local pure methods on UDT values read from arrays are fixture-backed after the
  value is bound to a local variable;
- receiver-style scalar-tree imported UDT methods and alias-qualified imported
  method calls over same-imported scalar-tree UDT values read from arrays are
  supported, with mismatched local/imported identities rejected.
- local/imported UDF or method same-scalar-tree UDT array returns are
  fixture-backed; their tuple returns preserve identity per destructured slot,
  and qualified user-defined or unqualified plain local UDF results returning a
  supported array kind support direct
  `.size()`/`.get(index)`/`.first()`/`.last()`/`.copy()` chaining. Concrete
  scalar UDT results from unqualified local UDFs may invoke existing pure user
  methods. The exact built-in `array.*` producer allowlist and cross-namespace
  array-capable path support those same five helpers, but only `.copy()` may
  continue a nested array chain and terminal element reads cannot invoke UDT
  methods. The later seven fixed cross-namespace producers and array-returning
  `matrix.mult` overloads return scalar arrays only and add no UDT/import
  identity. Namespace matrix-returning `matrix.mult` overloads and exact
  namespace `matrix.copy`/`matrix.transpose`/`matrix.submatrix` plus fixed-float
  namespace `matrix.kron`/`matrix.diff`/`matrix.pow`/`matrix.inv`/
  `matrix.pinv`/`matrix.eigenvectors` add only the exact five matrix
  readers/copy from items 22 through 31 and likewise carry no UDT/import
  identity. The five exact `matrix.new<T>` templates add the same readers/copy
  in item 32 while retaining their scalar element kind and likewise add no
  UDT/import identity. The exact scalar `map.new<K,V>` result path in item 33
  and namespace `map.copy(existing)` result path in item 34 carry only map
  template metadata and add no UDT/import identity. Mixed identities within one
  scalar return or tuple slot, non-scalar UDT arrays,
  non-array/non-UDT results,
  unknown/`na` results without a concrete supported type or identity,
  bound matrix-result receivers other than exact matrix-receiver
  `values.copy()`/`values.transpose()`/`values.submatrix(...)`/
  `values.kron(other)`/`values.diff(other)`/`values.pow(power)`/
  `values.inv()`/`values.pinv()`/`values.eigenvectors()`/
  matrix-valued `values.mult(other)`, UDF matrix-result receivers,
  built-in-qualified/template call
  receivers outside the exact static and dynamic paths, mutation side effects,
  and other direct array or matrix methods on call results remain
  semantic/parser/lowering boundaries. In particular, concat remains rejected
  inside UDFs even when followed by an allowed reader.

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
11. Local UDF and user-method returns preserve same-local scalar-tree UDT array
    identity through direct, alias, copy, constructor, nested-call, and final
    control-flow paths with call-specific A-to-B-to-A lowering. Done.
12. Local UDF and typed-method `for...in` over same-local scalar-tree UDT-array
    parameters preserves fresh value-loop identity for value-only/index-value
    statements and final scalar/field, UDT-element, or element-rebuilt UDT-array
    results, including aliases, named arguments, and A-to-B-to-A calls. Done.
13. Imported UDF and user-method returns preserve same-imported scalar-tree UDT
    array identity through direct, alias, copy/new/from, private nested, typed
    method, and final-control-flow paths. Source-aware type-position rewrites and
    import-instance metadata isolate the same physical library under two aliases.
    Done.
14. Tuple literals and local/imported UDF or user-method tuple returns preserve
    same-local or same-imported scalar-tree UDT-array identity independently per
    destructured slot, including direct/block/nested/final-flow, typed-`na`,
    typed-destination, tuple-valued ordinary declaration direct/self alias,
    control, shadow, and destructuring paths, A-to-B-to-A, and dual-alias
    paths. Same-identity or `na` reassignment preserves the fixed slot layout;
    conflicting identities in one slot or cross-identity direct/control-flow
    reassignment fail closed with `E_TUPLE_UDT_ARRAY_IDENTITY`. Done.
15. Qualified imported UDF/method results carrying a concrete same-imported
    scalar-tree UDT-array identity lower direct `.first()`/`.copy()` and nested
    `.copy().first()` through the array helper path, preserve A-to-B-to-A and
    dual-alias identity, keep copies independent, and do not hijack explicit
    same-named imported functions. Done.
16. Qualified same-local user-method results carrying a concrete scalar-tree
    UDT-array identity lower direct `.first()`/`.copy()` and nested
    `.copy().first()` through the same helper path. Receiver/type-qualified,
    A-to-B-to-A, generic-wrapper, explicit same-named method, and copy
    independence paths are fixture-backed. Done. At this historical slice
    boundary, unqualified local UDF results and broader direct helpers remained
    deferred.
17. The qualified same-local/imported call-result path extends the read-only
    helper set with `.size()`, `.get(index)`, and `.last()`. Simple-int and UDT
    element return types, named/`na`/negative indexes, precise bounds errors,
    empty and typed-`na` results, nested copy/read chains, A-to-B-to-A and
    dual-alias isolation, generic wrappers, and explicit same-named local,
    imported, or scalar-UDT dispatch controls are fixture-backed. Done. At this
    historical slice boundary, broader helpers, unqualified local UDF results,
    mixed/non-scalar identities, and call-result mutation remained gated.
18. Unqualified plain local UDF call-result receivers normalize through the
    impossible parser-only `$call_result` prefix. Results returning any
    currently supported array kind share the read-only
    `.size()`/`.get(index)`/`.first()`/`.last()`/`.copy()` lowering path with
    qualified user-defined results; concrete same-local/same-imported
    scalar-tree identity remains mandatory for UDT arrays. Concrete scalar UDT
    results may invoke existing pure user methods. Plain-callee validation keeps
    local UDFs named after built-in namespaces unambiguous. At this historical
    slice boundary, built-in-qualified/template call results remained
    parser-gated. Mixed or non-scalar UDT-array identities, non-array/non-UDT
    results, unknown/`na` results without a concrete supported type or identity,
    other array helpers, and call-result mutation remained rejected. Done.
19. Exact built-in array producers normalize through the separate
    `$builtin_array_result` prefix. The exact admitted producer set is
    `array.new_float`, `array.new_int`, `array.new_bool`, `array.new_string`,
    `array.new_color`, `array.new_line`, `array.new_linefill`,
    `array.new_polyline`, `array.new_label`, `array.new_box`,
    `array.new_table`, `array.new<chart.point>`, supported `array.new<UDT>`,
    `array.from`, `array.copy`, `array.slice`, `array.concat`, `array.abs`,
    `array.standardize`, and `array.sort_indices`; existing supported
    scalar/drawing-id/`chart.point`
    and concrete same-local/same-imported scalar-tree UDT `array.new<T>` source
    forms use their canonical constructor or checked UDT-template path. Only
    `.size()`, `.get(index)`, `.first()`, `.last()`, and `.copy()` may follow a
    producer. Only `.copy()` may return another array receiver for a nested
    allowed read/copy; the four readers are terminal and cannot invoke a user
    method or another call-result method, including on a returned UDT element.
    Producer arguments, supported array kind, and concrete scalar-tree UDT
    identity are revalidated and fail closed. Other `array.*` members, other
    namespaces/templates, unsupported UDT templates, and postfix mutation stay
    gated. The lexical `array` prefix is reserved for built-in recognition, so
    a user/import qualifier of that name is not a supported qualified
    call-result path. `array.slice` keeps live-view semantics and postfix
    `.copy()` snapshots its current window independently. `array.concat`
    mutates and returns its first input; its postfix reader is non-mutating, but
    concat remains rejected inside UDFs. Done.
20. A later cross-namespace scalar-array producer slice reuses
    `$builtin_array_result` for exactly `str.split`,
    `ta.pivot_point_levels`, `matrix.row`, `matrix.col`,
    `matrix.eigenvalues`, `map.keys`, and `map.values`. Each exposes only
    `.size()`, `.get(index)`, `.first()`, `.last()`, and `.copy()`; only
    `.copy()` may continue to another allowed read/copy. Row and column results
    are independent arrays matching the float/int/bool/string/color matrix
    element kind, eigenvalues retain the existing independent `array<float>`
    result for supported numeric matrices, and map key/value results are
    independent insertion-order arrays matching the corresponding
    int/float/bool/string/color template side. Empty/`na`, negative-index,
    bounds, typed destinations, UDF reads, and copy independence are
    fixture-backed. Namespace-qualified `matrix.mult(...)` direct-result
    chains, matrix-returning calls, unsupported matrix templates and map templates, all other
    namespaces/non-producers, and postfix mutation remain gated; the existing
    bound-receiver `matrix_id.mult(array).size()` path is unchanged. Built-in
    prefixes remain reserved. This scalar-only slice adds no
    UDT/import identity and no public schema field. Done.
21. The conditional `matrix.mult` result slice admits the namespace call as a
    parser candidate, then relies on `ReturnSpec::MatrixMult` to expose the
    five read/copy helpers only for matrix-by-array, array-by-matrix, and
    array-by-array results. All three resolve to `array<float>`, including int
    inputs. Matrix-by-matrix, matrix-by-scalar, and scalar-by-matrix resolve to
    `matrix<float>` and retain the generic direct call-result rejection. Only
    `.copy()` may continue a chain; invalid indexes, other helpers, mutation,
    empty/`na` values, typed destinations, UDF reads, and nested-copy
    independence are fixture-backed. The existing bound-receiver
    `matrix_id.mult(array).size()` path remains unchanged. This slice adds no
    UDT/import identity and no public schema field. Done.
22. The namespace matrix-result continuation routes `matrix.mult(...)` through
    `$builtin_matrix_result` and keeps result-type-directed dispatch. The three
    array-returning overload families retain the item 21 five-helper contract.
    Matrix-by-matrix, matrix-by-scalar, and scalar-by-matrix resolve to
    `matrix<float>` and expose only `.rows()`, `.columns()`,
    `.elements_count()`, `.get(row, column)`, and `.copy()`, including int-input
    float-result matrices. Only `.copy()` may continue another allowed
    read/copy chain. Wrong-result helpers, bad helper arity or types, mutation,
    and broader helpers fail closed; bound or UDF matrix-result receivers retain
    the generic direct call-result rejection, while the existing bound
    `matrix_id.mult(array).size()` path is unchanged. Empty/`na` values, typed
    destinations, UDF-contained namespace calls, copy independence, and the
    retained boundaries are fixture-backed. No UDT/import identity or public
    schema field is added. Done.
23. The exact namespace matrix-copy continuation routes `matrix.copy(values)`
    through `$builtin_matrix_result`. Its `SameAsArg` result preserves all five
    supported scalar matrix element kinds and exposes only `.rows()`,
    `.columns()`, `.elements_count()`, `.get(row, column)`, and `.copy()`, with
    named helper arguments and copy-only continuation. Empty/`na`, nested-copy,
    UDF-contained namespace reads, and source/copy independence are
    fixture-backed. Wrong receivers, invalid helper arguments, mutation,
    broader helpers, and bound `values.copy()` call-result reads fail closed.
    No UDT/import identity or public schema field is added. Done.
24. The exact namespace matrix-transpose continuation routes
    `matrix.transpose(values)` through `$builtin_matrix_result`. Its `SameAsArg`
    result preserves all five supported scalar element kinds while swapping
    row/column shape and allocating independent storage. It exposes only the
    five matrix read/copy helpers with named arguments and copy-only
    continuation. Zero dimensions, `na`, coordinate mapping, nested copies,
    UDF-contained namespace reads, and source independence are fixture-backed.
    Wrong receivers, invalid helper arguments, mutation, broader helpers, and
    bound `values.transpose()` call-result reads fail closed. No UDT/import
    identity or public schema field is added. Done.
25. The exact namespace matrix-submatrix continuation routes
    `matrix.submatrix(values, ...)` through `$builtin_matrix_result`. Its
    `SameAsArg` result preserves all five supported scalar element kinds while
    returning independent half-open ranges with default full bounds and empty
    row/column slices. It exposes only the five matrix read/copy helpers with
    named arguments and copy-only continuation. `na`, coordinate mapping,
    nested copies, UDF-contained namespace reads, and source independence are
    fixture-backed. Wrong producer/helper arguments, wrong receivers, mutation,
    broader helpers, and bound `values.submatrix()` call-result reads fail
    closed. No UDT/import identity or public schema field is added. Done.
26. The exact namespace matrix-kron continuation routes
    `matrix.kron(left, right)` through `$builtin_matrix_result`. Its fixed
    `simple matrix<float>` result accepts numeric matrix operands, expands both
    source dimensions, and exposes only the five matrix read/copy helpers with
    named arguments and copy-only continuation. Int-input float results, `na`,
    zero rows/columns, nested copies, UDF-contained namespace reads, and source
    independence are fixture-backed. Wrong producer/helper arguments,
    mutation, broader helpers, and bound `values.kron(other)` call-result reads
    fail closed. No UDT/import identity or public schema field is added. Done.
27. The exact namespace matrix-diff continuation routes
    `matrix.diff(left, right)` through `$builtin_matrix_result`. Its fixed
    `simple matrix<float>` result accepts matrix-matrix, matrix-scalar, and
    scalar-matrix numeric operands, preserves the selected matrix shape and
    left-to-right subtraction order, and exposes only the five matrix read/copy
    helpers with named arguments and copy-only continuation. Int-input float
    results, `na`, zero rows/columns, nested copies, UDF-contained namespace
    reads, and source independence are fixture-backed. Wrong producer/helper
    arguments, mutation, broader helpers, and bound `values.diff(other)`
    call-result reads fail closed. No UDT/import identity or public schema field
    is added. Done.
28. The exact namespace matrix-power continuation routes
    `matrix.pow(values, power)` through `$builtin_matrix_result`. Its fixed
    `simple matrix<float>` result accepts numeric square matrices and simple-int
    powers, preserves independent identity/copy/positive-power results, and
    exposes only the five matrix read/copy helpers with named arguments and
    copy-only continuation. Int-input float results, `na`, empty `0 x 0`,
    nested copies, UDF-contained namespace reads, and source independence are
    fixture-backed. Wrong producer/helper arguments, mutation, broader helpers,
    and bound `values.pow(power)` call-result reads fail closed. No UDT/import
    identity or public schema field is added. Done.
29. The exact namespace matrix-inverse continuation routes `matrix.inv(values)`
    through `$builtin_matrix_result`. Its fixed `simple matrix<float>` result
    preserves invertible square shape, returns an empty `0 x 0` matrix for
    empty input and `na` for singular or invalid-cell inputs, and exposes only
    the five matrix read/copy helpers with named arguments and copy-only
    continuation. Int-input float results, nested copies, UDF-contained
    namespace reads, and source independence are fixture-backed. Wrong
    producer/helper arguments, mutation, broader helpers, and bound
    `values.inv()` call-result reads fail closed. No UDT/import identity or
    public schema field is added. Done.
30. The exact namespace matrix-pseudo-inverse continuation routes
    `matrix.pinv(values)` through `$builtin_matrix_result`. Its fixed
    `simple matrix<float>` result swaps rectangular row/column counts,
    preserves singular matrix-valued results, returns swapped zero-cell shapes
    for zero-row or zero-column inputs, and yields `na` for invalid-cell
    inputs. It exposes only the five matrix read/copy helpers with named
    arguments and copy-only continuation. Int-input float results, nested
    copies, UDF-contained namespace reads, and source independence are
    fixture-backed. Wrong producer/helper arguments, mutation, broader
    helpers, and bound `values.pinv()` call-result reads fail closed. No
    UDT/import identity or public schema field is added. Done.
31. The exact namespace matrix-eigenvector continuation routes
    `matrix.eigenvectors(values)` through `$builtin_matrix_result`. Its fixed
    `simple matrix<float>` result preserves square shape for real complete
    eigenvector columns, returns an empty `0 x 0` matrix for empty input, and
    yields `na` for invalid-cell, non-real, or incomplete results. It exposes
    only the five matrix read/copy helpers with named arguments and copy-only
    continuation. Int-input float results, nested copies, UDF-contained
    namespace reads, and source independence are fixture-backed. Wrong
    producer/helper arguments, mutation, broader helpers, and bound
    `values.eigenvectors()` call-result reads fail closed; non-square runtime
    errors are unchanged. No UDT/import identity or public schema field is
    added. Done.
32. The exact matrix-constructor-template continuation routes
    `matrix.new<float>`, `matrix.new<int>`, `matrix.new<bool>`,
    `matrix.new<string>`, and `matrix.new<color>` results through
    `$builtin_matrix_result`. Each preserves its element kind, requested
    rectangular shape, type-compatible initial or default `na` cells, fresh
    allocation, and copy independence, and exposes only the five matrix
    read/copy helpers with named arguments and copy-only continuation. Zero
    dimensions, nested copies, UDF-contained template reads, and fresh-source
    behavior are fixture-backed. Invalid constructor/helper arguments,
    mutation, broader helpers, and unsupported/deferred templates fail closed.
    No UDT/import identity or public schema field is added. Done.
33. The exact scalar-map-constructor continuation routes supported
    `map.new<K,V>` templates through `$builtin_map_result`, where both `K` and
    `V` are int, float, bool, string, or color. Fresh empty maps retain their
    concrete key/value kinds and expose only `.size()`, `.get(key)`,
    `.contains(key)`, and `.copy()` with named arguments and copy-only
    continuation. All 25 template pairs, missing reads, nested copies,
    copy-then-mutate behavior, fresh allocation, and UDF-contained reads are
    fixture-backed. Wrong key/arity, mutation, direct `keys()`/`values()`,
    unsupported templates, broader helpers, and other map-result receivers
    fail closed. No UDT/import identity or public schema field is added. Done.
34. The exact namespace-map-copy continuation routes `map.copy(existing)`
    through `$builtin_map_result`. The result retains the source scalar
    key/value kinds and populated entries in independent backing storage and
    exposes only `.size()`, `.get(key)`, `.contains(key)`, and `.copy()` with
    named arguments and copy-only continuation. Populated reads, nested copy,
    source/copy independence, multiple scalar templates, and UDF-contained
    reads are fixture-backed. Wrong receiver/key/arity, mutation, direct
    `keys()`/`values()`, broader helpers, and other map-result receivers fail
    closed. No UDT/import identity or public schema field is added. Done.
35. The exact bound-matrix-copy continuation recognizes `values.copy()` only
    when `values` resolves to a supported concrete matrix kind. The result
    retains element kind, shape, and independent backing storage and exposes
    only rows/columns/elements_count/get/copy with copy-only continuation.
    Float/int/bool/string/color receivers, nested copy, UDF-contained reads,
    wrong indexes, broader helpers, non-matrix receivers, and the retained
    bound-transpose gate are fixture-backed. No UDT/import identity or public
    schema field is added. Done.
36. The exact bound-matrix-transpose continuation recognizes
    `values.transpose()` only when `values` resolves to a supported concrete
    matrix kind. The result retains element kind, swaps row/column shape, uses
    independent backing storage, and exposes only
    rows/columns/elements_count/get/copy with copy-only continuation. Five
    element kinds, nested copy, UDF-contained reads, wrong indexes, broader
    helpers, non-matrix receivers, and the retained bound-submatrix gate are
    fixture-backed. No UDT/import identity or public schema field is added.
    Done.
37. The exact bound-matrix-submatrix continuation recognizes
    `values.submatrix(...)` only when `values` resolves to a supported concrete
    matrix kind. The result retains element kind, selects an independent
    half-open range including default full and valid empty ranges, and exposes
    only rows/columns/elements_count/get/copy with copy-only continuation. Five
    element kinds, nested copy, UDF-contained reads, wrong ranges/indexes,
    broader helpers, non-matrix receivers, and the retained bound-kron gate are
    fixture-backed. No UDT/import identity or public schema field is added.
    Done.
38. The exact bound-matrix-Kronecker continuation recognizes
    `values.kron(other)` only when `values` resolves to a supported numeric
    matrix kind. The result expands both dimensions, uses independent fixed
    float-matrix storage, and exposes only
    rows/columns/elements_count/get/copy with copy-only continuation. Float/int
    operands, nested copy, UDF-contained reads, wrong operands/indexes, broader
    helpers, non-numeric/non-matrix receivers, and the retained bound-diff gate
    are fixture-backed. No UDT/import identity or public schema field is added.
    Done.
39. The exact bound-matrix-difference continuation recognizes
    `values.diff(other)` only when `values` resolves to a supported numeric
    matrix kind and `other` is a numeric matrix or scalar. The result preserves
    left-to-right direction and selected matrix shape, uses independent fixed
    float-matrix storage, and exposes only rows/columns/elements_count/get/copy
    with copy-only continuation. Matrix/scalar operands, nested copy,
    UDF-contained reads, wrong operands/indexes, broader helpers,
    non-numeric/non-matrix receivers, and the retained bound-pow gate are
    fixture-backed. No UDT/import identity or public schema field is added.
    Done.
40. The exact bound-matrix-power continuation recognizes `values.pow(power)`
    only when `values` resolves to a supported numeric square matrix kind and
    `power` is simple int. The result preserves square shape across identity,
    copy, and positive powers, uses independent fixed float-matrix storage, and
    exposes only rows/columns/elements_count/get/copy with copy-only
    continuation. Float/int receivers, nested copy, UDF-contained reads, wrong
    powers/indexes, broader helpers, non-numeric/non-matrix receivers, and the
    retained bound-inverse gate are fixture-backed. No UDT/import identity or
    public schema field is added. Done.
41. The exact bound-matrix-inverse continuation recognizes `values.inv()` only
    when `values` resolves to a supported numeric square matrix kind. The
    result preserves invertible square shape, returns empty `0 x 0` or `na` at
    the established boundaries, uses independent fixed float-matrix storage,
    and exposes only rows/columns/elements_count/get/copy with copy-only
    continuation. Float/int receivers, nested copy, UDF-contained reads,
    wrong indexes, broader helpers, non-numeric/non-matrix receivers, and the
    retained bound-pseudo-inverse gate are fixture-backed. No UDT/import
    identity or public schema field is added. Done.
42. The exact bound-matrix-pseudo-inverse continuation recognizes
    `values.pinv()` only when `values` resolves to a supported numeric matrix
    kind. The result swaps rectangular shape, preserves singular matrix results
    and swapped zero-cell shapes, returns `na` for invalid-cell inputs, uses
    independent fixed float-matrix storage, and exposes only
    rows/columns/elements_count/get/copy with copy-only continuation. Float/int
    receivers, nested copy, UDF-contained reads, wrong indexes, broader
    helpers, non-numeric/non-matrix receivers, and the retained bound
    `values.eigenvectors()` gate are fixture-backed. No UDT/import identity or
    public schema field is added. Done.
43. The exact bound-matrix-eigenvector continuation recognizes
    `values.eigenvectors()` only when `values` resolves to a supported numeric
    square matrix kind. The result preserves real square shape, returns empty
    `0 x 0` or `na` at the established boundaries, uses independent fixed
    float-matrix storage, and exposes only rows/columns/elements_count/get/copy
    with copy-only continuation. Float/int receivers, nested copy,
    UDF-contained reads, wrong indexes, broader helpers, non-numeric/non-matrix
    receivers, and the retained matrix-valued bound `values.mult(other)` gate
    are fixture-backed. No UDT/import identity or public schema field is added.
    Done.
44. The exact bound-matrix-multiplication continuation recognizes matrix-valued
    `values.mult(other)` only when `values` resolves to a supported numeric
    matrix kind and `other` is a numeric matrix or scalar. The result preserves
    multiplied or scalar-selected shape, `na` and zero-inner-dimension
    behavior, uses independent fixed float-matrix storage, and exposes only
    rows/columns/elements_count/get/copy with copy-only continuation.
    Matrix-array overloads retain array-helper dispatch. Float/int operands,
    nested copy, UDF-contained reads, wrong result helpers/indexes,
    non-numeric/non-matrix receivers, and the retained UDF matrix-result gate
    are fixture-backed. No UDT/import identity or public schema field is added.
    Done.

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
with incremental positive runtime slices. The current supported subset includes
the same-local and same-imported scalar-tree UDT `array.new<T>()`/`array.from`
paths with the helper set listed in Current Boundary and Accepted Forms,
ordinary `var` realtime rollback, fixture-backed scalar-tree UDT array `varip`,
typed `array<T>`/`T[]` declarations, and local or imported UDF/user-method array
returns within the call-boundary subset above. Qualified user-defined and
unqualified plain local UDF results returning any supported array kind share
the direct `.size()`/`.get(index)`/`.first()`/`.last()`/`.copy()` path;
UDT-array results still require a concrete same-local/same-imported scalar-tree
identity, and scalar UDT local-UDF results may invoke existing pure methods.
The exact built-in array producer allowlist in item 19 also shares those five
helpers through `$builtin_array_result`, with only `.copy()` nestable and
terminal element readers unable to invoke UDT methods. Item 20 adds the exact
seven cross-namespace scalar-array producers under the same five-helper and
copy-only continuation rule, without adding UDT/import identity. Item 21 adds
the array-returning `matrix.mult` overloads, item 22 adds the exact
namespace-only `matrix.mult` matrix-result read/copy set, and item 23 adds exact
namespace `matrix.copy` through `$builtin_matrix_result`; none adds UDT/import
identity. Item 24 adds exact namespace `matrix.transpose` on the same path with
shape swapping, item 25 adds exact namespace `matrix.submatrix` with range
copies, and item 26 adds fixed-float namespace `matrix.kron` with expanded
shape. Item 27 adds fixed-float namespace `matrix.diff` with selected-matrix
shape and operand direction, and item 28 adds fixed-float namespace `matrix.pow`
with identity/copy/positive powers. Item 29 adds fixed-float namespace
`matrix.inv` with invertible-square, empty, and `na` result boundaries; none
adds UDT/import identity. Item 30 adds fixed-float namespace `matrix.pinv` with
rectangular shape swapping, singular matrix-valued results, zero-cell shape
swapping, and invalid-cell `na`; it likewise adds no UDT/import identity.
Item 31 adds fixed-float namespace `matrix.eigenvectors` with square-shape real
complete results, empty `0 x 0`, and invalid/non-real/incomplete `na`; it also
adds no UDT/import identity.
Item 32 adds the five exact scalar `matrix.new<T>` templates with preserved
element kind, rectangular shape, initial/default-`na` cells, and fresh
allocation; it likewise adds no UDT/import identity.
Item 33 adds exact scalar `map.new<K,V>` results with map template metadata only
and likewise adds no UDT/import identity.
Item 34 adds exact namespace `map.copy(existing)` results with the same map
template metadata and retained copied entries; it likewise adds no UDT/import
identity.
Item 35 adds exact bound matrix-receiver `values.copy()` result reads while
carrying only the concrete matrix element kind and no UDT/import identity.
Item 36 adds exact bound matrix-receiver `values.transpose()` result reads with
the same element-kind-only metadata and no UDT/import identity.
Item 37 adds exact bound matrix-receiver `values.submatrix(...)` result reads
with the same element-kind-only metadata and no UDT/import identity.
Item 38 adds exact bound numeric-matrix-receiver `values.kron(other)` result
reads with fixed float-matrix metadata and no UDT/import identity.
Item 39 adds exact bound numeric-matrix-receiver `values.diff(other)` result
reads with fixed float-matrix metadata and no UDT/import identity.
Item 40 adds exact bound numeric-square-matrix-receiver `values.pow(power)`
result reads with fixed float-matrix metadata and no UDT/import identity.
Item 41 adds exact bound numeric-square-matrix-receiver `values.inv()` result
reads with fixed float-matrix metadata and no UDT/import identity.
Item 42 adds exact bound numeric-matrix-receiver `values.pinv()` result reads
with fixed float-matrix metadata and no UDT/import identity.
Item 43 adds exact bound numeric-square-matrix-receiver
`values.eigenvectors()` result reads with fixed float-matrix metadata and no
UDT/import identity.
Item 44 adds exact bound numeric-matrix-receiver matrix-valued
`values.mult(other)` result reads with fixed float-matrix metadata and no
UDT/import identity.
Broader UDT element families, bound or UDF
matrix-result receivers, built-in-qualified/template call-result receivers
outside the closed paths, unsupported `array.new<T>` templates, non-array/non-UDT results,
unknown/`na` results without a concrete supported type or identity, unsupported
mutation contexts, and helpers not listed there remain unsupported until later
fixture-backed slices implement syntax, analysis, runtime behavior, and
conformance updates together. The `array` lexical prefix remains reserved for
the built-in path; slice live-view/copy independence and concat's in-place,
UDF-rejected mutation keep their existing semantics.
