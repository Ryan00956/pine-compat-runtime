use pine_syntax::SourceFile;

use super::*;

fn compile_fixture(name: &str, source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new(name, source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{name}: {:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("legacy fixture HIR")
}

#[test]
fn v4_alias_fixture_matches_canonical_historical_output() {
    let legacy = compile_fixture(
        "aliases_legacy.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/aliases_legacy.pine"),
    );
    let canonical = compile_fixture(
        "aliases_canonical.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/aliases_canonical.pine"),
    );
    let bars = [
        bar_ohlc(10.0, 12.0, 9.0, 11.0),
        bar_ohlc(11.0, 14.0, 10.0, 13.0),
        bar_ohlc(13.0, 15.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 13.0, 14.0),
        bar_ohlc(14.0, 18.0, 14.0, 17.0),
    ];

    let legacy_result = run_historical(&legacy, &bars).expect("legacy v4 run");
    let canonical_result = run_historical(&canonical, &bars).expect("canonical run");

    assert_eq!(legacy_result, canonical_result);
}

#[test]
fn v4_input_fixture_preserves_metadata_callsites_and_overrides() {
    let legacy = compile_fixture(
        "inputs_legacy.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/inputs_legacy.pine"),
    );
    let canonical = compile_fixture(
        "inputs_canonical.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/inputs_canonical.pine"),
    );
    let legacy_inputs = input_calls(&legacy);
    let canonical_inputs = input_calls(&canonical);
    assert_eq!(legacy_inputs, canonical_inputs);
    assert_eq!(legacy_inputs.len(), 11);
    assert_eq!(
        legacy_inputs
            .iter()
            .map(|input| input.call_site_id)
            .collect::<Vec<_>>(),
        (1..=11).collect::<Vec<_>>()
    );

    let bars = [
        Bar {
            time: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1.0,
        },
        Bar {
            time: 2,
            open: 11.0,
            high: 14.0,
            low: 10.0,
            close: 13.0,
            volume: 1.0,
        },
        Bar {
            time: 3,
            open: 13.0,
            high: 15.0,
            low: 11.0,
            close: 12.0,
            volume: 1.0,
        },
        Bar {
            time: 4,
            open: 12.0,
            high: 16.0,
            low: 12.0,
            close: 15.0,
            volume: 1.0,
        },
    ];

    assert_eq!(
        run_historical(&legacy, &bars).expect("legacy default input run"),
        run_historical(&canonical, &bars).expect("canonical default input run")
    );

    let call_site = |title: &str| {
        legacy_inputs
            .iter()
            .find(|input| input.title.as_deref() == Some(title))
            .map(|input| input.call_site_id)
            .unwrap_or_else(|| panic!("missing input title {title}"))
    };
    let overrides = InputOverrides::new()
        .with_value(call_site("Length"), PineValue::Int(1))
        .with_value(call_site("Scale"), PineValue::Float(2.0))
        .with_value(call_site("Enabled"), PineValue::Bool(true))
        .with_value(call_site("Shade"), PineValue::Color(0x4CAF50))
        .with_value(call_site("Mode"), PineValue::String("SMA".to_owned()))
        .with_value(call_site("Symbol"), PineValue::String("AAPL".to_owned()))
        .with_value(call_site("Resolution"), PineValue::String("60".to_owned()))
        .with_value(
            call_site("Session"),
            PineValue::String("0930-1600".to_owned()),
        )
        .with_value(call_site("Start"), PineValue::Int(0))
        .with_value(call_site("Price"), PineValue::Float(1.0));
    let legacy_override = run_historical_with_input_overrides(&legacy, &bars, overrides.clone())
        .expect("legacy override run");
    let canonical_override = run_historical_with_input_overrides(&canonical, &bars, overrides)
        .expect("canonical override run");
    assert_eq!(legacy_override, canonical_override);
    assert_values_close(&legacy_override.plots[0].values, &[23.0, 27.0, 25.0, 31.0]);
}

#[test]
fn v4_output_fixture_matches_canonical_visual_data_and_metadata() {
    let legacy = compile_fixture(
        "outputs_legacy.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/outputs_legacy.pine"),
    );
    let canonical = compile_fixture(
        "outputs_canonical.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/outputs_canonical.pine"),
    );
    let bars = [
        bar_ohlc(10.0, 12.0, 9.0, 11.0),
        bar_ohlc(12.0, 13.0, 9.0, 10.0),
        bar_ohlc(10.0, 15.0, 10.0, 14.0),
    ];

    let legacy_result = run_historical(&legacy, &bars).expect("legacy output run");
    let canonical_result = run_historical(&canonical, &bars).expect("canonical output run");
    assert_eq!(legacy_result, canonical_result);

    assert_eq!(
        legacy_result.plots[0].style,
        PineValue::String("plot.style_columns".to_owned())
    );
    assert_eq!(
        legacy_result.plots[0].colors,
        vec![PineValue::Color(0x2196F399); bars.len()]
    );
    assert_eq!(
        legacy_result.plots[1].colors,
        vec![PineValue::Color(0xF23645CC); bars.len()]
    );
    assert_eq!(
        legacy_result.hlines[0].style,
        PineValue::String("hline.style_dotted".to_owned())
    );
    assert_eq!(
        legacy_result.fills[0].colors,
        vec![PineValue::Color(0x4CAF501A); bars.len()]
    );
    assert_eq!(
        legacy_result.bg_colors[0].values,
        vec![
            PineValue::Color(0x2196F31A),
            PineValue::Na,
            PineValue::Color(0x2196F31A),
        ]
    );
    assert_eq!(legacy_result.plots[0].metadata.offset, PineValue::Int(1));
    assert_eq!(
        legacy_result.plot_chars[0].metadata.show_last,
        PineValue::Int(2)
    );
}

#[test]
fn v4_dynamic_output_style_override_rejects_unknown_ordinals_at_runtime() {
    let legacy = compile_fixture(
        "outputs_legacy.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/outputs_legacy.pine"),
    );
    let style_call = input_calls(&legacy)
        .into_iter()
        .find(|input| input.title.as_deref() == Some("Plot style"))
        .expect("style input");
    let overrides = InputOverrides::new().with_value(style_call.call_site_id, PineValue::Int(99));
    let error = run_historical_with_input_overrides(&legacy, &[bar(1.0)], overrides)
        .expect_err("unknown v4 style ordinal must fail");

    assert!(
        error
            .message
            .contains("invalid Pine v4 plot style ordinal `99`")
    );
}

#[test]
fn v4_output_transparency_clamps_preserves_na_and_accepts_input_values() {
    let program = compile_fixture(
        "legacy_transparency_edges.pine",
        r#"//@version=4
study("transparency edges")
t = input(40, "Transparency")
plot(close, color=color.blue, transp=t)
plot(close, color=na, transp=40)
plot(close, color=color.blue, transp=na)
"#,
    );
    let transparency_call = input_calls(&program)
        .into_iter()
        .find(|input| input.title.as_deref() == Some("Transparency"))
        .expect("transparency input");

    let default_result = run_historical(&program, &[bar(1.0)]).expect("default transp run");
    assert_eq!(
        default_result.plots[0].colors,
        vec![PineValue::Color(0x2196F399)]
    );
    assert_eq!(default_result.plots[1].colors, vec![PineValue::Na]);
    assert_eq!(
        default_result.plots[2].colors,
        vec![PineValue::Color(0x2196F3)]
    );

    for (transp, expected) in [(-10, 0x2196F3), (120, 0x2196F300)] {
        let overrides = InputOverrides::new()
            .with_value(transparency_call.call_site_id, PineValue::Int(transp));
        let result = run_historical_with_input_overrides(&program, &[bar(1.0)], overrides)
            .expect("clamped transp run");
        assert_eq!(result.plots[0].colors, vec![PineValue::Color(expected)]);
    }
}

#[test]
fn v4_iff_eagerly_advances_both_stateful_results_while_ternary_stays_lazy() {
    let iff = compile_fixture(
        "legacy_iff_stateful.pine",
        r#"//@version=4
study("stateful iff")
plot(iff(close > open, ema(high, 2), ema(low, 2)))
"#,
    );
    let ternary = compile_fixture(
        "legacy_ternary_stateful.pine",
        r#"//@version=4
study("lazy ternary")
plot(close > open ? ema(high, 2) : ema(low, 2))
"#,
    );
    let bars = [
        bar_ohlc(1.0, 10.0, 1.0, 2.0),
        bar_ohlc(2.0, 20.0, 2.0, 3.0),
        bar_ohlc(3.0, 15.0, 5.0, 2.0),
        bar_ohlc(4.0, 14.0, 6.0, 3.0),
    ];

    let iff_result = run_historical(&iff, &bars).expect("strict iff run");
    let ternary_result = run_historical(&ternary, &bars).expect("lazy ternary run");
    assert_values_close(
        &iff_result.plots[0].values,
        &[
            10.0,
            16.666_666_666_666_664,
            3.888_888_888_888_889,
            5.296_296_296_296_296,
        ],
    );
    assert_values_close(
        &ternary_result.plots[0].values,
        &[10.0, 16.666_666_666_666_664, 5.0, 5.666_666_666_666_666],
    );
}

#[test]
fn v4_named_iff_evaluates_by_parameter_role_once() {
    let program = compile_fixture(
        "legacy_iff_named.pine",
        r#"//@version=4
study("named iff")
plot(iff(result2=lowest(low, 2), condition=close > open, result1=highest(high, 2)))
"#,
    );
    let bars = [
        bar_ohlc(1.0, 10.0, 1.0, 2.0),
        bar_ohlc(2.0, 20.0, 2.0, 3.0),
        bar_ohlc(3.0, 15.0, 5.0, 2.0),
    ];
    let result = run_historical(&program, &bars).expect("named iff run");
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Na, PineValue::Float(20.0), PineValue::Float(2.0),]
    );
}

#[test]
fn v4_offset_matches_history_for_constant_and_dynamic_offsets() {
    let legacy = compile_fixture(
        "legacy_offset.pine",
        r#"//@version=4
study("legacy offset", max_bars_back=3)
bars = input(1, "Bars")
plot(offset(close, bars))
"#,
    );
    let canonical = compile_fixture(
        "canonical_history.pine",
        r#"//@version=5
indicator("canonical history", max_bars_back=3)
bars = input.int(1, "Bars")
plot(close[bars])
"#,
    );
    assert!(legacy.history.has_dynamic_offsets);
    assert_eq!(legacy.max_bars_back, Some(3));
    let bars = [bar(10.0), bar(20.0), bar(30.0), bar(40.0)];
    assert_eq!(
        run_historical(&legacy, &bars).expect("legacy offset run"),
        run_historical(&canonical, &bars).expect("canonical history run")
    );

    let call_site_id = input_calls(&legacy)[0].call_site_id;
    let overridden = run_historical_with_input_overrides(
        &legacy,
        &bars,
        InputOverrides::new().with_value(call_site_id, PineValue::Int(2)),
    )
    .expect("overridden legacy offset run");
    assert_eq!(
        overridden.plots[0].values,
        vec![
            PineValue::Na,
            PineValue::Na,
            PineValue::Float(10.0),
            PineValue::Float(20.0)
        ]
    );

    let error = run_historical_with_input_overrides(
        &legacy,
        &bars,
        InputOverrides::new().with_value(call_site_id, PineValue::Int(-1)),
    )
    .expect_err("negative legacy dynamic offset must use the history guard");
    assert!(
        error
            .message
            .contains("history offset must be non-negative")
    );
}

#[test]
fn v4_rsi_selects_length_and_two_series_overloads_with_formula_edges() {
    let legacy_length = compile_fixture(
        "legacy_rsi_length.pine",
        r#"//@version=4
study("legacy RSI length")
plot(rsi(close, 2))
"#,
    );
    let canonical_length = compile_fixture(
        "canonical_rsi_length.pine",
        r#"//@version=5
indicator("canonical RSI length")
plot(ta.rsi(close, 2))
"#,
    );
    let legacy_series = compile_fixture(
        "legacy_rsi_series.pine",
        r#"//@version=4
study("legacy RSI series")
plot(rsi(close, open))
"#,
    );
    let canonical_formula = compile_fixture(
        "canonical_rsi_formula.pine",
        r#"//@version=5
indicator("canonical RSI formula")
plot(100.0 - (100.0 / (1.0 + close / open)))
"#,
    );
    let bars = [
        bar_ohlc(2.0, 2.0, 2.0, 2.0),
        bar_ohlc(2.0, 4.0, 1.0, 4.0),
        bar_ohlc(2.0, 2.0, 0.0, 0.0),
        bar_ohlc(0.0, 2.0, 0.0, 2.0),
    ];

    assert_eq!(
        run_historical(&legacy_length, &bars)
            .expect("legacy length RSI")
            .plots[0]
            .values,
        run_historical(&canonical_length, &bars)
            .expect("canonical length RSI")
            .plots[0]
            .values
    );
    assert_eq!(
        run_historical(&legacy_series, &bars)
            .expect("legacy series RSI")
            .plots[0]
            .values,
        run_historical(&canonical_formula, &bars)
            .expect("canonical RSI formula")
            .plots[0]
            .values
    );
}

#[test]
fn v4_session_defaults_exclude_weekends_without_rewriting_input_strings() {
    let v4 = compile_fixture(
        "legacy_session_days.pine",
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/session_defaults_legacy.pine"),
    );
    let v5 = compile_fixture(
        "modern_session_days.pine",
        r#"//@version=5
indicator("modern session days")
s = input.session("0000-2359", "Session")
plot(na(time("D", s, "UTC")) ? 0 : 1)
plot(na(time_close("D", s, "UTC")) ? 0 : 1)
plot(s == "0000-2359" ? 1 : 0)
"#,
    );
    let dated_bar = |time| Bar {
        time,
        open: 1.0,
        high: 1.0,
        low: 1.0,
        close: 1.0,
        volume: 1.0,
    };
    let bars = [
        dated_bar(1_609_459_200_000),
        dated_bar(1_609_545_600_000),
        dated_bar(1_609_632_000_000),
        dated_bar(1_609_718_400_000),
    ];

    let legacy = run_historical(&v4, &bars).expect("legacy session run");
    let modern = run_historical(&v5, &bars).expect("modern session run");
    assert_values_close(&legacy.plots[0].values, &[1.0, 0.0, 0.0, 1.0]);
    assert_values_close(&legacy.plots[1].values, &[1.0, 0.0, 0.0, 1.0]);
    assert_values_close(&legacy.plots[2].values, &[1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&modern.plots[0].values, &[1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&modern.plots[1].values, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn v4_phase6_semantics_match_incremental_and_realtime_confirmed_execution() {
    let program = compile_fixture(
        "legacy_phase6_realtime.pine",
        r#"//@version=4
study("legacy phase 6 realtime")
selected = iff(close > open, ema(high, 2), ema(low, 2))
previous = offset(close, 1)
up = max(change(close), 0)
down = -min(change(close), 0)
ratio = rsi(up, down)
plot(selected)
plot(previous)
plot(ratio)
"#,
    );
    let bars = [
        bar_ohlc(1.0, 10.0, 1.0, 2.0),
        bar_ohlc(2.0, 20.0, 2.0, 3.0),
        bar_ohlc(3.0, 15.0, 5.0, 2.0),
        bar_ohlc(4.0, 14.0, 6.0, 3.0),
    ];
    let historical = run_historical(&program, &bars).expect("phase 6 historical run");

    let mut incremental = RealtimeRuntime::new(&program);
    for bar in bars.iter().cloned() {
        incremental
            .update(BarUpdate::historical(bar))
            .expect("phase 6 incremental update");
    }
    assert_eq!(incremental.result(), historical);

    let mut realtime = RealtimeRuntime::new(&program);
    for bar in bars[..3].iter().cloned() {
        realtime
            .update(BarUpdate::historical(bar))
            .expect("phase 6 realtime history");
    }
    realtime
        .update(BarUpdate::forming(bar_ohlc(8.0, 30.0, 7.0, 9.0)))
        .expect("phase 6 forming update");
    realtime
        .update(BarUpdate::forming(bars[3]))
        .expect("phase 6 rolled-back forming update");
    let confirmed = realtime
        .update(BarUpdate::confirmed(bars[3]))
        .expect("phase 6 confirmed update");
    assert_eq!(confirmed, historical);
}
