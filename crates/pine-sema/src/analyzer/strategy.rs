use crate::prelude::*;

const STRATEGY_FIXED_DEFAULT_QTY_TYPE: &str = "strategy.fixed";

const PHASE_L_STRATEGY_STATE_VARIABLES: &[&str] = &[
    "strategy.position_size",
    "strategy.position_avg_price",
    "strategy.openprofit",
    "strategy.netprofit",
    "strategy.equity",
];

pub(crate) fn is_phase_l_strategy_state_variable(name: &str) -> bool {
    PHASE_L_STRATEGY_STATE_VARIABLES.contains(&name)
}

pub(crate) fn is_phase_l_supported_strategy_state_variable(name: &str) -> bool {
    PHASE_L_STRATEGY_STATE_VARIABLES.contains(&name)
}

impl Analyzer {
    pub(crate) fn validate_script_declaration_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
    ) {
        let Some(mode) = (match name {
            "indicator" => Some(ScriptMode::Indicator),
            "strategy" => Some(ScriptMode::Strategy),
            _ => None,
        }) else {
            return;
        };

        if self.block_depth > 0 || self.function_depth > 0 {
            self.diagnostics.push(Diagnostic::error(
                "E_SCRIPT_DECL_LOCATION",
                format!("`{name}` declarations must be top-level"),
                span,
            ));
            return;
        }

        if let Some((existing_mode, _)) = self.script_declaration {
            self.diagnostics.push(Diagnostic::error(
                "E_SCRIPT_DECL_DUPLICATE",
                format!(
                    "script already has a {:?} declaration; only one indicator(...) or strategy(...) declaration is allowed",
                    existing_mode
                ),
                span,
            ));
            return;
        }

        self.script_declaration = Some((mode, span));
        if mode == ScriptMode::Strategy {
            self.validate_strategy_declaration_args(args);
        }
    }

    pub(crate) fn validate_strategy_declaration_args(&mut self, args: &[CallArg]) {
        let mut default_qty_type_arg = None;
        let mut default_qty_value_arg = None;
        let mut fixed_default_qty_type = false;
        let mut default_qty_value = None;

        for (index, arg) in args.iter().enumerate() {
            let Some(name) = arg.name.as_deref().or_else(|| {
                [
                    "title",
                    "shorttitle",
                    "overlay",
                    "max_bars_back",
                    "initial_capital",
                    "default_qty_type",
                    "default_qty_value",
                ]
                .get(index)
                .copied()
            }) else {
                continue;
            };

            match name {
                "initial_capital" => {
                    let Some(initial_capital) = const_numeric_value(&arg.value) else {
                        continue;
                    };
                    if !initial_capital.is_finite() || initial_capital <= 0.0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E_CALL_ARG_VALUE",
                            "`strategy` argument `initial_capital` must be positive",
                            arg.span,
                        ));
                        continue;
                    }
                    self.strategy_settings.initial_capital = initial_capital;
                }
                "default_qty_type" => {
                    default_qty_type_arg = Some(arg);
                    let Some(default_qty_type) = const_string_value(&arg.value) else {
                        continue;
                    };
                    if default_qty_type != STRATEGY_FIXED_DEFAULT_QTY_TYPE {
                        self.diagnostics.push(Diagnostic::error(
                            "E_CALL_ARG_VALUE",
                            "`strategy` argument `default_qty_type` only supports strategy.fixed",
                            arg.span,
                        ));
                        continue;
                    }
                    fixed_default_qty_type = true;
                }
                "default_qty_value" => {
                    default_qty_value_arg = Some(arg);
                    let Some(qty) = const_numeric_value(&arg.value) else {
                        continue;
                    };
                    if !qty.is_finite() || qty <= 0.0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E_CALL_ARG_VALUE",
                            "`strategy` argument `default_qty_value` must be positive",
                            arg.span,
                        ));
                        continue;
                    }
                    default_qty_value = Some(qty);
                }
                _ => {}
            }
        }

        match (
            fixed_default_qty_type,
            default_qty_type_arg,
            default_qty_value_arg,
            default_qty_value,
        ) {
            (true, _, Some(_), Some(qty)) => {
                self.strategy_settings.default_qty =
                    Some(pine_ir::StrategyDefaultQuantity::Fixed(qty));
            }
            (true, Some(arg), None, _) => {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    "`strategy` argument `default_qty_value` is required when default_qty_type=strategy.fixed",
                    arg.span,
                ));
            }
            (false, None, Some(arg), Some(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    "`strategy` argument `default_qty_value` requires default_qty_type=strategy.fixed",
                    arg.span,
                ));
            }
            _ => {}
        }
    }

    pub(crate) fn validate_strategy_order_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
    ) {
        if !matches!(name, "strategy.entry" | "strategy.close" | "strategy.exit") {
            return;
        }

        if !matches!(self.script_declaration, Some((ScriptMode::Strategy, _))) {
            self.diagnostics.push(Diagnostic::error(
                "E_STRATEGY_MODE",
                format!("`{name}` is only supported in scripts declared with strategy(...)"),
                span,
            ));
        }

        if name == "strategy.entry" {
            self.validate_strategy_entry_args(args);
        } else if name == "strategy.exit" {
            self.validate_strategy_exit_args(args);
        }
    }

    pub(crate) fn validate_strategy_state_variable(&mut self, name: &str, span: Span) -> bool {
        if !is_phase_l_strategy_state_variable(name) {
            return false;
        }

        if !matches!(self.script_declaration, Some((ScriptMode::Strategy, _))) {
            self.diagnostics.push(Diagnostic::error(
                "E_STRATEGY_MODE",
                format!("`{name}` is only supported in scripts declared with strategy(...)"),
                span,
            ));
            return true;
        }

        if is_phase_l_supported_strategy_state_variable(name) {
            return false;
        }

        self.unsupported(name, STRATEGY_STATE_UNSUPPORTED_REASON, span);
        true
    }

    pub(crate) fn validate_strategy_entry_args(&mut self, args: &[CallArg]) {
        let mut has_qty = false;
        for (index, arg) in args.iter().enumerate() {
            let Some(name) = arg
                .name
                .as_deref()
                .or_else(|| ["id", "direction", "qty"].get(index).copied())
            else {
                continue;
            };
            match name {
                "direction" => {
                    let Some(direction) = const_string_value(&arg.value) else {
                        continue;
                    };
                    if direction != "strategy.long" {
                        self.diagnostics.push(Diagnostic::error(
                            "E_CALL_ARG_VALUE",
                            "`strategy.entry` argument `direction` only supports strategy.long",
                            arg.span,
                        ));
                    }
                }
                "qty" => {
                    has_qty = true;
                    if let Some(qty) = const_numeric_value(&arg.value)
                        && qty <= 0.0
                    {
                        self.diagnostics.push(Diagnostic::error(
                            "E_CALL_ARG_VALUE",
                            "`strategy.entry` argument `qty` must be positive",
                            arg.span,
                        ));
                    }
                }
                _ => {}
            }
        }

        if !has_qty && self.strategy_settings.default_qty.is_none() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                "`strategy.entry` requires `qty` unless strategy default_qty_type=strategy.fixed and default_qty_value are configured",
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
        }
    }

    pub(crate) fn validate_strategy_exit_args(&mut self, args: &[CallArg]) {
        let mut has_stop = false;
        let mut has_limit = false;
        for (index, arg) in args.iter().enumerate() {
            let Some(name) = arg
                .name
                .as_deref()
                .or_else(|| ["id", "from_entry", "stop", "limit"].get(index).copied())
            else {
                continue;
            };
            match name {
                "id" | "from_entry" => {}
                "stop" => has_stop = true,
                "limit" => has_limit = true,
                "qty" | "qty_percent" | "profit" | "loss" | "trail_price" | "trail_points"
                | "trail_offset" | "oca_name" | "comment" | "alert_message" => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        format!(
                            "`strategy.exit` argument `{name}` is not supported in Phase M Slice 1"
                        ),
                        arg.span,
                    ))
                }
                _ => {}
            }
        }
        if has_stop && has_limit {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                "`strategy.exit` combined stop and limit exits are not supported in Phase M Slice 4",
                args.iter()
                    .find(|arg| arg.name.as_deref() == Some("limit"))
                    .or_else(|| args.get(3))
                    .map_or(Span::default(), |arg| arg.span),
            ));
        } else if !has_stop && !has_limit && args.len() >= 2 {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                "`strategy.exit` requires either `stop` or `limit`",
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
        }
    }
}
