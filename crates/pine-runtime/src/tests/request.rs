use std::sync::Arc;

use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

fn runtime_program() -> pine_ir::HirProgram {
    compile_program("indicator(\"request scaffold\")\nplot(close)\n")
}

fn compile_program(text: &str) -> pine_ir::HirProgram {
    let source = SourceFile::new("test.pine", text);
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

fn custom_environment() -> RequestEnvironment {
    let timeframe = RequestTimeframe::parse("D").expect("daily timeframe");
    let chart = ChartContext::new("NYSE:IBM", timeframe.clone());
    let key = RequestKey::new("NASDAQ:AAPL", timeframe);
    let provider = InMemoryRequestDataProvider::from_streams([(key, vec![timed_bar(0, 10.0)])])
        .expect("valid request bars");
    RequestEnvironment::new(chart, Arc::new(provider))
}

#[test]
fn request_timeframe_parses_supported_subset() {
    let default = RequestTimeframe::parse("").expect("default timeframe");
    assert_eq!(default.value(), "1");
    assert_eq!(default.seconds(), 60);

    let daily = RequestTimeframe::parse("D").expect("daily timeframe");
    assert_eq!(daily.value(), "D");
    assert_eq!(daily.seconds(), 86_400);

    let error = RequestTimeframe::parse("1H").expect_err("unsupported timeframe");
    assert_eq!(error.value(), "1H");
    assert_eq!(error.to_string(), "unsupported request timeframe `1H`");
}

#[test]
fn validates_duplicate_and_unsorted_requested_bars() {
    let duplicate = validate_requested_bars(&[timed_bar(1, 1.0), timed_bar(1, 2.0)])
        .expect_err("duplicate bar time should fail");
    assert_eq!(duplicate.to_string(), "duplicate requested bar time `1`");

    let unsorted = validate_requested_bars(&[timed_bar(2, 1.0), timed_bar(1, 2.0)])
        .expect_err("unsorted bar time should fail");
    assert_eq!(
        unsorted.to_string(),
        "requested bars are not sorted: `1` follows `2`"
    );
}

#[test]
fn default_request_environment_has_chart_metadata_and_no_data_provider() {
    let environment = RequestEnvironment::default();
    assert_eq!(environment.chart().symbol(), "NASDAQ:AAPL");
    assert_eq!(environment.chart().timeframe().value(), "1");

    let key = RequestKey::new("NASDAQ:AAPL", RequestTimeframe::default());
    let error = environment
        .provider()
        .bars(&key)
        .expect_err("default provider has no requested data");
    assert_eq!(
        error.to_string(),
        "missing request data for symbol `NASDAQ:AAPL` timeframe `1`"
    );
}

#[test]
fn in_memory_provider_validates_and_returns_requested_bars() {
    let key = RequestKey::new(
        "NYSE:IBM",
        RequestTimeframe::parse("D").expect("daily timeframe"),
    );
    let bars = vec![timed_bar(0, 10.0), timed_bar(86_400_000, 11.0)];
    let provider = InMemoryRequestDataProvider::from_streams([(key.clone(), bars.clone())])
        .expect("valid request bars");

    assert_eq!(
        provider.bars(&key).expect("requested bars"),
        bars.as_slice()
    );
}

#[test]
fn historical_runtime_accepts_custom_request_environment_without_changing_run_behavior() {
    let program = runtime_program();
    let runtime = HistoricalRuntime::with_request_environment(&program, custom_environment());

    assert_eq!(runtime.request_environment().chart().symbol(), "NYSE:IBM");
    assert_eq!(
        runtime.request_environment().chart().timeframe().value(),
        "D"
    );

    let result = runtime.run(&[timed_bar(0, 5.0)]).expect("runtime result");
    assert_eq!(result.plots[0].values, vec![PineValue::Float(5.0)]);
}

#[test]
fn realtime_runtime_carries_custom_request_environment_through_rollback() {
    let program = runtime_program();
    let mut runtime = RealtimeRuntime::with_request_environment(&program, custom_environment());

    assert_eq!(runtime.request_environment().chart().symbol(), "NYSE:IBM");

    runtime
        .update(BarUpdate::historical(timed_bar(0, 1.0)))
        .expect("historical update");
    runtime
        .update(BarUpdate::forming(timed_bar(60_000, 2.0)))
        .expect("forming update");
    assert_eq!(runtime.request_environment().chart().symbol(), "NYSE:IBM");

    runtime
        .update(BarUpdate::forming(timed_bar(60_000, 3.0)))
        .expect("second forming update");
    assert_eq!(
        runtime.request_environment().chart().timeframe().value(),
        "D"
    );
}

#[test]
fn request_security_same_context_returns_expression_series() {
    let program = compile_program(
        "indicator(\"request identity\")\nplot(request.security(syminfo.tickerid, timeframe.period, close + open))\n",
    );
    let result = run_historical(
        &program,
        &[bar_ohlc(1.0, 2.0, 0.5, 4.0), bar_ohlc(3.0, 5.0, 2.0, 8.0)],
    )
    .expect("same-context request.security should run");

    assert_values_close(&result.plots[0].values, &[5.0, 11.0]);
}

#[test]
fn request_security_same_context_supports_history_and_na_helpers() {
    let program = compile_program(
        "indicator(\"request history\")\nvalue = request.security(syminfo.tickerid, timeframe.period, na(close[1]) ? close : close[1])\nplot(value)\n",
    );
    let result = run_historical(&program, &[bar(10.0), bar(12.0), bar(15.0)])
        .expect("same-context request.security history expression should run");

    assert_values_close(&result.plots[0].values, &[10.0, 10.0, 12.0]);
}

#[test]
fn request_security_uses_runtime_chart_metadata_for_identity_check() {
    let program = compile_program(
        "indicator(\"request custom chart\")\nplot(request.security(syminfo.tickerid, timeframe.period, close))\n",
    );
    let runtime = HistoricalRuntime::with_request_environment(&program, custom_environment());
    let result = runtime
        .run(&[timed_bar(0, 9.0), timed_bar(86_400_000, 11.0)])
        .expect("custom chart metadata should satisfy same-context identity");

    assert_values_close(&result.plots[0].values, &[9.0, 11.0]);
}
