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
fn runs_alert_frequency_fixture_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alert_frequency.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("alert frequency fixture should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(
        parsed["alerts"],
        serde_json::json!([
            {
                "id": 1,
                "barIndex": 0,
                "time": 0,
                "message": "Default once",
                "source": "alert"
            },
            {
                "id": 2,
                "barIndex": 0,
                "time": 0,
                "message": "Explicit once",
                "source": "alert"
            },
            {
                "id": 3,
                "barIndex": 0,
                "time": 0,
                "message": "All",
                "source": "alert"
            },
            {
                "id": 3,
                "barIndex": 0,
                "time": 0,
                "message": "All",
                "source": "alert"
            },
            {
                "id": 4,
                "barIndex": 0,
                "time": 0,
                "message": "Close",
                "source": "alert"
            }
        ])
    );
}

#[test]
fn run_script_csv_serializes_non_finite_values_as_json_null() {
    let output = run_script_csv(
        "indicator(\"nonfinite\")\nplot(1.0 / 0.0)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
    )
    .expect("script should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["plots"][0]["values"][0], serde_json::Value::Null);
    assert!(!output.contains("NaN"));
    assert!(!output.contains("Infinity"));
}

#[test]
fn run_script_csv_returns_plotchar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotchar.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("plotchar fixture should run");

    assert_snapshot("runtime_plotchar.json", &output);
}

#[test]
fn run_script_csv_returns_plotshape_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotshape.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("plotshape fixture should run");

    assert_snapshot("runtime_plotshape.json", &output);
}

#[test]
fn run_script_csv_returns_plotarrow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotarrow.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("plotarrow fixture should run");

    assert_snapshot("runtime_plotarrow.json", &output);
}

#[test]
fn run_script_csv_returns_plotbar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotbar.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("plotbar fixture should run");

    assert_snapshot("runtime_plotbar.json", &output);
}

#[test]
fn run_script_csv_returns_plotcandle_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotcandle.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("plotcandle fixture should run");

    assert_snapshot("runtime_plotcandle.json", &output);
}

#[test]
fn run_script_csv_returns_color_outputs_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/color_outputs.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("color outputs fixture should run");

    assert_snapshot("runtime_color_outputs.json", &output);
}

#[test]
fn run_script_csv_returns_hline_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/io.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("hline/fill fixture should run");

    assert_snapshot("runtime_hline_fill.json", &output);
}

#[test]
fn run_script_csv_returns_alertcondition_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alertcondition.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("alertcondition fixture should run");

    assert_snapshot("runtime_alertcondition.json", &output);
}

#[test]
fn run_script_csv_returns_alert_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alert.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("alert fixture should run");

    assert_snapshot("runtime_alert.json", &output);
}

#[test]
fn run_script_csv_returns_label_new_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_new.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label.new fixture should run");

    assert_snapshot("runtime_label_new.json", &output);
}

#[test]
fn run_script_csv_returns_label_mutation_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_mutation.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label mutation fixture should run");

    assert_snapshot("runtime_label_mutation.json", &output);
}

#[test]
fn run_script_csv_returns_label_control_flow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_control_flow.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label control-flow fixture should run");

    assert_snapshot("runtime_label_control_flow.json", &output);
}

#[test]
fn run_script_csv_returns_label_delete_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_delete.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label delete fixture should run");

    assert_snapshot("runtime_label_delete.json", &output);
}

#[test]
fn run_script_csv_returns_label_copy_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_copy.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label copy fixture should run");

    assert_snapshot("runtime_label_copy.json", &output);
}

#[test]
fn run_script_csv_returns_label_getters_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_getters.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label getters fixture should run");

    assert_snapshot("runtime_label_getters.json", &output);
}

#[test]
fn run_script_csv_returns_label_options_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_options.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label options fixture should run");

    assert_snapshot("runtime_label_options.json", &output);
}

#[test]
fn run_script_csv_returns_label_xloc_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_xloc.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label xloc fixture should run");

    assert_snapshot("runtime_label_xloc.json", &output);
}

#[test]
fn run_script_csv_returns_label_yloc_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_yloc.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label yloc fixture should run");

    assert_snapshot("runtime_label_yloc.json", &output);
}

#[test]
fn run_script_csv_returns_label_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("label array fixture should run");

    assert_snapshot("runtime_label_array.json", &output);
}

#[test]
fn run_script_csv_returns_array_helpers_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_helpers.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array helpers fixture should run");

    assert_snapshot("runtime_array_helpers.json", &output);
}

#[test]
fn run_script_csv_returns_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array fixture should run");

    assert_snapshot("runtime_array.json", &output);
}

#[test]
fn run_script_csv_returns_array_methods_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_methods.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array methods fixture should run");

    assert_snapshot("runtime_array_methods.json", &output);
}

#[test]
fn run_script_csv_returns_computed_array_operands_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/computed_array_operands.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("computed array operands fixture should run");

    assert_snapshot("runtime_computed_array_operands.json", &output);
}

#[test]
fn run_script_csv_returns_array_from_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_from.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array from fixture should run");

    assert_snapshot("runtime_array_from.json", &output);
}

#[test]
fn run_script_csv_returns_array_insert_remove_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_insert_remove.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array insert remove fixture should run");

    assert_snapshot("runtime_array_insert_remove.json", &output);
}

#[test]
fn run_script_csv_returns_array_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_fill.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array fill fixture should run");

    assert_snapshot("runtime_array_fill.json", &output);
}

#[test]
fn run_script_csv_returns_array_clear_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_clear.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array clear fixture should run");

    assert_snapshot("runtime_array_clear.json", &output);
}

#[test]
fn run_script_csv_returns_array_references_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_references.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array references fixture should run");

    assert_snapshot("runtime_array_references.json", &output);
}

#[test]
fn run_script_csv_returns_array_search_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_search.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array search fixture should run");

    assert_snapshot("runtime_array_search.json", &output);
}

#[test]
fn run_script_csv_returns_array_statistics_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_statistics.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array statistics fixture should run");

    assert_snapshot("runtime_array_statistics.json", &output);
}

#[test]
fn run_script_csv_returns_array_ordering_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_ordering.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array ordering fixture should run");

    assert_snapshot("runtime_array_ordering.json", &output);
}

#[test]
fn run_script_csv_returns_array_join_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_join.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array join fixture should run");

    assert_snapshot("runtime_array_join.json", &output);
}

#[test]
fn run_script_csv_returns_array_slice_concat_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/array_slice_concat.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("array slice concat fixture should run");

    assert_snapshot("runtime_array_slice_concat.json", &output);
}

#[test]
fn run_script_csv_returns_line_new_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_new.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line new fixture should run");

    assert_snapshot("runtime_line_new.json", &output);
}

#[test]
fn run_script_csv_returns_line_mutation_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_mutation.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line mutation fixture should run");

    assert_snapshot("runtime_line_mutation.json", &output);
}

#[test]
fn run_script_csv_returns_line_control_flow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_control_flow.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line control-flow fixture should run");

    assert_snapshot("runtime_line_control_flow.json", &output);
}

#[test]
fn run_script_csv_returns_line_getters_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_getters.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line getters fixture should run");

    assert_snapshot("runtime_line_getters.json", &output);
}

#[test]
fn run_script_csv_returns_line_delete_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_delete.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line delete fixture should run");

    assert_snapshot("runtime_line_delete.json", &output);
}

#[test]
fn run_script_csv_returns_line_copy_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_copy.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line copy fixture should run");

    assert_snapshot("runtime_line_copy.json", &output);
}

#[test]
fn run_script_csv_returns_line_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/line_array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("line array fixture should run");

    assert_snapshot("runtime_line_array.json", &output);
}

#[test]
fn run_script_csv_returns_box_new_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_new.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box new fixture should run");

    assert_snapshot("runtime_box_new.json", &output);
}

#[test]
fn run_script_csv_returns_box_mutation_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_mutation.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box mutation fixture should run");

    assert_snapshot("runtime_box_mutation.json", &output);
}

#[test]
fn run_script_csv_returns_box_control_flow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_control_flow.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box control-flow fixture should run");

    assert_snapshot("runtime_box_control_flow.json", &output);
}

#[test]
fn run_script_csv_returns_box_getters_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_getters.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box getters fixture should run");

    assert_snapshot("runtime_box_getters.json", &output);
}

#[test]
fn run_script_csv_returns_box_delete_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_delete.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box delete fixture should run");

    assert_snapshot("runtime_box_delete.json", &output);
}

#[test]
fn run_script_csv_returns_box_copy_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_copy.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box copy fixture should run");

    assert_snapshot("runtime_box_copy.json", &output);
}

#[test]
fn run_script_csv_returns_box_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/box_array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("box array fixture should run");

    assert_snapshot("runtime_box_array.json", &output);
}

#[test]
fn run_script_csv_returns_table_new_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_new.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table new fixture should run");

    assert_snapshot("runtime_table_new.json", &output);
}

#[test]
fn run_script_csv_returns_table_cell_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_cell.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table cell fixture should run");

    assert_snapshot("runtime_table_cell.json", &output);
}

#[test]
fn run_script_csv_returns_table_control_flow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_control_flow.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table control-flow fixture should run");

    assert_snapshot("runtime_table_control_flow.json", &output);
}

#[test]
fn run_script_csv_returns_table_delete_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_delete.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table delete fixture should run");

    assert_snapshot("runtime_table_delete.json", &output);
}

#[test]
fn run_script_csv_returns_table_clear_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_clear.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table clear fixture should run");

    assert_snapshot("runtime_table_clear.json", &output);
}

#[test]
fn run_script_csv_returns_table_merge_cells_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_merge_cells.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table merge cells fixture should run");

    assert_snapshot("runtime_table_merge_cells.json", &output);
}

#[test]
fn run_script_csv_returns_table_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/table_array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("table array fixture should run");

    assert_snapshot("runtime_table_array.json", &output);
}

#[test]
fn run_script_csv_returns_drawing_methods_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/drawing_methods.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("drawing methods fixture should run");

    assert_snapshot("runtime_drawing_methods.json", &output);
}

#[test]
fn run_script_csv_returns_loop_state_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/loop_state_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("loop state interactions fixture should run");

    assert_snapshot("runtime_loop_state_interactions.json", &output);
}

#[test]
fn run_script_csv_returns_branch_loop_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/branch_loop_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("branch loop interactions fixture should run");

    assert_snapshot("runtime_branch_loop_interactions.json", &output);
}

#[test]
fn run_script_csv_returns_switch_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/switch.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("switch fixture should run");

    assert_snapshot("runtime_switch.json", &output);
}

#[test]
fn run_script_csv_returns_block_statements_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/block_statements.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("block statements fixture should run");

    assert_snapshot("runtime_block_statements.json", &output);
}

#[test]
fn run_script_csv_returns_for_edges_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/for_edges.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("for edges fixture should run");

    assert_snapshot("runtime_for_edges.json", &output);
}

#[test]
fn run_script_csv_returns_for_stateful_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/for_stateful.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("for stateful fixture should run");

    assert_snapshot("runtime_for_stateful.json", &output);
}

#[test]
fn run_script_csv_returns_while_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/while.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("while fixture should run");

    assert_snapshot("runtime_while.json", &output);
}

#[test]
fn run_script_csv_returns_while_edges_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/while_edges.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("while edges fixture should run");

    assert_snapshot("runtime_while_edges.json", &output);
}

#[test]
fn run_script_csv_returns_while_stateful_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/while_stateful.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("while stateful fixture should run");

    assert_snapshot("runtime_while_stateful.json", &output);
}

#[test]
fn run_script_csv_returns_local_scope_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/local_scope.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("local scope fixture should run");

    assert_snapshot("runtime_local_scope.json", &output);
}

#[test]
fn run_script_csv_returns_history_edges_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/history_edges.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("history edges fixture should run");

    assert_snapshot("runtime_history_edges.json", &output);
}

#[test]
fn run_script_csv_returns_dynamic_history_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dynamic_history.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("dynamic history fixture should run");

    assert_snapshot("runtime_dynamic_history.json", &output);
}

#[test]
fn run_script_csv_returns_dynamic_history_scopes_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dynamic_history_scopes.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("dynamic history scopes fixture should run");

    assert_snapshot("runtime_dynamic_history_scopes.json", &output);
}

#[test]
fn run_script_csv_returns_series_history_offset_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/series_history_offset.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("series history offset fixture should run");

    assert_snapshot("runtime_series_history_offset.json", &output);
}

#[test]
fn run_script_csv_returns_max_bars_back_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/max_bars_back.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("max bars back fixture should run");

    assert_snapshot("runtime_max_bars_back.json", &output);
}

#[test]
fn run_script_csv_returns_varip_scalar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/varip_scalar.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("varip scalar fixture should run");

    assert_snapshot("runtime_varip_scalar.json", &output);
}

#[test]
fn run_script_csv_returns_varip_local_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/varip_local.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("varip local fixture should run");

    assert_snapshot("runtime_varip_local.json", &output);
}

#[test]
fn run_script_csv_returns_varip_array_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/varip_array.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("varip array fixture should run");

    assert_snapshot("runtime_varip_array.json", &output);
}

#[test]
fn run_script_csv_returns_request_security_same_context_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/request_security_same_context.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("request.security same-context fixture should run");

    assert_snapshot("runtime_request_security_same_context.json", &output);
}

#[test]
fn run_script_csv_returns_user_types_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/user_types.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("user-defined types fixture should run");

    assert_snapshot("runtime_user_types.json", &output);
}

#[test]
fn run_script_csv_returns_user_type_functions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/user_type_functions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("user-defined type functions fixture should run");

    assert_snapshot("runtime_user_type_functions.json", &output);
}

#[test]
fn run_script_csv_returns_user_methods_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/user_methods.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("user-defined methods fixture should run");

    assert_snapshot("runtime_user_methods.json", &output);
}

#[test]
fn analyze_script_reports_unsupported_user_type_field_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_user_type.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_UDT_FIELD_TYPE\""));
}

#[test]
fn analyze_script_reports_unsupported_user_method_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_user_method.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_METHOD_RECEIVER_TYPE\""));
}

#[test]
fn analyze_script_reports_unsupported_user_type_varip_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_user_type_varip.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_UNSUPPORTED_FEATURE\""));
    assert!(output.contains("\"feature\":\"varip\""));
    assert!(output.contains("other value families"));
}

#[test]
fn analyze_script_reports_unsupported_user_type_field_mutation_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_user_type_field_mutation.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_UNSUPPORTED_FEATURE\""));
    assert!(output.contains("\"feature\":\"function_side_effect\""));
    assert!(output.contains("mutating user-defined type fields"));
}

#[test]
fn analyze_script_reports_unsupported_user_method_side_effect_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_user_method_side_effect.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_UNSUPPORTED_FEATURE\""));
    assert!(output.contains("\"feature\":\"function_side_effect\""));
    assert!(output.contains("inside user-defined functions"));
}

#[test]
fn analyze_script_reports_unsupported_non_array_method_fixture() {
    let output = analyze_script(include_str!(
        "../../../../tests/fixtures/sema/unsupported_non_array_method.pine"
    ));

    assert!(output.contains("\"executable\":false"));
    assert!(output.contains("\"code\":\"E_METHOD_RECEIVER_TYPE\""));
}

#[test]
fn run_script_csv_returns_math_edge_cases_as_json_null() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/math_edge_cases.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("math edge-case fixture should run");

    assert_snapshot("runtime_math_edge_cases.json", &output);
}

#[test]
fn run_script_csv_returns_math_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/math.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("math fixture should run");

    assert_snapshot("runtime_math.json", &output);
}

#[test]
fn run_script_csv_returns_computed_lengths_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/computed_lengths.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("computed lengths fixture should run");

    assert_snapshot("runtime_computed_lengths.json", &output);
}

#[test]
fn run_script_csv_returns_conditional_ta_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/conditional_ta.pine"),
        "time,open,high,low,close,volume\n0,1,2,1,2,1\n1,2,4,2,4,1\n2,5,5,3,3,1\n3,3,6,3,6,1\n",
    )
    .expect("conditional TA fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0]["values"], serde_json::json!([null, 3, 3, 5]));
}

#[test]
fn run_script_csv_returns_conditional_ta_snapshot_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/conditional_ta.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("conditional TA fixture should run");

    assert_snapshot("runtime_conditional_ta.json", &output);
}

#[test]
fn run_script_csv_returns_udf_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/udf.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("UDF fixture should run");

    assert_snapshot("runtime_udf.json", &output);
}

#[test]
fn run_script_csv_returns_na_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/na.pine"),
        "time,open,high,low,close,volume\n0,1,2,1,2,1\n1,5,5,3,3,1\n2,2,4,2,4,1\n3,6,6,5,5,1\n",
    )
    .expect("NA fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 5);
    assert_eq!(plots[0]["values"], serde_json::json!([2, 2, 3, 4]));
    assert_eq!(plots[1]["values"], serde_json::json!([2, 2, 3, 4]));
    assert_eq!(plots[2]["values"], serde_json::json!([2, 2, 4, 4]));
    assert_eq!(plots[3]["values"], serde_json::json!([0, 0, 0, 0]));
    assert_eq!(plots[4]["values"], serde_json::json!([0, 0, 0, 0]));
}

#[test]
fn run_script_csv_returns_na_snapshot_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/na.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("NA fixture should run");

    assert_snapshot("runtime_na.json", &output);
}

#[test]
fn run_script_csv_returns_ta_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/ta.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("TA fixture should run");

    assert_snapshot("runtime_ta.json", &output);
}

#[test]
fn run_script_csv_returns_dema_tema_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dema_tema.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("DEMA/TEMA fixture should run");

    assert_snapshot("runtime_dema_tema.json", &output);
}

#[test]
fn run_script_csv_returns_swma_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/swma.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("swma fixture should run");

    assert_snapshot("runtime_swma.json", &output);
}

#[test]
fn run_script_csv_returns_stoch_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/stoch.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("stoch fixture should run");

    assert_snapshot("runtime_stoch.json", &output);
}

#[test]
fn run_script_csv_returns_wpr_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/wpr.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("wpr fixture should run");

    assert_snapshot("runtime_wpr.json", &output);
}

#[test]
fn run_script_csv_returns_atr_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/atr.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("atr fixture should run");

    assert_snapshot("runtime_atr.json", &output);
}

#[test]
fn run_script_csv_returns_supertrend_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/supertrend.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("supertrend fixture should run");

    assert_snapshot("runtime_supertrend.json", &output);
}

#[test]
fn run_script_csv_returns_dmi_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dmi.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("dmi fixture should run");

    assert_snapshot("runtime_dmi.json", &output);
}

#[test]
fn run_script_csv_returns_sar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/sar.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("sar fixture should run");

    assert_snapshot("runtime_sar.json", &output);
}

#[test]
fn run_script_csv_returns_cross_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/cross.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("cross fixture should run");

    assert_snapshot("runtime_cross.json", &output);
}

#[test]
fn run_script_csv_returns_mom_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/mom.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("mom fixture should run");

    assert_snapshot("runtime_mom.json", &output);
}

#[test]
fn run_script_csv_returns_roc_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/roc.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("roc fixture should run");

    assert_snapshot("runtime_roc.json", &output);
}

#[test]
fn run_script_csv_returns_trend_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/trend.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("trend fixture should run");

    assert_snapshot("runtime_trend.json", &output);
}

#[test]
fn run_script_csv_returns_barssince_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/barssince.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("barssince fixture should run");

    assert_snapshot("runtime_barssince.json", &output);
}

#[test]
fn run_script_csv_returns_valuewhen_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/valuewhen.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("valuewhen fixture should run");

    assert_snapshot("runtime_valuewhen.json", &output);
}

#[test]
fn run_script_csv_returns_extremes_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/extremes.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("extremes fixture should run");

    assert_snapshot("runtime_extremes.json", &output);
}

#[test]
fn run_script_csv_returns_extreme_bars_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/extreme_bars.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("extreme bars fixture should run");

    assert_snapshot("runtime_extreme_bars.json", &output);
}

#[test]
fn run_script_csv_returns_tsi_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/tsi.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("tsi fixture should run");

    assert_snapshot("runtime_tsi.json", &output);
}

#[test]
fn run_script_csv_returns_cmo_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/cmo.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("cmo fixture should run");

    assert_snapshot("runtime_cmo.json", &output);
}

#[test]
fn run_script_csv_returns_cci_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/cci.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("cci fixture should run");

    assert_snapshot("runtime_cci.json", &output);
}

#[test]
fn run_script_csv_returns_cog_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/cog.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("cog fixture should run");

    assert_snapshot("runtime_cog.json", &output);
}

#[test]
fn run_script_csv_returns_ao_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/ao.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("ao fixture should run");

    assert_snapshot("runtime_ao.json", &output);
}

#[test]
fn run_script_csv_returns_bop_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/bop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("bop fixture should run");

    assert_snapshot("runtime_bop.json", &output);
}

#[test]
fn run_script_csv_returns_bb_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/bb.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("bb fixture should run");

    assert_snapshot("runtime_bb.json", &output);
}

#[test]
fn run_script_csv_returns_bbw_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/bbw.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("bbw fixture should run");

    assert_snapshot("runtime_bbw.json", &output);
}

#[test]
fn run_script_csv_returns_kc_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/kc.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("kc fixture should run");

    assert_snapshot("runtime_kc.json", &output);
}

#[test]
fn run_script_csv_returns_kcw_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/kcw.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("kcw fixture should run");

    assert_snapshot("runtime_kcw.json", &output);
}

#[test]
fn run_script_csv_returns_pivots_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/pivots.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("pivots fixture should run");

    assert_snapshot("runtime_pivots.json", &output);
}

#[test]
fn run_script_csv_returns_pivot_point_levels_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/pivot_point_levels.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("pivot point levels fixture should run");

    assert_snapshot("runtime_pivot_point_levels.json", &output);
}

#[test]
fn run_script_csv_returns_cum_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/cum.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("cum fixture should run");

    assert_snapshot("runtime_cum.json", &output);
}

#[test]
fn run_script_csv_returns_all_time_extremes_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/all_time_extremes.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("all-time extremes fixture should run");

    assert_snapshot("runtime_all_time_extremes.json", &output);
}

#[test]
fn run_script_csv_returns_alma_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alma.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("alma fixture should run");

    assert_snapshot("runtime_alma.json", &output);
}

#[test]
fn run_script_csv_returns_linreg_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/linreg.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("linreg fixture should run");

    assert_snapshot("runtime_linreg.json", &output);
}

#[test]
fn run_script_csv_returns_accdist_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/accdist.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("accdist fixture should run");

    assert_snapshot("runtime_accdist.json", &output);
}

#[test]
fn run_script_csv_returns_iii_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/iii.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("iii fixture should run");

    assert_snapshot("runtime_iii.json", &output);
}

#[test]
fn run_script_csv_returns_nvi_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/nvi.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("nvi fixture should run");

    assert_snapshot("runtime_nvi.json", &output);
}

#[test]
fn run_script_csv_returns_obv_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/obv.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("obv fixture should run");

    assert_snapshot("runtime_obv.json", &output);
}

#[test]
fn run_script_csv_returns_pvi_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/pvi.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("pvi fixture should run");

    assert_snapshot("runtime_pvi.json", &output);
}

#[test]
fn run_script_csv_returns_pvt_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/pvt.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("pvt fixture should run");

    assert_snapshot("runtime_pvt.json", &output);
}

#[test]
fn run_script_csv_returns_vwap_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/vwap.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("vwap fixture should run");

    assert_snapshot("runtime_vwap.json", &output);
}

#[test]
fn run_script_csv_returns_wad_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/wad.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("wad fixture should run");

    assert_snapshot("runtime_wad.json", &output);
}

#[test]
fn run_script_csv_returns_wvad_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/wvad.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("wvad fixture should run");

    assert_snapshot("runtime_wvad.json", &output);
}

#[test]
fn run_script_csv_returns_correlation_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/correlation.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("correlation fixture should run");

    assert_snapshot("runtime_correlation.json", &output);
}

#[test]
fn run_script_csv_returns_covariance_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/covariance.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("covariance fixture should run");

    assert_snapshot("runtime_covariance.json", &output);
}

#[test]
fn run_script_csv_returns_median_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/median.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("median fixture should run");

    assert_snapshot("runtime_median.json", &output);
}

#[test]
fn run_script_csv_returns_mode_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/mode.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("mode fixture should run");

    assert_snapshot("runtime_mode.json", &output);
}

#[test]
fn run_script_csv_returns_percentile_linear_interpolation_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/percentile_linear_interpolation.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("percentile linear interpolation fixture should run");

    assert_snapshot("runtime_percentile_linear_interpolation.json", &output);
}

#[test]
fn run_script_csv_returns_percentile_nearest_rank_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/percentile_nearest_rank.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("percentile nearest rank fixture should run");

    assert_snapshot("runtime_percentile_nearest_rank.json", &output);
}

#[test]
fn run_script_csv_returns_percentrank_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/percentrank.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("percentrank fixture should run");

    assert_snapshot("runtime_percentrank.json", &output);
}

#[test]
fn run_script_csv_returns_stdev_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/stdev.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("stdev fixture should run");

    assert_snapshot("runtime_stdev.json", &output);
}

#[test]
fn run_script_csv_returns_variance_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/variance.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("variance fixture should run");

    assert_snapshot("runtime_variance.json", &output);
}

#[test]
fn run_script_csv_returns_range_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/range.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("range fixture should run");

    assert_snapshot("runtime_range.json", &output);
}

#[test]
fn run_script_csv_returns_dev_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dev.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("dev fixture should run");

    assert_snapshot("runtime_dev.json", &output);
}

#[test]
fn run_script_csv_returns_vwma_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/vwma.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("vwma fixture should run");

    assert_snapshot("runtime_vwma.json", &output);
}

#[test]
fn run_script_csv_returns_mfi_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/mfi.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("mfi fixture should run");

    assert_snapshot("runtime_mfi.json", &output);
}

#[test]
fn run_script_csv_returns_wma_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/wma.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("wma fixture should run");

    assert_snapshot("runtime_wma.json", &output);
}

#[test]
fn run_script_csv_returns_hma_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/hma.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("hma fixture should run");

    assert_snapshot("runtime_hma.json", &output);
}

#[test]
fn run_script_csv_returns_if_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/if.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("if fixture should run");

    assert_snapshot("runtime_if.json", &output);
}

#[test]
fn run_script_csv_returns_macd_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/macd.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("MACD fixture should run");

    assert_snapshot("runtime_macd.json", &output);
}

#[test]
fn run_script_csv_returns_strings_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strings.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strings fixture should run");

    assert_snapshot("runtime_strings.json", &output);
}

#[test]
fn run_script_csv_returns_colors_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/colors.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("colors fixture should run");

    assert_snapshot("runtime_colors.json", &output);
}

#[test]
fn run_script_csv_returns_syminfo_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/syminfo.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("syminfo fixture should run");

    assert_snapshot("runtime_syminfo.json", &output);
}

#[test]
fn run_script_csv_returns_generic_input_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/generic_input.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("generic input fixture should run");

    assert_snapshot("runtime_generic_input.json", &output);
}

#[test]
fn run_script_csv_returns_timeframe_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/timeframe.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("timeframe fixture should run");

    assert_snapshot("runtime_timeframe.json", &output);
}

#[test]
fn run_script_csv_returns_time_components_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/time_components.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("time components fixture should run");

    assert_snapshot("runtime_time_components.json", &output);
}

#[test]
fn run_script_csv_returns_global_series_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/global_series.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("global series fixture should run");

    assert_snapshot("runtime_global_series.json", &output);
}

#[test]
fn run_script_csv_returns_casts_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/casts.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("casts fixture should run");

    assert_snapshot("runtime_casts.json", &output);
}

#[test]
fn run_script_csv_returns_barstate_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/barstate.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("barstate fixture should run");

    assert_snapshot("runtime_barstate.json", &output);
}

#[test]
fn run_script_csv_returns_session_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/session.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("session fixture should run");

    assert_snapshot("runtime_session.json", &output);
}

#[test]
fn run_script_csv_returns_inputs_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/inputs.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("inputs fixture should run");

    assert_snapshot("runtime_inputs.json", &output);
}

#[test]
fn run_script_csv_rejects_non_finite_ohlcv_values() {
    for (column, row) in [
        ("open", "0,NaN,1,1,1,1"),
        ("high", "0,1,inf,1,1,1"),
        ("low", "0,1,1,-inf,1,1"),
        ("close", "0,1,1,1,infinity,1"),
        ("volume", "0,1,1,1,1,NaN"),
    ] {
        let message = run_script_csv_internal(
            "indicator(\"nonfinite\")\nplot(close)\n",
            &format!("time,open,high,low,close,volume\n{row}\n"),
        )
        .expect_err("non-finite CSV value should fail");

        assert!(
            message.contains(&format!("invalid `{column}` value")),
            "{message}"
        );
        assert!(message.contains("value must be finite"), "{message}");
    }
}

#[test]
fn run_script_csv_rejects_duplicate_bar_times() {
    let message = run_script_csv_internal(
        "indicator(\"duplicate\")\nplot(close)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n0,2,2,2,2,1\n",
    )
    .expect_err("duplicate main bar time should fail");

    assert_eq!(message, "duplicate bar time `0` in bars CSV");
}

#[test]
fn run_script_csv_rejects_unsorted_bar_times() {
    let message = run_script_csv_internal(
        "indicator(\"unsorted\")\nplot(close)\n",
        "time,open,high,low,close,volume\n1,2,2,2,2,1\n0,1,1,1,1,1\n",
    )
    .expect_err("unsorted main bar times should fail");

    assert_eq!(message, "bars CSV is not sorted: `0` follows `1`");
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
fn runs_strategy_exit_missing_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        "strategy(\"exit\")\nif bar_index == 0\n    strategy.exit(\"XL\", \"L\", stop=low)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_while_flat_noop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit while-flat no-op script should run");

    assert_snapshot("runtime_strategy_exit_while_flat_noop.json", &output);
}

#[test]
fn runs_strategy_exit_limit_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_limit_while_flat_noop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit limit while-flat no-op script should run");

    assert_snapshot("runtime_strategy_exit_limit_while_flat_noop.json", &output);
}

#[test]
fn runs_strategy_exit_profit_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit profit while-flat no-op script should run");

    assert_snapshot("runtime_strategy_exit_profit_while_flat_noop.json", &output);
}

#[test]
fn runs_strategy_exit_loss_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_loss_while_flat_noop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit loss while-flat no-op script should run");

    assert_snapshot("runtime_strategy_exit_loss_while_flat_noop.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket while-flat no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_while_flat_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_stop_profit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_stop_profit_bracket_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit stop profit bracket while-flat no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_stop_profit_bracket_while_flat_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_loss_limit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_limit_bracket_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit loss limit bracket while-flat no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_loss_limit_bracket_while_flat_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_loss_profit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_profit_bracket_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit loss profit bracket while-flat no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_loss_profit_bracket_while_flat_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_trailing_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_trailing_while_flat_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit trailing while-flat no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_trailing_while_flat_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_wrong_entry_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy wrong-entry no-op fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_limit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_limit_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy limit wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_limit_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_profit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy profit wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_profit_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_loss_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy loss wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_loss_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy bracket wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_stop_profit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy stop profit bracket wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_loss_limit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy loss limit bracket wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_loss_profit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy loss profit bracket wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_trailing_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_trailing_unmatched_from_entry_noop.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy trailing wrong-entry no-op script should run");

    assert_snapshot(
        "runtime_strategy_exit_trailing_unmatched_from_entry_noop.json",
        &output,
    );
}

#[test]
fn runs_strategy_entry_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_entry.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy entry script should run");

    assert_snapshot("runtime_strategy_entry.json", &output);
}

#[test]
fn runs_strategy_entry_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_entry.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy entry fixture should run");

    assert_snapshot("runtime_strategy_entry.json", &output);
}

#[test]
fn runs_strategy_entry_limit_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_entry_limit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy limit entry script should run");

    assert!(output.contains("\"values\":[0,2,2,2]"));
    assert!(output.contains("\"values\":[null,2,2,2]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2}]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("limit"));
}

#[test]
fn runs_strategy_entry_stop_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_entry_stop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy stop entry script should run");

    assert!(output.contains("\"values\":[0,0,2,2]"));
    assert!(output.contains("\"values\":[null,null,3,3]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":2,\"price\":3}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":2,\"size\":2,\"avgPrice\":3}]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("stop"));
}

#[test]
fn runs_strategy_entry_stop_limit_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_entry_stop_limit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy stop-limit entry script should run");

    assert!(output.contains("\"values\":[0,0,0,2]"));
    assert!(output.contains("\"values\":[null,null,null,4]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":2,\"price\":4}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":3,\"size\":2,\"avgPrice\":4}]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("stop"));
    assert!(!output.contains("limit"));
}

#[test]
fn runs_strategy_pyramiding_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_pyramiding.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy pyramiding fixture should run");

    assert!(output.contains("\"values\":[0,1,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4]"));
    assert!(output.contains("\"values\":[null,2,2.75,2.75]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75}]"
    ));
    assert!(output.contains("\"trades\":[]"));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_pyramiding_close_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_pyramiding_close.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy pyramiding close fixture should run");

    assert!(output.contains("\"values\":[0,1,1,0]"));
    assert!(output.contains("\"values\":[0,1,3,0]"));
    assert!(output.contains("\"values\":[0,0,1,2]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":1,\"profit\":1},{\"id\":\"L2\",\"entryBarIndex\":2,\"exitBarIndex\":3,\"entryTime\":3,\"exitTime\":4,\"entryPrice\":3,\"exitPrice\":4,\"qty\":3,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":2,\"size\":3,\"avgPrice\":3},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closeTrades"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_close_all_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_pyramiding_close_all.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy pyramiding close_all fixture should run");

    assert!(output.contains("\"values\":[0,1,0,0]"));
    assert!(output.contains("\"values\":[0,1,0,0]"));
    assert!(output.contains("\"values\":[0,0,2,2]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":1,\"profit\":1},{\"id\":\"L2\",\"entryBarIndex\":2,\"exitBarIndex\":2,\"entryTime\":3,\"exitTime\":3,\"entryPrice\":3,\"exitPrice\":3,\"qty\":3,\"profit\":0}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closeTrades"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_exit_from_entry_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_pyramiding_exit_from_entry.pine"),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_from_entry_bars.csv"
        ),
    )
    .expect("strategy pyramiding exit from_entry fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains("\"values\":[0,1,2,2,1]"));
    assert!(output.contains("\"values\":[0,1,4,4,3]"));
    assert!(output.contains("\"values\":[0,0,0,0,1]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XL1\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":3,\"qty\":1,\"profit\":1}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":3,\"size\":3,\"avgPrice\":3}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_exit_profit_from_entry_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry_bars.csv"
        ),
    )
    .expect("strategy pyramiding profit exit from_entry fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains("\"values\":[0,1,2,2,1]"));
    assert!(output.contains("\"values\":[0,1,4,4,3]"));
    assert!(output.contains("\"values\":[0,0,0,0,1]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XP1\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":3,\"size\":3,\"avgPrice\":3}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_exit_same_id_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_pyramiding_exit_same_id.pine"),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_same_id_bars.csv"
        ),
    )
    .expect("strategy pyramiding same-id exit fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains("\"values\":[0,1,2,2,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,2]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XL\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XL\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":5,\"qty\":1,\"profit\":3},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":3,\"entryTime\":3,\"exitTime\":4,\"entryPrice\":4,\"exitPrice\":5,\"qty\":3,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_exit_bracket_from_entry_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_bracket_from_entry.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry_bars.csv"
        ),
    )
    .expect("strategy pyramiding bracket exit from_entry fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains("\"values\":[0,1,2,2,1]"));
    assert!(output.contains("\"values\":[0,1,4,4,3]"));
    assert!(output.contains("\"values\":[0,0,0,0,1]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XB1\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":3,\"size\":3,\"avgPrice\":3}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_pyramiding_exit_trail_points_from_entry_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_trail_points_from_entry.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_trail_points_from_entry_bars.csv"
        ),
    )
    .expect("strategy pyramiding trailing exit from_entry fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains("\"values\":[0,1,2,2,2,1]"));
    assert!(output.contains("\"values\":[0,1,4,4,4,3]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XT1\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":1,\"profit\":1.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":4}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("trailPrice"));
    assert!(!output.contains("trailOffset"));
}

#[test]
fn runs_strategy_same_tick_limit_entries_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick limit entries script should run");

    assert!(output.contains("\"values\":[0,2,2]"));
    assert!(output.contains("\"values\":[0,4,4]"));
    assert!(output.contains("\"values\":[null,9,9]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":9},{\"id\":\"L2\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":9},{\"barIndex\":1,\"size\":4,\"avgPrice\":9}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_same_tick_limit_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick limit entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_limit_same_tick_limit_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_same_tick_stop_entries_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick stop entries script should run");

    assert!(output.contains("\"values\":[0,2,2]"));
    assert!(output.contains("\"values\":[0,4,4]"));
    assert!(output.contains("\"values\":[null,11,11]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":11},{\"id\":\"L2\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":3,\"price\":11}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":11},{\"barIndex\":1,\"size\":4,\"avgPrice\":11}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_same_tick_stop_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick stop entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_limit_same_tick_stop_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_same_tick_stop_limit_entries_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick stop-limit entries script should run");

    assert!(output.contains("\"values\":[0,0,2,2]"));
    assert!(output.contains("\"values\":[0,0,4,4]"));
    assert!(output.contains("\"values\":[null,null,10,10]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":1,\"price\":10},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":10}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":2,\"size\":1,\"avgPrice\":10},{\"barIndex\":2,\"size\":4,\"avgPrice\":10}]"
    ));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_same_tick_stop_limit_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries_bars.csv"
        ),
    )
    .expect("strategy same-tick stop-limit entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_limit_same_tick_stop_limit_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_default_quantity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy default quantity script should run");

    assert_snapshot("runtime_strategy_default_quantity.json", &output);
}

#[test]
fn runs_strategy_default_quantity_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_default_quantity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy default quantity fixture should run");

    assert_snapshot("runtime_strategy_default_quantity.json", &output);
}

#[test]
fn runs_strategy_builtin_default_quantity_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_builtin_default_quantity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy builtin default quantity fixture should run");

    assert_snapshot("runtime_strategy_builtin_default_quantity.json", &output);
}

#[test]
fn runs_strategy_default_quantity_override_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_default_quantity_override.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy default quantity override fixture should run");

    assert_snapshot("runtime_strategy_default_quantity_override.json", &output);
}

#[test]
fn runs_strategy_percent_of_equity_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_percent_of_equity_default_quantity.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy percent default quantity script should run");

    assert_snapshot(
        "runtime_strategy_percent_of_equity_default_quantity.json",
        &output,
    );
}

#[test]
fn runs_strategy_percent_of_equity_default_quantity_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_percent_of_equity_default_quantity.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy percent default quantity fixture should run");

    assert_snapshot(
        "runtime_strategy_percent_of_equity_default_quantity.json",
        &output,
    );
}

#[test]
fn runs_strategy_cash_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cash_default_quantity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash default quantity script should run");

    assert_snapshot("runtime_strategy_cash_default_quantity.json", &output);
}

#[test]
fn runs_strategy_cash_default_quantity_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cash_default_quantity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash default quantity fixture should run");

    assert_snapshot("runtime_strategy_cash_default_quantity.json", &output);
}

#[test]
fn runs_strategy_cash_default_quantity_limit_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_cash_default_quantity_limit.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash default quantity limit fixture should run");

    assert_snapshot("runtime_strategy_cash_default_quantity_limit.json", &output);
}

#[test]
fn runs_strategy_cash_default_quantity_override_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_cash_default_quantity_override.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash default quantity override fixture should run");

    assert_snapshot(
        "runtime_strategy_cash_default_quantity_override.json",
        &output,
    );
}

#[test]
fn runs_strategy_commission_cash_per_contract_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_commission_cash_per_contract.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash-per-contract commission fixture should run");

    assert_snapshot(
        "runtime_strategy_commission_cash_per_contract.json",
        &output,
    );
}

#[test]
fn runs_strategy_commission_cash_per_order_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_commission_cash_per_order.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cash-per-order commission fixture should run");

    assert_snapshot("runtime_strategy_commission_cash_per_order.json", &output);
}

#[test]
fn runs_strategy_commission_percent_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_commission_percent.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy percent commission fixture should run");

    assert_snapshot("runtime_strategy_commission_percent.json", &output);
}

#[test]
fn runs_strategy_slippage_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_slippage.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy slippage fixture should run");

    assert_snapshot("runtime_strategy_slippage.json", &output);
}

#[test]
fn runs_strategy_exit_slippage_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_slippage.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit slippage fixture should run");

    assert_snapshot("runtime_strategy_exit_slippage.json", &output);
}

#[test]
fn runs_strategy_limit_verification_entry_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_limit_verification_entry.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy limit verification entry fixture should run");

    assert_snapshot("runtime_strategy_limit_verification_entry.json", &output);
}

#[test]
fn runs_strategy_limit_verification_exit_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_limit_verification_exit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy limit verification exit fixture should run");

    assert_snapshot("runtime_strategy_limit_verification_exit.json", &output);
}

#[test]
fn runs_strategy_position_state_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_position_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy position state script should run");

    assert_snapshot("runtime_strategy_position_state.json", &output);
}

#[test]
fn runs_strategy_position_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_position_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy position state fixture should run");

    assert_snapshot("runtime_strategy_position_state.json", &output);
}

#[test]
fn runs_strategy_equity_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_equity.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy equity fixture should run");

    assert_snapshot("runtime_strategy_equity.json", &output);
}

#[test]
fn runs_strategy_profit_state_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy profit state script should run");

    assert_snapshot("runtime_strategy_profit_state.json", &output);
}

#[test]
fn runs_strategy_profit_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy profit state fixture should run");

    assert_snapshot("runtime_strategy_profit_state.json", &output);
}

#[test]
fn runs_strategy_variable_interactions_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_variable_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy variable interaction script should run");

    assert_snapshot("runtime_strategy_variable_interactions.json", &output);
}

#[test]
fn runs_strategy_variable_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_variable_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy variable interactions fixture should run");

    assert_snapshot("runtime_strategy_variable_interactions.json", &output);
}

#[test]
fn runs_strategy_trade_counts_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_counts.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy trade count script should run");

    assert_snapshot("runtime_strategy_trade_counts.json", &output);
}

#[test]
fn runs_strategy_trade_counts_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_counts.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy trade count fixture should run");

    assert_snapshot("runtime_strategy_trade_counts.json", &output);
}

#[test]
fn runs_strategy_exit_trade_counts_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trade_counts.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit trade count fixture should run");

    assert_snapshot("runtime_strategy_exit_trade_counts.json", &output);
}

#[test]
fn runs_strategy_closedtrades_fields_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_closedtrades_fields.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy closed trade fields script should run");

    assert_snapshot("runtime_strategy_closedtrades_fields.json", &output);
}

#[test]
fn runs_strategy_opentrades_fields_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_opentrades_fields.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_opentrades_fields_bars.csv"),
    )
    .expect("strategy open trade fields script should run");

    assert_snapshot("runtime_strategy_opentrades_fields.json", &output);
}

#[test]
fn runs_strategy_margin_capital_held_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_capital_held_long.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy margin capital held script should run");

    assert_snapshot("runtime_strategy_margin_capital_held_long.json", &output);
}

#[test]
fn runs_strategy_margin_entry_affordability_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_margin_entry_affordability_long.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy margin entry affordability script should run");

    assert_snapshot("runtime_strategy_margin_entry_affordability.json", &output);
}

#[test]
fn runs_strategy_margin_call_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_call_long.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_call_long_bars.csv"),
    )
    .expect("strategy margin call script should run");

    assert_snapshot("runtime_strategy_margin_call_long.json", &output);
}

#[test]
fn runs_strategy_trade_outcome_counts_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )
    .expect("strategy trade outcome count script should run");

    assert_snapshot("runtime_strategy_trade_outcome_counts.json", &output);
}

#[test]
fn runs_strategy_trade_outcome_counts_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )
    .expect("strategy trade outcome count fixture should run");

    assert_snapshot("runtime_strategy_trade_outcome_counts.json", &output);
}

#[test]
fn runs_strategy_profit_percent_state_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_percent_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )
    .expect("strategy profit percent state script should run");

    assert_snapshot("runtime_strategy_profit_percent_state.json", &output);
}

#[test]
fn runs_strategy_profit_percent_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_percent_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )
    .expect("strategy profit percent state fixture should run");

    assert_snapshot("runtime_strategy_profit_percent_state.json", &output);
}

#[test]
fn runs_strategy_close_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close script should run");

    assert_snapshot("runtime_strategy_close.json", &output);
}

#[test]
fn runs_strategy_close_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close fixture should run");

    assert_snapshot("runtime_strategy_close.json", &output);
}

#[test]
fn runs_strategy_close_qty_partial_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close_qty_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close qty partial fixture should run");

    assert_snapshot("runtime_strategy_close_qty_partial.json", &output);
}

#[test]
fn runs_strategy_close_qty_full_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close_qty_full_clamp.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close qty full clamp fixture should run");

    assert_snapshot("runtime_strategy_close_qty_full_clamp.json", &output);
}

#[test]
fn runs_strategy_close_qty_percent_precedence_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_close_qty_percent_precedence.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close qty_percent precedence fixture should run");

    assert_snapshot(
        "runtime_strategy_close_qty_percent_precedence.json",
        &output,
    );
}

#[test]
fn runs_strategy_close_all_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close_all.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close_all script should run");

    assert_snapshot("runtime_strategy_close_all.json", &output);
}

#[test]
fn runs_strategy_close_all_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close_all.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close_all fixture should run");

    assert_snapshot("runtime_strategy_close_all.json", &output);
}

#[test]
fn runs_strategy_exit_stop_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_stop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit stop script should run");

    assert_snapshot("runtime_strategy_exit_stop.json", &output);
}

#[test]
fn runs_strategy_exit_stop_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_stop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit stop fixture should run");

    assert_snapshot("runtime_strategy_exit_stop.json", &output);
}

#[test]
fn runs_strategy_cancel_entry_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cancel_entry.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cancel entry script should run");

    assert_snapshot("runtime_strategy_cancel_entry.json", &output);
}

#[test]
fn runs_strategy_cancel_all_entry_exit_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cancel all entry exit script should run");

    assert_snapshot("runtime_strategy_cancel_all_entry_exit.json", &output);
}

#[test]
fn runs_strategy_exit_limit_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XL\", \"L\", limit=12)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,11,12,10,11,1\n",
    )
    .expect("strategy exit limit script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.long\",\"qty\":2,\"price\":11},{\"id\":\"XL\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":12}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":20,\"exitTime\":20,\"entryPrice\":11,\"exitPrice\":12,\"qty\":2,\"profit\":2}]"
    ));
}

#[test]
fn runs_strategy_exit_limit_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_limit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit limit fixture should run");

    assert_snapshot("runtime_strategy_exit_limit.json", &output);
}

#[test]
fn runs_strategy_exit_profit_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_profit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit profit script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XP\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"));
}

#[test]
fn runs_strategy_exit_loss_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_loss.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_loss_bars.csv"),
    )
    .expect("strategy exit loss script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":10},{\"id\":\"XL\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":10,\"exitPrice\":9,\"qty\":2,\"profit\":-2}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":10},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"));
}

#[test]
fn runs_strategy_exit_profit_loss_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_loss_interactions.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_loss_interactions_bars.csv"
        ),
    )
    .expect("strategy exit profit/loss interactions fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_profit_loss_interactions.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_fixture_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_both_hit.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_both_hit_bars.csv"),
    )
    .expect("strategy exit bracket fixture should run");

    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":100},{\"id\":\"XB\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":95}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":100,\"exitPrice\":95,\"qty\":2,\"profit\":-10}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":100},{\"barIndex\":1,\"size\":0,\"avgPrice\":null}]"
    ));
}

#[test]
fn runs_strategy_exit_bracket_creation_bar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_creation_bar.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket creation-bar fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_creation_bar.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket interactions fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_interactions.json", &output);
}

#[test]
fn runs_strategy_exit_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit interactions fixture should run");

    assert_snapshot("runtime_strategy_exit_interactions.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_invalid_leg_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_invalid_leg.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket invalid-leg fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_invalid_leg.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_loss_profit_loss_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_loss_profit_loss_fill.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_loss_profit_loss_bars.csv"
        ),
    )
    .expect("strategy exit loss-profit bracket loss-fill fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_loss_profit_loss_fill.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_loss_profit_profit_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_loss_profit_profit_fill.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit loss-profit bracket profit-fill fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_loss_profit_profit_fill.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_mixed_pairs_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_mixed_pairs.pine"),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_mixed_pairs_bars.csv"
        ),
    )
    .expect("strategy exit mixed bracket pairs fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_mixed_pairs.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_repeated_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_repeated.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit repeated bracket fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_repeated.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_replacement.pine"),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_replacement_bars.csv"
        ),
    )
    .expect("strategy exit bracket replacement fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_replacement.json", &output);
}

#[test]
fn runs_strategy_exit_omitted_bracket_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_omitted_bracket_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit omitted bracket replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_omitted_bracket_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_bracket_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket state fixture should run");

    assert_snapshot("runtime_strategy_exit_bracket_state.json", &output);
}

#[test]
fn runs_strategy_exit_bracket_stop_limit_limit_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_stop_limit_limit_fill.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit stop-limit bracket limit-fill fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_stop_limit_limit_fill.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_bracket_stop_limit_stop_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_stop_limit_stop_fill.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit stop-limit bracket stop-fill fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_bracket_stop_limit_stop_fill.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_trailing_fixture_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trail_price_fill.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XT\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
}

#[test]
fn runs_strategy_exit_trail_points_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trail_points_fill.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trail_points fill fixture should run");

    assert_snapshot("runtime_strategy_exit_trail_points_fill.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing state fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_state.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_replacement.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing replacement fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_replacement.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_activation_bar_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_trailing_activation_bar.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing activation bar fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_trailing_activation_bar.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_trailing_ratchet_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_ratchet.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing ratchet fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_ratchet.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_repeated_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_repeated.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing repeated fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_repeated.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_invalid_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_invalid.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing invalid fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_invalid.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_close_cancel_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_close_cancel.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing close cancel fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_close_cancel.json", &output);
}

#[test]
fn runs_strategy_exit_trailing_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_interactions.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit trailing interactions fixture should run");

    assert_snapshot("runtime_strategy_exit_trailing_interactions.json", &output);
}

#[test]
fn runs_strategy_exit_omitted_trailing_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_omitted_trailing_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit omitted trailing replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_omitted_trailing_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_partial_fixture_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_stop_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit partial quantity fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XQ\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":0.75,\"price\":2.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":0.75,\"profit\":0.375}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":1,\"size\":1.25,\"avgPrice\":2}]"
    ));
    assert!(!output.contains("pending"));
    assert!(!output.contains("remainingQty"));
}

#[test]
fn runs_strategy_exit_qty_limit_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_limit_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty limit partial fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_limit_partial.json", &output);
}

#[test]
fn runs_strategy_exit_qty_bracket_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_bracket_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty bracket partial fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_bracket_partial.json", &output);
}

#[test]
fn runs_strategy_exit_reservation_qty_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_clamp.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty clamp fixture should run");

    assert_snapshot("runtime_strategy_exit_reservation_qty_clamp.json", &output);
}

#[test]
fn runs_strategy_exit_reservation_qty_stop_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_stop_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty stop multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_stop_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_limit_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_limit_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty limit multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_limit_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_stop_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_stop_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent stop multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_stop_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_mixed_stop_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_stop_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty mixed stop multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_mixed_stop_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_clamp.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent clamp fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_clamp.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_bracket_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_clamp.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty bracket clamp fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_bracket_clamp.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_bracket_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty bracket replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_bracket_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_bracket_stop_limit_downside_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_stop_limit_downside_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty bracket stop limit downside multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_bracket_stop_limit_downside_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_bracket_stop_limit_upside_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_stop_limit_upside_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty bracket stop limit upside multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_bracket_stop_limit_upside_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_mixed_bracket_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_bracket_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty mixed bracket multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_mixed_bracket_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_bracket_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_multi.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent bracket multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_bracket_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_bracket_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent bracket replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_bracket_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_bracket_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_clamp.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation qty percent bracket clamp fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_bracket_clamp.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_mixed_trailing_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_trailing_multi.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty mixed trailing multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_mixed_trailing_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_state_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_state.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing state fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_state.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_trailing_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_clamp.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty trailing clamp fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_trailing_clamp.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_trailing_points_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_points_multi.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty trailing points multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_trailing_points_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_trailing_price_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_price_multi.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty trailing price multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_trailing_price_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_trailing_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_replacement.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty trailing replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_trailing_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_activation_mixed_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_activation_mixed_fill.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing activation mixed fill fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_activation_mixed_fill.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_single_downside_order_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_single_downside_order.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing single downside order fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_single_downside_order.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_bracket_downside_order_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bracket_downside_order.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing bracket downside order fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_bracket_downside_order.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_mixed_side_precedence_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_side_precedence.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing mixed side precedence fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_mixed_side_precedence.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_mixed_state_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_state.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing mixed state fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_mixed_state.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_trailing_replacement_mixed_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_replacement_mixed.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )
    .expect("strategy exit reservation trailing replacement mixed fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_trailing_replacement_mixed.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_trailing_multi_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_multi.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty percent trailing multi fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_trailing_multi.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_trailing_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_replacement.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty percent trailing replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_trailing_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_qty_percent_trailing_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_clamp.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"
        ),
    )
    .expect("strategy exit reservation qty percent trailing clamp fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_qty_percent_trailing_clamp.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_trailing_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_trailing_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit qty trailing partial fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_trailing_partial.json", &output);
}

#[test]
fn runs_strategy_exit_qty_full_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_full_clamp.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty full clamp fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_full_clamp.json", &output);
}

#[test]
fn runs_strategy_exit_qty_repeated_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_repeated.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty repeated fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_repeated.json", &output);
}

#[test]
fn runs_strategy_exit_qty_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_replacement.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty replacement fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_replacement.json", &output);
}

#[test]
fn runs_strategy_exit_qty_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty state fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_state.json", &output);
}

#[test]
fn runs_strategy_exit_qty_precedence_fixture_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_precedence_stop.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty precedence fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XQ\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":0.75,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":0.75,\"profit\":0.75}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":1.25,\"avgPrice\":2}]"
    ));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_exit_qty_precedence_bracket_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_precedence_bracket.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty precedence bracket fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_precedence_bracket.json", &output);
}

#[test]
fn runs_strategy_exit_qty_precedence_trailing_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_precedence_trailing.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_precedence_trailing_bars.csv"
        ),
    )
    .expect("strategy exit qty precedence trailing fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_qty_precedence_trailing.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_percent_partial_fixture_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_stop_partial.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit percent quantity fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XP\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":2.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":1,\"profit\":0.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":1,\"size\":1,\"avgPrice\":2}]"
    ));
    assert!(output.contains("\"values\":[0,2,1,1]"));
    assert!(output.contains("\"values\":[0,0,0.5,0.5]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
}

#[test]
fn runs_strategy_exit_qty_percent_limit_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_limit_partial.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent limit partial fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_qty_percent_limit_partial.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_percent_bracket_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_bracket_partial.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent bracket partial fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_qty_percent_bracket_partial.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_percent_trailing_partial_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_trailing_partial.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit qty percent trailing partial fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_qty_percent_trailing_partial.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_percent_full_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_percent_full.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent full fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_percent_full.json", &output);
}

#[test]
fn runs_strategy_exit_qty_percent_full_clamp_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_full_clamp.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent full clamp fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_percent_full_clamp.json", &output);
}

#[test]
fn runs_strategy_exit_qty_percent_repeated_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_percent_repeated.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent repeated fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_percent_repeated.json", &output);
}

#[test]
fn runs_strategy_exit_qty_percent_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_qty_percent_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_qty_percent_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_qty_percent_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_qty_percent_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit qty percent state fixture should run");

    assert_snapshot("runtime_strategy_exit_qty_percent_state.json", &output);
}

#[test]
fn runs_strategy_exit_reservation_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_mixed_side_precedence.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XS\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":0.5,\"price\":2.5},{\"id\":\"XL\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":1.5,\"price\":1.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":0.5,\"profit\":0.25},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":1.5,\"qty\":1.5,\"profit\":-0.75}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":1,\"size\":1.5,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
}

#[test]
fn runs_strategy_exit_reservation_state_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_reservation_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation state fixture should run");

    assert_snapshot("runtime_strategy_exit_reservation_state.json", &output);
}

#[test]
fn runs_strategy_exit_reservation_interactions_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_interactions.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation interactions fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_interactions.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_omitted_single_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_omitted_single_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit omitted single replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_omitted_single_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_omitted_replaces_reservations_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_omitted_replaces_reservations.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit omitted replacement fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XFULL\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":2.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":2,\"profit\":1}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,0]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"trades\":["));
    assert!(output.contains("\"position\":["));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQuantity"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("triggerSide"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_current_all_entry_exit_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_current.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_current_bars.csv"
        ),
    )
    .expect("strategy omitted current all-entry exit fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XL\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XL\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":5,\"qty\":1,\"profit\":3},{\"id\":\"L2\",\"entryBarIndex\":2,\"exitBarIndex\":3,\"entryTime\":3,\"exitTime\":4,\"entryPrice\":4,\"exitPrice\":5,\"qty\":3,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_persistent_all_entry_exit_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_persistent.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_persistent_bars.csv"
        ),
    )
    .expect("strategy omitted persistent all-entry exit fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_from_entry_persistent.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_profit_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted profit from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_profit_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_profit_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted profit same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XP\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4},{\"id\":\"XP\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":6}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":4,\"exitPrice\":6,\"qty\":3,\"profit\":6}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":3,\"size\":3,\"avgPrice\":4},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_profit_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted profit same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_profit_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XL\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":6},{\"id\":\"XL\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":4}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":8,\"exitPrice\":6,\"qty\":1,\"profit\":-2},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":4,\"qty\":3,\"profit\":-6}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":2,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":3,\"size\":3,\"avgPrice\":6},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_loss_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_profit_bracket_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss+profit bracket same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":8}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":8,\"exitPrice\":6,\"qty\":1,\"profit\":-2},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":8,\"qty\":3,\"profit\":6}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":2,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":3,\"size\":3,\"avgPrice\":6},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_loss_profit_bracket_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss+profit bracket same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_profit_bracket_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss-profit bracket from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_stop_profit_bracket_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted stop-profit bracket from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_limit_bracket_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss-limit bracket from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_stop_limit_bracket_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted stop-limit bracket from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_trail_points_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted trail-points from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_points_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_trail_price_from_entries_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted trail-price from entries fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_price_from_entries.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_stop_profit_bracket_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop+profit bracket same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":8},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":3,\"entryTime\":3,\"exitTime\":4,\"entryPrice\":6,\"exitPrice\":8,\"qty\":3,\"profit\":6},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":2,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":3,\"size\":1,\"avgPrice\":8},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,1,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_stop_profit_bracket_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop+profit bracket same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_limit_bracket_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss+limit bracket same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":8,\"exitPrice\":6,\"qty\":1,\"profit\":-2},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":2,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":3,\"size\":3,\"avgPrice\":6},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_loss_limit_bracket_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss+limit bracket same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_stop_limit_bracket_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop+limit bracket same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":9},{\"id\":\"XB\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":8,\"exitPrice\":9,\"qty\":1,\"profit\":1},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":3,\"entryTime\":3,\"exitTime\":4,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":2,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,0]"));
    assert!(output.contains("\"values\":[0,1,4,4,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_stop_limit_bracket_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop+limit bracket same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_trail_points_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail_points same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":3.5},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":1,\"profit\":1.5},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":3.5,\"qty\":3,\"profit\":1.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":3,\"avgPrice\":3},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"values\":[0,0,0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_trail_points_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail_points same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_points_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_trail_price_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail_price same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":3.5},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":1,\"profit\":1.5},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":3.5,\"qty\":3,\"profit\":1.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"values\":[0,0,0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("closedTrades"));
}

#[test]
fn runs_strategy_omitted_trail_price_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail_price same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_price_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_active_entry_attachment_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_attachment.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit active-entry attachment fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XL\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":2.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":2,\"profit\":1}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":1,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,0,0]"));
    assert!(output.contains("\"values\":[0,0,1,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_attachment_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_attachment.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit active-entry attachment fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_active_entry_attachment.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_active_entry_profit_attachment_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_profit_attachment.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit active-entry profit attachment fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XP\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":2.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":2.5,\"qty\":2,\"profit\":1}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,0]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_loss_attachment_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_loss_attachment.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit active-entry loss attachment fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":3},{\"id\":\"XL\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":2}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":3,\"exitPrice\":2,\"qty\":2,\"profit\":-2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":3},{\"barIndex\":1,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,0,0]"));
    assert!(output.contains("\"values\":[0,0,1,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_trail_points_attachment_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_trail_points_attachment.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit active-entry trail-points attachment fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XT\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":3,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,2]"));
    assert!(output.contains("\"values\":[0,0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_stop_profit_bracket_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_stop_profit_bracket.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit active-entry stop-profit bracket fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XB\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,0]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_loss_limit_bracket_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_loss_limit_bracket.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit active-entry loss-limit bracket fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XB\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,0]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_active_entry_loss_profit_bracket_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_active_entry_loss_profit_bracket.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )
    .expect("strategy exit active-entry loss-profit bracket fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 1);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XB\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":3.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3.5,\"qty\":2,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,0]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservation"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_exit_bracket_reservation_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_bracket_host_parity.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit bracket reservation fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2},{\"id\":\"XB1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.exit\",\"qty\":0.5,\"price\":2},{\"id\":\"XB2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":2,\"qty\":0.5,\"profit\":0},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":1,\"profit\":1}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":1,\"size\":1.5,\"avgPrice\":2},{\"barIndex\":2,\"size\":0.5,\"avgPrice\":2}]"
    ));
    assert!(output.contains("\"values\":[0,2,1.5,0.5]"));
    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("bracketLeg"));
    assert!(!output.contains("bracket"));
}

#[test]
fn runs_strategy_exit_reservation_bracket_single_downside_precedence_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_bracket_single_downside_precedence.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation bracket single downside precedence fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_bracket_single_downside_precedence.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_bracket_single_replacement_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_bracket_single_replacement.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation bracket single replacement fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_bracket_single_replacement.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_bracket_single_upside_order_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_bracket_single_upside_order.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation bracket single upside order fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_bracket_single_upside_order.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_reservation_bracket_state_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_bracket_state.pine"
        ),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy exit reservation bracket state fixture should run");

    assert_snapshot(
        "runtime_strategy_exit_reservation_bracket_state.json",
        &output,
    );
}

#[test]
fn runs_strategy_exit_trailing_reservation_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_host_parity.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_reservation_trailing_host_parity_bars.csv"
        ),
    )
    .expect("strategy exit trailing reservation fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":3},{\"id\":\"XT1\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.exit\",\"qty\":0.75,\"price\":3.5},{\"id\":\"XT2\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1.25,\"price\":3.3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":3,\"exitPrice\":3.5,\"qty\":0.75,\"profit\":0.375},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":3.3,\"qty\":1.25,\"profit\":0.3749999999999998}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":3},{\"barIndex\":3,\"size\":1.25,\"avgPrice\":3},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,2,2,2,1.25]"));
    assert!(output.contains("\"values\":[0,0,0,0,0.375]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("trailing"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_profit_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted profit persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XP\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XP\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":7}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":5,\"qty\":1,\"profit\":3},{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":4,\"exitPrice\":7,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":3,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":4},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_loss_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XL\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XL\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":3,\"qty\":3,\"profit\":-9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_profit_persistent_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted profit persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":4},{\"id\":\"XP\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XP\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":7}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":5,\"qty\":1,\"profit\":3},{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":4,\"exitPrice\":7,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":3,\"size\":4,\"avgPrice\":3.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":4},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_profit_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted profit persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_profit_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_persistent_same_id_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XL\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XL\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":3,\"qty\":3,\"profit\":-9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_loss_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_profit_bracket_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss-profit bracket persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_stop_profit_bracket_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted stop-profit bracket persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":4,\"entryTime\":4,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9},{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":5,\"entryTime\":2,\"exitTime\":6,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":1,\"avgPrice\":8},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,1,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_omitted_loss_profit_bracket_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss-profit bracket persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_loss_profit_bracket_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss-profit bracket persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_omitted_stop_profit_bracket_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop-profit bracket persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":4,\"entryTime\":4,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":5,\"entryTime\":2,\"exitTime\":6,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":1,\"avgPrice\":8},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,1,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_stop_profit_bracket_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop-profit bracket persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_omitted_loss_limit_bracket_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss-limit bracket persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_omitted_stop_limit_bracket_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop-limit bracket persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":9},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":9,\"qty\":1,\"profit\":1},{\"id\":\"L\",\"entryBarIndex\":3,\"exitBarIndex\":4,\"entryTime\":4,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,0]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,2]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_stop_limit_bracket_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted stop-limit bracket persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_omitted_trail_price_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail-price persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4.5},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":4.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":4.5,\"qty\":1,\"profit\":2.5},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":4.5,\"qty\":3,\"profit\":4.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"values\":[0,0,0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("trailing"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_trail_price_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail-price persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_omitted_trail_points_persistent_same_id_from_csv_to_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail-points persistent same-id fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":4}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2},{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":4,\"qty\":3,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":3,\"avgPrice\":3},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"values\":[0,0,0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("trailing"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("targetTradeKey"));
    assert!(!output.contains("target_trade_key"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_trail_points_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted trail-points persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_limit_bracket_persistent_same_id_fixture_contract() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id_bars.csv"
        ),
    )
    .expect("strategy omitted loss-limit bracket persistent same-id fixture should run");

    assert_snapshot(
        "runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.json",
        &output,
    );
}

#[test]
fn runs_strategy_omitted_loss_limit_bracket_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted loss-limit bracket persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":5},{\"id\":\"XB\",\"barIndex\":5,\"time\":6,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":5,\"qty\":1,\"profit\":-3},{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":5,\"entryTime\":4,\"exitTime\":6,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":3,\"avgPrice\":6},{\"barIndex\":5,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,1,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,3,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_stop_limit_bracket_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted stop-limit bracket persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":8},{\"id\":\"L2\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":3,\"price\":6},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":9},{\"id\":\"XB\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":8,\"exitPrice\":9,\"qty\":1,\"profit\":1},{\"id\":\"L2\",\"entryBarIndex\":3,\"exitBarIndex\":4,\"entryTime\":4,\"exitTime\":5,\"entryPrice\":6,\"exitPrice\":9,\"qty\":3,\"profit\":9}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":8},{\"barIndex\":3,\"size\":4,\"avgPrice\":6.5},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,1,2,2,0]"));
    assert!(output.contains("\"values\":[0,1,1,4,4,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("profitTarget"));
    assert!(!output.contains("stopLoss"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_trail_price_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted trail-price persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4.5},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":4.5}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":4.5,\"qty\":1,\"profit\":2.5},{\"id\":\"L2\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":4.5,\"qty\":3,\"profit\":4.5}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("trailing"));
    assert!(!output.contains("stop_price"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn runs_strategy_omitted_trail_points_persistent_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries.pine"
        ),
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries_bars.csv"
        ),
    )
    .expect("strategy omitted trail-points persistent fixture should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert_eq!(output.matches("\"direction\":\"strategy.exit\"").count(), 2);
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L1\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":1,\"price\":2},{\"id\":\"L2\",\"barIndex\":2,\"time\":3,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":1,\"price\":4},{\"id\":\"XT\",\"barIndex\":4,\"time\":5,\"direction\":\"strategy.exit\",\"qty\":3,\"price\":4}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L1\",\"entryBarIndex\":1,\"exitBarIndex\":4,\"entryTime\":2,\"exitTime\":5,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2},{\"id\":\"L2\",\"entryBarIndex\":2,\"exitBarIndex\":4,\"entryTime\":3,\"exitTime\":5,\"entryPrice\":3,\"exitPrice\":4,\"qty\":3,\"profit\":3}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":1,\"avgPrice\":2},{\"barIndex\":2,\"size\":4,\"avgPrice\":2.75},{\"barIndex\":4,\"size\":3,\"avgPrice\":3},{\"barIndex\":4,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(output.contains("\"values\":[0,1,2,2,2]"));
    assert!(output.contains("\"values\":[0,1,4,4,4]"));
    assert!(output.contains("\"strategy\":{\"orders\":"));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reservedQuantity"));
    assert!(!output.contains("reserved_quantity"));
    assert!(!output.contains("remainingQty"));
    assert!(!output.contains("remaining_quantity"));
    assert!(!output.contains("qtyPercent"));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("trailing"));
    assert!(!output.contains("activation"));
    assert!(!output.contains("exitReason"));
}

const REQUEST_HOST_SOURCE: &str =
    include_str!("../../../../tests/fixtures/request/request_security_host.pine");
const REQUEST_HOST_CHART_CSV: &str =
    include_str!("../../../../tests/fixtures/request/chart_1m.csv");
const REQUEST_HOST_BARS_JSON: &str = r#"{
  "NYSE:IBM:1": [
    {"time":0,"open":10,"high":11,"low":9,"close":20,"volume":100},
    {"time":60000,"open":11,"high":12,"low":10,"close":21,"volume":100},
    {"time":240000,"open":12,"high":13,"low":11,"close":22,"volume":100},
    {"time":300000,"open":13,"high":14,"low":12,"close":23,"volume":100},
    {"time":540000,"open":14,"high":15,"low":13,"close":24,"volume":100}
  ],
  "NYSE:IBM:5": [
    {"time":0,"open":90,"high":110,"low":80,"close":100,"volume":1000},
    {"time":300000,"open":190,"high":210,"low":180,"close":200,"volume":1000}
  ]
}"#;
const REQUEST_HOST_BARS_MISSING_HIGHER_JSON: &str = r#"{
  "NYSE:IBM:1": [
    {"time":0,"open":10,"high":11,"low":9,"close":20,"volume":100},
    {"time":60000,"open":11,"high":12,"low":10,"close":21,"volume":100},
    {"time":240000,"open":12,"high":13,"low":11,"close":22,"volume":100},
    {"time":300000,"open":13,"high":14,"low":12,"close":23,"volume":100},
    {"time":540000,"open":14,"high":15,"low":13,"close":24,"volume":100}
  ]
}"#;

#[test]
fn request_host_data_runs_through_direct_wasm_api() {
    let output = run_script_csv_with_request_bars(
        REQUEST_HOST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        REQUEST_HOST_BARS_JSON,
    )
    .expect("request fixture should run through direct WASM API");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert!(output.contains("\"values\":[30,32,34,36,38]"));
    assert!(output.contains("\"values\":[null,null,100,100,200]"));
}

#[test]
fn request_host_data_reports_missing_request_key() {
    let message = run_script_csv_with_request_bars_internal(
        REQUEST_HOST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        REQUEST_HOST_BARS_MISSING_HIGHER_JSON,
    )
    .expect_err("missing requested key should fail");

    assert!(
        message.contains("missing request data for symbol `NYSE:IBM` timeframe `5`"),
        "{message}"
    );
}

#[test]
fn run_csv_with_request_bars_matches_direct_request_api() {
    let direct_output = run_script_csv_with_request_bars(
        REQUEST_HOST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        REQUEST_HOST_BARS_JSON,
    )
    .expect("direct request fixture should run");
    let program = compile_script(REQUEST_HOST_SOURCE).expect("request fixture should compile");

    let compiled_output = program
        .run_csv_with_request_bars(REQUEST_HOST_CHART_CSV, REQUEST_HOST_BARS_JSON)
        .expect("compiled request fixture should run");
    let repeated_output = program
        .run_csv_with_request_bars(REQUEST_HOST_CHART_CSV, REQUEST_HOST_BARS_JSON)
        .expect("compiled request fixture should run again");

    assert_eq!(compiled_output, direct_output);
    assert_eq!(repeated_output, direct_output);
}

#[test]
fn run_csv_with_request_bars_reports_missing_request_key() {
    let program = compile_script(REQUEST_HOST_SOURCE).expect("request fixture should compile");
    let message = program
        .run_csv_with_request_bars_internal(
            REQUEST_HOST_CHART_CSV,
            REQUEST_HOST_BARS_MISSING_HIGHER_JSON,
        )
        .expect_err("missing requested key should fail");

    assert!(
        message.contains("missing request data for symbol `NYSE:IBM` timeframe `5`"),
        "{message}"
    );
}

const IMPORT_SOURCE: &str =
    "indicator(\"imports\")\nimport user/lib/1 as lib\nplot(lib.scale(close) + lib.offset)\n";
const IMPORT_REQUEST_SOURCE: &str = "indicator(\"import request\")\nimport user/lib/1 as lib\nsame = request.security(\"NYSE:IBM\", timeframe.period, open + close)\nhigher = request.security(\"NYSE:IBM\", \"5\", close)\nplot(lib.scale(same))\nplot(higher + lib.offset)\n";
const IMPORT_LIBRARY_JSON: &str = "{\"user/lib/1\":\"library(\\\"lib\\\")\\nexport offset = 2\\nexport scale(value) => value * offset\\n\"}";

fn import_fixture_library_json() -> String {
    serde_json::json!({
        "user/lib/1": include_str!("../../../../tests/fixtures/libraries/import_lib.pine"),
    })
    .to_string()
}

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
fn library_source_json_returns_import_fixture_contract() {
    let library_json = import_fixture_library_json();
    let output = run_script_csv_with_libraries(
        include_str!("../../../../tests/fixtures/runtime/import.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
        &library_json,
    )
    .expect("import fixture should run");

    assert_snapshot("runtime_import.json", &output);
}

#[test]
fn library_source_json_returns_import_state_fixture_contract() {
    let library_json = import_fixture_library_json();
    let output = run_script_csv_with_libraries(
        include_str!("../../../../tests/fixtures/runtime/import_state.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
        &library_json,
    )
    .expect("import state fixture should run");

    assert_snapshot("runtime_import_state.json", &output);
}

#[test]
fn library_source_json_combines_with_request_bars() {
    let output = run_script_csv_with_libraries_and_request_bars(
        IMPORT_REQUEST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        IMPORT_LIBRARY_JSON,
        REQUEST_HOST_BARS_JSON,
    )
    .expect("import plus request fixture should run");

    assert!(output.contains("\"values\":[60,64,68,72,76]"));
    assert!(output.contains("\"values\":[null,null,102,102,202]"));
}

#[test]
fn library_source_json_combined_api_reports_library_input_errors() {
    let message = run_script_csv_with_libraries_and_request_bars_internal(
        IMPORT_REQUEST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        "[]",
        REQUEST_HOST_BARS_JSON,
    )
    .expect_err("malformed library JSON should fail");

    assert!(message.contains("library sources must be a JSON object"));
}

#[test]
fn library_source_json_combined_api_reports_request_input_errors() {
    let message = run_script_csv_with_libraries_and_request_bars_internal(
        IMPORT_REQUEST_SOURCE,
        REQUEST_HOST_CHART_CSV,
        IMPORT_LIBRARY_JSON,
        "[]",
    )
    .expect_err("malformed request bars JSON should fail");

    assert!(message.contains("request bars must be a JSON object"));
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
