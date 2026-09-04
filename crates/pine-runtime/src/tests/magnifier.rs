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

fn enabled_strategy(source: &str) -> pine_ir::HirProgram {
    let file = SourceFile::new("magnifier-enabled.pine", source);
    let analysis = analyze_source(&file);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.expect("HIR");
    assert!(
        program.strategy_settings.use_bar_magnifier,
        "enabled_strategy sources must set use_bar_magnifier=true"
    );
    program
}

fn ohlc(time: i64, open: f64, high: f64, low: f64, close: f64) -> Bar {
    Bar {
        time,
        open,
        high,
        low,
        close,
        volume: 1.0,
    }
}

#[test]
fn magnifier_three_lower_bars_walk_independent_host_paths() {
    use crate::runtime::strategy_scheduler::StrategyPathPhase;

    let program = enabled_strategy(
        r#"
strategy("three lower bars", use_bar_magnifier=true)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1_000, 10.0)]),
        group(
            1,
            vec![
                ohlc(2_000, 10.0, 10.4, 9.8, 10.2),
                ohlc(2_300, 10.2, 10.8, 10.1, 10.6),
                ohlc(2_600, 11.0, 11.8, 10.5, 11.0),
            ],
        ),
    ])
    .expect("valid");
    let mut runtime = HistoricalRuntime::new(&program).with_magnifier_input(input);
    runtime
        .append_bars(&[timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)])
        .expect("run");
    let hosts: Vec<_> = runtime
        .strategy_path_trace
        .iter()
        .filter(|entry| entry.chart_bar_index == 1)
        .map(|entry| entry.host_bar_index)
        .collect();
    assert!(hosts.contains(&0));
    assert!(hosts.contains(&1));
    assert!(hosts.contains(&2));
    assert!(
        runtime.strategy_path_trace.iter().any(|entry| {
            entry.chart_bar_index == 1
                && entry.host_bar_index == 2
                && entry.path_phase == StrategyPathPhase::HostOpen
                && (entry.mark - 11.0).abs() < 1e-10
        }),
        "gap 10.6 -> 11.0 must be a host-open point: {:?}",
        runtime.strategy_path_trace
    );
    let fallback = runtime
        .result()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.starts_with("W_MAGNIFIER"));
    assert!(!fallback);
}

#[test]
fn magnifier_empty_group_falls_back_once_and_keeps_chart_identity() {
    let program = enabled_strategy(
        r#"
strategy("empty group", use_bar_magnifier=true)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1, 1.0)]),
        MagnifierChartBarInput {
            chart_bar_index: 1,
            bars: Vec::new(),
        },
    ])
    .expect("valid");
    let result = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1_000, 1.0), timed_bar(2_000, 2.0)])
        .expect("run");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    assert_eq!(warnings, vec!["W_MAGNIFIER_GAP"]);
}

#[test]
fn magnifier_doji_lower_bar_terminates_without_replay() {
    let program = enabled_strategy(
        r#"
strategy("doji", use_bar_magnifier=true)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![group(
        0,
        vec![
            ohlc(1, 10.0, 10.0, 10.0, 10.0),
            ohlc(2, 10.0, 10.0, 10.0, 10.0),
        ],
    )])
    .expect("valid");
    HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1, 10.0)])
        .expect("doji walk must terminate");
}

#[test]
fn magnifier_fill_resumes_from_current_host_leg_without_replay() {
    use crate::runtime::strategy_scheduler::StrategyPathPhase;

    let program = enabled_strategy(
        r#"
strategy("resume", calc_on_order_fills=true, initial_capital=100000, use_bar_magnifier=true)
if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
if strategy.position_size > 0
    strategy.exit("EX", "EN", limit=11.5)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![group(
        1,
        vec![
            ohlc(2_000, 10.0, 10.4, 9.8, 10.2),
            ohlc(2_300, 10.2, 10.8, 10.1, 10.6),
            ohlc(2_600, 10.6, 11.8, 10.5, 11.0),
        ],
    )])
    .expect("valid");
    let mut runtime = HistoricalRuntime::new(&program).with_magnifier_input(input);
    runtime
        .append_bars(&[timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)])
        .expect("run");
    let result = runtime.result();
    let strategy = result.strategy.expect("strategy");
    assert!(
        strategy
            .orders
            .iter()
            .any(|order| order.id == "EN" && order.bar_index == 1),
        "{:?}",
        strategy.orders
    );
    let after_entry: Vec<_> = runtime
        .strategy_path_trace
        .iter()
        .filter(|entry| entry.chart_bar_index == 1)
        .map(|entry| entry.host_bar_index)
        .collect();
    assert!(after_entry.contains(&1));
    assert!(after_entry.contains(&2));
    assert!(
        runtime
            .strategy_path_trace
            .iter()
            .any(|entry| entry.chart_bar_index == 1
                && entry.host_bar_index == 1
                && entry.path_phase == StrategyPathPhase::PathLeg),
        "{:?}",
        runtime.strategy_path_trace
    );
}

#[test]
fn magnifier_gap_fills_stop_at_next_open_not_requested_price() {
    let program = enabled_strategy(
        r#"
strategy("gap stop", pyramiding=2, initial_capital=100000, use_bar_magnifier=true)
if bar_index == 0
    strategy.entry("STP", strategy.long, qty=1, stop=10.5)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1_000, 10.0)]),
        group(
            1,
            vec![
                ohlc(2_000, 10.0, 10.2, 9.9, 10.1),
                ohlc(2_300, 11.0, 11.2, 10.8, 11.1),
            ],
        ),
    ])
    .expect("valid");
    let result = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)])
        .expect("run");
    let strategy = result.strategy.expect("strategy");
    let fill = strategy
        .orders
        .iter()
        .find(|order| order.id == "STP")
        .expect("stop fill");
    assert_eq!(fill.bar_index, 1);
    assert!((fill.price - 11.0).abs() < 1e-10, "{fill:?}");
    assert_eq!(fill.time, 2_000);
}

#[test]
fn magnifier_same_chart_bar_entry_and_exit_uses_chart_bar_index() {
    let program = enabled_strategy(
        r#"
strategy("same bar", initial_capital=100000, use_bar_magnifier=true)
if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
    strategy.exit("EX", "EN", limit=11.5)
plot(close)
"#,
    );
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1_000, 10.0)]),
        group(
            1,
            vec![
                ohlc(2_000, 10.0, 10.4, 9.8, 10.2),
                ohlc(2_300, 10.2, 10.8, 10.1, 10.6),
                ohlc(2_600, 10.6, 11.8, 10.5, 11.0),
            ],
        ),
    ])
    .expect("valid");
    let result = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&[timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)])
        .expect("run");
    let strategy = result.strategy.expect("strategy");
    let ids: Vec<_> = strategy
        .orders
        .iter()
        .map(|order| order.id.as_str())
        .collect();
    assert!(ids.contains(&"EN"), "{ids:?}");
    assert!(ids.contains(&"EX"), "{ids:?}");
    assert!(
        strategy.orders.iter().all(|order| order.bar_index == 1),
        "{:?}",
        strategy.orders
    );
}

#[test]
fn magnifier_calc_on_order_fills_false_does_not_add_script_passes() {
    let with_fills = enabled_strategy(
        r#"
strategy("fills on", calc_on_order_fills=true, initial_capital=100000, use_bar_magnifier=true)
if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
if strategy.position_size > 0
    strategy.exit("EX", "EN", limit=11.5)
plot(close)
"#,
    );
    let without_fills = enabled_strategy(
        r#"
strategy("fills off", calc_on_order_fills=false, initial_capital=100000, use_bar_magnifier=true)
if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
if strategy.position_size > 0
    strategy.exit("EX", "EN", limit=11.5)
plot(close)
"#,
    );
    let bars = [timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)];
    let input = magnifier_input_from_groups(vec![
        group(0, vec![timed_bar(1_000, 10.0)]),
        group(
            1,
            vec![
                ohlc(2_000, 10.0, 10.4, 9.8, 10.2),
                ohlc(2_300, 10.2, 10.8, 10.1, 10.6),
                ohlc(2_600, 10.6, 11.8, 10.5, 11.0),
            ],
        ),
    ])
    .expect("valid");
    let mut on = HistoricalRuntime::new(&with_fills).with_magnifier_input(input.clone());
    on.append_bars(&bars).expect("on");
    let mut off = HistoricalRuntime::new(&without_fills).with_magnifier_input(input);
    off.append_bars(&bars).expect("off");
    assert!(
        on.strategy_scheduler.script_passes() > off.strategy_scheduler.script_passes(),
        "on={} off={}",
        on.strategy_scheduler.script_passes(),
        off.strategy_scheduler.script_passes()
    );
    assert_eq!(off.strategy_scheduler.recalculation_passes(), 0);
}

#[test]
fn magnifier_false_setting_matches_standard_ohlc_baseline() {
    let source = r#"
strategy("baseline", initial_capital=100000)
if bar_index == 0
    strategy.entry("EN", strategy.long, qty=1, stop=10.5)
plot(close)
"#;
    let file = SourceFile::new("baseline.pine", source);
    let analysis = analyze_source(&file);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.expect("HIR");
    assert!(!program.strategy_settings.use_bar_magnifier);
    let bars = [timed_bar(1_000, 10.0), ohlc(2_000, 10.0, 12.0, 8.0, 11.0)];
    let baseline = HistoricalRuntime::new(&program)
        .run(&bars)
        .expect("baseline");
    let input =
        magnifier_input_from_groups(vec![group(1, vec![ohlc(2_000, 10.0, 10.1, 9.9, 10.0)])])
            .expect("valid");
    let with_input = HistoricalRuntime::new(&program)
        .with_magnifier_input(input)
        .run(&bars)
        .expect("inert");
    assert_eq!(baseline, with_input);
}
