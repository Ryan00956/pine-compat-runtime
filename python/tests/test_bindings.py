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

    assert report["schemaVersion"] == 2
    assert report["executable"] is True
    assert report["diagnostics"] == []
    assert any(
        feature["feature"] == "plot"
        for feature in report["compatibility"]["supported"]
    )


def test_compile_script_returns_program_with_run_method():
    program = pine_compat.compile_script('indicator("demo")\nplot(close)\n')
    result = program.run(BARS)

    assert result["schemaVersion"] == 3
    assert set(result) == RUNTIME_RESULT_KEYS
    assert result["labels"] == []
    assert result["lines"] == []
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


def test_run_script_converts_non_finite_plot_values_to_none():
    result = pine_compat.run_script('indicator("demo")\nplot(1.0 / 0.0)\n', BARS)

    assert result["plots"][0]["values"] == [None, None, None]


def test_run_script_returns_plotchar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotchar.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["plotChars"] == [
        {
            "id": 1,
            "values": [None, False, True],
            "chars": [None, "x", "x"],
            "colors": [None, 32768, 32768],
        }
    ]


def test_run_script_returns_plotshape_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotshape.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["plotShapes"] == [
        {
            "id": 1,
            "values": [None, False, True],
            "styles": [None, "shape.triangleup", "shape.triangleup"],
            "locations": [None, "location.belowbar", "location.belowbar"],
            "colors": [None, 32768, 32768],
            "texts": [None, "Buy", "Buy"],
            "textColors": [None, 16777215, 16777215],
            "sizes": [None, "size.small", "size.small"],
        }
    ]


def test_run_script_returns_plotarrow_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotarrow.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["plotArrows"] == [
        {
            "id": 1,
            "values": [None, 0.0, 1.0],
            "colorUps": [None, 32768, 32768],
            "colorDowns": [None, 16711680, 16711680],
            "minHeights": [None, 5, 5],
            "maxHeights": [None, 20, 20],
        }
    ]


def test_run_script_returns_plotbar_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotbar.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["plotBars"] == [
        {
            "id": 1,
            "opens": [None, 2.0, 3.0],
            "highs": [None, 2.0, 3.0],
            "lows": [None, 2.0, 3.0],
            "closes": [None, 2.0, 3.0],
            "colors": [None, 32768, 32768],
        }
    ]


def test_run_script_returns_plotcandle_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/plotcandle.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"] == [
        {
            "id": 1,
            "opens": [None, 2.0, 3.0],
            "highs": [None, 2.0, 3.0],
            "lows": [None, 2.0, 3.0],
            "closes": [None, 2.0, 3.0],
            "colors": [None, 32768, 32768],
            "wickColors": [None, 16777215, 16777215],
            "borderColors": [None, 16711680, 16711680],
        }
    ]


def test_run_script_returns_color_outputs_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/color_outputs.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["bgColors"] == [
        {
            "id": 1,
            "values": [None, 32768, 32768],
        }
    ]
    assert result["barColors"] == [
        {
            "id": 2,
            "values": [None, None, 16711680],
        }
    ]


def test_run_script_returns_hline_fill_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/io.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [None, 2.25, 3.75]
    assert result["hlines"] == [
        {
            "id": 10,
            "price": 2.0,
        }
    ]
    assert result["fills"] == [
        {
            "id": 11,
            "firstId": 7,
            "secondId": 10,
        }
    ]


def test_run_script_returns_alertcondition_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/alertcondition.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert result["diagnostics"] == []
    assert result["alerts"] == [
        {
            "id": 3,
            "barIndex": 1,
            "time": 2,
            "message": "Branch alert",
            "source": "Branch",
        },
        {
            "id": 1,
            "barIndex": 2,
            "time": 3,
            "message": "Close is above two",
            "source": "Above two",
        },
        {
            "id": 1,
            "barIndex": 3,
            "time": 4,
            "message": "Close is above two",
            "source": "Above two",
        },
    ]


def test_run_script_returns_alert_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/alert.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert result["diagnostics"] == []
    assert result["alerts"] == [
        {
            "id": 1,
            "barIndex": 0,
            "time": 1,
            "message": "Every bar",
            "source": "alert",
        },
        {
            "id": 1,
            "barIndex": 1,
            "time": 2,
            "message": "Every bar",
            "source": "alert",
        },
        {
            "id": 2,
            "barIndex": 1,
            "time": 2,
            "message": "Branch alert",
            "source": "alert",
        },
        {
            "id": 1,
            "barIndex": 2,
            "time": 3,
            "message": "Every bar",
            "source": "alert",
        },
        {
            "id": 3,
            "barIndex": 2,
            "time": 3,
            "message": "Loop alert",
            "source": "alert",
        },
        {
            "id": 1,
            "barIndex": 3,
            "time": 4,
            "message": "Every bar",
            "source": "alert",
        },
    ]


def test_run_script_returns_label_new_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_new.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
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
                    "color": None,
                    "style": "label.style_label_down",
                    "textColor": None,
                    "size": "size.normal",
                    "tooltip": "",
                    "textAlign": "text.align_center",
                    "textFontFamily": "font.family_default",
                    "textFormatting": 0,
                }
            ],
        }
    ]


def test_run_script_returns_label_mutation_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/label_mutation.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_label_mutation.json").read_text())

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


def test_run_script_returns_array_helpers_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/array_helpers.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/runtime_array_helpers.json").read_text())

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


def test_run_script_returns_math_edge_cases_as_none():
    source = (ROOT / "tests/fixtures/runtime/math_edge_cases.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 8
    for plot in result["plots"]:
        assert plot["values"] == [None, None, None]


def test_run_script_returns_math_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/math.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 28
    expected = [
        [2.0, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 1.0, 2.0],
        [1.0, 1.4142135623730951, 1.7320508075688772],
        [1.0, 1.2599210498948732, 1.4422495703074083],
        [0.0, 0.6931471805599453, 1.0986122886681098],
        [0.0, 0.3010299956639812, 0.47712125471966244],
        [2.718281828459045, 7.38905609893065, 20.085536923187668],
        [3.141592653589793, 1.5707963267948966, 0.0],
        [-1.5707963267948966, 0.0, 1.5707963267948966],
        [0.7853981633974483, 1.1071487177940904, 1.2490457723982544],
        [-1.0, 0.0, 1.0],
        [57.29577951308232, 114.59155902616465, 171.88733853924697],
        [0.017453292519943295, 0.03490658503988659, 0.05235987755982989],
        [8.095942459548628, 8.095942459548628, 8.095942459548628],
        [0.8414709848078965, 0.9092974268256817, 0.1411200080598672],
        [0.5403023058681398, -0.4161468365471424, -0.9899924966004454],
        [1.5574077246549023, -2.185039863261519, -0.1425465430742778],
        [1.0, 4.0, 9.0],
        [2.23606797749979, 3.605551275463989, 5.0],
        [0.33, 0.67, 1.0],
        [1.01, 2.0100000000000002, 3.0100000000000002],
        [0.01, 0.01, 0.01],
        [17.044006538018998, 14.290355590862742, 15.77378930199836],
        [0.41627086372635447, 0.2277553881254537, 0.8642252595989811],
        [None, None, 6.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_computed_lengths_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/computed_lengths.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 2
    assert result["plots"][0]["values"] == [None, 1.5, 2.5]
    assert result["plots"][1]["values"] == [None, 3.0, 5.0]


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


def test_run_script_returns_udf_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/udf.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 1
    assert result["plots"][0]["values"] == [None, 4.5, 6.5]


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
    assert len(result["plots"]) == 3
    assert result["plots"][0]["values"] == [2.0, 2.0, 3.0, 4.0]
    assert result["plots"][1]["values"] == [2.0, 2.0, 3.0, 4.0]
    assert result["plots"][2]["values"] == [2.0, 2.0, 4.0, 4.0]


def test_run_script_returns_ta_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/ta.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 5
    expected = [
        [1.0, 1.6666666666666665, 2.5555555555555554],
        [1.0, 1.5, 2.25],
        [None, 100.0, 100.0],
        [None, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_dema_tema_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/dema_tema.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 3
    assert result["plots"][0]["values"] == [1.0, 1.75, 2.75]
    assert result["plots"][1]["values"] == [1.0, 1.875, 2.9375]
    assert result["plots"][2]["values"] == [None, None, None]


def test_run_script_returns_macd_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/macd.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 3
    expected = [
        [0.0, 0.16666666666666652, 0.30555555555555536],
        [0.0, 0.11111111111111101, 0.24074074074074056],
        [0.0, 0.05555555555555551, 0.0648148148148148],
    ]
    for plot, values in zip(result["plots"], expected):
        for actual, expected_value in zip(plot["values"], values):
            assert abs(actual - expected_value) < 1e-12


def test_run_script_returns_strings_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/strings.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 28
    assert result["plots"][0]["values"] == [3.0, 3.0, 3.0]
    for plot in result["plots"][1:]:
        assert plot["values"] == [1.0, 1.0, 1.0]


def test_run_script_returns_colors_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/colors.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["bgColors"]) == 1
    assert result["bgColors"][0]["values"] == [4288217216.0, 4288217216.0, 4288217216.0]
    assert len(result["plots"]) == 5
    expected = [
        [1.0, 2.0, 3.0],
        [1.0, 1.0, 1.0],
        [458.0, 458.0, 458.0],
        [458.0, 458.0, 458.0],
        [255.0, 192.0, 383.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_syminfo_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/syminfo.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 7
    expected = [
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.01, 0.01, 0.01],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [100.0, 100.0, 100.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_generic_input_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/generic_input.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 1
    assert result["plots"][0]["values"] == [None, 2.25, 3.75]


def test_run_script_returns_timeframe_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/timeframe.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 18
    expected = [
        [1.0, 1.0, 1.0],
        [60.0, 60.0, 60.0],
        [60.0, 60.0, 60.0],
        [1.0, 1.0, 1.0],
        [45.0, 45.0, 45.0],
        [3600.0, 3600.0, 3600.0],
        [86400.0, 86400.0, 86400.0],
        [1209600.0, 1209600.0, 1209600.0],
        [7776000.0, 7776000.0, 7776000.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_time_components_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/time_components.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 21
    expected = [
        [1970.0, 1970.0, 1970.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [5.0, 5.0, 5.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [2021.0, 2021.0, 2021.0],
        [2.0, 2.0, 2.0],
        [5.0, 5.0, 5.0],
        [2.0, 2.0, 2.0],
        [3.0, 3.0, 3.0],
        [3.0, 3.0, 3.0],
        [4.0, 4.0, 4.0],
        [5.0, 5.0, 5.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_global_series_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/global_series.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 12
    expected = [
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 2.0],
        [60000.0, 60001.0, 60002.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [0.0, 1.0, 2.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_casts_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/casts.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 10
    expected = [
        [None, 1.5, 2.5],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
        [3.0, 1.0, 3.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_barstate_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/barstate.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 6
    expected = [
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
    ]
    for plot, values in zip(result["plots"], expected):
        assert plot["values"] == values


def test_run_script_returns_session_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/session.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 3
    assert result["plots"][0]["values"] == [1.0, 1.0, 1.0]
    assert result["plots"][1]["values"] == [0.0, 0.0, 0.0]
    assert result["plots"][2]["values"] == [0.0, 0.0, 0.0]


def test_run_script_returns_inputs_fixture_contract():
    source = (ROOT / "tests/fixtures/runtime/inputs.pine").read_text()
    result = pine_compat.run_script(source, BARS)

    assert result["diagnostics"] == []
    assert len(result["plots"]) == 1
    assert result["plots"][0]["values"] == [0.0, 0.0, 3.0]


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


def test_run_script_returns_strategy_default_quantity_contract():
    result = pine_compat.run_script(
        'strategy("demo", default_qty_type=strategy.fixed, default_qty_value=3)\nif bar_index == 1\n    strategy.entry("D", strategy.long)\nplot(strategy.position_size)\n',
        BARS,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "D",
            "barIndex": 2,
            "time": 2,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 3.0, "avgPrice": 3.0}
    ]
    assert result["plots"][0]["values"] == [0.0, 0.0, 3.0]


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
    result = pine_compat.run_script(
        'strategy("demo", initial_capital=1000, default_qty_type=strategy.percent_of_equity, default_qty_value=25)\nif bar_index == 1\n    strategy.entry("D", strategy.long)\nplot(strategy.position_size)\n',
        BARS,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "D",
            "barIndex": 2,
            "time": 2,
            "direction": "strategy.long",
            "qty": 125.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 125.0, "avgPrice": 3.0}
    ]
    assert result["plots"][0]["values"] == [0.0, 0.0, 125.0]


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
    result = pine_compat.run_script(
        'strategy("demo", initial_capital=1000, default_qty_type=strategy.cash, default_qty_value=100)\nif bar_index == 1\n    strategy.entry("D", strategy.long)\nplot(strategy.position_size)\n',
        BARS,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "D",
            "barIndex": 2,
            "time": 2,
            "direction": "strategy.long",
            "qty": 50.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 50.0, "avgPrice": 3.0}
    ]
    assert result["plots"][0]["values"] == [0.0, 0.0, 50.0]


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
    result = pine_compat.run_script(
        'strategy("demo")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=2)\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nif bar_index == 2\n    strategy.close("L")\nplot(strategy.position_size)\nplot(strategy.position_avg_price)\nplot(strategy.max_contracts_held_all)\nplot(strategy.max_contracts_held_long)\nplot(strategy.max_contracts_held_short)\n',
        BARS,
    )

    assert result["plots"] == [
        {"id": 1, "values": [0.0, 0.0, 2.0]},
        {"id": 2, "values": [None, None, 3.0]},
        {"id": 4, "values": [0.0, 0.0, 2.0]},
        {"id": 5, "values": [None, None, 3.0]},
        {"id": 7, "values": [0.0, 0.0, 0.0]},
        {"id": 8, "values": [None, None, None]},
        {"id": 9, "values": [0.0, 0.0, 2.0]},
        {"id": 10, "values": [0.0, 0.0, 2.0]},
        {"id": 11, "values": [0.0, 0.0, 0.0]},
    ]


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
    result = pine_compat.run_script(
        'strategy("demo", initial_capital=1000)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\nplot(strategy.max_runup)\nplot(strategy.max_runup_percent)\nplot(strategy.max_drawdown)\nplot(strategy.max_drawdown_percent)\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=2)\nplot(strategy.openprofit)\nplot(strategy.netprofit)\nplot(strategy.equity)\nplot(strategy.max_runup)\nplot(strategy.max_runup_percent)\nplot(strategy.max_drawdown)\nplot(strategy.max_drawdown_percent)\n',
        BARS,
    )

    assert result["plots"] == [
        {"id": 1, "values": [0.0, 0.0, 0.0]},
        {"id": 2, "values": [0.0, 0.0, 0.0]},
        {"id": 3, "values": [1000.0, 1000.0, 1000.0]},
        {"id": 4, "values": [0.0, 0.0, 0.0]},
        {"id": 5, "values": [0.0, 0.0, 0.0]},
        {"id": 6, "values": [0.0, 0.0, 0.0]},
        {"id": 7, "values": [0.0, 0.0, 0.0]},
        {"id": 9, "values": [0.0, 0.0, 0.0]},
        {"id": 10, "values": [0.0, 0.0, 0.0]},
        {"id": 11, "values": [1000.0, 1000.0, 1000.0]},
        {"id": 12, "values": [0.0, 0.0, 0.0]},
        {"id": 13, "values": [0.0, 0.0, 0.0]},
        {"id": 14, "values": [0.0, 0.0, 0.0]},
        {"id": 15, "values": [0.0, 0.0, 0.0]},
    ]


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
    result = pine_compat.run_script(
        'strategy("demo")\nscale(value) => value * 10\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=2)\nplot(strategy.position_size[1])\nplot(strategy.openprofit[1])\nplot(scale(strategy.position_size))\n',
        BARS,
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, 0.0, 0.0],
        [None, 0.0, 0.0],
        [0.0, 0.0, 20.0],
    ]


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
    result = pine_compat.run_script(
        'strategy("demo")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=1)\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\nif bar_index == 2\n    strategy.close("L")\nplot(strategy.closedtrades)\nplot(strategy.opentrades)\n',
        BARS,
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 0, 0],
        [0, 0, 1],
        [0, 0, 0],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 0],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["orders"][0]["id"] == "L"
    assert result["strategy"]["trades"][0]["id"] == "L"


def test_run_script_returns_strategy_trade_count_fixture_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_trade_counts.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 0, 0, 1],
        [0, 0, 1, 0],
        [0, 0, 0, 1],
        [0, 0, 1, 0],
        [0, 0, 1, 1],
        [0, 0, 0, 0],
        [None, 0, 0, 1],
        [None, 0, 0, 0],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 3.0,
        }
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 2,
            "entryTime": 3,
            "exitTime": 3,
            "entryPrice": 3.0,
            "exitPrice": 3.0,
            "qty": 1.0,
            "profit": 0.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 1.0, "avgPrice": 3.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_strategy_exit_trade_count_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_trade_counts.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 0, 0, 1],
        [0, 1, 1, 0],
        [None, 0, 0, 0],
        [None, 0, 1, 1],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
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
            "id": "XL",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 2.5,
        },
    ]
    assert result["strategy"]["trades"][0]["profit"] == 0.5
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_strategy_closedtrades_field_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_closedtrades_fields.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, None, 2.0, 2.0],
        [0, 0, 1, 1],
        [None, None, 3.0, 3.0],
        [0, 0, 1, 1],
        [None, None, 1, 1],
        [None, None, 2, 2],
        [None, None, 2, 2],
        [None, None, 3, 3],
        [None, None, 0.0, 0.0],
        [None, None, 2.0, 2.0],
        [None, None, 2.0, 2.0],
        [None, None, 2.0, 2.0],
        [None, None, 0.0, 0.0],
        [1, 1, 1, 1],
        [1, 1, 1, 1],
        [None, None, None, None],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["trades"][0]["entryPrice"] == 2.0
    assert result["strategy"]["trades"][0]["exitPrice"] == 3.0
    assert result["strategy"]["trades"][0]["entryTime"] == 2
    assert result["strategy"]["trades"][0]["exitTime"] == 3
    assert result["strategy"]["trades"][0]["qty"] == 2.0
    assert result["strategy"]["trades"][0]["profit"] == 2.0
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_opentrades_fields_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [None, 2.0, None],
        [None, 1, None],
        [None, 1, None],
        [None, 20, None],
        [None, 2.0, None],
        [None, 0.0, None],
        [None, 0.0, None],
        [None, 4.0, None],
        [None, 2.0, None],
        [None, None, None],
    ] + [[None, None, None] for _ in range(27)]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["trades"][0]["entryPrice"] == 2.0
    assert result["strategy"]["trades"][0]["exitPrice"] == 3.0
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_margin_capital_held_plot():
    source = (ROOT / "tests/fixtures/runtime/strategy_margin_capital_held_long.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [[0.0, 2.0, 3.0, 0.0]]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["trades"][0]["entryPrice"] == 2.0
    assert result["strategy"]["trades"][0]["exitPrice"] == 4.0
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_margin_entry_affordability_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_margin_entry_affordability_long.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0, 4.0],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["orders"] == [
        {
            "id": "covered-market",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 4.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 3, "size": 1.0, "avgPrice": 4.0}
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 3,
        "cash": 0.0,
        "marketValue": 4.0,
        "equity": 4.0,
        "netProfit": 0.0,
    }
    assert result["strategy"]["diagnostics"] == [
        {
            "code": "E_STRATEGY_MARGIN",
            "message": "`strategy.entry` requires more margin than available equity",
        },
        {
            "code": "E_STRATEGY_MARGIN",
            "message": "`strategy.entry` requires more margin than available equity",
        },
    ]
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_margin_call_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_margin_call_long.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_margin_call_long_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 48.0, 48.0],
        [0.0, 36.0, 36.0],
        [0, 1, 1],
    ]
    assert set(result["strategy"]) == set(EMPTY_STRATEGY_RESULT)
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 100.0,
            "price": 4.0,
        },
        {
            "id": "Margin Call",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.short",
            "qty": 52.0,
            "price": 3.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 4.0,
            "exitPrice": 3.0,
            "qty": 52.0,
            "profit": -52.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 100.0, "avgPrice": 4.0},
        {"barIndex": 1, "size": 48.0, "avgPrice": 4.0},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 2,
        "cash": -79.0,
        "marketValue": 144.0,
        "equity": 65.0,
        "netProfit": -100.0,
    }
    assert result["strategy"]["diagnostics"] == []
    assert "closedTrades" not in result["strategy"]
    assert "openTrades" not in result["strategy"]


def test_run_script_returns_strategy_trade_outcome_count_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_trade_outcome_counts.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0, 0, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 1, 1, 1, 2, 2, 2, 3],
        [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 2.0],
        [None, None, 1.0, 1.0, 1.0, -0.5, -0.5, -0.5, -1.0 / 3.0],
        [None, None, 50.0, 50.0, 50.0, 0.0, 0.0, 0.0, 0.0],
        [None, None, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [None, None, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0],
        [None, None, None, None, None, 2.0, 2.0, 2.0, 2.0],
        [None, None, None, None, None, 50.0, 50.0, 50.0, 50.0],
    ]
    assert [trade["profit"] for trade in result["strategy"]["trades"]] == [1.0, -2.0, 0.0]
    assert "winTrades" not in result["strategy"]
    assert "lossTrades" not in result["strategy"]
    assert "evenTrades" not in result["strategy"]


def test_run_script_returns_strategy_profit_percent_plots():
    source = (ROOT / "tests/fixtures/runtime/strategy_profit_percent_state.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_trade_outcome_counts_bars.csv"),
    )

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0],
        [0.0, 0.0, 0.1, 0.1, 0.1, -0.1, -0.1, -0.1, -0.1],
        [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
        [0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 2.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2, 0.2, 0.2],
    ]
    assert "profitPercent" not in result["strategy"]


def test_run_script_returns_strategy_close_trade_contract():
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 1\n    strategy.entry("L", strategy.long, qty=2)\nif bar_index == 2\n    strategy.close("L")\n',
        BARS,
    )

    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 3.0,
            "exitPrice": 3.0,
            "qty": 2.0,
            "profit": 0.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 2, "size": 2.0, "avgPrice": 3.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 2,
        "cash": 100000.0,
        "marketValue": 0.0,
        "equity": 100000.0,
        "netProfit": 0.0,
    }


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
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 1.25, 1.25],
        [0.0, 0.0, 0.75, 0.75],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0, 1.0],
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 0.75,
            "profit": 0.75,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 1.25, "avgPrice": 2.0},
    ]
    assert "pending" not in result["strategy"]


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
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 4.0, 3.0, 2.0],
        [0.0, 0.0, 1.0, 3.0],
        [0.0, 0.0, 1.0, 2.0],
        [0.0, 1.0, 1.0, 1.0],
    ]
    assert result["strategy"]["trades"] == [
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
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 3,
            "entryTime": 2,
            "exitTime": 4,
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 4.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 3.0, "avgPrice": 2.0},
        {"barIndex": 3, "size": 2.0, "avgPrice": 2.0},
    ]
    assert "qty_percent" not in result["strategy"]
    assert "pending" not in result["strategy"]


def test_run_script_returns_strategy_close_all_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_close_all.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 0.0],
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
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 3.0,
            "qty": 2.0,
            "profit": 2.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert "closeAll" not in result["strategy"]
    assert "pending" not in result["strategy"]


def test_run_script_returns_strategy_cancel_entry_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_cancel_entry.pine").read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 0.0, 0.0],
        [None, None, None, None],
    ]
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
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "cancel" not in result["strategy"]


def test_run_script_returns_strategy_cancel_all_entry_exit_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"
    ).read_text()
    result = pine_compat.run_script(source, fixture_bars("tests/fixtures/runtime/bars.csv"))

    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 0.0, 0.0, 0.0],
        [0, 0, 0, 0],
    ]
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "cancel" not in result["strategy"]


def test_run_script_returns_strategy_exit_stop_trade_contract():
    bars = [
        {"time": 10, "open": 10.0, "high": 10.0, "low": 10.0, "close": 10.0, "volume": 1.0},
        {"time": 20, "open": 11.0, "high": 12.0, "low": 8.0, "close": 11.0, "volume": 1.0},
    ]
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 0\n    strategy.entry("L", strategy.long, qty=2)\n    strategy.exit("XL", "L", stop=9)\n',
        bars,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 20,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 11.0,
        },
        {
            "id": "XL",
            "barIndex": 1,
            "time": 20,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 9.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 20,
            "exitTime": 20,
            "entryPrice": 11.0,
            "exitPrice": 9.0,
            "qty": 2.0,
            "profit": -4.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 11.0},
        {"barIndex": 1, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 1,
        "cash": 99996.0,
        "marketValue": 0.0,
        "equity": 99996.0,
        "netProfit": -4.0,
    }


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
    bars = [
        {"time": 10, "open": 10.0, "high": 10.0, "low": 10.0, "close": 10.0, "volume": 1.0},
        {"time": 20, "open": 11.0, "high": 12.0, "low": 10.0, "close": 11.0, "volume": 1.0},
    ]
    result = pine_compat.run_script(
        'strategy("demo")\nif bar_index == 0\n    strategy.entry("L", strategy.long, qty=2)\n    strategy.exit("XL", "L", limit=12)\n',
        bars,
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 20,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 11.0,
        },
        {
            "id": "XL",
            "barIndex": 1,
            "time": 20,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 12.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 20,
            "exitTime": 20,
            "entryPrice": 11.0,
            "exitPrice": 12.0,
            "qty": 2.0,
            "profit": 2.0,
        }
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 1,
        "cash": 100002.0,
        "marketValue": 0.0,
        "equity": 100002.0,
        "netProfit": 2.0,
    }


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

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
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 3, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 3,
        "cash": 100003.0,
        "marketValue": 0.0,
        "equity": 100003.0,
        "netProfit": 3.0,
    }


def test_run_script_returns_strategy_exit_loss_trade_contract():
    source = (ROOT / "tests/fixtures/runtime/strategy_exit_loss.pine").read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_loss_bars.csv"),
    )

    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 10.0,
        },
        {
            "id": "XL",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 9.0,
        },
    ]
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 10.0,
            "exitPrice": 9.0,
            "qty": 2.0,
            "profit": -2.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 10.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 3,
        "cash": 99998.0,
        "marketValue": 0.0,
        "equity": 99998.0,
        "netProfit": -2.0,
    }


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_bracket_both_hit_bars.csv"),
    )

    assert result["plots"][0]["values"] == [0.0, 2.0, 0.0, 0.0]
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 100.0,
        },
        {
            "id": "XB",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 95.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 1
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 100.0,
            "exitPrice": 95.0,
            "qty": 2.0,
            "profit": -10.0,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 100.0},
        {"barIndex": 1, "size": 0.0, "avgPrice": None},
    ]
    assert result["strategy"]["equity"][-1] == {
        "barIndex": 3,
        "cash": 99990.0,
        "marketValue": 0.0,
        "equity": 99990.0,
        "netProfit": -10.0,
    }


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/strategy_exit_trailing_bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 1
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
        }
    ]
    assert result["strategy"]["diagnostics"] == []


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
            "id": "XQ",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 0.75,
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
            "qty": 0.75,
            "profit": 0.375,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 1, "size": 1.25, "avgPrice": 2.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "remainingQty" not in result["strategy"]


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
            "id": "XQ",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 0.75,
            "price": 3.0,
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
            "exitPrice": 3.0,
            "qty": 0.75,
            "profit": 0.75,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 1.25, "avgPrice": 2.0},
    ]
    assert result["strategy"]["diagnostics"] == []
    assert "qtyPercent" not in result["strategy"]
    assert "qty_percent" not in result["strategy"]
    assert "pending" not in result["strategy"]


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 2.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 1
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 1,
            "entryTime": 2,
            "exitTime": 2,
            "entryPrice": 2.0,
            "exitPrice": 2.5,
            "qty": 1.0,
            "profit": 0.5,
        }
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 1.0, 1.0],
        [0.0, 0.0, 0.5, 0.5],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "remainingQty" not in result["strategy"]
    assert "qtyPercent" not in result["strategy"]
    assert "qty_percent" not in result["strategy"]


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
            "id": "XS",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.exit",
            "qty": 0.5,
            "price": 2.5,
        },
        {
            "id": "XL",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 1.5,
            "price": 1.5,
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
            "exitPrice": 2.5,
            "qty": 0.5,
            "profit": 0.25,
        },
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 2,
            "entryTime": 2,
            "exitTime": 3,
            "entryPrice": 2.0,
            "exitPrice": 1.5,
            "qty": 1.5,
            "profit": -0.75,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0},
        {"barIndex": 1, "size": 1.5, "avgPrice": 2.0},
        {"barIndex": 2, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 2.0, 1.5, 0.0],
        [0.0, 0.0, 0.25, -0.5],
        [0.0, 0.0, 1.0, 2.0],
        [0.0, 1.0, 1.0, 0.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    assert "pending" not in result["strategy"]
    assert "remainingQty" not in result["strategy"]
    assert "qtyPercent" not in result["strategy"]
    assert "qty_percent" not in result["strategy"]


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
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
            "id": "XFULL",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.exit",
            "qty": 2.0,
            "price": 2.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 1
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
    assert "remainingQuantity" not in strategy_json
    assert "remaining_quantity" not in strategy_json
    assert "remainingQty" not in strategy_json
    assert "qtyPercent" not in strategy_json
    assert "qty_percent" not in strategy_json
    assert "triggerSide" not in strategy_json
    assert "activation" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_strategy_exit_active_entry_attachment_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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


def test_run_script_returns_strategy_exit_active_entry_profit_attachment_contract():
    source = (
        ROOT / "tests/fixtures/runtime/strategy_exit_active_entry_profit_attachment.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars("tests/fixtures/runtime/bars.csv"),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    assert result["schemaVersion"] == 3
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
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_from_entry_current_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
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
            "id": "L2",
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
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 2.0],
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
    assert "exitReason" not in strategy_json


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
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "id": "XP",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.0,
        },
        {
            "id": "XP",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 6.0,
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
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 4.0,
            "exitPrice": 6.0,
            "qty": 3.0,
            "profit": 6.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 3.5},
        {"barIndex": 3, "size": 3.0, "avgPrice": 4.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_loss_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XL",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 6.0,
        },
        {
            "id": "XL",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 4.0,
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
            "entryPrice": 8.0,
            "exitPrice": 6.0,
            "qty": 1.0,
            "profit": -2.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 4.0,
            "qty": 3.0,
            "profit": -6.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 3, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_loss_profit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 8.0,
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
            "entryPrice": 8.0,
            "exitPrice": 6.0,
            "qty": 1.0,
            "profit": -2.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 8.0,
            "qty": 3.0,
            "profit": 6.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 3, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


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
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 8.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 3,
            "entryTime": 3,
            "exitTime": 4,
            "entryPrice": 6.0,
            "exitPrice": 8.0,
            "qty": 3.0,
            "profit": 6.0,
        },
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 3, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_loss_limit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
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
            "entryPrice": 8.0,
            "exitPrice": 6.0,
            "qty": 1.0,
            "profit": -2.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 3, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_stop_limit_bracket_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 2,
            "time": 3,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 9.0,
        },
        {
            "id": "XB",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
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
            "entryPrice": 8.0,
            "exitPrice": 9.0,
            "qty": 1.0,
            "profit": 1.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 3,
            "entryTime": 3,
            "exitTime": 4,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 3, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 0.0],
        [0.0, 1.0, 4.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 2.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_trail_points_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "price": 3.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 3.5,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 3.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 1.0,
            "profit": 1.5,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 3.5,
            "qty": 3.0,
            "profit": 1.5,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 3.0, "avgPrice": 3.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_trail_price_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "price": 3.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 3.5,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 3.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 3.5,
            "qty": 1.0,
            "profit": 1.5,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 3.5,
            "qty": 3.0,
            "profit": 1.5,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
    ]
    assert result["diagnostics"] == []
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert "pending" not in strategy_json
    assert "reservation" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "closedTrades" not in strategy_json


def test_run_script_returns_omitted_profit_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 4.0,
        },
        {
            "id": "XP",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XP",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 7.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": 3.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 4.0,
            "exitPrice": 7.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 3.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 4.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L2",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XL",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XL",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 3.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 3.0,
            "qty": 3.0,
            "profit": -9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_profit_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_profit_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 4.0,
        },
        {
            "id": "XP",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XP",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 7.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": 3.0,
        },
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 4.0,
            "exitPrice": 7.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 3.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 4.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XL",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XL",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
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
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 3.0,
            "qty": 3.0,
            "profit": -9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_profit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L2",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_stop_profit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L2",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 4,
            "entryTime": 4,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 5,
            "entryTime": 2,
            "exitTime": 6,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_profit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_profit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_stop_profit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_profit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 4,
            "entryTime": 4,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 5,
            "entryTime": 2,
            "exitTime": 6,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_limit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_stop_limit_bracket_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 9.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 9.0,
            "qty": 1.0,
            "profit": 1.0,
        },
        {
            "id": "L",
            "entryBarIndex": 3,
            "exitBarIndex": 4,
            "entryTime": 4,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_trail_price_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "price": 3.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.5,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 4.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 4.5,
            "qty": 1.0,
            "profit": 2.5,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 4.5,
            "qty": 3.0,
            "profit": 4.5,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
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
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_trail_points_persistent_same_id_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_same_id_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "price": 3.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 4.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        },
        {
            "id": "L",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 4.0,
            "qty": 3.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 3.0, "avgPrice": 3.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
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
    assert "targetTradeKey" not in strategy_json
    assert "target_trade_key" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_loss_limit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_loss_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L2",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 5.0,
        },
        {
            "id": "XB",
            "barIndex": 5,
            "time": 6,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 5.0,
            "qty": 1.0,
            "profit": -3.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 5,
            "entryTime": 4,
            "exitTime": 6,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 3.0, "avgPrice": 6.0},
        {"barIndex": 5, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_stop_limit_bracket_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_stop_limit_bracket_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
    assert result["strategy"]["orders"] == [
        {
            "id": "L1",
            "barIndex": 1,
            "time": 2,
            "direction": "strategy.long",
            "qty": 1.0,
            "price": 8.0,
        },
        {
            "id": "L2",
            "barIndex": 3,
            "time": 4,
            "direction": "strategy.long",
            "qty": 3.0,
            "price": 6.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 9.0,
        },
        {
            "id": "XB",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 9.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 8.0,
            "exitPrice": 9.0,
            "qty": 1.0,
            "profit": 1.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 3,
            "exitBarIndex": 4,
            "entryTime": 4,
            "exitTime": 5,
            "entryPrice": 6.0,
            "exitPrice": 9.0,
            "qty": 3.0,
            "profit": 9.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 8.0},
        {"barIndex": 3, "size": 4.0, "avgPrice": 6.5},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 1.0, 2.0, 2.0, 0.0],
        [0.0, 1.0, 1.0, 4.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 2.0],
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
    assert "profitTarget" not in strategy_json
    assert "stopLoss" not in strategy_json
    assert "stop_price" not in strategy_json
    assert "exitReason" not in strategy_json


def test_run_script_returns_omitted_trail_price_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_price_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.5,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 4.5,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 4.5,
            "qty": 1.0,
            "profit": 2.5,
        },
        {
            "id": "L2",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 4.5,
            "qty": 3.0,
            "profit": 4.5,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
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


def test_run_script_returns_omitted_trail_points_persistent_fixture_contract():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        fixture_bars(
            "tests/fixtures/runtime/strategy_pyramiding_exit_omitted_trail_points_persistent_from_entries_bars.csv"
        ),
    )

    assert set(result.keys()) == STRATEGY_RUNTIME_RESULT_KEYS
    assert result["schemaVersion"] == 3
    assert set(result["strategy"].keys()) == set(EMPTY_STRATEGY_RESULT.keys())
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
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 1.0,
            "price": 4.0,
        },
        {
            "id": "XT",
            "barIndex": 4,
            "time": 5,
            "direction": "strategy.exit",
            "qty": 3.0,
            "price": 4.0,
        },
    ]
    assert [order["direction"] for order in result["strategy"]["orders"]].count(
        "strategy.exit"
    ) == 2
    assert result["strategy"]["trades"] == [
        {
            "id": "L1",
            "entryBarIndex": 1,
            "exitBarIndex": 4,
            "entryTime": 2,
            "exitTime": 5,
            "entryPrice": 2.0,
            "exitPrice": 4.0,
            "qty": 1.0,
            "profit": 2.0,
        },
        {
            "id": "L2",
            "entryBarIndex": 2,
            "exitBarIndex": 4,
            "entryTime": 3,
            "exitTime": 5,
            "entryPrice": 3.0,
            "exitPrice": 4.0,
            "qty": 3.0,
            "profit": 3.0,
        },
    ]
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 1.0, "avgPrice": 2.0},
        {"barIndex": 2, "size": 4.0, "avgPrice": 2.75},
        {"barIndex": 4, "size": 3.0, "avgPrice": 3.0},
        {"barIndex": 4, "size": 0.0, "avgPrice": None},
    ]
    assert [plot["values"] for plot in result["plots"]] == [
        [0.0, 1.0, 2.0, 2.0, 2.0],
        [0.0, 1.0, 4.0, 4.0, 4.0],
        [0.0, 0.0, 0.0, 0.0, 0.0],
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
    assert "activation" not in strategy_json
    assert "exitReason" not in strategy_json


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
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_limit_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_limit_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_profit_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_profit_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_bracket_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_stop_profit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_stop_profit_bracket_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_limit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_limit_bracket_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_profit_bracket_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_profit_bracket_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_trailing_while_flat_fixture_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_trailing_while_flat_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_limit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_limit_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_profit_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_profit_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_stop_profit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_stop_profit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_limit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_limit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_loss_profit_bracket_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_loss_profit_bracket_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


def test_run_script_treats_strategy_exit_trailing_wrong_entry_as_noop():
    source = (
        ROOT
        / "tests/fixtures/runtime/strategy_exit_trailing_unmatched_from_entry_noop.pine"
    ).read_text()
    result = pine_compat.run_script(
        source,
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == [
        {
            "id": "L",
            "barIndex": 1,
            "time": 1,
            "direction": "strategy.long",
            "qty": 2.0,
            "price": 2.0,
        }
    ]
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == [
        {"barIndex": 1, "size": 2.0, "avgPrice": 2.0}
    ]
    assert result["strategy"]["diagnostics"] == []
    strategy_json = json.dumps(result["strategy"])
    assert '"direction": "strategy.exit"' not in strategy_json
    assert "pending" not in strategy_json
    assert "reserved" not in strategy_json


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

    assert result["schemaVersion"] == 3
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
                    "color": None,
                    "style": "label.style_label_down",
                    "textColor": None,
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
    assert snapshots[2]["color"] == 0x008000


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
                    "color": None,
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
                    "color": 0x008000,
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
                    "color": None,
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
                    "color": None,
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
                    "color": None,
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
                    "color": 0x008000,
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
                    "color": 0x008000,
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
                    "bgColor": None,
                    "borderColor": None,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": None,
                    "textSize": "size.normal",
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
                    "bgColor": 0x008000,
                    "borderColor": None,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": None,
                    "textSize": "size.normal",
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
                    "bgColor": None,
                    "borderColor": None,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": None,
                    "textSize": "size.normal",
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
                    "bgColor": None,
                    "borderColor": None,
                    "borderWidth": 1,
                    "borderStyle": "line.style_solid",
                    "extend": "extend.none",
                    "text": "",
                    "textColor": None,
                    "textSize": "size.normal",
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
    assert result["tables"][0]["bgColor"] == 0x008000


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
                    "bgColor": 0x008000,
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
                    assert cell.pop("tooltip") == ""
                    assert cell.pop("textFontFamily") == "font.family_default"
                    assert cell.pop("textFormatting") == 0
    return tables


def test_run_script_returns_table_outputs():
    result = pine_compat.run_script(
        'indicator("tables")\nif bar_index == 1\n    table_id = table.new(position.top_right, 2, 2)\n    table.cell(table_id, 0, 0, "A", bgcolor=color.green, text_color=color.white)\n    table.cell_set_text(table_id, 0, 0, "B")\n    table.cell_set_bgcolor(table_id, 0, 0, color.red)\n    table.cell_set_text_color(table_id, 0, 0, color.blue)\n    table.cell_set_width(table_id, 0, 0, 25)\n    table.cell_set_height(table_id, 0, 0, 40)\n    table.cell_set_text_size(table_id, 0, 0, size.small)\n    table.cell_set_text_halign(table_id, 0, 0, text.align_left)\n    table.cell_set_text_valign(table_id, 0, 0, text.align_top)\n    table.set_position(table_id, position.bottom_right)\n    table.set_bgcolor(table_id, color.yellow)\n    table.set_frame_color(table_id, color.black)\n    table.set_frame_width(table_id, 3)\n    table.set_border_color(table_id, color.white)\n    table.set_border_width(table_id, 4)\nplot(close)\n',
        BARS,
    )

    assert table_snapshots_without_empty_merges(result["tables"]) == [
        {
            "id": 1,
            "position": "position.bottom_right",
            "bgColor": 0xFFFF00,
            "frameColor": 0,
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
                            "bgColor": 0x008000,
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
                            "bgColor": 0x008000,
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
                            "bgColor": 0xFF0000,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
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
                            "bgColor": 0xFF0000,
                            "textColor": 0x0000FF,
                            "width": 25,
                            "height": 40,
                            "textSize": "size.small",
                            "textHalign": "text.align_left",
                            "textValign": "text.align_top",
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
                            "bgColor": 0x008000,
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
    assert result["plotChars"][0]["colors"] == [0x008000, 0x008000, 0x008000]


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
    assert result["plotShapes"][0]["colors"] == [0x008000, 0x008000, 0x008000]
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
    assert result["plotArrows"][0]["colorUps"] == [0x008000, 0x008000, 0x008000]
    assert result["plotArrows"][0]["colorDowns"] == [0xFF0000, 0xFF0000, 0xFF0000]
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
    assert result["plotBars"][0]["colors"] == [0x008000, 0x008000, 0x008000]


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
    assert result["plotCandles"][0]["colors"] == [0x008000, 0x008000, 0x008000]
    assert result["plotCandles"][0]["wickColors"] == [0xFFFFFF, 0xFFFFFF, 0xFFFFFF]
    assert result["plotCandles"][0]["borderColors"] == [0xFF0000, 0xFF0000, 0xFF0000]
