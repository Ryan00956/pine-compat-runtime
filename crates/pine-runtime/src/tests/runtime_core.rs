use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirLiteral, HirStmt, HirStmtKind,
    PineType, Qualifier, SymbolId, ValueKind,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn reports_unsupported_runtime_call_from_top_level_dispatcher() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("unsupported dispatch")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.expect("HIR");
    let mut runtime = HistoricalRuntime::new(&program);

    let error = runtime
        .eval_call("runtime.unknown", CallSiteId(999), &[])
        .expect_err("unsupported runtime call should fail");

    assert_eq!(error.message, "unsupported runtime call `runtime.unknown`");
}

#[test]
fn rejects_hir_expression_past_runtime_eval_depth() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("runtime depth")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut program = analysis.hir.expect("HIR");
    program.statements = vec![HirStmt {
        kind: HirStmtKind::Expr(nested_unary_expr(MAX_RUNTIME_EVAL_DEPTH + 1)),
    }];

    let error =
        run_historical(&program, &[bar(1.0)]).expect_err("deep runtime expression should fail");

    assert_eq!(
        error.message,
        "runtime expression evaluation exceeded maximum depth"
    );
}

#[test]
fn evaluates_hir_while_expression_zero_iteration_as_na() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("runtime while expression zero")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut program = analysis.hir.expect("HIR");
    replace_first_plot_arg(
        &mut program,
        while_expr(bool_expr(false), Vec::new(), int_expr(1)),
    );

    let result = run_historical(&program, &[bar(1.0)]).expect("while expression runtime result");

    assert_eq!(result.plots[0].values, vec![PineValue::Na]);
}

#[test]
fn evaluates_hir_while_expression_latest_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("runtime while expression")
x = 0
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut program = analysis.hir.expect("HIR");
    let x = program
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol")
        .id;
    let increment = HirStmt {
        kind: HirStmtKind::Reassign {
            symbol: x,
            value: binary_expr(HirBinaryOp::Add, symbol_expr(x, int_type()), int_expr(1)),
        },
    };
    let condition = binary_expr(HirBinaryOp::Lt, symbol_expr(x, int_type()), int_expr(3));
    replace_first_plot_arg(
        &mut program,
        while_expr(condition, vec![increment], symbol_expr(x, int_type())),
    );

    let result = run_historical(&program, &[bar(1.0)]).expect("while expression runtime result");

    assert_eq!(result.plots[0].values, vec![PineValue::Int(3)]);
}

#[test]
fn evaluates_hir_while_expression_loop_control_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("runtime while expression control")
x = 0
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut program = analysis.hir.expect("HIR");
    let x = program
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol")
        .id;
    let increment = HirStmt {
        kind: HirStmtKind::Reassign {
            symbol: x,
            value: binary_expr(HirBinaryOp::Add, symbol_expr(x, int_type()), int_expr(1)),
        },
    };
    let continue_at_two = HirStmt {
        kind: HirStmtKind::If {
            condition: binary_expr(HirBinaryOp::Eq, symbol_expr(x, int_type()), int_expr(2)),
            then_branch: vec![HirStmt {
                kind: HirStmtKind::Continue,
            }],
            else_branch: Vec::new(),
        },
    };
    let break_at_four = HirStmt {
        kind: HirStmtKind::If {
            condition: binary_expr(HirBinaryOp::Eq, symbol_expr(x, int_type()), int_expr(4)),
            then_branch: vec![HirStmt {
                kind: HirStmtKind::Break,
            }],
            else_branch: Vec::new(),
        },
    };
    let condition = binary_expr(HirBinaryOp::Lt, symbol_expr(x, int_type()), int_expr(5));
    let result_expr = binary_expr(HirBinaryOp::Mul, symbol_expr(x, int_type()), int_expr(10));
    replace_first_plot_arg(
        &mut program,
        while_expr(
            condition,
            vec![increment, continue_at_two, break_at_four],
            result_expr,
        ),
    );

    let result = run_historical(&program, &[bar(1.0)]).expect("while expression runtime result");

    assert_eq!(result.plots[0].values, vec![PineValue::Int(30)]);
}

fn nested_unary_expr(depth: u32) -> HirExpr {
    let mut expr = int_expr(1);
    for _ in 0..depth {
        expr = HirExpr {
            kind: HirExprKind::Unary {
                op: pine_ir::HirUnaryOp::Plus,
                expr: Box::new(expr),
            },
            pine_type: int_type(),
            series_id: None,
        };
    }
    expr
}

fn bool_expr(value: bool) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(HirLiteral::Bool(value)),
        pine_type: PineType::new(Qualifier::Const, ValueKind::Bool),
        series_id: None,
    }
}

fn int_expr(value: i64) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(HirLiteral::Int(value)),
        pine_type: int_type(),
        series_id: None,
    }
}

fn symbol_expr(symbol: SymbolId, pine_type: PineType) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Symbol(symbol),
        pine_type,
        series_id: None,
    }
}

fn binary_expr(op: HirBinaryOp, left: HirExpr, right: HirExpr) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        pine_type: match op {
            HirBinaryOp::Eq
            | HirBinaryOp::NotEq
            | HirBinaryOp::Gt
            | HirBinaryOp::Gte
            | HirBinaryOp::Lt
            | HirBinaryOp::Lte
            | HirBinaryOp::And
            | HirBinaryOp::Or => PineType::new(Qualifier::Series, ValueKind::Bool),
            _ => PineType::new(Qualifier::Series, ValueKind::Int),
        },
        series_id: None,
    }
}

fn while_expr(condition: HirExpr, statements: Vec<HirStmt>, result: HirExpr) -> HirExpr {
    HirExpr {
        kind: HirExprKind::While {
            condition: Box::new(condition),
            statements,
            result: Box::new(result),
        },
        pine_type: int_type(),
        series_id: None,
    }
}

fn replace_first_plot_arg(program: &mut pine_ir::HirProgram, value: HirExpr) {
    for statement in &mut program.statements {
        let HirStmtKind::Expr(HirExpr {
            kind: HirExprKind::Call { callee, args, .. },
            ..
        }) = &mut statement.kind
        else {
            continue;
        };
        if callee == "plot" {
            args[0] = HirCallArg { name: None, value };
            return;
        }
    }
    panic!("expected plot call");
}

fn int_type() -> PineType {
    PineType::new(Qualifier::Const, ValueKind::Int)
}

#[test]
fn preserves_var_state_across_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("var")
var x = 0
x := x + 1
plot(close + x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(2.0),
            PineValue::Float(4.0),
            PineValue::Float(6.0),
        ]
    );
}

#[test]
fn numeric_equality_uses_exact_pine_comparison() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("exact equality")
plot(0.1 + 0.2 == 0.3 ? 99 : 1)
plot(1 == 1.0 ? 1 : 0)
plot(na(0 / 0) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0]);
    assert_values_close(&result.plots[1].values, &[1.0]);
    assert_values_close(&result.plots[2].values, &[1.0]);
}

#[test]
fn profiles_runtime_storage() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("profile")
ma = ta.sma(close, 2)
plot(ma)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.profile.bars, 3);
    assert_eq!(profiled.profile.series_buffers, 0);
    assert_eq!(profiled.profile.series_values, 0);
    assert!(profiled.profile.series_capacity >= profiled.profile.series_values);
    assert_eq!(profiled.profile.max_series_depth, 0);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::StaticTrimmed
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 0);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(!profiled.profile.history_has_dynamic_offsets);
    assert_eq!(profiled.profile.rolling_window_slots, 1);
    assert_eq!(profiled.profile.rolling_window_values, 2);
    assert!(
        profiled.profile.rolling_window_value_capacity >= profiled.profile.rolling_window_values
    );
    assert_eq!(profiled.profile.plots, 1);
    assert_eq!(profiled.profile.plot_values, 3);
    assert!(profiled.profile.plot_capacity >= profiled.profile.plot_values);
    assert_eq!(profiled.profile.plot_shapes, 0);
    assert_eq!(profiled.profile.plot_arrows, 0);
    assert_eq!(profiled.profile.plot_bars, 0);
    assert_eq!(profiled.profile.plot_candles, 0);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[1..], &[1.5, 2.5]);
}

#[test]
fn trims_constant_history_to_required_depth() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("static history")
plot(close[2])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_eq!(profiled.result.plots[0].values[1], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[2..], &[1.0, 2.0]);
    assert_eq!(profiled.profile.max_series_depth, 2);
    assert_eq!(profiled.profile.series_values, 2);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::StaticTrimmed
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 2);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(!profiled.profile.history_has_dynamic_offsets);
}

#[test]
fn keeps_full_history_when_dynamic_offsets_exist() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("dynamic history retention")
length = input.int(1, "Length")
plot(close[length])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
    assert_eq!(profiled.profile.max_series_depth, 4);
    assert!(profiled.profile.series_values >= 4);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::DynamicFull
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 0);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(profiled.profile.history_has_dynamic_offsets);
}

#[test]
fn max_bars_back_bounds_dynamic_history_retention() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("dynamic history retention", max_bars_back=2)
offset = bar_index == 0 ? 0 : 3
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Float(1.0));
    assert_eq!(profiled.result.plots[0].values[1..], vec![PineValue::Na; 3]);
    assert_eq!(profiled.profile.max_series_depth, 2);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::MaxBarsBack
    );
    assert_eq!(profiled.profile.history_max_bars_back, Some(2));
    assert!(profiled.profile.history_has_dynamic_offsets);
    assert_eq!(profiled.profile.history_dynamic_retention_misses, 3);
    assert_eq!(
        profiled.profile.history_dynamic_retention_max_missed_offset,
        Some(3)
    );
}

#[test]
fn max_bars_back_function_bounds_only_declared_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("series max_bars_back")
max_bars_back(close, 2)
offset = bar_index == 0 ? 0 : 3
plot(close[offset])
plot(open[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 2);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Float(1.0));
    assert_eq!(profiled.result.plots[0].values[1..], vec![PineValue::Na; 3]);
    assert_eq!(profiled.result.plots[1].values[0], PineValue::Float(1.0));
    assert_eq!(profiled.result.plots[1].values[1], PineValue::Na);
    assert_eq!(profiled.result.plots[1].values[2], PineValue::Na);
    assert_eq!(profiled.result.plots[1].values[3], PineValue::Float(1.0));
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::MaxBarsBack
    );
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert_eq!(profiled.profile.history_dynamic_retention_misses, 3);
    assert_eq!(
        profiled.profile.history_dynamic_retention_max_missed_offset,
        Some(3)
    );
}

#[test]
fn append_bar_matches_full_historical_run() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("incremental")
ma = ta.sma(close, 3)
e = ta.ema(close, 2)
plot(ma)
plot(e)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];

    let full = run_historical(&hir, &bars).expect("full result");
    let mut runtime = HistoricalRuntime::new(&hir);
    for (index, bar) in bars.iter().copied().enumerate() {
        runtime.append_bar(bar).expect("append result");
        assert_eq!(runtime.profile().bars, index + 1);
    }
    let incremental = runtime.result();

    assert_eq!(incremental, full);
}

#[test]
fn bar_update_model_marks_committing_updates() {
    let bar = bar(1.0);

    assert!(BarUpdate::historical(bar).commits_series());
    assert!(BarUpdate::confirmed(bar).commits_series());
    assert!(!BarUpdate::forming(bar).commits_series());
}

#[test]
fn runs_barstate_isfirst_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate")
plot(barstate.isfirst ? 1 : 0)
plot(barstate.islast ? 1 : 0)
plot(barstate.islastconfirmedhistory ? 1 : 0)
plot(barstate.isnew ? 1 : 0)
plot(barstate.isconfirmed ? 1 : 0)
plot(barstate.ishistory ? 1 : 0)
plot(barstate.isrealtime ? 1 : 0)
plot(session.ismarket ? 1 : 0)
plot(session.ispremarket ? 1 : 0)
plot(session.ispostmarket ? 1 : 0)
plot(session.ismarket and not session.ispremarket and not session.ispostmarket ? 1 : 0)
plot(session.isfirstbar ? 1 : 0)
plot(session.islastbar ? 1 : 0)
plot(session.isfirstbar_regular ? 1 : 0)
plot(session.islastbar_regular ? 1 : 0)
plot(syminfo.session == session.regular ? 1 : 0)
plot(syminfo.session == session.extended ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[9].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[12].values, &[0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[14].values, &[0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[0.0, 0.0, 0.0]);
}

#[test]
fn append_bar_treats_current_open_ended_historical_bar_as_last() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate append")
plot(barstate.islast ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let hir = analysis.hir.expect("HIR");
    let mut runtime = HistoricalRuntime::new(&hir);
    runtime.append_bar(bar(1.0)).expect("first append");
    runtime.append_bar(bar(2.0)).expect("second append");
    let result = runtime.result();

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
}
