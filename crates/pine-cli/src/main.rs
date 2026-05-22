use std::{env, fs, process::ExitCode};

use pine_runtime::{
    Bar, PUBLIC_OUTPUT_SCHEMA_VERSION, public_runtime_profiled_result_json,
    public_runtime_result_json, run_historical,
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
                println!(
                    "{}",
                    public_runtime_profiled_result_json(&result.result, &result.profile)
                );
            } else {
                let result = run_historical(&hir, &bars)
                    .map_err(|err| format!("runtime failed: {}", err.message))?;
                println!("{}", public_runtime_result_json(&result));
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

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_runtime::{
        HistoryRetentionMode, RuntimeProfile, RuntimeResult, public_runtime_profiled_result_json,
        public_runtime_result_json,
    };
    use std::{env, fs, path::PathBuf};

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

        let output = public_runtime_result_json(&result);

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

        let output = public_runtime_profiled_result_json(&result, &profile);

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

    #[test]
    fn runtime_outputs_match_golden_snapshots() {
        for (snapshot, fixture) in [
            (
                "runtime_basic_plot.json",
                "tests/fixtures/runtime/snapshot_plot.pine",
            ),
            (
                "runtime_plotchar.json",
                "tests/fixtures/runtime/plotchar.pine",
            ),
            (
                "runtime_plotshape.json",
                "tests/fixtures/runtime/plotshape.pine",
            ),
            (
                "runtime_plotarrow.json",
                "tests/fixtures/runtime/plotarrow.pine",
            ),
            (
                "runtime_plotbar.json",
                "tests/fixtures/runtime/plotbar.pine",
            ),
            (
                "runtime_plotcandle.json",
                "tests/fixtures/runtime/plotcandle.pine",
            ),
            (
                "runtime_color_outputs.json",
                "tests/fixtures/runtime/color_outputs.pine",
            ),
            ("runtime_hline_fill.json", "tests/fixtures/runtime/io.pine"),
        ] {
            assert_snapshot(snapshot, &runtime_fixture_json(fixture));
        }
    }

    #[test]
    fn matrix_output_matches_golden_snapshot() {
        assert_snapshot("matrix.json", &matrix_json(&conformance_entries()));
    }

    fn runtime_fixture_json(fixture: &str) -> String {
        let workspace = workspace_dir();
        let source_text = fs::read_to_string(workspace.join(fixture)).expect("fixture source");
        let source = SourceFile::new(fixture, source_text);
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{fixture} diagnostics: {:?}",
            analysis.diagnostics
        );
        let bars = parse_bars_csv(include_str!("../../../tests/fixtures/runtime/bars.csv"))
            .expect("bars fixture");
        let result =
            run_historical(&analysis.hir.expect("fixture HIR"), &bars).expect("runtime result");
        public_runtime_result_json(&result)
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
