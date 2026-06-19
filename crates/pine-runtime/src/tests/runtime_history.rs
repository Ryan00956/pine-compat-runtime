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

#[test]
fn runs_official_array_history_example_shape() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("History referencing")
a = array.new<float>(1)
array.set(a, 0, close)
previous = a[1]
previousClose1 = na(previous) ? na : previous.get(0)
previousClose2 = close[1]
plot(previousClose1)
plot(previousClose2)
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
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values[1..], &[1.0, 2.0, 3.0]);
}

#[test]
fn reads_previous_label_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label array history")
current = label.new(bar_index, close, "id")
ids = array.new_label(1)
ids.set(0, current)
previous_ids = ids[1]
previous_id = na(previous_ids) ? na : previous_ids.get(0)
plot(na(previous_id) ? na : label.get_x(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_line_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("line array history")
current = line.new(bar_index, close, bar_index + 1, high)
ids = array.new_line(1)
ids.set(0, current)
previous_ids = ids[1]
previous_id = na(previous_ids) ? na : previous_ids.get(0)
plot(na(previous_id) ? na : line.get_x1(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_box_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("box array history")
current = box.new(bar_index, high, bar_index + 1, low)
ids = array.new_box(1)
ids.set(0, current)
previous_ids = ids[1]
previous_id = na(previous_ids) ? na : previous_ids.get(0)
plot(na(previous_id) ? na : box.get_left(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_linefill_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("linefill array history")
upper = line.new(bar_index, high, bar_index + 1, high)
lower = line.new(bar_index, low, bar_index + 1, low)
current = linefill.new(upper, lower, color.green)
ids = array.new_linefill(1)
ids.set(0, current)
previous_ids = ids[1]
previous_id = na(previous_ids) ? na : previous_ids.get(0)
previous_line = na(previous_id) ? na : linefill.get_line1(previous_id)
plot(na(previous_line) ? na : line.get_x1(previous_line))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_table_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("table array history")
current = table.new(position.top_right, 1, 1)
ids = array.new_table(1)
ids.set(0, current)
previous_ids = ids[1]
previous_id = na(previous_ids) ? na : previous_ids.get(0)
visible = table.all
plot(na(previous_id) ? na : array.indexof(visible, previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_chart_point_array_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("chart point array history")
current = chart.point.from_index(bar_index, close)
ids = array.new<chart.point>(1)
ids.set(0, current)
previous_ids = ids[1]
previous_point = na(previous_ids) ? na : previous_ids.get(0)
plot(na(previous_point) ? na : previous_point.index)
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array slice history")
source = array.new_float(2)
source.set(0, close)
source.set(1, high)
window = source.slice(0, 1)
previous_window = window[1]
plot(na(previous_window) ? na : previous_window.get(0))
if not na(previous_window)
    previous_window.set(0, 100)
plot(window.get(0))
plot(source.get(0))
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

    assert_eq!(result.plots.len(), 3);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0, 4.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn reads_previous_label_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label array slice history")
current = label.new(bar_index, close, "id")
source = array.new_label(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_id = na(previous_window) ? na : previous_window.get(0)
plot(na(previous_id) ? na : label.get_x(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_line_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("line array slice history")
current = line.new(bar_index, close, bar_index + 1, high)
source = array.new_line(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_id = na(previous_window) ? na : previous_window.get(0)
plot(na(previous_id) ? na : line.get_x1(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_box_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("box array slice history")
current = box.new(bar_index, high, bar_index + 1, low)
source = array.new_box(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_id = na(previous_window) ? na : previous_window.get(0)
plot(na(previous_id) ? na : box.get_left(previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_linefill_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("linefill array slice history")
upper = line.new(bar_index, high, bar_index + 1, high)
lower = line.new(bar_index, low, bar_index + 1, low)
current = linefill.new(upper, lower, color.green)
source = array.new_linefill(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_id = na(previous_window) ? na : previous_window.get(0)
previous_line = na(previous_id) ? na : linefill.get_line1(previous_id)
plot(na(previous_line) ? na : line.get_x1(previous_line))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_table_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("table array slice history")
current = table.new(position.top_right, 1, 1)
source = array.new_table(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_id = na(previous_window) ? na : previous_window.get(0)
visible = table.all
plot(na(previous_id) ? na : array.indexof(visible, previous_id))
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}

#[test]
fn reads_previous_chart_point_array_slice_instance_history() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("chart point array slice history")
current = chart.point.from_index(bar_index, close)
source = array.new<chart.point>(2)
source.set(0, current)
window = source.slice(0, 1)
previous_window = window[1]
previous_point = na(previous_window) ? na : previous_window.get(0)
plot(na(previous_point) ? na : previous_point.index)
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
    assert_values_close(&result.plots[0].values[1..], &[0.0, 1.0, 2.0]);
}
