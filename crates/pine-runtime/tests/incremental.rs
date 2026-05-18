use std::{fs, path::PathBuf};

use pine_runtime::{Bar, HistoricalRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn runtime_fixtures_match_incremental_append_execution() {
    let fixtures_dir = workspace_fixture("tests/fixtures/runtime");
    let bars = load_bars(&workspace_fixture("tests/fixtures/runtime/bars.csv"));
    let mut checked = 0;

    for entry in fs::read_dir(&fixtures_dir).expect("runtime fixture dir should be readable") {
        let path = entry.expect("fixture entry should be readable").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("pine") {
            continue;
        }

        let text = fs::read_to_string(&path).expect("fixture should be readable");
        let source = SourceFile::new(path.display().to_string(), text);
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("runtime fixture should lower to HIR");

        let full = run_historical(&hir, &bars).expect("full execution should succeed");
        let mut runtime = HistoricalRuntime::new(&hir);
        for bar in bars.iter().copied() {
            runtime
                .append_bar(bar)
                .expect("append execution should succeed");
        }
        let incremental = runtime.result();

        assert_eq!(
            incremental,
            full,
            "{} incremental result should match full recomputation",
            path.display()
        );
        checked += 1;
    }

    assert!(checked >= 7, "expected runtime fixtures to be checked");
}

fn load_bars(path: &PathBuf) -> Vec<Bar> {
    let text = fs::read_to_string(path).expect("bars fixture should be readable");
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if line.trim().is_empty() || index == 0 {
                return None;
            }
            let columns: Vec<_> = line.split(',').map(str::trim).collect();
            assert_eq!(columns.len(), 6, "bars fixture should have 6 columns");
            Some(Bar {
                time: columns[0].parse().expect("time should parse"),
                open: columns[1].parse().expect("open should parse"),
                high: columns[2].parse().expect("high should parse"),
                low: columns[3].parse().expect("low should parse"),
                close: columns[4].parse().expect("close should parse"),
                volume: columns[5].parse().expect("volume should parse"),
            })
        })
        .collect()
}
