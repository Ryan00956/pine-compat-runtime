use crate::prelude::*;

const REQUEST_SECURITY_UNSUPPORTED_REASON: &str = "only same-context request.security(syminfo.tickerid, timeframe.period, expression) scalar expressions and selected tuple expressions, plus provider-backed same-or-higher-timeframe scalar expressions, are supported; optional parameters, lower-timeframe requests, and side-effecting requested expressions are not implemented";

impl Analyzer {
    pub(crate) fn analyze_request_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
    ) -> Option<PineType> {
        match name {
            "request.security" => self.analyze_request_security(span, args),
            _ => {
                self.unsupported(
                    name,
                    "this request function is outside the supported request.security subset",
                    span,
                );
                None
            }
        }
    }

    fn analyze_request_security(&mut self, span: Span, args: &[CallArg]) -> Option<PineType> {
        let signature = pine_builtins::get_phase_1_builtin("request.security")?;
        self.compatibility.supported.push(FeatureUse {
            feature: "request.security".to_owned(),
            span,
        });

        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();
        self.validate_call_args(signature, args, &arg_types);

        let mut unsupported = false;
        if args.len() != 3 {
            unsupported = true;
        }
        if args.iter().any(|arg| arg.name.is_some()) {
            unsupported = true;
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                "`request.security` currently supports positional arguments only",
                span,
            ));
        }

        let same_context_symbol = args
            .first()
            .is_some_and(|arg| expr_name(&arg.value).as_deref() == Some("syminfo.tickerid"));
        let provider_symbol = args.first().is_some_and(|arg| {
            matches!(&arg.value.kind, ExprKind::Literal(Literal::String(value)) if !value.trim().is_empty())
        });
        if !same_context_symbol && !provider_symbol {
            unsupported = true;
        }
        let same_chart_timeframe = args
            .get(1)
            .is_some_and(|arg| expr_name(&arg.value).as_deref() == Some("timeframe.period"));
        let literal_timeframe = args.get(1).is_some_and(|arg| {
            matches!(&arg.value.kind, ExprKind::Literal(Literal::String(value)) if !value.trim().is_empty())
        });
        if !same_chart_timeframe && !literal_timeframe {
            unsupported = true;
        }

        let expression_type = arg_types.get(2).copied().flatten();
        let same_context_request = same_context_symbol && same_chart_timeframe;
        if expression_type.is_none_or(|pine_type| {
            if same_context_request {
                !is_request_same_context_type(pine_type)
            } else {
                !is_request_scalar_type(pine_type)
            }
        }) {
            unsupported = true;
        }
        let supported_expression = args.get(2).is_some_and(|arg| {
            if provider_symbol || literal_timeframe {
                request_expression_is_provider_scalar(&arg.value)
            } else {
                request_expression_is_same_context_value(&arg.value)
            }
        });
        if !supported_expression {
            unsupported = true;
        }
        if unsupported {
            self.unsupported(
                "request.security",
                REQUEST_SECURITY_UNSUPPORTED_REASON,
                span,
            );
            return expression_type.map(series_request_type);
        }

        expression_type.map(series_request_type)
    }
}

fn series_request_type(pine_type: PineType) -> PineType {
    PineType::new(Qualifier::Series, pine_type.kind)
}

fn is_request_scalar_type(pine_type: PineType) -> bool {
    matches!(
        pine_type.kind,
        ValueKind::Int
            | ValueKind::Float
            | ValueKind::Bool
            | ValueKind::String
            | ValueKind::Color
            | ValueKind::Na
    )
}

fn is_request_same_context_type(pine_type: PineType) -> bool {
    is_request_scalar_type(pine_type) || pine_type.kind == ValueKind::Tuple
}

fn request_expression_is_same_context_value(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Tuple(items) => items.iter().all(request_expression_is_same_context_value),
        _ => request_expression_is_pure_scalar(expr),
    }
}

fn request_expression_is_pure_scalar(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) => true,
        ExprKind::QualifiedName(_) => expr_name(expr)
            .as_deref()
            .is_none_or(|name| !is_strategy_state_variable(name)),
        ExprKind::Unary { expr, .. } => request_expression_is_pure_scalar(expr),
        ExprKind::Binary { left, right, .. } => {
            request_expression_is_pure_scalar(left) && request_expression_is_pure_scalar(right)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            request_expression_is_pure_scalar(condition)
                && request_expression_is_pure_scalar(then_expr)
                && request_expression_is_pure_scalar(else_expr)
        }
        ExprKind::History { expr, offset } => {
            request_expression_is_pure_scalar(expr) && request_expression_is_pure_scalar(offset)
        }
        ExprKind::Call { callee, args } => {
            let Some(name) = expr_name(callee) else {
                return false;
            };
            (request_scalar_call_is_supported(name.as_str())
                || request_tuple_call_is_supported(name.as_str()))
                && args.iter().all(|arg| {
                    arg.name.is_none() && request_expression_is_same_context_value(&arg.value)
                })
        }
        ExprKind::Switch { selector, arms } => {
            selector
                .as_deref()
                .is_none_or(request_expression_is_same_context_value)
                && arms.iter().all(|arm| {
                    arm.condition
                        .as_ref()
                        .is_none_or(request_expression_is_same_context_value)
                        && request_expression_is_same_context_value(&arm.result)
                })
        }
        ExprKind::Tuple(_) | ExprKind::For { .. } => false,
    }
}

fn request_tuple_call_is_supported(name: &str) -> bool {
    matches!(name, "ta.macd")
}

fn request_expression_is_provider_scalar(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) => true,
        ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => expr_name(expr)
            .as_deref()
            .is_some_and(is_request_provider_scalar_name),
        ExprKind::Unary { expr, .. } => request_expression_is_provider_scalar(expr),
        ExprKind::Binary { left, right, .. } => {
            request_expression_is_provider_scalar(left)
                && request_expression_is_provider_scalar(right)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            request_expression_is_provider_scalar(condition)
                && request_expression_is_provider_scalar(then_expr)
                && request_expression_is_provider_scalar(else_expr)
        }
        ExprKind::History { expr, offset } => {
            request_expression_is_provider_scalar(expr)
                && request_expression_is_provider_scalar(offset)
        }
        ExprKind::Call { callee, args } => {
            let Some(name) = expr_name(callee) else {
                return false;
            };
            request_scalar_call_is_supported(name.as_str())
                && args.iter().all(|arg| {
                    arg.name.is_none() && request_expression_is_provider_scalar(&arg.value)
                })
        }
        ExprKind::Switch { selector, arms } => {
            selector
                .as_deref()
                .is_none_or(request_expression_is_provider_scalar)
                && arms.iter().all(|arm| {
                    arm.condition
                        .as_ref()
                        .is_none_or(request_expression_is_provider_scalar)
                        && request_expression_is_provider_scalar(&arm.result)
                })
        }
        ExprKind::Tuple(_) | ExprKind::For { .. } => false,
    }
}

fn is_request_provider_scalar_name(name: &str) -> bool {
    matches!(
        name,
        "open"
            | "high"
            | "low"
            | "close"
            | "volume"
            | "time"
            | "ta.accdist"
            | "ta.iii"
            | "ta.nvi"
            | "ta.obv"
            | "ta.pvi"
            | "ta.pvt"
            | "ta.wvad"
    )
}

fn request_scalar_call_is_supported(name: &str) -> bool {
    matches!(
        name,
        "na" | "nz"
            | "math.abs"
            | "math.max"
            | "math.min"
            | "math.avg"
            | "math.floor"
            | "math.ceil"
            | "math.trunc"
            | "math.sqrt"
            | "math.cbrt"
            | "math.log"
            | "math.log10"
            | "math.exp"
            | "math.acos"
            | "math.asin"
            | "math.atan"
            | "math.sign"
            | "math.todegrees"
            | "math.toradians"
            | "math.sin"
            | "math.cos"
            | "math.tan"
            | "math.pow"
            | "math.hypot"
            | "math.round"
            | "math.round_to_mintick"
            | "math.sum"
            | "ta.cum"
            | "ta.sma"
            | "ta.ema"
            | "ta.dema"
            | "ta.tema"
            | "ta.rma"
            | "ta.rsi"
            | "ta.tsi"
            | "ta.cmo"
            | "ta.cci"
            | "ta.cog"
            | "ta.bop"
            | "ta.ao"
            | "ta.max"
            | "ta.min"
            | "ta.mfi"
            | "ta.stoch"
            | "ta.wpr"
            | "ta.sar"
            | "ta.tr"
            | "ta.atr"
            | "ta.highest"
            | "ta.lowest"
            | "ta.highestbars"
            | "ta.lowestbars"
            | "ta.change"
            | "ta.mom"
            | "ta.roc"
            | "ta.range"
            | "ta.dev"
            | "ta.vwap"
            | "ta.bbw"
            | "ta.kcw"
            | "ta.pivothigh"
            | "ta.pivotlow"
            | "ta.correlation"
            | "ta.covariance"
            | "ta.median"
            | "ta.mode"
            | "ta.percentile_linear_interpolation"
            | "ta.percentile_nearest_rank"
            | "ta.percentrank"
            | "ta.stdev"
            | "ta.variance"
            | "ta.wma"
            | "ta.vwma"
            | "ta.swma"
            | "ta.hma"
            | "ta.alma"
            | "ta.linreg"
            | "ta.rising"
            | "ta.falling"
            | "ta.barssince"
            | "ta.valuewhen"
            | "ta.cross"
            | "ta.crossover"
            | "ta.crossunder"
    )
}
