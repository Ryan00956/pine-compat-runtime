import pine_compat
import json
import math
from pathlib import Path


BARS = [
    {"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0},
    {"time": 1, "open": 2.0, "high": 2.0, "low": 2.0, "close": 2.0, "volume": 1.0},
    {"time": 2, "open": 3.0, "high": 3.0, "low": 3.0, "close": 3.0, "volume": 1.0},
]

RUNTIME_RESULT_KEYS = {
    "schemaVersion",
    "plots",
    "plotChars",
    "plotShapes",
    "plotArrows",
    "plotBars",
    "plotCandles",
    "bgColors",
    "barColors",
    "hlines",
    "fills",
    "labels",
    "lines",
    "lineFills",
    "polylines",
    "boxes",
    "tables",
    "alerts",
    "diagnostics",
}

STRATEGY_RUNTIME_RESULT_KEYS = RUNTIME_RESULT_KEYS | {"strategy"}

EMPTY_STRATEGY_RESULT = {
    "orders": [],
    "trades": [],
    "position": [],
    "equity": [],
    "alerts": [],
    "diagnostics": [],
}

FLAT_EQUITY = [
    {
        "barIndex": 0,
        "cash": 100000.0,
        "marketValue": 0.0,
        "equity": 100000.0,
        "netProfit": 0.0,
    },
    {
        "barIndex": 1,
        "cash": 100000.0,
        "marketValue": 0.0,
        "equity": 100000.0,
        "netProfit": 0.0,
    },
    {
        "barIndex": 2,
        "cash": 100000.0,
        "marketValue": 0.0,
        "equity": 100000.0,
        "netProfit": 0.0,
    },
]

ROOT = Path(__file__).resolve().parents[2]


def fixture_bars(path):
    rows = []
    lines = (ROOT / path).read_text().strip().splitlines()
    for line in lines[1:]:
        time, open_, high, low, close, volume = line.split(",")
        rows.append(
            {
                "time": int(time),
                "open": float(open_),
                "high": float(high),
                "low": float(low),
                "close": float(close),
                "volume": float(volume),
            }
        )
    return rows


def test_analyze_script_reports_executable_script():
    report = pine_compat.analyze_script('indicator("demo")\nplot(close)\n')

    assert report["schemaVersion"] == 3
    assert report["executable"] is True
    assert report["diagnostics"] == []
    assert report["inputs"] == []
    assert any(
        feature["feature"] == "plot"
        for feature in report["compatibility"]["supported"]
    )


def test_analyze_script_reports_input_call_sites():
    report = pine_compat.analyze_script(
        'indicator("inputs")\n'
        'length = input.int(2, "Length")\n'
        'mode = input.string("SMA", title="Mode")\n'
        'plot(close)\n'
    )

    assert report["schemaVersion"] == 3
    assert report["diagnostics"] == []
    assert [
        {"name": item["name"], "title": item["title"]}
        for item in report["inputs"]
    ] == [
        {"name": "input.int", "title": "Length"},
        {"name": "input.string", "title": "Mode"},
    ]
    assert all(isinstance(item["callSiteId"], int) for item in report["inputs"])


def test_program_run_accepts_call_site_keyed_input_overrides():
    source = (
        'indicator("input overrides")\n'
        'length = input.int(2, "Length")\n'
        'scale = input.float(1.0, "Scale")\n'
        'enabled = input.bool(true, "Enabled")\n'
        'mode = input.string("SMA", "Mode")\n'
        'shade = input.color(color.red, "Shade")\n'
        'base = enabled and mode == "SMA" ? ta.sma(close, length) * scale : open\n'
        'plot(base)\n'
        'plot(color.r(shade))\n'
    )
    report = pine_compat.analyze_script(source)
    input_ids = {
        item["title"]: item["callSiteId"]
        for item in report["inputs"]
    }
    program = pine_compat.compile_script(source)

    default = program.run(BARS)
    assert default["plots"][0]["values"] == [None, 1.5, 2.5]

    overrides = {
        input_ids["Length"]: 1,
        input_ids["Scale"]: 2.0,
        input_ids["Enabled"]: True,
        input_ids["Mode"]: "SMA",
        input_ids["Shade"]: 0x4CAF50,
    }
    result = program.run(BARS, input_overrides=overrides)
    assert result["plots"][0]["values"] == [2.0, 4.0, 6.0]
    assert result["plots"][1]["values"] == [76.0, 76.0, 76.0]

    script_result = pine_compat.run_script(source, BARS, input_overrides=overrides)
    assert script_result["plots"][0]["values"] == [2.0, 4.0, 6.0]


def test_program_run_accepts_generic_color_input_override_string():
    source = (
        'indicator("generic color")\n'
        'shade = input(color.red, "Shade")\n'
        'plot(color.r(shade))\n'
    )
    report = pine_compat.analyze_script(source)
    call_site_id = report["inputs"][0]["callSiteId"]
    program = pine_compat.compile_script(source)

    result = program.run(BARS, input_overrides={call_site_id: "#4CAF50"})

    assert result["plots"][0]["values"] == [76.0, 76.0, 76.0]


def test_program_run_rejects_unknown_input_override_call_site():
    program = pine_compat.compile_script(
        'indicator("inputs")\n'
        'length = input.int(2, "Length")\n'
        'plot(ta.sma(close, length))\n'
    )

    try:
        program.run(BARS, input_overrides={999999: 1})
    except ValueError as error:
        assert "unknown callSiteId 999999" in str(error)
    else:
        raise AssertionError("unknown input override callSiteId should fail")


def test_compile_script_returns_program_with_run_method():
    program = pine_compat.compile_script('indicator("demo")\nplot(close)\n')
    result = program.run(BARS)

    assert result["schemaVersion"] == 7
    assert set(result) == RUNTIME_RESULT_KEYS
    assert result["labels"] == []
    assert result["lines"] == []
    assert result["lineFills"] == []
    assert result["polylines"] == []
    assert result["boxes"] == []
    assert result["tables"] == []
    assert result["alerts"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["diagnostics"] == []


def test_run_script_rejects_non_finite_bar_values():
    bars = [
        {"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": math.nan, "volume": 1.0}
    ]

    try:
        pine_compat.run_script('indicator("demo")\nplot(close)\n', bars)
    except ValueError as error:
        assert "bar `close` value must be finite" in str(error)
    else:
        raise AssertionError("non-finite bar value should fail")


def test_run_script_rejects_duplicate_bar_times():
    bars = [
        {"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0},
        {"time": 0, "open": 2.0, "high": 2.0, "low": 2.0, "close": 2.0, "volume": 1.0},
    ]

    try:
        pine_compat.run_script('indicator("demo")\nplot(close)\n', bars)
    except ValueError as error:
        assert "duplicate bar time `0`" in str(error)
    else:
        raise AssertionError("duplicate bar time should fail")


def test_run_script_rejects_unsorted_bar_times():
    bars = [
        {"time": 1, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0},
        {"time": 0, "open": 2.0, "high": 2.0, "low": 2.0, "close": 2.0, "volume": 1.0},
    ]

    try:
        pine_compat.run_script('indicator("demo")\nplot(close)\n', bars)
    except ValueError as error:
        assert "bars are not sorted: `0` follows `1`" in str(error)
    else:
        raise AssertionError("unsorted bar time should fail")


def test_run_script_converts_non_finite_plot_values_to_none():
    result = pine_compat.run_script('indicator("demo")\nplot(1.0 / 0.0)\n', BARS)

    assert result["plots"][0]["values"] == [None, None, None]


def test_run_script_returns_plotchar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotchar.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_plotchar.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_plotshape_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotshape.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_plotshape.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_plotarrow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotarrow.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_plotarrow.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_plotbar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotbar.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_plotbar.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_plotcandle_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotcandle.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_plotcandle.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_color_outputs_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/color_outputs.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_color_outputs.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_chart_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/chart.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_chart.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_ticker_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/ticker.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_ticker.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_hline_fill_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/io.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_hline_fill.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_alertcondition_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/alertcondition.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_alertcondition.json").read_text())

    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert result == expected


def test_run_script_returns_alert_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/alert.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_alert.json").read_text())

    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert result == expected


def test_run_script_returns_label_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_new.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_new.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_mutation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_mutation.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_mutation.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_control_flow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_control_flow.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_label_control_flow.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_delete_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_delete.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_delete.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_copy_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_copy.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_copy.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_getters_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_getters.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_getters.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_options_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_options.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_options.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_xloc_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_xloc.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_xloc.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_yloc_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_yloc.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_yloc.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_label_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_scalar_typed_declarations_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/scalar_typed_declarations.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_scalar_typed_declarations.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_chart_point_typed_decl_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/chart_point_typed_decl.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_chart_point_typed_decl.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_typed_declarations_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_typed_declarations.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_typed_declarations.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_chart_point_array_typed_declarations_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/chart_point_array_typed_declarations.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_chart_point_array_typed_declarations.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_object_array_typed_declarations_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/object_array_typed_declarations.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_object_array_typed_declarations.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_drawing_typed_declarations_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/drawing_typed_declarations.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_drawing_typed_declarations.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_helpers_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_helpers.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_helpers.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_methods_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_methods.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_methods.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_computed_array_operands_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/computed_array_operands.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_computed_array_operands.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_type_alias_declarations_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/array_type_alias_declarations.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_type_alias_declarations.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_from_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_from.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_from.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_insert_remove_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_insert_remove.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_insert_remove.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_fill_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_fill.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_fill.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_clear_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_clear.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_clear.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_references_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_references.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_references.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_search_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_search.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_search.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_statistics_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_statistics.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_statistics.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_ordering_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_ordering.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_ordering.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_join_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_join.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_join.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_array_slice_concat_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_slice_concat.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_array_slice_concat.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_new.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_new.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_mutation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_mutation.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_mutation.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_control_flow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_control_flow.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_line_control_flow.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_getters_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_getters.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_getters.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_delete_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_delete.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_delete.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_copy_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_copy.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_copy.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_line_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/line_array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_line_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_array.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_linefill_array.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_new.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_linefill_new.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_set_color_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_set_color.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_linefill_set_color.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_getters_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_getters.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_linefill_getters.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_all_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_all.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_linefill_all.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linefill_delete_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linefill_delete.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_linefill_delete.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_polyline_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/polyline_new.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_polyline_new.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected
    assert result["polylines"][0]["snapshots"][0]["points"][0] == {
        "time": None,
        "index": 0,
        "price": 1.0,
    }


def test_run_script_returns_polyline_lifecycle_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/polyline_lifecycle.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_polyline_lifecycle.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected
    assert [snapshot["exists"] for snapshot in result["polylines"][0]["snapshots"]] == [
        True,
        False,
    ]
    assert result["plots"][0]["values"][-1] == 0


def test_run_script_returns_box_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_new.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_new.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_mutation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_mutation.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_mutation.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_control_flow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_control_flow.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_box_control_flow.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_getters_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_getters.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_getters.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_delete_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_delete.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_delete.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_copy_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_copy.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_copy.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_box_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/box_array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_box_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_new.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_table_new.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_cell_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_cell.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_table_cell.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_control_flow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_control_flow.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_table_control_flow.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_delete_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_delete.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_table_delete.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_clear_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_clear.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_table_clear.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_merge_cells_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_merge_cells.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_table_merge_cells.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_table_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/table_array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_table_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_drawing_methods_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/drawing_methods.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_drawing_methods.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_loop_state_interactions_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/loop_state_interactions.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_loop_state_interactions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_branch_loop_interactions_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/branch_loop_interactions.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_branch_loop_interactions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_switch_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/switch.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_switch.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_block_statements_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/block_statements.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_block_statements.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_edges_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_edges.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_for_edges.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_for_in.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_bool_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_bool.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_bool.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_color_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_color.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_color.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_float_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_float.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_float.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_mutation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_mutation.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_mutation.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_control_flow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_control_flow.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_control_flow.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_stateful_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_stateful.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_stateful.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_string_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_string.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_string.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_in_zero_iteration_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_in_zero_iteration.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_in_zero_iteration.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_for_stateful_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/for_stateful.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_for_stateful.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_while_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/while.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_while.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_while_edges_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/while_edges.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_while_edges.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_while_stateful_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/while_stateful.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_while_stateful.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_local_scope_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/local_scope.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_local_scope.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_history_edges_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/history_edges.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_history_edges.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_dynamic_history_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dynamic_history.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_dynamic_history.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_dynamic_history_scopes_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dynamic_history_scopes.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_dynamic_history_scopes.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_series_history_offset_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/series_history_offset.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_series_history_offset.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_max_bars_back_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/max_bars_back.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_max_bars_back.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_varip_scalar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/varip_scalar.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_varip_scalar.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_varip_local_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/varip_local.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_varip_local.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_varip_array_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/varip_array.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_varip_array.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_user_type_varip_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/user_type_varip.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_user_type_varip.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_request_security_same_context_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/request_security_same_context.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_request_security_same_context.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_user_types_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/user_types.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_user_types.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_user_type_functions_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/user_type_functions.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_user_type_functions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_user_methods_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/user_methods.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_user_methods.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_compile_script_reports_unsupported_user_type_field_fixture():
    source = (ROOT / "tests/fixtures/sema/unsupported_user_type.pine").read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        assert "E_UDT_FIELD_TYPE" in str(error)
    else:
        raise AssertionError("unsupported UDT field fixture should fail")


def test_compile_script_reports_unsupported_user_method_fixture():
    source = (ROOT / "tests/fixtures/sema/unsupported_user_method.pine").read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        assert "E_METHOD_RECEIVER_TYPE" in str(error)
    else:
        raise AssertionError("unsupported UDT method fixture should fail")


def test_compile_script_reports_unsupported_user_type_varip_fixture():
    source = (ROOT / "tests/fixtures/sema/unsupported_user_type_varip.pine").read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        message = str(error)
        assert "E_UNSUPPORTED_FEATURE" in message
        assert "`varip` is not supported" in message
        assert (
            "UDT varip supports only explicit scalar-tree declarations"
            in message
        )
    else:
        raise AssertionError("unsupported UDT varip fixture should fail")


def test_compile_script_reports_unsupported_user_type_field_mutation_fixture():
    source = (
        ROOT / "tests/fixtures/sema/unsupported_user_type_field_mutation.pine"
    ).read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        message = str(error)
        assert "E_UNSUPPORTED_FEATURE" in message
        assert "`function_side_effect` is not supported" in message
        assert "mutating fields on global user-defined type values" in message
    else:
        raise AssertionError("unsupported UDT field mutation fixture should fail")


def test_compile_script_reports_unsupported_user_method_side_effect_fixture():
    source = (
        ROOT / "tests/fixtures/sema/unsupported_user_method_side_effect.pine"
    ).read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        message = str(error)
        assert "E_UNSUPPORTED_FEATURE" in message
        assert "`function_side_effect` is not supported" in message
        assert "inside user-defined functions" in message
    else:
        raise AssertionError("unsupported UDT method side-effect fixture should fail")


def test_compile_script_reports_unsupported_non_array_method_fixture():
    source = (ROOT / "tests/fixtures/sema/unsupported_non_array_method.pine").read_text()

    try:
        pine_compat.compile_script(source)
    except ValueError as error:
        assert "E_METHOD_RECEIVER_TYPE" in str(error)
    else:
        raise AssertionError("unsupported non-array method fixture should fail")


def test_run_script_returns_import_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/import.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_import.json").read_text())
    library = (ROOT / "tests/fixtures/libraries/import_lib.pine").read_text()

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
        library_sources={"user/lib/1": library},
    )

    assert result == expected


def test_run_script_returns_import_state_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/import_state.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_import_state.json").read_text())
    library = (ROOT / "tests/fixtures/libraries/import_lib.pine").read_text()

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
        library_sources={"user/lib/1": library},
    )

    assert result == expected


def test_run_script_returns_math_edge_cases_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/math_edge_cases.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_math_edge_cases.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_math_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/math.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_math.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_computed_lengths_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/computed_lengths.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_computed_lengths.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_conditional_ta_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/conditional_ta.pine").read_text()
    bars = [
        {"time": 0, "open": 1.0, "high": 2.0, "low": 1.0, "close": 2.0, "volume": 1.0},
        {"time": 1, "open": 2.0, "high": 4.0, "low": 2.0, "close": 4.0, "volume": 1.0},
        {"time": 2, "open": 5.0, "high": 5.0, "low": 3.0, "close": 3.0, "volume": 1.0},
        {"time": 3, "open": 3.0, "high": 6.0, "low": 3.0, "close": 6.0, "volume": 1.0},
    ]
    result = pine_compat.run_script(source, bars)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 1
    assert result["plots"][0]["values"] == [None, 3.0, 3.0, 5.0]


def test_run_script_returns_conditional_ta_snapshot_contract():
    source = (ROOT / "tests/fixtures/runtime/conditional_ta.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_conditional_ta.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_udf_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/udf.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_udf.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_na_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/na.pine").read_text()
    bars = [
        {"time": 0, "open": 1.0, "high": 2.0, "low": 1.0, "close": 2.0, "volume": 1.0},
        {"time": 1, "open": 5.0, "high": 5.0, "low": 3.0, "close": 3.0, "volume": 1.0},
        {"time": 2, "open": 2.0, "high": 4.0, "low": 2.0, "close": 4.0, "volume": 1.0},
        {"time": 3, "open": 6.0, "high": 6.0, "low": 5.0, "close": 5.0, "volume": 1.0},
    ]
    result = pine_compat.run_script(source, bars)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 6
    assert result["plots"][0]["values"] == [2.0, 2.0, 3.0, 4.0]
    assert result["plots"][1]["values"] == [2.0, 2.0, 3.0, 4.0]
    assert result["plots"][2]["values"] == [2.0, 2.0, 4.0, 4.0]
    assert result["plots"][3]["values"] == [0.0, 0.0, 0.0, 0.0]
    assert result["plots"][4]["values"] == [0.0, 0.0, 0.0, 0.0]
    assert result["plots"][5]["values"] == [1, 0, 0, 0]


def test_run_script_returns_na_snapshot_contract():
    source = (ROOT / "tests/fixtures/runtime/na.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_na.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_ta_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/ta.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_ta.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_dema_tema_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dema_tema.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_dema_tema.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_macd_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/macd.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_macd.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strings_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strings.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_strings.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_compound_assignments_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/compound_assignments.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_compound_assignments.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_colors_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/colors.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_colors.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_syminfo_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/syminfo.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_syminfo.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_generic_input_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/generic_input.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_generic_input.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_timeframe_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/timeframe.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_timeframe.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_time_components_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/time_components.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_time_components.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_swma_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/swma.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_swma.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_stoch_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/stoch.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_stoch.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_wpr_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/wpr.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_wpr.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_atr_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/atr.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_atr.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_supertrend_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/supertrend.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_supertrend.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_dmi_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dmi.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_dmi.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_sar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/sar.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_sar.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_cross_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/cross.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_cross.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_mom_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/mom.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_mom.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_roc_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/roc.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_roc.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_trend_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/trend.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_trend.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_barssince_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/barssince.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_barssince.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_valuewhen_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/valuewhen.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_valuewhen.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_extremes_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/extremes.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_extremes.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_extreme_bars_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/extreme_bars.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_extreme_bars.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_tsi_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/tsi.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_tsi.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_cmo_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/cmo.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_cmo.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_cci_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/cci.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_cci.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_cog_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/cog.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_cog.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_ao_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/ao.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_ao.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_bop_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/bop.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_bop.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_bb_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/bb.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_bb.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_bbw_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/bbw.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_bbw.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_kc_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/kc.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_kc.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_kcw_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/kcw.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_kcw.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_pivots_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/pivots.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_pivots.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_pivot_point_levels_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/pivot_point_levels.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_pivot_point_levels.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_cum_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/cum.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_cum.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_all_time_extremes_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/all_time_extremes.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_all_time_extremes.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_alma_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/alma.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_alma.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_linreg_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/linreg.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_linreg.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_accdist_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/accdist.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_accdist.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_iii_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/iii.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_iii.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_nvi_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/nvi.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_nvi.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_obv_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/obv.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_obv.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_pvi_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/pvi.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_pvi.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_pvt_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/pvt.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_pvt.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_vwap_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/vwap.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_vwap.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_wad_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/wad.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_wad.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_wvad_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/wvad.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_wvad.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_correlation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/correlation.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_correlation.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_covariance_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/covariance.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_covariance.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_median_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/median.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_median.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_mode_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/mode.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_mode.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_percentile_linear_interpolation_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/percentile_linear_interpolation.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_percentile_linear_interpolation.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_percentile_nearest_rank_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/percentile_nearest_rank.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_percentile_nearest_rank.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_percentrank_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/percentrank.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_percentrank.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_stdev_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/stdev.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_stdev.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_variance_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/variance.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_variance.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_range_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/range.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_range.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_dev_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dev.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_dev.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_vwma_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/vwma.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_vwma.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_mfi_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/mfi.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_mfi.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_wma_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/wma.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_wma.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_hma_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/hma.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_hma.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_if_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/if.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_if.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_global_series_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/global_series.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_global_series.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_casts_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/casts.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_casts.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_barstate_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/barstate.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_barstate.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_session_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/session.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_session.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_inputs_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/inputs.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_inputs.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_compile_script_rejects_deep_input_without_aborting_process():
    expression = "(" * 300 + "close" + ")" * 300

    try:
        pine_compat.compile_script(f'indicator("deep")\nplot({expression})\n')
    except ValueError as error:
        assert "E_PARSE_EXPR_DEPTH" in str(error)
    else:
        raise AssertionError("deep input should fail with diagnostics")


def test_run_script_returns_empty_strategy_contract_for_strategy_mode():
    result = pine_compat.run_script(
        'strategy("demo")\nplot(close)\n',
        BARS,
    )

    assert set(result) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["strategy"] == {
        **EMPTY_STRATEGY_RESULT,
        "equity": FLAT_EQUITY,
    }
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]


def test_run_script_returns_strategy_entry_contract():
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=2)\nplot(close)\n',
        BARS,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 2,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 2.0, "avgPrice": 3.0}
    ]
    assert result["strategy"]["equity"] == [
        FLAT_EQUITY[0],
        {
            "barIndex": 1,
            "cash": 100000.0,
            "marketValue": 0.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
        {
            "barIndex": 2,
            "cash": 99994.0,
            "marketValue": 6.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
    ]


def test_run_script_returns_strategy_entry_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_entry.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_entry.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_entry_limit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_entry_limit.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 2.0],
        [None, 2.0, 2.0, 2.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert "pending" not in result["strategy"]
    assert "limit" not in result["strategy"]


def test_run_script_returns_strategy_entry_stop_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_entry_stop.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 2.0, 2.0],
        [None, None, 3.0, 3.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 2.0, "avgPrice": 3.0}
    ]
    assert "pending" not in result["strategy"]
    assert "stop" not in result["strategy"]


def test_run_script_returns_strategy_entry_stop_limit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_entry_stop_limit.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 0.0, 2.0],
        [None, None, None, 4.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 4.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 3, "size": 2.0, "avgPrice": 4.0}
    ]
    assert "pending" not in result["strategy"]
    assert "stop" not in result["strategy"]
    assert "limit" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_pyramiding.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"][:3]] == [
        [0, 1, 2, 2],
        [0.0, 1.0, 4.0, 4.0],
        [None, 2.0, 2.75, 2.75],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_close_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_pyramiding_close.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"][:4]] == [
        [0, 1, 1, 0],
        [0.0, 1.0, 3.0, 0.0],
        [None, 2.0, 3.0, None],
        [0, 0, 1, 2],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 1.0,
            "profit": 1.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 2,
            "exitBarIndex": 3,
            "entryTime": 3,
            "exitTime": 4,
            "entryPrice": 3.0,
            "exitPrice": 4.0,
            "qty": 3.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 2, "size": 3.0, "avgPrice": 3.0},
        {"barIndex": 3, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_close_all_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_pyramiding_close_all.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 0, 0],
        [0.0, 1.0, 0.0, 0.0],
        [0, 0, 2, 2],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 1.0,
            "profit": 1.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 2,
            "exitBarIndex": 2,
            "entryTime": 3,
            "exitTime": 3,
            "entryPrice": 3.0,
            "exitPrice": 3.0,
            "qty": 3.0,
            "profit": 0.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_exit_from_entry_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_pyramiding_exit_from_entry.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_pyramiding_exit_from_entry_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 2, 2, 1],
        [0.0, 1.0, 4.0, 4.0, 3.0],
        [0, 0, 0, 0, 1],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
        {
            "id": "XL1",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 1.0,
            "profit": 1.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 3, "size": 3.0, "avgPrice": 3.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_exit_profit_from_entry_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 2, 2, 1],
        [0.0, 1.0, 4.0, 4.0, 3.0],
        [0, 0, 0, 0, 1],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
        {
            "id": "XP1",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 3, "size": 3.0, "avgPrice": 3.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_exit_same_id_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_pyramiding_exit_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_pyramiding_exit_same_id_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 2, 2, 0],
        [0.0, 1.0, 4.0, 4.0, 0.0],
        [0, 0, 0, 0, 2],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 4.0,
        },
        {
            "id": "XL",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XL",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 5.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": 3.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 3,
            "entryTime": 3,
            "exitTime": 4,
            "entryPrice": 4.0,
            "exitPrice": 5.0,
            "qty": 3.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 3.5},
        {"barIndex": 3, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_exit_bracket_from_entry_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_pyramiding_exit_bracket_from_entry.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_profit_from_entry_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 2, 2, 1],
        [0.0, 1.0, 4.0, 4.0, 3.0],
        [0, 0, 0, 0, 1],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        },
        {
            "id": "XB1",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 3, "size": 3.0, "avgPrice": 3.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]


def test_run_script_returns_strategy_pyramiding_exit_trail_points_from_entry_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_trail_points_from_entry.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_trail_points_from_entry_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 1, 2, 2, 2, 1],
        [0.0, 1.0, 4.0, 4.0, 4.0, 3.0],
        [0, 0, 0, 0, 0, 1],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 2.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 4.0,
        },
        {
            "id": "XT1",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 3.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 1.0,
            "profit": 1.5,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 3.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 4.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "closedTrades" not in result["strategy"]
    assert "trailPrice" not in result["strategy"]
    assert "trailOffset" not in result["strategy"]


def test_run_script_returns_same_tick_limit_entries_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 2, 2],
        [0.0, 4.0, 4.0],
        [None, 9.0, 9.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 9.0,
        },
        {
            "id": "L2",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 9.0},
        {"barIndex": 1, "size": 4.0, "avgPrice": 9.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]


def test_run_script_returns_same_tick_limit_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_limit_same_tick_limit_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_limit_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_same_tick_stop_entries_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 2, 2],
        [0.0, 4.0, 4.0],
        [None, 11.0, 11.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 11.0,
        },
        {
            "id": "L2",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 11.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 11.0},
        {"barIndex": 1, "size": 4.0, "avgPrice": 11.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]


def test_run_script_returns_same_tick_stop_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_limit_same_tick_stop_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_same_tick_stop_limit_entries_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries_bars.csv"
        ),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 0, 2, 2],
        [0.0, 0.0, 4.0, 4.0],
        [None, None, 10.0, 10.0],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 10.0,
        },
        {
            "id": "L2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 10.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 1.0, "avgPrice": 10.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 10.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]


def test_run_script_returns_same_tick_stop_limit_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_limit_same_tick_stop_limit_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_limit_same_tick_stop_limit_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_default_quantity_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_default_quantity.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_default_quantity.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_default_quantity_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_default_quantity.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_default_quantity.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_builtin_default_quantity_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_builtin_default_quantity.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_builtin_default_quantity.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_default_quantity_override_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_default_quantity_override.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_default_quantity_override.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_percent_of_equity_default_quantity_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_percent_of_equity_default_quantity.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_percent_of_equity_default_quantity.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_percent_of_equity_default_quantity_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_percent_of_equity_default_quantity.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_percent_of_equity_default_quantity.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cash_default_quantity_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cash_default_quantity.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cash_default_quantity.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cash_default_quantity_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cash_default_quantity.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cash_default_quantity.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cash_default_quantity_limit_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cash_default_quantity_limit.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cash_default_quantity_limit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cash_default_quantity_override_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cash_default_quantity_override.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_cash_default_quantity_override.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_position_state_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_position_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_position_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_position_state_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_position_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_position_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_equity_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_equity.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_equity.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_profit_state_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_profit_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_profit_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_profit_state_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_profit_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_profit_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_profit_summary_plots():
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 0\n    strategy.entry("W", strategy.long, qty=1)\nif bar_index == 2\n    strategy.close("W")\nif bar_index == 3\n    strategy.entry("L", strategy.long, qty=1)\nif bar_index == 5\n    strategy.close("L")\nplot(strategy.netprofit)\nplot(strategy.grossprofit)\nplot(strategy.grossloss)\nplot(strategy.avg_trade)\nplot(strategy.avg_winning_trade)\nplot(strategy.avg_losing_trade)\n',
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0],
        [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 2.0],
        [None, None, 1.0, 1.0, 1.0, -0.5, -0.5, -0.5, -0.5],
        [None, None, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [None, None, None, None, None, 2.0, 2.0, 2.0, 2.0],
    ]


def test_run_script_returns_strategy_variable_interaction_plots():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_variable_interactions.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_variable_interactions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_variable_interactions_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_variable_interactions.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_variable_interactions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_trade_count_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_trade_counts.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_trade_counts.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_trade_count_fixture_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_trade_counts.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_trade_counts.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trade_count_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trade_counts.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trade_counts.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_closedtrades_field_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_closedtrades_fields.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_closedtrades_fields.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cash_per_contract_commission_plots():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_commission_cash_per_contract.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, 1.0, None, None],
        [None, None, 2.0, 2.0],
        [0.0, 0.0, 0.0, 0.0],
        [100000.0, 99999.0, 100000.0, 100000.0],
    ]
    assert result["strategy"]["trades"][0]["profit"] == 0.0
    assert result["strategy"]["equity"][1]["cash"] == 99995.0
    assert result["strategy"]["equity"][1]["equity"] == 99999.0


def test_run_script_returns_strategy_cash_per_order_commission_plots():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_commission_cash_per_order.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, 1.5, None, None],
        [None, None, 3.0, 3.0],
        [0.0, 0.0, -1.0, -1.0],
        [100000.0, 99998.5, 99999.0, 99999.0],
    ]
    assert result["strategy"]["trades"][0]["profit"] == -1.0
    assert result["strategy"]["equity"][1]["cash"] == 99994.5
    assert result["strategy"]["equity"][1]["equity"] == 99998.5


def test_run_script_returns_strategy_percent_commission_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_commission_percent.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, 0.4, None, None],
        [None, None, 1.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [100000.0, 99999.6, 100001.0, 100001.0],
    ]
    assert result["strategy"]["trades"][0]["profit"] == 1.0
    assert result["strategy"]["equity"][1]["cash"] == 99995.6
    assert result["strategy"]["equity"][1]["equity"] == 99999.6


def test_run_script_returns_strategy_slippage_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_slippage.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, None, 3.0, 3.0],
        [None, None, 2.0, 2.0],
        [None, None, -2.0, -2.0],
        [100000.0, 99998.0, 99998.0, 99998.0],
    ]
    assert result["strategy"]["orders"][0]["price"] == 3.0
    assert result["strategy"]["trades"][0]["entryPrice"] == 3.0
    assert result["strategy"]["trades"][0]["exitPrice"] == 2.0
    assert result["strategy"]["trades"][0]["profit"] == -2.0


def test_run_script_returns_strategy_exit_slippage_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_slippage.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, None, None, 2.0],
        [None, None, None, -2.0],
        [100000.0, 99998.0, 100000.0, 99998.0],
    ]
    assert result["strategy"]["orders"][0]["price"] == 3.0
    assert result["strategy"]["orders"][1]["price"] == 2.0
    assert result["strategy"]["trades"][0]["entryPrice"] == 3.0
    assert result["strategy"]["trades"][0]["exitPrice"] == 2.0
    assert result["strategy"]["trades"][0]["profit"] == -2.0


def test_run_script_returns_strategy_limit_verification_entry_plots():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_limit_verification_entry.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 0.0, 0.0],
        [0, 0, 0, 0],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == [
        {
            "barIndex": 0,
            "cash": 100000.0,
            "marketValue": 0.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
        {
            "barIndex": 1,
            "cash": 100000.0,
            "marketValue": 0.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
        {
            "barIndex": 2,
            "cash": 100000.0,
            "marketValue": 0.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
        {
            "barIndex": 3,
            "cash": 100000.0,
            "marketValue": 0.0,
            "equity": 100000.0,
            "netProfit": 0.0,
        },
    ]


def test_run_script_returns_strategy_limit_verification_exit_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_limit_verification_exit.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, None, None, None],
        [None, None, None, None],
    ]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XL",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"][0]["exitPrice"] == 3.0
    assert result["strategy"]["trades"][0]["profit"] == 2.0


def test_run_script_returns_strategy_opentrades_field_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_opentrades_fields.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_opentrades_fields.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_opentrades_fields_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_margin_capital_held_plot():
    source = (ROOT / "tests/fixtures/runtime/strategy_margin_capital_held_long.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_margin_capital_held_long.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_margin_entry_affordability_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_margin_entry_affordability_long.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_margin_entry_affordability.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_margin_call_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_margin_call_long.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_margin_call_long.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_margin_call_long_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_trade_outcome_count_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_trade_outcome_counts.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_trade_outcome_counts.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_profit_percent_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_profit_percent_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_profit_percent_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_profit_percent_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_profit_percent_state.pine").read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_profit_percent_state.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_strategy_close.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_strategy_close.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_qty_partial_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close_qty_partial.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_close_qty_partial.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_qty_full_clamp_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_close_qty_full_clamp.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_close_qty_full_clamp.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_qty_percent_precedence_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_close_qty_percent_precedence.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_close_qty_percent_precedence.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_all_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close_all.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_strategy_close_all.json").read_text())

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_exit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close_exit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_close_exit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_close_all_exit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close_all_exit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_close_all_exit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_entry_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_entry.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cancel_entry.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_exit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_exit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cancel_exit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_noop_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_noop.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cancel_noop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_all_entry_exit_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_cancel_all_entry_exit.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_all_exit_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_all_exit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cancel_all_exit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_cancel_all_noop_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_all_noop.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_cancel_all_noop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_stop_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_stop.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_stop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_stop_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_stop.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_stop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_limit_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_limit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_limit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_limit_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_limit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_limit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_profit_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_profit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_profit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_loss_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_loss.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_loss.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_loss_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_profit_loss_interactions_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_profit_loss_interactions.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_profit_loss_interactions.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_profit_loss_interactions_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_bracket_both_hit.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_bracket_both_hit.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_bracket_both_hit_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_creation_bar_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_creation_bar.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_bracket_creation_bar.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_interactions_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_interactions.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_bracket_interactions.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_interactions_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_interactions.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_interactions.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_invalid_leg_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_invalid_leg.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_bracket_invalid_leg.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_loss_profit_loss_fill_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_loss_profit_loss_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_loss_profit_loss_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_bracket_loss_profit_loss_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_loss_profit_profit_fill_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_loss_profit_profit_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_loss_profit_profit_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_mixed_pairs_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_mixed_pairs.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_bracket_mixed_pairs.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_bracket_mixed_pairs_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_repeated_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_repeated.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_bracket_repeated.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_replacement.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_bracket_replacement.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_bracket_replacement_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_omitted_bracket_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_omitted_bracket_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_omitted_bracket_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_state_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_state.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_bracket_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_stop_limit_limit_fill_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_limit_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_stop_limit_limit_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_bracket_stop_limit_stop_fill_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_stop_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_stop_limit_stop_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trail_price_fill.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trail_price_fill.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trail_points_fill_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_trail_points_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_trail_points_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_state_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trailing_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trailing_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_trailing_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_trailing_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_activation_bar_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_trailing_activation_bar.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_trailing_activation_bar.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_ratchet_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trailing_ratchet.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trailing_ratchet.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_repeated_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trailing_repeated.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trailing_repeated.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_invalid_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trailing_invalid.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_trailing_invalid.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_close_cancel_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_trailing_close_cancel.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_trailing_close_cancel.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_interactions_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_trailing_interactions.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_trailing_interactions.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_omitted_trailing_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_omitted_trailing_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_omitted_trailing_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_partial_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_qty_stop_partial.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_stop_partial.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_limit_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_limit_partial.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_limit_partial.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_bracket_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_bracket_partial.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_bracket_partial.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_clamp_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_qty_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_reservation_qty_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_stop_multi_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_qty_stop_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_reservation_qty_stop_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_limit_multi_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_qty_limit_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_reservation_qty_limit_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_qty_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_stop_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_stop_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_stop_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_mixed_stop_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_stop_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_mixed_stop_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_clamp_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_bracket_clamp_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_bracket_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_bracket_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_bracket_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_bracket_stop_limit_downside_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_stop_limit_downside_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_bracket_stop_limit_downside_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_bracket_stop_limit_upside_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_bracket_stop_limit_upside_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_bracket_stop_limit_upside_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_mixed_bracket_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_bracket_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_mixed_bracket_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_bracket_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_bracket_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_bracket_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_bracket_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_bracket_clamp_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_bracket_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_bracket_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_mixed_trailing_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_mixed_trailing_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_mixed_trailing_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_state_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_state.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_state.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_trailing_clamp_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_trailing_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_trailing_points_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_points_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_trailing_points_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_trailing_price_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_price_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_trailing_price_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_trailing_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_trailing_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_trailing_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_activation_mixed_fill_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_activation_mixed_fill.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_activation_mixed_fill.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_single_downside_order_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_single_downside_order.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_single_downside_order.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_bracket_downside_order_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_bracket_downside_order.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_bracket_downside_order.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_mixed_side_precedence_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_side_precedence.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_mixed_side_precedence.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_mixed_state_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_state.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_mixed_state.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_trailing_replacement_mixed_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_trailing_replacement_mixed.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_trailing_replacement_mixed.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_mixed_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_trailing_multi_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_multi.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_trailing_multi.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_trailing_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_trailing_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_qty_percent_trailing_clamp_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_qty_percent_trailing_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_qty_percent_trailing_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_reservation_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_trailing_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_trailing_partial.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_trailing_partial.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_full_clamp_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_full_clamp.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_full_clamp.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_repeated_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_qty_repeated.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_repeated.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_replacement.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_replacement.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_state_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_qty_state.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_precedence_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_qty_precedence_stop.pine").read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_precedence_stop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_precedence_bracket_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_precedence_bracket.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_precedence_bracket.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_precedence_trailing_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_precedence_trailing.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_precedence_trailing.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_qty_precedence_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_stop_partial.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_stop_partial.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_limit_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_limit_partial.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_limit_partial.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_bracket_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_bracket_partial.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_qty_percent_bracket_partial.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_trailing_partial_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_trailing_partial.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_qty_percent_trailing_partial.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_full_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_full.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_full.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_full_clamp_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_full_clamp.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_full_clamp.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_repeated_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_repeated.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_repeated.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_qty_percent_state_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_qty_percent_state.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_qty_percent_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_mixed_side_precedence.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_mixed_side_precedence.json"
        ).read_text()
    )
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_state_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_state.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_reservation_state.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_interactions_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_interactions.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_reservation_interactions.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_omitted_single_replacement_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_omitted_single_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_omitted_single_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_omitted_replaces_reservations_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_omitted_replaces_reservations.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_omitted_replaces_reservations.json"
        ).read_text()
    )
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_active_entry_attachment_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XL",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 2.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 2.0,
            "exitPrice": 2.5,
            "qty": 2.0,
            "profit": 1.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 1, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_attachment_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_attachment.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_active_entry_attachment.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_active_entry_profit_attachment_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_profit_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XP",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 2.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 2.5,
            "qty": 2.0,
            "profit": 1.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_loss_attachment_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_loss_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 3.0,
        },
        {
            "id": "XL",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 2.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 3.0,
            "exitPrice": 2.0,
            "qty": 2.0,
            "profit": -2.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 3.0},
        {"barIndex": 1, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_trail_points_attachment_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_active_entry_trail_points_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XT",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 3.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 2.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 3, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 2.0],
        [0.0, 0.0, 0.0, 0.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_stop_profit_bracket_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_stop_profit_bracket.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XB",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 3.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 2.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_loss_limit_bracket_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_loss_limit_bracket.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XB",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 3.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 2.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_loss_profit_bracket_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_loss_profit_bracket.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XB",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 3.5,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 2.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_bracket_reservation_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_bracket_host_parity.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        },
        {
            "id": "XB1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 0.5,
            "price": 2.0,
        },
        {
            "id": "XB2",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 3.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 2.0,
            "exitPrice": 2.0,
            "qty": 0.5,
            "profit": 0.0,
        },
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 1.0,
            "profit": 1.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 1, "size": 1.5, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.5, "avgPrice": 2.0},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 1.5, 0.5],
        [0.0, 0.0, 0.0, 1.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "reservedQuantity" not in result["strategy"]
    assert "reserved_quantity" not in result["strategy"]
    assert "remainingQty" not in result["strategy"]
    assert "remaining_quantity" not in result["strategy"]
    assert "qtyPercent" not in result["strategy"]
    assert "qty_percent" not in result["strategy"]
    assert "bracketLeg" not in result["strategy"]
    assert "bracket" not in result["strategy"]


def test_run_script_returns_strategy_exit_reservation_bracket_single_downside_precedence_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_bracket_single_downside_precedence.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_bracket_single_downside_precedence.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_bracket_single_replacement_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_bracket_single_replacement.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_bracket_single_replacement.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_bracket_single_upside_order_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_reservation_bracket_single_upside_order.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_bracket_single_upside_order.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_reservation_bracket_state_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_bracket_state.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_reservation_bracket_state.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_trailing_reservation_fixture_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_reservation_trailing_host_parity.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_exit_reservation_trailing_host_parity_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 7
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 3.0,
        },
        {
            "id": "XT1",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 0.75,
            "price": 3.5,
        },
        {
            "id": "XT2",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.25,
            "price": 3.3,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 3.0,
            "exitPrice": 3.5,
            "qty": 0.75,
            "profit": 0.375,
        },
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 3.3,
            "qty": 1.25,
            "profit": 0.3749999999999998,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 3.0},
        {"barIndex": 3, "size": 1.25, "avgPrice": 3.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 2.0, 2.0, 1.25],
        [0.0, 0.0, 0.0, 0.0, 0.375],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservedQuantity" not in strategy_json
    assert "reserved_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "remaining_quantity" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "trailing" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "activation" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_current_all_entry_exit_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_current.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_from_entry_current.json"
        ).read_text()
    )
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_current_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_persistent_all_entry_exit_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_persistent.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_from_entry_persistent.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_persistent_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_profit_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_profit_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_profit_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_profit_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_profit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_profit_bracket_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_profit_bracket_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_limit_bracket_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_limit_bracket_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_points_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_points_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_price_from_entries_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_price_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_profit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_limit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_limit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_points_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_points_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_price_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_price_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_profit_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_profit_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_profit_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_profit_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_profit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_profit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_profit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_profit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_limit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_limit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_price_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_points_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_loss_limit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_stop_limit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_price_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_omitted_trail_points_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries_bars.csv"
        ),
    )

    assert result == expected


def test_run_script_returns_strategy_runtime_diagnostics():
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 0\n    strategy.entry("L", strategy.long, qty=close-close)\n',
        BARS,
    )

    assert result["strategy"]["diagnostics"] == [
        {
            "code": "E_STRATEGY_QTY",
            "message": "`strategy.entry` quantity must be positive",
        }
    ]


def test_run_script_treats_strategy_exit_missing_entry_as_noop():
    result = pine_compat.run_script(
        'strategy("exit")\nif bar_index == 0\n    strategy.exit("XL", "L", stop=low)\n',
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []


def test_run_script_treats_strategy_exit_while_flat_fixture_as_noop():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (ROOT / "tests/snapshots/runtime_strategy_exit_while_flat_noop.json").read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_limit_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_limit_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_limit_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_profit_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_profit_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_profit_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_bracket_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_stop_profit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_stop_profit_bracket_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_stop_profit_bracket_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_limit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_limit_bracket_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_limit_bracket_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_profit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_profit_bracket_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_profit_bracket_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_trailing_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_trailing_while_flat_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_trailing_while_flat_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_returns_strategy_exit_wrong_entry_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT / "tests/snapshots/runtime_strategy_exit_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_limit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_limit_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_limit_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_profit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_profit_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_profit_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_bracket_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_stop_profit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_limit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_loss_profit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_run_script_treats_strategy_exit_trailing_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_trailing_unmatched_from_entry_noop.pine"
    ).read_text()
    expected = json.loads(
        (
            ROOT
            / "tests/snapshots/runtime_strategy_exit_trailing_unmatched_from_entry_noop.json"
        ).read_text()
    )

    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert result == expected


def test_analyze_script_accepts_library_sources_without_import_use():
    report = pine_compat.analyze_script(
        'indicator("root")\nplot(close)\n',
        library_sources={"user/lib/1": 'library("lib")\n'},
    )

    assert report["executable"] is True
    assert report["diagnostics"] == []


def test_compile_script_requires_import_alias_for_library_source():
    try:
        pine_compat.compile_script(
            'import user/lib/1\nindicator("root")\n',
            library_sources={"user/lib/1": 'library("lib")\n'},
        )
    except ValueError as error:
        assert "E_IMPORT_ALIAS_REQUIRED" in str(error)
    else:
        raise AssertionError("unaliased import should fail")


def test_run_script_accepts_imported_pure_function_subset():
    result = pine_compat.run_script(
        'indicator("root")\nimport user/lib/1 as lib\nplot(lib.scale(close) + lib.offset)\n',
        BARS,
        library_sources={
            "user/lib/1": 'library("lib")\nexport offset = 2\nexport scale(value) => value * offset\n'
        },
    )

    assert result["plots"][0]["values"] == [4.0, 6.0, 8.0]


def test_compile_script_rejects_invalid_library_source_key():
    try:
        pine_compat.compile_script(
            'indicator("root")\nplot(close)\n',
            library_sources={"user/lib 1": 'library("lib")\n'},
        )
    except ValueError as error:
        assert "invalid library source key `user/lib 1`" in str(error)
    else:
        raise AssertionError("invalid library key should fail")


def test_run_script_compiles_and_executes():
    result = pine_compat.run_script(
        'indicator("math")\nplot(math.max(close, 2))\n',
        BARS,
    )

    assert result["schemaVersion"] == 7
    assert result["plots"][0]["values"] == [2, 2, 3]


def test_run_script_accepts_library_sources_without_import_use():
    result = pine_compat.run_script(
        'indicator("root")\nplot(close)\n',
        BARS,
        library_sources={"user/lib/1": 'library("lib")\n'},
    )

    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]


def test_run_script_returns_alertcondition_events():
    result = pine_compat.run_script(
        'indicator("alerts")\nalertcondition(close > 1, "Above", "Close is above one")\n',
        BARS,
    )

    assert result["alerts"] == [
        {
            "id": 1,
            "barIndex": 1,
            "time": 1,
            "message": "Close is above one",
            "source": "Above",
        },
        {
            "id": 1,
            "barIndex": 2,
            "time": 2,
            "message": "Close is above one",
            "source": "Above",
        },
    ]


def test_run_script_returns_alert_events():
    result = pine_compat.run_script(
        'indicator("alerts")\nif bar_index == 1\n    alert("Reached")\n',
        BARS,
    )

    assert result["alerts"] == [
        {
            "id": 1,
            "barIndex": 1,
            "time": 1,
            "message": "Reached",
            "source": "alert",
        }
    ]


def test_run_script_returns_dynamic_alert_message_events():
    result = pine_compat.run_script(
        'indicator("alerts")\nalert(str.tostring(close))\n',
        BARS,
    )

    assert result["alerts"] == [
        {
            "id": 1,
            "barIndex": 0,
            "time": 0,
            "message": "1",
            "source": "alert",
        },
        {
            "id": 1,
            "barIndex": 1,
            "time": 1,
            "message": "2",
            "source": "alert",
        },
        {
            "id": 1,
            "barIndex": 2,
            "time": 2,
            "message": "3",
            "source": "alert",
        },
    ]


def test_run_script_returns_alert_frequency_events():
    source = (ROOT / "tests/fixtures/runtime/alert_frequency.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["alerts"] == [
        {
            "id": 1,
            "barIndex": 0,
            "time": 0,
            "message": "Default once",
            "source": "alert",
        },
        {
            "id": 2,
            "barIndex": 0,
            "time": 0,
            "message": "Explicit once",
            "source": "alert",
        },
        {
            "id": 3,
            "barIndex": 0,
            "time": 0,
            "message": "All",
            "source": "alert",
        },
        {
            "id": 3,
            "barIndex": 0,
            "time": 0,
            "message": "All",
            "source": "alert",
        },
        {
            "id": 4,
            "barIndex": 0,
            "time": 0,
            "message": "Close",
            "source": "alert",
        },
    ]


def test_render_strategy_order_fill_alert_template_replaces_message():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_metadata.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))
    alert = result["strategy"]["alerts"][1]

    rendered = pine_compat.render_strategy_order_fill_alert_template(
        "Order: {{strategy.order.alert_message}}", alert
    )

    assert rendered == "Order: loss alert"
    assert "renderedMessage" not in alert


def test_render_strategy_order_fill_alert_template_rejects_unknown_placeholder():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_metadata.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))
    alert = result["strategy"]["alerts"][1]

    try:
        pine_compat.render_strategy_order_fill_alert_template("{{close}}", alert)
    except ValueError as error:
        assert "unsupported strategy order-fill alert placeholder `{{close}}`" in str(error)
    else:
        raise AssertionError("unknown host placeholder should fail")


def test_render_strategy_order_fill_running_alert_replaces_message():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_metadata.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))
    alert = result["strategy"]["alerts"][1]
    config = {
        "scriptSnapshotId": "snapshot-1",
        "symbol": "TEST:SYMBOL",
        "timeframe": "1",
        "eventSelection": "strategyOrderFills",
        "messageTemplate": "Order: {{strategy.order.alert_message}}",
        "realtimePolicy": "realtimeOnly",
    }

    rendered = pine_compat.render_strategy_order_fill_running_alert(config, alert)

    assert rendered == "Order: loss alert"
    assert "renderedMessage" not in alert
    assert "renderedMessage" not in result["strategy"]["alerts"][1]


def test_render_strategy_order_fill_running_alert_keeps_both_design_only():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_metadata.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))
    alert = result["strategy"]["alerts"][1]
    config = {
        "scriptSnapshotId": "snapshot-1",
        "symbol": "TEST:SYMBOL",
        "timeframe": "1",
        "eventSelection": "both",
        "messageTemplate": "Order: {{strategy.order.alert_message}}",
        "realtimePolicy": "realtimeOnly",
    }

    try:
        pine_compat.render_strategy_order_fill_running_alert(config, alert)
    except ValueError as error:
        assert (
            "running alert event selection `both` cannot evaluate a strategy order-fill event"
            in str(error)
        )
    else:
        raise AssertionError("both selection should remain design-only for this helper")


def test_render_strategy_order_fill_running_alert_rejects_unknown_placeholder():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_metadata.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))
    alert = result["strategy"]["alerts"][1]
    config = {
        "scriptSnapshotId": "snapshot-1",
        "symbol": "TEST:SYMBOL",
        "timeframe": "1",
        "eventSelection": "strategyOrderFills",
        "messageTemplate": "{{close}}",
        "realtimePolicy": "realtimeOnly",
    }

    try:
        pine_compat.render_strategy_order_fill_running_alert(config, alert)
    except ValueError as error:
        assert "unsupported strategy order-fill alert placeholder `{{close}}`" in str(error)
    else:
        raise AssertionError("unknown host placeholder should fail")


def test_run_script_accepts_request_bars():
    result = pine_compat.run_script(
        'indicator("request")\nplot(request.security("NYSE:IBM", timeframe.period, close))\n',
        BARS,
        {
            "NYSE:IBM:1": [
                {
                    "time": 0,
                    "open": 10.0,
                    "high": 11.0,
                    "low": 9.0,
                    "close": 20.0,
                    "volume": 100.0,
                },
                {
                    "time": 1,
                    "open": 11.0,
                    "high": 12.0,
                    "low": 10.0,
                    "close": 21.0,
                    "volume": 100.0,
                },
                {
                    "time": 2,
                    "open": 12.0,
                    "high": 13.0,
                    "low": 11.0,
                    "close": 22.0,
                    "volume": 100.0,
                },
            ],
        },
    )

    assert result["plots"][0]["values"] == [20.0, 21.0, 22.0]


def test_run_script_request_fixture_matches_cli_contract():
    source = (ROOT / "tests/fixtures/request/request_security_host.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/request/chart_1m.csv"),
        {
            "NYSE:IBM:1": fixture_bars("tests/fixtures/request/ibm_1m.csv"),
            "NYSE:IBM:5": fixture_bars("tests/fixtures/request/ibm_5m.csv"),
        },
    )

    assert result["plots"][0]["values"] == [30.0, 32.0, 34.0, 36.0, 38.0]
    assert result["plots"][1]["values"] == [None, None, 100.0, 100.0, 200.0]
    assert result["plots"][2]["values"] == [10.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][3]["values"] == [34.0, 35.0, 36.0, 37.0, 38.0]
    assert result["plots"][4]["values"] == [None, 41.0, 43.0, 45.0, 47.0]
    assert result["plots"][5]["values"] == [20.01, 21.01, 22.01, 23.01, 24.01]
    assert result["plots"][6]["values"] == [None, 100.0, 100.0, 100.0, 100.0]
    assert result["plots"][7]["values"] == [2.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][8]["values"] == [None, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][9]["values"] == [
        2.0,
        4.666666666666667,
        6.4444444444444455,
        7.629629629629631,
        8.419753086419753,
    ]
    assert result["plots"][10]["values"] == [None, None, 13.0, 14.0, 15.0]
    assert result["plots"][11]["values"] == [None, None, 9.0, 10.0, 11.0]
    assert result["plots"][12]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][13]["values"] == [None, None, 2.0, 2.0, 2.0]
    assert result["plots"][14]["values"] == [
        None,
        None,
        10.0,
        9.523809523809524,
        9.090909090909092,
    ]
    assert result["plots"][15]["values"] == [None, None, 2.0, 2.0, 2.0]
    assert result["plots"][16]["values"] == [
        None,
        None,
        0.6666666666666666,
        0.6666666666666666,
        0.6666666666666666,
    ]
    assert result["plots"][17]["values"] == [0.0, 0.0, 1.0, 1.0, 1.0]
    assert result["plots"][18]["values"] == [0.0, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][19]["values"] == [0.0, 1.0, 0.0, 0.0, 0.0]
    assert result["plots"][20]["values"] == [0.0, 1.0, 0.0, 0.0, 0.0]
    assert result["plots"][21]["values"] == [0.0, 0.0, 1.0, 0.0, 0.0]
    assert result["plots"][22]["values"] == [20.0, 41.0, 63.0, 86.0, 110.0]
    assert result["plots"][23]["values"] == [
        None,
        None,
        0.816496580927726,
        0.816496580927726,
        0.816496580927726,
    ]
    assert result["plots"][24]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][25]["values"] == [
        None,
        None,
        0.6666666666666666,
        0.6666666666666666,
        0.6666666666666666,
    ]
    assert result["plots"][26]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][27]["values"] == [
        None,
        None,
        21.333333333333332,
        22.333333333333332,
        23.333333333333332,
    ]
    assert result["plots"][28]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][29]["values"] == [None, None, None, 21.5, 22.5]
    assert result["plots"][30]["values"] == [None, None, None, None, 24.0]
    assert result["plots"][31]["values"] == [
        None,
        None,
        None,
        22.462027683060324,
        23.462027683060324,
    ]
    assert result["plots"][32]["values"] == [None, None, 22.0, 23.0, 24.0]
    assert result["plots"][33]["values"] == [
        None,
        None,
        0.15552315827194782,
        0.1484539238050411,
        0.14199940537873496,
    ]
    assert result["plots"][34]["values"] == [
        None,
        None,
        0.9999999999999858,
        1.0000000000000284,
        1.0000000000000284,
    ]
    assert result["plots"][35]["values"] == [
        None,
        None,
        0.6666666666666572,
        0.6666666666666856,
        0.6666666666666856,
    ]
    assert result["plots"][36]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][37]["values"] == [None, None, 20.0, 21.0, 22.0]
    assert result["plots"][38]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][39]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][40]["values"] == [None, None, 100.0, 100.0, 100.0]
    assert result["plots"][41]["values"] == [
        20.0,
        20.333333333333332,
        20.88888888888889,
        21.59259259259259,
        22.395061728395063,
    ]
    assert result["plots"][42]["values"] == [20.0, 20.75, 21.75, 22.8125, 23.875]
    assert result["plots"][43]["values"] == [20.0, 20.875, 21.9375, 23.0, 24.03125]
    assert result["plots"][44]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][45]["values"] == [None, None, None, 100.0, 100.0]
    assert result["plots"][46]["values"] == [None, None, 100.0, 100.0, 100.0]
    assert result["plots"][47]["values"] == [None, None, 325.0, 325.0, 325.0]
    assert result["plots"][48]["values"] == [None, None, 225.0, 225.0, 225.0]
    assert result["plots"][49]["values"] == [None, 9.0, 9.0, 9.16, 9.4504]
    assert result["plots"][50]["values"] == [
        None,
        None,
        100.00000000000001,
        100.00000000000001,
        100.00000000000001,
    ]
    assert result["plots"][51]["values"] == [
        None,
        None,
        -1.9682539682539681,
        -1.9696969696969697,
        -1.9710144927536233,
    ]
    assert result["plots"][52]["values"] == [5.0, 5.0, 5.0, 5.0, 5.0]
    assert result["plots"][53]["values"] == [None, None, None, None, None]
    assert result["plots"][54]["values"] == [20.0, 21.0, 22.0, 23.0, 24.0]
    assert result["plots"][55]["values"] == [10.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][56]["values"] == [
        0.4,
        1.170731707317073,
        1.5058823529411764,
        1.6271186440677967,
        1.6476964769647697,
    ]
    assert result["plots"][57]["values"] == [None, None, None, None, None]
    assert result["plots"][58]["values"] == [None, None, None, None, None]
    assert result["plots"][59]["values"] == [0.0, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][60]["values"] == [None, None, 0.0, 0.0, 0.0]
    assert result["plots"][61]["values"] == [None, None, 2.0, 2.0, 2.0]
    assert result["plots"][62]["values"] == [None, None, None, 22.0, 23.0]
    assert result["plots"][63]["values"] == [20.0, 20.5, 21.0, 21.5, 22.0]
    assert result["plots"][64]["values"] == [1000.0, 2000.0, 3000.0, 4000.0, 5000.0]
    assert result["plots"][65]["values"] == [0.1, 0.1, 0.1, 0.1, 0.1]
    assert result["plots"][66]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][67]["values"] == [None, 100.0, 200.0, 300.0, 400.0]
    assert result["plots"][68]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][69]["values"] == [
        None,
        5.0,
        9.761904761904763,
        14.30735930735931,
        18.65518539431583,
    ]
    assert result["plots"][70]["values"] == [500.0, 500.0, 500.0, 500.0, 500.0]
    assert result["plots"][71]["values"] == [
        0.0,
        0.16666666666666785,
        0.30555555555555713,
        0.39351851851851904,
        0.4436728395061742,
    ]
    assert result["plots"][72]["values"] == [
        0.0,
        0.1111111111111119,
        0.24074074074074206,
        0.3425925925925934,
        0.40997942386831393,
    ]
    assert result["plots"][73]["values"] == [
        0.0,
        0.055555555555555955,
        0.06481481481481507,
        0.05092592592592565,
        0.03369341563786027,
    ]
    assert result["plots"][74]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][75]["values"] == [
        None,
        None,
        22.632993161855453,
        23.632993161855453,
        24.632993161855453,
    ]
    assert result["plots"][76]["values"] == [
        None,
        None,
        19.367006838144547,
        20.367006838144547,
        21.367006838144547,
    ]
    assert result["plots"][77]["values"] == [20.0, 20.5, 21.25, 22.125, 23.0625]
    assert result["plots"][78]["values"] == [24.0, 32.5, 37.25, 40.125, 42.0625]
    assert result["plots"][79]["values"] == [16.0, 8.5, 5.25, 4.125, 4.0625]
    assert result["plots"][80]["values"] == [14.0, 6.0, 6.0, 6.0, 6.0]
    assert result["plots"][81]["values"] == [1.0, -1.0, -1.0, -1.0, -1.0]
    assert result["plots"][82]["values"] == [
        0.0,
        7.1428571428571415,
        8.620689655172411,
        9.223300970873785,
        9.530791788856305,
    ]
    assert result["plots"][83]["values"] == [0.0, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][84]["values"] == [0.0, 50.0, 75.0, 87.5, 93.75]
    assert result["plots"][85]["values"] == [20.0, 20.5, 21.0, 21.5, 22.0]
    assert result["plots"][86]["values"] == [
        20.0,
        21.5,
        22.632993161855474,
        23.73606797749979,
        24.82842712474619,
    ]
    assert result["plots"][87]["values"] == [
        20.0,
        19.5,
        19.367006838144526,
        19.26393202250021,
        19.17157287525381,
    ]
    assert result["plots"][88]["values"] == [20.0, 21.0, 22.0, 23.0, 24.0]
    assert result["plots"][89]["values"] == [21.0, 22.0, 23.0, 24.0, 25.0]
    assert result["plots"][90]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][91]["values"] == [None, None, 100.0, 100.0, 200.0]
    assert result["plots"][92]["values"] == [None, None, 101.0, 101.0, 201.0]
    assert result["plots"][93]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][94]["values"] == [
        None,
        None,
        0.0,
        0.0,
        16.666666666666657,
    ]
    assert result["plots"][95]["values"] == [
        None,
        None,
        0.0,
        0.0,
        11.111111111111104,
    ]
    assert result["plots"][96]["values"] == [
        None,
        None,
        0.0,
        0.0,
        5.555555555555554,
    ]
    assert result["plots"][97]["values"] == [None, None, None, None, 150.0]
    assert result["plots"][98]["values"] == [None, None, None, None, 250.0]
    assert result["plots"][99]["values"] == [None, None, None, None, 50.0]
    assert result["plots"][100]["values"] == [
        None,
        None,
        100.0,
        100.0,
        166.66666666666666,
    ]
    assert result["plots"][101]["values"] == [
        None,
        None,
        160.0,
        160.0,
        333.3333333333333,
    ]
    assert result["plots"][102]["values"] == [None, None, 40.0, 40.0, 0.0]
    assert result["plots"][103]["values"] == [None, None, 100.0, 100.0, 150.0]
    assert result["plots"][104]["values"] == [None, None, 100.0, 100.0, 250.0]
    assert result["plots"][105]["values"] == [None, None, 100.0, 100.0, 50.0]
    assert result["plots"][106]["values"] == [
        None,
        None,
        155.0,
        155.0,
        81.66666666666667,
    ]
    assert result["plots"][107]["values"] == [None, None, 1.0, 1.0, -1.0]
    assert result["plots"][108]["values"] == [
        None,
        None,
        0.0,
        0.0,
        71.42857142857143,
    ]
    assert result["plots"][109]["values"] == [None, None, 0.0, 0.0, 0.0]
    assert result["plots"][110]["values"] == [None, None, 0.0, 0.0, 50.0]
    assert result["plots"][111]["values"] == [None, 20.0, 21.0, 22.0, 23.0]
    assert result["plots"][112]["values"] == [10.0, 20.0, 21.0, 22.0, 23.0]
    assert result["plots"][113]["values"] == [0.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][114]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][115]["values"] == [None, None, 90.0, 90.0, 100.0]
    assert result["plots"][116]["values"] == [None, None, 0.0, 0.0, 100.0]
    assert result["plots"][117]["values"] == [20.0, 21.0, 22.0, 23.0, 24.0]
    assert result["plots"][118]["values"] == [10.0, 11.0, 12.0, 13.0, 14.0]
    assert result["plots"][119]["values"] == [10.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][120]["values"] == [None, None, 100.0, 100.0, 200.0]
    assert result["plots"][121]["values"] == [None, None, 90.0, 90.0, 190.0]
    assert result["plots"][122]["values"] == [None, None, 10.0, 10.0, 10.0]
    assert result["plots"][125]["values"] == [None, 20.5, 21.5, 22.5, 23.5]
    assert result["plots"][126]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][127]["values"] == [20.0, 41.0, 63.0, 86.0, 110.0]
    assert result["plots"][128]["values"] == [None, None, None, None, 150.0]
    assert result["plots"][129]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][130]["values"] == [None, None, 100.0, 100.0, 300.0]
    assert result["plots"][131]["values"] == [0.0, 1.0, 0.0, 0.0, 0.0]
    assert result["plots"][132]["values"] == [0.0, 1.0, 0.0, 0.0, 0.0]
    assert result["plots"][133]["values"] == [0.0, 0.0, 1.0, 0.0, 0.0]
    assert result["plots"][134]["values"] == [None, None, 0.0, 0.0, 1.0]
    assert result["plots"][135]["values"] == [None, None, 0.0, 0.0, 1.0]
    assert result["plots"][136]["values"] == [None, None, 0.0, 0.0, 1.0]
    assert result["plots"][137]["values"] == [0.0, 0.0, 1.0, 1.0, 1.0]
    assert result["plots"][138]["values"] == [0.0, 0.0, 1.0, 1.0, 1.0]
    assert result["plots"][139]["values"] == [0.0, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][140]["values"] == [None, None, 0.0, 0.0, 1.0]
    assert result["plots"][141]["values"] == [None, None, 0.0, 0.0, 1.0]
    assert result["plots"][142]["values"] == [None, None, 0.0, 0.0, 0.0]
    assert result["plots"][143]["values"] == [0.0, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][144]["values"] == [None, None, None, 22.0, 23.0]
    assert result["plots"][145]["values"] == [None, None, 0.0, 0.0, 0.0]
    assert result["plots"][146]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][147]["values"] == [None, None, 0.0, 0.0, 0.0]
    assert result["plots"][148]["values"] == [None, None, 2.0, 2.0, 2.0]
    assert result["plots"][149]["values"] == [None, None, None, None, 0.0]
    assert result["plots"][150]["values"] == [None, None, None, None, 1.0]
    assert result["plots"][151]["values"] == [None, None, None, 0.0, None]
    assert result["plots"][152]["values"] == [None, None, None, 0.0, None]
    assert result["plots"][153]["values"] == [None, None, None, None, 200.0]
    assert result["plots"][154]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][155]["values"] == [
        None,
        None,
        0.9999999999999858,
        1.0000000000000284,
        1.0000000000000284,
    ]
    assert result["plots"][156]["values"] == [
        None,
        None,
        0.6666666666666572,
        0.6666666666666856,
        0.6666666666666856,
    ]
    assert result["plots"][157]["values"] == [None, None, None, None, 1.0]
    assert result["plots"][158]["values"] == [None, None, None, None, 2500.0]
    assert result["plots"][159]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][160]["values"] == [None, None, 20.0, 21.0, 22.0]
    assert result["plots"][161]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][162]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][163]["values"] == [None, None, 100.0, 100.0, 100.0]
    assert result["plots"][164]["values"] == [
        None,
        None,
        33.33333333333333,
        33.33333333333333,
        33.33333333333333,
    ]
    assert result["plots"][165]["values"] == [None, None, None, None, 150.0]
    assert result["plots"][166]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][167]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][168]["values"] == [None, None, None, None, 150.0]
    assert result["plots"][169]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][170]["values"] == [None, None, None, None, 50.0]
    assert result["plots"][171]["values"] == [
        None,
        None,
        0.816496580927726,
        0.816496580927726,
        0.816496580927726,
    ]
    assert result["plots"][172]["values"] == [
        None,
        None,
        0.6666666666666666,
        0.6666666666666666,
        0.6666666666666666,
    ]
    assert result["plots"][173]["values"] == [None, None, None, None, 50.0]
    assert result["plots"][174]["values"] == [None, None, None, None, 2500.0]
    assert result["plots"][175]["values"] == [
        None,
        None,
        21.333333333333332,
        22.333333333333332,
        23.333333333333332,
    ]
    assert result["plots"][176]["values"] == [None, None, 21.0, 22.0, 23.0]
    assert result["plots"][177]["values"] == [
        None,
        None,
        None,
        None,
        166.66666666666666,
    ]
    assert result["plots"][178]["values"] == [None, None, None, None, 150.0]
    assert result["plots"][179]["values"] == [None, None, None, 21.5, 22.5]
    assert result["plots"][180]["values"] == [None, None, None, None, 24.0]
    assert result["plots"][181]["values"] == [
        None,
        None,
        None,
        22.462027683060324,
        23.462027683060324,
    ]
    assert result["plots"][182]["values"] == [None, None, 22.0, 23.0, 24.0]
    assert result["plots"][183]["values"] == [
        20.0,
        20.333333333333332,
        20.88888888888889,
        21.59259259259259,
        22.395061728395063,
    ]
    assert result["plots"][184]["values"] == [20.0, 20.75, 21.75, 22.8125, 23.875]
    assert result["plots"][185]["values"] == [20.0, 20.875, 21.9375, 23.0, 24.03125]
    assert result["plots"][186]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][187]["values"] == [None, None, None, 100.0, 100.0]
    assert result["plots"][188]["values"] == [None, None, 100.0, 100.0, 100.0]
    assert result["plots"][189]["values"] == [None, None, 325.0, 325.0, 325.0]
    assert result["plots"][190]["values"] == [None, None, 225.0, 225.0, 225.0]
    assert result["plots"][191]["values"] == [None, 9.0, 9.0, 9.16, 9.4504]
    assert result["plots"][192]["values"] == [
        None,
        None,
        100.00000000000001,
        100.00000000000001,
        100.00000000000001,
    ]
    assert result["plots"][193]["values"] == [
        None,
        None,
        -1.9682539682539681,
        -1.9696969696969697,
        -1.9710144927536233,
    ]
    assert result["plots"][194]["values"] == [5.0, 5.0, 5.0, 5.0, 5.0]
    assert result["plots"][195]["values"] == [20.0, 21.0, 22.0, 23.0, 24.0]
    assert result["plots"][196]["values"] == [10.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][197]["values"] == [
        0.4,
        1.170731707317073,
        1.5058823529411764,
        1.6271186440677967,
        1.6476964769647697,
    ]
    assert result["plots"][198]["values"] == [20.0, 20.5, 21.0, 21.5, 22.0]
    assert result["plots"][199]["values"] == [
        1000.0,
        2000.0,
        3000.0,
        4000.0,
        5000.0,
    ]
    assert result["plots"][200]["values"] == [0.1, 0.1, 0.1, 0.1, 0.1]
    assert result["plots"][201]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][202]["values"] == [None, 100.0, 200.0, 300.0, 400.0]
    assert result["plots"][203]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][204]["values"] == [
        None,
        5.0,
        9.761904761904763,
        14.30735930735931,
        18.65518539431583,
    ]
    assert result["plots"][205]["values"] == [500.0, 500.0, 500.0, 500.0, 500.0]
    assert result["plots"][206]["values"] == [None, None, None, None, None]
    assert result["plots"][207]["values"] == [None, None, 100.0, 100.0, 200.0]
    assert result["plots"][208]["values"] == [None, None, 90.0, 90.0, 90.0]
    assert result["plots"][209]["values"] == [
        None,
        None,
        333.3333333333333,
        333.3333333333333,
        666.6666666666666,
    ]
    assert result["plots"][210]["values"] == [
        None,
        None,
        0.0003333333333333333,
        0.0003333333333333333,
        0.0003333333333333333,
    ]
    assert result["plots"][211]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][212]["values"] == [None, None, None, None, 1000.0]
    assert result["plots"][213]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][214]["values"] == [None, None, None, None, 1000.0]
    assert result["plots"][215]["values"] == [
        None,
        None,
        333.3333333333333,
        333.3333333333333,
        333.3333333333333,
    ]
    assert result["plots"][216]["values"] == [None, None, None, None, None]
    assert result["plots"][217]["values"] == [None, None, None, None, None]
    assert result["plots"][218]["values"] == [None, None, None, None, None]
    assert result["plots"][219]["values"] == [None, None, None, None, None]
    assert result["plots"][220]["values"] == [None, None, None, None, None]
    assert result["plots"][221]["values"] == [None, None, 100.0, 100.0, 133.33333333333334]
    assert result["plots"][222]["values"] == [None, None, 100.0, 100.0, 175.0]
    assert result["plots"][223]["values"] == [None, None, 100.0, 100.0, 187.5]
    assert result["plots"][224]["values"] == [None, None, None, None, 1.0]
    assert result["plots"][225]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][226]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][227]["values"] == [
        None,
        None,
        None,
        None,
        92.3076923076923,
    ]
    assert result["plots"][228]["values"] == [
        None,
        None,
        None,
        None,
        -7.6923076923076925,
    ]
    assert result["plots"][229]["values"] == [None, None, None, None, 80.0]
    assert result["plots"][230]["values"] == [
        None,
        None,
        None,
        None,
        66.66666666666667,
    ]
    assert result["plots"][231]["values"] == [
        None,
        None,
        None,
        None,
        -1.3333333333333333,
    ]
    assert result["plots"][232]["values"] == [
        None,
        None,
        0.3333333333333333,
        0.3333333333333333,
        0.3333333333333333,
    ]
    assert result["plots"][233]["values"] == [None, None, 1.2, 1.2, 2.0]
    assert result["plots"][234]["values"] == [None, None, 100.0, 100.0, 150.0]
    assert result["plots"][235]["values"] == [None, None, 30.0, 30.0, 110.0]
    assert result["plots"][236]["values"] == [None, None, 30.0, 30.0, 70.0]
    assert result["plots"][237]["values"] == [None, None, None, None, 210.0]
    assert result["plots"][238]["values"] == [None, None, None, None, 80.0]
    assert result["plots"][239]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][240]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][241]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][242]["values"] == [None, None, None, None, 50.0]
    assert result["plots"][243]["values"] == [
        None,
        None,
        100.0,
        100.0,
        166.66666666666666,
    ]
    assert result["plots"][244]["values"] == [None, None, None, None, 100.0]
    assert result["plots"][245]["values"] == [
        None,
        None,
        None,
        None,
        1.3333333333333333,
    ]
    assert result["plots"][246]["values"] == [None, None, None, None, 210.0]
    assert result["plots"][247]["values"] == [None, None, None, None, 80.0]
    assert result["plots"][248]["values"] == [None, None, None, None, 0.0]
    assert result["plots"][249]["values"] == [None, None, None, None, 1.0]
    assert result["plots"][123]["values"] == [None, None, None, None, 300.0]
    assert result["plots"][124]["values"] == [None, None, 100.01, 100.01, 200.01]
    assert result["plots"][250]["values"] == [None, None, 33.0, 33.0, 66.0]
    assert result["plots"][251]["values"] == [None, None, 2.0, 2.0, 3.0]
    assert result["plots"][252]["values"] == [None, None, 33.33, 33.33, 66.67]
    assert result["plots"][253]["values"] == [
        None,
        None,
        10.0,
        10.0,
        14.142135623730951,
    ]
    assert result["plots"][254]["values"] == [
        None,
        None,
        4.641588833612779,
        4.641588833612779,
        5.848035476425732,
    ]
    assert result["plots"][255]["values"] == [None, None, 2.0, 2.0, 2.3010299956639813]
    assert result["plots"][256]["values"] == [
        None,
        None,
        0.8414709848078965,
        0.8414709848078965,
        0.9092974268256817,
    ]
    assert result["plots"][257]["values"] == [
        None,
        None,
        0.6216099682706644,
        0.6216099682706644,
        -0.32328956686350335,
    ]
    assert result["plots"][258]["values"] == [
        None,
        None,
        0.10033467208545055,
        0.10033467208545055,
        0.10033467208545055,
    ]
    assert result["plots"][259]["values"] == [None, None, 1.0, 1.0, 4.0]
    assert result["plots"][260]["values"] == [
        None,
        None,
        1.3453624047073711,
        1.3453624047073711,
        2.7586228448267445,
    ]
    assert result["plots"][261]["values"] == [
        None,
        None,
        4.605170185988092,
        4.605170185988092,
        5.298317366548036,
    ]
    assert result["plots"][262]["values"] == [
        None,
        None,
        2.718281828459045,
        2.718281828459045,
        7.38905609893065,
    ]
    assert result["plots"][263]["values"] == [
        None,
        None,
        1.0471975511965979,
        1.0471975511965979,
        0.0,
    ]
    assert result["plots"][264]["values"] == [
        None,
        None,
        0.5235987755982989,
        0.5235987755982989,
        1.5707963267948966,
    ]
    assert result["plots"][265]["values"] == [
        None,
        None,
        0.7853981633974483,
        0.7853981633974483,
        1.1071487177940904,
    ]
    assert result["plots"][266]["values"] == [None, None, 95.0, 95.0, 195.0]
    assert result["plots"][267]["values"] == [None, None, 33.0, 33.0, 66.0]
    assert result["plots"][268]["values"] == [None, None, 1.0, 1.0, 1.0]
    assert result["plots"][269]["values"] == [
        None,
        None,
        57.29577951308232,
        57.29577951308232,
        114.59155902616465,
    ]
    assert result["plots"][270]["values"] == [
        None,
        None,
        0.15707963267948966,
        0.15707963267948966,
        0.33161255787892263,
    ]
    assert result["plots"][271]["values"] == [6.0, 7.0, 7.0, 7.0, 8.0]
    assert result["plots"][272]["values"] == [2.0, 2.0, 2.0, 3.0, 3.0]
    assert result["plots"][273]["values"] == [2.86, 3.0, 3.14, 3.29, 3.43]
    assert result["plots"][274]["values"] == [
        4.47213595499958,
        4.58257569495584,
        4.69041575982343,
        4.795831523312719,
        4.898979485566356,
    ]
    assert result["plots"][275]["values"] == [
        2.7144176165949068,
        2.7589241763811208,
        2.8020393306553872,
        2.8438669798515654,
        2.8844991406148166,
    ]
    assert result["plots"][276]["values"] == [
        1.3010299956639813,
        1.3222192947339193,
        1.3424226808222062,
        1.3617278360175928,
        1.380211241711606,
    ]
    assert result["plots"][277]["values"] == [
        0.19866933079506122,
        0.20845989984609956,
        0.21822962308086932,
        0.2279775235351884,
        0.23770262642713458,
    ]
    assert result["plots"][278]["values"] == [
        0.9950041652780258,
        0.9939560979566968,
        0.9928086358538663,
        0.9915618937147881,
        0.9902159962126371,
    ]
    assert result["plots"][279]["values"] == [
        0.10033467208545055,
        0.10033467208545055,
        0.10033467208545055,
        0.10033467208545055,
        0.10033467208545055,
    ]
    assert result["plots"][280]["values"] == [
        0.04000000000000001,
        0.04409999999999999,
        0.0484,
        0.0529,
        0.0576,
    ]
    assert result["plots"][281]["values"] == [
        0.223606797749979,
        0.23706539182259395,
        0.25059928172283336,
        0.2641968962724581,
        0.2778488797889961,
    ]
    assert result["plots"][282]["values"] == [
        2.995732273553991,
        3.044522437723423,
        3.091042453358316,
        3.1354942159291497,
        3.1780538303479458,
    ]
    assert result["plots"][283]["values"] == [
        1.2214027581601699,
        1.2336780599567432,
        1.2460767305873808,
        1.2586000099294778,
        1.2712491503214047,
    ]
    assert result["plots"][284]["values"] == [
        1.4706289056333368,
        1.4656024257545082,
        1.46057327680715,
        1.455541327127319,
        1.4505064444001086,
    ]
    assert result["plots"][285]["values"] == [
        0.1001674211615598,
        0.10519390104038849,
        0.11022304998774664,
        0.1152549996675776,
        0.12028988239478806,
    ]
    assert result["plots"][286]["values"] == [
        0.19739555984988078,
        0.206992194219821,
        0.21655030497608926,
        0.22606838799388393,
        0.23554498072086333,
    ]
    assert result["plots"][287]["values"] == [12.5, 13.5, 14.5, 15.5, 16.5]
    assert result["plots"][288]["values"] == [6.0, 7.0, 7.0, 7.0, 8.0]
    assert result["plots"][289]["values"] == [1.0, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][290]["values"] == [
        11.459155902616466,
        12.032113697747288,
        12.60507149287811,
        13.178029288008934,
        13.750987083139757,
    ]
    assert result["plots"][291]["values"] == [
        0.017453292519943295,
        0.019198621771937627,
        0.020943951023931952,
        0.022689280275926284,
        0.024434609527920613,
    ]
    assert result["plots"][292]["values"] == [2.0, 10.0, 10.0, 10.0, 10.0]
    assert result["plots"][293]["values"] == [2.0, 6.0, 8.0, 9.0, 9.5]
    assert result["plots"][294]["values"] == [None, 12.0, 13.0, 14.0, 15.0]
    assert result["plots"][295]["values"] == [None, 9.0, 10.0, 11.0, 12.0]
    assert result["plots"][296]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][297]["values"] == [
        None,
        5.0,
        4.761904761904762,
        4.545454545454546,
        4.3478260869565215,
    ]
    assert result["plots"][298]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][299]["values"] == [None, 0.5, 0.5, 0.5, 0.5]
    assert result["plots"][300]["values"] == [
        20.0,
        20.666666666666668,
        21.555555555555557,
        22.51851851851852,
        23.506172839506174,
    ]
    assert result["plots"][301]["values"] == [None, 100.0, 100.0, 100.0, 100.0]
    assert result["plots"][302]["values"] == [
        None,
        0.0975609756097561,
        0.09302325581395349,
        0.08888888888888889,
        0.0851063829787234,
    ]
    assert result["plots"][303]["values"] == [None, 12.0, 13.0, 14.0, 15.0]
    assert result["plots"][304]["values"] == [None, 9.0, 10.0, 11.0, 12.0]
    assert result["plots"][305]["values"] == [None, 0.0, 0.0, 0.0, 0.0]
    assert result["plots"][306]["values"] == [None, 1.0, 1.0, 1.0, 1.0]
    assert result["plots"][307]["values"] == [None, 41.0, 43.0, 45.0, 47.0]
    assert result["plots"][308]["values"] == [20.01, 21.01, 22.01, 23.01, 24.01]


def test_run_script_reports_missing_request_bars():
    program = pine_compat.compile_script(
        'indicator("request")\nplot(request.security("NYSE:IBM", timeframe.period, close))\n'
    )

    try:
        program.run(BARS)
    except ValueError as error:
        assert "missing request data for symbol `NYSE:IBM` timeframe `1`" in str(error)
    else:
        raise AssertionError("missing request data should fail")


def test_run_script_returns_label_outputs():
    result = pine_compat.run_script(
        'indicator("labels")\nif bar_index == 0\n    label_id = label.new(bar_index, high, "start")\nplot(close)\n',
        BARS,
    )

    assert result["labels"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 0,
                    "exists": True,
                    "x": 0,
                    "y": 1.0,
                    "text": "start",
                    "xloc": "xloc.bar_index",
                    "yloc": "yloc.price",
                    "color": 0x2196F3,
                    "style": "label.style_label_down",
                    "textColor": 0xFFFFFF,
                    "size": "size.normal",
                    "tooltip": "",
                    "textAlign": "text.align_center",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                }
            ],
        }
    ]


def test_run_script_returns_label_text_formatting_outputs():
    result = pine_compat.run_script(
        'indicator("label formatting")\nif bar_index == 1\n    label_id = label.new(bar_index, high, "start", text_formatting=text.format_bold)\n    label.set_text_formatting(label_id, text.format_bold + text.format_italic)\nlabel.set_text_formatting(na, text.format_italic)\nplot(close)\n',
        BARS,
    )

    snapshots = result["labels"][0]["snapshots"]
    assert snapshots[0]["textFormatting"] == 1
    assert snapshots[1]["textFormatting"] == 3


def test_run_script_returns_label_array_outputs():
    result = pine_compat.run_script(
        'indicator("label array")\nvar labels = array.new_label()\nif bar_index == 0\n    id = label.new(bar_index, high, "start")\n    array.push(labels, id)\nif bar_index == 1\n    copied = array.copy(labels)\n    label.set_text(array.get(copied, 0), "array")\n    if array.includes(labels, array.first(labels))\n        from_array = labels.get(0)\n        from_array.set_color(color.green)\nplot(array.size(labels))\n',
        BARS,
    )

    assert result["plots"][0]["values"] == [1, 1, 1]
    snapshots = result["labels"][0]["snapshots"]
    assert snapshots[0]["text"] == "start"
    assert snapshots[1]["text"] == "array"
    assert snapshots[2]["color"] == 0x4CAF50


def test_run_script_returns_line_outputs():
    result = pine_compat.run_script(
        'indicator("lines")\nif bar_index == 1\n    line_id = line.new(bar_index, low, bar_index, high)\nplot(close)\n',
        BARS,
    )

    assert result["lines"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 1,
                    "y1": 2.0,
                    "x2": 1,
                    "y2": 2.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x2196F3,
                    "width": 1,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                }
            ],
        }
    ]


def test_run_script_returns_line_new_style_outputs():
    result = pine_compat.run_script(
        'indicator("line new style")\nif bar_index == 1\n    line_id = line.new(x1=bar_index, y1=low, x2=bar_index + 1, y2=high, xloc=xloc.bar_index, extend=extend.right, color=color.green, style=line.style_dashed, width=2, force_overlay=false)\nplot(close)\n',
        BARS,
    )

    assert result["lines"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 1,
                    "y1": 2.0,
                    "x2": 2,
                    "y2": 2.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x4CAF50,
                    "width": 2,
                    "style": "line.style_dashed",
                    "extend": "extend.right",
                }
            ],
        }
    ]


def test_run_script_returns_line_set_xloc_outputs():
    result = pine_compat.run_script(
        'indicator("line set xloc")\nif bar_index == 1\n    line_id = line.new(bar_index, low, bar_index + 1, high)\n    line.set_xloc(line_id, bar_index - 1, bar_index + 3, xloc.bar_index)\nline.set_xloc(na, bar_index, bar_index, xloc.bar_index)\nplot(close)\n',
        BARS,
    )

    assert result["lines"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 1,
                    "y1": 2.0,
                    "x2": 2,
                    "y2": 2.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x2196F3,
                    "width": 1,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 0,
                    "y1": 2.0,
                    "x2": 4,
                    "y2": 2.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x2196F3,
                    "width": 1,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                },
            ],
        }
    ]


def test_run_script_returns_line_array_outputs():
    result = pine_compat.run_script(
        'indicator("line array")\nvar lines = array.new_line()\nif bar_index == 0\n    id = line.new(bar_index, low, bar_index + 1, high)\n    array.push(lines, id)\nif bar_index == 1\n    copied = array.copy(lines)\n    line.set_color(array.get(copied, 0), color.green)\n    if array.includes(lines, array.first(lines))\n        from_array = lines.get(0)\n        from_array.set_width(2)\nplot(array.size(lines))\n',
        BARS,
    )

    assert result["plots"][0]["values"] == [1, 1, 1]
    assert result["lines"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 0,
                    "exists": True,
                    "x1": 0,
                    "y1": 1.0,
                    "x2": 1,
                    "y2": 1.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x2196F3,
                    "width": 1,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 0,
                    "y1": 1.0,
                    "x2": 1,
                    "y2": 1.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x4CAF50,
                    "width": 1,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "x1": 0,
                    "y1": 1.0,
                    "x2": 1,
                    "y2": 1.0,
                    "xloc": "xloc.bar_index",
                    "color": 0x4CAF50,
                    "width": 2,
                    "style": "line.style_solid",
                    "extend": "extend.none",
                },
            ],
        }
    ]


def test_run_script_returns_line_getter_plot_values():
    result = pine_compat.run_script(
        'indicator("line getters")\nvar line_id = line.new(bar_index, low, bar_index + 1, high)\nif bar_index == 1\n    line.set_x1(line_id, bar_index - 10)\n    line.set_x2(line_id, bar_index + 10)\n    line.set_y1(line_id, low - 10)\n    line.set_y2(line_id, high + 10)\nplot(line.get_x1(line_id))\nplot(line.get_y1(line_id))\nplot(line.get_x2(line_id))\nplot(line.get_y2(line_id))\nplot(line.get_price(line_id, bar_index + 5))\n',
        BARS,
    )

    assert result["plots"][0]["values"] == [0, -9, -9]
    assert result["plots"][1]["values"] == [1.0, -8.0, -8.0]
    assert result["plots"][2]["values"] == [1, 11, 11]
    assert result["plots"][3]["values"] == [1.0, 12.0, 12.0]
    assert result["plots"][4]["values"] == [1.0, 7.0, 8.0]


def test_run_script_returns_box_outputs():
    result = pine_compat.run_script(
        'indicator("boxes")\nif bar_index == 1\n    box_id = box.new(bar_index, high, bar_index, low)\n    box.set_bgcolor(box_id, color.green)\nplot(close)\n',
        BARS,
    )

    assert result["boxes"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "left": 1,
                    "top": 2.0,
                    "right": 1,
                    "bottom": 2.0,
                    "xloc": "xloc.bar_index",
                    "bgColor": 0x2196F3,
                    "borderColor": 0x2196F3,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": 0x363A45,
                    "textSize": "size.auto",
                    "textHalign": "text.align_center",
                    "textValign": "text.align_center",
                    "textWrap": "text.wrap_none",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "left": 1,
                    "top": 2.0,
                    "right": 1,
                    "bottom": 2.0,
                    "xloc": "xloc.bar_index",
                    "bgColor": 0x4CAF50,
                    "borderColor": 0x2196F3,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": 0x363A45,
                    "textSize": "size.auto",
                    "textHalign": "text.align_center",
                    "textValign": "text.align_center",
                    "textWrap": "text.wrap_none",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                },
            ],
        }
    ]


def test_run_script_returns_box_set_xloc_outputs():
    result = pine_compat.run_script(
        'indicator("box set xloc")\nif bar_index == 1\n    box_id = box.new(bar_index, high, bar_index + 1, low)\n    box.set_xloc(box_id, bar_index - 1, bar_index + 3, xloc.bar_index)\nbox.set_xloc(na, bar_index, bar_index + 1, xloc.bar_index)\nplot(close)\n',
        BARS,
    )

    assert result["boxes"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "left": 1,
                    "top": 2.0,
                    "right": 2,
                    "bottom": 2.0,
                    "xloc": "xloc.bar_index",
                    "bgColor": 0x2196F3,
                    "borderColor": 0x2196F3,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": 0x363A45,
                    "textSize": "size.auto",
                    "textHalign": "text.align_center",
                    "textValign": "text.align_center",
                    "textWrap": "text.wrap_none",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "left": 0,
                    "top": 2.0,
                    "right": 4,
                    "bottom": 2.0,
                    "xloc": "xloc.bar_index",
                    "bgColor": 0x2196F3,
                    "borderColor": 0x2196F3,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": 0x363A45,
                    "textSize": "size.auto",
                    "textHalign": "text.align_center",
                    "textValign": "text.align_center",
                    "textWrap": "text.wrap_none",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                },
            ],
        }
    ]


def test_run_script_accepts_drawing_object_method_syntax():
    result = pine_compat.run_script(
        'indicator("drawing methods")\nvar label_id = label.new(bar_index, high, "start")\nvar line_id = line.new(bar_index, low, bar_index + 1, high)\nvar box_id = box.new(bar_index, high, bar_index + 1, low)\nvar table_id = table.new(position.top_right, 1, 1)\nif bar_index == 1\n    label_id.set_text("method")\n    label_id.set_xy(bar_index, close)\n    line_id.set_xy1(bar_index, low)\n    line_id.set_color(color.green)\n    box_id.set_lefttop(bar_index, high)\n    box_id.set_xloc(bar_index - 1, bar_index + 1, xloc.bar_index)\n    table_id.cell(0, 0, "A")\n    table_id.set_bgcolor(color.green)\nplot(str.length(label_id.get_text()))\nplot(line_id.get_x1())\nplot(box_id.get_right())\nplot(close)\n',
        BARS,
    )

    assert result["plots"][0]["values"] == [5, 6, 6]
    assert result["plots"][1]["values"] == [0, 1, 1]
    assert result["plots"][2]["values"] == [1, 2, 2]
    assert result["tables"][0]["bgColor"] == 0x4CAF50


def test_run_script_returns_box_new_style_outputs():
    result = pine_compat.run_script(
        'indicator("box new style")\nif bar_index == 1\n    box_id = box.new(left=bar_index, top=high, right=bar_index + 1, bottom=low, border_color=color.white, border_width=2, border_style=line.style_dashed, extend=extend.right, xloc=xloc.bar_index, bgcolor=color.green, text="styled", text_size=size.small, text_color=color.white, text_halign=text.align_left, text_valign=text.align_top, text_wrap=text.wrap_auto, text_font_family=font.family_monospace, force_overlay=false, text_formatting=text.format_bold + text.format_italic)\nplot(close)\n',
        BARS,
    )

    assert result["boxes"] == [
        {
            "id": 1,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "left": 1,
                    "top": 2.0,
                    "right": 2,
                    "bottom": 2.0,
                    "xloc": "xloc.bar_index",
                    "bgColor": 0x4CAF50,
                    "borderColor": 0xFFFFFF,
                    "borderWidth": 2,
                    "borderStyle": "line.style_dashed",
                    "extend": "extend.right",
                    "text": "styled",
                    "textColor": 0xFFFFFF,
                    "textSize": "size.small",
                    "textHalign": "text.align_left",
                    "textValign": "text.align_top",
                    "textWrap": "text.wrap_auto",
                    "textFontFamily": "font.family_monospace",
                    "textFormatting": 3,
                }
            ],
        }
    ]


def test_run_script_returns_box_text_formatting_outputs():
    result = pine_compat.run_script(
        'indicator("box formatting")\nif bar_index == 1\n    box_id = box.new(bar_index, high, bar_index, low)\n    box.set_text_formatting(box_id, text.format_bold + text.format_italic)\nbox.set_text_formatting(na, text.format_italic)\nplot(close)\n',
        BARS,
    )

    snapshots = result["boxes"][0]["snapshots"]
    assert snapshots[0]["textFormatting"] == 0
    assert snapshots[1]["textFormatting"] == 3


def table_snapshots_without_empty_merges(tables):
    for table in tables:
        for snapshot in table["snapshots"]:
            if snapshot["exists"]:
                assert snapshot.pop("mergedCells") == []
                for cell in snapshot["cells"]:
                    if cell.get("textWrap") == "text.wrap_none":
                        cell.pop("textWrap")
                    assert cell.pop("tooltip") == ""
                    assert cell.pop("textFontFamily") == "font.family_default"
                    assert cell.pop("textFormatting") == 0
    return tables


def test_run_script_returns_table_outputs():
    result = pine_compat.run_script(
        'indicator("tables")\nif bar_index == 1\n    table_id = table.new(position.top_right, 2, 2)\n    table.cell(table_id, 0, 0, "A", bgcolor=color.green, text_color=color.white)\n    table.cell_set_text(table_id, 0, 0, "B")\n    table.cell_set_bgcolor(table_id, 0, 0, color.red)\n    table.cell_set_text_color(table_id, 0, 0, color.blue)\n    table.cell_set_width(table_id, 0, 0, 25)\n    table.cell_set_height(table_id, 0, 0, 40)\n    table.cell_set_text_size(table_id, 0, 0, size.small)\n    table.cell_set_text_halign(table_id, 0, 0, text.align_left)\n    table.cell_set_text_valign(table_id, 0, 0, text.align_top)\n    table.cell_set_text_wrap(table_id, 0, 0, text.wrap_auto)\n    table.set_position(table_id, position.bottom_right)\n    table.set_bgcolor(table_id, color.yellow)\n    table.set_frame_color(table_id, color.black)\n    table.set_frame_width(table_id, 3)\n    table.set_border_color(table_id, color.white)\n    table.set_border_width(table_id, 4)\nplot(close)\n',
        BARS,
    )

    assert table_snapshots_without_empty_merges(result["tables"]) == [
        {
            "id": 1,
            "position": "position.bottom_right",
            "bgColor": 0xFDD835,
            "frameColor": 0x363A45,
            "frameWidth": 3,
            "borderColor": 0xFFFFFF,
            "borderWidth": 4,
            "columns": 2,
            "rows": 2,
            "snapshots": [
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": 0x4CAF50,
                            "textColor": 0xFFFFFF,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0x4CAF50,
                            "textColor": 0xFFFFFF,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0xFFFFFF,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": 40,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": 40,
                            "textSize": "size.small",
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": 40,
                            "textSize": "size.small",
                            "textHalign": "text.align_left",
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": 40,
                            "textSize": "size.small",
                            "textHalign": "text.align_left",
                            "textValign": "text.align_top",
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0xF23645,
                            "textColor": 0x2196F3,
                            "width": 25,
                            "height": 40,
                            "textSize": "size.small",
                            "textHalign": "text.align_left",
                            "textValign": "text.align_top",
                            "textWrap": "text.wrap_auto",
                        }
                    ],
                },
            ],
        }
    ]


def test_run_script_returns_table_delete_outputs():
    result = pine_compat.run_script(
        'indicator("table delete")\nvar table_id = table.new(position.top_right, 1, 1)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A")\n    table.delete(table_id)\nif bar_index == 2\n    table.cell(table_id, 0, 0, "ignored")\n    table.delete(table_id)\ntable.delete(na)\nplot(close)\n',
        BARS,
    )

    assert table_snapshots_without_empty_merges(result["tables"]) == [
        {
            "id": 1,
            "position": "position.top_right",
            "bgColor": None,
            "frameColor": None,
            "frameWidth": None,
            "borderColor": None,
            "borderWidth": None,
            "columns": 1,
            "rows": 1,
            "snapshots": [
                {"barIndex": 0, "exists": True, "cells": []},
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": None,
                            "textColor": None,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {"barIndex": 1, "exists": False},
            ],
        }
    ]


def test_run_script_returns_table_clear_outputs():
    result = pine_compat.run_script(
        'indicator("table clear")\nvar table_id = table.new(position.top_right, 2, 2)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A")\n    table.cell(table_id, 1, 0, "B", bgcolor=color.green)\n    table.clear(table_id, 1, 0, 1, 0)\nif bar_index == 2\n    table.clear(table_id, 0, 0, 0, 0)\ntable.clear(na, 0, 0, 0, 0)\nplot(close)\n',
        BARS,
    )

    assert table_snapshots_without_empty_merges(result["tables"]) == [
        {
            "id": 1,
            "position": "position.top_right",
            "bgColor": None,
            "frameColor": None,
            "frameWidth": None,
            "borderColor": None,
            "borderWidth": None,
            "columns": 2,
            "rows": 2,
            "snapshots": [
                {"barIndex": 0, "exists": True, "cells": []},
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": None,
                            "textColor": None,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": None,
                            "textColor": None,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        },
                        {
                            "column": 1,
                            "row": 0,
                            "text": "B",
                            "bgColor": 0x4CAF50,
                            "textColor": None,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        },
                    ],
                },
                {
                    "barIndex": 1,
                    "exists": True,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": None,
                            "textColor": None,
                            "width": None,
                            "height": None,
                            "textSize": None,
                            "textHalign": None,
                            "textValign": None,
                        }
                    ],
                },
                {"barIndex": 2, "exists": True, "cells": []},
            ],
        }
    ]


def test_run_script_returns_table_merge_cell_outputs():
    result = pine_compat.run_script(
        'indicator("table merge")\nvar table_id = table.new(position.top_right, 3, 2)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A")\n    table.merge_cells(table_id, 0, 0, 2, 0)\n    table.cell(table_id, 0, 1, "B")\n    table.merge_cells(table_id, 0, 1, 1, 1)\nif bar_index == 2\n    table.clear(table_id, 0, 1, 1, 1)\ntable.merge_cells(na, 0, 0, 0, 0)\nplot(close)\n',
        BARS,
    )

    snapshots = result["tables"][0]["snapshots"]
    assert snapshots[0]["mergedCells"] == []
    assert snapshots[2]["mergedCells"] == [
        {"startColumn": 0, "startRow": 0, "endColumn": 2, "endRow": 0}
    ]
    assert snapshots[4]["mergedCells"] == [
        {"startColumn": 0, "startRow": 0, "endColumn": 2, "endRow": 0},
        {"startColumn": 0, "startRow": 1, "endColumn": 1, "endRow": 1},
    ]
    assert snapshots[5]["mergedCells"] == [
        {"startColumn": 0, "startRow": 0, "endColumn": 2, "endRow": 0}
    ]


def test_run_script_returns_table_cell_tooltip_outputs():
    result = pine_compat.run_script(
        'indicator("table tooltip")\nvar table_id = table.new(position.top_right, 1, 1)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A", tooltip="initial")\n    table.cell_set_tooltip(table_id, 0, 0, "updated")\ntable.cell_set_tooltip(na, 0, 0, "noop")\nplot(close)\n',
        BARS,
    )

    snapshots = result["tables"][0]["snapshots"]
    assert snapshots[1]["cells"][0]["tooltip"] == "initial"
    assert snapshots[2]["cells"][0]["tooltip"] == "updated"


def test_run_script_returns_table_cell_text_font_family_outputs():
    result = pine_compat.run_script(
        'indicator("table font")\nvar table_id = table.new(position.top_right, 1, 1)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A", text_font_family=font.family_monospace)\n    table.cell_set_text_font_family(table_id, 0, 0, font.family_default)\ntable.cell_set_text_font_family(na, 0, 0, font.family_monospace)\nplot(close)\n',
        BARS,
    )

    snapshots = result["tables"][0]["snapshots"]
    assert snapshots[1]["cells"][0]["textFontFamily"] == "font.family_monospace"
    assert snapshots[2]["cells"][0]["textFontFamily"] == "font.family_default"


def test_run_script_returns_table_cell_text_formatting_outputs():
    result = pine_compat.run_script(
        'indicator("table formatting")\nvar table_id = table.new(position.top_right, 1, 1)\nif bar_index == 1\n    table.cell(table_id, 0, 0, "A", text_formatting=text.format_bold)\n    table.cell_set_text_formatting(table_id, 0, 0, text.format_bold + text.format_italic)\ntable.cell_set_text_formatting(na, 0, 0, text.format_italic)\nplot(close)\n',
        BARS,
    )

    snapshots = result["tables"][0]["snapshots"]
    assert snapshots[1]["cells"][0]["textFormatting"] == 1
    assert snapshots[2]["cells"][0]["textFormatting"] == 3


def test_run_script_returns_plotchar_outputs():
    result = pine_compat.run_script(
        'indicator("markers")\nplotchar(close > 2, char="x", color=color.green)\nplot(close)\n',
        BARS,
    )

    assert result["plotChars"][0]["values"] == [False, False, True]
    assert result["plotChars"][0]["chars"] == ["x", "x", "x"]
    assert result["plotChars"][0]["colors"] == [0x4CAF50, 0x4CAF50, 0x4CAF50]


def test_run_script_returns_plotshape_outputs():
    result = pine_compat.run_script(
        'indicator("shapes")\nplotshape(close > 2, style=shape.triangleup, location=location.belowbar, color=color.green, text="Buy", textcolor=color.white, size=size.small)\nplot(close)\n',
        BARS,
    )

    assert result["plotShapes"][0]["values"] == [False, False, True]
    assert set(result["plotShapes"][0]) == {
        "id",
        "values",
        "styles",
        "locations",
        "colors",
        "texts",
        "textColors",
        "sizes",
    }
    assert result["plotShapes"][0]["styles"] == [
        "shape.triangleup",
        "shape.triangleup",
        "shape.triangleup",
    ]
    assert result["plotShapes"][0]["locations"] == [
        "location.belowbar",
        "location.belowbar",
        "location.belowbar",
    ]
    assert result["plotShapes"][0]["colors"] == [0x4CAF50, 0x4CAF50, 0x4CAF50]
    assert result["plotShapes"][0]["texts"] == ["Buy", "Buy", "Buy"]
    assert result["plotShapes"][0]["textColors"] == [
        0xFFFFFF,
        0xFFFFFF,
        0xFFFFFF,
    ]
    assert result["plotShapes"][0]["sizes"] == [
        "size.small",
        "size.small",
        "size.small",
    ]


def test_run_script_returns_plotarrow_outputs():
    result = pine_compat.run_script(
        'indicator("arrows")\nplotarrow(close - 2, colorup=color.green, colordown=color.red, minheight=5, maxheight=20)\nplot(close)\n',
        BARS,
    )

    assert result["plotArrows"][0]["values"] == [-1.0, 0.0, 1.0]
    assert result["plotArrows"][0]["colorUps"] == [0x4CAF50, 0x4CAF50, 0x4CAF50]
    assert result["plotArrows"][0]["colorDowns"] == [0xF23645, 0xF23645, 0xF23645]
    assert result["plotArrows"][0]["minHeights"] == [5, 5, 5]
    assert result["plotArrows"][0]["maxHeights"] == [20, 20, 20]


def test_run_script_returns_plotbar_outputs():
    result = pine_compat.run_script(
        'indicator("bars")\nplotbar(open, high, low, close, color=color.green)\nplot(close)\n',
        BARS,
    )

    assert result["plotBars"][0]["opens"] == [1.0, 2.0, 3.0]
    assert result["plotBars"][0]["highs"] == [1.0, 2.0, 3.0]
    assert result["plotBars"][0]["lows"] == [1.0, 2.0, 3.0]
    assert result["plotBars"][0]["closes"] == [1.0, 2.0, 3.0]
    assert result["plotBars"][0]["colors"] == [0x4CAF50, 0x4CAF50, 0x4CAF50]


def test_run_script_returns_plotcandle_outputs():
    result = pine_compat.run_script(
        'indicator("candles")\nplotcandle(open, high, low, close, color=color.green, wickcolor=color.white, bordercolor=color.red)\nplot(close)\n',
        BARS,
    )

    assert result["plotCandles"][0]["opens"] == [1.0, 2.0, 3.0]
    assert set(result["plotCandles"][0]) == {
        "id",
        "opens",
        "highs",
        "lows",
        "closes",
        "colors",
        "wickColors",
        "borderColors",
    }
    assert result["plotCandles"][0]["highs"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["lows"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["closes"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["colors"] == [0x4CAF50, 0x4CAF50, 0x4CAF50]
    assert result["plotCandles"][0]["wickColors"] == [0xFFFFFF, 0xFFFFFF, 0xFFFFFF]
    assert result["plotCandles"][0]["borderColors"] == [0xF23645, 0xF23645, 0xF23645]


def _runtime_fixture_result(source_name, bars_name="bars.csv"):
    source = (ROOT / "tests/fixtures/runtime" / source_name).read_text()
    bars = fixture_bars(f"tests/fixtures/runtime/{bars_name}")
    return pine_compat.run_script(source, bars)


def test_array_history_host_golden_parity():
    actual = {
        snapshot: _runtime_fixture_result(source)
        for snapshot, source in [
            ("runtime_array_history.json", "array_history.pine"),
            ("runtime_array_label_history.json", "array_label_history.pine"),
            ("runtime_array_line_history.json", "array_line_history.pine"),
            ("runtime_array_box_history.json", "array_box_history.pine"),
            ("runtime_array_linefill_history.json", "array_linefill_history.pine"),
            ("runtime_array_polyline_history.json", "array_polyline_history.pine"),
            ("runtime_array_table_history.json", "array_table_history.pine"),
            (
                "runtime_array_chart_point_history.json",
                "array_chart_point_history.pine",
            ),
            ("runtime_array_slice_history.json", "array_slice_history.pine"),
            (
                "runtime_array_label_slice_history.json",
                "array_label_slice_history.pine",
            ),
            (
                "runtime_array_line_slice_history.json",
                "array_line_slice_history.pine",
            ),
            (
                "runtime_array_box_slice_history.json",
                "array_box_slice_history.pine",
            ),
            (
                "runtime_array_linefill_slice_history.json",
                "array_linefill_slice_history.pine",
            ),
            (
                "runtime_array_polyline_slice_history.json",
                "array_polyline_slice_history.pine",
            ),
            (
                "runtime_array_table_slice_history.json",
                "array_table_slice_history.pine",
            ),
            (
                "runtime_array_chart_point_slice_history.json",
                "array_chart_point_slice_history.pine",
            ),
        ]
    }
    expected = {
        Path(path).name: json.loads((ROOT / path).read_text())
        for path in [
            "tests/snapshots/runtime_array_history.json",
            "tests/snapshots/runtime_array_label_history.json",
            "tests/snapshots/runtime_array_line_history.json",
            "tests/snapshots/runtime_array_box_history.json",
            "tests/snapshots/runtime_array_linefill_history.json",
            "tests/snapshots/runtime_array_polyline_history.json",
            "tests/snapshots/runtime_array_table_history.json",
            "tests/snapshots/runtime_array_chart_point_history.json",
            "tests/snapshots/runtime_array_slice_history.json",
            "tests/snapshots/runtime_array_label_slice_history.json",
            "tests/snapshots/runtime_array_line_slice_history.json",
            "tests/snapshots/runtime_array_box_slice_history.json",
            "tests/snapshots/runtime_array_linefill_slice_history.json",
            "tests/snapshots/runtime_array_polyline_slice_history.json",
            "tests/snapshots/runtime_array_table_slice_history.json",
            "tests/snapshots/runtime_array_chart_point_slice_history.json",
        ]
    }

    assert actual == expected


def test_strategy_remaining_host_golden_parity():
    actual = {
        snapshot: _runtime_fixture_result(source, bars)
        for snapshot, source, bars in [
            ("runtime_strategy_close_noop.json", "strategy_close_noop.pine", "bars.csv"),
            (
                "runtime_strategy_closedtrades_fields_pyramiding.json",
                "strategy_closedtrades_fields_pyramiding.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_commission_cash_per_contract.json",
                "strategy_commission_cash_per_contract.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_commission_cash_per_order.json",
                "strategy_commission_cash_per_order.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_commission_percent.json",
                "strategy_commission_percent.pine",
                "bars.csv",
            ),
            ("runtime_strategy_empty.json", "strategy_no_order.pine", "bars.csv"),
            (
                "runtime_strategy_entry_limit.json",
                "strategy_entry_limit.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_entry_stop.json",
                "strategy_entry_stop.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_entry_stop_limit.json",
                "strategy_entry_stop_limit.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_loss_attachment.json",
                "strategy_exit_active_entry_loss_attachment.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_loss_limit_bracket.json",
                "strategy_exit_active_entry_loss_limit_bracket.pine",
                "strategy_exit_trailing_bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_loss_profit_bracket.json",
                "strategy_exit_active_entry_loss_profit_bracket.pine",
                "strategy_exit_trailing_bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_profit_attachment.json",
                "strategy_exit_active_entry_profit_attachment.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_stop_profit_bracket.json",
                "strategy_exit_active_entry_stop_profit_bracket.pine",
                "strategy_exit_trailing_bars.csv",
            ),
            (
                "runtime_strategy_exit_active_entry_trail_points_attachment.json",
                "strategy_exit_active_entry_trail_points_attachment.pine",
                "strategy_exit_trailing_bars.csv",
            ),
            (
                "runtime_strategy_exit_reservation_bracket_host_parity.json",
                "strategy_exit_reservation_bracket_host_parity.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_exit_reservation_trailing_host_parity.json",
                "strategy_exit_reservation_trailing_host_parity.pine",
                "strategy_exit_reservation_trailing_host_parity_bars.csv",
            ),
            (
                "runtime_strategy_exit_slippage.json",
                "strategy_exit_slippage.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_limit_verification_entry.json",
                "strategy_limit_verification_entry.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_limit_verification_exit.json",
                "strategy_limit_verification_exit.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_opentrades_fields_pyramiding.json",
                "strategy_opentrades_fields_pyramiding.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_pyramiding.json",
                "strategy_pyramiding.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_close.json",
                "strategy_pyramiding_close.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_close_all.json",
                "strategy_pyramiding_close_all.pine",
                "bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_exit_bracket_from_entry.json",
                "strategy_pyramiding_exit_bracket_from_entry.pine",
                "strategy_pyramiding_exit_profit_from_entry_bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_exit_from_entry.json",
                "strategy_pyramiding_exit_from_entry.pine",
                "strategy_pyramiding_exit_from_entry_bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_exit_profit_from_entry.json",
                "strategy_pyramiding_exit_profit_from_entry.pine",
                "strategy_pyramiding_exit_profit_from_entry_bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_exit_same_id.json",
                "strategy_pyramiding_exit_same_id.pine",
                "strategy_pyramiding_exit_same_id_bars.csv",
            ),
            (
                "runtime_strategy_pyramiding_exit_trail_points_from_entry.json",
                "strategy_pyramiding_exit_trail_points_from_entry.pine",
                "strategy_pyramiding_exit_trail_points_from_entry_bars.csv",
            ),
            (
                "runtime_strategy_slippage.json",
                "strategy_slippage.pine",
                "bars.csv",
            ),
        ]
    }
    expected = {
        Path(path).name: json.loads((ROOT / path).read_text())
        for path in [
            "tests/snapshots/runtime_strategy_close_noop.json",
            "tests/snapshots/runtime_strategy_closedtrades_fields_pyramiding.json",
            "tests/snapshots/runtime_strategy_commission_cash_per_contract.json",
            "tests/snapshots/runtime_strategy_commission_cash_per_order.json",
            "tests/snapshots/runtime_strategy_commission_percent.json",
            "tests/snapshots/runtime_strategy_empty.json",
            "tests/snapshots/runtime_strategy_entry_limit.json",
            "tests/snapshots/runtime_strategy_entry_stop.json",
            "tests/snapshots/runtime_strategy_entry_stop_limit.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_loss_attachment.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_loss_limit_bracket.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_loss_profit_bracket.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_profit_attachment.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_stop_profit_bracket.json",
            "tests/snapshots/runtime_strategy_exit_active_entry_trail_points_attachment.json",
            "tests/snapshots/runtime_strategy_exit_reservation_bracket_host_parity.json",
            "tests/snapshots/runtime_strategy_exit_reservation_trailing_host_parity.json",
            "tests/snapshots/runtime_strategy_exit_slippage.json",
            "tests/snapshots/runtime_strategy_limit_verification_entry.json",
            "tests/snapshots/runtime_strategy_limit_verification_exit.json",
            "tests/snapshots/runtime_strategy_opentrades_fields_pyramiding.json",
            "tests/snapshots/runtime_strategy_pyramiding.json",
            "tests/snapshots/runtime_strategy_pyramiding_close.json",
            "tests/snapshots/runtime_strategy_pyramiding_close_all.json",
            "tests/snapshots/runtime_strategy_pyramiding_exit_bracket_from_entry.json",
            "tests/snapshots/runtime_strategy_pyramiding_exit_from_entry.json",
            "tests/snapshots/runtime_strategy_pyramiding_exit_profit_from_entry.json",
            "tests/snapshots/runtime_strategy_pyramiding_exit_same_id.json",
            "tests/snapshots/runtime_strategy_pyramiding_exit_trail_points_from_entry.json",
            "tests/snapshots/runtime_strategy_slippage.json",
        ]
    }

    assert actual == expected


def test_map_matrix_representative_host_golden_parity():
    actual = {
        snapshot: _runtime_fixture_result(source)
        for snapshot, source in [
            ("runtime_map_methods.json", "map_methods.pine"),
            ("runtime_map_history.json", "map_history.pine"),
            ("runtime_map_for_in.json", "map_for_in.pine"),
            ("runtime_map_varip.json", "map_varip.pine"),
            ("runtime_matrix_int.json", "matrix_int.pine"),
            ("runtime_matrix_bool.json", "matrix_bool.pine"),
            ("runtime_matrix_history_shape.json", "matrix_history_shape.pine"),
            ("runtime_matrix_for_in.json", "matrix_for_in.pine"),
            ("runtime_matrix_mult.json", "matrix_mult.pine"),
            ("runtime_matrix_inv.json", "matrix_inv.pine"),
            ("runtime_matrix_varip.json", "matrix_varip.pine"),
            ("runtime_matrix_zero_dimensions.json", "matrix_zero_dimensions.pine"),
        ]
    }
    expected = {
        Path(path).name: json.loads((ROOT / path).read_text())
        for path in [
            "tests/snapshots/runtime_map_methods.json",
            "tests/snapshots/runtime_map_history.json",
            "tests/snapshots/runtime_map_for_in.json",
            "tests/snapshots/runtime_map_varip.json",
            "tests/snapshots/runtime_matrix_int.json",
            "tests/snapshots/runtime_matrix_bool.json",
            "tests/snapshots/runtime_matrix_history_shape.json",
            "tests/snapshots/runtime_matrix_for_in.json",
            "tests/snapshots/runtime_matrix_mult.json",
            "tests/snapshots/runtime_matrix_inv.json",
            "tests/snapshots/runtime_matrix_varip.json",
            "tests/snapshots/runtime_matrix_zero_dimensions.json",
        ]
    }

    assert actual == expected
