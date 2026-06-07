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
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("plotchar fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["plotChars"],
        serde_json::json!([
            {
                "id": 1,
                "values": [null, false, true],
                "chars": [null, "x", "x"],
                "colors": [null, 32768, 32768]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_plotshape_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotshape.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("plotshape fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["plotShapes"],
        serde_json::json!([
            {
                "id": 1,
                "values": [null, false, true],
                "styles": [null, "shape.triangleup", "shape.triangleup"],
                "locations": [null, "location.belowbar", "location.belowbar"],
                "colors": [null, 32768, 32768],
                "texts": [null, "Buy", "Buy"],
                "textColors": [null, 16777215, 16777215],
                "sizes": [null, "size.small", "size.small"]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_plotarrow_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotarrow.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("plotarrow fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["plotArrows"],
        serde_json::json!([
            {
                "id": 1,
                "values": [null, 0, 1],
                "colorUps": [null, 32768, 32768],
                "colorDowns": [null, 16711680, 16711680],
                "minHeights": [null, 5, 5],
                "maxHeights": [null, 20, 20]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_plotbar_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotbar.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("plotbar fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["plotBars"],
        serde_json::json!([
            {
                "id": 1,
                "opens": [null, 2, 3],
                "highs": [null, 2, 3],
                "lows": [null, 2, 3],
                "closes": [null, 2, 3],
                "colors": [null, 32768, 32768]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_plotcandle_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/plotcandle.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("plotcandle fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["plotCandles"],
        serde_json::json!([
            {
                "id": 1,
                "opens": [null, 2, 3],
                "highs": [null, 2, 3],
                "lows": [null, 2, 3],
                "closes": [null, 2, 3],
                "colors": [null, 32768, 32768],
                "wickColors": [null, 16777215, 16777215],
                "borderColors": [null, 16711680, 16711680]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_color_outputs_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/color_outputs.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("color outputs fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["bgColors"],
        serde_json::json!([
            {
                "id": 1,
                "values": [null, 32768, 32768]
            }
        ])
    );
    assert_eq!(
        parsed["barColors"],
        serde_json::json!([
            {
                "id": 2,
                "values": [null, null, 16711680]
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_hline_fill_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/io.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("hline/fill fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["plots"][0]["values"],
        serde_json::json!([null, 2.25, 3.75])
    );
    assert_eq!(
        parsed["hlines"],
        serde_json::json!([
            {
                "id": 10,
                "price": 2
            }
        ])
    );
    assert_eq!(
        parsed["fills"],
        serde_json::json!([
            {
                "id": 11,
                "firstId": 7,
                "secondId": 10
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_alertcondition_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alertcondition.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("alertcondition fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["alerts"],
        serde_json::json!([
            {
                "id": 3,
                "barIndex": 1,
                "time": 2,
                "message": "Branch alert",
                "source": "Branch"
            },
            {
                "id": 1,
                "barIndex": 2,
                "time": 3,
                "message": "Close is above two",
                "source": "Above two"
            },
            {
                "id": 1,
                "barIndex": 3,
                "time": 4,
                "message": "Close is above two",
                "source": "Above two"
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_alert_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/alert.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("alert fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["alerts"],
        serde_json::json!([
            {
                "id": 1,
                "barIndex": 0,
                "time": 1,
                "message": "Every bar",
                "source": "alert"
            },
            {
                "id": 1,
                "barIndex": 1,
                "time": 2,
                "message": "Every bar",
                "source": "alert"
            },
            {
                "id": 2,
                "barIndex": 1,
                "time": 2,
                "message": "Branch alert",
                "source": "alert"
            },
            {
                "id": 1,
                "barIndex": 2,
                "time": 3,
                "message": "Every bar",
                "source": "alert"
            },
            {
                "id": 3,
                "barIndex": 2,
                "time": 3,
                "message": "Loop alert",
                "source": "alert"
            },
            {
                "id": 1,
                "barIndex": 3,
                "time": 4,
                "message": "Every bar",
                "source": "alert"
            }
        ])
    );
}

#[test]
fn run_script_csv_returns_label_new_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/label_new.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("label.new fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"][0]["values"], serde_json::json!([1, 2, 3]));
    assert_eq!(
        parsed["labels"],
        serde_json::json!([
            {
                "id": 1,
                "snapshots": [
                    {
                        "barIndex": 0,
                        "exists": true,
                        "x": 0,
                        "y": 1,
                        "text": "start",
                        "xloc": "xloc.bar_index",
                        "yloc": "yloc.price",
                        "color": null,
                        "style": "label.style_label_down",
                        "textColor": null,
                        "size": "size.normal",
                        "tooltip": "",
                        "textAlign": "text.align_center",
                        "textFontFamily": "font.family_default",
                        "textFormatting": 0
                    }
                ]
            }
        ])
    );
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
fn run_script_csv_returns_math_edge_cases_as_json_null() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/math_edge_cases.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("math edge-case fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["plots"].as_array().expect("plots").len(), 8);
    for plot in parsed["plots"].as_array().expect("plots") {
        assert_eq!(plot["values"], serde_json::json!([null, null]));
    }
    assert!(!output.contains("NaN"));
    assert!(!output.contains("Infinity"));
}

#[test]
#[allow(clippy::approx_constant)]
fn run_script_csv_returns_math_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/math.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("math fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 28);
    let expected = [
        serde_json::json!([2, 1]),
        serde_json::json!([2, 1]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1.4142135623730951]),
        serde_json::json!([1, 1.2599210498948732]),
        serde_json::json!([0, 0.6931471805599453]),
        serde_json::json!([0, 0.3010299956639812]),
        serde_json::json!([2.718281828459045, 7.38905609893065]),
        serde_json::json!([3.141592653589793, 1.5707963267948966]),
        serde_json::json!([-1.5707963267948966, 0]),
        serde_json::json!([0.7853981633974483, 1.1071487177940904]),
        serde_json::json!([-1, 0]),
        serde_json::json!([57.29577951308232, 114.59155902616465]),
        serde_json::json!([0.017453292519943295, 0.03490658503988659]),
        serde_json::json!([8.095942459548628, 8.095942459548628]),
        serde_json::json!([0.8414709848078965, 0.9092974268256816]),
        serde_json::json!([0.5403023058681398, -0.4161468365471424]),
        serde_json::json!([1.5574077246549023, -2.185039863261519]),
        serde_json::json!([1, 4]),
        serde_json::json!([2.23606797749979, 3.605551275463989]),
        serde_json::json!([0.33, 0.67]),
        serde_json::json!([1.01, 2.01]),
        serde_json::json!([0.01, 0.01]),
        serde_json::json!([17.044006538018998, 14.290355590862742]),
        serde_json::json!([0.41627086372635447, 0.2277553881254537]),
        serde_json::json!([null, null]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_computed_lengths_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/computed_lengths.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("computed lengths fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 2);
    assert_eq!(plots[0]["values"], serde_json::json!([null, 1.5]));
    assert_eq!(plots[1]["values"], serde_json::json!([null, 3]));
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
fn run_script_csv_returns_udf_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/udf.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("UDF fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0]["values"], serde_json::json!([null, 4.5, 6.5]));
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
    assert_eq!(plots.len(), 3);
    assert_eq!(plots[0]["values"], serde_json::json!([2, 2, 3, 4]));
    assert_eq!(plots[1]["values"], serde_json::json!([2, 2, 3, 4]));
    assert_eq!(plots[2]["values"], serde_json::json!([2, 2, 4, 4]));
}

#[test]
fn run_script_csv_returns_ta_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/ta.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("TA fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 5);
    let ema_values = plots[0]["values"].as_array().expect("EMA values");
    let expected_ema = [1.0, 1.6666666666666665, 2.5555555555555554];
    for (actual, expected) in ema_values.iter().zip(expected_ema) {
        let actual = actual.as_f64().expect("numeric EMA value");
        assert!((actual - expected).abs() < 1e-12);
    }
    assert_eq!(plots[1]["values"], serde_json::json!([1, 1.5, 2.25]));
    assert_eq!(plots[2]["values"], serde_json::json!([null, 100, 100]));
    assert_eq!(plots[3]["values"], serde_json::json!([null, 1, 1]));
    assert_eq!(plots[4]["values"], serde_json::json!([0, 1, 1]));
}

#[test]
fn run_script_csv_returns_dema_tema_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/dema_tema.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("DEMA/TEMA fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 3);
    assert_eq!(plots[0]["values"], serde_json::json!([1, 1.75, 2.75]));
    assert_eq!(plots[1]["values"], serde_json::json!([1, 1.875, 2.9375]));
    assert_eq!(plots[2]["values"], serde_json::json!([null, null, null]));
}

#[test]
fn run_script_csv_returns_macd_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/macd.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("MACD fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 3);
    let expected = [
        [0.0, 0.16666666666666652, 0.30555555555555536],
        [0.0, 0.11111111111111101, 0.24074074074074056],
        [0.0, 0.05555555555555551, 0.0648148148148148],
    ];
    for (plot, expected_values) in plots.iter().zip(expected) {
        let values = plot["values"].as_array().expect("MACD plot values");
        for (actual, expected) in values.iter().zip(expected_values) {
            let actual = actual.as_f64().expect("numeric MACD value");
            assert!((actual - expected).abs() < 1e-12);
        }
    }
}

#[test]
fn run_script_csv_returns_strings_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strings.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strings fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 28);
    assert_eq!(plots[0]["values"], serde_json::json!([3, 3]));
    for plot in &plots[1..] {
        assert_eq!(plot["values"], serde_json::json!([1, 1]));
    }
}

#[test]
fn run_script_csv_returns_colors_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/colors.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("colors fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["bgColors"].as_array().expect("bgColors").len(), 1);
    assert_eq!(
        parsed["bgColors"][0]["values"],
        serde_json::json!([4288217216u64, 4288217216u64])
    );
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 5);
    let expected = [
        serde_json::json!([1, 2]),
        serde_json::json!([1, 1]),
        serde_json::json!([458, 458]),
        serde_json::json!([458, 458]),
        serde_json::json!([255, 192]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_syminfo_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/syminfo.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("syminfo fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 7);
    let expected = [
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([0.01, 0.01]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([100, 100]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_generic_input_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/generic_input.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("generic input fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0]["values"], serde_json::json!([null, 2.25]));
}

#[test]
fn run_script_csv_returns_timeframe_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/timeframe.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("timeframe fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 18);
    let expected = [
        serde_json::json!([1, 1]),
        serde_json::json!([60, 60]),
        serde_json::json!([60, 60]),
        serde_json::json!([1, 1]),
        serde_json::json!([45, 45]),
        serde_json::json!([3600, 3600]),
        serde_json::json!([86400, 86400]),
        serde_json::json!([1209600, 1209600]),
        serde_json::json!([7776000, 7776000]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 0]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_time_components_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/time_components.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("time components fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 21);
    let expected = [
        serde_json::json!([1970, 1970]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([5, 5]),
        serde_json::json!([0, 0]),
        serde_json::json!([0, 0]),
        serde_json::json!([0, 0]),
        serde_json::json!([2021, 2021]),
        serde_json::json!([2, 2]),
        serde_json::json!([5, 5]),
        serde_json::json!([2, 2]),
        serde_json::json!([3, 3]),
        serde_json::json!([3, 3]),
        serde_json::json!([4, 4]),
        serde_json::json!([5, 5]),
        serde_json::json!([0, 0]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_global_series_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/global_series.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("global series fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 12);
    let expected = [
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 1]),
        serde_json::json!([0, 1]),
        serde_json::json!([60000, 60001]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([1, 2]),
        serde_json::json!([0, 1]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_casts_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/casts.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("casts fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 10);
    let expected = [
        serde_json::json!([null, 1.5]),
        serde_json::json!([0, 1]),
        serde_json::json!([0, 0]),
        serde_json::json!([0, 1]),
        serde_json::json!([1, 0]),
        serde_json::json!([3, 1]),
        serde_json::json!([0, 0]),
        serde_json::json!([1, 1]),
        serde_json::json!([0, 0]),
        serde_json::json!([1, 1]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_barstate_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/barstate.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("barstate fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 6);
    let expected = [
        serde_json::json!([1, 0]),
        serde_json::json!([0, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([1, 1]),
        serde_json::json!([0, 0]),
    ];
    for (plot, values) in plots.iter().zip(expected) {
        assert_eq!(plot["values"], values);
    }
}

#[test]
fn run_script_csv_returns_session_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/session.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("session fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 3);
    assert_eq!(plots[0]["values"], serde_json::json!([1, 1]));
    assert_eq!(plots[1]["values"], serde_json::json!([0, 0]));
    assert_eq!(plots[2]["values"], serde_json::json!([0, 0]));
}

#[test]
fn run_script_csv_returns_inputs_fixture_contract() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/inputs.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("inputs fixture should run");

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");
    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    let plots = parsed["plots"].as_array().expect("plots");
    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0]["values"], serde_json::json!([0, 0]));
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
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_limit_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_limit_while_flat_noop.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit limit while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_profit_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit profit while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_exit_loss_while_flat_noop.pine"),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit loss while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit bracket while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_stop_profit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_stop_profit_bracket_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit stop profit bracket while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_limit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_limit_bracket_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit loss limit bracket while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_profit_bracket_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_profit_bracket_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit loss profit bracket while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_trailing_while_flat_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_trailing_while_flat_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n",
    )
    .expect("strategy exit trailing while-flat no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["orders"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["position"], serde_json::json!([]));
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_limit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_limit_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy limit wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_profit_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_profit_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy profit wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy loss wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_bracket_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy bracket wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_stop_profit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy stop profit bracket wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_limit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy loss limit bracket wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_loss_profit_bracket_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy loss profit bracket wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_exit_trailing_wrong_entry_from_csv_as_noop_json() {
    let output = run_script_csv(
        include_str!(
            "../../../../tests/fixtures/runtime/strategy_exit_trailing_unmatched_from_entry_noop.pine"
        ),
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy trailing wrong-entry no-op script should run");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("strict JSON output");

    assert_eq!(parsed["diagnostics"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["orders"],
        serde_json::json!([
            {
                "id": "L",
                "barIndex": 1,
                "time": 1,
                "direction": "strategy.long",
                "qty": 2,
                "price": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["trades"], serde_json::json!([]));
    assert_eq!(
        parsed["strategy"]["position"],
        serde_json::json!([
            {
                "barIndex": 1,
                "size": 2,
                "avgPrice": 2
            }
        ])
    );
    assert_eq!(parsed["strategy"]["diagnostics"], serde_json::json!([]));
    assert!(!output.contains("\"direction\":\"strategy.exit\""));
    assert!(!output.contains("pending"));
    assert!(!output.contains("reserved"));
}

#[test]
fn runs_strategy_entry_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(close)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy entry script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":2,\"time\":2,\"direction\":\"strategy.long\",\"qty\":2,\"price\":3}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":2,\"size\":2,\"avgPrice\":3}]"));
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
fn runs_strategy_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\", default_qty_type=strategy.fixed, default_qty_value=3)\nif bar_index == 1\n    strategy.entry(\"D\", strategy.long)\nplot(strategy.position_size)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy default quantity script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"D\",\"barIndex\":2,\"time\":2,\"direction\":\"strategy.long\",\"qty\":3,\"price\":3}]"
    ));
    assert!(output.contains("\"values\":[0,0,3]"));
}

#[test]
fn runs_strategy_percent_of_equity_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\", initial_capital=1000, default_qty_type=strategy.percent_of_equity, default_qty_value=25)\nif bar_index == 1\n    strategy.entry(\"D\", strategy.long)\nplot(strategy.position_size)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy percent default quantity script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"D\",\"barIndex\":2,\"time\":2,\"direction\":\"strategy.long\",\"qty\":125,\"price\":3}]"
    ));
    assert!(output.contains("\"values\":[0,0,125]"));
}

#[test]
fn runs_strategy_cash_default_quantity_from_csv_to_strategy_json() {
    let output = run_script_csv(
        "strategy(\"demo\", initial_capital=1000, default_qty_type=strategy.cash, default_qty_value=100)\nif bar_index == 1\n    strategy.entry(\"D\", strategy.long)\nplot(strategy.position_size)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy cash default quantity script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"D\",\"barIndex\":2,\"time\":2,\"direction\":\"strategy.long\",\"qty\":50,\"price\":3}]"
    ));
    assert!(output.contains("\"values\":[0,0,50]"));
}

#[test]
fn runs_strategy_position_state_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 2\n    strategy.close(\"L\")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nplot(strategy.max_contracts_held_all)\nplot(strategy.max_contracts_held_long)\nplot(strategy.max_contracts_held_short)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy position state script should run");

    assert!(output.contains("\"values\":[0,0,2]"));
    assert!(output.contains("\"values\":[null,null,3]"));
    assert!(output.contains("\"values\":[0,0,0]"));
    assert!(output.contains("\"values\":[null,null,null]"));
    assert!(output.contains("\"values\":[0,0,2]"));
}

#[test]
fn runs_strategy_profit_state_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\", initial_capital=1000)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\nplot(strategy.max_runup)\nplot(strategy.max_runup_percent)\nplot(strategy.max_drawdown)\nplot(strategy.max_drawdown_percent)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\nplot(strategy.max_runup)\nplot(strategy.max_runup_percent)\nplot(strategy.max_drawdown)\nplot(strategy.max_drawdown_percent)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy profit state script should run");

    assert!(output.contains("\"values\":[0,0,0]"));
    assert!(output.contains("\"values\":[0,0,0]"));
    assert!(output.contains("\"values\":[1000,1000,1000]"));
}

#[test]
fn runs_strategy_variable_interactions_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nscale(value) => value * 10\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nplot(strategy.position_size[1])\nplot(strategy.openprofit[1])\nplot(scale(strategy.position_size))\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy variable interaction script should run");

    assert!(output.contains("\"values\":[null,0,0]"));
    assert!(output.contains("\"values\":[null,0,0]"));
    assert!(output.contains("\"values\":[0,0,20]"));
}

#[test]
fn runs_strategy_trade_counts_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=1)\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 2\n    strategy.close(\"L\")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy trade count script should run");

    assert!(output.contains("\"values\":[0,0,1]"));
    assert!(output.contains("\"values\":[0,0,0]"));
    assert!(output.contains("\"strategy\":{\"orders\":["));
    assert!(output.contains("\"trades\":["));
    assert!(output.contains("\"position\":["));
    assert!(output.contains("\"equity\":["));
    assert!(output.contains("\"diagnostics\":[]}"));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
}

#[test]
fn runs_strategy_closedtrades_fields_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_closedtrades_fields.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy closed trade fields script should run");

    assert!(output.contains("\"values\":[null,null,2,2]"));
    assert!(output.contains("\"values\":[0,0,1,1]"));
    assert!(output.contains("\"values\":[null,null,3,3]"));
    assert!(output.contains("\"values\":[0,0,1,1]"));
    assert!(output.contains("\"values\":[null,null,0,0]"));
    assert!(output.contains("\"values\":[1,1,1,1]"));
    assert!(output.contains("\"values\":[1,1,1,1]"));
    assert!(output.contains("\"values\":[null,null,null,null]"));
    assert!(output.contains("\"entryTime\":2"));
    assert!(output.contains("\"exitTime\":3"));
    assert!(output.contains("\"profit\":2"));
    assert!(output.contains("\"qty\":2"));
    assert!(output.contains("\"trades\":["));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
}

#[test]
fn runs_strategy_opentrades_fields_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_opentrades_fields.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy open trade fields script should run");

    assert!(output.contains("\"values\":[null,2,null,null]"));
    assert!(output.contains("\"values\":[null,1,null,null]"));
    assert!(output.contains("\"values\":[null,null,null,null]"));
    assert!(output.contains("\"trades\":["));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
}

#[test]
fn runs_strategy_margin_capital_held_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_capital_held_long.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy margin capital held script should run");

    assert!(output.contains("\"values\":[0,2,3,0]"));
    assert!(output.contains("\"trades\":["));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
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

    assert!(output.contains("\"values\":[0,0,0,1]"));
    assert!(output.contains("\"values\":[0,0,0,4]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"covered-market\",\"barIndex\":3,\"time\":4,\"direction\":\"strategy.long\",\"qty\":1,\"price\":4}]"
    ));
    assert!(output.contains("\"position\":[{\"barIndex\":3,\"size\":1,\"avgPrice\":4}]"));
    assert!(output.contains("\"code\":\"E_STRATEGY_MARGIN\""));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
}

#[test]
fn runs_strategy_margin_call_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_call_long.pine"),
        include_str!("../../../../tests/fixtures/runtime/strategy_margin_call_long_bars.csv"),
    )
    .expect("strategy margin call script should run");

    assert!(output.contains("\"values\":[0,48,48]"));
    assert!(output.contains("\"values\":[0,36,36]"));
    assert!(output.contains("\"values\":[0,1,1]"));
    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.long\",\"qty\":100,\"price\":4},{\"id\":\"Margin Call\",\"barIndex\":1,\"time\":2,\"direction\":\"strategy.short\",\"qty\":52,\"price\":3}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":4,\"exitPrice\":3,\"qty\":52,\"profit\":-52}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":100,\"avgPrice\":4},{\"barIndex\":1,\"size\":48,\"avgPrice\":4}]"
    ));
    assert!(!output.contains("closedTrades"));
    assert!(!output.contains("openTrades"));
}

#[test]
fn runs_strategy_trade_outcome_counts_from_csv_to_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"W\", strategy.long, qty=1)\nif bar_index == 2\n    strategy.close(\"W\")\nif bar_index == 3\n    strategy.entry(\"L\", strategy.long, qty=1)\nif bar_index == 5\n    strategy.close(\"L\")\nif bar_index == 6\n    strategy.entry(\"E\", strategy.long, qty=1)\nif bar_index == 8\n    strategy.close(\"E\")\nplot(strategy.wintrades)\nplot(strategy.losstrades)\nplot(strategy.eventrades)\nplot(strategy.closedtrades)\nplot(strategy.grossprofit)\nplot(strategy.grossloss)\nplot(strategy.avg_trade)\nplot(strategy.avg_trade_percent)\nplot(strategy.avg_winning_trade)\nplot(strategy.avg_winning_trade_percent)\nplot(strategy.avg_losing_trade)\nplot(strategy.avg_losing_trade_percent)\n",
        "time,open,high,low,close,volume\n1,1,1,1,1,100\n2,2,2,2,2,100\n3,3,3,3,3,100\n4,4,4,4,4,100\n5,4,4,4,4,100\n6,2,2,2,2,100\n7,3,3,3,3,100\n8,5,5,5,5,100\n9,5,5,5,5,100\n",
    )
    .expect("strategy trade outcome count script should run");

    assert!(output.contains("\"values\":[0,0,1,1,1,1,1,1,1]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,1,1,1,1]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,0,0,0,1]"));
    assert!(output.contains("\"values\":[0,0,1,1,1,2,2,2,3]"));
    assert!(output.contains("\"values\":[0,0,1,1,1,1,1,1,1]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,2,2,2,2]"));
    assert!(output.contains("\"values\":[null,null,1,1,1,-0.5,-0.5,-0.5,-0.3333333333333333]"));
    assert!(output.contains("\"values\":[null,null,50,50,50,0,0,0,0]"));
    assert!(output.contains("\"values\":[null,null,1,1,1,1,1,1,1]"));
    assert!(output.contains("\"values\":[null,null,50,50,50,50,50,50,50]"));
    assert!(output.contains("\"values\":[null,null,null,null,null,2,2,2,2]"));
    assert!(output.contains("\"values\":[null,null,null,null,null,50,50,50,50]"));
    assert!(output.contains("\"profit\":1"));
    assert!(output.contains("\"profit\":-2"));
    assert!(output.contains("\"profit\":0"));
    assert!(!output.contains("winTrades"));
    assert!(!output.contains("lossTrades"));
    assert!(!output.contains("evenTrades"));
}

#[test]
fn runs_strategy_profit_percent_state_from_csv_to_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_percent_state.pine"),
        "time,open,high,low,close,volume\n1,1,1,1,1,100\n2,2,2,2,2,100\n3,3,3,3,3,100\n4,4,4,4,4,100\n5,4,4,4,4,100\n6,2,2,2,2,100\n7,3,3,3,3,100\n8,5,5,5,5,100\n9,5,5,5,5,100\n",
    )
    .expect("strategy profit percent state script should run");

    assert!(output.contains("\"values\":[0,0,1,1,1,-1,-1,-1,-1]"));
    assert!(output.contains("\"values\":[0,0,0.1,0.1,0.1,-0.1,-0.1,-0.1,-0.1]"));
    assert!(output.contains("\"values\":[0,0,0,0,0,0.2,0.2,0.2,0.2]"));
    assert!(!output.contains("profitPercent"));
}

#[test]
fn runs_strategy_close_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 1\n    strategy.entry(\"L\", strategy.long, qty=2)\nif bar_index == 2\n    strategy.close(\"L\")\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
    )
    .expect("strategy close script should run");

    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":2,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":2,\"entryPrice\":3,\"exitPrice\":3,\"qty\":2,\"profit\":0}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":2,\"size\":2,\"avgPrice\":3},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
}

#[test]
fn runs_strategy_close_qty_partial_from_csv_to_trade_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_close_qty_partial.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy close qty partial fixture should run");

    assert!(output.contains("\"values\":[0,2,1.25,1.25]"));
    assert!(output.contains("\"values\":[0,0,0.75,0.75]"));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":0.75,\"profit\":0.75}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":1.25,\"avgPrice\":2}]"
    ));
    assert!(!output.contains("pending"));
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

    assert!(output.contains("\"values\":[0,4,3,2]"));
    assert!(output.contains("\"values\":[0,0,1,3]"));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":2,\"exitTime\":3,\"entryPrice\":2,\"exitPrice\":3,\"qty\":1,\"profit\":1},{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":3,\"entryTime\":2,\"exitTime\":4,\"entryPrice\":2,\"exitPrice\":4,\"qty\":1,\"profit\":2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":4,\"avgPrice\":2},{\"barIndex\":2,\"size\":3,\"avgPrice\":2},{\"barIndex\":3,\"size\":2,\"avgPrice\":2}]"
    ));
    assert!(!output.contains("qty_percent"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_close_all_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.close_all()\n    strategy.entry(\"L\", strategy.long, qty=2)\nif bar_index == 2\n    strategy.close_all()\nif bar_index == 3\n    strategy.close_all()\nplot(strategy.position_size)\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\n",
        "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n3,4,4,4,4,1\n",
    )
    .expect("strategy close_all script should run");

    assert!(output.contains("\"values\":[0,2,0,0]"));
    assert!(output.contains("\"values\":[0,0,1,1]"));
    assert!(output.contains("\"values\":[0,1,0,0]"));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":2,\"entryTime\":1,\"exitTime\":2,\"entryPrice\":2,\"exitPrice\":3,\"qty\":2,\"profit\":2}]"
    ));
    assert!(output.contains(
        "\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2},{\"barIndex\":2,\"size\":0,\"avgPrice\":null}]"
    ));
    assert!(!output.contains("closeAll"));
    assert!(!output.contains("pending"));
}

#[test]
fn runs_strategy_exit_stop_from_csv_to_trade_json() {
    let output = run_script_csv(
        "strategy(\"demo\")\nif bar_index == 0\n    strategy.entry(\"L\", strategy.long, qty=2)\n    strategy.exit(\"XL\", \"L\", stop=9)\n",
        "time,open,high,low,close,volume\n10,10,10,10,10,1\n20,11,12,8,11,1\n",
    )
    .expect("strategy exit stop script should run");

    assert!(output.contains(
        "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.long\",\"qty\":2,\"price\":11},{\"id\":\"XL\",\"barIndex\":1,\"time\":20,\"direction\":\"strategy.exit\",\"qty\":2,\"price\":9}]"
    ));
    assert!(output.contains(
        "\"trades\":[{\"id\":\"L\",\"entryBarIndex\":1,\"exitBarIndex\":1,\"entryTime\":20,\"exitTime\":20,\"entryPrice\":11,\"exitPrice\":9,\"qty\":2,\"profit\":-4}]"
    ));
}

#[test]
fn runs_strategy_cancel_entry_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cancel_entry.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cancel entry script should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert!(output.contains("\"values\":[0,0,0,0]"));
    assert!(output.contains("\"values\":[null,null,null,null]"));
    assert!(output.contains("\"orders\":[]"));
    assert!(output.contains("\"trades\":[]"));
    assert!(output.contains("\"position\":[]"));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("cancel"));
}

#[test]
fn runs_strategy_cancel_all_entry_exit_fixture_from_csv_to_public_strategy_json() {
    let output = run_script_csv(
        include_str!("../../../../tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"),
        include_str!("../../../../tests/fixtures/runtime/bars.csv"),
    )
    .expect("strategy cancel all entry exit script should run");

    assert!(output.contains(&format!(
        "\"schemaVersion\":{}",
        PUBLIC_RUNTIME_SCHEMA_VERSION
    )));
    assert!(output.contains("\"values\":[0,0,0,0]"));
    assert!(output.contains("\"orders\":[]"));
    assert!(output.contains("\"trades\":[]"));
    assert!(output.contains("\"position\":[]"));
    assert!(output.contains("\"diagnostics\":[]"));
    assert!(!output.contains("pending"));
    assert!(!output.contains("cancel"));
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
