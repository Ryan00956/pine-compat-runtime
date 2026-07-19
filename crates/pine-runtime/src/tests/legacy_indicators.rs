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
