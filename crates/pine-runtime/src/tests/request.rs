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
fn request_security_isolates_provider_cum_state() {
    let program = compile_program(
        "indicator(\"request cum\")\nprovider = request.security(\"NYSE:IBM\", timeframe.period, ta.cum(close))\nchart = ta.cum(close)\nplot(provider)\nplot(chart)\n",
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
        .expect("provider ta.cum expression should run");

    assert_values_close(&result.plots[0].values, &[20.0, 42.0, 66.0]);
    assert_values_close(&result.plots[1].values, &[5.0, 12.0, 21.0]);
}

#[test]
fn request_security_evaluates_provider_round_to_mintick() {
    let program = compile_program(
        "indicator(\"request mintick\")\nrounded = request.security(\"NYSE:IBM\", timeframe.period, math.round_to_mintick(close + 0.006))\nplot(rounded)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 9.0),
        ])
        .expect("provider math.round_to_mintick expression should run");

    assert_values_close(&result.plots[0].values, &[20.01, 21.01, 22.01]);
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
fn request_security_isolates_provider_rsi_state_from_chart_state() {
    let program = compile_program(
        "indicator(\"request rsi\")\nprovider = request.security(\"NYSE:IBM\", timeframe.period, ta.rsi(close, 3))\nchart = ta.rsi(close, 3)\nplot(provider)\nplot(chart)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 24.0),
            timed_bar(180_000, 22.0),
            timed_bar(240_000, 26.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 9.0),
            timed_bar(180_000, 11.0),
            timed_bar(240_000, 13.0),
        ])
        .expect("provider ta.rsi expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[1..],
        &[100.0, 100.0, 66.66666666666666, 83.33333333333333],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..], &[100.0, 100.0, 100.0, 100.0]);
}

#[test]
fn request_security_isolates_provider_atr_state_from_chart_state() {
    let program = compile_program(
        "indicator(\"request atr\")\nprovider = request.security(\"NYSE:IBM\", timeframe.period, ta.atr(3))\nchart = ta.atr(3)\nplot(provider)\nplot(chart)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 9.0, 10.0, 8.0, 9.0, 100.0),
            timed_ohlcv(60_000, 11.0, 12.0, 11.0, 11.0, 100.0),
            timed_ohlcv(120_000, 7.0, 8.0, 6.0, 7.0, 100.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_ohlcv(0, 5.0, 5.5, 4.5, 5.0, 100.0),
            timed_ohlcv(60_000, 6.0, 6.5, 5.5, 6.0, 100.0),
            timed_ohlcv(120_000, 7.0, 7.5, 6.5, 7.0, 100.0),
        ])
        .expect("provider ta.atr expression should run");

    assert_values_close(
        &result.plots[0].values,
        &[2.0, 2.3333333333333335, 3.2222222222222223],
    );
    assert_values_close(
        &result.plots[1].values,
        &[1.0, 1.1666666666666667, 1.277777777777778],
    );
}

#[test]
fn request_security_evaluates_provider_true_range_in_requested_context() {
    let program = compile_program(
        "indicator(\"request tr\")\nprovider_tr = request.security(\"NYSE:IBM\", timeframe.period, ta.tr())\nprovider_strict = request.security(\"NYSE:IBM\", timeframe.period, ta.tr(false))\nchart_tr = ta.tr()\nchart_strict = ta.tr(false)\nplot(provider_tr)\nplot(provider_strict)\nplot(chart_tr)\nplot(chart_strict)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 9.0, 10.0, 8.0, 9.0, 100.0),
            timed_ohlcv(60_000, 11.0, 12.0, 11.0, 11.0, 100.0),
            timed_ohlcv(120_000, 7.0, 8.0, 6.0, 7.0, 100.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_ohlcv(0, 5.0, 5.5, 4.5, 5.0, 100.0),
            timed_ohlcv(60_000, 6.0, 6.5, 5.5, 6.0, 100.0),
            timed_ohlcv(120_000, 7.0, 7.5, 6.5, 7.0, 100.0),
        ])
        .expect("provider ta.tr expressions should run");

    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..], &[3.0, 5.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.5, 1.5]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_values_close(&result.plots[3].values[1..], &[1.5, 1.5]);
}

#[test]
fn request_security_isolates_provider_extrema_state_from_chart_state() {
    let program = compile_program(
        "indicator(\"request extrema\")\nprovider_hi = request.security(\"NYSE:IBM\", timeframe.period, ta.highest(3))\nprovider_lo = request.security(\"NYSE:IBM\", timeframe.period, ta.lowest(3))\nchart_hi = ta.highest(3)\nchart_lo = ta.lowest(3)\nplot(provider_hi)\nplot(provider_lo)\nplot(chart_hi)\nplot(chart_lo)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 1.0, 5.0, 1.0, 4.0, 100.0),
            timed_ohlcv(60_000, 1.0, 7.0, 2.0, 6.0, 100.0),
            timed_ohlcv(120_000, 1.0, 6.0, 3.0, 5.0, 100.0),
            timed_ohlcv(180_000, 1.0, 8.0, 0.0, 7.0, 100.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_ohlcv(0, 1.0, 3.0, 2.0, 2.0, 100.0),
            timed_ohlcv(60_000, 1.0, 4.0, 1.0, 3.0, 100.0),
            timed_ohlcv(120_000, 1.0, 5.0, 2.0, 4.0, 100.0),
            timed_ohlcv(180_000, 1.0, 6.0, 3.0, 5.0, 100.0),
        ])
        .expect("provider ta.highest/ta.lowest expressions should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[7.0, 8.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 0.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[5.0, 6.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..], &[1.0, 1.0]);
}

#[test]
fn request_security_evaluates_provider_momentum_in_requested_context() {
    let program = compile_program(
        "indicator(\"request momentum\")\nprovider_change = request.security(\"NYSE:IBM\", timeframe.period, ta.change(close))\nprovider_mom = request.security(\"NYSE:IBM\", timeframe.period, ta.mom(close, 2))\nprovider_roc = request.security(\"NYSE:IBM\", timeframe.period, ta.roc(close, 2))\nchart_change = ta.change(close)\nchart_mom = ta.mom(close, 2)\nchart_roc = ta.roc(close, 2)\nplot(provider_change)\nplot(provider_mom)\nplot(provider_roc)\nplot(chart_change)\nplot(chart_mom)\nplot(chart_roc)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 26.0),
            timed_bar(180_000, 25.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
        ])
        .expect("provider ta.change/ta.mom/ta.roc expressions should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 4.0, -1.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[6.0, 3.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[30.0, 13.636363636363635]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_values_close(&result.plots[3].values[1..], &[2.0, 4.0, 6.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..], &[6.0, 10.0]);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_values_close(&result.plots[5].values[2..], &[120.0, 142.85714285714286]);
}

#[test]
fn request_security_evaluates_provider_dispersion_in_requested_context() {
    let program = compile_program(
        "indicator(\"request dispersion\")\nprovider_range = request.security(\"NYSE:IBM\", timeframe.period, ta.range(close, 3))\nprovider_dev = request.security(\"NYSE:IBM\", timeframe.period, ta.dev(close, 3))\nchart_range = ta.range(close, 3)\nchart_dev = ta.dev(close, 3)\nplot(provider_range)\nplot(provider_dev)\nplot(chart_range)\nplot(chart_dev)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 21.0),
            timed_bar(180_000, 25.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 6.0),
            timed_bar(120_000, 8.0),
            timed_bar(180_000, 11.0),
        ])
        .expect("provider ta.range/ta.dev expressions should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 4.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[2..],
        &[0.6666666666666666, 1.5555555555555554],
    );
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[3.0, 5.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[3].values[2..],
        &[1.1111111111111112, 1.7777777777777777],
    );
}

#[test]
fn request_security_evaluates_provider_stdev_in_requested_context() {
    let program = compile_program(
        "indicator(\"request stdev\")\nprovider_biased = request.security(\"NYSE:IBM\", timeframe.period, ta.stdev(close, 3))\nprovider_sample = request.security(\"NYSE:IBM\", timeframe.period, ta.stdev(close, 3, false))\nchart_biased = ta.stdev(close, 3)\nchart_sample = ta.stdev(close, 3, false)\nplot(provider_biased)\nplot(provider_sample)\nplot(chart_biased)\nplot(chart_sample)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
        ])
        .expect("provider ta.stdev expressions should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.816496580927726, 1.247219128924647],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 1.5275252316519468]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[2].values[2..],
        &[2.494438257849294, 4.109609335312651],
    );
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[3].values[2..],
        &[3.0550504633038935, 5.033222956847166],
    );
}

#[test]
fn request_security_evaluates_provider_variance_in_requested_context() {
    let program = compile_program(
        "indicator(\"request variance\")\nprovider_biased = request.security(\"NYSE:IBM\", timeframe.period, ta.variance(close, 3))\nprovider_sample = request.security(\"NYSE:IBM\", timeframe.period, ta.variance(close, 3, false))\nchart_biased = ta.variance(close, 3)\nchart_sample = ta.variance(close, 3, false)\nplot(provider_biased)\nplot(provider_sample)\nplot(chart_biased)\nplot(chart_sample)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
        ])
        .expect("provider ta.variance expressions should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.6666666666666666, 1.5555555555555556],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 2.3333333333333335]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[2].values[2..],
        &[6.222222222222221, 16.88888888888889],
    );
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[3].values[2..],
        &[9.333333333333332, 25.333333333333336],
    );
}

#[test]
fn request_security_evaluates_provider_wma_in_requested_context() {
    let program = compile_program(
        "indicator(\"request wma\")\nprovider_wma = request.security(\"NYSE:IBM\", timeframe.period, ta.wma(close, 3))\nchart_wma = ta.wma(close, 3)\nplot(provider_wma)\nplot(chart_wma)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
        ])
        .expect("provider ta.wma expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[21.333333333333332, 22.833333333333332],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[2..],
        &[8.666666666666666, 13.333333333333334],
    );
}

#[test]
fn request_security_evaluates_provider_swma_in_requested_context() {
    let program = compile_program(
        "indicator(\"request swma\")\nprovider_swma = request.security(\"NYSE:IBM\", timeframe.period, ta.swma(close))\nchart_swma = ta.swma(close)\nplot(provider_swma)\nplot(chart_swma)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.swma expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[3..],
        &[21.666666666666668, 23.333333333333332],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[3..],
        &[9.666666666666666, 14.666666666666666],
    );
}

#[test]
fn request_security_evaluates_provider_hma_in_requested_context() {
    let program = compile_program(
        "indicator(\"request hma\")\nprovider_hma = request.security(\"NYSE:IBM\", timeframe.period, ta.hma(close, 4))\nchart_hma = ta.hma(close, 4)\nplot(provider_hma)\nplot(chart_hma)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
            timed_bar(300_000, 31.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
            timed_bar(300_000, 35.0),
        ])
        .expect("provider ta.hma expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
        assert_eq!(plot.values[2], PineValue::Na);
        assert_eq!(plot.values[3], PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[4..],
        &[26.422222222222224, 30.38888888888889],
    );
    assert_values_close(
        &result.plots[1].values[4..],
        &[23.777777777777775, 33.77777777777778],
    );
}

#[test]
fn request_security_evaluates_provider_alma_in_requested_context() {
    let program = compile_program(
        "indicator(\"request alma\")\nprovider_alma = request.security(\"NYSE:IBM\", timeframe.period, ta.alma(close, 4, 0.85, 6))\nchart_alma = ta.alma(close, 4, 0.85, 6)\nplot(provider_alma)\nplot(chart_alma)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.alma expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
        assert_eq!(plot.values[2], PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[3..],
        &[22.96743661472369, 25.429886558589775],
    );
    assert_values_close(
        &result.plots[1].values[3..],
        &[13.859773117179548, 20.783828483300205],
    );
}

#[test]
fn request_security_evaluates_provider_bbw_in_requested_context() {
    let program = compile_program(
        "indicator(\"request bbw\")\nprovider_bbw = request.security(\"NYSE:IBM\", timeframe.period, ta.bbw(close, 3, 2))\nchart_bbw = ta.bbw(close, 3, 2)\nplot(provider_bbw)\nplot(chart_bbw)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.bbw expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.15552315827194782, 0.22338253055366813, 0.3377761097517248],
    );
    assert_values_close(
        &result.plots[1].values[2..],
        &[1.3014460475735448, 1.4090089149643377, 1.2984641912517172],
    );
}

#[test]
fn request_security_evaluates_provider_correlation_in_requested_context() {
    let program = compile_program(
        "indicator(\"request correlation\")\nprovider_corr = request.security(\"NYSE:IBM\", timeframe.period, ta.correlation(close, high, 3))\nchart_corr = ta.correlation(close, high, 3)\nplot(provider_corr)\nplot(chart_corr)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 20.0, 25.0, 19.0, 20.0, 1.0),
            timed_ohlcv(60_000, 21.0, 23.0, 20.0, 21.0, 1.0),
            timed_ohlcv(120_000, 22.0, 26.0, 21.0, 22.0, 1.0),
            timed_ohlcv(180_000, 24.0, 28.0, 23.0, 24.0, 1.0),
            timed_ohlcv(240_000, 27.0, 31.0, 26.0, 27.0, 1.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_ohlcv(0, 5.0, 6.0, 4.0, 5.0, 1.0),
            timed_ohlcv(60_000, 7.0, 9.0, 6.0, 7.0, 1.0),
            timed_ohlcv(120_000, 11.0, 13.0, 10.0, 11.0, 1.0),
            timed_ohlcv(180_000, 17.0, 20.0, 16.0, 17.0, 1.0),
            timed_ohlcv(240_000, 25.0, 30.0, 24.0, 25.0, 1.0),
        ])
        .expect("provider ta.correlation expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.3273268353539886, 0.9538209664765321, 1.0],
    );
    assert_values_close(
        &result.plots[1].values[2..],
        &[0.9941916256019202, 0.9991507429465935, 0.9998148662392726],
    );
}

#[test]
fn request_security_evaluates_provider_covariance_in_requested_context() {
    let program = compile_program(
        "indicator(\"request covariance\")\nprovider_cov = request.security(\"NYSE:IBM\", timeframe.period, ta.covariance(close, high, 3))\nchart_cov = ta.covariance(close, high, 3)\nplot(provider_cov)\nplot(chart_cov)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_ohlcv(0, 20.0, 25.0, 19.0, 20.0, 1.0),
            timed_ohlcv(60_000, 21.0, 23.0, 20.0, 21.0, 1.0),
            timed_ohlcv(120_000, 22.0, 26.0, 21.0, 22.0, 1.0),
            timed_ohlcv(180_000, 24.0, 28.0, 23.0, 24.0, 1.0),
            timed_ohlcv(240_000, 27.0, 31.0, 26.0, 27.0, 1.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_ohlcv(0, 5.0, 6.0, 4.0, 5.0, 1.0),
            timed_ohlcv(60_000, 7.0, 9.0, 6.0, 7.0, 1.0),
            timed_ohlcv(120_000, 11.0, 13.0, 10.0, 11.0, 1.0),
            timed_ohlcv(180_000, 17.0, 20.0, 16.0, 17.0, 1.0),
            timed_ohlcv(240_000, 25.0, 30.0, 24.0, 25.0, 1.0),
        ])
        .expect("provider ta.covariance expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.3333333333333333, 2.4444444444444446, 4.222222222222222],
    );
    assert_values_close(
        &result.plots[1].values[2..],
        &[7.111111111111112, 18.666666666666664, 40.0],
    );
}

#[test]
fn request_security_evaluates_provider_median_in_requested_context() {
    let program = compile_program(
        "indicator(\"request median\")\nprovider_median = request.security(\"NYSE:IBM\", timeframe.period, ta.median(close, 3))\nchart_median = ta.median(close, 3)\nplot(provider_median)\nplot(chart_median)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.median expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[2..], &[21.0, 22.0, 24.0]);
    assert_values_close(&result.plots[1].values[2..], &[7.0, 11.0, 17.0]);
}

#[test]
fn request_security_evaluates_provider_mode_in_requested_context() {
    let program = compile_program(
        "indicator(\"request mode\")\nprovider_mode = request.security(\"NYSE:IBM\", timeframe.period, ta.mode(close, 3))\nchart_mode = ta.mode(close, 3)\nplot(provider_mode)\nplot(chart_mode)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.mode expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[2..], &[20.0, 21.0, 22.0]);
    assert_values_close(&result.plots[1].values[2..], &[5.0, 7.0, 11.0]);
}

#[test]
fn request_security_evaluates_provider_percentile_nearest_rank_in_requested_context() {
    let program = compile_program(
        "indicator(\"request percentile nearest rank\")\nprovider_percentile = request.security(\"NYSE:IBM\", timeframe.period, ta.percentile_nearest_rank(close, 3, 50))\nchart_percentile = ta.percentile_nearest_rank(close, 3, 50)\nplot(provider_percentile)\nplot(chart_percentile)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.percentile_nearest_rank expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[2..], &[21.0, 22.0, 24.0]);
    assert_values_close(&result.plots[1].values[2..], &[7.0, 11.0, 17.0]);
}

#[test]
fn request_security_evaluates_provider_percentile_linear_interpolation_in_requested_context() {
    let program = compile_program(
        "indicator(\"request percentile linear\")\nprovider_percentile = request.security(\"NYSE:IBM\", timeframe.period, ta.percentile_linear_interpolation(close, 3, 50))\nchart_percentile = ta.percentile_linear_interpolation(close, 3, 50)\nplot(provider_percentile)\nplot(chart_percentile)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.percentile_linear_interpolation expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[2..], &[21.0, 22.0, 24.0]);
    assert_values_close(&result.plots[1].values[2..], &[7.0, 11.0, 17.0]);
}

#[test]
fn request_security_evaluates_provider_percentrank_in_requested_context() {
    let program = compile_program(
        "indicator(\"request percentrank\")\nprovider_rank = request.security(\"NYSE:IBM\", timeframe.period, ta.percentrank(close, 3))\nchart_rank = ta.percentrank(close, 3)\nplot(provider_rank)\nplot(chart_rank)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.percentrank expression should run");

    for plot in &result.plots {
        assert_eq!(plot.values[0], PineValue::Na);
        assert_eq!(plot.values[1], PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[2..], &[100.0, 100.0, 100.0]);
    assert_values_close(&result.plots[1].values[2..], &[100.0, 100.0, 100.0]);
}

#[test]
fn request_security_evaluates_provider_linreg_in_requested_context() {
    let program = compile_program(
        "indicator(\"request linreg\")\nprovider_linreg = request.security(\"NYSE:IBM\", timeframe.period, ta.linreg(close, 3, 0))\nchart_linreg = ta.linreg(close, 3, 0)\nplot(provider_linreg)\nplot(chart_linreg)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 21.0),
            timed_bar(120_000, 22.0),
            timed_bar(180_000, 24.0),
            timed_bar(240_000, 27.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 7.0),
            timed_bar(120_000, 11.0),
            timed_bar(180_000, 17.0),
            timed_bar(240_000, 25.0),
        ])
        .expect("provider ta.linreg expression should run");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[22.0, 23.833333333333332, 26.833333333333332],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[2..],
        &[10.666666666666666, 16.666666666666668, 24.666666666666668],
    );
}

#[test]
fn request_security_evaluates_provider_trend_flags_in_requested_context() {
    let program = compile_program(
        "indicator(\"request trend flags\")\nprovider_rising = request.security(\"NYSE:IBM\", timeframe.period, ta.rising(close, 2) ? 1 : 0)\nprovider_falling = request.security(\"NYSE:IBM\", timeframe.period, ta.falling(close, 2) ? 1 : 0)\nchart_rising = ta.rising(close, 2) ? 1 : 0\nchart_falling = ta.falling(close, 2) ? 1 : 0\nplot(provider_rising)\nplot(provider_falling)\nplot(chart_rising)\nplot(chart_falling)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 20.0),
            timed_bar(60_000, 22.0),
            timed_bar(120_000, 21.0),
            timed_bar(180_000, 19.0),
            timed_bar(240_000, 18.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 6.0),
            timed_bar(120_000, 8.0),
            timed_bar(180_000, 7.0),
            timed_bar(240_000, 10.0),
        ])
        .expect("provider ta.rising/ta.falling expressions should run");

    assert_values_close(&result.plots[0].values, &[0.0, 0.0, 0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 0.0, 0.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 0.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[0.0; 5]);
}

#[test]
fn request_security_evaluates_provider_cross_flags_in_requested_context() {
    let program = compile_program(
        "indicator(\"request cross\")\nprovider_cross = request.security(\"NYSE:IBM\", timeframe.period, ta.cross(close, 2.0) ? 1 : 0)\nprovider_over = request.security(\"NYSE:IBM\", timeframe.period, ta.crossover(close, 2.0) ? 1 : 0)\nprovider_under = request.security(\"NYSE:IBM\", timeframe.period, ta.crossunder(close, 2.0) ? 1 : 0)\nchart_cross = ta.cross(close, 3.0) ? 1 : 0\nchart_over = ta.crossover(close, 3.0) ? 1 : 0\nchart_under = ta.crossunder(close, 3.0) ? 1 : 0\nplot(provider_cross)\nplot(provider_over)\nplot(provider_under)\nplot(chart_cross)\nplot(chart_over)\nplot(chart_under)\n",
    );
    let environment = external_symbol_environment(
        "NYSE:IBM",
        vec![
            timed_bar(0, 1.0),
            timed_bar(60_000, 3.0),
            timed_bar(120_000, 1.0),
            timed_bar(180_000, 2.0),
            timed_bar(240_000, 4.0),
        ],
    );
    let result = HistoricalRuntime::with_request_environment(&program, environment)
        .run(&[
            timed_bar(0, 5.0),
            timed_bar(60_000, 1.0),
            timed_bar(120_000, 5.0),
            timed_bar(180_000, 1.0),
            timed_bar(240_000, 5.0),
        ])
        .expect("provider cross expressions should run");

    assert_values_close(&result.plots[0].values, &[0.0, 1.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 1.0, 0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 0.0, 1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[3].values, &[0.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[0.0, 1.0, 0.0, 1.0, 0.0]);
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
