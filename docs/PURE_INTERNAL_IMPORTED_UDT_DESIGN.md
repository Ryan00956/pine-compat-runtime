# Pure Internal Imported UDT Identity Design Gate

Status: active implementation plan. The design gate is closed, and the
runtime-backed scalar-field imported constructor, direct field-read, ordinary
same-imported-UDT reassignment, explicit typed declaration, selected
same-imported-identity control-expression results, and direct or nested UDF
passthrough/constructor-return subset, ordinary `var` persistence,
scalar-field same-imported-identity `varip` persistence, and scalar-field
mutation in top-level, branch, `for`-loop, `while`-loop, and UDF-local
statement contexts are implemented. Broader imported UDT flow remains gated
below.

This document defines the internal path for future imported user-defined type
identity across source graphs. It is scoped to parser-visible qualified names,
semantic type identity, module export tables, HIR lowering, runtime UDT values,
method dispatch, fixtures, and conformance. It does not cover remote library
lookup, registry resolution, filesystem access inside core crates, host UI, or
public serialization changes.

## Current Boundary

The current import subset supports host-provided exact-key imports with aliases,
exported const expressions, pure exported functions, scalar-field imported UDT
constructors with direct field reads, and ordinary same-imported-UDT
reassignment, plus scalar-field imported UDT typed declarations initialized or
reassigned from the same imported identity, same-imported-identity ternary,
`if`, `switch`, `while`, and `for` expression results, direct or nested imported UDT UDF
parameter passthrough returns, direct or nested constructor-return UDFs, and
ordinary imported UDT `var` declarations, plus scalar-field imported UDT
`varip` declarations initialized from `na`, same-imported constructors, or
direct constructor inference, plus scalar-field mutation in top-level, branch,
`for`-loop, `while`-loop, and UDF-local statement contexts outside method
bodies and UDF parameter/global side-effect boundaries.
Imported UDT collections, history, nested field mutation, and imported methods
remain unsupported.

Current evidence:

- `docs/PURE_INTERNAL_ROADMAP.md` lists imported UDT identity across source
  graphs and imported methods as remaining structured-data work.
- `tests/fixtures/conformance.tsv` marks `import` partial and records the
  scalar-field imported UDT constructor/direct field-read/reassignment/typed
  declaration/direct-or-nested UDF passthrough subset, while broader imported
  UDT flow and imported methods remain unsupported.
- `docs/CONFORMANCE.md`, `docs/EXECUTION_SEMANTICS.md`, and
  `docs/SEMANTIC_MODEL.md` describe the narrow executable imported UDT
  constructor/direct field-read/reassignment/typed declaration/direct UDF
  passthrough plus nested passthrough-chain subset and keep imported method
  tables plus broader imported UDT flow outside the executable subset.
- `crates/pine-sema/src/source_graph.rs` assigns deterministic root/library
  `SourceId`s from host-provided source text and normalized exact import keys.
- `crates/pine-sema/src/modules.rs` currently collects exported constants,
  exported functions, and exported UDT declarations. Exported UDTs now carry
  module-local source-scoped identity metadata (`SourceId`, type name) plus
  parser-level field layout metadata at the export table boundary. The import
  plan now records alias-qualified imported UDT metadata such as `lib.Point`,
  including scalar `PineType` metadata for `int`, `float`, `bool`, `string`, and
  `color` fields, and passes it into the analyzer, while constants/functions are
  still the only imported declarations rewritten into the root analysis plan.
  Scalar-field exported UDT constructors may now pass module validation for the
  first positive runtime subset. Deferred-field exported UDT constructors remain
  rejected with `E_IMPORT_UNSUPPORTED_UDT`, and private UDTs remain rejected as
  non-exported symbols.
- `crates/pine-sema/src/analyzer/user_types.rs` now records root-local
  `UserTypeInfo` identity metadata as `(SourceId::root(), type_name)` while
  semantic symbol/expression mark paths mirror same-root UDT identity metadata
  beside their existing type-name strings. Lowering-created declaration,
  parameter, and receiver symbols now use the same identity mirror helper. HIR
  constructors carry `HirUserTypeIdentity { source_id, type_name }` metadata.
  The analyzer can now accept alias-qualified scalar-field imported
  constructors such as `lib.Point.new`, validate local-style positional/named
  field arguments, mark imported source-scoped identity, resolve direct scalar
  field reads such as `p.x`, and allow ordinary reassignment from the same
  imported identity. It can also accept explicit scalar imported typed
  declarations such as `lib.Point p = lib.Point.new(close)` and reject
  local/imported typed declaration identity mismatches. Same-imported-identity
  ternary, `if`, `switch`, `while`, and `for` expression results are accepted, while
  local/imported branch identity mismatches are rejected. Ordinary imported UDT
  `var` declarations use the existing persistent value slot path, while
  local/imported `var` identity mismatches remain rejected. Scalar-field
  imported UDT `varip` declarations use the existing value-clone intrabar
  slot path; local/imported `varip` identity mismatches remain rejected.
  Deferred-field imported constructors remain rejected. The existing pure-UDF passthrough
  identity path now also accepts direct imported UDT parameter returns such as
  `passthrough(p) => p` when the call argument and target use the same imported
  identity. Runtime UDT values still execute as field-vector values; source
  identity is carried in semantic and HIR metadata for compatibility checks
  rather than in `PineValue`.
- Library method collection now records the declared receiver type name and,
  when it resolves to a library UDT, the receiver's source-scoped identity. This
  metadata is used only to improve imported method diagnostics; receiver-style
  calls on imported UDT values and alias-qualified imported method access both
  remain rejected.
- `tests/fixtures/libraries/import_udt_lib.pine` provides a library with an
  exported scalar UDT for `tests/fixtures/runtime/import_udt_constructor.pine`,
  plus a deferred-field exported UDT and method for rejected boundaries, while
  `tests/fixtures/libraries/import_private_udt_lib.pine` covers the private UDT
  boundary and `tests/fixtures/libraries/import_duplicate_udt_lib.pine` covers
  duplicate exported UDT names, while
  `tests/fixtures/libraries/import_duplicate_udt_const_lib.pine` covers UDT and
  const exports sharing the same name and
  `tests/fixtures/libraries/import_duplicate_udt_function_lib.pine` covers UDT
  and function exports sharing the same name.
- `tests/fixtures/runtime/import_udt_constructor.pine` keeps
  `lib.Point.new(...)` plus `p.x` executable, and
  `tests/fixtures/runtime/import_udt_reassignment.pine` keeps ordinary
  same-imported-UDT reassignment executable, and
  `tests/fixtures/runtime/import_udt_typed_declaration.pine` keeps
  `lib.Point` typed declaration plus same-imported-UDT reassignment executable,
  and `tests/fixtures/runtime/import_udt_var.pine` keeps inferred and explicit
  `lib.Point` `var` declarations plus same-imported-UDT reassignment
  executable,
  and `tests/fixtures/runtime/import_udt_varip.pine` plus
  `tests/fixtures/realtime/import_udt_varip.pine` keep scalar-field imported
  UDT `varip` declarations executable through historical and realtime
  intrabar persistence,
  and `tests/fixtures/runtime/import_udt_field_mutation.pine` keeps scalar-field
  imported UDT mutation executable at top level,
  and `tests/fixtures/runtime/import_udt_field_mutation_control_flow.pine`
  keeps scalar-field imported UDT mutation executable in branch, `for`-loop,
  and `while`-loop statement contexts,
  and `tests/fixtures/runtime/import_udt_udf_local_field_mutation.pine` keeps
  scalar-field imported UDT mutation executable for UDF-local variables returned
  from pure functions,
  while
  `tests/fixtures/sema/unsupported_imported_udt_parameter_field_mutation.pine`
  keeps imported UDT parameter field mutation inside pure functions rejected as
  a side-effect boundary,
  and
  `tests/fixtures/sema/unsupported_imported_udt_global_field_mutation.pine`
  keeps imported UDT global field mutation inside pure functions rejected as a
  side-effect boundary,
  and `tests/fixtures/sema/unsupported_imported_udt_history.pine` keeps imported
  UDT value history references rejected,
  and
  `tests/fixtures/sema/unsupported_imported_udt_nested_field_mutation.pine`
  keeps parser-level nested imported field mutation rejected,
  and `tests/fixtures/sema/unsupported_imported_udt_array_decl.pine` plus
  `tests/fixtures/sema/unsupported_imported_udt_array_alias_decl.pine` keep
  `array<lib.Point>` and `lib.Point[]` imported UDT array declarations rejected,
  and `tests/fixtures/sema/unsupported_imported_udt_array_from.pine` keeps
  `array.from` inference from imported UDT values rejected,
  and `tests/fixtures/syntax/unsupported_imported_udt_array_new.pine` keeps
  imported UDT `array.new<lib.Point>()` templates rejected at the parser
  boundary,
  and `tests/fixtures/runtime/import_udt_udf_passthrough.pine` keeps direct
  imported UDT UDF parameter passthrough executable,
  and `tests/fixtures/runtime/import_udt_udf_nested_passthrough.pine` keeps
  nested imported UDT UDF parameter passthrough chains executable,
  while
  `tests/fixtures/sema/unsupported_imported_udt_constructor.pine` keeps a
  deferred-field imported constructor rejected, and
  `tests/fixtures/sema/unsupported_imported_udt_varip.pine` keeps deferred-field
  imported UDT construction rejected in a `varip` initializer, while
  `tests/fixtures/sema/unsupported_imported_udt_assignment_identity.pine` keeps
  local/imported structural lookalikes rejected as different identities, while
  `tests/fixtures/sema/unsupported_imported_udt_typed_decl_identity.pine` keeps
  typed declarations from accepting local/imported structural lookalikes, while
  `tests/fixtures/sema/unsupported_imported_udt_var_identity.pine` keeps `var`
  declarations from accepting local/imported structural lookalikes, while
  `tests/fixtures/sema/unsupported_imported_udt_varip_identity.pine` keeps
  `varip` declarations from accepting local/imported structural lookalikes,
  while
  `tests/fixtures/sema/unsupported_imported_udt_udf_passthrough_identity.pine`
  keeps direct UDF passthrough from erasing local/imported identity mismatches,
  while
  `tests/fixtures/sema/unsupported_imported_udt_udf_nested_passthrough_identity.pine`
  keeps nested passthrough chains from erasing local/imported identity
  mismatches, while
  `tests/fixtures/sema/unsupported_imported_private_udt_constructor.pine` keeps
  private library UDT construction rejected as private symbol access,
  `tests/fixtures/sema/unsupported_import_duplicate_exported_udt.pine` keeps
  duplicate exported UDT names rejected,
  `tests/fixtures/sema/unsupported_import_duplicate_exported_udt_const.pine`
  keeps duplicate exported UDT/const names rejected,
  `tests/fixtures/sema/unsupported_import_duplicate_exported_udt_function.pine`
  keeps duplicate exported UDT/function names rejected,
  `tests/fixtures/sema/unsupported_imported_method.pine` keeps receiver-style
  imported UDT method lookup rejected with targeted imported-method diagnostics and
  `tests/fixtures/sema/unsupported_imported_method_qualified.pine` keeps
  alias-qualified imported method access rejected.
- `crates/pine-sema/src/tests/compatibility.rs` also asserts scalar-field
  imported UDT constructors analyze successfully, deferred-field imported UDT
  constructors fail with `E_IMPORT_UNSUPPORTED_UDT`, private imported UDT
  constructors fail with `E_IMPORT_PRIVATE_SYMBOL`, local/imported assignment
  identity mismatches fail with `E_UDT_ASSIGN_TYPE`, scalar imported UDT typed
  declarations analyze successfully, typed declaration identity mismatches fail
  with `E_UDT_ASSIGN_TYPE`, direct imported UDT UDF passthrough analyzes
  successfully, nested imported UDT UDF passthrough analyzes successfully,
  passthrough identity mismatches fail with `E_UDT_ASSIGN_TYPE`, duplicate
  exported UDT names plus UDT/const and UDT/function export name collisions
  fail with `E_IMPORT_DUPLICATE_EXPORT`,
  receiver-style and alias-qualified imported method access fail with
  `E_IMPORT_UNSUPPORTED_METHOD`.

Do not widen imported UDTs beyond the scalar constructor/direct field-read,
ordinary reassignment, explicit typed declaration, same-imported-identity
ternary/`if`/`switch`/`while`/`for` expression results, direct or nested UDF
passthrough, direct or nested constructor-return subset, ordinary `var`,
scalar-field `varip`, and scalar-field mutation in top-level, branch,
`for`-loop, `while`-loop, and UDF-local statement contexts until a runtime slice
implements the behavior and updates fixtures, conformance, snapshots, and docs
together.

## Target Shape

Imported UDTs should be source-graph-scoped type identities, not root-local names
that happen to share spelling.

Target identity properties:

- every UDT definition has a stable semantic identity such as
  `(SourceId, type_name)`;
- root-local `Point` and imported `lib.Point` are different identities even if
  their field lists are structurally identical;
- the same imported type reached through the same resolved source graph identity
  has the same semantic identity wherever it is referenced;
- imported UDT values can be assigned, passed, returned, and field-read only when
  identity compatibility is proven;
- diagnostics should display the user-facing name (`Point`, `lib.Point`, or a
  later canonical form) while comparing internal identities.

The first positive subset should support values, not host-visible object ids.
Runtime values can continue to use the current `PineValue::UserType` field-vector
representation if semantic analysis and lowering carry enough type identity and
field layout metadata to preserve compatibility.

## Export And Import Policy

Initial export policy:

- exported type declarations in library sources become addressable through the
  import alias, for example `lib.Point`;
- non-exported type declarations remain private to their source unit;
- duplicate exported type names in one module are rejected;
- private symbol access through an import alias continues to fail;
- re-exporting imported types remains unsupported in the first positive subset;
- remote lookup and implicit library resolution remain host-owned and out of
  core scope.

Initial import policy:

- `import user/lib/1 as lib` remains the only supported import spelling;
- `lib.Point.new(...)` is the first constructor spelling to consider;
- bare `Point.new(...)` never resolves to an imported type without a local type
  declaration or explicit import alias qualification;
- `array<lib.Point>` and `lib.Point[]` should remain deferred until typed
  declaration parsing and UDT array semantics cover dotted UDT element names.

## Field And Constructor Policy

First positive imported UDT subset:

- scalar fields only: `int`, `float`, `bool`, `string`, and `color`;
- constructor argument rules mirror local UDT constructors, including positional
  and named field arguments;
- field reads use the imported type's declaration order and field names;
- scalar field mutation follows the local UDT mutation writeback path in
  top-level, branch, `for`-loop, `while`-loop, and UDF-local statement
  contexts;
- imported field mutation on UDF parameters or globals, inside methods, nested
  imported field mutation, and imported collection/history mutation remain
  unsupported;
- imported UDT values can be stored in ordinary variables and explicit typed
  locals only when the initializer and later reassignment carry the same
  imported identity.

Deferred field families:

- nested imported or local UDT fields;
- arrays, maps, matrices, tuples, drawing ids, chart points, strategy records,
  and other reference-like fields;
- recursive and forward-declared fields;
- imported UDT history references.

## Method Policy

Imported method support should follow imported type identity. Do not expose
library method tables as loose functions.

Initial policy:

- a method declared in a library is associated with the receiver type identity
  from that library source;
- method lookup on an imported UDT receiver searches that identity's method
  table, not root-local methods with the same receiver spelling;
- root-local methods cannot attach to imported receiver identities in the first
  subset;
- imported methods must satisfy the same pure-method and no-side-effect rules as
  local methods;
- method return identity must be tracked exactly like constructor and parameter
  identity.

Imported methods should remain rejected until imported UDT constructors, value
identity, field reads, assignment compatibility, and UDF parameter/return
identity are stable.

## Analyzer And Lowering Policy

Future implementation should avoid string-only identity comparisons:

- extend module collection to record exported UDT declarations and method
  declarations with source identities;
- introduce an internal `UserTypeId` or equivalent that includes `SourceId` and
  local type name;
- replace root-local UDT identity maps that store only names where imported
  values can flow;
- carry field layout and display names through semantic analysis and lowering;
- keep compile-cache keys tied to root source plus every host-provided library
  key/name/text, as today, so imported type identity cannot reuse stale graphs;
- emit precise diagnostics for unknown exports, private symbols, unsupported
  imported UDTs, mismatched imported/local identities, and unsupported imported
  method variants.

HIR and runtime should not need a new public JSON shape for the first value-only
subset. Any later public exposure of imported UDT values must be a separate
contract slice.

## Realtime, History, And Collections

First imported UDT support should not introduce new persistence behavior:

- ordinary, `var`, and scalar-field `varip` imported UDT values behave like
  local UDT values;
- history references, UDT arrays, and nested collection storage
  remain deferred until local equivalents and imported identity rules are both
  fixture-backed;
- realtime rollback follows existing value rollback once imported UDT values are
  represented as ordinary UDT values with stable identity metadata.

## Slice Order

Recommended future slices:

1. Export table shape: collect exported UDT declarations and keep them rejected
   with targeted diagnostics before constructor support. This negative boundary
   is fixture-backed; private library UDTs remain non-exported symbols and
   duplicate exported UDT names plus UDT/const and UDT/function name collisions
   are rejected through the shared export table. Exported UDT entries now retain
   parser-level field layout metadata for later constructor analysis, and the
   import plan carries alias-qualified imported UDT metadata with scalar
   `PineType` field classifications into the analyzer without changing accepted
   scripts.
2. Identity plumbing: introduce source-scoped UDT identity in semantic analysis
   without changing accepted scripts. The module/export boundary now records
   exported UDT identities as `(SourceId, type_name)`, and analyzer root-local
   `UserTypeInfo` records now carry `(SourceId::root(), type_name)` metadata
   with semantic symbol/expression identity mirrors plus lowering symbol mirror
   writes. HIR UDT constructors now carry
   `HirUserTypeIdentity { source_id, type_name }` metadata for both root-local
   and supported imported constructors.
3. Constructor, field-read, and ordinary reassignment subset:
   `lib.Point.new(...)` for scalar-field imported UDTs with runtime snapshots
   proving direct field-read value behavior is implemented. Ordinary
   same-imported-UDT reassignment is also fixture-backed, while local/imported
   structural lookalikes remain rejected as distinct identities. Explicit
   scalar imported typed declarations initialized or reassigned from the same
   imported identity are fixture-backed, while local/imported typed declaration
   identity mismatches remain rejected. Same-imported-identity ternary, `if`,
   `switch`, `while`, and `for` expression results are fixture-backed, while
   local/imported branch identity mismatches remain rejected. Deferred-field
   constructors, field mutation, history,
   collections, and methods remain rejected.
4. UDF passthrough: allow imported UDT values to flow through pure UDFs while
   rejecting mismatched identities. Direct parameter passthrough returns such
   as `passthrough(p) => p` are now fixture-backed for same imported identity,
   while local/imported identity mismatches remain rejected. Broader UDF flow,
   including constructor-return helpers and control expression returns, remains
   deferred. Nested passthrough chains are now fixture-backed for same imported
   identity, while nested local/imported identity mismatches remain rejected.
5. Imported methods: support pure methods whose receiver identity is imported
   and whose parameters/returns stay inside the supported identity subset.
   Library method metadata now records receiver identity for diagnostics, but no
   method table is imported into root analysis yet.
6. Collections and history only after imported value identity is stable across
   historical and realtime execution.
