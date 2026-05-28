use crate::prelude::*;

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
        for (index, arg) in args.iter().enumerate() {
            let is_initial_capital = arg.name.as_deref() == Some("initial_capital")
                || (arg.name.is_none() && index == 4);
            if !is_initial_capital {
                continue;
            }

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
    }

    pub(crate) fn validate_strategy_order_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
    ) {
        if !matches!(name, "strategy.entry" | "strategy.close") {
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
}
