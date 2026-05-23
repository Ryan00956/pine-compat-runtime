use super::*;
use std::{env, fs, path::PathBuf};

#[test]
fn analyzes_script_to_json() {
    let output = analyze_script("indicator(\"demo\")\nplot(close)\n");

    assert!(output.contains("\"schemaVersion\":2"));
    assert!(output.contains("\"executable\":true"));
    assert!(output.contains("\"feature\":\"plot\""));
}

#[test]
fn runs_script_from_csv_to_json() {
    let output = run_script_csv(
        "indicator(\"demo\")\nplot(close)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("script should run");

    assert!(output.contains("\"schemaVersion\":2"));
    assert!(output.contains("\"values\":[1,2]"));
    assert!(output.contains("\"plotChars\":[]"));
    assert!(output.contains("\"plotShapes\":[]"));
    assert!(output.contains("\"plotArrows\":[]"));
    assert!(output.contains("\"plotBars\":[]"));
    assert!(output.contains("\"plotCandles\":[]"));
    assert!(output.contains("\"labels\":[]"));
    assert!(output.contains("\"lines\":[]"));
    assert!(output.contains("\"boxes\":[]"));
    assert!(output.contains("\"tables\":[]"));
}

#[test]
fn request_host_data_is_documented_wasm_gap() {
    let message = run_script_csv_internal(
        "indicator(\"request\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, close))\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
    )
    .expect_err("wasm has no request dataset injection yet");

    assert!(
        message.contains("missing request data for symbol `NYSE:IBM` timeframe `1`"),
        "{message}"
    );
}

#[test]
fn analysis_outputs_match_golden_snapshots() {
    assert_snapshot(
        "analysis_supported.json",
        &analyze_script(include_str!(
            "../../../../tests/fixtures/runtime/snapshot_plot.pine"
        )),
    );
    assert_snapshot(
        "analysis_unsupported.json",
        &analyze_script(include_str!(
            "../../../../tests/fixtures/sema/unsupported_request.pine"
        )),
    );
}

fn assert_snapshot(name: &str, actual: &str) {
    let snapshot_path = workspace_dir().join("tests/snapshots").join(name);
    if env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::create_dir_all(snapshot_path.parent().expect("snapshot parent"))
            .expect("create snapshot dir");
        fs::write(&snapshot_path, format!("{actual}\n")).expect("write snapshot");
        return;
    }

    let expected = fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", snapshot_path.display()));
    assert_eq!(actual.trim_end(), expected.trim_end(), "{name} changed");
}

fn workspace_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
