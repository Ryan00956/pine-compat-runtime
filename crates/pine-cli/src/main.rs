use std::{env, fs, process::ExitCode};

use pine_runtime::{Bar, PineValue, RuntimeProfile, RuntimeResult, run_historical};
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
    status: &'static str,
    notes: &'static str,
}

fn conformance_entries() -> Vec<MatrixEntry> {
    let mut entries: Vec<_> = pine_builtins::PHASE_1_BUILTINS
        .iter()
        .map(|signature| MatrixEntry {
            feature: signature.name.to_owned(),
            status: "supported",
            notes: "Phase 4 executable subset",
        })
        .collect();

    entries.extend([
        MatrixEntry {
            feature: "if".to_owned(),
            status: "supported",
            notes: "conditional callsites advance only when their branch executes",
        },
        MatrixEntry {
            feature: "expression-body functions".to_owned(),
            status: "supported",
            notes: "lowered by inlining; positional and named arguments supported",
        },
        MatrixEntry {
            feature: "multi-statement functions".to_owned(),
            status: "unsupported",
            notes: "requires local block and return-value semantics",
        },
        MatrixEntry {
            feature: "block-local declarations".to_owned(),
            status: "unsupported",
            notes: "declare before if blocks and reassign inside branches",
        },
        MatrixEntry {
            feature: "recursive functions".to_owned(),
            status: "unsupported",
            notes: "rejected during semantic analysis",
        },
        MatrixEntry {
            feature: "function side effects".to_owned(),
            status: "unsupported",
            notes: "plot, hline, fill, indicator, and input calls inside UDFs rejected",
        },
        MatrixEntry {
            feature: "color.* named constants".to_owned(),
            status: "partial",
            notes: "common registry only",
        },
        MatrixEntry {
            feature: "#RRGGBB/#RRGGBBAA color literals".to_owned(),
            status: "supported",
            notes: "normalized runtime color value",
        },
        MatrixEntry {
            feature: "history references".to_owned(),
            status: "partial",
            notes: "constant non-negative offsets only",
        },
        MatrixEntry {
            feature: "var".to_owned(),
            status: "supported",
            notes: "historical persistence",
        },
        MatrixEntry {
            feature: "varip".to_owned(),
            status: "unsupported",
            notes: "intrabar persistence not implemented",
        },
        MatrixEntry {
            feature: "request.*".to_owned(),
            status: "unsupported",
            notes: "multi-symbol and multi-timeframe data out of Phase 1",
        },
        MatrixEntry {
            feature: "array.*".to_owned(),
            status: "unsupported",
            notes: "array storage and mutation out of Phase 1",
        },
        MatrixEntry {
            feature: "import".to_owned(),
            status: "unsupported",
            notes: "library imports out of Phase 1",
        },
        MatrixEntry {
            feature: "strategy.*".to_owned(),
            status: "unsupported",
            notes: "broker emulation out of current scope",
        },
        MatrixEntry {
            feature: "alert/alertcondition".to_owned(),
            status: "unsupported",
            notes: "alerts out of Phase 1",
        },
        MatrixEntry {
            feature: "label/line/box/table/polyline".to_owned(),
            status: "unsupported",
            notes: "drawing object systems out of Phase 1",
        },
        MatrixEntry {
            feature: "dynamic history offsets".to_owned(),
            status: "unsupported",
            notes: "Phase 1 requires static offsets",
        },
    ]);

    entries
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
        "{:<feature_width$}  {:<status_width$}  notes\n",
        "feature", "status"
    ));
    output.push_str(&format!(
        "{:-<feature_width$}  {:-<status_width$}  -----\n",
        "", ""
    ));
    for entry in entries {
        output.push_str(&format!(
            "{:<feature_width$}  {:<status_width$}  {}\n",
            entry.feature, entry.status, entry.notes
        ));
    }
    output
}

fn matrix_json(entries: &[MatrixEntry]) -> String {
    let mut output = String::from("[");
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"feature\":\"{}\",\"status\":\"{}\",\"notes\":\"{}\"}}",
            json_escape(&entry.feature),
            json_escape(entry.status),
            json_escape(entry.notes)
        ));
    }
    output.push(']');
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
    let mut output = String::from("{");
    output.push_str("\"plots\":");
    output.push_str(&plots_json(&result.plots));
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
            "\"symbolSlots\":{},",
            "\"symbolCapacity\":{},",
            "\"currentSeriesSlots\":{},",
            "\"currentSeriesCapacity\":{},",
            "\"varSlots\":{},",
            "\"varCapacity\":{},",
            "\"callStateSlots\":{},",
            "\"callStateCapacity\":{},",
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
        profile.symbol_slots,
        profile.symbol_capacity,
        profile.current_series_slots,
        profile.current_series_capacity,
        profile.var_slots,
        profile.var_capacity,
        profile.call_state_slots,
        profile.call_state_capacity,
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
        profile.hlines,
        profile.hline_capacity,
        profile.fills,
        profile.fill_capacity
    )
}

fn plots_json(plots: &[pine_runtime::PlotSeries]) -> String {
    let mut output = String::from("[");
    for (plot_index, plot) in plots.iter().enumerate() {
        if plot_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot.id));
        for (value_index, value) in plot.values.iter().enumerate() {
            if value_index > 0 {
                output.push(',');
            }
            output.push_str(&value_json(value));
        }
        output.push_str("]}");
    }
    output.push(']');
    output
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
        PineValue::Na | PineValue::Void => "null".to_owned(),
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

        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "math.max" && entry.status == "supported")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "ta.macd" && entry.status == "supported")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.feature == "if" && entry.status == "supported")
        );
        assert!(entries.iter().any(|entry| {
            entry.feature == "expression-body functions" && entry.status == "supported"
        }));
        assert!(entries.iter().any(|entry| {
            entry.feature == "multi-statement functions" && entry.status == "unsupported"
        }));
        assert!(entries.iter().any(|entry| {
            entry.feature == "block-local declarations" && entry.status == "unsupported"
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
    fn formats_matrix_as_text() {
        let entries = vec![MatrixEntry {
            feature: "indicator".to_owned(),
            status: "supported",
            notes: "Phase 4 executable subset",
        }];

        let output = matrix_text(&entries);

        assert!(output.contains("feature"));
        assert!(output.contains("indicator"));
        assert!(output.contains("supported"));
    }

    #[test]
    fn formats_matrix_as_json() {
        let entries = vec![MatrixEntry {
            feature: "request.*".to_owned(),
            status: "unsupported",
            notes: "multi-symbol",
        }];

        let output = matrix_json(&entries);

        assert_eq!(
            output,
            r#"[{"feature":"request.*","status":"unsupported","notes":"multi-symbol"}]"#
        );
    }

    #[test]
    fn formats_profiled_result_json() {
        let result = RuntimeResult {
            plots: vec![],
            hlines: vec![],
            fills: vec![],
            diagnostics: vec![],
        };
        let profile = RuntimeProfile {
            bars: 3,
            series_buffers: 2,
            series_values: 6,
            series_capacity: 8,
            symbol_slots: 10,
            symbol_capacity: 14,
            current_series_slots: 0,
            current_series_capacity: 14,
            var_slots: 1,
            var_capacity: 3,
            call_state_slots: 1,
            call_state_capacity: 3,
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
            hlines: 0,
            hline_capacity: 0,
            fills: 0,
            fill_capacity: 0,
        };

        let output = profiled_result_json(&result, &profile);

        assert!(output.contains(r#""profile""#));
        assert!(output.contains(r#""bars":3"#));
        assert!(output.contains(r#""seriesValues":6"#));
        assert!(output.contains(r#""rollingWindowValues":2"#));
    }
}
