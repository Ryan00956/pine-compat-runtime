# Pure Internal UDT Varip Design

Status: closed design gate; typed and direct-constructor scalar-field subset
implemented for same-local UDTs and scalar-field imported UDTs.

This gate defines how user-defined type values may enter the `varip` subset.
The current positive subset is limited to explicitly typed same-local or
same-imported scalar-field UDT values initialized from `na`, same-UDT
constructors, fixture-backed same-UDT ternary expressions, fixture-backed
same-UDT switch expressions, fixture-backed same-UDT if expressions, or
fixture-backed same-UDT for expressions, plus direct-constructor-inferred
same-local or same-imported scalar-field UDT values.

## Current Boundary

The executable `varip` subset supports scalar `int`, `float`, `bool`,
`string`, `color`, and `na` values, plus scalar typed-array ids and their
backing stores. Realtime forming updates clone the confirmed runtime, seed only
`varip` slots from the previous forming runtime, and then execute the current
update. Scalar typed-array `varip` slots also seed the referenced array backing
store.

Non-constructor-inferred UDT `varip` values remain outside that subset. The
current negative boundary is covered by
`tests/fixtures/sema/unsupported_user_type_varip.pine`, which rejects:

```pine
type Point
    float x

varip p = bar_index == 0 ? Point.new(close) : Point.new(open)
```

Same-local scalar-field UDT array `varip` declarations are fixture-backed by
`tests/fixtures/runtime/user_type_array_varip.pine`,
`tests/fixtures/realtime/user_type_array_varip.pine`, and
`tests/fixtures/sema/supported_user_type_array_varip_decl.pine`. Non-scalar UDT
array `varip` declarations are rejected by
`tests/fixtures/sema/unsupported_user_type_array_varip_decl.pine`, and
nested-field UDT `varip` values are rejected by
`tests/fixtures/sema/unsupported_user_type_varip_nested_field.pine`.
Mismatched UDT assignment into an already typed UDT `varip` slot is rejected by
`tests/fixtures/sema/unsupported_user_type_varip_assign_identity.pine`.
Deferred-field imported UDT `varip` construction remains rejected by
`tests/fixtures/sema/unsupported_imported_udt_varip.pine`, while
local/imported lookalike assignment into an imported `varip` slot is rejected by
`tests/fixtures/sema/unsupported_imported_udt_varip_identity.pine`.

Ordinary `var` UDT values already have realtime rollback coverage:

- `tests/fixtures/realtime/user_type_var_rollback.pine` covers inferred UDT
  `var` values;
- `tests/fixtures/realtime/user_type_typed_var_rollback.pine` covers typed
  local UDT `var` declarations initialized from `na`, constructors, and
  same-UDT expressions.

Those fixtures are the rollback baseline that a positive UDT `varip` slice must
contrast against.

## Target Shape

The first positive UDT `varip` subset should be smaller than general UDT value
support:

- same-source scalar-field UDT values only, local or imported;
- local/global declaration sites that already pass ordinary typed UDT
  declaration rules;
- typed declarations initialized from `na`, a same-UDT constructor, a
  fixture-backed same-UDT ternary expression, a fixture-backed same-UDT
  switch expression, a fixture-backed same-UDT if expression, or a
  fixture-backed same-UDT for expression;
- later assignment from the same UDT identity;
- scalar field reads and, for local UDTs only, field mutations on the root local
  symbol; imported UDTs use same-identity reassignment because imported field
  mutation remains unsupported;
- imported identities are limited to scalar-field UDT values with source-scoped
  identity metadata;
- no nested UDT fields;
- no non-scalar UDT arrays;
- no UDT history references;
- no drawing ids, chart points, tuples, maps, matrices, or arrays as UDT fields;
- no field mutation inside UDF or method bodies beyond the subset already
  accepted for ordinary local UDT mutation.

Historical bars should execute UDT `varip` like `var`: initialize the slot once
and reuse it on later historical bars. Realtime forming updates should carry the
current intrabar UDT value forward from the previous forming update after the
declaration has run at least once. The confirmed update for that bar should
seed from the last forming value, execute once, and commit the resulting UDT
value into the confirmed runtime.

## Semantic Policy

Allow only declarations where both identity and runtime storage are explicit:

```pine
type Point
    float x

varip Point p = na
if na(p)
    p := Point.new(close)
else
    p.x := p.x + 1
plot(p.x)
```

An untyped constructor initializer such as `varip p = Point.new(close)` or
`varip p = lib.Point.new(close)` is accepted when the semantic analyzer binds it
to the same concrete local or imported scalar-field UDT identity used by typed
declarations.

Explicitly typed same-UDT ternary, switch, if, and for initializers, such as
`varip Point p = condition ? Point.new(close) : Point.new(open)` or a
selector-form `switch`, an `if` expression whose branches return the same local UDT, or a `for`
expression whose body returns the same local UDT, use the same
identity and value-clone handoff as other typed UDT `varip` declarations.
Untyped non-constructor inference remains rejected.

Reject:

- nested UDT fields or non-scalar fields;
- direct UDT arrays or UDT arrays wrapped by another value;
- UDT history references such as `p[1]`;
- mismatched constructor or assignment identities;
- method or UDF side effects that are still rejected for ordinary UDT values.

Diagnostic wording should continue to report feature `varip` while naming the
unsupported value family precisely enough to distinguish UDT values from UDT
arrays. The UDT value diagnostic should name the supported explicit or
direct-constructor same-local or same-imported scalar-field subset, and keep
untyped non-constructor inference, nested or non-scalar UDT values, and UDT
arrays outside that subset.

## Runtime Policy

The existing realtime handoff already clones `PineValue` slots when seeding
`PersistenceKind::Varip` from the previous forming runtime. A UDT value is a
plain runtime value, so the first positive scalar-field subset should use
value-clone semantics:

- seed the whole UDT field vector by value;
- treat field mutation as mutation of the carried slot value;
- avoid introducing object identity for scalar-field UDT values;
- keep array backing-store seeding separate from UDT value seeding;
- preserve existing ordinary `var` rollback by not widening non-`varip`
  persistence behavior.

If later UDT value support grows references, imported identity side tables, or
nested field stores, this gate should be reopened before enabling UDT `varip`
for those shapes.

## Required Fixtures

A positive implementation slice should add at least:

- runtime historical fixture for typed scalar-field UDT `varip` initialized from
  `na` and later same-UDT assignment;
- realtime fixture showing local UDT `varip` field mutation or imported UDT
  same-identity reassignment persists across repeated forming updates;
- paired realtime fixture showing ordinary UDT `var` still rolls back on
  repeated forming updates;
- semantic fixture rejecting mismatched UDT assignment into a `varip` UDT slot;
- semantic fixture keeping non-scalar UDT array `varip` rejected;
- semantic fixture keeping deferred-field imported UDT `varip` rejected.

Snapshot expectations must show the intrabar difference directly: ordinary UDT
`var` output resets from confirmed state on each forming update, while UDT
`varip` output continues from the previous forming update.

## Suggested Slice Order

1. Tighten negative diagnostics for UDT value versus UDT array `varip` if the
   current message is too broad. Done.
2. Add typed scalar-field UDT `varip` semantic acceptance for local same-source
   UDT identities only. Done.
3. Add runtime and realtime fixtures proving field-vector clone handoff and
   confirmed-bar commit. Done for the typed scalar-field subset.
4. Add same-local scalar-field UDT array `varip` handoff by reusing the
   runtime-owned array backing-store seeding path and preserving UDT element
   identity metadata. Done.
4. Add untyped constructor inference only after typed declarations are stable.
   Done for direct same-local scalar-field constructors only.
5. Add explicitly typed same-UDT ternary initializer fixtures under the existing
   scalar-field value-clone handoff. Done.
6. Add explicitly typed same-UDT switch initializer fixtures under the same
   scalar-field value-clone handoff. Done.
7. Add explicitly typed same-UDT if-expression initializer fixtures under the
   same scalar-field value-clone handoff. Done.
8. Add explicitly typed same-UDT for-expression initializer fixtures under the
   same scalar-field value-clone handoff. Done.
9. Add scalar-field imported UDT `varip` declarations under the same value-clone
   handoff. Done for `na`, same-imported constructors, and direct constructor
   inference.
10. Consider UDT arrays only after UDT array history and imported UDT identity
    interactions are independently designed.

Completion for the first positive subset requires focused semantic/runtime
tests, CLI fixture snapshots, `git diff --check`, and `scripts/verify.sh`.
