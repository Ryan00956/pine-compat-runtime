use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn stores_expression_history_before_reading_previous_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("expression history")
plot((close + open)[1])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 4.0, 3.0, 4.0),
        bar_ohlc(5.0, 6.0, 5.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.0, 7.0]);
}

#[test]
fn runs_simple_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("simple history")
var values = array.new_int()
array.push(values, 1)
offset = math.min(array.size(values), 1)
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0]);
}

#[test]
fn runs_series_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("series history")
offset = bar_index == 0 ? 0 : 1
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_values_close(&profiled.result.plots[0].values, &[1.0, 1.0, 2.0, 3.0]);
    assert_eq!(profiled.profile.max_series_depth, 4);
}

#[test]
fn series_history_offset_out_of_range_returns_na() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("series history out of range")
plot(close[bar_index + 1])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values, vec![PineValue::Na; 3]);
}

#[test]
fn rejects_negative_dynamic_history_offset_at_runtime() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("negative dynamic history")
values = array.new_int()
offset = array.indexof(values, 1)
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("runtime should reject negative dynamic history offset");
    assert!(error.message.contains("non-negative"), "{}", error.message);
}

#[test]
fn runs_input_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("input history")
length = input.int(2, "Length")
plot(close[length])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[1.0, 2.0]);
}

#[test]
fn reads_previous_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array history")
var values = array.new_float(1)
values.set(0, close)
previous = values[1]
plot(bar_index == 0 ? na : previous.get(0))
if bar_index > 0
    previous.set(0, 100)
plot(values.get(0))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0, 4.0]);
}
