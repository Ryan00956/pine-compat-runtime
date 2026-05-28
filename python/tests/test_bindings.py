import pine_compat
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


def test_analyze_script_accepts_library_sources_without_enabling_imports():
    report = pine_compat.analyze_script(
        'indicator("root")\nplot(close)\n',
        library_sources={"user/lib/1": 'library("lib")\n'},
    )

    assert report["executable"] is True
    assert report["diagnostics"] == []


def test_compile_script_keeps_import_unsupported_with_library_source():
    try:
        pine_compat.compile_script(
            'import user/lib/1\nindicator("root")\n',
            library_sources={"user/lib/1": 'library("lib")\n'},
        )
    except ValueError as error:
        assert "E_UNSUPPORTED_FEATURE" in str(error)
        assert "library imports are not supported in Phase 1" in str(error)
    else:
        raise AssertionError("import should remain unsupported")


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


def test_run_script_accepts_library_sources_without_enabling_imports():
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
                }
            ],
        }
    ]


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
                },
            ],
        }
    ]


def test_run_script_returns_table_outputs():
    result = pine_compat.run_script(
        'indicator("tables")\nif bar_index == 1\n    table_id = table.new(position.top_right, 2, 2)\n    table.cell(table_id, 0, 0, "A", bgcolor=color.green, text_color=color.white)\nplot(close)\n',
        BARS,
    )

    assert result["tables"] == [
        {
            "id": 1,
            "position": "position.top_right",
            "columns": 2,
            "rows": 2,
            "snapshots": [
                {
                    "barIndex": 1,
                    "cells": [],
                },
                {
                    "barIndex": 1,
                    "cells": [
                        {
                            "column": 0,
                            "row": 0,
                            "text": "A",
                            "bgColor": 0x008000,
                            "textColor": 0xFFFFFF,
                        }
                    ],
                },
            ],
        }
    ]


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
