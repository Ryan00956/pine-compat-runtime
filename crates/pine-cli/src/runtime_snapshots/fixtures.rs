mod arrays;
mod control_flow;
mod core;
mod matrix;
mod strategy_accounting;
mod strategy_margin;
mod strategy_orders;
mod strategy_pyramiding;
mod strategy_reservations;

use arrays::ARRAY_RUNTIME_SNAPSHOT_FIXTURES;
use control_flow::CONTROL_FLOW_RUNTIME_SNAPSHOT_FIXTURES;
use core::{CORE_POST_MATRIX_RUNTIME_SNAPSHOT_FIXTURES, CORE_RUNTIME_SNAPSHOT_FIXTURES};
use matrix::MATRIX_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_accounting::STRATEGY_ACCOUNTING_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_margin::STRATEGY_MARGIN_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_orders::STRATEGY_ORDER_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_pyramiding::STRATEGY_PYRAMIDING_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_reservations::STRATEGY_RESERVATION_RUNTIME_SNAPSHOT_FIXTURES;

type RuntimeSnapshotFixture = (&'static str, &'static str);
type RuntimeSnapshotFixtureGroup = &'static [RuntimeSnapshotFixture];

const RUNTIME_SNAPSHOT_FIXTURE_GROUPS: &[RuntimeSnapshotFixtureGroup] = &[
    CORE_RUNTIME_SNAPSHOT_FIXTURES,
    ARRAY_RUNTIME_SNAPSHOT_FIXTURES,
    MATRIX_RUNTIME_SNAPSHOT_FIXTURES,
    CORE_POST_MATRIX_RUNTIME_SNAPSHOT_FIXTURES,
    CONTROL_FLOW_RUNTIME_SNAPSHOT_FIXTURES,
    STRATEGY_ORDER_RUNTIME_SNAPSHOT_FIXTURES,
    STRATEGY_RESERVATION_RUNTIME_SNAPSHOT_FIXTURES,
    STRATEGY_ACCOUNTING_RUNTIME_SNAPSHOT_FIXTURES,
    STRATEGY_PYRAMIDING_RUNTIME_SNAPSHOT_FIXTURES,
    STRATEGY_MARGIN_RUNTIME_SNAPSHOT_FIXTURES,
];

pub(crate) fn runtime_snapshot_fixtures() -> impl Iterator<Item = RuntimeSnapshotFixture> {
    RUNTIME_SNAPSHOT_FIXTURE_GROUPS
        .iter()
        .flat_map(|fixtures| fixtures.iter().copied())
}

pub(crate) type RuntimeLibrarySnapshotFixture = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

pub(crate) const RUNTIME_LIBRARY_SNAPSHOT_FIXTURES: &[RuntimeLibrarySnapshotFixture] = &[
    (
        "runtime_import.json",
        "tests/fixtures/runtime/import.pine",
        &[("user/lib/1", "tests/fixtures/libraries/import_lib.pine")],
    ),
    (
        "runtime_import_state.json",
        "tests/fixtures/runtime/import_state.pine",
        &[("user/lib/1", "tests/fixtures/libraries/import_lib.pine")],
    ),
    (
        "runtime_import_udt_constructor.json",
        "tests/fixtures/runtime/import_udt_constructor.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_reassignment.json",
        "tests/fixtures/runtime/import_udt_reassignment.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_ternary.json",
        "tests/fixtures/runtime/import_udt_ternary.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_if_expression.json",
        "tests/fixtures/runtime/import_udt_if_expression.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_switch_statement_block.json",
        "tests/fixtures/runtime/import_udt_switch_statement_block.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_while_expression.json",
        "tests/fixtures/runtime/import_udt_while_expression.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_for_expression.json",
        "tests/fixtures/runtime/import_udt_for_expression.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_typed_declaration.json",
        "tests/fixtures/runtime/import_udt_typed_declaration.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_var.json",
        "tests/fixtures/runtime/import_udt_var.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_varip.json",
        "tests/fixtures/runtime/import_udt_varip.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_field_mutation.json",
        "tests/fixtures/runtime/import_udt_field_mutation.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_field_mutation_control_flow.json",
        "tests/fixtures/runtime/import_udt_field_mutation_control_flow.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_udf_passthrough.json",
        "tests/fixtures/runtime/import_udt_udf_passthrough.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_udf_nested_passthrough.json",
        "tests/fixtures/runtime/import_udt_udf_nested_passthrough.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_udf_constructor_return.json",
        "tests/fixtures/runtime/import_udt_udf_constructor_return.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_udf_local_field_mutation.json",
        "tests/fixtures/runtime/import_udt_udf_local_field_mutation.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
    (
        "runtime_import_udt_udf_nested_constructor_return.json",
        "tests/fixtures/runtime/import_udt_udf_nested_constructor_return.pine",
        &[("user/udt/1", "tests/fixtures/libraries/import_udt_lib.pine")],
    ),
];
