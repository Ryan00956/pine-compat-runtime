use pine_ir::HirProgram;
use pine_runtime::{
    Bar, ChartContext, InMemoryRequestDataProvider, PUBLIC_ANALYSIS_SCHEMA_VERSION,
    PUBLIC_RUNTIME_SCHEMA_VERSION, PineValue, RequestEnvironment, RequestKey, RequestTimeframe,
    run_historical_with_request_environment,
};
use pine_sema::{Analysis, AnalysisInput, analyze_input};
use pine_syntax::{Diagnostic, Severity, SourceFile, Span};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule, PySequence};

#[pyclass(name = "Program", skip_from_py_object)]
#[derive(Clone)]
struct PyProgram {
    hir: HirProgram,
}

#[pymethods]
impl PyProgram {
    #[pyo3(signature = (bars, request_bars=None))]
    fn run(
        &self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        request_bars: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let bars = parse_bars(bars)?;
        let request_environment = parse_request_environment(request_bars)?;
        let result = run_historical_with_request_environment(&self.hir, &bars, request_environment)
            .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_to_py(py, &result)
    }
}

#[pyfunction(signature = (source, library_sources=None))]
fn compile_script(source: &str, library_sources: Option<&Bound<'_, PyAny>>) -> PyResult<PyProgram> {
    let input = analysis_input_from_python(source, library_sources)?;
    let source_file = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        return Err(PyValueError::new_err(format_diagnostics(
            &source_file,
            &analysis.diagnostics,
        )));
    }

    let hir = analysis
        .hir
        .ok_or_else(|| PyValueError::new_err("analysis did not produce executable HIR"))?;
    Ok(PyProgram { hir })
}

#[pyfunction(signature = (source, library_sources=None))]
fn analyze_script(
    py: Python<'_>,
    source: &str,
    library_sources: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let input = analysis_input_from_python(source, library_sources)?;
    let source_file = input.root().clone();
    let analysis = analyze_input(&input);
    analysis_to_py(py, &source_file, &analysis)
}

#[pyfunction(signature = (source, bars, request_bars=None, library_sources=None))]
fn run_script(
    py: Python<'_>,
    source: &str,
    bars: &Bound<'_, PyAny>,
    request_bars: Option<&Bound<'_, PyAny>>,
    library_sources: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let program = compile_script(source, library_sources)?;
    program.run(py, bars, request_bars)
}

#[pymodule]
fn pine_compat(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyProgram>()?;
    module.add_function(wrap_pyfunction!(compile_script, module)?)?;
    module.add_function(wrap_pyfunction!(analyze_script, module)?)?;
    module.add_function(wrap_pyfunction!(run_script, module)?)?;
    Ok(())
}

fn parse_bars(bars: &Bound<'_, PyAny>) -> PyResult<Vec<Bar>> {
    let mut parsed = Vec::new();
    for item in bars.try_iter()? {
        let item = item?;
        parsed.push(parse_bar(&item)?);
    }
    Ok(parsed)
}

fn analysis_input_from_python(
    source: &str,
    library_sources: Option<&Bound<'_, PyAny>>,
) -> PyResult<AnalysisInput> {
    let root = SourceFile::new("<python>", source);
    let Some(library_sources) = library_sources else {
        return Ok(AnalysisInput::new(root));
    };
    let dict = library_sources.cast::<PyDict>().map_err(|_| {
        PyValueError::new_err("library_sources must be a dict mapping import key to source text")
    })?;
    let mut sources = Vec::with_capacity(dict.len());
    for (key, value) in dict {
        let key: String = key.extract()?;
        let text: String = value.extract().map_err(|_| {
            PyValueError::new_err("library_sources values must be source text strings")
        })?;
        sources.push((
            key.clone(),
            SourceFile::new(format!("<python:{key}>"), text),
        ));
    }
    AnalysisInput::with_library_sources(root, sources)
        .map_err(|err| PyValueError::new_err(err.to_string()))
}

fn parse_request_environment(
    request_bars: Option<&Bound<'_, PyAny>>,
) -> PyResult<RequestEnvironment> {
    let Some(request_bars) = request_bars else {
        return Ok(RequestEnvironment::default());
    };
    let dict = request_bars.cast::<PyDict>().map_err(|_| {
        PyValueError::new_err("request_bars must be a dict mapping SYMBOL:TIMEFRAME to bars")
    })?;
    let mut streams = Vec::with_capacity(dict.len());
    for (key, value) in dict {
        let key: String = key.extract()?;
        let request_key = parse_request_key(&key)?;
        streams.push((request_key, parse_bars(&value)?));
    }
    let provider = InMemoryRequestDataProvider::from_streams(streams)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(RequestEnvironment::new(
        ChartContext::default(),
        std::sync::Arc::new(provider),
    ))
}

fn parse_request_key(key: &str) -> PyResult<RequestKey> {
    let Some((symbol, timeframe)) = key.rsplit_once(':') else {
        return Err(PyValueError::new_err(
            "request_bars keys must use SYMBOL:TIMEFRAME",
        ));
    };
    if symbol.trim().is_empty() {
        return Err(PyValueError::new_err(
            "request_bars symbol must not be empty",
        ));
    }
    let timeframe =
        RequestTimeframe::parse(timeframe).map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(RequestKey::new(symbol.trim(), timeframe))
}

fn parse_bar(item: &Bound<'_, PyAny>) -> PyResult<Bar> {
    if let Ok(dict) = item.cast::<PyDict>() {
        return Ok(Bar {
            time: dict_number(dict, "time")?,
            open: dict_number(dict, "open")?,
            high: dict_number(dict, "high")?,
            low: dict_number(dict, "low")?,
            close: dict_number(dict, "close")?,
            volume: dict_number(dict, "volume")?,
        });
    }

    if let Ok(sequence) = item.cast::<PySequence>() {
        if sequence.len()? != 6 {
            return Err(PyValueError::new_err(
                "bar sequences must contain time, open, high, low, close, volume",
            ));
        }
        return Ok(Bar {
            time: sequence.get_item(0)?.extract()?,
            open: sequence.get_item(1)?.extract()?,
            high: sequence.get_item(2)?.extract()?,
            low: sequence.get_item(3)?.extract()?,
            close: sequence.get_item(4)?.extract()?,
            volume: sequence.get_item(5)?.extract()?,
        });
    }

    Err(PyValueError::new_err(
        "bars must be dictionaries or 6-item sequences",
    ))
}

fn dict_number<T>(dict: &Bound<'_, PyDict>, name: &str) -> PyResult<T>
where
    T: for<'py> FromPyObject<'py, 'py, Error = PyErr>,
{
    dict.get_item(name)?
        .ok_or_else(|| PyValueError::new_err(format!("bar is missing `{name}`")))?
        .extract()
}

fn analysis_to_py(py: Python<'_>, source: &SourceFile, analysis: &Analysis) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("schemaVersion", PUBLIC_ANALYSIS_SCHEMA_VERSION)?;
    output.set_item("languageVersion", analysis.compatibility.language_version)?;
    output.set_item(
        "diagnostics",
        diagnostics_to_py(py, source, &analysis.diagnostics)?,
    )?;
    output.set_item("compatibility", compatibility_to_py(py, source, analysis)?)?;
    output.set_item("executable", analysis.hir.is_some())?;
    Ok(output.into_any().unbind())
}

fn compatibility_to_py(
    py: Python<'_>,
    source: &SourceFile,
    analysis: &Analysis,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    let supported = PyList::empty(py);
    for feature in &analysis.compatibility.supported {
        let item = PyDict::new(py);
        item.set_item("feature", &feature.feature)?;
        item.set_item("span", span_to_py(py, source, feature.span)?)?;
        supported.append(item)?;
    }

    let unsupported = PyList::empty(py);
    for feature in &analysis.compatibility.unsupported {
        let item = PyDict::new(py);
        item.set_item("feature", &feature.feature)?;
        item.set_item("reason", &feature.reason)?;
        item.set_item("span", span_to_py(py, source, feature.span)?)?;
        unsupported.append(item)?;
    }

    output.set_item("supported", supported)?;
    output.set_item("unsupported", unsupported)?;
    Ok(output.into_any().unbind())
}

fn diagnostics_to_py(
    py: Python<'_>,
    source: &SourceFile,
    diagnostics: &[Diagnostic],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for diagnostic in diagnostics {
        let item = PyDict::new(py);
        item.set_item("code", &diagnostic.code)?;
        item.set_item("severity", severity_name(diagnostic.severity))?;
        item.set_item("message", &diagnostic.message)?;
        item.set_item("span", span_to_py(py, source, diagnostic.span)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn span_to_py(py: Python<'_>, source: &SourceFile, span: Span) -> PyResult<Py<PyAny>> {
    let line_col = source.line_col(span.start);
    let output = PyDict::new(py);
    output.set_item("start", span.start)?;
    output.set_item("end", span.end)?;
    output.set_item("line", line_col.line)?;
    output.set_item("column", line_col.column)?;
    Ok(output.into_any().unbind())
}

fn runtime_result_to_py(
    py: Python<'_>,
    result: &pine_runtime::RuntimeResult,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("schemaVersion", PUBLIC_RUNTIME_SCHEMA_VERSION)?;
    output.set_item("plots", plots_to_py(py, &result.plots)?)?;
    output.set_item("plotChars", plot_chars_to_py(py, &result.plot_chars)?)?;
    output.set_item("plotShapes", plot_shapes_to_py(py, &result.plot_shapes)?)?;
    output.set_item("plotArrows", plot_arrows_to_py(py, &result.plot_arrows)?)?;
    output.set_item("plotBars", plot_bars_to_py(py, &result.plot_bars)?)?;
    output.set_item("plotCandles", plot_candles_to_py(py, &result.plot_candles)?)?;
    output.set_item("bgColors", colors_to_py(py, &result.bg_colors)?)?;
    output.set_item("barColors", colors_to_py(py, &result.bar_colors)?)?;
    output.set_item("hlines", hlines_to_py(py, &result.hlines)?)?;
    output.set_item("fills", fills_to_py(py, &result.fills)?)?;
    output.set_item("labels", labels_to_py(py, &result.labels)?)?;
    output.set_item("lines", lines_to_py(py, &result.lines)?)?;
    output.set_item("boxes", boxes_to_py(py, &result.boxes)?)?;
    output.set_item("tables", tables_to_py(py, &result.tables)?)?;
    output.set_item("alerts", alerts_to_py(py, &result.alerts)?)?;
    if let Some(strategy) = &result.strategy {
        output.set_item("strategy", strategy_result_to_py(py, strategy)?)?;
    }
    output.set_item("diagnostics", PyList::empty(py))?;
    Ok(output.into_any().unbind())
}

fn strategy_result_to_py(
    py: Python<'_>,
    strategy: &pine_runtime::StrategyResult,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("orders", strategy_orders_to_py(py, &strategy.orders)?)?;
    output.set_item("trades", strategy_trades_to_py(py, &strategy.trades)?)?;
    output.set_item("position", strategy_position_to_py(py, &strategy.position)?)?;
    output.set_item(
        "equity",
        empty_strategy_list_to_py(py, strategy.equity.len())?,
    )?;
    output.set_item(
        "diagnostics",
        empty_strategy_list_to_py(py, strategy.diagnostics.len())?,
    )?;
    Ok(output.into_any().unbind())
}

fn strategy_trades_to_py(
    py: Python<'_>,
    trades: &[pine_runtime::StrategyTrade],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for trade in trades {
        let item = PyDict::new(py);
        item.set_item("id", &trade.id)?;
        item.set_item("entryBarIndex", trade.entry_bar_index)?;
        item.set_item("exitBarIndex", trade.exit_bar_index)?;
        item.set_item("entryTime", trade.entry_time)?;
        item.set_item("exitTime", trade.exit_time)?;
        item.set_item("entryPrice", trade.entry_price)?;
        item.set_item("exitPrice", trade.exit_price)?;
        item.set_item("qty", trade.qty)?;
        item.set_item("profit", trade.profit)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn strategy_orders_to_py(
    py: Python<'_>,
    orders: &[pine_runtime::StrategyOrderEvent],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for order in orders {
        let item = PyDict::new(py);
        item.set_item("id", &order.id)?;
        item.set_item("barIndex", order.bar_index)?;
        item.set_item("time", order.time)?;
        item.set_item("direction", &order.direction)?;
        item.set_item("qty", order.qty)?;
        item.set_item("price", order.price)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn strategy_position_to_py(
    py: Python<'_>,
    position: &[pine_runtime::StrategyPositionSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in position {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("size", snapshot.size)?;
        item.set_item("avgPrice", snapshot.avg_price)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn empty_strategy_list_to_py(py: Python<'_>, len: usize) -> PyResult<Py<PyAny>> {
    debug_assert_eq!(len, 0);
    Ok(PyList::empty(py).into_any().unbind())
}

fn plots_to_py(py: Python<'_>, plots: &[pine_runtime::PlotSeries]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot in plots {
        let item = PyDict::new(py);
        item.set_item("id", plot.id)?;
        item.set_item("values", values_to_py(py, &plot.values)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn colors_to_py(py: Python<'_>, colors: &[pine_runtime::ColorSeries]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for colors in colors {
        let item = PyDict::new(py);
        item.set_item("id", colors.id)?;
        item.set_item("values", values_to_py(py, &colors.values)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_chars_to_py(
    py: Python<'_>,
    plot_chars: &[pine_runtime::PlotCharSeries],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_char in plot_chars {
        let item = PyDict::new(py);
        item.set_item("id", plot_char.id)?;
        item.set_item("values", values_to_py(py, &plot_char.values)?)?;
        item.set_item("chars", values_to_py(py, &plot_char.chars)?)?;
        item.set_item("colors", values_to_py(py, &plot_char.colors)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_shapes_to_py(
    py: Python<'_>,
    plot_shapes: &[pine_runtime::PlotShapeSeries],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_shape in plot_shapes {
        let item = PyDict::new(py);
        item.set_item("id", plot_shape.id)?;
        item.set_item("values", values_to_py(py, &plot_shape.values)?)?;
        item.set_item("styles", values_to_py(py, &plot_shape.styles)?)?;
        item.set_item("locations", values_to_py(py, &plot_shape.locations)?)?;
        item.set_item("colors", values_to_py(py, &plot_shape.colors)?)?;
        item.set_item("texts", values_to_py(py, &plot_shape.texts)?)?;
        item.set_item("textColors", values_to_py(py, &plot_shape.text_colors)?)?;
        item.set_item("sizes", values_to_py(py, &plot_shape.sizes)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_arrows_to_py(
    py: Python<'_>,
    plot_arrows: &[pine_runtime::PlotArrowSeries],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_arrow in plot_arrows {
        let item = PyDict::new(py);
        item.set_item("id", plot_arrow.id)?;
        item.set_item("values", values_to_py(py, &plot_arrow.values)?)?;
        item.set_item("colorUps", values_to_py(py, &plot_arrow.color_ups)?)?;
        item.set_item("colorDowns", values_to_py(py, &plot_arrow.color_downs)?)?;
        item.set_item("minHeights", values_to_py(py, &plot_arrow.min_heights)?)?;
        item.set_item("maxHeights", values_to_py(py, &plot_arrow.max_heights)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_bars_to_py(
    py: Python<'_>,
    plot_bars: &[pine_runtime::PlotBarSeries],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_bar in plot_bars {
        let item = PyDict::new(py);
        item.set_item("id", plot_bar.id)?;
        item.set_item("opens", values_to_py(py, &plot_bar.opens)?)?;
        item.set_item("highs", values_to_py(py, &plot_bar.highs)?)?;
        item.set_item("lows", values_to_py(py, &plot_bar.lows)?)?;
        item.set_item("closes", values_to_py(py, &plot_bar.closes)?)?;
        item.set_item("colors", values_to_py(py, &plot_bar.colors)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_candles_to_py(
    py: Python<'_>,
    plot_candles: &[pine_runtime::PlotCandleSeries],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_candle in plot_candles {
        let item = PyDict::new(py);
        item.set_item("id", plot_candle.id)?;
        item.set_item("opens", values_to_py(py, &plot_candle.opens)?)?;
        item.set_item("highs", values_to_py(py, &plot_candle.highs)?)?;
        item.set_item("lows", values_to_py(py, &plot_candle.lows)?)?;
        item.set_item("closes", values_to_py(py, &plot_candle.closes)?)?;
        item.set_item("colors", values_to_py(py, &plot_candle.colors)?)?;
        item.set_item("wickColors", values_to_py(py, &plot_candle.wick_colors)?)?;
        item.set_item(
            "borderColors",
            values_to_py(py, &plot_candle.border_colors)?,
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn hlines_to_py(py: Python<'_>, hlines: &[pine_runtime::HLineOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for hline in hlines {
        let item = PyDict::new(py);
        item.set_item("id", hline.id)?;
        item.set_item("price", value_to_py(py, &hline.price)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn fills_to_py(py: Python<'_>, fills: &[pine_runtime::FillOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for fill in fills {
        let item = PyDict::new(py);
        item.set_item("id", fill.id)?;
        item.set_item("firstId", fill.first_id)?;
        item.set_item("secondId", fill.second_id)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn labels_to_py(py: Python<'_>, labels: &[pine_runtime::LabelOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for label in labels {
        let item = PyDict::new(py);
        item.set_item("id", label.id)?;
        item.set_item("snapshots", label_snapshots_to_py(py, &label.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn label_snapshots_to_py(
    py: Python<'_>,
    snapshots: &[pine_runtime::LabelSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("x", value_to_py(py, &snapshot.x)?)?;
            item.set_item("y", value_to_py(py, &snapshot.y)?)?;
            item.set_item("text", value_to_py(py, &snapshot.text)?)?;
            item.set_item("xloc", value_to_py(py, &snapshot.xloc)?)?;
            item.set_item("yloc", value_to_py(py, &snapshot.yloc)?)?;
            item.set_item("color", value_to_py(py, &snapshot.color)?)?;
            item.set_item("style", value_to_py(py, &snapshot.style)?)?;
            item.set_item("textColor", value_to_py(py, &snapshot.text_color)?)?;
            item.set_item("size", value_to_py(py, &snapshot.size)?)?;
            item.set_item("tooltip", value_to_py(py, &snapshot.tooltip)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn lines_to_py(py: Python<'_>, lines: &[pine_runtime::LineOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for line in lines {
        let item = PyDict::new(py);
        item.set_item("id", line.id)?;
        item.set_item("snapshots", line_snapshots_to_py(py, &line.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn line_snapshots_to_py(
    py: Python<'_>,
    snapshots: &[pine_runtime::LineSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("x1", value_to_py(py, &snapshot.x1)?)?;
            item.set_item("y1", value_to_py(py, &snapshot.y1)?)?;
            item.set_item("x2", value_to_py(py, &snapshot.x2)?)?;
            item.set_item("y2", value_to_py(py, &snapshot.y2)?)?;
            item.set_item("color", value_to_py(py, &snapshot.color)?)?;
            item.set_item("width", value_to_py(py, &snapshot.width)?)?;
            item.set_item("style", value_to_py(py, &snapshot.style)?)?;
            item.set_item("extend", value_to_py(py, &snapshot.extend)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn boxes_to_py(py: Python<'_>, boxes: &[pine_runtime::BoxOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for box_output in boxes {
        let item = PyDict::new(py);
        item.set_item("id", box_output.id)?;
        item.set_item("snapshots", box_snapshots_to_py(py, &box_output.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn box_snapshots_to_py(
    py: Python<'_>,
    snapshots: &[pine_runtime::BoxSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("left", value_to_py(py, &snapshot.left)?)?;
            item.set_item("top", value_to_py(py, &snapshot.top)?)?;
            item.set_item("right", value_to_py(py, &snapshot.right)?)?;
            item.set_item("bottom", value_to_py(py, &snapshot.bottom)?)?;
            item.set_item("bgColor", value_to_py(py, &snapshot.bg_color)?)?;
            item.set_item("borderColor", value_to_py(py, &snapshot.border_color)?)?;
            item.set_item("borderWidth", value_to_py(py, &snapshot.border_width)?)?;
            item.set_item("borderStyle", value_to_py(py, &snapshot.border_style)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn tables_to_py(py: Python<'_>, tables: &[pine_runtime::TableOutput]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for table in tables {
        let item = PyDict::new(py);
        item.set_item("id", table.id)?;
        item.set_item("position", value_to_py(py, &table.position)?)?;
        item.set_item("columns", table.columns)?;
        item.set_item("rows", table.rows)?;
        item.set_item("snapshots", table_snapshots_to_py(py, &table.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn table_snapshots_to_py(
    py: Python<'_>,
    snapshots: &[pine_runtime::TableSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("cells", table_cells_to_py(py, &snapshot.cells)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn table_cells_to_py(
    py: Python<'_>,
    cells: &[pine_runtime::TableCellSnapshot],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for cell in cells {
        let item = PyDict::new(py);
        item.set_item("column", cell.column)?;
        item.set_item("row", cell.row)?;
        item.set_item("text", value_to_py(py, &cell.text)?)?;
        item.set_item("bgColor", value_to_py(py, &cell.bg_color)?)?;
        item.set_item("textColor", value_to_py(py, &cell.text_color)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn alerts_to_py(py: Python<'_>, alerts: &[pine_runtime::AlertEvent]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for alert in alerts {
        let item = PyDict::new(py);
        item.set_item("id", alert.id)?;
        item.set_item("barIndex", alert.bar_index)?;
        item.set_item("time", alert.time)?;
        item.set_item("message", &alert.message)?;
        item.set_item("source", &alert.source)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn values_to_py(py: Python<'_>, values: &[PineValue]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for value in values {
        output.append(value_to_py(py, value)?)?;
    }
    Ok(output.into_any().unbind())
}

fn value_to_py(py: Python<'_>, value: &PineValue) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    append_value(py, &output, value)?;
    Ok(output.get_item(0)?.unbind())
}

fn append_value(py: Python<'_>, output: &Bound<'_, PyList>, value: &PineValue) -> PyResult<()> {
    match value {
        PineValue::Int(value) => output.append(*value),
        PineValue::Float(value) => output.append(*value),
        PineValue::Bool(value) => output.append(*value),
        PineValue::String(value) => output.append(value),
        PineValue::Color(value)
        | PineValue::Plot(value)
        | PineValue::HLine(value)
        | PineValue::Label(value)
        | PineValue::Line(value)
        | PineValue::Box(value)
        | PineValue::Table(value) => output.append(*value),
        PineValue::UserType(values) | PineValue::Tuple(values) => {
            output.append(values_to_py(py, values)?)
        }
        PineValue::Array(_) | PineValue::Na | PineValue::Void => output.append(py.None()),
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn format_diagnostics(source: &SourceFile, diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let line_col = source.line_col(diagnostic.span.start);
            format!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
