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


def test_run_script_returns_strategy_exit_missing_entry_diagnostics():
    result = pine_compat.run_script(
        'strategy("exit")\nif bar_index == 0\n    strategy.exit("XL", "L", stop=low)\n',
        BARS,
    )

    assert result["diagnostics"] == []
    assert result["strategy"]["orders"] == []
    assert result["strategy"]["trades"] == []
    assert result["strategy"]["position"] == []
    assert result["strategy"]["equity"] == FLAT_EQUITY
    assert result["strategy"]["diagnostics"] == [
        {
            "code": "E_STRATEGY_EXIT_ENTRY",
            "message": "`strategy.exit` from_entry must match the current long entry",
        }
    ]


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
