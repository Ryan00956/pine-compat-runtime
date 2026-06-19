use std::{fs, path::PathBuf};

use pine_runtime::{AlertEvent, Bar, BarUpdate, PineValue, RealtimeRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn forming_close_fixture_rolls_back_repeated_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/forming_close.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back");
    assert_values(&result.plots[0].values, &[1.0, 3.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit");
    assert_values(&result.plots[0].values, &[1.0, 4.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0, 4.0]);
}

#[test]
fn alertcondition_fixture_rolls_back_forming_events() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/alertcondition_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert!(result.alerts.is_empty());

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(result.alerts[0].bar_index, 1);
    assert_eq!(result.alerts[0].message, "Close is above one");
    assert!(runtime.confirmed_result().alerts.is_empty());

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back alert event");
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(result.alerts[0].bar_index, 1);
    assert!(runtime.confirmed_result().alerts.is_empty());

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit alert event");
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(runtime.confirmed_result().alerts.len(), 1);
}

#[test]
fn alert_fixture_rolls_back_forming_events() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/alert_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(result.alerts[0].bar_index, 0);
    assert_eq!(result.alerts[0].message, "Realtime alert");

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.alerts.len(), 2);
    assert_eq!(result.alerts[1].bar_index, 1);
    assert_eq!(runtime.confirmed_result().alerts.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back alert event");
    assert_eq!(result.alerts.len(), 2);
    assert_eq!(result.alerts[1].bar_index, 1);
    assert_eq!(runtime.confirmed_result().alerts.len(), 1);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit alert event");
    assert_eq!(result.alerts.len(), 2);
    assert_eq!(runtime.confirmed_result().alerts.len(), 2);
}

#[test]
fn alert_policy_fixture_recomputes_forming_events_and_commits_confirmed_state() {
    let fixture = "tests/fixtures/realtime/alert_policy.pine";
    let mut runtime = runtime_for_fixture(fixture);

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_alerts(&result.alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("forming update should expose current forming alert events");
    assert_alerts(
        &result.alerts,
        &[
            (1, "alert", "Above one"),
            (1, "alert", "Above two"),
            (1, "Above two condition", "Condition above two"),
        ],
    );
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("second forming update should drop abandoned alert events");
    assert_alerts(&result.alerts, &[(1, "alert", "Above one")]);
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(0.0)))
        .expect("third forming update should drop all abandoned alert events");
    assert_alerts(&result.alerts, &[]);
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit alert events");
    assert_alerts(
        &result.alerts,
        &[
            (1, "alert", "Above one"),
            (1, "alert", "Above two"),
            (1, "Above two condition", "Condition above two"),
        ],
    );
    assert_eq!(runtime.confirmed_result().alerts, result.alerts);

    let hir = hir_for_fixture(fixture);
    let historical = run_historical(&hir, &[bar(1.0), bar(4.0)])
        .expect("equivalent historical execution should run");
    assert_eq!(
        alert_summaries(&result.alerts),
        alert_summaries(&historical.alerts)
    );
}

#[test]
fn alert_frequency_close_fixture_only_emits_on_confirmed_updates() {
    let fixture = "tests/fixtures/realtime/alert_frequency_close.pine";
    let mut runtime = runtime_for_fixture(fixture);

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_alerts(&result.alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("forming update should not emit close-frequency alert");
    assert_alerts(&result.alerts, &[]);
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(4.0)))
        .expect("second forming update should still not emit close-frequency alert");
    assert_alerts(&result.alerts, &[]);
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(5.0)))
        .expect("confirmed update should commit close-frequency alert");
    assert_alerts(&result.alerts, &[(1, "alert", "Close alert")]);
    assert_eq!(runtime.confirmed_result().alerts, result.alerts);

    let hir = hir_for_fixture(fixture);
    let historical = run_historical(&hir, &[bar(1.0), bar(5.0)])
        .expect("equivalent historical execution should run");
    assert_eq!(
        alert_summaries(&result.alerts),
        alert_summaries(&historical.alerts)
    );
}

#[test]
fn alert_frequency_rollback_fixture_recomputes_once_and_all_events() {
    let fixture = "tests/fixtures/realtime/alert_frequency_rollback.pine";
    let mut runtime = runtime_for_fixture(fixture);

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_alerts(&result.alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("forming update should expose current frequency-filtered alerts");
    assert_alerts(
        &result.alerts,
        &[
            (1, "alert", "Default once"),
            (1, "alert", "All calls"),
            (1, "alert", "All calls"),
        ],
    );
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::forming(bar(0.0)))
        .expect("second forming update should drop abandoned frequency alerts");
    assert_alerts(&result.alerts, &[]);
    assert_alerts(&runtime.confirmed_result().alerts, &[]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit frequency-filtered alerts");
    assert_alerts(
        &result.alerts,
        &[
            (1, "alert", "Default once"),
            (1, "alert", "All calls"),
            (1, "alert", "All calls"),
        ],
    );
    assert_eq!(runtime.confirmed_result().alerts, result.alerts);

    let hir = hir_for_fixture(fixture);
    let historical = run_historical(&hir, &[bar(1.0), bar(4.0)])
        .expect("equivalent historical execution should run");
    assert_eq!(
        alert_summaries(&result.alerts),
        alert_summaries(&historical.alerts)
    );
}

#[test]
fn var_rollback_fixture_restores_confirmed_state_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/var_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back var state");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit var state");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(5.0)))
        .expect("next forming update should start from new confirmed var state");
    assert_values(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn user_type_var_fixture_rolls_back_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/user_type_var_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back UDT var state");
    assert_values(&result.plots[0].values, &[1.0, 3.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit UDT var state");
    assert_values(&result.plots[0].values, &[1.0, 4.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0, 4.0]);
}

#[test]
fn varip_scalar_fixture_persists_intrabar_state_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/varip_scalar.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);
    assert_values(&result.plots[1].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should retain varip state");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 3.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit varip state");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 4.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(5.0)))
        .expect("next forming update should start from confirmed varip state");
    assert_values(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[1].values, &[1.0, 4.0, 5.0]);
}

#[test]
fn varip_local_fixture_persists_intrabar_state_per_declaration_site() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/varip_local.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[0.0]);
    assert_values(&result.plots[1].values, &[2.0]);
    assert_values(&result.plots[2].values, &[2.0]);
    assert_values(&result.plots[3].values, &[1.0]);
    assert_values(&result.plots[4].values, &[10.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update with skipped branch should run");
    assert_values(&result.plots[0].values, &[0.0, 0.0]);
    assert_values(&result.plots[1].values, &[2.0, 4.0]);
    assert_values(&result.plots[2].values, &[2.0, 4.0]);
    assert_values(&result.plots[3].values, &[1.0, 2.0]);
    assert_values(&result.plots[4].values, &[10.0, 20.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("first reached branch should initialize after skipped updates");
    assert_values(&result.plots[0].values, &[0.0, 1.0]);
    assert_values(&result.plots[1].values, &[2.0, 6.0]);
    assert_values(&result.plots[2].values, &[2.0, 6.0]);
    assert_values(&result.plots[3].values, &[1.0, 3.0]);
    assert_values(&result.plots[4].values, &[10.0, 30.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(4.0)))
        .expect("second reached branch should retain intrabar state");
    assert_values(&result.plots[0].values, &[0.0, 2.0]);
    assert_values(&result.plots[1].values, &[2.0, 8.0]);
    assert_values(&result.plots[2].values, &[2.0, 8.0]);
    assert_values(&result.plots[3].values, &[1.0, 4.0]);
    assert_values(&result.plots[4].values, &[10.0, 40.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(5.0)))
        .expect("confirmed update should commit local varip state");
    assert_values(&result.plots[0].values, &[0.0, 3.0]);
    assert_values(&result.plots[1].values, &[2.0, 10.0]);
    assert_values(&result.plots[2].values, &[2.0, 10.0]);
    assert_values(&result.plots[3].values, &[1.0, 5.0]);
    assert_values(&result.plots[4].values, &[10.0, 50.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(6.0)))
        .expect("next forming update should start from confirmed local varip state");
    assert_values(&result.plots[0].values, &[0.0, 3.0, 4.0]);
    assert_values(&result.plots[1].values, &[2.0, 10.0, 12.0]);
    assert_values(&result.plots[2].values, &[2.0, 10.0, 12.0]);
    assert_values(&result.plots[3].values, &[1.0, 5.0, 6.0]);
    assert_values(&result.plots[4].values, &[10.0, 50.0, 60.0]);
}

#[test]
fn conditional_ta_fixture_rolls_back_callsite_state_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/conditional_ta_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar_ohlc(1.0, 2.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar_ohlc(3.0, 4.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[2.0, 3.333333333333333]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar_ohlc(7.0, 8.0)))
        .expect("second forming update should roll back callsite state");
    assert_values(&result.plots[0].values, &[2.0, 6.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[2.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar_ohlc(4.0, 5.0)))
        .expect("confirmed update should commit callsite state");
    assert_values(&result.plots[0].values, &[2.0, 4.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[2.0, 4.0]);

    let result = runtime
        .update(BarUpdate::forming(bar_ohlc(4.0, 3.0)))
        .expect("forming update with skipped branch should run");
    assert_values(&result.plots[0].values, &[2.0, 4.0, 3.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[2.0, 4.0]);

    let result = runtime
        .update(BarUpdate::forming(bar_ohlc(7.0, 8.0)))
        .expect("forming update should ignore skipped-branch callsite state");
    assert_values(&result.plots[0].values, &[2.0, 4.0, 6.666666666666667]);
}

#[test]
fn array_rollback_fixture_restores_confirmed_store_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/array_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);
    assert_values(&result.plots[1].values, &[1.0]);
    assert_values(&result.plots[2].values, &[1.0]);
    assert_values(&result.plots[3].values, &[0.0]);
    assert_values(&result.plots[4].values, &[1.0]);
    assert_values(&result.plots[5].values, &[1.0]);
    assert_values(&result.plots[6].values, &[1.0]);
    assert_values(&result.plots[7].values, &[1.0]);
    assert_values(&result.plots[8].values, &[1.0]);
    assert_values(&result.plots[9].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 1.0]);
    assert_values(&result.plots[2].values, &[1.0, 2.0]);
    assert_values(&result.plots[3].values, &[0.0, 0.0]);
    assert_values(&result.plots[4].values, &[1.0, 2.0]);
    assert_values(&result.plots[5].values, &[1.0, 1.0]);
    assert_values(&result.plots[6].values, &[1.0, 2.0]);
    assert_values(&result.plots[7].values, &[1.0, 1.0]);
    assert_values(&result.plots[8].values, &[1.0, 2.0]);
    assert_values(&result.plots[9].values, &[1.0, 1.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back array store");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 1.0]);
    assert_values(&result.plots[2].values, &[1.0, 2.0]);
    assert_values(&result.plots[3].values, &[0.0, 0.0]);
    assert_values(&result.plots[4].values, &[1.0, 2.0]);
    assert_values(&result.plots[5].values, &[1.0, 1.0]);
    assert_values(&result.plots[6].values, &[1.0, 2.0]);
    assert_values(&result.plots[7].values, &[1.0, 1.0]);
    assert_values(&result.plots[8].values, &[1.0, 2.0]);
    assert_values(&result.plots[9].values, &[1.0, 1.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit array mutation");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[2].values, &[1.0, 2.0]);
    assert_values(&result.plots[4].values, &[1.0, 2.0]);
    assert_values(&result.plots[6].values, &[1.0, 2.0]);
    assert_values(&result.plots[8].values, &[1.0, 2.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(5.0)))
        .expect("next forming update should start from confirmed array store");
    assert_values(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[2].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[4].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[6].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[8].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn varip_array_fixture_persists_intrabar_backing_store_between_forming_updates() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/varip_array.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);
    assert_values(&result.plots[1].values, &[1.0]);
    assert_values(&result.plots[2].values, &[2.0]);
    assert_values(&result.plots[3].values, &[1.0]);
    assert_values(&result.plots[4].values, &[2.0]);
    assert_values(&result.plots[5].values, &[0.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 2.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0]);
    assert_values(&result.plots[2].values, &[2.0, 3.0]);
    assert_values(&result.plots[3].values, &[1.0, 2.0]);
    assert_values(&result.plots[4].values, &[2.0, 3.0]);
    assert_values(&result.plots[5].values, &[0.0, 0.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should retain varip array state");
    assert_values(&result.plots[0].values, &[1.0, 3.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0]);
    assert_values(&result.plots[2].values, &[2.0, 4.0]);
    assert_values(&result.plots[3].values, &[1.0, 3.0]);
    assert_values(&result.plots[4].values, &[2.0, 4.0]);
    assert_values(&result.plots[5].values, &[0.0, 1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(4.0)))
        .expect("third forming update should keep retaining varip array state");
    assert_values(&result.plots[0].values, &[1.0, 4.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0]);
    assert_values(&result.plots[2].values, &[2.0, 5.0]);
    assert_values(&result.plots[3].values, &[1.0, 4.0]);
    assert_values(&result.plots[4].values, &[2.0, 5.0]);
    assert_values(&result.plots[5].values, &[0.0, 2.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(5.0)))
        .expect("confirmed update should commit varip array state");
    assert_values(&result.plots[0].values, &[1.0, 5.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0]);
    assert_values(&result.plots[2].values, &[2.0, 6.0]);
    assert_values(&result.plots[3].values, &[1.0, 5.0]);
    assert_values(&result.plots[4].values, &[2.0, 6.0]);
    assert_values(&result.plots[5].values, &[0.0, 3.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(6.0)))
        .expect("next forming update should start from confirmed varip array state");
    assert_values(&result.plots[0].values, &[1.0, 5.0, 6.0]);
    assert_values(&result.plots[1].values, &[1.0, 2.0, 3.0]);
    assert_values(&result.plots[2].values, &[2.0, 6.0, 7.0]);
    assert_values(&result.plots[3].values, &[1.0, 5.0, 6.0]);
    assert_values(&result.plots[4].values, &[2.0, 6.0, 7.0]);
    assert_values(&result.plots[5].values, &[0.0, 3.0, 4.0]);
}

#[test]
fn dynamic_history_fixture_rolls_back_forming_history() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/dynamic_history_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_values(&result.plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_values(&result.plots[0].values, &[1.0, 1.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should read confirmed history only");
    assert_values(&result.plots[0].values, &[1.0, 1.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit dynamic history");
    assert_values(&result.plots[0].values, &[1.0, 1.0]);
    assert_values(&runtime.confirmed_result().plots[0].values, &[1.0, 1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(5.0)))
        .expect("next forming update should use latest confirmed history");
    assert_values(&result.plots[0].values, &[1.0, 1.0, 4.0]);
}

#[test]
fn label_fixture_rolls_back_forming_lifecycle_changes() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/label_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.labels.len(), 1);
    assert_eq!(result.labels[0].id, 1);
    assert_eq!(result.labels[0].snapshots.len(), 1);
    assert_eq!(
        result.labels[0].snapshots[0].text,
        PineValue::String("confirmed".to_owned())
    );

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.labels.len(), 2);
    assert_eq!(result.labels[0].snapshots.len(), 3);
    assert_eq!(result.labels[0].snapshots[2].x, PineValue::Int(1));
    assert_eq!(result.labels[0].snapshots[2].y, PineValue::Float(2.0));
    assert_eq!(
        result.labels[0].snapshots[2].text,
        PineValue::String("forming".to_owned())
    );
    assert_eq!(result.labels[1].id, 2);
    assert_eq!(result.labels[1].snapshots.len(), 2);
    assert!(!result.labels[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().labels.len(), 1);
    assert_eq!(runtime.confirmed_result().labels[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back labels");
    assert_eq!(result.labels.len(), 2);
    assert_eq!(result.labels[0].id, 1);
    assert_eq!(result.labels[0].snapshots.len(), 2);
    assert!(!result.labels[0].snapshots[1].exists);
    assert_eq!(result.labels[1].id, 2);
    assert_eq!(result.labels[1].snapshots.len(), 2);
    assert!(!result.labels[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().labels.len(), 1);
    assert_eq!(runtime.confirmed_result().labels[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit from confirmed label state");
    assert_eq!(result.labels.len(), 1);
    assert_eq!(result.labels[0].id, 1);
    assert_eq!(result.labels[0].snapshots.len(), 1);
    assert!(result.labels[0].snapshots[0].exists);
}

#[test]
fn line_fixture_rolls_back_forming_lifecycle_changes() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/line_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.lines.len(), 1);
    assert_eq!(result.lines[0].id, 1);
    assert_eq!(result.lines[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.lines.len(), 2);
    assert_eq!(result.lines[0].snapshots.len(), 3);
    assert_eq!(result.lines[0].snapshots[2].x1, PineValue::Int(1));
    assert_eq!(result.lines[0].snapshots[2].y1, PineValue::Float(2.0));
    assert_eq!(
        result.lines[0].snapshots[2].color,
        PineValue::Color(0x4CAF50)
    );
    assert_eq!(result.lines[1].id, 2);
    assert_eq!(result.lines[1].snapshots.len(), 2);
    assert!(!result.lines[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().lines.len(), 1);
    assert_eq!(runtime.confirmed_result().lines[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back lines");
    assert_eq!(result.lines.len(), 2);
    assert_eq!(result.lines[0].id, 1);
    assert_eq!(result.lines[0].snapshots.len(), 2);
    assert!(!result.lines[0].snapshots[1].exists);
    assert_eq!(result.lines[1].id, 2);
    assert_eq!(result.lines[1].snapshots.len(), 2);
    assert!(!result.lines[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().lines.len(), 1);
    assert_eq!(runtime.confirmed_result().lines[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit from confirmed line state");
    assert_eq!(result.lines.len(), 1);
    assert_eq!(result.lines[0].id, 1);
    assert_eq!(result.lines[0].snapshots.len(), 1);
    assert!(result.lines[0].snapshots[0].exists);
}

#[test]
fn box_fixture_rolls_back_forming_lifecycle_changes() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/box_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.boxes.len(), 1);
    assert_eq!(result.boxes[0].id, 1);
    assert_eq!(result.boxes[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.boxes.len(), 2);
    assert_eq!(result.boxes[0].snapshots.len(), 3);
    assert_eq!(result.boxes[0].snapshots[2].left, PineValue::Int(1));
    assert_eq!(result.boxes[0].snapshots[2].top, PineValue::Float(2.0));
    assert_eq!(
        result.boxes[0].snapshots[2].bg_color,
        PineValue::Color(0x4CAF50)
    );
    assert_eq!(result.boxes[1].id, 2);
    assert_eq!(result.boxes[1].snapshots.len(), 2);
    assert!(!result.boxes[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().boxes.len(), 1);
    assert_eq!(runtime.confirmed_result().boxes[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back boxes");
    assert_eq!(result.boxes.len(), 2);
    assert_eq!(result.boxes[0].id, 1);
    assert_eq!(result.boxes[0].snapshots.len(), 2);
    assert!(!result.boxes[0].snapshots[1].exists);
    assert_eq!(result.boxes[1].id, 2);
    assert_eq!(result.boxes[1].snapshots.len(), 2);
    assert!(!result.boxes[1].snapshots[1].exists);
    assert_eq!(runtime.confirmed_result().boxes.len(), 1);
    assert_eq!(runtime.confirmed_result().boxes[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit from confirmed box state");
    assert_eq!(result.boxes.len(), 1);
    assert_eq!(result.boxes[0].id, 1);
    assert_eq!(result.boxes[0].snapshots.len(), 1);
    assert!(result.boxes[0].snapshots[0].exists);
}

#[test]
fn table_fixture_rolls_back_forming_cell_changes() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/table_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.tables.len(), 1);
    assert_eq!(result.tables[0].id, 1);
    assert_eq!(result.tables[0].snapshots.len(), 2);
    assert_eq!(
        result.tables[0].snapshots[1].cells[0].text,
        PineValue::String("confirmed".to_owned())
    );

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.tables.len(), 2);
    assert_eq!(result.tables[0].snapshots.len(), 3);
    assert_eq!(
        result.tables[0].snapshots[2].cells[0].text,
        PineValue::String("forming".to_owned())
    );
    assert_eq!(
        result.tables[0].snapshots[2].cells[0].bg_color,
        PineValue::Color(0x4CAF50)
    );
    assert_eq!(result.tables[1].id, 2);
    assert_eq!(result.tables[1].snapshots.len(), 2);
    assert_eq!(runtime.confirmed_result().tables.len(), 1);
    assert_eq!(runtime.confirmed_result().tables[0].snapshots.len(), 2);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back tables");
    assert_eq!(result.tables.len(), 2);
    assert_eq!(result.tables[0].id, 1);
    assert_eq!(result.tables[0].snapshots.len(), 3);
    assert_eq!(
        result.tables[0].snapshots[2].cells[0].text,
        PineValue::String("delete-like".to_owned())
    );
    assert_eq!(result.tables[1].id, 2);
    assert_eq!(result.tables[1].snapshots.len(), 2);
    assert_eq!(runtime.confirmed_result().tables.len(), 1);
    assert_eq!(runtime.confirmed_result().tables[0].snapshots.len(), 2);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit from confirmed table state");
    assert_eq!(result.tables.len(), 1);
    assert_eq!(result.tables[0].id, 1);
    assert_eq!(result.tables[0].snapshots.len(), 2);
    assert_eq!(
        result.tables[0].snapshots[1].cells[0].text,
        PineValue::String("confirmed".to_owned())
    );
}

#[test]
fn polyline_fixture_rolls_back_forming_lifecycle_changes() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/polyline_rollback.pine");

    let result = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update should run");
    assert_eq!(result.polylines.len(), 1);
    assert_eq!(result.polylines[0].id, 1);
    assert_eq!(result.polylines[0].snapshots.len(), 1);
    assert_eq!(result.polylines[0].snapshots[0].points.len(), 1);
    assert!(result.polylines[0].snapshots[0].exists);
    assert_values(&result.plots[0].values, &[1.0]);

    let result = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update should run");
    assert_eq!(result.polylines.len(), 2);
    assert_eq!(result.polylines[0].id, 1);
    assert_eq!(result.polylines[0].snapshots.len(), 1);
    assert!(result.polylines[0].snapshots[0].exists);
    assert_eq!(result.polylines[1].id, 2);
    assert_eq!(result.polylines[1].snapshots.len(), 2);
    assert_eq!(result.polylines[1].snapshots[0].points.len(), 2);
    assert_eq!(
        result.polylines[1].snapshots[0].line_color,
        PineValue::Color(0x4CAF50)
    );
    assert!(!result.polylines[1].snapshots[1].exists);
    assert_values(&result.plots[0].values, &[1.0, 1.0]);
    assert_eq!(runtime.confirmed_result().polylines.len(), 1);
    assert_eq!(runtime.confirmed_result().polylines[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update should roll back polylines");
    assert_eq!(result.polylines.len(), 2);
    assert_eq!(result.polylines[0].id, 1);
    assert_eq!(result.polylines[0].snapshots.len(), 2);
    assert!(!result.polylines[0].snapshots[1].exists);
    assert_eq!(result.polylines[1].id, 2);
    assert_eq!(result.polylines[1].snapshots.len(), 2);
    assert_eq!(result.polylines[1].snapshots[0].points.len(), 2);
    assert!(!result.polylines[1].snapshots[1].exists);
    assert_values(&result.plots[0].values, &[1.0, 0.0]);
    assert_eq!(runtime.confirmed_result().polylines.len(), 1);
    assert_eq!(runtime.confirmed_result().polylines[0].snapshots.len(), 1);

    let result = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update should commit from confirmed polyline state");
    assert_eq!(result.polylines.len(), 1);
    assert_eq!(result.polylines[0].id, 1);
    assert_eq!(result.polylines[0].snapshots.len(), 1);
    assert!(result.polylines[0].snapshots[0].exists);
    assert_values(&result.plots[0].values, &[1.0, 1.0]);
}

fn runtime_for_fixture(path: &str) -> RealtimeRuntime<'static> {
    let hir = hir_for_fixture(path);
    RealtimeRuntime::new(Box::leak(Box::new(hir)))
}

fn hir_for_fixture(path: &str) -> pine_ir::HirProgram {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    analysis.hir.expect("fixture should lower to HIR")
}

fn bar(close: f64) -> Bar {
    bar_ohlc(close, close)
}

fn bar_ohlc(open: f64, close: f64) -> Bar {
    Bar {
        time: 0,
        open,
        high: open.max(close),
        low: open.min(close),
        close,
        volume: 1.0,
    }
}

fn assert_values(actual: &[PineValue], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        let actual = actual
            .as_f64()
            .unwrap_or_else(|| panic!("expected numeric value, got {actual:?}"));
        assert!(
            (actual - expected).abs() < 1e-10,
            "expected {expected}, got {actual}"
        );
    }
}

fn assert_alerts(actual: &[AlertEvent], expected: &[(usize, &str, &str)]) {
    let actual: Vec<_> = actual
        .iter()
        .map(|event| {
            (
                event.bar_index,
                event.source.as_str(),
                event.message.as_str(),
            )
        })
        .collect();
    assert_eq!(actual, expected);
}

fn alert_summaries(alerts: &[AlertEvent]) -> Vec<(u32, usize, i64, &str, &str)> {
    alerts
        .iter()
        .map(|event| {
            (
                event.id,
                event.bar_index,
                event.time,
                event.source.as_str(),
                event.message.as_str(),
            )
        })
        .collect()
}
