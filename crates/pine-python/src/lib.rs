use pine_ir::HirProgram;
use pine_runtime::{
    Bar, ChartContext, InMemoryRequestDataProvider, PUBLIC_ANALYSIS_SCHEMA_VERSION,
    RequestEnvironment, RequestKey, RequestTimeframe, input_calls,
    run_historical_with_request_environment,
};
use pine_sema::{Analysis, AnalysisInput, analyze_input};
use pine_syntax::{Diagnostic, SourceFile, Span};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule, PySequence};
mod alerts;
mod diagnostics;
mod outputs;
mod tables;
#[cfg(test)]
mod tests;
use alerts::{render_strategy_order_fill_alert_template, render_strategy_order_fill_running_alert};
use diagnostics::{diagnostics_have_errors, format_diagnostics, severity_name};
use outputs::runtime_result_to_py;

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
    if diagnostics_have_errors(&analysis.diagnostics) {
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
    module.add_function(wrap_pyfunction!(
        render_strategy_order_fill_alert_template,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        render_strategy_order_fill_running_alert,
        module
    )?)?;
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
            time: dict_i64(dict, "time")?,
            open: dict_finite_f64(dict, "open")?,
            high: dict_finite_f64(dict, "high")?,
            low: dict_finite_f64(dict, "low")?,
            close: dict_finite_f64(dict, "close")?,
            volume: dict_finite_f64(dict, "volume")?,
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
            open: finite_bar_value(sequence.get_item(1)?.extract()?, "open")?,
            high: finite_bar_value(sequence.get_item(2)?.extract()?, "high")?,
            low: finite_bar_value(sequence.get_item(3)?.extract()?, "low")?,
            close: finite_bar_value(sequence.get_item(4)?.extract()?, "close")?,
            volume: finite_bar_value(sequence.get_item(5)?.extract()?, "volume")?,
        });
    }

    Err(PyValueError::new_err(
        "bars must be dictionaries or 6-item sequences",
    ))
}

fn dict_i64(dict: &Bound<'_, PyDict>, name: &str) -> PyResult<i64> {
    dict.get_item(name)?
        .ok_or_else(|| PyValueError::new_err(format!("bar is missing `{name}`")))?
        .extract()
}

fn dict_finite_f64(dict: &Bound<'_, PyDict>, name: &str) -> PyResult<f64> {
    let value = dict
        .get_item(name)?
        .ok_or_else(|| PyValueError::new_err(format!("bar is missing `{name}`")))?
        .extract()?;
    finite_bar_value(value, name)
}

fn finite_bar_value(value: f64, name: &str) -> PyResult<f64> {
    if value.is_finite() {
        return Ok(value);
    }

    Err(PyValueError::new_err(format!(
        "bar `{name}` value must be finite"
    )))
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
    output.set_item("inputs", inputs_to_py(py, analysis)?)?;
    Ok(output.into_any().unbind())
}

fn inputs_to_py(py: Python<'_>, analysis: &Analysis) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    if let Some(hir) = &analysis.hir {
        for input in input_calls(hir) {
            let item = PyDict::new(py);
            item.set_item("callSiteId", input.call_site_id)?;
            item.set_item("name", input.name)?;
            item.set_item("title", input.title)?;
            output.append(item)?;
        }
    }
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
