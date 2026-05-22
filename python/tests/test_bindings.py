import pine_compat


BARS = [
    {"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0},
    {"time": 1, "open": 2.0, "high": 2.0, "low": 2.0, "close": 2.0, "volume": 1.0},
    {"time": 2, "open": 3.0, "high": 3.0, "low": 3.0, "close": 3.0, "volume": 1.0},
]


def test_analyze_script_reports_executable_script():
    report = pine_compat.analyze_script('indicator("demo")\nplot(close)\n')

    assert report["schemaVersion"] == 1
    assert report["executable"] is True
    assert report["diagnostics"] == []
    assert any(
        feature["feature"] == "plot"
        for feature in report["compatibility"]["supported"]
    )


def test_compile_script_returns_program_with_run_method():
    program = pine_compat.compile_script('indicator("demo")\nplot(close)\n')
    result = program.run(BARS)

    assert result["schemaVersion"] == 1
    assert result["plots"][0]["values"] == [1.0, 2.0, 3.0]
    assert result["diagnostics"] == []


def test_run_script_compiles_and_executes():
    result = pine_compat.run_script(
        'indicator("math")\nplot(math.max(close, 2))\n',
        BARS,
    )

    assert result["schemaVersion"] == 1
    assert result["plots"][0]["values"] == [2, 2, 3]


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
    assert result["plotCandles"][0]["highs"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["lows"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["closes"] == [1.0, 2.0, 3.0]
    assert result["plotCandles"][0]["colors"] == [0x008000, 0x008000, 0x008000]
    assert result["plotCandles"][0]["wickColors"] == [0xFFFFFF, 0xFFFFFF, 0xFFFFFF]
    assert result["plotCandles"][0]["borderColors"] == [0xFF0000, 0xFF0000, 0xFF0000]
