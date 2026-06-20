mod core;
mod strategy_accounting;
mod strategy_margin;
mod strategy_orders;
mod strategy_pyramiding;
mod strategy_reservations;

use core::CORE_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_accounting::STRATEGY_ACCOUNTING_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_margin::STRATEGY_MARGIN_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_orders::STRATEGY_ORDER_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_pyramiding::STRATEGY_PYRAMIDING_RUNTIME_SNAPSHOT_FIXTURES;
use strategy_reservations::STRATEGY_RESERVATION_RUNTIME_SNAPSHOT_FIXTURES;

type RuntimeSnapshotFixture = (&'static str, &'static str);
type RuntimeSnapshotFixtureGroup = &'static [RuntimeSnapshotFixture];

const RUNTIME_SNAPSHOT_FIXTURE_GROUPS: &[RuntimeSnapshotFixtureGroup] = &[
    CORE_RUNTIME_SNAPSHOT_FIXTURES,
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
];
