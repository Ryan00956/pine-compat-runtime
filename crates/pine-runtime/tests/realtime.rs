use std::{fs, path::PathBuf};

use pine_runtime::{Bar, BarUpdate, PineValue, RealtimeRuntime};
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

fn runtime_for_fixture(path: &str) -> RealtimeRuntime<'static> {
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
    let hir = analysis.hir.expect("fixture should lower to HIR");
    RealtimeRuntime::new(Box::leak(Box::new(hir)))
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
