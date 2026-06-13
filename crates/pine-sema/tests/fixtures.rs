use std::{fs, path::PathBuf};

use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn reports_unsupported_request_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_request.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn reports_unsupported_request_lower_tf_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_lower_tf.pine",
        "request.security_lower_tf",
        "outside the supported request.security subset",
    );
}

#[test]
fn reports_unsupported_request_math_calls_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_math_calls.pine",
        "request.security",
        "same-context request.security",
    );
}

#[test]
fn accepts_provider_request_context_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/request_security_provider_context.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn reports_unsupported_varip_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_varip.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("tuples")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_varip_drawing_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_varip_drawing.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("drawing object ids")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_strategy_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn accepts_supported_strategy_declaration_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_declaration.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy"),
        "{} supported features: {:?}",
        path.display(),
        analysis.compatibility.supported
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
}

#[test]
fn accepts_supported_strategy_pyramiding_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_pyramiding.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.pyramiding_limit, 2);
}

#[test]
fn accepts_supported_strategy_initial_capital_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_initial_capital.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.initial_capital, 2500.0);
}

#[test]
fn accepts_supported_strategy_default_quantity_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_entry_qty(100.0, 10.0),
        Some(3.0)
    );
}

#[test]
fn accepts_supported_strategy_percent_of_equity_default_quantity_fixture() {
    let path = workspace_fixture(
        "tests/fixtures/sema/supported_strategy_percent_of_equity_default_quantity.pine",
    );
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_qty,
        Some(pine_ir::StrategyDefaultQuantity::PercentOfEquity(25.0))
    );
    assert_eq!(
        hir.strategy_settings.default_entry_qty(1000.0, 10.0),
        Some(25.0)
    );
}

#[test]
fn accepts_supported_strategy_cash_default_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_cash_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_qty,
        Some(pine_ir::StrategyDefaultQuantity::Cash(100.0))
    );
    assert_eq!(
        hir.strategy_settings.default_entry_qty(1000.0, 10.0),
        Some(10.0)
    );
}

#[test]
fn accepts_supported_strategy_commission_cash_per_contract_fixture() {
    let path = workspace_fixture(
        "tests/fixtures/sema/supported_strategy_commission_cash_per_contract.pine",
    );
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::CashPerContract(0.5))
    );
}

#[test]
fn accepts_supported_strategy_commission_cash_per_order_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_commission_cash_per_order.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::CashPerOrder(1.5))
    );
}

#[test]
fn accepts_supported_strategy_commission_percent_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_commission_percent.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::Percent(10.0))
    );
}

#[test]
fn accepts_supported_strategy_slippage_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_slippage.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.slippage_ticks, 100.0);
}

#[test]
fn accepts_supported_strategy_limit_verification_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_limit_verification.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.backtest_fill_limit_ticks, 100.0);
}

#[test]
fn accepts_supported_strategy_margin_declaration_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_margin_declaration.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.margin_long,
        pine_ir::StrategyMarginSetting::explicit(25.0)
    );
    assert_eq!(
        hir.strategy_settings.margin_short,
        pine_ir::StrategyMarginSetting::explicit(50.0)
    );
}

#[test]
fn reports_strategy_initial_capital_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_initial_capital.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_default_quantity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_default_quantity.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_commission_unknown_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_commission_unknown.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_slippage_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_slippage.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_limit_verification_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_limit_verification.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_margin_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_margin_declaration.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_declaration_properties_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        "E_CALL_ARG_NAME",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        &[
            "calc_on_order_fills",
            "calc_on_every_tick",
            "process_orders_on_close",
            "currency",
            "close_entries_rule",
            "risk_free_rate",
            "use_bar_magnifier",
            "fill_orders_on_standard_ohlc",
        ],
    );
}

#[test]
fn reports_unsupported_strategy_pyramiding_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        &["pyramiding"],
    );
}

#[test]
fn reports_unsupported_strategy_order_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_orders.pine",
        &["strategy.order"],
    );
}

#[test]
fn reports_unsupported_strategy_exit_variant_fixtures() {
    for (path, code) in [
        (
            "tests/fixtures/sema/unsupported_strategy_exit_stop_loss.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_limit_profit.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_three_triggers.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_four_triggers.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_qty_percent_same_side.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_qty_same_side.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_profit_trailing.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_price_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_points_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_offset_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_price_points.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_bracket.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_indicator.pine",
            "E_STRATEGY_MODE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_function_side_effect.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_request_strategy_trailing_exit.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_partial_quantity.pine",
            "E_CALL_ARITY",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_missing_entry.pine",
            "E_CALL_ARITY",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
            "E_CALL_ARG_TYPE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_function_side_effect.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_request_strategy_exit.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
    ] {
        assert_diagnostic_fixture(path, code);
    }
}

#[test]
fn reports_strategy_exit_quantity_guardrail_messages() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_qty_percent_same_side.pine",
        &["`strategy.exit` combined trigger families are not supported"],
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_qty_same_side.pine",
        &["`strategy.exit` combined trigger families are not supported"],
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_partial_quantity.pine",
        &["`strategy.exit` requires one of `stop`, `limit`, `profit`, or `loss`"],
    );
}

#[test]
fn accepts_supported_strategy_exit_fixtures() {
    for fixture in [
        "tests/fixtures/sema/supported_strategy_exit_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss.pine",
        "tests/fixtures/sema/supported_strategy_exit_stop_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_stop_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_trail_price.pine",
        "tests/fixtures/sema/supported_strategy_exit_trail_points.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_trailing.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_loss.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_trailing.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_trailing.pine",
        "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    ] {
        let path = workspace_fixture(fixture);
        let text = fs::read_to_string(&path).expect("fixture should be readable");
        let source = SourceFile::new(path.display().to_string(), text);
        let analysis = analyze_source(&source);

        assert!(
            analysis.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == "strategy.exit")
        );
        assert!(analysis.hir.is_some());
    }
}

#[test]
fn accepts_supported_strategy_entry_default_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_entry_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy entry should lower");
    assert_eq!(
        hir.strategy_settings.default_entry_qty(100.0, 10.0),
        Some(1.0)
    );
}

#[test]
fn accepts_supported_strategy_entry_fixture() {
    for fixture in [
        "tests/fixtures/sema/supported_strategy_entry.pine",
        "tests/fixtures/sema/supported_strategy_entry_limit.pine",
        "tests/fixtures/sema/supported_strategy_entry_stop.pine",
        "tests/fixtures/sema/supported_strategy_entry_stop_limit.pine",
    ] {
        let path = workspace_fixture(fixture);
        let text = fs::read_to_string(&path).expect("fixture should be readable");
        let source = SourceFile::new(path.display().to_string(), text);
        let analysis = analyze_source(&source);

        assert!(
            analysis.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == "strategy.entry")
        );
        assert!(analysis.hir.is_some());
    }
}

#[test]
fn reports_strategy_entry_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_entry_short_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_short.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_entry_short.pine",
        &["strategy.long"],
    );
}

#[test]
fn reports_strategy_entry_qty_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn accepts_supported_strategy_close_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn reports_unsupported_strategy_close_partial_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/unsupported_strategy_close_partial_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_NAME"
                && diagnostic
                    .message
                    .contains("partial quantity arguments must be named")
        }),
        "{} diagnostics should reject positional strategy.close quantity: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_VALUE"
                && diagnostic
                    .message
                    .contains("argument `qty` must be finite and positive")
        }),
        "{} diagnostics should reject non-positive strategy.close qty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_VALUE"
                && diagnostic
                    .message
                    .contains("argument `qty_percent` must be finite and positive")
        }),
        "{} diagnostics should reject non-positive strategy.close qty_percent: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let name = "immediately";
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_NAME" && diagnostic.message.contains(name)
        }),
        "{} diagnostics should reject strategy.close argument `{}`: {:?}",
        path.display(),
        name,
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_strategy_order_metadata_type_guardrails() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
        &[
            "argument `comment` does not accept",
            "argument `disable_alert` does not accept",
            "argument `alert_message` does not accept",
        ],
    );
}

#[test]
fn reports_strategy_close_immediately_remains_unsupported() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
        &["immediately"],
    );
}

#[test]
fn accepts_supported_strategy_close_qty_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_qty_percent_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty_percent.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics should be empty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_qty_precedence_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty_precedence.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics should be empty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_all_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_all.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close_all")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_position_state_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_position_state.pine",
        &[
            "strategy.position_size",
            "strategy.position_avg_price",
            "strategy.max_contracts_held_all",
            "strategy.max_contracts_held_long",
            "strategy.max_contracts_held_short",
        ],
    );
}

#[test]
fn accepts_supported_strategy_profit_state_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_profit_state.pine",
        &[
            "strategy.openprofit",
            "strategy.netprofit",
            "strategy.netprofit_percent",
            "strategy.grossprofit",
            "strategy.grossprofit_percent",
            "strategy.grossloss",
            "strategy.grossloss_percent",
            "strategy.avg_trade",
            "strategy.avg_trade_percent",
            "strategy.avg_winning_trade",
            "strategy.avg_winning_trade_percent",
            "strategy.avg_losing_trade",
            "strategy.avg_losing_trade_percent",
            "strategy.max_runup",
            "strategy.max_runup_percent",
            "strategy.max_drawdown",
            "strategy.max_drawdown_percent",
            "strategy.equity",
        ],
    );
}

#[test]
fn accepts_supported_strategy_variable_interactions_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_variable_interactions.pine",
        &[
            "strategy.position_size",
            "strategy.openprofit",
            "strategy.netprofit",
        ],
    );
}

#[test]
fn accepts_supported_strategy_trade_counts_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_counts.pine",
        &["strategy.closedtrades", "strategy.opentrades"],
    );
}

#[test]
fn accepts_supported_strategy_closedtrades_fields_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_closedtrades_fields.pine",
        &[
            "strategy.closedtrades.entry_price",
            "strategy.closedtrades.entry_id",
            "strategy.closedtrades.exit_price",
            "strategy.closedtrades.exit_id",
            "strategy.closedtrades.entry_bar_index",
            "strategy.closedtrades.exit_bar_index",
            "strategy.closedtrades.entry_time",
            "strategy.closedtrades.exit_time",
            "strategy.closedtrades.commission",
            "strategy.closedtrades.size",
            "strategy.closedtrades.profit",
            "strategy.closedtrades.max_runup",
            "strategy.closedtrades.max_drawdown",
        ],
    );
}

#[test]
fn accepts_supported_strategy_opentrades_fields_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_opentrades_fields.pine",
        &[
            "strategy.opentrades.entry_price",
            "strategy.opentrades.entry_id",
            "strategy.opentrades.entry_bar_index",
            "strategy.opentrades.entry_time",
            "strategy.opentrades.size",
            "strategy.opentrades.profit",
            "strategy.opentrades.commission",
            "strategy.opentrades.max_runup",
            "strategy.opentrades.max_drawdown",
            "strategy.opentrades.capital_held",
        ],
    );
}

#[test]
fn accepts_supported_strategy_trade_count_interactions_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_count_interactions.pine",
        &["strategy.closedtrades", "strategy.opentrades"],
    );
}

#[test]
fn accepts_supported_strategy_trade_outcome_counts_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_outcome_counts.pine",
        &[
            "strategy.wintrades",
            "strategy.losstrades",
            "strategy.eventrades",
        ],
    );
}

#[test]
fn reports_strategy_close_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_close_all_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_all_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_state_indicator_fixture() {
    assert_strategy_state_mode_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_indicator.pine",
    );
}

#[test]
fn reports_strategy_state_variables_fixture() {
    assert_strategy_state_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_variables.pine",
    );
}

#[test]
fn reports_unknown_strategy_variable_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_unknown_variable.pine",
        &["strategy.future_metric"],
    );
}

#[test]
fn reports_unsupported_strategy_order_and_trade_namespace_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
        &["strategy.risk.max_drawdown"],
    );
}

#[test]
fn reports_strategy_closedtrades_fields_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_closedtrades_fields_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn accepts_supported_strategy_cancel_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_cancel.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.cancel")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_cancel_all_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_cancel_all.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.cancel_all")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn reports_request_strategy_state_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_strategy_state.pine",
        "request.security",
        "same-context request.security",
    );
}

#[test]
fn reports_strategy_state_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_mutation.pine",
        "strategy state variable mutation",
        "read-only",
    );
}

#[test]
fn reports_unsupported_strategy_duplicate_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_duplicate_declaration.pine",
        "E_SCRIPT_DECL_DUPLICATE",
    );
}

#[test]
fn reports_unsupported_strategy_local_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_local_declaration.pine",
        "E_SCRIPT_DECL_LOCATION",
    );
}

#[test]
fn reports_unsupported_drawing_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_drawing.pine",
        "line.set_first_point",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_array_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array.pine",
        "array.new_linefill",
        "array function",
    );
}

#[test]
fn reports_unsupported_array_clear_linefill_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array_clear_linefill.pine",
        "array.new_linefill",
        "array function",
    );
}

#[test]
fn reports_unsupported_array_reverse_linefill_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array_reverse_linefill.pine",
        "array.new_linefill",
        "array function",
    );
}

#[test]
fn reports_unsupported_array_join_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_slice_linefill_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array_slice_linefill.pine",
        "array.new_linefill",
        "array function",
    );
}

#[test]
fn reports_unsupported_array_sort_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_concat_mismatch_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_concat_mismatch.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_import_fixture_missing_host_library() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_import.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_IMPORT_MISSING_LIBRARY"
            || diagnostic.code == "E_IMPORT_ALIAS_REQUIRED"
    }));
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_library_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_library.pine",
        "library",
        "library declarations",
    );
}

#[test]
fn reports_unsupported_export_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_export.pine",
        "export",
        "export declarations",
    );
}

#[test]
fn reports_unsupported_user_type_field_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_user_type.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_FIELD_TYPE" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_user_type_varip_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_type_varip.pine",
        "varip",
        "other value families",
    );
}

#[test]
fn reports_unsupported_user_type_field_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_type_field_mutation.pine",
        "function_side_effect",
        "mutating user-defined type fields inside user-defined functions or methods",
    );
}

#[test]
fn reports_unsupported_user_method_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_user_method.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_RECEIVER_TYPE")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_user_method_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_method_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_non_array_method_fixture_as_receiver_diagnostic() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_non_array_method.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_RECEIVER_TYPE"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_alert_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_dynamic_frequency_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_dynamic_frequency.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_unknown_frequency_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_unknown_frequency.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{close}}`",
    );
}

#[test]
fn reports_unsupported_alertcondition_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alertcondition_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{timenow}}`",
    );
}

#[test]
fn reports_unsupported_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_function_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_unsupported_array_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array_function_side_effect.pine",
        "function_side_effect",
        "array mutation",
    );
}

#[test]
fn reports_unsupported_drawing_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_drawing_function_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_unsupported_alert_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_function_side_effect.pine",
        "function_side_effect",
        "alertcondition",
    );
}

#[test]
fn reports_unsupported_imperative_alert_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_imperative_alert_function_side_effect.pine",
        "function_side_effect",
        "alert",
    );
}

#[test]
fn reports_unsupported_strategy_order_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_order_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_dynamic_history_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_dynamic_history.pine",
        "dynamic_history_offset",
        "integer expression",
    );
}

#[test]
fn reports_unsupported_negative_history_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_negative_history.pine",
        "negative_history_offset",
        "non-negative",
    );
}

#[test]
fn reports_unsupported_recursive_function_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_recursive_function.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_RECURSIVE_FUNCTION"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_supported_block_statement_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_block_statements.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    for feature in ["if", "for"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{} supported features: {:?}",
            path.display(),
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

fn assert_unsupported_fixture(path: &str, feature: &str, reason: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.compatibility.unsupported.iter().any(
            |unsupported| unsupported.feature == feature && unsupported.reason.contains(reason)
        ),
        "{} unsupported features: {:?}",
        path.display(),
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

fn assert_diagnostic_fixture(path: &str, code: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

fn assert_diagnostic_messages(path: &str, messages: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    for message in messages {
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(message)),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
    }
    assert!(analysis.hir.is_none());
}

fn assert_strategy_unsupported_fixture(path: &str, features: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    for feature in features {
        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|unsupported| unsupported.feature == *feature
                    && unsupported.reason.contains("broker emulation")),
            "{} unsupported features: {:?}",
            path.display(),
            analysis.compatibility.unsupported
        );
    }
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "E_UNKNOWN_FUNCTION"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

fn assert_strategy_state_mode_fixture(path: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);
    let variables = [
        "strategy.position_size",
        "strategy.position_avg_price",
        "strategy.openprofit",
        "strategy.netprofit",
        "strategy.equity",
        "strategy.closedtrades",
        "strategy.wintrades",
        "strategy.losstrades",
        "strategy.eventrades",
        "strategy.opentrades",
    ];

    for variable in variables {
        assert!(
            analysis.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "E_STRATEGY_MODE" && diagnostic.message.contains(variable)
            }),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
    }
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_none());
}

fn assert_strategy_state_supported_fixture(path: &str, variables: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    for variable in variables {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == *variable),
            "{} supported features: {:?}",
            path.display(),
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

fn assert_strategy_state_unsupported_fixture(path: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);
    let variables = ["strategy.buy_and_hold_return_percent"];

    for variable in variables {
        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|unsupported| {
                    unsupported.feature == variable
                        && unsupported.reason.contains("broker emulation")
                }),
            "{} unsupported features: {:?}",
            path.display(),
            analysis.compatibility.unsupported
        );
    }
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "E_UNKNOWN_SYMBOL"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}
