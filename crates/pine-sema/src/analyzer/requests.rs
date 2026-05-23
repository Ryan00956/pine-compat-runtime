use crate::prelude::*;

const REQUEST_SECURITY_UNSUPPORTED_REASON: &str = "only same-context request.security(syminfo.tickerid, timeframe.period, expression) and provider-backed same-timeframe direct OHLCV expressions are supported; multi-timeframe, optional parameters, and side-effecting requested expressions are not implemented";

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
                    "multi-symbol and multi-timeframe data requests are not supported in Phase 1",
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
        let current_timeframe = args
            .get(1)
            .is_some_and(|arg| expr_name(&arg.value).as_deref() == Some("timeframe.period"));
        if !current_timeframe {
            unsupported = true;
        }

        let expression_type = arg_types.get(2).copied().flatten();
        if expression_type.is_none_or(|pine_type| !is_request_scalar_type(pine_type)) {
            unsupported = true;
        }
        let pure_scalar_expression = args
            .get(2)
            .is_some_and(|arg| request_expression_is_pure_scalar(&arg.value));
        if !pure_scalar_expression {
            unsupported = true;
        }
        if provider_symbol
            && args
                .get(2)
                .is_none_or(|arg| !request_expression_is_direct_source(&arg.value))
        {
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

fn request_expression_is_direct_source(expr: &Expr) -> bool {
    expr_name(expr).is_some_and(|name| {
        matches!(
            name.as_str(),
            "open" | "high" | "low" | "close" | "volume" | "time"
        )
    })
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

fn request_expression_is_pure_scalar(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => true,
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
            matches!(name.as_str(), "na" | "nz")
                && args
                    .iter()
                    .all(|arg| arg.name.is_none() && request_expression_is_pure_scalar(&arg.value))
        }
        ExprKind::Switch { selector, arms } => {
            selector
                .as_deref()
                .is_none_or(request_expression_is_pure_scalar)
                && arms.iter().all(|arm| {
                    arm.condition
                        .as_ref()
                        .is_none_or(request_expression_is_pure_scalar)
                        && request_expression_is_pure_scalar(&arm.result)
                })
        }
        ExprKind::Tuple(_) | ExprKind::For { .. } => false,
    }
}
