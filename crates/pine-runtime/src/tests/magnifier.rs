use pine_syntax::SourceFile;

use super::*;

fn indicator_program() -> pine_ir::HirProgram {
    let source = SourceFile::new(
        "magnifier.pine",
        r#"
indicator("magnifier ownership")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("HIR")
}

fn strategy_program_false() -> pine_ir::HirProgram {
    let source = SourceFile::new(
        "magnifier-strategy.pine",
        r#"
strategy("magnifier ownership", use_bar_magnifier=false)
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("HIR")
}

fn timed_bar(time: i64, close: f64) -> Bar {
    Bar {
        time,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

fn group(chart_bar_index: usize, bars: Vec<Bar>) -> MagnifierChartBarInput {
    MagnifierChartBarInput {
        chart_bar_index,
        bars,
    }
}

#[test]
fn historical_runtime_default_magnifier_input_is_empty() {
    let program = indicator_program();
    let runtime = HistoricalRuntime::new(&program);
    assert!(runtime.magnifier_input().is_empty());
    let baseline = runtime
        .clone()
        .run(&[timed_bar(1, 1.0), timed_bar(2, 2.0)])
        .expect("baseline");
    let with_empty = HistoricalRuntime::new(&program)
        .with_magnifier_input(MagnifierInput::new())
        .run(&[timed_bar(1, 1.0), timed_bar(2, 2.0)])
        .expect("empty magnifier");
    assert_eq!(baseline, with_empty);
}

#[test]
fn historical_runtime_owns_sparse_magnifier_input_without_changing_output() {
    let program = indicator_program();
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1, 1.1)]),
        group(2, vec![timed_bar(3, 3.1)]),
    ])
    .expect("valid");
    let runtime = HistoricalRuntime::new(&program).with_magnifier_input(input.clone());
    assert_eq!(runtime.magnifier_input(), &input);
    let cloned = runtime.clone();
    assert_eq!(cloned.magnifier_input(), &input);
    let baseline = HistoricalRuntime::new(&program)
        .run(&[timed_bar(1, 1.0), timed_bar(2, 2.0), timed_bar(3, 3.0)])
        .expect("baseline");
    let with_input = runtime
        .run(&[timed_bar(1, 1.0), timed_bar(2, 2.0), timed_bar(3, 3.0)])
        .expect("inert magnifier");
    assert_eq!(baseline, with_input);
}

#[test]
fn historical_runtime_rejects_out_of_range_magnifier_before_bar_zero() {
    let program = indicator_program();
    let input =
        magnifier_input_from_groups(vec![group(2, vec![timed_bar(1, 1.0)])]).expect("valid");
    let error = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1, 1.0), timed_bar(2, 2.0)])
        .expect_err("out of range");
    assert!(
        error.message.contains("E_MAGNIFIER_CHART_BAR_RANGE"),
        "{}",
        error.message
    );
}

#[test]
fn historical_runtime_false_setting_keeps_valid_input_inert() {
    let program = strategy_program_false();
    assert!(!program.strategy_settings.use_bar_magnifier);
    let input =
        magnifier_input_from_groups(vec![group(0, vec![timed_bar(1, 9.0)])]).expect("valid");
    let baseline = HistoricalRuntime::new(&program)
        .run(&[timed_bar(1, 1.0)])
        .expect("baseline");
    let with_input = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1, 1.0)])
        .expect("inert");
    assert_eq!(baseline, with_input);
}

#[test]
fn realtime_runtime_rejects_magnifier_group_for_forming_bar() {
    let program = indicator_program();
    let input =
        magnifier_input_from_groups(vec![group(0, vec![timed_bar(1, 9.0)])]).expect("valid");
    let error = RealtimeRuntime::new(&program)
        .with_magnifier_input(input)
        .update(BarUpdate::forming(timed_bar(1, 1.0)))
        .expect_err("forming slot");
    assert!(
        error.message.contains("E_MAGNIFIER_FORMING_BAR"),
        "{}",
        error.message
    );
}
