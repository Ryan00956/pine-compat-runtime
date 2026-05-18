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
    Bar {
        time: 0,
        open: close,
        high: close,
        low: close,
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
