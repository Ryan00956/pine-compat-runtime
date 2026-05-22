use std::{env, fs, process::ExitCode};

use pine_runtime::{
    Bar, HistoryRetentionMode, PUBLIC_OUTPUT_SCHEMA_VERSION, PineValue, RuntimeProfile,
    RuntimeResult, run_historical,
};
use pine_sema::analyze_source;
use pine_syntax::{SourceFile, parse_source};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage());
    };

    if command == "matrix" {
        return run_matrix(args.collect());
    }

    let Some(path) = args.next() else {
        return Err(usage());
    };

    let text = fs::read_to_string(&path).map_err(|err| format!("failed to read {path}: {err}"))?;
    let source = SourceFile::new(path, text);

    match command.as_str() {
        "analyze" => {
            let analysis = analyze_source(&source);
            println!("diagnostics: {}", analysis.diagnostics.len());
            println!(
                "supported: {}, unsupported: {}",
                analysis.compatibility.supported.len(),
                analysis.compatibility.unsupported.len()
            );
            for diagnostic in analysis.diagnostics {
                let line_col = source.line_col(diagnostic.span.start);
                println!(
                    "{}:{:?}:{}:{}: {}",
                    diagnostic.code,
                    diagnostic.severity,
                    line_col.line,
                    line_col.column,
                    diagnostic.message
                );
            }
            Ok(())
        }
        "fmt-ast" => {
            let parsed = parse_source(&source);
            println!("{:#?}", parsed.program);
            for diagnostic in parsed.diagnostics {
                let line_col = source.line_col(diagnostic.span.start);
                println!(
                    "{}:{:?}:{}:{}: {}",
                    diagnostic.code,
                    diagnostic.severity,
                    line_col.line,
                    line_col.column,
                    diagnostic.message
                );
            }
            Ok(())
        }
        "run" => {
            let Some(flag) = args.next() else {
                return Err(usage());
            };
            if flag != "--bars" {
                return Err(usage());
            }
            let Some(bars_path) = args.next() else {
                return Err(usage());
            };
            let profile = match args.next().as_deref() {
                None => false,
                Some("--profile") => true,
                Some(_) => return Err(usage()),
            };

            let analysis = analyze_source(&source);
            if !analysis.diagnostics.is_empty() {
                for diagnostic in analysis.diagnostics {
                    let line_col = source.line_col(diagnostic.span.start);
                    eprintln!(
                        "{}:{:?}:{}:{}: {}",
                        diagnostic.code,
                        diagnostic.severity,
                        line_col.line,
                        line_col.column,
                        diagnostic.message
                    );
                }
                return Err("analysis failed".to_owned());
            }
            let Some(hir) = analysis.hir else {
                return Err("analysis did not produce executable HIR".to_owned());
            };

            let bars_text = fs::read_to_string(&bars_path)
                .map_err(|err| format!("failed to read {bars_path}: {err}"))?;
            let bars = parse_bars_csv(&bars_text)?;
            if profile {
                let result = pine_runtime::run_historical_profiled(&hir, &bars)
                    .map_err(|err| format!("runtime failed: {}", err.message))?;
                println!("{}", profiled_result_json(&result.result, &result.profile));
            } else {
                let result = run_historical(&hir, &bars)
                    .map_err(|err| format!("runtime failed: {}", err.message))?;
                println!("{}", result_json(&result));
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: pine-compat <analyze|fmt-ast> <script.pine>\n       pine-compat run <script.pine> --bars <bars.csv> [--profile]\n       pine-compat matrix [--format text|json]".to_owned()
}

fn run_matrix(args: Vec<String>) -> Result<(), String> {
    let format = match args.as_slice() {
        [] => MatrixFormat::Text,
        [flag, format] if flag == "--format" && format == "text" => MatrixFormat::Text,
        [flag, format] if flag == "--format" && format == "json" => MatrixFormat::Json,
        _ => return Err(usage()),
    };

    let entries = conformance_entries();
    match format {
        MatrixFormat::Text => println!("{}", matrix_text(&entries)),
        MatrixFormat::Json => println!("{}", matrix_json(&entries)),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatrixFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatrixEntry {
    feature: String,
    status: String,
    notes: String,
    fixtures: Vec<String>,
}

fn conformance_entries() -> Vec<MatrixEntry> {
    conformance_entries_from_tsv(include_str!("../../../tests/fixtures/conformance.tsv"))
}

fn conformance_entries_from_tsv(text: &str) -> Vec<MatrixEntry> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            if line.is_empty() || index == 0 {
                return None;
            }

            let columns: Vec<_> = line.split('\t').collect();
            assert_eq!(
                columns.len(),
                4,
                "invalid conformance metadata at line {}",
                index + 1
            );
            Some(MatrixEntry {
                feature: columns[0].to_owned(),
                status: columns[1].to_owned(),
                notes: columns[2].to_owned(),
                fixtures: columns[3]
                    .split(';')
                    .filter(|fixture| !fixture.is_empty())
                    .map(str::to_owned)
                    .collect(),
            })
        })
        .collect()
}

fn matrix_text(entries: &[MatrixEntry]) -> String {
    let feature_width = entries
        .iter()
        .map(|entry| entry.feature.len())
        .chain([7])
        .max()
        .unwrap_or(7);
    let status_width = entries
        .iter()
        .map(|entry| entry.status.len())
        .chain([6])
        .max()
        .unwrap_or(6);

    let mut output = String::new();
    output.push_str(&format!(
        "{:<feature_width$}  {:<status_width$}  fixtures  notes\n",
        "feature", "status"
    ));
    output.push_str(&format!(
        "{:-<feature_width$}  {:-<status_width$}  --------  -----\n",
        "", ""
    ));
    for entry in entries {
        output.push_str(&format!(
            "{:<feature_width$}  {:<status_width$}  {}  {}\n",
            entry.feature,
            entry.status,
            entry.fixtures.join(";"),
            entry.notes
        ));
    }
    output
}

fn matrix_json(entries: &[MatrixEntry]) -> String {
    let mut output = format!(
        "{{\"schemaVersion\":{},\"features\":[",
        PUBLIC_OUTPUT_SCHEMA_VERSION
    );
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str("{\"feature\":\"");
        output.push_str(&json_escape(&entry.feature));
        output.push_str("\",\"status\":\"");
        output.push_str(&json_escape(&entry.status));
        output.push_str("\",\"notes\":\"");
        output.push_str(&json_escape(&entry.notes));
        output.push_str("\",\"fixtures\":[");
        for (fixture_index, fixture) in entry.fixtures.iter().enumerate() {
            if fixture_index > 0 {
                output.push(',');
            }
            output.push('"');
            output.push_str(&json_escape(fixture));
            output.push('"');
        }
        output.push_str("]}");
    }
    output.push_str("]}");
    output
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

fn profiled_result_json(result: &RuntimeResult, profile: &RuntimeProfile) -> String {
    let mut output = result_json(result);
    output.pop();
    output.push_str(",\"profile\":");
    output.push_str(&profile_json(profile));
    output.push('}');
    output
}

fn profile_json(profile: &RuntimeProfile) -> String {
    format!(
        concat!(
            "{{",
            "\"bars\":{},",
            "\"seriesBuffers\":{},",
            "\"seriesValues\":{},",
            "\"seriesCapacity\":{},",
            "\"maxSeriesDepth\":{},",
            "\"historyRetentionMode\":\"{}\",",
            "\"historyMaxConstantOffset\":{},",
            "\"historyMaxBarsBack\":{},",
            "\"historyHasDynamicOffsets\":{},",
            "\"symbolSlots\":{},",
            "\"symbolCapacity\":{},",
            "\"currentSeriesSlots\":{},",
            "\"currentSeriesCapacity\":{},",
            "\"varSlots\":{},",
            "\"varCapacity\":{},",
            "\"arraySlots\":{},",
            "\"arrayCapacity\":{},",
            "\"arrayValues\":{},",
            "\"arrayValueCapacity\":{},",
            "\"callStateSlots\":{},",
            "\"callStateCapacity\":{},",
            "\"valuewhenStateSlots\":{},",
            "\"valuewhenStateCapacity\":{},",
            "\"valuewhenStateValues\":{},",
            "\"valuewhenStateValueCapacity\":{},",
            "\"rollingWindowSlots\":{},",
            "\"rollingWindowCapacity\":{},",
            "\"rollingWindowValues\":{},",
            "\"rollingWindowValueCapacity\":{},",
            "\"rsiStateSlots\":{},",
            "\"rsiStateCapacity\":{},",
            "\"macdStateSlots\":{},",
            "\"macdStateCapacity\":{},",
            "\"plots\":{},",
            "\"plotValues\":{},",
            "\"plotCapacity\":{},",
            "\"plotChars\":{},",
            "\"plotCharValues\":{},",
            "\"plotCharCapacity\":{},",
            "\"plotShapes\":{},",
            "\"plotShapeValues\":{},",
            "\"plotShapeCapacity\":{},",
            "\"plotArrows\":{},",
            "\"plotArrowValues\":{},",
            "\"plotArrowCapacity\":{},",
            "\"plotBars\":{},",
            "\"plotBarValues\":{},",
            "\"plotBarCapacity\":{},",
            "\"plotCandles\":{},",
            "\"plotCandleValues\":{},",
            "\"plotCandleCapacity\":{},",
            "\"bgColors\":{},",
            "\"bgColorValues\":{},",
            "\"bgColorCapacity\":{},",
            "\"barColors\":{},",
            "\"barColorValues\":{},",
            "\"barColorCapacity\":{},",
            "\"hlines\":{},",
            "\"hlineCapacity\":{},",
            "\"fills\":{},",
            "\"fillCapacity\":{}",
            "}}"
        ),
        profile.bars,
        profile.series_buffers,
        profile.series_values,
        profile.series_capacity,
        profile.max_series_depth,
        history_retention_mode_json(profile.history_retention_mode),
        profile.history_max_constant_offset,
        option_u32_json(profile.history_max_bars_back),
        profile.history_has_dynamic_offsets,
        profile.symbol_slots,
        profile.symbol_capacity,
        profile.current_series_slots,
        profile.current_series_capacity,
        profile.var_slots,
        profile.var_capacity,
        profile.array_slots,
        profile.array_capacity,
        profile.array_values,
        profile.array_value_capacity,
        profile.call_state_slots,
        profile.call_state_capacity,
        profile.valuewhen_state_slots,
        profile.valuewhen_state_capacity,
        profile.valuewhen_state_values,
        profile.valuewhen_state_value_capacity,
        profile.rolling_window_slots,
        profile.rolling_window_capacity,
        profile.rolling_window_values,
        profile.rolling_window_value_capacity,
        profile.rsi_state_slots,
        profile.rsi_state_capacity,
        profile.macd_state_slots,
        profile.macd_state_capacity,
        profile.plots,
        profile.plot_values,
        profile.plot_capacity,
        profile.plot_chars,
        profile.plot_char_values,
        profile.plot_char_capacity,
        profile.plot_shapes,
        profile.plot_shape_values,
        profile.plot_shape_capacity,
        profile.plot_arrows,
        profile.plot_arrow_values,
        profile.plot_arrow_capacity,
        profile.plot_bars,
        profile.plot_bar_values,
        profile.plot_bar_capacity,
        profile.plot_candles,
        profile.plot_candle_values,
        profile.plot_candle_capacity,
        profile.bg_colors,
        profile.bg_color_values,
        profile.bg_color_capacity,
        profile.bar_colors,
        profile.bar_color_values,
        profile.bar_color_capacity,
        profile.hlines,
        profile.hline_capacity,
        profile.fills,
        profile.fill_capacity
    )
}

fn history_retention_mode_json(mode: HistoryRetentionMode) -> &'static str {
    match mode {
        HistoryRetentionMode::StaticTrimmed => "staticTrimmed",
        HistoryRetentionMode::DynamicFull => "dynamicFull",
        HistoryRetentionMode::MaxBarsBack => "maxBarsBack",
    }
}

fn option_u32_json(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
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

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_includes_supported_builtins_and_unsupported_features() {
        let entries = conformance_entries();

        for signature in pine_builtins::PHASE_1_BUILTINS {
            let expected_status = if signature.name.starts_with("array.") {
                "partial"
            } else {
                "supported"
            };
            assert!(
                entries.iter().any(|entry| entry.feature == signature.name
                    && entry.status == expected_status
                    && !entry.fixtures.is_empty()),
                "{} should have fixture-derived metadata",
                signature.name
            );
        }
        assert!(entries.iter().any(|entry| entry.feature == "math.max"
            && entry.status == "supported"
            && !entry.fixtures.is_empty()));
        assert!(entries.iter().any(|entry| entry.feature == "ta.macd"
            && entry.status == "supported"
            && !entry.fixtures.is_empty()));
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "if" && entry.status == "supported")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "for" && entry.status == "partial")
        );
        assert!(entries.iter().any(|entry| {
            entry.feature == "expression-body functions" && entry.status == "supported"
        }));
        assert!(entries.iter().any(|entry| {
            entry.feature == "multi-statement functions" && entry.status == "supported"
        }));
        assert!(entries.iter().any(|entry| {
            entry.feature == "block-local declarations" && entry.status == "supported"
        }));
        assert!(entries.iter().any(|entry| {
            entry.feature == "recursive functions" && entry.status == "unsupported"
        }));
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "request.*" && entry.status == "unsupported")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "varip" && entry.status == "unsupported")
        );
    }

    #[test]
    fn conformance_metadata_references_existing_fixtures() {
        let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for entry in conformance_entries() {
            assert!(
                !entry.fixtures.is_empty(),
                "{} should reference at least one fixture",
                entry.feature
            );
            for fixture in entry.fixtures {
                assert!(
                    workspace.join(&fixture).exists(),
                    "{} fixture path should exist for {}",
                    fixture,
                    entry.feature
                );
            }
        }
    }

    #[test]
    fn formats_matrix_as_text() {
        let entries = vec![MatrixEntry {
            feature: "indicator".to_owned(),
            status: "supported".to_owned(),
            notes: "fixture-derived executable subset".to_owned(),
            fixtures: vec!["tests/fixtures/runtime/io.pine".to_owned()],
        }];

        let output = matrix_text(&entries);

        assert!(output.contains("feature"));
        assert!(output.contains("fixtures"));
        assert!(output.contains("indicator"));
        assert!(output.contains("supported"));
        assert!(output.contains("tests/fixtures/runtime/io.pine"));
    }

    #[test]
    fn formats_matrix_as_json() {
        let entries = vec![MatrixEntry {
            feature: "request.*".to_owned(),
            status: "unsupported".to_owned(),
            notes: "multi-symbol".to_owned(),
            fixtures: vec!["tests/fixtures/sema/unsupported_request.pine".to_owned()],
        }];

        let output = matrix_json(&entries);

        assert_eq!(
            output,
            r#"{"schemaVersion":1,"features":[{"feature":"request.*","status":"unsupported","notes":"multi-symbol","fixtures":["tests/fixtures/sema/unsupported_request.pine"]}]}"#
        );
    }

    #[test]
    fn formats_runtime_result_json_with_schema_version() {
        let result = RuntimeResult {
            plots: vec![],
            plot_chars: vec![],
            plot_shapes: vec![],
            plot_arrows: vec![],
            plot_bars: vec![],
            plot_candles: vec![],
            bg_colors: vec![],
            bar_colors: vec![],
            hlines: vec![],
            fills: vec![],
            diagnostics: vec![],
        };

        let output = result_json(&result);

        assert!(output.starts_with(r#"{"schemaVersion":1,"#));
        assert!(output.contains(r#""diagnostics":[]"#));
    }

    #[test]
    fn formats_profiled_result_json() {
        let result = RuntimeResult {
            plots: vec![],
            plot_chars: vec![],
            plot_shapes: vec![],
            plot_arrows: vec![],
            plot_bars: vec![],
            plot_candles: vec![],
            bg_colors: vec![],
            bar_colors: vec![],
            hlines: vec![],
            fills: vec![],
            diagnostics: vec![],
        };
        let profile = RuntimeProfile {
            bars: 3,
            series_buffers: 2,
            series_values: 6,
            series_capacity: 8,
            max_series_depth: 3,
            history_retention_mode: HistoryRetentionMode::DynamicFull,
            history_max_constant_offset: 2,
            history_max_bars_back: None,
            history_has_dynamic_offsets: true,
            symbol_slots: 10,
            symbol_capacity: 14,
            current_series_slots: 0,
            current_series_capacity: 14,
            var_slots: 1,
            var_capacity: 3,
            array_slots: 1,
            array_capacity: 3,
            array_values: 2,
            array_value_capacity: 2,
            call_state_slots: 1,
            call_state_capacity: 3,
            valuewhen_state_slots: 1,
            valuewhen_state_capacity: 3,
            valuewhen_state_values: 2,
            valuewhen_state_value_capacity: 2,
            rolling_window_slots: 1,
            rolling_window_capacity: 3,
            rolling_window_values: 2,
            rolling_window_value_capacity: 2,
            rsi_state_slots: 0,
            rsi_state_capacity: 0,
            macd_state_slots: 0,
            macd_state_capacity: 0,
            plots: 0,
            plot_values: 0,
            plot_capacity: 0,
            plot_chars: 0,
            plot_char_values: 0,
            plot_char_capacity: 0,
            plot_shapes: 0,
            plot_shape_values: 0,
            plot_shape_capacity: 0,
            plot_arrows: 0,
            plot_arrow_values: 0,
            plot_arrow_capacity: 0,
            plot_bars: 0,
            plot_bar_values: 0,
            plot_bar_capacity: 0,
            plot_candles: 0,
            plot_candle_values: 0,
            plot_candle_capacity: 0,
            bg_colors: 0,
            bg_color_values: 0,
            bg_color_capacity: 0,
            bar_colors: 0,
            bar_color_values: 0,
            bar_color_capacity: 0,
            hlines: 0,
            hline_capacity: 0,
            fills: 0,
            fill_capacity: 0,
        };

        let output = profiled_result_json(&result, &profile);

        assert!(output.starts_with(r#"{"schemaVersion":1,"#));
        assert!(output.contains(r#""profile""#));
        assert!(output.contains(r#""bars":3"#));
        assert!(output.contains(r#""seriesValues":6"#));
        assert!(output.contains(r#""maxSeriesDepth":3"#));
        assert!(output.contains(r#""historyRetentionMode":"dynamicFull""#));
        assert!(output.contains(r#""historyMaxConstantOffset":2"#));
        assert!(output.contains(r#""historyMaxBarsBack":null"#));
        assert!(output.contains(r#""historyHasDynamicOffsets":true"#));
        assert!(output.contains(r#""arrayValues":2"#));
        assert!(output.contains(r#""valuewhenStateValues":2"#));
        assert!(output.contains(r#""rollingWindowValues":2"#));
        assert!(output.contains(r#""plotChars":0"#));
        assert!(output.contains(r#""plotShapes":0"#));
        assert!(output.contains(r#""plotArrows":0"#));
        assert!(output.contains(r#""plotBars":0"#));
        assert!(output.contains(r#""plotCandles":0"#));
    }
}
