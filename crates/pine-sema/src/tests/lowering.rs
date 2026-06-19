use super::*;
use crate::analyzer::context::MAX_LOWERING_TEMP_SYMBOLS;

#[test]
fn infers_history_requirements() {
    let analysis =
        analyze("len = input.int(1, \"Length\")\nplot(close[3])\nplot((close + open)[len])\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.history.max_constant_offset, 3);
    assert!(hir.history.has_dynamic_offsets);
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.max_constant_offset == 3),
        "{:?}",
        hir.series_history
    );
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.has_dynamic_offsets),
        "{:?}",
        hir.series_history
    );
}

#[test]
fn infers_implicit_builtin_history_requirements() {
    let analysis = analyze(
        "len = input.int(1, \"Length\")\nplot(ta.tr)\nplot(ta.tr())\nplot(ta.change(open, 2))\nplot(ta.change(close, len))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.history.max_constant_offset, 2);
    assert!(hir.history.has_dynamic_offsets);
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.max_constant_offset == 2),
        "{:?}",
        hir.series_history
    );
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.has_dynamic_offsets),
        "{:?}",
        hir.series_history
    );
}

#[test]
fn infers_array_history_requirements() {
    let analysis = analyze(
        "var values = array.new_float(1)\nvalues.set(0, close)\nprevious = values[1]\nplot(bar_index == 0 ? na : previous.get(0))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let values = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "values")
        .expect("values symbol should exist");
    let series_id = values
        .series_id
        .expect("array symbol should be tracked as a series");
    assert!(
        hir.series_history.iter().any(|requirement| {
            requirement.series_id == series_id && requirement.max_constant_offset == 1
        }),
        "{:?}",
        hir.series_history
    );
}

#[test]
fn lowers_if_statement_to_hir() {
    let analysis = analyze("if close > open\n    plot(close)\nelse\n    plot(open)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "if")
    );
    let hir = analysis.hir.expect("if statement should lower");
    assert!(matches!(hir.statements[0].kind, HirStmtKind::If { .. }));
}

#[test]
fn lowers_valid_script_to_hir() {
    let analysis = analyze(
        r#"indicator("Demo", overlay=true)
length = input.int(20, "Length")
ma = ta.sma(close, length)
plot(ma)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("valid script should lower to HIR");
    assert_eq!(hir.statements.len(), 4);
    assert!(hir.next_call_site_id >= 3);
    assert!(hir.next_series_id > 10);
}

#[test]
fn lowers_var_declaration_to_var_slot() {
    let analysis = analyze("var x = 0\nx := x + 1\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("valid script should lower to HIR");
    let symbol = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol should exist");
    assert_eq!(symbol.persistence, PersistenceKind::Var);
    assert_eq!(symbol.var_slot_id, Some(VarSlotId(0)));
}

#[test]
fn lowers_plain_declaration_without_persistence() {
    let analysis = analyze("x = 0\nplot(x)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("valid script should lower to HIR");
    let symbol = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol should exist");
    assert_eq!(symbol.persistence, PersistenceKind::None);
    assert_eq!(symbol.var_slot_id, None);
}

#[test]
fn skips_hir_when_semantic_errors_exist() {
    let analysis = analyze("plot()\n");

    assert!(analysis.hir.is_none());
}

#[test]
fn lowers_tuple_assignment() {
    let analysis = analyze("[a, b] = [close, open]\nplot(a)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("valid tuple assignment should lower");
    assert!(
        hir.symbols
            .iter()
            .any(|symbol| symbol.name == "a" && symbol.series_id.is_some())
    );
}

#[test]
fn rejects_lowering_temp_symbol_budget_exhaustion() {
    let mut source = String::from("id(x) => x\n");
    for index in 0..=MAX_LOWERING_TEMP_SYMBOLS {
        source.push_str(&format!("x{index} = id(1)\n"));
    }

    let analysis = analyze(&source);

    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_LOWERING_BUDGET"
                && diagnostic.message.contains("temporary symbols")
        }),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}
