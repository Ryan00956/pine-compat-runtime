use pine_ir::HirProgram;
use pine_runtime::{Bar, PUBLIC_OUTPUT_SCHEMA_VERSION, public_runtime_result_json, run_historical};
use pine_sema::{Analysis, analyze_source};
use pine_syntax::{Diagnostic, Severity, SourceFile, Span};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Program)]
pub struct WasmProgram {
    hir: HirProgram,
}

#[wasm_bindgen(js_name = compileScript)]
pub fn compile_script(source: &str) -> Result<WasmProgram, JsValue> {
    let source_file = SourceFile::new("<wasm>", source);
    let analysis = analyze_source(&source_file);
    if !analysis.diagnostics.is_empty() {
        return Err(JsValue::from_str(&format_diagnostics(
            &source_file,
            &analysis.diagnostics,
        )));
    }

    let hir = analysis
        .hir
        .ok_or_else(|| JsValue::from_str("analysis did not produce executable HIR"))?;
    Ok(WasmProgram { hir })
}

#[wasm_bindgen(js_name = analyzeScript)]
pub fn analyze_script(source: &str) -> String {
    let source_file = SourceFile::new("<wasm>", source);
    let analysis = analyze_source(&source_file);
    analysis_json(&source_file, &analysis)
}

#[wasm_bindgen(js_name = runScriptCsv)]
pub fn run_script_csv(source: &str, bars_csv: &str) -> Result<String, JsValue> {
    let program = compile_script(source)?;
    program.run_csv(bars_csv)
}

#[wasm_bindgen]
impl WasmProgram {
    #[wasm_bindgen(js_name = runCsv)]
    pub fn run_csv(&self, bars_csv: &str) -> Result<String, JsValue> {
        let bars = parse_bars_csv(bars_csv).map_err(|err| JsValue::from_str(&err))?;
        let result = run_historical(&self.hir, &bars)
            .map_err(|err| JsValue::from_str(&format!("runtime failed: {}", err.message)))?;
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
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_OUTPUT_SCHEMA_VERSION);
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
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::PathBuf};

    #[test]
    fn analyzes_script_to_json() {
        let output = analyze_script("indicator(\"demo\")\nplot(close)\n");

        assert!(output.contains("\"schemaVersion\":2"));
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

        assert!(output.contains("\"schemaVersion\":2"));
        assert!(output.contains("\"values\":[1,2]"));
        assert!(output.contains("\"plotChars\":[]"));
        assert!(output.contains("\"plotShapes\":[]"));
        assert!(output.contains("\"plotArrows\":[]"));
        assert!(output.contains("\"plotBars\":[]"));
        assert!(output.contains("\"plotCandles\":[]"));
        assert!(output.contains("\"labels\":[]"));
        assert!(output.contains("\"lines\":[]"));
        assert!(output.contains("\"boxes\":[]"));
    }

    #[test]
    fn analysis_outputs_match_golden_snapshots() {
        assert_snapshot(
            "analysis_supported.json",
            &analyze_script(include_str!(
                "../../../tests/fixtures/runtime/snapshot_plot.pine"
            )),
        );
        assert_snapshot(
            "analysis_unsupported.json",
            &analyze_script(include_str!(
                "../../../tests/fixtures/sema/unsupported_request.pine"
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
}
