use pine_ir::HirProgram;
use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::{Analysis, analyze_source};
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
    fn run(&self, py: Python<'_>, bars: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let bars = parse_bars(bars)?;
        let result =
            run_historical(&self.hir, &bars).map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_to_py(py, &result)
    }
}

#[pyfunction]
fn compile_script(source: &str) -> PyResult<PyProgram> {
    let source_file = SourceFile::new("<python>", source);
    let analysis = analyze_source(&source_file);
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

#[pyfunction]
fn analyze_script(py: Python<'_>, source: &str) -> PyResult<Py<PyAny>> {
    let source_file = SourceFile::new("<python>", source);
    let analysis = analyze_source(&source_file);
    analysis_to_py(py, &source_file, &analysis)
}

#[pyfunction]
fn run_script(py: Python<'_>, source: &str, bars: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let program = compile_script(source)?;
    program.run(py, bars)
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
    output.set_item("plots", plots_to_py(py, &result.plots)?)?;
    output.set_item("hlines", hlines_to_py(py, &result.hlines)?)?;
    output.set_item("fills", fills_to_py(py, &result.fills)?)?;
    output.set_item("diagnostics", PyList::empty(py))?;
    Ok(output.into_any().unbind())
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
        PineValue::Color(value) | PineValue::Plot(value) | PineValue::HLine(value) => {
            output.append(*value)
        }
        PineValue::Tuple(values) => output.append(values_to_py(py, values)?),
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
