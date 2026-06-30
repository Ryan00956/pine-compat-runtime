# Pure Internal Map Design Gate

Status: closed as a documentation-only design gate. This slice does not change
syntax acceptance, semantic analysis, runtime behavior, conformance status,
snapshots, or public output.

This document defines the first internal path for future `map.*` support. It is
scoped to interpreter internals only: parser, semantic analysis, runtime storage,
history, rollback, and conformance. It does not cover remote data, host UI,
rendering, or public JSON/Python/WASM map serialization.

## Current Boundary

`map.*` is intentionally unsupported today.

Current evidence:

- `tests/fixtures/conformance.tsv` records `map.*` as `unsupported`.
- `tests/fixtures/sema/unsupported_map_new_template.pine` calls
  `map.new<string, float>()`, and
  `tests/fixtures/sema/unsupported_map_new_dotted_template.pine` calls
  `map.new<chart.point, chart.point>()`. `tests/fixtures/sema/unsupported_map.pine`
  calls `map.put(...)`,
  `tests/fixtures/sema/unsupported_map_get.pine` calls `map.get(...)`, and
  `tests/fixtures/sema/unsupported_map_contains.pine` calls
  `map.contains(...)`. `tests/fixtures/sema/unsupported_map_size.pine` calls
  `map.size(...)`, and `tests/fixtures/sema/unsupported_map_remove.pine` calls
  `map.remove(...)`. `tests/fixtures/sema/unsupported_map_clear.pine` calls
  `map.clear(...)`, and `tests/fixtures/sema/unsupported_map_copy.pine` calls
  `map.copy(...)`. `tests/fixtures/sema/unsupported_map_keys.pine` calls
  `map.keys(...)`, and `tests/fixtures/sema/unsupported_map_values.pine` calls
  `map.values(...)`. `tests/fixtures/sema/unsupported_map_put_all.pine` calls
  `map.put_all(...)`.
- `crates/pine-sema/src/analyzer/unsupported.rs` reports
  `map collections are not implemented; map.* requires a dedicated key/value
  storage model`.
- `crates/pine-sema/tests/fixtures.rs` asserts the unsupported diagnostic.

Do not widen `map.*` until a runtime slice implements the behavior and updates
fixtures, conformance, snapshots, and docs together.

## Target Shape

The first positive map subset should mirror the existing array discipline:

- maps are runtime-owned ids, not host-visible structures;
- assignment passes map ids by reference;
- `map.copy()` is the explicit independent-copy boundary;
- non-`var` declarations allocate when executed;
- `var` declarations preserve the map id and backing storage across bars;
- rollback restores map backing storage to the confirmed snapshot;
- map growth is bounded by a runtime limit and visible in runtime profiles.

The first runtime value should add a new internal id family such as
`PineValue::Map(u32)` with a dedicated runtime store. Do not reuse array ids or
array element storage for map entries. Maps need key lookup, replacement,
deletion, and iteration semantics that are different from array slot indexing.

## Key And Value Policy

First positive key families:

- `int`
- `float`
- `bool`
- `string`
- `color`

First positive value families:

- `int`
- `float`
- `bool`
- `string`
- `color`
- `na`

Deferred key and value families:

- arrays;
- maps;
- matrices;
- user-defined types;
- tuples;
- object ids;
- chart points;
- strategy/order/trade records.

Rationale:

- scalar keys avoid object identity and lifetime problems in the first slice;
- scalar values avoid nested collection rollback and aliasing problems;
- UDT and collection values should wait for explicit identity and history rules.

## Equality And Ordering

Map key equality must be deterministic and independent of host formatting.

Initial policy:

- `int`, `bool`, `string`, and `color` keys compare by exact value;
- `float` keys compare by exact finite bit/value semantics chosen by the runtime
  design slice;
- `na` keys are rejected in semantic analysis or at runtime before insertion;
- non-finite float keys are rejected.

Iteration order should not be a hidden implementation detail. The first subset
should choose insertion order for `map.keys()` and `map.values()` if those
helpers are implemented. Replacement of an existing key should update the value
without moving the key in order. Deleting and reinserting a key should append it
at the new insertion point.

## Type Model

The semantic model should represent map types explicitly, not as generic arrays.

Future type candidates:

- `map<int, float>`
- `map<string, float>`
- `map<string, string>`
- `map<simple-key, scalar-value>` only if the analyzer can keep diagnostics clear.

The first slice should avoid broad generic inference. A good first constructor is
an explicit typed constructor such as `map.new<string, float>()`, followed by
`map.put`, `map.get`, and `map.contains` for that exact key/value pair.

Bare `map` declarations and mixed value maps should stay unsupported until type
identity, `na` value behavior, and assignment compatibility are designed.

## Runtime Operations

Candidate first operation set:

- `map.new<K, V>()`
- `map.put(id, key, value)`
- `map.get(id, key)`
- `map.contains(id, key)`
- `map.remove(id, key)`
- `map.clear(id)`
- `map.size(id)`
- `map.copy(id)`

Candidate later operation set:

- `map.keys(id)`
- `map.values(id)`
- `map.put_all(target, source)`

Keep method-call aliases out of the first positive slice unless the namespace
calls are already fixture-backed. Method calls should lower to the same built-in
operations only after receiver typing is stable.

## History And Realtime

First history policy:

- no map history references in the first positive runtime slice;
- no `varip` map values in the first positive runtime slice;
- map state must still roll back correctly for ordinary realtime forming updates.

Later history policy:

- `previous = m[1]` should return a fresh copy of the committed map snapshot, not
  an alias into past storage;
- nested maps or collection values require a deeper copy policy before support;
- dynamic history over map ids should use the same retention guardrails as other
  supported history values.

## Function And Method Boundaries

Initial policy:

- passing map ids to user-defined functions is allowed only after reference and
  mutation semantics are designed;
- map mutation inside user-defined functions stays unsupported in the first
  positive slice, matching the current conservative array mutation boundary;
- method calls on maps stay unsupported until namespace-call behavior is stable.

## Diagnostics

Before positive support lands, keep the current unsupported diagnostic:

```text
map collections are not implemented; map.* requires a dedicated key/value storage model
```

When support starts, unsupported variants should fail with precise diagnostics:

- unsupported key type;
- unsupported value type;
- `na` key;
- non-finite float key;
- unknown map method;
- map mutation inside an unsupported side-effect context;
- map history or `varip` use outside the supported subset.

## Slice Order

Recommended future slices:

1. Semantic design lock: add type names, signatures, and negative fixtures while
   keeping `map.*` unsupported.
2. Runtime store skeleton: add an internal map store and profile counters with no
   accepted Pine syntax.
3. First positive scalar subset:
   `map.new<string, float>`, `map.put`, `map.get`, `map.contains`, and
   `map.size`.
4. Mutation utilities: `map.remove`, `map.clear`, and replacement-order fixtures.
5. Copy semantics: `map.copy` and assignment/reference fixtures.
6. Realtime rollback fixtures for map mutation.
7. Optional method-call aliases after namespace calls are stable.
8. Optional `map.keys` / `map.values` if array return typing and insertion order
   are fully fixture-backed.
9. Map history snapshots only after copy/deep-copy policy is explicit.

## Completion Gate For Future Positive Support

Any positive map support must include:

- semantic fixtures for accepted and rejected key/value forms;
- runtime fixtures and golden snapshots;
- realtime rollback tests when mutation is supported;
- incremental-vs-historical parity tests when history or state timing matters;
- profile or guardrail tests for map storage growth;
- synchronized `tests/fixtures/conformance.tsv`, `docs/CONFORMANCE.md`, matrix
  snapshot, release notes, and this design document;
- `git diff --check`;
- `scripts/verify.sh`.

## Closed Slice Result

This design gate closes only the planning prerequisite. The supported runtime
subset is unchanged. `map.*` remains unsupported until a later slice implements
fixture-backed syntax, analysis, runtime behavior, and conformance updates.
