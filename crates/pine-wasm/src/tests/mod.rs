use super::*;
use pine_runtime::{PUBLIC_ANALYSIS_SCHEMA_VERSION, PUBLIC_RUNTIME_SCHEMA_VERSION};
use std::{env, fs, path::PathBuf};

#[test]
fn analyzes_script_to_json() {
    let output = analyze_script("indicator(\"demo\")\nplot(close)\n");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_ANALYSIS_SCHEMA_VERSION
    )));
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

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
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
    assert!(output.contains("\"alerts\":[]"));
}

#[test]
fn runs_strategy_script_from_csv_to_empty_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nplot(close)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy script should run");

    assert!(output.contains("\"values\":[1,2]"));
    assert!(output.contains(
        "\"strategy\":{\"orders\":[],\"trades\":[],\"position\":[],\"equity\":[{\"barIndex\":0,\"cash\":100000,\"marketValue\":0,\"equity\":100000,\"netProfit\":0},{\"barIndex\":1,\"cash\":100000,\"marketValue\":0,\"equity\":100000,\"netProfit\":0}],\"diagnostics\":[]}"
    ));
}

#[test]
fn runs_strategy_entry_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(close)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy entry script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":1,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2}]"));
}

#[test]
fn runs_strategy_close_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nif bar_index == 2\n    strategy.close(\"L\")\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy close script should run");

    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":1,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":3,\"qty\":2,\"profit\":2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
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

const IMPORT_SOURCE: &str =
    "indicator(\"imports\")\nimport user/lib/1 as lib\nplot(lib.scale(close) + lib.offset)\n";
const IMPORT_LIBRARY_JSON: &str = "{\"user/lib/1\":\"library(\\\"lib\\\")\\nexport offset = 2\\nexport scale(value) => value * offset\\n\"}";

#[test]
fn library_source_json_runs_imported_function_subset() {
    let output = run_script_csv_with_libraries(
        IMPORT_SOURCE,
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
        IMPORT_LIBRARY_JSON,
    )
    .expect("imported function subset should run");

    assert!(output.contains("\"values\":[4,6]"));
}

#[test]
fn library_source_json_reports_missing_library() {
    let output = analyze_script("import user/lib/1\nindicator(\"root\")\n");

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"feature\":\"import\""));
    assert!(output.contains("\"code\":\"E_IMPORT_MISSING_LIBRARY\""));
    assert!(output.contains("\"code\":\"E_IMPORT_ALIAS_REQUIRED\""));
}

#[test]
fn library_source_json_reports_malformed_host_input() {
    let output = analyze_script_with_libraries(IMPORT_SOURCE, "[]");

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_HOST_INPUT\""));
    assert!(output.contains("library sources must be a JSON object"));
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
