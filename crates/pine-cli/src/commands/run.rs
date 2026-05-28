use std::{fs, sync::Arc};

use pine_runtime::{
    ChartContext, InMemoryRequestDataProvider, RequestEnvironment, RequestKey, RequestTimeframe,
    public_runtime_profiled_result_json, public_runtime_result_json,
    run_historical_profiled_with_request_environment, run_historical_with_request_environment,
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
}

#[derive(Debug)]
struct RequestBarsSpec {
    key: RequestKey,
    path: String,
}

fn run_with_options(options: &RunOptions) -> Result<(), String> {
    println!("{}", run_json_with_options(options)?);
    Ok(())
}

fn run_json_with_options(options: &RunOptions) -> Result<String, String> {
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
    if options.profile {
        let result =
            run_historical_profiled_with_request_environment(&hir, &bars, request_environment)
                .map_err(|err| format!("runtime failed: {}", err.message))?;
        Ok(public_runtime_profiled_result_json(
            &result.result,
            &result.profile,
        ))
    } else {
        let result = run_historical_with_request_environment(&hir, &bars, request_environment)
            .map_err(|err| format!("runtime failed: {}", err.message))?;
        Ok(public_runtime_result_json(&result))
    }
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
        };

        let output = run_json_with_options(&options).expect("request integration fixture");

        assert!(output.contains("\"values\":[30,32,34,36,38]"));
        assert!(output.contains("\"values\":[null,null,100,100,200]"));
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
        };

        let output = run_json_with_options(&options).expect("import integration fixture");

        assert!(output.contains("\"values\":[4,6,8,10]"));
    }
}
