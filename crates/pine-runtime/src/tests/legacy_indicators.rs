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
