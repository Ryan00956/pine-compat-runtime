use std::{fs, sync::Arc};

use pine_runtime::{
    ChartContext, InMemoryRequestDataProvider, RequestEnvironment, RequestKey, RequestTimeframe,
    RunningAlertConfig, RuntimeResult, public_runtime_profiled_result_json,
    public_runtime_result_json, run_historical_profiled_with_request_environment,
    run_historical_with_request_environment,
};
use pine_sema::analyze_input;

use crate::bars_csv::parse_bars_csv;
use crate::library_sources::{
    LibrarySourceSpec, analysis_input_from_paths, parse_library_source_spec,
};
use crate::usage;

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let options = parse_options(&args)?;
    run_with_options(&options)
}

#[derive(Debug)]
struct RunOptions {
    path: String,
    bars_path: String,
    profile: bool,
    request_bars: Vec<RequestBarsSpec>,
    library_sources: Vec<LibrarySourceSpec>,
    strategy_alert_template: Option<StrategyAlertTemplateOptions>,
    strategy_running_alert: Option<StrategyRunningAlertOptions>,
}

#[derive(Debug)]
struct StrategyAlertTemplateOptions {
    template: String,
    index: usize,
}

#[derive(Debug)]
struct StrategyRunningAlertOptions {
    config: RunningAlertConfig,
    index: usize,
}

#[derive(Debug)]
struct RequestBarsSpec {
    key: RequestKey,
    path: String,
}

fn run_with_options(options: &RunOptions) -> Result<(), String> {
    println!("{}", run_output_with_options(options)?);
    Ok(())
}

fn run_output_with_options(options: &RunOptions) -> Result<String, String> {
    if options.profile
        && (options.strategy_alert_template.is_some() || options.strategy_running_alert.is_some())
    {
        return Err("strategy alert rendering cannot be combined with --profile".to_owned());
    }
    if let Some(template) = &options.strategy_alert_template {
        let result = run_result_with_options(options)?;
        return render_strategy_alert_template(&result, template);
    }
    if let Some(running_alert) = &options.strategy_running_alert {
        let result = run_result_with_options(options)?;
        return render_strategy_running_alert(&result, running_alert);
    }
    run_json_with_options(options)
}

fn run_json_with_options(options: &RunOptions) -> Result<String, String> {
    if options.profile {
        return run_profiled_json_with_options(options);
    }
    let result = run_result_with_options(options)?;
    Ok(public_runtime_result_json(&result))
}

fn run_profiled_json_with_options(options: &RunOptions) -> Result<String, String> {
    let input = analysis_input_from_paths(&options.path, &options.library_sources)?;
    let source = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        for diagnostic in analysis.diagnostics {
            let line_col = source.line_col(diagnostic.span.start);
            eprintln!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            );
        }
        return Err("analysis failed".to_owned());
    }
    let Some(hir) = analysis.hir else {
        return Err("analysis did not produce executable HIR".to_owned());
    };

    let bars_text = fs::read_to_string(&options.bars_path)
        .map_err(|err| format!("failed to read {}: {err}", options.bars_path))?;
    let bars = parse_bars_csv(&bars_text)?;
    let request_environment = request_environment_from_specs(&options.request_bars)?;
    let result = run_historical_profiled_with_request_environment(&hir, &bars, request_environment)
        .map_err(|err| format!("runtime failed: {}", err.message))?;
    Ok(public_runtime_profiled_result_json(
        &result.result,
        &result.profile,
    ))
}

fn run_result_with_options(options: &RunOptions) -> Result<RuntimeResult, String> {
    let input = analysis_input_from_paths(&options.path, &options.library_sources)?;
    let source = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        for diagnostic in analysis.diagnostics {
            let line_col = source.line_col(diagnostic.span.start);
            eprintln!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            );
        }
        return Err("analysis failed".to_owned());
    }
    let Some(hir) = analysis.hir else {
        return Err("analysis did not produce executable HIR".to_owned());
    };

    let bars_text = fs::read_to_string(&options.bars_path)
        .map_err(|err| format!("failed to read {}: {err}", options.bars_path))?;
    let bars = parse_bars_csv(&bars_text)?;
    let request_environment = request_environment_from_specs(&options.request_bars)?;
    run_historical_with_request_environment(&hir, &bars, request_environment)
        .map_err(|err| format!("runtime failed: {}", err.message))
}

fn render_strategy_alert_template(
    result: &RuntimeResult,
    template: &StrategyAlertTemplateOptions,
) -> Result<String, String> {
    let strategy = result
        .strategy
        .as_ref()
        .ok_or_else(|| "runtime result does not contain strategy alerts".to_owned())?;
    let alert = strategy.alerts.get(template.index).ok_or_else(|| {
        format!(
            "strategy alert index {} is out of range for {} alert(s)",
            template.index,
            strategy.alerts.len()
        )
    })?;
    pine_runtime::render_strategy_order_fill_alert_template(&template.template, alert)
        .map_err(|err| err.to_string())
}

fn render_strategy_running_alert(
    result: &RuntimeResult,
    running_alert: &StrategyRunningAlertOptions,
) -> Result<String, String> {
    let strategy = result
        .strategy
        .as_ref()
        .ok_or_else(|| "runtime result does not contain strategy alerts".to_owned())?;
    let alert = strategy.alerts.get(running_alert.index).ok_or_else(|| {
        format!(
            "strategy alert index {} is out of range for {} alert(s)",
            running_alert.index,
            strategy.alerts.len()
        )
    })?;
    pine_runtime::render_strategy_order_fill_running_alert(&running_alert.config, alert)
        .map_err(|err| err.to_string())
}

fn parse_options(args: &[String]) -> Result<RunOptions, String> {
    let Some(path) = args.first() else {
        return Err(usage());
    };
    let mut options = RunOptions {
        path: path.clone(),
        bars_path: String::new(),
        profile: false,
        request_bars: Vec::new(),
        library_sources: Vec::new(),
        strategy_alert_template: None,
        strategy_running_alert: None,
    };
    let mut strategy_alert_template = None;
    let mut strategy_running_alert_template = None;
    let mut running_alert_script_snapshot_id = None;
    let mut running_alert_symbol = None;
    let mut running_alert_timeframe = None;
    let mut strategy_alert_index = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options.bars_path = value.clone();
            }
            "--profile" => {
                options.profile = true;
            }
            "--render-strategy-order-alert-template" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_alert_template = Some(value.clone());
            }
            "--render-strategy-running-alert" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_running_alert_template = Some(value.clone());
            }
            "--running-alert-script-snapshot-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_script_snapshot_id = Some(value.clone());
            }
            "--running-alert-symbol" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_symbol = Some(value.clone());
            }
            "--running-alert-timeframe" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_timeframe = Some(value.clone());
            }
            "--strategy-alert-index" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_alert_index = Some(parse_strategy_alert_index(value)?);
            }
            "--request-bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options.request_bars.push(parse_request_bars_spec(value)?);
            }
            "--library-source" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options
                    .library_sources
                    .push(parse_library_source_spec(value)?);
            }
            _ => return Err(usage()),
        }
        index += 1;
    }
    if options.bars_path.is_empty() {
        return Err(usage());
    }
    if strategy_alert_template.is_some() && strategy_running_alert_template.is_some() {
        return Err(usage());
    }
    options.strategy_alert_template = match (strategy_alert_template, &strategy_alert_index) {
        (None, _) => None,
        (Some(template), Some(index)) => Some(StrategyAlertTemplateOptions {
            template,
            index: *index,
        }),
        (Some(_), None) => return Err(usage()),
    };
    options.strategy_running_alert = if options.strategy_alert_template.is_some() {
        if strategy_running_alert_template.is_some()
            || running_alert_script_snapshot_id.is_some()
            || running_alert_symbol.is_some()
            || running_alert_timeframe.is_some()
        {
            return Err(usage());
        }
        None
    } else {
        match (
            strategy_running_alert_template,
            running_alert_script_snapshot_id,
            running_alert_symbol,
            running_alert_timeframe,
            strategy_alert_index,
        ) {
            (None, None, None, None, None) => None,
            (
                Some(template),
                Some(script_snapshot_id),
                Some(symbol),
                Some(timeframe),
                Some(index),
            ) => Some(StrategyRunningAlertOptions {
                config: RunningAlertConfig::new_strategy_order_fills(
                    script_snapshot_id,
                    symbol,
                    timeframe,
                    template,
                ),
                index,
            }),
            _ => return Err(usage()),
        }
    };
    Ok(options)
}

fn parse_strategy_alert_index(value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| "strategy alert index must be a non-negative integer".to_owned())
}

fn parse_request_bars_spec(spec: &str) -> Result<RequestBarsSpec, String> {
    let Some((key, path)) = spec.split_once('=') else {
        return Err("request bars must use SYMBOL:TIMEFRAME=path.csv".to_owned());
    };
    if path.trim().is_empty() {
        return Err("request bars path must not be empty".to_owned());
    }
    let Some((symbol, timeframe)) = key.rsplit_once(':') else {
        return Err("request bars key must use SYMBOL:TIMEFRAME".to_owned());
    };
    if symbol.trim().is_empty() {
        return Err("request bars symbol must not be empty".to_owned());
    }
    let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| err.to_string())?;
    Ok(RequestBarsSpec {
        key: RequestKey::new(symbol.trim(), timeframe),
        path: path.to_owned(),
    })
}

fn request_environment_from_specs(specs: &[RequestBarsSpec]) -> Result<RequestEnvironment, String> {
    if specs.is_empty() {
        return Ok(RequestEnvironment::default());
    }

    let mut streams = Vec::with_capacity(specs.len());
    for spec in specs {
        let text = fs::read_to_string(&spec.path)
            .map_err(|err| format!("failed to read {}: {err}", spec.path))?;
        streams.push((spec.key.clone(), parse_bars_csv(&text)?));
    }
    let provider =
        InMemoryRequestDataProvider::from_streams(streams).map_err(|err| err.to_string())?;
    Ok(RequestEnvironment::new(
        ChartContext::default(),
        Arc::new(provider),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn workspace_path(path: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
            .display()
            .to_string()
    }

    #[test]
    fn parses_request_bars_spec_with_exchange_prefixed_symbol() {
        let spec = parse_request_bars_spec("NASDAQ:AAPL:1=request.csv")
            .expect("request bars spec should parse");

        assert_eq!(spec.key.symbol(), "NASDAQ:AAPL");
        assert_eq!(spec.key.timeframe().value(), "1");
        assert_eq!(spec.path, "request.csv");
    }

    #[test]
    fn parses_run_options_with_library_source() {
        let options = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--library-source".to_owned(),
            "user/lib/1=lib.pine".to_owned(),
        ])
        .expect("run options");

        assert_eq!(options.path, "script.pine");
        assert_eq!(options.bars_path, "bars.csv");
        assert_eq!(options.library_sources.len(), 1);
        assert_eq!(options.library_sources[0].key, "user/lib/1");
        assert_eq!(options.library_sources[0].path, "lib.pine");
        assert!(options.strategy_alert_template.is_none());
        assert!(options.strategy_running_alert.is_none());
    }

    #[test]
    fn parses_run_options_with_strategy_alert_template() {
        let options = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--render-strategy-order-alert-template".to_owned(),
            "Order: {{strategy.order.alert_message}}".to_owned(),
            "--strategy-alert-index".to_owned(),
            "1".to_owned(),
        ])
        .expect("run options");
        let template = options
            .strategy_alert_template
            .as_ref()
            .expect("strategy alert template");

        assert_eq!(template.template, "Order: {{strategy.order.alert_message}}");
        assert_eq!(template.index, 1);
        assert!(options.strategy_running_alert.is_none());
    }

    #[test]
    fn parses_run_options_with_strategy_running_alert() {
        let options = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--render-strategy-running-alert".to_owned(),
            "Running: {{strategy.order.alert_message}}".to_owned(),
            "--strategy-alert-index".to_owned(),
            "1".to_owned(),
            "--running-alert-script-snapshot-id".to_owned(),
            "snapshot-1".to_owned(),
            "--running-alert-symbol".to_owned(),
            "NASDAQ:AAPL".to_owned(),
            "--running-alert-timeframe".to_owned(),
            "60".to_owned(),
        ])
        .expect("run options");
        let running_alert = options
            .strategy_running_alert
            .as_ref()
            .expect("strategy running alert");

        assert!(options.strategy_alert_template.is_none());
        assert_eq!(running_alert.index, 1);
        assert_eq!(running_alert.config.script_snapshot_id, "snapshot-1");
        assert_eq!(running_alert.config.symbol, "NASDAQ:AAPL");
        assert_eq!(running_alert.config.timeframe, "60");
        assert_eq!(
            running_alert.config.message_template,
            "Running: {{strategy.order.alert_message}}"
        );
    }

    #[test]
    fn rejects_partial_strategy_alert_template_options() {
        let error = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--render-strategy-order-alert-template".to_owned(),
            "{{strategy.order.alert_message}}".to_owned(),
        ])
        .expect_err("strategy alert index is required");

        assert!(error.contains("usage: pine-compat"));
    }

    #[test]
    fn rejects_partial_strategy_running_alert_options() {
        let error = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--render-strategy-running-alert".to_owned(),
            "{{strategy.order.alert_message}}".to_owned(),
            "--strategy-alert-index".to_owned(),
            "0".to_owned(),
            "--running-alert-script-snapshot-id".to_owned(),
            "snapshot-1".to_owned(),
        ])
        .expect_err("running alert symbol and timeframe are required");

        assert!(error.contains("usage: pine-compat"));
    }

    #[test]
    fn rejects_mixed_strategy_alert_rendering_options() {
        let error = parse_options(&[
            "script.pine".to_owned(),
            "--bars".to_owned(),
            "bars.csv".to_owned(),
            "--render-strategy-order-alert-template".to_owned(),
            "{{strategy.order.alert_message}}".to_owned(),
            "--render-strategy-running-alert".to_owned(),
            "{{strategy.order.alert_message}}".to_owned(),
            "--strategy-alert-index".to_owned(),
            "0".to_owned(),
            "--running-alert-script-snapshot-id".to_owned(),
            "snapshot-1".to_owned(),
            "--running-alert-symbol".to_owned(),
            "NASDAQ:AAPL".to_owned(),
            "--running-alert-timeframe".to_owned(),
            "60".to_owned(),
        ])
        .expect_err("only one alert rendering mode is allowed");

        assert!(error.contains("usage: pine-compat"));
    }

    #[test]
    fn builds_request_environment_from_csv_specs() {
        let path = std::env::temp_dir().join(format!(
            "pine-request-bars-{}-{}.csv",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(&path, "time,open,high,low,close,volume\n0,10,11,9,12,100\n")
            .expect("write request bars");
        let spec = parse_request_bars_spec(&format!("NYSE:IBM:1={}", path.display()))
            .expect("request bars spec");

        let environment =
            request_environment_from_specs(&[spec]).expect("request environment from CSV");
        let bars = environment
            .provider()
            .bars(&RequestKey::new("NYSE:IBM", RequestTimeframe::default()))
            .expect("request bars");

        assert_eq!(bars[0].close, 12.0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn duplicate_request_bars_keys_fail() {
        let path = std::env::temp_dir().join(format!(
            "pine-request-bars-duplicate-{}.csv",
            std::process::id()
        ));
        fs::write(&path, "time,open,high,low,close,volume\n0,1,1,1,1,1\n")
            .expect("write request bars");
        let first = parse_request_bars_spec(&format!("NYSE:IBM:1={}", path.display()))
            .expect("first request bars spec");
        let second = parse_request_bars_spec(&format!("NYSE:IBM:1={}", path.display()))
            .expect("second request bars spec");

        let error = match request_environment_from_specs(&[first, second]) {
            Ok(_) => panic!("duplicate request bars should fail"),
            Err(error) => error,
        };

        assert!(error.contains("duplicate request data for symbol `NYSE:IBM` timeframe `1`"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runs_request_bars_integration_fixture() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/request/request_security_host.pine"),
            bars_path: workspace_path("tests/fixtures/request/chart_1m.csv"),
            profile: false,
            library_sources: Vec::new(),
            request_bars: vec![
                parse_request_bars_spec(&format!(
                    "NYSE:IBM:1={}",
                    workspace_path("tests/fixtures/request/ibm_1m.csv")
                ))
                .expect("same timeframe request bars"),
                parse_request_bars_spec(&format!(
                    "NYSE:IBM:5={}",
                    workspace_path("tests/fixtures/request/ibm_5m.csv")
                ))
                .expect("higher timeframe request bars"),
            ],
            strategy_alert_template: None,
            strategy_running_alert: None,
        };

        let output = run_json_with_options(&options).expect("request integration fixture");

        assert!(output.contains("\"values\":[30,32,34,36,38]"));
        assert!(output.contains("\"values\":[null,null,100,100,200]"));
        assert!(output.contains("\"values\":[10,10,10,10,10]"));
        assert!(output.contains("\"values\":[34,35,36,37,38]"));
        assert!(output.contains("\"values\":[null,41,43,45,47]"));
        assert!(output.contains("\"values\":[20.01,21.01,22.01,23.01,24.01]"));
        assert!(output.contains("\"values\":[null,100,100,100,100]"));
        assert!(output.contains("\"values\":[2,10,10,10,10]"));
        assert!(output.contains("\"values\":[null,10,10,10,10]"));
        assert!(output.contains(
            "\"values\":[2,4.666666666666667,6.4444444444444455,7.629629629629631,8.419753086419753]"
        ));
        assert!(output.contains("\"values\":[null,null,13,14,15]"));
        assert!(output.contains("\"values\":[null,null,9,10,11]"));
        assert!(output.contains("\"values\":[null,1,1,1,1]"));
        assert!(output.contains("\"values\":[null,null,2,2,2]"));
        assert!(output.contains("\"values\":[null,null,10,9.523809523809524,9.090909090909092]"));
        assert!(output.contains("\"values\":[null,null,2,2,2]"));
        assert!(output.contains(
            "\"values\":[null,null,0.6666666666666666,0.6666666666666666,0.6666666666666666]"
        ));
        assert!(output.contains("\"values\":[0,0,1,1,1]"));
        assert!(output.matches("\"values\":[0,0,0,0,0]").count() >= 2);
        assert!(output.contains("\"values\":[0,1,0,0,0]"));
        assert!(output.contains("\"values\":[0,1,0,0,0]"));
        assert!(output.contains("\"values\":[0,0,1,0,0]"));
        assert!(output.contains("\"values\":[20,41,63,86,110]"));
        assert!(output.contains(
            "\"values\":[null,null,0.816496580927726,0.816496580927726,0.816496580927726]"
        ));
        assert!(output.contains(
            "\"values\":[null,null,0.9999999999999858,1.0000000000000284,1.0000000000000284]"
        ));
        assert!(output.contains(
            "\"values\":[null,null,0.6666666666666666,0.6666666666666666,0.6666666666666666]"
        ));
        assert!(output.contains("\"values\":[null,null,1,1,1]"));
        assert!(output.contains(
            "\"values\":[null,null,21.333333333333332,22.333333333333332,23.333333333333332]"
        ));
        assert!(output.contains("\"values\":[null,null,null,21.5,22.5]"));
        assert!(output.contains("\"values\":[null,null,null,null,24]"));
        assert!(
            output.contains("\"values\":[null,null,null,22.462027683060324,23.462027683060324]")
        );
        assert!(output.contains("\"values\":[null,null,22,23,24]"));
        assert!(output.contains(
            "\"values\":[null,null,0.15552315827194782,0.1484539238050411,0.14199940537873496]"
        ));
        assert!(output.contains("\"values\":[null,null,1,1,1]"));
        assert!(output.contains(
            "\"values\":[null,null,0.6666666666666572,0.6666666666666856,0.6666666666666856]"
        ));
        assert!(output.contains("\"values\":[null,null,21,22,23]"));
        assert!(output.contains("\"values\":[null,null,20,21,22]"));
        assert!(output.contains("\"values\":[null,null,100,100,100]"));
        assert!(output.contains(
            "\"values\":[20,20.333333333333332,20.88888888888889,21.59259259259259,22.395061728395063]"
        ));
        assert!(output.contains("\"values\":[20,20.75,21.75,22.8125,23.875]"));
        assert!(output.contains("\"values\":[20,20.875,21.9375,23,24.03125]"));
        assert!(output.contains("\"values\":[null,1,1,1,1]"));
        assert!(output.contains("\"values\":[null,null,null,100,100]"));
        assert!(output.contains("\"values\":[null,null,100,100,100]"));
        assert!(output.contains("\"values\":[null,null,325,325,325]"));
        assert!(output.contains("\"values\":[null,null,225,225,225]"));
        assert!(output.contains("\"values\":[null,9,9,9.16,9.4504]"));
        assert!(output.contains(
            "\"values\":[null,null,100.00000000000001,100.00000000000001,100.00000000000001]"
        ));
        assert!(output.contains(
            "\"values\":[null,null,-1.9682539682539681,-1.9696969696969697,-1.9710144927536233]"
        ));
        assert!(output.contains("\"values\":[5,5,5,5,5]"));
        assert!(output.contains("\"values\":[20,21,22,23,24]"));
        assert!(output.contains("\"values\":[10,10,10,10,10]"));
        assert!(output.contains(
            "\"values\":[0.4,1.170731707317073,1.5058823529411764,1.6271186440677967,1.6476964769647697]"
        ));
        assert!(
            output
                .matches("\"values\":[null,null,null,null,null]")
                .count()
                >= 3
        );
        assert!(output.matches("\"values\":[null,null,21,22,23]").count() >= 4);
        assert!(output.contains("\"values\":[null,null,0,0,0]"));
        assert!(output.contains("\"values\":[null,null,2,2,2]"));
        assert!(output.contains("\"values\":[null,null,null,22,23]"));
        assert!(output.contains("\"values\":[20,20.5,21,21.5,22]"));
        assert!(output.contains("\"values\":[1000,2000,3000,4000,5000]"));
        assert!(output.contains("\"values\":[0.1,0.1,0.1,0.1,0.1]"));
        assert!(output.matches("\"values\":[1,1,1,1,1]").count() >= 2);
        assert!(output.contains("\"values\":[null,100,200,300,400]"));
        assert!(
            output.contains(
                "\"values\":[null,5,9.761904761904763,14.30735930735931,18.65518539431583]"
            )
        );
        assert!(output.contains("\"values\":[500,500,500,500,500]"));
        assert!(output.contains(
            "\"values\":[0,0.16666666666666785,0.30555555555555713,0.39351851851851904,0.4436728395061742]"
        ));
        assert!(output.contains(
            "\"values\":[0,0.1111111111111119,0.24074074074074206,0.3425925925925934,0.40997942386831393]"
        ));
        assert!(output.contains(
            "\"values\":[0,0.055555555555555955,0.06481481481481507,0.05092592592592565,0.03369341563786027]"
        ));
        assert!(output.contains("\"values\":[null,null,21,22,23]"));
        assert!(output.contains(
            "\"values\":[null,null,22.632993161855453,23.632993161855453,24.632993161855453]"
        ));
        assert!(output.contains(
            "\"values\":[null,null,19.367006838144547,20.367006838144547,21.367006838144547]"
        ));
        assert!(output.contains("\"values\":[20,21,22,23,24]"));
        assert!(output.contains("\"values\":[21,22,23,24,25]"));
        assert!(output.contains("\"values\":[20,20.5,21.25,22.125,23.0625]"));
        assert!(output.contains("\"values\":[24,32.5,37.25,40.125,42.0625]"));
        assert!(output.contains("\"values\":[16,8.5,5.25,4.125,4.0625]"));
        assert!(output.contains("\"values\":[14,6,6,6,6]"));
        assert!(output.contains("\"values\":[1,-1,-1,-1,-1]"));
        assert!(output.contains(
            "\"values\":[0,7.1428571428571415,8.620689655172411,9.223300970873785,9.530791788856305]"
        ));
        assert!(output.contains("\"values\":[0,50,75,87.5,93.75]"));
        assert!(output.contains("\"values\":[20,20.5,21,21.5,22]"));
        assert!(output.contains(
            "\"values\":[20,21.5,22.632993161855474,23.73606797749979,24.82842712474619]"
        ));
        assert!(output.contains(
            "\"values\":[20,19.5,19.367006838144526,19.26393202250021,19.17157287525381]"
        ));
        assert!(output.contains("\"values\":[null,null,101,101,201]"));
        assert!(output.contains("\"values\":[null,null,0,0,16.666666666666657]"));
        assert!(output.contains("\"values\":[null,null,null,null,150]"));
        assert!(output.contains("\"values\":[null,null,null,null,250]"));
        assert!(output.contains("\"values\":[null,null,null,null,50]"));
        assert!(output.contains("\"values\":[null,null,100,100,166.66666666666666]"));
        assert!(output.contains("\"values\":[null,null,160,160,333.3333333333333]"));
        assert!(output.contains("\"values\":[null,null,40,40,0]"));
        assert!(output.contains("\"values\":[null,null,100,100,150]"));
        assert!(output.contains("\"values\":[null,null,100,100,250]"));
        assert!(output.contains("\"values\":[null,null,100,100,50]"));
        assert!(output.contains("\"values\":[null,null,155,155,81.66666666666667]"));
        assert!(output.contains("\"values\":[null,null,1,1,-1]"));
        assert!(output.contains("\"values\":[null,null,0,0,71.42857142857143]"));
        assert!(output.contains("\"values\":[null,null,0,0,50]"));
        assert!(output.contains("\"values\":[null,20,21,22,23]"));
        assert!(output.contains("\"values\":[10,20,21,22,23]"));
        assert!(output.contains("\"values\":[0,1,1,1,1]"));
    }

    #[test]
    fn runs_imported_function_with_library_source_integration_fixture() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/import.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: vec![LibrarySourceSpec {
                key: "user/lib/1".to_owned(),
                path: workspace_path("tests/fixtures/libraries/import_lib.pine"),
            }],
            strategy_alert_template: None,
            strategy_running_alert: None,
        };

        let output = run_json_with_options(&options).expect("import integration fixture");

        assert!(output.contains("\"values\":[4,6,8,10]"));
    }

    #[test]
    fn run_json_treats_strategy_exit_wrong_entry_as_noop() {
        let base = std::env::temp_dir().join(format!(
            "pine-cli-wrong-entry-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let bars_path = base.with_extension("csv");
        fs::write(
            &bars_path,
            "time,open,high,low,close,volume\n0,1,1,1,1,1\n1,2,2,2,2,1\n2,3,3,3,3,1\n",
        )
        .expect("write bars");

        let options = RunOptions {
            path: workspace_path(
                "tests/fixtures/runtime/strategy_exit_unmatched_from_entry_noop.pine",
            ),
            bars_path: bars_path.display().to_string(),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: None,
            strategy_running_alert: None,
        };
        let output = run_json_with_options(&options).expect("strategy no-op output");

        assert!(output.contains(
            "\"orders\":[{\"id\":\"L\",\"barIndex\":1,\"time\":1,\"direction\":\"strategy.long\",\"qty\":2,\"price\":2}]"
        ));
        assert!(output.contains("\"trades\":[]"));
        assert!(output.contains("\"position\":[{\"barIndex\":1,\"size\":2,\"avgPrice\":2}]"));
        assert!(output.contains("\"diagnostics\":[]"));
        assert!(!output.contains("\"direction\":\"strategy.exit\""));
        assert!(!output.contains("pending"));
        assert!(!output.contains("reserved"));
        let _ = fs::remove_file(bars_path);
    }

    #[test]
    fn run_output_renders_strategy_order_alert_template() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/strategy_exit_metadata.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: Some(StrategyAlertTemplateOptions {
                template: "Order: {{strategy.order.alert_message}}".to_owned(),
                index: 1,
            }),
            strategy_running_alert: None,
        };

        let output = run_output_with_options(&options).expect("rendered alert template");

        assert_eq!(output, "Order: loss alert");
    }

    #[test]
    fn run_output_rejects_unknown_strategy_order_alert_placeholder() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/strategy_exit_metadata.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: Some(StrategyAlertTemplateOptions {
                template: "{{close}}".to_owned(),
                index: 1,
            }),
            strategy_running_alert: None,
        };

        let error = run_output_with_options(&options).expect_err("unknown placeholder fails");

        assert!(error.contains("unsupported strategy order-fill alert placeholder `{{close}}`"));
    }

    #[test]
    fn run_output_renders_strategy_running_alert() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/strategy_exit_metadata.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: None,
            strategy_running_alert: Some(StrategyRunningAlertOptions {
                config: RunningAlertConfig::new_strategy_order_fills(
                    "snapshot-1",
                    "NYSE:IBM",
                    "1",
                    "Running: {{strategy.order.alert_message}}",
                ),
                index: 1,
            }),
        };

        let output = run_output_with_options(&options).expect("rendered running alert");

        assert_eq!(output, "Running: loss alert");
    }

    #[test]
    fn run_output_rejects_unknown_strategy_running_alert_placeholder() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/strategy_exit_metadata.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: None,
            strategy_running_alert: Some(StrategyRunningAlertOptions {
                config: RunningAlertConfig::new_strategy_order_fills(
                    "snapshot-1",
                    "NYSE:IBM",
                    "1",
                    "{{close}}",
                ),
                index: 1,
            }),
        };

        let error = run_output_with_options(&options).expect_err("unknown placeholder fails");

        assert!(error.contains("unsupported strategy order-fill alert placeholder `{{close}}`"));
    }

    #[test]
    fn run_json_keeps_strategy_alert_template_output_out_of_default_json() {
        let options = RunOptions {
            path: workspace_path("tests/fixtures/runtime/strategy_exit_metadata.pine"),
            bars_path: workspace_path("tests/fixtures/runtime/bars.csv"),
            profile: false,
            request_bars: Vec::new(),
            library_sources: Vec::new(),
            strategy_alert_template: None,
            strategy_running_alert: None,
        };

        let output = run_json_with_options(&options).expect("default strategy alert JSON");

        assert!(output.contains("\"message\":\"loss alert\""));
        assert!(!output.contains("renderedMessage"));
    }
}
