use pine_ir::{
    HirCallArg, HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt, HirStmtKind, PineType,
    Qualifier, ScriptMode, ValueKind,
};
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
    assert_eq!(strategy.equity.len(), 2);
    assert_eq!(strategy.equity[0].bar_index, 0);
    assert_eq!(strategy.equity[0].cash, 100_000.0);
    assert_eq!(strategy.equity[0].market_value, 0.0);
    assert_eq!(strategy.equity[0].equity, 100_000.0);
    assert_eq!(strategy.equity[0].net_profit, 0.0);
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

fn const_float_arg(name: &str, value: f64) -> HirCallArg {
    HirCallArg {
        name: Some(name.to_owned()),
        value: HirExpr {
            kind: HirExprKind::Literal(HirLiteral::Float(value)),
            pine_type: PineType::new(Qualifier::Const, ValueKind::Float),
            series_id: None,
        },
    }
}

fn strategy_exit_args_mut(program: &mut HirProgram) -> &mut Vec<HirCallArg> {
    fn find_in_stmts(statements: &mut [HirStmt]) -> Option<&mut Vec<HirCallArg>> {
        for statement in statements {
            match &mut statement.kind {
                HirStmtKind::Expr(expr) => {
                    if let Some(args) = find_in_expr(expr) {
                        return Some(args);
                    }
                }
                HirStmtKind::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    if let Some(args) = find_in_stmts(then_branch) {
                        return Some(args);
                    }
                    if let Some(args) = find_in_stmts(else_branch) {
                        return Some(args);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_in_expr(expr: &mut HirExpr) -> Option<&mut Vec<HirCallArg>> {
        if let HirExprKind::Call { callee, args, .. } = &mut expr.kind
            && callee == "strategy.exit"
        {
            return Some(args);
        }
        None
    }

    find_in_stmts(&mut program.statements).expect("strategy.exit call")
}

#[test]
fn strategy_entry_opens_long_position_at_current_close() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("entry")
if bar_index == 1
    strategy.entry("L", strategy.long, qty=2)
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.orders[0].id, "L");
    assert_eq!(strategy.orders[0].bar_index, 1);
    assert_eq!(strategy.orders[0].time, 20);
    assert_eq!(strategy.orders[0].direction, "strategy.long");
    assert_eq!(strategy.orders[0].qty, 2.0);
    assert_eq!(strategy.orders[0].price, 2.0);
    assert_eq!(strategy.position.len(), 1);
    assert_eq!(strategy.position[0].bar_index, 1);
    assert_eq!(strategy.position[0].size, 2.0);
    assert_eq!(strategy.position[0].avg_price, Some(2.0));
    assert_eq!(strategy.equity[1].cash, 99_996.0);
    assert_eq!(strategy.equity[1].market_value, 4.0);
    assert_eq!(strategy.equity[1].equity, 100_000.0);
}

#[test]
fn strategy_entry_ignores_repeated_entry_without_pyramiding() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("entry")
strategy.entry("L1", strategy.long, qty=1)
strategy.entry("L2", strategy.long, qty=1)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result =
        run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.orders[0].id, "L1");
    assert_eq!(strategy.position.len(), 1);
}

#[test]
fn strategy_entry_uses_fixed_default_qty_when_qty_is_absent() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("entry", default_qty_type=strategy.fixed, default_qty_value=3)
if bar_index == 0
    strategy.entry("D", strategy.long)
if bar_index == 1
    strategy.entry("E", strategy.long, qty=5)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(
        analysis
            .hir
            .as_ref()
            .unwrap()
            .strategy_settings
            .default_entry_qty(),
        Some(3.0)
    );

    let result =
        run_historical(&analysis.hir.expect("HIR"), &[bar(2.0), bar(4.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.orders[0].id, "D");
    assert_eq!(strategy.orders[0].qty, 3.0);
    assert_eq!(strategy.orders[0].price, 2.0);
    assert_eq!(strategy.position[0].size, 3.0);
    assert_eq!(strategy.equity[0].cash, 99_994.0);
    assert_eq!(strategy.equity[0].market_value, 6.0);
    assert_eq!(strategy.equity[1].equity, 100_006.0);
}

#[test]
fn strategy_entry_uses_builtin_default_qty_when_qty_is_absent() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("entry")
if bar_index == 0
    strategy.entry("D", strategy.long)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(
        analysis
            .hir
            .as_ref()
            .unwrap()
            .strategy_settings
            .default_entry_qty(),
        Some(1.0)
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(2.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.orders[0].id, "D");
    assert_eq!(strategy.orders[0].qty, 1.0);
    assert_eq!(strategy.orders[0].price, 2.0);
}

#[test]
fn strategy_entry_explicit_qty_overrides_fixed_default_qty() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("entry", default_qty_type=strategy.fixed, default_qty_value=3)
if bar_index == 0
    strategy.entry("E", strategy.long, qty=5)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(2.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.orders[0].id, "E");
    assert_eq!(strategy.orders[0].qty, 5.0);
    assert_eq!(strategy.position[0].size, 5.0);
    assert_eq!(strategy.equity[0].cash, 99_990.0);
}

#[test]
fn strategy_close_records_closed_trade_and_flat_position() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("close")
if bar_index == 1
    strategy.entry("L", strategy.long, qty=2)
if bar_index == 2
    strategy.close("L")
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 1.0,
        },
        Bar {
            time: 30,
            open: 3.0,
            high: 3.0,
            low: 3.0,
            close: 3.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.trades[0].entry_bar_index, 1);
    assert_eq!(strategy.trades[0].exit_bar_index, 2);
    assert_eq!(strategy.trades[0].entry_time, 20);
    assert_eq!(strategy.trades[0].exit_time, 30);
    assert_eq!(strategy.trades[0].entry_price, 2.0);
    assert_eq!(strategy.trades[0].exit_price, 3.0);
    assert_eq!(strategy.trades[0].qty, 2.0);
    assert_eq!(strategy.trades[0].profit, 2.0);
    assert_eq!(strategy.position.len(), 2);
    assert_eq!(strategy.position[1].size, 0.0);
    assert_eq!(strategy.position[1].avg_price, None);
    assert_eq!(strategy.equity[2].cash, 100_002.0);
    assert_eq!(strategy.equity[2].market_value, 0.0);
    assert_eq!(strategy.equity[2].equity, 100_002.0);
    assert_eq!(strategy.equity[2].net_profit, 2.0);
}

#[test]
fn strategy_exit_stop_stages_pending_exit_without_public_fill() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
strategy.entry("L", strategy.long, qty=1)
strategy.exit("XL", "L", stop=low)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(2.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert!(strategy.trades.is_empty());
    assert_eq!(strategy.position.len(), 1);
    assert_eq!(strategy.equity.len(), 1);
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_stop_without_matching_entry_records_strategy_diagnostic() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
strategy.exit("XL", "L", stop=low)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(2.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert!(strategy.orders.is_empty());
    assert!(strategy.trades.is_empty());
    assert!(strategy.position.is_empty());
    assert_eq!(strategy.equity.len(), 1);
    assert_eq!(strategy.diagnostics.len(), 1);
    assert_eq!(strategy.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn strategy_exit_trailing_hir_dispatches_to_broker_validation() {
    let source = SourceFile::new(
        "strategy_exit_trailing_dispatch_hir.pine",
        r#"strategy("exit")
strategy.exit("XT", "L", stop=95)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut hir = analysis.hir.expect("HIR");
    let args = strategy_exit_args_mut(&mut hir);
    let stop_arg = args
        .iter_mut()
        .find(|arg| arg.name.as_deref() == Some("stop"))
        .expect("stop arg");
    stop_arg.name = Some("trail_price".to_owned());
    args.push(const_float_arg("trail_offset", 50.0));

    let result = run_historical(&hir, &[bar(100.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert!(strategy.orders.is_empty());
    assert!(strategy.trades.is_empty());
    assert!(strategy.position.is_empty());
    assert_eq!(strategy.diagnostics.len(), 1);
    assert_eq!(strategy.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn strategy_exit_malformed_trailing_hir_is_guarded_before_fixed_exit() {
    let source = SourceFile::new(
        "strategy_exit_trailing_guard_hir.pine",
        r#"strategy("exit")
strategy.exit("XT", "L", stop=95)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut hir = analysis.hir.expect("HIR");
    let args = strategy_exit_args_mut(&mut hir);
    args.push(const_float_arg("trail_price", 100.0));
    args.push(const_float_arg("trail_offset", 50.0));

    let result = run_historical(&hir, &[bar(100.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert!(strategy.orders.is_empty());
    assert!(strategy.trades.is_empty());
    assert!(strategy.position.is_empty());
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_stop_fills_on_later_low_crossing_bar() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XL", "L", stop=9)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 10.0,
            low: 8.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 11.0,
            high: 12.0,
            low: 8.0,
            close: 11.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "XL");
    assert_eq!(strategy.orders[1].bar_index, 1);
    assert_eq!(strategy.orders[1].direction, "strategy.exit");
    assert_eq!(strategy.orders[1].qty, 2.0);
    assert_eq!(strategy.orders[1].price, 9.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.trades[0].entry_bar_index, 0);
    assert_eq!(strategy.trades[0].exit_bar_index, 1);
    assert_eq!(strategy.trades[0].entry_price, 10.0);
    assert_eq!(strategy.trades[0].exit_price, 9.0);
    assert_eq!(strategy.trades[0].qty, 2.0);
    assert_eq!(strategy.trades[0].profit, -2.0);
    assert_eq!(strategy.position.len(), 2);
    assert_eq!(strategy.position[1].size, 0.0);
    assert_eq!(strategy.equity[1].cash, 99_998.0);
    assert_eq!(strategy.equity[1].market_value, 0.0);
    assert_eq!(strategy.equity[1].net_profit, -2.0);
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_qty_single_trigger_forms_dispatch_partial_quantity() {
    for (name, exit_call, high, low, expected_price) in [
        (
            "stop",
            r#"strategy.exit("XQ", "L", stop=95, qty=0.75)"#,
            100.0,
            94.0,
            95.0,
        ),
        (
            "limit",
            r#"strategy.exit("XQ", "L", limit=110, qty=0.75)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "profit",
            r#"strategy.exit("XQ", "L", profit=1000, qty=0.75)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "loss",
            r#"strategy.exit("XQ", "L", loss=500, qty=0.75)"#,
            100.0,
            94.0,
            95.0,
        ),
    ] {
        let source = SourceFile::new(
            format!("strategy_exit_qty_{name}.pine"),
            format!(
                r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    {exit_call}
"#
            ),
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );

        let result = run_historical(
            &analysis.hir.expect("HIR"),
            &[
                bar_ohlc(100.0, 100.0, 100.0, 100.0),
                bar_ohlc(100.0, high, low, 100.0),
            ],
        )
        .expect("runtime result");
        let strategy = result.strategy.expect("strategy output");

        assert_eq!(strategy.orders.len(), 2, "{name}");
        assert_eq!(strategy.orders[1].id, "XQ", "{name}");
        assert_eq!(strategy.orders[1].qty, 0.75, "{name}");
        assert_eq!(strategy.orders[1].price, expected_price, "{name}");
        assert_eq!(strategy.trades.len(), 1, "{name}");
        assert_eq!(strategy.trades[0].qty, 0.75, "{name}");
        assert_eq!(strategy.trades[0].exit_price, expected_price, "{name}");
        assert_eq!(strategy.position[1].size, 1.25, "{name}");
        assert_eq!(strategy.position[1].avg_price, Some(100.0), "{name}");
        assert!(strategy.diagnostics.is_empty(), "{name}");
    }
}

#[test]
fn strategy_exit_stop_not_reached_keeps_position_open() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", stop=9)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 11.0,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert!(strategy.trades.is_empty());
    assert_eq!(strategy.position.len(), 1);
    assert_eq!(strategy.equity[1].market_value, 11.0);
    assert_eq!(strategy.equity[1].net_profit, 1.0);
}

#[test]
fn strategy_exit_stop_replacement_uses_updated_stop_on_later_bar() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", stop=8)
if bar_index == 1
    strategy.exit("XL", "L", stop=9)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 10.0,
            high: 10.0,
            low: 8.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 30,
            open: 10.0,
            high: 10.0,
            low: 8.0,
            close: 10.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_bar_index, 2);
    assert_eq!(strategy.trades[0].exit_price, 9.0);
    assert_eq!(strategy.orders[1].price, 9.0);
}

#[test]
fn strategy_close_cancels_pending_stop_before_evaluation() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", stop=9)
if bar_index == 1
    strategy.close("L")
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 11.0,
            high: 11.0,
            low: 8.0,
            close: 11.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_price, 11.0);
    assert_eq!(strategy.trades[0].profit, 1.0);
}

#[test]
fn strategy_exit_limit_fills_on_later_high_crossing_bar() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XL", "L", limit=12)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 12.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 11.0,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "XL");
    assert_eq!(strategy.orders[1].bar_index, 1);
    assert_eq!(strategy.orders[1].direction, "strategy.exit");
    assert_eq!(strategy.orders[1].price, 12.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.trades[0].exit_price, 12.0);
    assert_eq!(strategy.trades[0].profit, 4.0);
    assert_eq!(strategy.position[1].size, 0.0);
    assert_eq!(strategy.equity[1].cash, 100_004.0);
}

#[test]
fn strategy_exit_limit_not_reached_keeps_position_open() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", limit=12)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(10.0, 10.0, 10.0, 10.0),
            bar_ohlc(11.0, 11.0, 10.0, 11.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert!(strategy.trades.is_empty());
    assert_eq!(strategy.position.len(), 1);
    assert_eq!(strategy.equity[1].market_value, 11.0);
}

#[test]
fn strategy_exit_limit_replacement_uses_updated_limit_on_later_bar() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", limit=12)
if bar_index == 1
    strategy.exit("XL", "L", limit=11)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(10.0, 10.0, 10.0, 10.0),
            bar_ohlc(10.0, 11.0, 10.0, 10.0),
            bar_ohlc(10.0, 11.0, 10.0, 10.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_bar_index, 2);
    assert_eq!(strategy.trades[0].exit_price, 11.0);
}

#[test]
fn strategy_exit_profit_ticks_fill_through_converted_limit() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XP", "L", profit=200)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(10.0, 12.0, 10.0, 10.0),
            bar_ohlc(11.0, 12.0, 10.0, 11.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "XP");
    assert_eq!(strategy.orders[1].bar_index, 1);
    assert_eq!(strategy.orders[1].direction, "strategy.exit");
    assert_eq!(strategy.orders[1].price, 12.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.trades[0].exit_price, 12.0);
    assert_eq!(strategy.trades[0].profit, 4.0);
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_loss_ticks_fill_through_converted_stop() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XL", "L", loss=100)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(10.0, 10.0, 9.0, 10.0),
            bar_ohlc(10.0, 10.0, 9.0, 10.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "XL");
    assert_eq!(strategy.orders[1].bar_index, 1);
    assert_eq!(strategy.orders[1].direction, "strategy.exit");
    assert_eq!(strategy.orders[1].price, 9.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.trades[0].exit_price, 9.0);
    assert_eq!(strategy.trades[0].profit, -2.0);
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_bracket_forms_dispatch_to_bracket_pending_exit() {
    for (name, exit_call, high, low, expected_price) in [
        (
            "stop_limit",
            r#"strategy.exit("XB", "L", stop=95, limit=110)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "stop_profit",
            r#"strategy.exit("XB", "L", stop=95, profit=1000)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "loss_limit",
            r#"strategy.exit("XB", "L", loss=500, limit=110)"#,
            111.0,
            94.0,
            95.0,
        ),
        (
            "loss_profit",
            r#"strategy.exit("XB", "L", loss=500, profit=1000)"#,
            111.0,
            94.0,
            95.0,
        ),
    ] {
        let source = SourceFile::new(
            format!("strategy_exit_bracket_{name}.pine"),
            format!(
                r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    {exit_call}
"#
            ),
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );

        let result = run_historical(
            &analysis.hir.expect("HIR"),
            &[
                bar_ohlc(100.0, 100.0, 100.0, 100.0),
                bar_ohlc(100.0, high, low, 100.0),
            ],
        )
        .expect("runtime result");
        let strategy = result.strategy.expect("strategy output");

        assert_eq!(strategy.orders.len(), 2, "{name}");
        assert_eq!(strategy.orders[1].id, "XB", "{name}");
        assert_eq!(strategy.orders[1].bar_index, 1, "{name}");
        assert_eq!(strategy.orders[1].direction, "strategy.exit", "{name}");
        assert_eq!(strategy.orders[1].price, expected_price, "{name}");
        assert_eq!(strategy.trades.len(), 1, "{name}");
        assert_eq!(strategy.trades[0].id, "L", "{name}");
        assert_eq!(strategy.trades[0].exit_price, expected_price, "{name}");
        assert!(strategy.diagnostics.is_empty(), "{name}");
    }
}

#[test]
fn strategy_exit_qty_bracket_forms_dispatch_partial_quantity() {
    for (name, exit_call, high, low, expected_price) in [
        (
            "stop_limit",
            r#"strategy.exit("BQ", "L", stop=95, limit=110, qty=0.5)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "stop_profit",
            r#"strategy.exit("BQ", "L", stop=95, profit=1000, qty=0.5)"#,
            111.0,
            100.0,
            110.0,
        ),
        (
            "loss_limit",
            r#"strategy.exit("BQ", "L", loss=500, limit=110, qty=0.5)"#,
            111.0,
            94.0,
            95.0,
        ),
        (
            "loss_profit",
            r#"strategy.exit("BQ", "L", loss=500, profit=1000, qty=0.5)"#,
            111.0,
            94.0,
            95.0,
        ),
    ] {
        let source = SourceFile::new(
            format!("strategy_exit_qty_bracket_{name}.pine"),
            format!(
                r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    {exit_call}
"#
            ),
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );

        let result = run_historical(
            &analysis.hir.expect("HIR"),
            &[
                bar_ohlc(100.0, 100.0, 100.0, 100.0),
                bar_ohlc(100.0, high, low, 100.0),
            ],
        )
        .expect("runtime result");
        let strategy = result.strategy.expect("strategy output");

        assert_eq!(strategy.orders.len(), 2, "{name}");
        assert_eq!(strategy.orders[1].id, "BQ", "{name}");
        assert_eq!(strategy.orders[1].qty, 0.5, "{name}");
        assert_eq!(strategy.orders[1].price, expected_price, "{name}");
        assert_eq!(strategy.trades.len(), 1, "{name}");
        assert_eq!(strategy.trades[0].qty, 0.5, "{name}");
        assert_eq!(strategy.trades[0].exit_price, expected_price, "{name}");
        assert_eq!(strategy.position[1].size, 1.5, "{name}");
        assert!(strategy.diagnostics.is_empty(), "{name}");
    }
}

#[test]
fn strategy_exit_qty_trailing_dispatches_partial_quantity() {
    let source = SourceFile::new(
        "strategy_exit_qty_trailing.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("TQ", "L", trail_points=100, trail_offset=50, qty=0.5)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(100.0, 100.0, 100.0, 100.0),
            bar_ohlc(100.0, 102.0, 101.75, 102.0),
            bar_ohlc(102.0, 102.0, 101.0, 101.25),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "TQ");
    assert_eq!(strategy.orders[1].bar_index, 2);
    assert_eq!(strategy.orders[1].qty, 0.5);
    assert_eq!(strategy.orders[1].price, 101.5);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].qty, 0.5);
    assert_eq!(strategy.trades[0].exit_price, 101.5);
    assert_eq!(strategy.position[1].size, 1.5);
    assert!(strategy.diagnostics.is_empty());
}

#[test]
fn strategy_exit_bracket_invalid_downside_price_preempts_upside_tick_diagnostic() {
    let source = SourceFile::new(
        "strategy_exit_bracket_invalid_order.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XB", "L", stop=close / (close - close), profit=0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result =
        run_historical(&analysis.hir.expect("HIR"), &[bar(100.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert!(strategy.trades.is_empty());
    assert_eq!(strategy.diagnostics.len(), 1);
    assert_eq!(strategy.diagnostics[0].code, "E_STRATEGY_EXIT_PRICE");
}

#[test]
fn strategy_exit_invalid_bracket_preserves_existing_pending_exit() {
    let source = SourceFile::new(
        "strategy_exit_bracket_invalid_preserves_pending.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("KEEP", "L", limit=120)
if bar_index == 1
    strategy.exit("BAD", "L", stop=95, profit=0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(100.0, 100.0, 100.0, 100.0),
            bar_ohlc(100.0, 110.0, 100.0, 100.0),
            bar_ohlc(100.0, 121.0, 100.0, 100.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "KEEP");
    assert_eq!(strategy.orders[1].bar_index, 2);
    assert_eq!(strategy.orders[1].price, 120.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_price, 120.0);
    assert_eq!(strategy.diagnostics.len(), 1);
    assert_eq!(strategy.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
}

#[test]
fn strategy_exit_invalid_qty_preserves_existing_pending_exit() {
    let source = SourceFile::new(
        "strategy_exit_invalid_qty_preserves_pending.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("KEEP", "L", limit=120)
if bar_index == 1
    strategy.exit("BAD", "L", stop=95, qty=0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(100.0, 100.0, 100.0, 100.0),
            bar_ohlc(100.0, 110.0, 100.0, 100.0),
            bar_ohlc(100.0, 121.0, 94.0, 100.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.orders[1].id, "KEEP");
    assert_eq!(strategy.orders[1].bar_index, 2);
    assert_eq!(strategy.orders[1].qty, 2.0);
    assert_eq!(strategy.orders[1].price, 120.0);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_price, 120.0);
    assert_eq!(strategy.diagnostics.len(), 1);
    assert_eq!(strategy.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn strategy_exit_bracket_runtime_json_uses_existing_strategy_shape() {
    let source = SourceFile::new(
        "strategy_exit_bracket_json_shape.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=2)
    strategy.exit("XB", "L", stop=95, limit=110)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(100.0, 100.0, 100.0, 100.0),
            bar_ohlc(100.0, 111.0, 100.0, 100.0),
        ],
    )
    .expect("runtime result");
    let output = public_runtime_result_json(&result);

    assert!(output.contains(r#""strategy":{"orders":"#));
    assert!(!output.contains("pending"));
    assert!(!output.contains("bracket"));
    assert!(!output.contains("leg"));
    assert!(!output.contains("exitReason"));
}

#[test]
fn strategy_close_cancels_pending_limit_before_evaluation() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("exit")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", limit=12)
if bar_index == 1
    strategy.close("L")
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(10.0, 10.0, 10.0, 10.0),
            bar_ohlc(11.0, 12.0, 10.0, 11.0),
        ],
    )
    .expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.orders.len(), 1);
    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].exit_price, 11.0);
}

#[test]
fn strategy_initial_capital_and_equity_mark_open_position_to_close() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("equity", initial_capital=1000)
if bar_index == 0
    strategy.entry("L", strategy.long, qty=10)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(
        analysis
            .hir
            .as_ref()
            .unwrap()
            .strategy_settings
            .initial_capital,
        1000.0
    );

    let bars = [
        Bar {
            time: 10,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        },
        Bar {
            time: 20,
            open: 9.0,
            high: 9.0,
            low: 9.0,
            close: 9.0,
            volume: 1.0,
        },
        Bar {
            time: 30,
            open: 12.0,
            high: 12.0,
            low: 12.0,
            close: 12.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.equity.len(), 3);
    assert_eq!(strategy.equity[0].cash, 900.0);
    assert_eq!(strategy.equity[0].market_value, 100.0);
    assert_eq!(strategy.equity[0].equity, 1000.0);
    assert_eq!(strategy.equity[1].cash, 900.0);
    assert_eq!(strategy.equity[1].market_value, 90.0);
    assert_eq!(strategy.equity[1].equity, 990.0);
    assert_eq!(strategy.equity[1].net_profit, -10.0);
    assert_eq!(strategy.equity[2].cash, 900.0);
    assert_eq!(strategy.equity[2].market_value, 120.0);
    assert_eq!(strategy.equity[2].equity, 1020.0);
    assert_eq!(strategy.equity[2].net_profit, 20.0);
}

#[test]
fn strategy_close_without_matching_position_is_noop() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("close")
strategy.close("L")
strategy.entry("L", strategy.long, qty=1)
strategy.close("other")
strategy.close("L")
strategy.close("L")
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");
    let strategy = result.strategy.expect("strategy output");

    assert_eq!(strategy.trades.len(), 1);
    assert_eq!(strategy.trades[0].id, "L");
    assert_eq!(strategy.position.len(), 2);
}

#[test]
fn strategy_position_state_variables_follow_broker_mutations() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("position state")
plot(strategy.position_size)
plot(strategy.position_avg_price)
if bar_index == 1
    strategy.entry("L", strategy.long, qty=2)
plot(strategy.position_size)
plot(strategy.position_avg_price)
if bar_index == 2
    strategy.close("L")
plot(strategy.position_size)
plot(strategy.position_avg_price)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[bar(1.0), bar(2.0), bar(3.0), bar(4.0)],
    )
    .expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[1].values,
        vec![
            PineValue::Na,
            PineValue::Na,
            PineValue::Float(2.0),
            PineValue::Na,
        ]
    );
    assert_eq!(
        result.plots[2].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(2.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[3].values,
        vec![
            PineValue::Na,
            PineValue::Float(2.0),
            PineValue::Float(2.0),
            PineValue::Na,
        ]
    );
    assert_eq!(
        result.plots[4].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[5].values,
        vec![
            PineValue::Na,
            PineValue::Float(2.0),
            PineValue::Na,
            PineValue::Na,
        ]
    );
}

#[test]
fn strategy_trade_count_variables_follow_entry_and_close_mutations() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("trade count state")
plot(strategy.closedtrades)
plot(strategy.opentrades)
if bar_index == 1
    strategy.entry("L", strategy.long, qty=2)
plot(strategy.closedtrades)
plot(strategy.opentrades)
if bar_index == 2
    strategy.close("L")
plot(strategy.closedtrades)
plot(strategy.opentrades)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[bar(1.0), bar(2.0), bar(3.0), bar(4.0)],
    )
    .expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(1),
        ]
    );
    assert_eq!(
        result.plots[1].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(1),
            PineValue::Int(0),
        ]
    );
    assert_eq!(
        result.plots[2].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(1),
        ]
    );
    assert_eq!(
        result.plots[3].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(1),
            PineValue::Int(1),
            PineValue::Int(0),
        ]
    );
    assert_eq!(
        result.plots[4].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(1),
            PineValue::Int(1),
        ]
    );
    assert_eq!(
        result.plots[5].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(1),
            PineValue::Int(0),
            PineValue::Int(0),
        ]
    );
}

#[test]
fn strategy_trade_count_variables_observe_pending_exit_on_next_bar() {
    let source = SourceFile::new(
        "strategy.pine",
        r#"strategy("pending exit trade counts")
if bar_index == 0
    strategy.entry("L", strategy.long, qty=1)
    strategy.exit("XL", "L", limit=2.5)
plot(strategy.closedtrades)
plot(strategy.opentrades)
plot(strategy.closedtrades[1])
plot(strategy.opentrades[1])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[
            bar_ohlc(1.0, 1.0, 1.0, 1.0),
            bar_ohlc(2.0, 2.0, 2.0, 2.0),
            bar_ohlc(3.0, 3.0, 3.0, 3.0),
            bar_ohlc(4.0, 4.0, 4.0, 4.0),
        ],
    )
    .expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(1),
        ]
    );
    assert_eq!(
        result.plots[1].values,
        vec![
            PineValue::Int(1),
            PineValue::Int(1),
            PineValue::Int(1),
            PineValue::Int(0),
        ]
    );
    assert_eq!(
        result.plots[2].values,
        vec![
            PineValue::Na,
            PineValue::Int(0),
            PineValue::Int(0),
            PineValue::Int(0),
        ]
    );
    assert_eq!(
        result.plots[3].values,
        vec![
            PineValue::Na,
            PineValue::Int(1),
            PineValue::Int(1),
            PineValue::Int(1),
        ]
    );
}

#[test]
fn strategy_profit_state_variables_follow_realized_and_open_profit() {
    let source = SourceFile::new(
        "strategy.pine",
        include_str!("../../../../tests/fixtures/runtime/strategy_profit_state.pine"),
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[bar(1.0), bar(2.0), bar(3.0), bar(1.0), bar(4.0), bar(6.0)],
    )
    .expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(-2.0),
            PineValue::Float(0.0),
            PineValue::Float(2.0),
        ]
    );
    assert_eq!(
        result.plots[1].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(-2.0),
            PineValue::Float(-2.0),
        ]
    );
    assert_eq!(
        result.plots[2].values,
        vec![
            PineValue::Float(1000.0),
            PineValue::Float(1000.0),
            PineValue::Float(1002.0),
            PineValue::Float(998.0),
            PineValue::Float(998.0),
            PineValue::Float(1000.0),
        ]
    );
    assert_eq!(
        result.plots[3].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[4].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(-2.0),
            PineValue::Float(-2.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[5].values,
        vec![
            PineValue::Float(1000.0),
            PineValue::Float(1000.0),
            PineValue::Float(1002.0),
            PineValue::Float(998.0),
            PineValue::Float(998.0),
            PineValue::Float(1000.0),
        ]
    );
}

#[test]
fn strategy_variables_work_in_supported_expression_contexts() {
    let source = SourceFile::new(
        "strategy.pine",
        include_str!("../../../../tests/fixtures/runtime/strategy_variable_interactions.pine"),
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[bar(1.0), bar(2.0), bar(3.0), bar(4.0)],
    )
    .expect("runtime result");

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(-1.0),
            PineValue::Float(2.0),
            PineValue::Float(2.0),
            PineValue::Float(-1.0),
        ]
    );
    assert_eq!(
        result.plots[1].values,
        vec![
            PineValue::Float(-1.0),
            PineValue::Float(2.0),
            PineValue::Float(4.0),
            PineValue::Float(-1.0),
        ]
    );
    assert_eq!(
        result.plots[2].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(4.0),
            PineValue::Float(4.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[3].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(2.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[4].values,
        vec![
            PineValue::Float(0.0),
            PineValue::Float(20.0),
            PineValue::Float(20.0),
            PineValue::Float(0.0),
        ]
    );
    assert_eq!(
        result.plots[5].values,
        vec![
            PineValue::Na,
            PineValue::Float(0.0),
            PineValue::Float(2.0),
            PineValue::Float(2.0),
        ]
    );
    assert_eq!(
        result.plots[6].values,
        vec![
            PineValue::Na,
            PineValue::Float(0.0),
            PineValue::Float(0.0),
            PineValue::Float(2.0),
        ]
    );
}
