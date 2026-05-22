use pine_ir::HirProgram;
use pine_runtime::{Bar, PUBLIC_OUTPUT_SCHEMA_VERSION, PineValue, RuntimeResult, run_historical};
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
        Ok(result_json(&result))
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

fn result_json(result: &RuntimeResult) -> String {
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_OUTPUT_SCHEMA_VERSION);
    output.push_str("\"plots\":");
    output.push_str(&plots_json(&result.plots));
    output.push_str(",\"plotChars\":");
    output.push_str(&plot_chars_json(&result.plot_chars));
    output.push_str(",\"plotShapes\":");
    output.push_str(&plot_shapes_json(&result.plot_shapes));
    output.push_str(",\"plotArrows\":");
    output.push_str(&plot_arrows_json(&result.plot_arrows));
    output.push_str(",\"plotBars\":");
    output.push_str(&plot_bars_json(&result.plot_bars));
    output.push_str(",\"plotCandles\":");
    output.push_str(&plot_candles_json(&result.plot_candles));
    output.push_str(",\"bgColors\":");
    output.push_str(&colors_json(&result.bg_colors));
    output.push_str(",\"barColors\":");
    output.push_str(&colors_json(&result.bar_colors));
    output.push_str(",\"hlines\":");
    output.push_str(&hlines_json(&result.hlines));
    output.push_str(",\"fills\":");
    output.push_str(&fills_json(&result.fills));
    output.push_str(",\"diagnostics\":[]");
    output.push('}');
    output
}

fn plots_json(plots: &[pine_runtime::PlotSeries]) -> String {
    let mut output = String::from("[");
    for (plot_index, plot) in plots.iter().enumerate() {
        if plot_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot.id));
        values_json_into(&mut output, &plot.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn colors_json(colors: &[pine_runtime::ColorSeries]) -> String {
    let mut output = String::from("[");
    for (color_index, colors) in colors.iter().enumerate() {
        if color_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", colors.id));
        values_json_into(&mut output, &colors.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_chars_json(plot_chars: &[pine_runtime::PlotCharSeries]) -> String {
    let mut output = String::from("[");
    for (plot_char_index, plot_char) in plot_chars.iter().enumerate() {
        if plot_char_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_char.id));
        values_json_into(&mut output, &plot_char.values);
        output.push_str("],\"chars\":[");
        values_json_into(&mut output, &plot_char.chars);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_char.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_shapes_json(plot_shapes: &[pine_runtime::PlotShapeSeries]) -> String {
    let mut output = String::from("[");
    for (plot_shape_index, plot_shape) in plot_shapes.iter().enumerate() {
        if plot_shape_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_shape.id));
        values_json_into(&mut output, &plot_shape.values);
        output.push_str("],\"styles\":[");
        values_json_into(&mut output, &plot_shape.styles);
        output.push_str("],\"locations\":[");
        values_json_into(&mut output, &plot_shape.locations);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_shape.colors);
        output.push_str("],\"texts\":[");
        values_json_into(&mut output, &plot_shape.texts);
        output.push_str("],\"textColors\":[");
        values_json_into(&mut output, &plot_shape.text_colors);
        output.push_str("],\"sizes\":[");
        values_json_into(&mut output, &plot_shape.sizes);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_arrows_json(plot_arrows: &[pine_runtime::PlotArrowSeries]) -> String {
    let mut output = String::from("[");
    for (plot_arrow_index, plot_arrow) in plot_arrows.iter().enumerate() {
        if plot_arrow_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_arrow.id));
        values_json_into(&mut output, &plot_arrow.values);
        output.push_str("],\"colorUps\":[");
        values_json_into(&mut output, &plot_arrow.color_ups);
        output.push_str("],\"colorDowns\":[");
        values_json_into(&mut output, &plot_arrow.color_downs);
        output.push_str("],\"minHeights\":[");
        values_json_into(&mut output, &plot_arrow.min_heights);
        output.push_str("],\"maxHeights\":[");
        values_json_into(&mut output, &plot_arrow.max_heights);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_bars_json(plot_bars: &[pine_runtime::PlotBarSeries]) -> String {
    let mut output = String::from("[");
    for (plot_bar_index, plot_bar) in plot_bars.iter().enumerate() {
        if plot_bar_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_bar.id));
        values_json_into(&mut output, &plot_bar.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_bar.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_bar.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_bar.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_bar.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_candles_json(plot_candles: &[pine_runtime::PlotCandleSeries]) -> String {
    let mut output = String::from("[");
    for (plot_candle_index, plot_candle) in plot_candles.iter().enumerate() {
        if plot_candle_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_candle.id));
        values_json_into(&mut output, &plot_candle.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_candle.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_candle.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_candle.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_candle.colors);
        output.push_str("],\"wickColors\":[");
        values_json_into(&mut output, &plot_candle.wick_colors);
        output.push_str("],\"borderColors\":[");
        values_json_into(&mut output, &plot_candle.border_colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn values_json_into(output: &mut String, values: &[PineValue]) {
    for (value_index, value) in values.iter().enumerate() {
        if value_index > 0 {
            output.push(',');
        }
        output.push_str(&value_json(value));
    }
}

fn hlines_json(hlines: &[pine_runtime::HLineOutput]) -> String {
    let mut output = String::from("[");
    for (index, hline) in hlines.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"price\":{}}}",
            hline.id,
            value_json(&hline.price)
        ));
    }
    output.push(']');
    output
}

fn fills_json(fills: &[pine_runtime::FillOutput]) -> String {
    let mut output = String::from("[");
    for (index, fill) in fills.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"firstId\":{},\"secondId\":{}}}",
            fill.id, fill.first_id, fill.second_id
        ));
    }
    output.push(']');
    output
}

fn value_json(value: &PineValue) -> String {
    match value {
        PineValue::Int(value) => value.to_string(),
        PineValue::Float(value) => value.to_string(),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => format!("\"{}\"", json_escape(value)),
        PineValue::Color(value) => value.to_string(),
        PineValue::Plot(value) | PineValue::HLine(value) => value.to_string(),
        PineValue::Tuple(values) => {
            let mut output = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&value_json(value));
            }
            output.push(']');
            output
        }
        PineValue::Array(_) | PineValue::Na | PineValue::Void => "null".to_owned(),
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

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_script_to_json() {
        let output = analyze_script("indicator(\"demo\")\nplot(close)\n");

        assert!(output.contains("\"schemaVersion\":1"));
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

        assert!(output.contains("\"schemaVersion\":1"));
        assert!(output.contains("\"values\":[1,2]"));
        assert!(output.contains("\"plotChars\":[]"));
        assert!(output.contains("\"plotShapes\":[]"));
        assert!(output.contains("\"plotArrows\":[]"));
        assert!(output.contains("\"plotBars\":[]"));
        assert!(output.contains("\"plotCandles\":[]"));
    }
}
