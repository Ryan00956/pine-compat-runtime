const STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine",
];

const STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES: &[&str] =
    &["tests/fixtures/sema/unsupported_strategy_declaration_properties.pine"];

const STRATEGY_RISK_BOUNDARY_FIXTURES: &[&str] =
    &["tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine"];

const STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_entry_short.pine",
    "tests/fixtures/runtime/strategy_entry_short_reverses_long.pine",
    "tests/fixtures/runtime/strategy_entry_long_reverses_short.pine",
    "tests/fixtures/runtime/strategy_entry_limit_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_limit_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_named_const_short_direction.pine",
    "tests/fixtures/sema/supported_strategy_entry_limit_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_stop_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_stop_limit_short.pine",
];

const STRATEGY_EXIT_SHORT_POSITIVE_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_exit_stop_short.pine",
    "tests/fixtures/runtime/strategy_exit_limit_short.pine",
    "tests/fixtures/runtime/strategy_exit_profit_short.pine",
    "tests/fixtures/runtime/strategy_exit_loss_short.pine",
    "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_stop_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_limit_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_trail_price_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_trail_points_fill_short.pine",
];

const STRATEGY_MAX_CONTRACTS_HELD_SHORT_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_position_state.pine",
    "tests/fixtures/runtime/strategy_entry_short.pine",
];

const STRATEGY_ORDER_SHORT_REVERSAL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_order_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_short_flat_noop.pine",
    "tests/fixtures/runtime/strategy_order_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short.pine",
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
];

const STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_order_market_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_short_flat_noop.pine",
    "tests/fixtures/runtime/strategy_order_metadata.pine",
    "tests/fixtures/sema/supported_strategy_order.pine",
    "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
];

const STRATEGY_CANCEL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_cancel_entry.pine",
    "tests/fixtures/runtime/strategy_cancel_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_noop.pine",
    "tests/fixtures/sema/supported_strategy_cancel.pine",
];

const STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_all_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_all_noop.pine",
    "tests/fixtures/sema/supported_strategy_cancel_all.pine",
];

const STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_close_all.pine",
    "tests/fixtures/runtime/strategy_close_all_short.pine",
    "tests/fixtures/runtime/strategy_close_all_exit.pine",
    "tests/fixtures/runtime/strategy_close_metadata.pine",
    "tests/fixtures/sema/supported_strategy_close_all.pine",
    "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
    "tests/fixtures/sema/unsupported_strategy_close_all_indicator.pine",
];

const STRATEGY_CLOSE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_close.pine",
    "tests/fixtures/runtime/strategy_close_short.pine",
    "tests/fixtures/runtime/strategy_close_noop.pine",
    "tests/fixtures/runtime/strategy_close_exit.pine",
    "tests/fixtures/runtime/strategy_close_qty_partial.pine",
    "tests/fixtures/runtime/strategy_close_qty_full_clamp.pine",
    "tests/fixtures/runtime/strategy_close_qty_percent_precedence.pine",
    "tests/fixtures/sema/supported_strategy_close.pine",
    "tests/fixtures/sema/supported_strategy_close_qty.pine",
    "tests/fixtures/sema/supported_strategy_close_qty_percent.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
    "tests/fixtures/sema/unsupported_strategy_close_indicator.pine",
];

const STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_margin_declaration.pine",
    "tests/fixtures/sema/unsupported_strategy_margin_declaration.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_long.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_short.pine",
    "tests/fixtures/runtime/strategy_margin_entry_affordability_long.pine",
    "tests/fixtures/runtime/strategy_margin_entry_affordability_short.pine",
    "tests/fixtures/runtime/strategy_margin_call_long.pine",
    "tests/fixtures/runtime/strategy_margin_call_short.pine",
];

const STRATEGY_MARGIN_STATE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_margin_capital_held_long.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_short.pine",
    "tests/fixtures/runtime/strategy_margin_call_long.pine",
    "tests/fixtures/runtime/strategy_margin_call_short.pine",
];

const STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_close_entries_rule_fifo.pine",
    "tests/fixtures/sema/supported_strategy_close_entries_rule_any.pine",
    "tests/fixtures/sema/unsupported_strategy_close_entries_rule_unknown.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_fifo.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_fifo_close_all.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_close.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_close_short.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry_short.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial_short.pine",
];

pub(super) fn validate_entry(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    validate_strategy_oca_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_execution_timing_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_risk_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_order_supported_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_all_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_cancel_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_short_reversal_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_max_contracts_held_short_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_margin_account_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_entries_rule_boundary_fixture_paths(line_number, feature, fixtures)?;

    Ok(())
}

fn validate_strategy_oca_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy constants" => &[
            "tests/fixtures/runtime/strategy_constants.pine",
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine",
        ],
        "strategy.order" => &["tests/fixtures/sema/unsupported_strategy_orders.pine"],
        "strategy.*" => STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES,
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while OCA constants remain pure strings and custom OCA order behavior remains unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_execution_timing_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !matches!(feature, "strategy" | "strategy.*") {
        return Ok(());
    }

    for fixture in STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy execution timing and recalculation settings remain unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.*" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk broker directives remain unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_order_supported_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.order" {
        return Ok(());
    }

    for fixture in STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.order support remains limited to the fixture-backed explicit-qty market/limit/stop/stop-limit long, limit/stop/stop-limit-short add-or-increase, and reduce-only market-short subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.close" {
        return Ok(());
    }

    for fixture in STRATEGY_CLOSE_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.close support remains limited to fixture-backed full closes, fixed/percent partial closes, flat/wrong-entry no-ops, and full-close pending-exit cleanup"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_all_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.close_all" {
        return Ok(());
    }

    for fixture in STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.close_all support remains limited to fixture-backed long-position closes, flat/repeated no-ops, metadata, and indicator/immediately rejections"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_cancel_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy.cancel" => STRATEGY_CANCEL_BOUNDARY_FIXTURES,
        "strategy.cancel_all" => STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES,
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy cancellation support remains limited to fixture-backed pending entry and pending exit cancellation without public cancellation records"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_short_reversal_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: Vec<&str> = match feature {
        "strategy.entry" => STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES.to_vec(),
        "strategy.order" => STRATEGY_ORDER_SHORT_REVERSAL_BOUNDARY_FIXTURES.to_vec(),
        "strategy.exit" => STRATEGY_EXIT_SHORT_POSITIVE_FIXTURES.to_vec(),
        "strategy.*" => vec!["tests/fixtures/sema/unsupported_strategy_orders.pine"],
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(&fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while short exposure and automatic reversal remain unsupported outside the fixture-backed market, limit, stop, and stop-limit strategy.entry short, limit/stop/stop-limit strategy.order short, short stop/limit/profit/loss/bracket/trailing strategy.exit, and reduce-only market strategy.order subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_max_contracts_held_short_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.max_contracts_held_short" {
        return Ok(());
    }

    for fixture in STRATEGY_MAX_CONTRACTS_HELD_SHORT_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.max_contracts_held_short remains 0.0 in the long-only subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_margin_account_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy" => STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES,
        "strategy.opentrades.*" | "strategy.opentrades.capital_held" => {
            STRATEGY_MARGIN_STATE_BOUNDARY_FIXTURES
        }
        "strategy.margin_liquidation_price" => &[
            "tests/fixtures/runtime/strategy_margin_call_long.pine",
            "tests/fixtures/runtime/strategy_margin_call_short.pine",
        ],
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy margin/account support remains limited to the fixture-backed long-only plus short capital-held/affordability/forced-liquidation/liquidation-price subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_entries_rule_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy" => STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES,
        "strategy.*" => {
            &["tests/fixtures/sema/unsupported_strategy_close_entries_rule_unknown.pine"]
        }
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while close_entries_rule support remains limited to FIFO plus fixture-backed id-specific long and short ANY close/exit allocation"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::try_conformance_entries_from_tsv;
    use super::*;

    #[test]
    fn rejects_strategy_star_row_without_oca_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine";
        let mut fixtures = vec!["tests/fixtures/sema/unsupported_strategy.pine"];
        fixtures.extend(
            STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.*\tunsupported\tcustom OCA parameters and strategy.order OCA behavior remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy OCA boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_execution_timing_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        ];
        fixtures.extend(
            STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tprocess_orders_on_close, calc_on_order_fills, calc_on_every_tick, use_bar_magnifier, and fill_orders_on_standard_ohlc remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy execution timing boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_star_row_without_risk_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine";
        let mut fixtures = vec![
            "tests/fixtures/sema/unsupported_strategy.pine",
            "tests/fixtures/sema/unsupported_strategy_orders.pine",
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(
            STRATEGY_RISK_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.*\tunsupported\tstrategy.risk broker directives remain unsupported until broker risk state is implemented\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy risk boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_order_row_without_supported_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_order_stop_limit_long.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_order_market_long.pine"];
        fixtures.extend(
            STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.order\tpartial\texplicit-qty market, limit, stop, stop-limit long orders plus reduce-only market short orders are supported while omitted qty, short exposure, reversals, short price-based orders, OCA, and unsupported metadata types remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.order supported boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_entry.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel\tpartial\tstrategy.cancel cancels matching pending entry and pending exit ids while filled or unknown ids are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_noop.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_entry.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel\tpartial\tstrategy.cancel cancels matching pending entry and pending exit ids while unknown, already-filled, or already-cancelled ids are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_all_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_all_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel_all\tpartial\tstrategy.cancel_all cancels supported pending entries and pending exits while empty pending books are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel_all pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_all_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_all_noop.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel_all\tpartial\tstrategy.cancel_all cancels supported pending entries and pending exits while empty pending books are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel_all no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_all_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_all.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close_metadata.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close_all\tpartial\tstrategy.close_all closes current long positions while flat or already-closed calls are no-op and immediately remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close_all no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close\tpartial\tstrategy.close closes current long entries, supports partial quantities, treats wrong-entry ids as no-op, and clears matching pending exits only when the close fully flattens the entry\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_all_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_all_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close_all.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close_all\tpartial\tstrategy.close_all closes current long positions and clears pending exits for that entry while flat or already-closed calls are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close_all pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_margin_account_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_margin_call_long.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(
            STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tmargin_long and margin_short declaration parsing is supported while runtime margin behavior remains long-only and short margin remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy margin/account boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_close_entries_rule_boundary_fixture_set() {
        let missing =
            "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tclose_entries_rule FIFO and fixture-backed id-specific ANY close and exit allocation are supported while broader ANY behavior remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy close_entries_rule boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_contracts_held_short_row_without_long_only_zero_fixture() {
        let missing = "tests/fixtures/runtime/strategy_position_state.pine";
        let tsv = "feature\tstatus\tnotes\tfixtures\nstrategy.max_contracts_held_short\tpartial\tremains 0.0 because short entries are unsupported\ttests/fixtures/sema/supported_strategy_position_state.pine;tests/fixtures/sema/unsupported_strategy_state_indicator.pine\n";
        let error = try_conformance_entries_from_tsv(tsv)
            .expect_err("missing long-only max_contracts_held_short fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_entry_row_without_market_short_runtime_fixture() {
        let missing = "tests/fixtures/runtime/strategy_entry_short.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_entry.pine",
            "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        ];
        fixtures.extend(
            STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.entry\tpartial\tmarket strategy.short entries are fixture-backed while reversal remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing market short strategy.entry runtime fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_entry_row_without_stop_limit_short_runtime_fixture() {
        let missing = "tests/fixtures/runtime/strategy_entry_stop_limit_short.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_entry.pine",
            "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        ];
        fixtures.extend(
            STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.entry\tpartial\tstop-limit strategy.short entries are fixture-backed\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing stop-limit short strategy.entry runtime fixture should fail");

        assert!(error.contains(missing));
    }
}
