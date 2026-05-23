use std::{fs, sync::Arc};

use pine_runtime::{
    ChartContext, InMemoryRequestDataProvider, RequestEnvironment, RequestKey, RequestTimeframe,
    public_runtime_profiled_result_json, public_runtime_result_json,
    run_historical_profiled_with_request_environment, run_historical_with_request_environment,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use crate::bars_csv::parse_bars_csv;
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
}

#[derive(Debug)]
struct RequestBarsSpec {
    key: RequestKey,
    path: String,
}

fn run_with_options(options: &RunOptions) -> Result<(), String> {
    let text = fs::read_to_string(&options.path)
        .map_err(|err| format!("failed to read {}: {err}", options.path))?;
    let source = SourceFile::new(&options.path, text);
    let analysis = analyze_source(&source);
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
    if options.profile {
        let result =
            run_historical_profiled_with_request_environment(&hir, &bars, request_environment)
                .map_err(|err| format!("runtime failed: {}", err.message))?;
        println!(
            "{}",
            public_runtime_profiled_result_json(&result.result, &result.profile)
        );
    } else {
        let result = run_historical_with_request_environment(&hir, &bars, request_environment)
            .map_err(|err| format!("runtime failed: {}", err.message))?;
        println!("{}", public_runtime_result_json(&result));
    }
    Ok(())
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
    };
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
            "--request-bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options.request_bars.push(parse_request_bars_spec(value)?);
            }
            _ => return Err(usage()),
        }
        index += 1;
    }
    if options.bars_path.is_empty() {
        return Err(usage());
    }
    Ok(options)
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

    #[test]
    fn parses_request_bars_spec_with_exchange_prefixed_symbol() {
        let spec = parse_request_bars_spec("NASDAQ:AAPL:1=request.csv")
            .expect("request bars spec should parse");

        assert_eq!(spec.key.symbol(), "NASDAQ:AAPL");
        assert_eq!(spec.key.timeframe().value(), "1");
        assert_eq!(spec.path, "request.csv");
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
}
