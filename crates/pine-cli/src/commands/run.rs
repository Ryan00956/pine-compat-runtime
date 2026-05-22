use std::fs;

use pine_runtime::{
    public_runtime_profiled_result_json, public_runtime_result_json, run_historical,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use crate::bars_csv::parse_bars_csv;
use crate::usage;

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let [path, flag, bars_path] = args.as_slice() else {
        if args.len() == 4 && args[1] == "--bars" && args[3] == "--profile" {
            return run_with_profile(&args[0], &args[2], true);
        }
        return Err(usage());
    };
    if flag != "--bars" {
        return Err(usage());
    }
    run_with_profile(path, bars_path, false)
}

fn run_with_profile(path: &str, bars_path: &str, profile: bool) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    let source = SourceFile::new(path, text);
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

    let bars_text = fs::read_to_string(bars_path)
        .map_err(|err| format!("failed to read {bars_path}: {err}"))?;
    let bars = parse_bars_csv(&bars_text)?;
    if profile {
        let result = pine_runtime::run_historical_profiled(&hir, &bars)
            .map_err(|err| format!("runtime failed: {}", err.message))?;
        println!(
            "{}",
            public_runtime_profiled_result_json(&result.result, &result.profile)
        );
    } else {
        let result = run_historical(&hir, &bars)
            .map_err(|err| format!("runtime failed: {}", err.message))?;
        println!("{}", public_runtime_result_json(&result));
    }
    Ok(())
}
