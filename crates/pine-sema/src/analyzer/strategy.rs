use crate::prelude::*;

const STRATEGY_FIXED_DEFAULT_QTY_TYPE: &str = "strategy.fixed";

const STRATEGY_STATE_VARIABLES: &[&str] = &[
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StrategyExitArgFamily {
    Identity,
    DownsidePriceTrigger,
    DownsideTickTrigger,
    UpsidePriceTrigger,
    UpsideTickTrigger,
    TrailingActivation,
    TrailingOffset,
    Quantity,
    PercentQuantity,
    UnsupportedOption,
}

fn strategy_exit_arg_family(name: &str) -> Option<StrategyExitArgFamily> {
    match name {
        "id" | "from_entry" => Some(StrategyExitArgFamily::Identity),
        "stop" => Some(StrategyExitArgFamily::DownsidePriceTrigger),
        "loss" => Some(StrategyExitArgFamily::DownsideTickTrigger),
        "limit" => Some(StrategyExitArgFamily::UpsidePriceTrigger),
        "profit" => Some(StrategyExitArgFamily::UpsideTickTrigger),
        "trail_price" | "trail_points" => Some(StrategyExitArgFamily::TrailingActivation),
        "trail_offset" => Some(StrategyExitArgFamily::TrailingOffset),
        "qty" => Some(StrategyExitArgFamily::Quantity),
        "qty_percent" => Some(StrategyExitArgFamily::PercentQuantity),
        "oca_name" | "comment" | "alert_message" => Some(StrategyExitArgFamily::UnsupportedOption),
        _ => None,
    }
}

pub(crate) fn is_strategy_state_variable(name: &str) -> bool {
    STRATEGY_STATE_VARIABLES.contains(&name)
}

pub(crate) fn is_supported_strategy_state_variable(name: &str) -> bool {
    STRATEGY_STATE_VARIABLES.contains(&name)
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

        if let (_, _, Some(_), Some(qty)) = (
            fixed_default_qty_type,
            default_qty_type_arg,
            default_qty_value_arg,
            default_qty_value,
        ) {
            self.strategy_settings.default_qty = Some(pine_ir::StrategyDefaultQuantity::Fixed(qty));
        }
    }

    pub(crate) fn validate_strategy_order_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
    ) {
        if !matches!(
            name,
            "strategy.entry" | "strategy.close" | "strategy.close_all" | "strategy.exit"
        ) {
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
        if !is_strategy_state_variable(name) {
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

        if is_supported_strategy_state_variable(name) {
            return false;
        }

        self.unsupported(name, STRATEGY_STATE_UNSUPPORTED_REASON, span);
        true
    }

    pub(crate) fn validate_strategy_entry_args(&mut self, args: &[CallArg]) {
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
    }

    pub(crate) fn validate_strategy_exit_args(&mut self, args: &[CallArg]) {
        let mut has_stop = false;
        let mut has_limit = false;
        let mut has_profit = false;
        let mut has_loss = false;
        let mut has_trail_price = false;
        let mut has_trail_points = false;
        let mut has_trail_offset = false;
        let mut has_qty = false;
        let mut has_qty_percent = false;
        let mut has_unsupported_arg = false;
        for (index, arg) in args.iter().enumerate() {
            let Some(name) = arg
                .name
                .as_deref()
                .or_else(|| ["id", "from_entry", "stop", "limit"].get(index).copied())
            else {
                if arg.name.is_none() {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        "`strategy.exit` profit and loss arguments must be named arguments",
                        arg.span,
                    ));
                }
                continue;
            };
            let Some(family) = strategy_exit_arg_family(name) else {
                continue;
            };
            match family {
                StrategyExitArgFamily::Identity => {}
                StrategyExitArgFamily::DownsidePriceTrigger => has_stop = true,
                StrategyExitArgFamily::DownsideTickTrigger => has_loss = true,
                StrategyExitArgFamily::UpsidePriceTrigger => has_limit = true,
                StrategyExitArgFamily::UpsideTickTrigger => has_profit = true,
                StrategyExitArgFamily::TrailingActivation => {
                    if name == "trail_price" {
                        has_trail_price = true;
                    } else {
                        has_trail_points = true;
                    }
                }
                StrategyExitArgFamily::TrailingOffset => has_trail_offset = true,
                StrategyExitArgFamily::Quantity => has_qty = true,
                StrategyExitArgFamily::PercentQuantity => has_qty_percent = true,
                StrategyExitArgFamily::UnsupportedOption => {
                    has_unsupported_arg = true;
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        format!(
                            "`strategy.exit` argument `{name}` is not supported in the current strategy subset"
                        ),
                        arg.span,
                    ))
                }
            }
        }
        if has_qty && has_qty_percent {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                "`strategy.exit` cannot combine `qty` and `qty_percent` in the current strategy subset",
                args.iter()
                    .find(|arg| arg.name.as_deref() == Some("qty_percent"))
                    .map_or(Span::default(), |arg| arg.span),
            ));
        }
        let trigger_count = usize::from(has_stop)
            + usize::from(has_limit)
            + usize::from(has_profit)
            + usize::from(has_loss);
        let trailing_activation_count =
            usize::from(has_trail_price) + usize::from(has_trail_points);
        let has_trailing_args = trailing_activation_count > 0 || has_trail_offset;
        if has_trailing_args {
            if trigger_count > 0 {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    "`strategy.exit` cannot combine trailing exits with fixed stop, limit, profit, or loss triggers in the current strategy subset",
                    args.iter()
                        .find(|arg| {
                            matches!(
                                arg.name.as_deref(),
                                Some(
                                    "stop"
                                        | "limit"
                                        | "profit"
                                        | "loss"
                                        | "trail_price"
                                        | "trail_points"
                                        | "trail_offset"
                                )
                            )
                        })
                        .map_or(Span::default(), |arg| arg.span),
                ));
            }
            if trailing_activation_count != 1 || !has_trail_offset {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    "`strategy.exit` trailing exits require exactly one of `trail_price` or `trail_points` plus `trail_offset`",
                    args.iter()
                        .find(|arg| {
                            matches!(
                                arg.name.as_deref(),
                                Some("trail_price" | "trail_points" | "trail_offset")
                            )
                        })
                        .map_or(Span::default(), |arg| arg.span),
                ));
            }
            return;
        }
        let downside_trigger_count = usize::from(has_stop) + usize::from(has_loss);
        let upside_trigger_count = usize::from(has_limit) + usize::from(has_profit);
        let supported_trigger_shape = trigger_count <= 1
            || (trigger_count == 2 && downside_trigger_count == 1 && upside_trigger_count == 1);
        if !supported_trigger_shape {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                "`strategy.exit` combined trigger families are not supported in the current strategy subset",
                args.iter()
                    .find(|arg| matches!(arg.name.as_deref(), Some("limit" | "profit" | "loss")))
                    .or_else(|| args.get(3))
                    .map_or(Span::default(), |arg| arg.span),
            ));
        } else if trigger_count == 0 && args.len() >= 2 && !has_unsupported_arg {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                "`strategy.exit` requires one of `stop`, `limit`, `profit`, or `loss`",
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
        }
    }
}
