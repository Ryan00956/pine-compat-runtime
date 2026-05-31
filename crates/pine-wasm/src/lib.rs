use pine_ir::HirProgram;
use pine_runtime::{
    Bar, PUBLIC_ANALYSIS_SCHEMA_VERSION, RequestEnvironment, public_runtime_result_json,
    run_historical_with_request_environment,
};
use pine_sema::{Analysis, AnalysisInput, analyze_input};
use pine_syntax::{Diagnostic, Severity, SourceFile, Span};
use wasm_bindgen::prelude::*;

mod library_sources;
mod request_bars;
use library_sources::analysis_input_with_libraries;
use request_bars::request_environment_from_json;

#[wasm_bindgen(js_name = Program)]
pub struct WasmProgram {
    hir: HirProgram,
}

#[wasm_bindgen(js_name = compileScript)]
pub fn compile_script(source: &str) -> Result<WasmProgram, JsValue> {
    compile_program(analysis_input(source)).map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen(js_name = compileScriptWithLibraries)]
pub fn compile_script_with_libraries(
    source: &str,
    library_sources_json: &str,
) -> Result<WasmProgram, JsValue> {
    let input = analysis_input_with_libraries(source, library_sources_json)
        .map_err(|err| JsValue::from_str(&err))?;
    compile_program(input).map_err(|err| JsValue::from_str(&err))
}

fn compile_program(input: AnalysisInput) -> Result<WasmProgram, String> {
    let source_file = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        return Err(format_diagnostics(&source_file, &analysis.diagnostics));
    }

    let hir = analysis
        .hir
        .ok_or_else(|| "analysis did not produce executable HIR".to_string())?;
    Ok(WasmProgram { hir })
}

#[wasm_bindgen(js_name = analyzeScript)]
pub fn analyze_script(source: &str) -> String {
    let input = analysis_input(source);
    analyze_input_json(input)
}

#[wasm_bindgen(js_name = analyzeScriptWithLibraries)]
pub fn analyze_script_with_libraries(source: &str, library_sources_json: &str) -> String {
    match analysis_input_with_libraries(source, library_sources_json) {
        Ok(input) => analyze_input_json(input),
        Err(message) => analysis_error_json(&message),
    }
}

fn analyze_input_json(input: AnalysisInput) -> String {
    let source_file = input.root().clone();
    let analysis = analyze_input(&input);
    analysis_json(&source_file, &analysis)
}

#[wasm_bindgen(js_name = runScriptCsv)]
pub fn run_script_csv(source: &str, bars_csv: &str) -> Result<String, JsValue> {
    run_script_csv_internal(source, bars_csv).map_err(|err| JsValue::from_str(&err))
}

fn run_script_csv_internal(source: &str, bars_csv: &str) -> Result<String, String> {
    let program = compile_program(analysis_input(source))?;
    program.run_csv_internal(bars_csv)
}

#[wasm_bindgen(js_name = runScriptCsvWithRequestBars)]
pub fn run_script_csv_with_request_bars(
    source: &str,
    bars_csv: &str,
    request_bars_json: &str,
) -> Result<String, JsValue> {
    run_script_csv_with_request_bars_internal(source, bars_csv, request_bars_json)
        .map_err(|err| JsValue::from_str(&err))
}

fn run_script_csv_with_request_bars_internal(
    source: &str,
    bars_csv: &str,
    request_bars_json: &str,
) -> Result<String, String> {
    let program = compile_program(analysis_input(source))?;
    let request_environment = request_environment_from_json(request_bars_json)?;
    program.run_csv_with_request_environment_internal(bars_csv, request_environment)
}

#[wasm_bindgen(js_name = runScriptCsvWithLibraries)]
pub fn run_script_csv_with_libraries(
    source: &str,
    bars_csv: &str,
    library_sources_json: &str,
) -> Result<String, JsValue> {
    run_script_csv_with_libraries_internal(source, bars_csv, library_sources_json)
        .map_err(|err| JsValue::from_str(&err))
}

fn run_script_csv_with_libraries_internal(
    source: &str,
    bars_csv: &str,
    library_sources_json: &str,
) -> Result<String, String> {
    let input = analysis_input_with_libraries(source, library_sources_json)?;
    let program = compile_program(input)?;
    program.run_csv_internal(bars_csv)
}

fn analysis_input(source: &str) -> AnalysisInput {
    AnalysisInput::new(SourceFile::new("<wasm>", source))
}

#[wasm_bindgen]
impl WasmProgram {
    #[wasm_bindgen(js_name = runCsv)]
    pub fn run_csv(&self, bars_csv: &str) -> Result<String, JsValue> {
        self.run_csv_internal(bars_csv)
            .map_err(|err| JsValue::from_str(&err))
    }
}

impl WasmProgram {
    fn run_csv_internal(&self, bars_csv: &str) -> Result<String, String> {
        self.run_csv_with_request_environment_internal(bars_csv, RequestEnvironment::default())
    }

    fn run_csv_with_request_environment_internal(
        &self,
        bars_csv: &str,
        request_environment: RequestEnvironment,
    ) -> Result<String, String> {
        let bars = parse_bars_csv(bars_csv)?;
        let result = run_historical_with_request_environment(&self.hir, &bars, request_environment)
            .map_err(|err| format!("runtime failed: {}", err.message))?;
        Ok(public_runtime_result_json(&result))
    }
}

fn parse_bars_csv(text: &str) -> Result<Vec<Bar>, String> {
    let mut bars = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line_index == 0 && line.to_ascii_lowercase().contains("close") {
            continue;
        }

        let columns: Vec<_> = line.split(',').map(str::trim).collect();
        if columns.len() != 6 {
            return Err(format!(
                "invalid bars CSV at line {}: expected 6 columns time,open,high,low,close,volume",
                line_index + 1
            ));
        }

        bars.push(Bar {
            time: parse_column(columns[0], line_index, "time")?,
            open: parse_column(columns[1], line_index, "open")?,
            high: parse_column(columns[2], line_index, "high")?,
            low: parse_column(columns[3], line_index, "low")?,
            close: parse_column(columns[4], line_index, "close")?,
            volume: parse_column(columns[5], line_index, "volume")?,
        });
    }
    Ok(bars)
}

fn parse_column<T: std::str::FromStr>(
    value: &str,
    line_index: usize,
    name: &str,
) -> Result<T, String> {
    value.parse::<T>().map_err(|_| {
        format!(
            "invalid `{name}` value `{value}` at bars CSV line {}",
            line_index + 1
        )
    })
}

fn analysis_json(source: &SourceFile, analysis: &Analysis) -> String {
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_ANALYSIS_SCHEMA_VERSION);
    output.push_str("\"languageVersion\":");
    match analysis.compatibility.language_version {
        Some(version) => output.push_str(&version.to_string()),
        None => output.push_str("null"),
    }
    output.push_str(",\"executable\":");
    output.push_str(if analysis.hir.is_some() {
        "true"
    } else {
        "false"
    });
    output.push_str(",\"diagnostics\":");
    output.push_str(&diagnostics_json(source, &analysis.diagnostics));
    output.push_str(",\"compatibility\":{");
    output.push_str("\"supported\":");
    output.push_str(&features_json(
        source,
        analysis
            .compatibility
            .supported
            .iter()
            .map(|feature| (&feature.feature, None, feature.span)),
    ));
    output.push_str(",\"unsupported\":");
    output.push_str(&features_json(
        source,
        analysis
            .compatibility
            .unsupported
            .iter()
            .map(|feature| (&feature.feature, Some(&feature.reason), feature.span)),
    ));
    output.push_str("}}");
    output
}

fn analysis_error_json(message: &str) -> String {
    format!(
        "{{\"schemaVersion\":{},\"languageVersion\":null,\"executable\":false,\"diagnostics\":[{{\"code\":\"E_HOST_INPUT\",\"severity\":\"error\",\"message\":\"{}\",\"span\":{{\"start\":0,\"end\":0,\"line\":1,\"column\":1}}}}],\"compatibility\":{{\"supported\":[],\"unsupported\":[]}}}}",
        PUBLIC_ANALYSIS_SCHEMA_VERSION,
        json_escape(message)
    )
}

fn features_json<'a>(
    source: &SourceFile,
    features: impl Iterator<Item = (&'a String, Option<&'a String>, Span)>,
) -> String {
    let mut output = String::from("[");
    for (index, (feature, reason, span)) in features.enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"feature\":\"{}\"", json_escape(feature)));
        if let Some(reason) = reason {
            output.push_str(&format!(",\"reason\":\"{}\"", json_escape(reason)));
        }
        output.push_str(",\"span\":");
        output.push_str(&span_json(source, span));
        output.push('}');
    }
    output.push(']');
    output
}

fn diagnostics_json(source: &SourceFile, diagnostics: &[Diagnostic]) -> String {
    let mut output = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"code\":\"{}\",\"severity\":\"{}\",\"message\":\"{}\",\"span\":{}}}",
            json_escape(&diagnostic.code),
            severity_name(diagnostic.severity),
            json_escape(&diagnostic.message),
            span_json(source, diagnostic.span)
        ));
    }
    output.push(']');
    output
}

fn span_json(source: &SourceFile, span: Span) -> String {
    let line_col = source.line_col(span.start);
    format!(
        "{{\"start\":{},\"end\":{},\"line\":{},\"column\":{}}}",
        span.start, span.end, line_col.line, line_col.column
    )
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

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            ch if (ch as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests;
