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
fn runs_strategy_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\", default_qty_type=strategy.fixed, default_qty_value=3)\nif bar_index == 1\n    strategy.entry(\"D\", strategy.long)\nplot(strategy.position_size)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy default quantity script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"D\",\"barIndex\":1,\"time\":1,\"direction\":\"strategy.long\",\"qty\":3,\"price\":2}]"
    ));
    assert!(output.contains("\"values\":[0,3]"));
}

#[test]
fn runs_strategy_position_state_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 2\n    strategy.close(\"L\")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy position state script should run");

    assert!(output.contains("\"values\":[0,0,2]"));
    assert!(output.contains("\"values\":[null,null,2]"));
    assert!(output.contains("\"values\":[0,2,0]"));
    assert!(output.contains("\"values\":[null,2,null]"));
}

#[test]
fn runs_strategy_profit_state_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\", initial_capital=1000)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy profit state script should run");

    assert!(output.contains("\"values\":[0,0,2]"));
    assert!(output.contains("\"values\":[0,0,0]"));
    assert!(output.contains("\"values\":[1000,1000,1002]"));
}

#[test]
fn runs_strategy_variable_interactions_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nscale(value) => value * 10\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.position_size[1])\nplot(strategy.openprofit[1])\nplot(scale(strategy.position_size))\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy variable interaction script should run");

    assert!(output.contains("\"values\":[null,0,2]"));
    assert!(output.contains("\"values\":[null,0,0]"));
    assert!(output.contains("\"values\":[0,20,20]"));
}

#[test]
fn runs_strategy_trade_counts_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=1)\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 2\n    strategy.close(\"L\")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy trade count script should run");

    assert!(output.contains("\"values\":[0,0,1]"));
    assert!(output.contains("\"values\":[0,1,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":["));
    assert!(output.contains("\"trades\":["));
    assert!(output.contains("\"position\":["));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
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
fn runs_strategy_exit_stop_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XL\", \"L\", stop=9)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,11,12,8,11,1\n",
    )
    .expect("strategy exit stop script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":0,\"time\":10,\"direction\":\"strategy.long\",\"qty\":2,\"price\":10},{\"id\":\"XL\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":0,\"exitBarIndex\":1,\"entryTime\":10,\"exitTime\":20,\"entryPrice\":10,\"exitPrice\":9,\"qty\":2,\"profit\":-2}]"
    ));
}

#[test]
fn runs_strategy_exit_limit_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XL\", \"L\", limit=12)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,11,12,10,11,1\n",
    )
    .expect("strategy exit limit script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":0,\"time\":10,\"direction\":\"strategy.long\",\"qty\":2,\"price\":10},{\"id\":\"XL\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":12}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":0,\"exitBarIndex\":1,\"entryTime\":10,\"exitTime\":20,\"entryPrice\":10,\"exitPrice\":12,\"qty\":2,\"profit\":4}]"
    ));
}

#[test]
fn runs_strategy_exit_profit_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XP\", \"L\", profit=200)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,11,12,10,11,1\n",
    )
    .expect("strategy exit profit script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":0,\"time\":10,\"direction\":\"strategy.long\",\"qty\":2,\"price\":10},{\"id\":\"XP\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":12}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":0,\"exitBarIndex\":1,\"entryTime\":10,\"exitTime\":20,\"entryPrice\":10,\"exitPrice\":12,\"qty\":2,\"profit\":4}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":0,\"size\":2,\"avgPrice\":10},{\"barIndex\":1,\"size\":0,\"avgPrice\":null}]"));
}

#[test]
fn runs_strategy_exit_loss_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XL\", \"L\", loss=100)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,10,10,9,10,1\n",
    )
    .expect("strategy exit loss script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":0,\"time\":10,\"direction\":\"strategy.long\",\"qty\":2,\"price\":10},{\"id\":\"XL\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":0,\"exitBarIndex\":1,\"entryTime\":10,\"exitTime\":20,\"entryPrice\":10,\"exitPrice\":9,\"qty\":2,\"profit\":-2}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":0,\"size\":2,\"avgPrice\":10},{\"barIndex\":1,\"size\":0,\"avgPrice\":null}]"));
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
fn json_escape_escapes_control_characters() {
    assert_eq!(
        json_escape("quote \" slash \\ newline\n tab\t bell\u{07}"),
        "quote \\\" slash \\\\ newline\\n tab\\t bell\\u0007"
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
