use pine_ir::ScriptMode;
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn strategy_declaration_emits_empty_strategy_result() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("empty")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.script_mode, ScriptMode::Strategy);

    let result = run_historical(&hir, &[bar(1.0), bar(2.0)]).expect("runtime result");

    assert!(result.strategy.is_some());
    let strategy = result.strategy.expect("strategy output");
    assert!(strategy.orders.is_empty());
    assert!(strategy.trades.is_empty());
    assert!(strategy.position.is_empty());
    assert!(strategy.equity.is_empty());
    assert!(strategy.diagnostics.is_empty());
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Float(1.0), PineValue::Float(2.0)]
    );
}

#[test]
fn indicator_result_does_not_include_strategy_output() {
    let source = SourceFile::new(
        "indicator.pine",
        r#"indicator("plain")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert!(result.strategy.is_none());
}
