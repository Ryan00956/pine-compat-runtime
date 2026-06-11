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

fn timed_ohlcv(time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Bar {
    Bar {
        time,
        open,
        high,
        low,
        close,
        volume,
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

fn external_symbol_environment(symbol: &str, bars: Vec<Bar>) -> RequestEnvironment {
    let key = RequestKey::new(symbol, RequestTimeframe::default());
    let provider =
        InMemoryRequestDataProvider::from_streams([(key, bars)]).expect("valid request bars");
    RequestEnvironment::new(ChartContext::default(), Arc::new(provider))
}

fn external_symbol_environment_with_timeframe(
    symbol: &str,
    timeframe: &str,
    bars: Vec<Bar>,
) -> RequestEnvironment {
    let key = RequestKey::new(
        symbol,
        RequestTimeframe::parse(timeframe).expect("request timeframe"),
    );
    let provider =
        InMemoryRequestDataProvider::from_streams([(key, bars)]).expect("valid request bars");
    RequestEnvironment::new(ChartContext::default(), Arc::new(provider))
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
fn in_memory_provider_rejects_duplicate_request_keys() {
    let key = RequestKey::new(
        "NYSE:IBM",
        RequestTimeframe::parse("D").expect("daily timeframe"),
    );
    let error = InMemoryRequestDataProvider::from_streams([
        (key.clone(), vec![timed_bar(0, 10.0)]),
        (key, vec![timed_bar(86_400_000, 11.0)]),
    ])
    .expect_err("duplicate request key should fail");

    assert_eq!(
        error.to_string(),
        "duplicate request data for symbol `NYSE:IBM` timeframe `D`"
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

#[test]
fn request_security_reads_same_timeframe_external_symbol_from_provider() {
    let program = compile_program(
        "indicator(\"request external\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, close))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![timed_bar(0, 20.0), timed_bar(60_000, 21.0)],
    );
    let runtime = HistoricalRuntime::with_request_environment(&program, environment);
    let result = runtime
        .run(&[timed_bar(0, 5.0), timed_bar(60_000, 6.0)])
        .expect("external request data should run");

    assert_values_close(&result.plots[0].values, &[20.0, 21.0]);
}

#[test]
fn request_security_evaluates_provider_arithmetic_in_requested_context() {
    let program = compile_program(
        "indicator(\"request arithmetic\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, open + close))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 100.0, 101.0, 99.0, 20.0, 1.0),
            timed_ohlcv(60_000, 110.0, 111.0, 109.0, 21.0, 1.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[timed_bar(0, 5.0), timed_bar(60_000, 6.0)])
        .expect("provider arithmetic should run");

    assert_values_close(&result.plots[0].values, &[120.0, 131.0]);
}

#[test]
fn request_security_evaluates_provider_math_extremes_in_requested_context() {
    let program = compile_program(
        "indicator(\"request math extremes\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, math.max(close, open) - math.min(close, open)))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 100.0, 101.0, 99.0, 20.0, 1.0),
            timed_ohlcv(60_000, 21.0, 111.0, 19.0, 110.0, 1.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[timed_bar(0, 5.0), timed_bar(60_000, 6.0)])
        .expect("provider math extremes should run");

    assert_values_close(&result.plots[0].values, &[80.0, 89.0]);
}

#[test]
fn request_security_evaluates_provider_stateless_math_calls() {
    let program = compile_program(
        "indicator(\"request stateless math\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, math.abs(open - close) + math.floor(math.avg(open, close)) + math.pow(2, 2) + math.hypot(3, 4)))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 100.0, 101.0, 99.0, 20.0, 1.0),
            timed_ohlcv(60_000, 21.0, 111.0, 19.0, 110.0, 1.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[timed_bar(0, 5.0), timed_bar(60_000, 6.0)])
        .expect("provider stateless math should run");

    assert_values_close(&result.plots[0].values, &[149.0, 163.0]);
}

#[test]
fn request_security_isolates_provider_math_sum_state() {
    let program = compile_program(
        "indicator(\"request math sum\")\nprovider = request.security(\"NYSE:IBM\", timeframe.period, math.sum(close, 2))\nchart = math.sum(close, 2)\nplot(provider)\nplot(chart)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 9.0),
        ])
        .expect("provider math.sum expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[42.0, 46.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..], &[12.0, 16.0]);
}

#[test]
fn request_security_evaluates_provider_history_references() {
    let program = compile_program(
        "indicator(\"request history\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, close[1]))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 6.0),
            timed_bar(120_000, 7.0),
        ])
        .expect("provider history should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[20.0, 22.0]);
}

#[test]
fn request_security_isolates_provider_ta_state_from_chart_state() {
    let program = compile_program(
        "indicator(\"request ta\")\nprovider = request.security(\"NYSE:IBM\", timeframe.period, ta.sma(close, 2))\nchart = ta.sma(close, 2)\nplot(provider)\nplot(chart)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 9.0),
        ])
        .expect("provider ta expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[21.0, 23.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..], &[6.0, 8.0]);
}

#[test]
fn request_security_evaluates_provider_ema_in_requested_context() {
    let program = compile_program(
        "indicator(\"request ema\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, ta.ema(close, 2)))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 23.0),
            timed_bar(120_000, 26.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 6.0),
            timed_bar(120_000, 7.0),
        ])
        .expect("provider ema expression should run");

    assert_values_close(
        &result.plots[0].values,
        &[20.0, 22.0, 24.666_666_666_666_668],
    );
}

#[test]
fn request_security_aligns_higher_timeframe_without_future_values() {
    let program = compile_program(
        "indicator(\"request htf\")\nplot(request.security(\"NYSE:IBM\", \"5\", close))\n",
    );
    let environment = external_symbol_environment_with_timeframe(
        "NYSE:IBM",
        "5",
        vec![timed_bar(0, 100.0), timed_bar(300_000, 200.0)],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 1.0),
            timed_bar(60_000, 2.0),
            timed_bar(240_000, 3.0),
            timed_bar(300_000, 4.0),
            timed_bar(540_000, 5.0),
        ])
        .expect("higher timeframe request should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[100.0, 100.0, 200.0]);
}

#[test]
fn request_security_higher_timeframe_gap_fills_last_confirmed_value() {
    let program = compile_program(
        "indicator(\"request htf gap\")\nplot(request.security(\"NYSE:IBM\", \"5\", close))\n",
    );
    let environment = external_symbol_environment_with_timeframe(
        "NYSE:IBM",
        "5",
        vec![timed_bar(0, 100.0), timed_bar(900_000, 300.0)],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(240_000, 1.0),
            timed_bar(600_000, 2.0),
            timed_bar(1_140_000, 3.0),
        ])
        .expect("higher timeframe gap fill should run");

    assert_values_close(&result.plots[0].values, &[100.0, 100.0, 300.0]);
}

#[test]
fn request_security_higher_timeframe_supports_chart_symbol_provider_data() {
    let program = compile_program(
        "indicator(\"request chart htf\")\nplot(request.security(syminfo.tickerid, \"5\", close + open))\n",
    );
    let key = RequestKey::new(
        "NASDAQ:AAPL",
        RequestTimeframe::parse("5").expect("five minute timeframe"),
    );
    let provider = InMemoryRequestDataProvider::from_streams([(
        key,
        vec![timed_ohlcv(0, 10.0, 11.0, 9.0, 100.0, 1.0)],
    )])
    .expect("valid request bars");
    let environment = RequestEnvironment::new(ChartContext::default(), Arc::new(provider));
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[timed_bar(240_000, 1.0)])
        .expect("chart-symbol higher timeframe provider request should run");

    assert_values_close(&result.plots[0].values, &[110.0]);
}

#[test]
fn request_security_rejects_lower_timeframe_provider_requests() {
    let program = compile_program(
        "indicator(\"request ltf\")\nplot(request.security(\"NYSE:IBM\", \"30S\", close))\n",
    );
    let runtime =
        HistoricalRuntime::with_request_environment(&program, RequestEnvironment::default());
    let error = runtime
        .run(&[timed_bar(0, 1.0)])
        .expect_err("lower timeframe should fail");

    assert_eq!(
        error.message,
        "request.security lower timeframe requests are not supported for symbol `NYSE:IBM` timeframe `30S` on chart timeframe `1`"
    );
}

#[test]
fn realtime_request_security_higher_timeframe_uses_confirmed_requested_bars() {
    let program = compile_program(
        "indicator(\"request realtime htf\")\nplot(request.security(\"NYSE:IBM\", \"5\", close))\n",
    );
    let mut runtime = RealtimeRuntime::with_request_environment(
        &program,
        external_symbol_environment_with_timeframe(
            "NYSE:IBM",
            "5",
            vec![timed_bar(0, 100.0), timed_bar(300_000, 200.0)],
        ),
    );

    runtime
        .update(BarUpdate::historical(timed_bar(0, 1.0)))
        .expect("historical update");
    let first_forming = runtime
        .update(BarUpdate::forming(timed_bar(240_000, 2.0)))
        .expect("forming update");
    let second_forming = runtime
        .update(BarUpdate::forming(timed_bar(240_000, 3.0)))
        .expect("second forming update");

    assert_eq!(first_forming.plots[0].values[0], PineValue::Na);
    assert_values_close(&first_forming.plots[0].values[1..], &[100.0]);
    assert_eq!(second_forming.plots[0].values[0], PineValue::Na);
    assert_values_close(&second_forming.plots[0].values[1..], &[100.0]);
}

#[test]
fn request_security_caches_requested_context_values_by_callsite() {
    let program = compile_program(
        "indicator(\"request cache\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, ta.sma(close, 2)))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 24.0),
        ],
    );
    let mut runtime = HistoricalRuntime::with_request_environment(&program, environment);

    runtime
        .append_bar(timed_bar(0, 5.0))
        .expect("first bar should run");
    assert_eq!(runtime.request_cache.len(), 1);
    runtime
        .append_bar(timed_bar(60_000, 7.0))
        .expect("second bar should run");
    assert_eq!(runtime.request_cache.len(), 1);
}

#[test]
fn request_security_cache_isolates_same_context_different_callsite_expressions() {
    let program = compile_program(
        "indicator(\"request cache isolation\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, open))\nplot(request.security(\"NYSE:IBM\", timeframe.period, close))\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 10.0, 25.0, 5.0, 20.0, 1.0),
            timed_ohlcv(60_000, 11.0, 26.0, 6.0, 21.0, 1.0),
        ],
    );
    let mut runtime = HistoricalRuntime::with_request_environment(&program, environment);

    runtime
        .append_bar(timed_bar(0, 5.0))
        .expect("first bar should run");
    runtime
        .append_bar(timed_bar(60_000, 7.0))
        .expect("second bar should run");
    let result = runtime.result();

    assert_eq!(runtime.request_cache.len(), 2);
    assert_values_close(&result.plots[0].values, &[10.0, 11.0]);
    assert_values_close(&result.plots[1].values, &[20.0, 21.0]);
}

#[test]
fn request_security_reports_missing_external_dataset() {
    let program = compile_program(
        "indicator(\"request missing\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, close))\n",
    );
    let runtime =
        HistoricalRuntime::with_request_environment(&program, RequestEnvironment::default());
    let error = runtime
        .run(&[timed_bar(0, 5.0)])
        .expect_err("missing provider data should fail");

    assert_eq!(
        error.message,
        "missing request data for symbol `NYSE:IBM` timeframe `1`"
    );
}

#[test]
fn realtime_request_security_reuses_immutable_provider_data_during_rollback() {
    let program = compile_program(
        "indicator(\"request realtime\")\nplot(request.security(\"NYSE:IBM\", timeframe.period, close))\n",
    );
    let mut runtime = RealtimeRuntime::with_request_environment(
        &program,
        external_symbol_environment(
            "NYSE:IBM",
            vec![timed_bar(0, 20.0), timed_bar(60_000, 21.0)],
        ),
    );

    runtime
        .update(BarUpdate::historical(timed_bar(0, 5.0)))
        .expect("historical update");
    let first_forming = runtime
        .update(BarUpdate::forming(timed_bar(60_000, 6.0)))
        .expect("forming update");
    let second_forming = runtime
        .update(BarUpdate::forming(timed_bar(60_000, 7.0)))
        .expect("second forming update");

    assert_values_close(&first_forming.plots[0].values, &[20.0, 21.0]);
    assert_values_close(&second_forming.plots[0].values, &[20.0, 21.0]);
}
