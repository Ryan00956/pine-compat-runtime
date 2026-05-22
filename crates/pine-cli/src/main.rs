use std::{env, process::ExitCode};

mod bars_csv;
mod commands;
mod conformance;
mod json;

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

    match command.as_str() {
        "analyze" => commands::analyze::run(args.collect()),
        "fmt-ast" => commands::fmt_ast::run(args.collect()),
        "run" => commands::run::run(args.collect()),
        "matrix" => commands::matrix::run(args.collect()),
        _ => Err(usage()),
    }
}

pub(crate) fn usage() -> String {
    "usage: pine-compat <analyze|fmt-ast> <script.pine>\n       pine-compat run <script.pine> --bars <bars.csv> [--profile]\n       pine-compat matrix [--format text|json]".to_owned()
}

#[cfg(test)]
mod tests {
    use crate::bars_csv::parse_bars_csv;
    use crate::commands::matrix::{matrix_json, matrix_text};
    use crate::conformance::{
        MatrixEntry, conformance_entries, try_conformance_entries_from_tsv, validate_fixture_paths,
    };
    use pine_runtime::{
        HistoryRetentionMode, RuntimeProfile, RuntimeResult, public_runtime_profiled_result_json,
        public_runtime_result_json, run_historical,
    };
    use pine_sema::analyze_source;
    use pine_syntax::SourceFile;
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
        validate_fixture_paths(&conformance_entries(), &workspace_dir())
            .expect("fixture paths should exist");
    }

    #[test]
    fn rejects_malformed_conformance_rows() {
        let error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tsupported\tmissing fixture column\n",
        )
        .expect_err("row should be malformed");

        assert!(error.contains("expected 4 tab-separated columns"));
    }

    #[test]
    fn rejects_duplicate_conformance_features() {
        let error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tsupported\tone\ttests/fixtures/runtime/io.pine\nindicator\tsupported\ttwo\ttests/fixtures/runtime/io.pine\n",
        )
        .expect_err("feature should be duplicate");

        assert!(error.contains("duplicate feature `indicator`"));
    }

    #[test]
    fn rejects_invalid_conformance_statuses() {
        let error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tready\tnotes\ttests/fixtures/runtime/io.pine\n",
        )
        .expect_err("status should be invalid");

        assert!(error.contains("invalid status `ready`"));
    }

    #[test]
    fn rejects_empty_conformance_fixtures() {
        let error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tsupported\tnotes\t\n",
        )
        .expect_err("fixtures should be empty");

        assert!(error.contains("fixtures must list at least one path"));
    }

    #[test]
    fn rejects_missing_conformance_fixture_paths() {
        let entries = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tsupported\tnotes\ttests/fixtures/runtime/missing.pine\n",
        )
        .expect("metadata shape should be valid");
        let error = validate_fixture_paths(&entries, &workspace_dir())
            .expect_err("fixture should be missing");

        assert!(error.contains("tests/fixtures/runtime/missing.pine fixture path should exist"));
    }

    #[test]
    fn rejects_status_fixture_mismatches() {
        let unsupported_error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nrequest.*\tunsupported\tnotes\ttests/fixtures/runtime/io.pine\n",
        )
        .expect_err("unsupported feature should require unsupported sema fixture");
        assert!(unsupported_error.contains("unsupported sema diagnostic fixture coverage"));

        let supported_error = try_conformance_entries_from_tsv(
            "feature\tstatus\tnotes\tfixtures\nindicator\tsupported\tnotes\ttests/fixtures/sema/unsupported_request.pine\n",
        )
        .expect_err("supported feature should require executable or positive fixture");
        assert!(supported_error.contains("must reference runtime"));
    }

    #[test]
    fn matrix_includes_known_unsupported_platform_families() {
        let entries = conformance_entries();
        for feature in [
            "varip",
            "request.*",
            "import",
            "strategy.*",
            "alert/alertcondition",
            "label/line/box/table/polyline",
            "non-int history offsets",
            "negative history offsets",
        ] {
            assert!(
                entries
                    .iter()
                    .any(|entry| entry.feature == feature && entry.status == "unsupported"),
                "{feature} should remain explicitly unsupported in matrix"
            );
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
            r#"{"schemaVersion":2,"features":[{"feature":"request.*","status":"unsupported","notes":"multi-symbol","fixtures":["tests/fixtures/sema/unsupported_request.pine"]}]}"#
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
            labels: vec![],
            diagnostics: vec![],
        };

        let output = public_runtime_result_json(&result);

        assert!(output.starts_with(r#"{"schemaVersion":2,"#));
        assert!(output.contains(r#""labels":[]"#));
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
            labels: vec![],
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
            labels: 0,
            label_snapshots: 0,
            label_capacity: 0,
            label_snapshot_capacity: 0,
        };

        let output = public_runtime_profiled_result_json(&result, &profile);

        assert!(output.starts_with(r#"{"schemaVersion":2,"#));
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
        assert!(output.contains(r#""labels":0"#));
        assert!(output.contains(r#""labelSnapshots":0"#));
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
