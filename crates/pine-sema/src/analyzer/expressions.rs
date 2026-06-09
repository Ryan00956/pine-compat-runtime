use crate::prelude::*;

impl Analyzer {
    pub(crate) fn analyze_expr(&mut self, expr: &Expr) -> Option<PineType> {
        if !self.enter_expr_analysis(expr.span) {
            return None;
        }

        let result = self.analyze_expr_inner(expr);
        self.exit_expr_analysis();
        result
    }

    fn analyze_expr_inner(&mut self, expr: &Expr) -> Option<PineType> {
        match &expr.kind {
            ExprKind::Literal(literal) => {
                if matches!(literal, Literal::ColorHex(_)) {
                    self.compatibility.supported.push(FeatureUse {
                        feature: "hex color literal".to_owned(),
                        span: expr.span,
                    });
                }
                Some(literal_type(literal))
            }
            ExprKind::Identifier(name) => {
                self.check_feature_expr(expr);
                self.resolve_symbol(name, expr.span)
            }
            ExprKind::QualifiedName(parts) => {
                if let Some(field_type) = self.resolve_user_type_field_access(parts, expr.span) {
                    return Some(field_type);
                }
                let name = expr_name(expr)?;
                self.resolve_qualified_value(&name, expr.span)
            }
            ExprKind::Unary { op, expr } => {
                let expr_type = self.analyze_expr(expr)?;
                self.infer_unary(*op, expr_type, expr.span)
            }
            ExprKind::Binary { op, left, right } => {
                let left_type = self.analyze_expr(left);
                let right_type = self.analyze_expr(right);
                match (left_type, right_type) {
                    (Some(left_type), Some(right_type)) => {
                        self.infer_binary(*op, left_type, right_type, expr.span)
                    }
                    _ => None,
                }
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    self.expect_bool(condition_type, condition.span);
                }
                let then_type = self.analyze_expr(then_expr);
                let else_type = self.analyze_expr(else_expr);
                match (condition_type, then_type, else_type) {
                    (Some(condition_type), Some(then_type), Some(else_type)) => {
                        let pine_type = self.merge_branch_types(
                            condition_type,
                            then_type,
                            else_type,
                            expr.span,
                        )?;
                        if pine_type.kind == ValueKind::UserType
                            && !self.mark_ternary_user_type(expr.span, then_expr, else_expr)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "E_BRANCH_TYPE",
                                "ternary user-defined type branches must resolve to the same local UDT",
                                expr.span,
                            ));
                            return None;
                        }
                        Some(pine_type)
                    }
                    _ => None,
                }
            }
            ExprKind::Switch { selector, arms } => {
                self.analyze_switch_expr(selector.as_deref(), arms, expr.span)
            }
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.analyze_for_expr(counter, from, to, step.as_deref(), body, expr.span),
            ExprKind::Tuple(items) => {
                for item in items {
                    self.analyze_expr(item);
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => {
                let pine_type = self.analyze_call(callee, args, expr.span);
                if let Some(pine_type) = pine_type {
                    self.expr_types.insert(span_key(expr.span), pine_type);
                }
                pine_type
            }
            ExprKind::History { expr, offset } => {
                let value_type = self.analyze_expr(expr);
                let offset_type = self.analyze_expr(offset);
                self.validate_history_offset(offset, offset_type);
                if matches!(
                    value_type.map(|pine_type| pine_type.kind),
                    Some(ValueKind::UserType)
                ) {
                    self.unsupported(
                        "user-defined type history",
                        "history references on user-defined type values are not supported in the current UDT subset",
                        expr.span,
                    );
                    return None;
                }
                value_type.map(|value_type| PineType::new(Qualifier::Series, value_type.kind))
            }
        }
    }

    fn enter_expr_analysis(&mut self, span: Span) -> bool {
        if self.expr_depth >= MAX_SEMA_EXPR_DEPTH {
            self.diagnostics.push(Diagnostic::error(
                "E_SEMA_EXPR_DEPTH",
                "expression is too deeply nested for semantic analysis",
                span,
            ));
            return false;
        }

        self.expr_depth += 1;
        true
    }

    fn exit_expr_analysis(&mut self) {
        self.expr_depth = self.expr_depth.saturating_sub(1);
    }

    pub(crate) fn analyze_switch_expr(
        &mut self,
        selector: Option<&Expr>,
        arms: &[SwitchArm],
        span: Span,
    ) -> Option<PineType> {
        let selector_type = selector.and_then(|selector| self.analyze_expr(selector));
        let mut condition_qualifier = selector_type.map_or(Qualifier::Const, |ty| ty.qualifier);
        let mut result_type = None;
        let mut has_type_error = false;

        self.compatibility.supported.push(FeatureUse {
            feature: "switch".to_owned(),
            span,
        });

        for arm in arms {
            if let Some(condition) = &arm.condition {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    condition_qualifier =
                        strongest_qualifier(condition_qualifier, condition_type.qualifier);
                    if selector.is_none() {
                        self.expect_bool(condition_type, condition.span);
                    }
                }
            }

            if let Some(arm_type) = self.analyze_expr(&arm.result) {
                match merge_result_types(result_type, arm_type) {
                    Some(merged) => result_type = Some(merged),
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            "E_BRANCH_TYPE",
                            format!(
                                "switch arms have incompatible types {:?} and {:?}",
                                result_type.unwrap_or(UNKNOWN).kind,
                                arm_type.kind
                            ),
                            span,
                        ));
                        has_type_error = true;
                    }
                }
            }
        }

        if has_type_error {
            return None;
        }

        result_type.and_then(|pine_type| {
            let pine_type = PineType::new(
                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                pine_type.kind,
            );
            if pine_type.kind == ValueKind::UserType && !self.mark_switch_user_type(span, arms) {
                self.diagnostics.push(Diagnostic::error(
                    "E_BRANCH_TYPE",
                    "switch user-defined type arms must resolve to the same local UDT",
                    span,
                ));
                return None;
            }
            Some(pine_type)
        })
    }

    pub(crate) fn analyze_for_expr(
        &mut self,
        counter: &str,
        from: &Expr,
        to: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
        span: Span,
    ) -> Option<PineType> {
        let from_type = self.analyze_expr(from);
        let to_type = self.analyze_expr(to);
        let step_type = step.and_then(|step| self.analyze_expr(step));
        if let Some(from_type) = from_type {
            self.expect_int(from_type, from.span);
        }
        if let Some(to_type) = to_type {
            self.expect_int(to_type, to.span);
        }
        if let Some((step, step_type)) = step.zip(step_type) {
            self.expect_int(step_type, step.span);
            self.expect_non_zero_loop_step(step);
        }

        self.compatibility.supported.push(FeatureUse {
            feature: "for".to_owned(),
            span,
        });

        let counter_type = PineType::new(
            strongest_qualifier(
                from_type.unwrap_or(UNKNOWN).qualifier,
                to_type.unwrap_or(UNKNOWN).qualifier,
            ),
            ValueKind::Int,
        );
        self.block_depth += 1;
        self.loop_depth += 1;
        self.scope.push_scope();
        let counter_symbol =
            self.define_local_symbol(counter, counter_type, None, self.function_depth == 0);
        self.bind_symbol(counter, span, counter_symbol);

        let return_type = if let Some((last, prefix)) = body.split_last() {
            for statement in prefix {
                self.analyze_stmt(statement);
            }
            match &last.kind {
                StmtKind::Expr(expr) => self.analyze_expr(expr),
                _ => {
                    self.analyze_stmt(last);
                    self.diagnostics.push(Diagnostic::error(
                        "E_LOOP_RETURN",
                        "for expression body must end with an expression",
                        last.span,
                    ));
                    None
                }
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RETURN",
                "for expression body must end with an expression",
                span,
            ));
            None
        };

        self.scope.pop_scope();
        self.loop_depth -= 1;
        self.block_depth -= 1;
        return_type
    }

    pub(crate) fn validate_history_offset(&mut self, offset: &Expr, offset_type: Option<PineType>) {
        if let Some(value) = const_int_value(offset) {
            if value < 0 {
                self.unsupported(
                    "negative_history_offset",
                    "history offsets must be non-negative in the current supported subset",
                    offset.span,
                );
            }
            return;
        }

        let Some(offset_type) = offset_type else {
            self.unsupported(
                "dynamic_history_offset",
                "dynamic history offsets require an integer expression in the current supported subset",
                offset.span,
            );
            return;
        };

        if offset_type.kind == ValueKind::Int {
            return;
        }

        self.unsupported(
            "dynamic_history_offset",
            "dynamic history offsets require an integer expression in the current supported subset",
            offset.span,
        );
    }

    pub(crate) fn resolve_qualified_value(&mut self, name: &str, span: Span) -> Option<PineType> {
        if self.validate_strategy_state_variable(name, span) {
            return None;
        }
        if pine_builtins::named_color(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Color));
        }
        if pine_builtins::named_float_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Float));
        }
        if pine_builtins::named_int_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Int));
        }
        if pine_builtins::named_string_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::String));
        }
        if let Some(pine_type) = pine_builtins::builtin_series_value_type(name) {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(pine_type);
        }

        self.check_feature_name(name, span);
        if name.starts_with("color.") {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_COLOR",
                format!("unknown named color `{name}`"),
                span,
            ));
        }
        None
    }

    pub(crate) fn resolve_symbol(&mut self, name: &str, span: Span) -> Option<PineType> {
        if let Some(symbol) = self.scope.resolve(name) {
            self.bind_symbol(name, span, symbol);
            Some(symbol.pine_type)
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_SYMBOL",
                format!("unknown symbol `{name}`"),
                span,
            ));
            None
        }
    }

    pub(crate) fn validate_assignment(
        &mut self,
        name: &str,
        target_type: PineType,
        value_type: PineType,
        span: Span,
    ) {
        if !can_assign(target_type, value_type) {
            self.diagnostics.push(Diagnostic::error(
                "E_ASSIGN_TYPE",
                format!(
                    "cannot assign {:?} {:?} to `{}` of type {:?} {:?}",
                    value_type.qualifier,
                    value_type.kind,
                    name,
                    target_type.qualifier,
                    target_type.kind
                ),
                span,
            ));
        }
    }

    pub(crate) fn infer_unary(
        &mut self,
        op: UnaryOp,
        expr_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        match op {
            UnaryOp::Plus | UnaryOp::Minus if is_numeric(expr_type.kind) => Some(expr_type),
            UnaryOp::Not if expr_type.kind == ValueKind::Bool => Some(expr_type),
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_OPERATOR_TYPE",
                    format!(
                        "operator {:?} does not accept {:?} {:?}",
                        op, expr_type.qualifier, expr_type.kind
                    ),
                    span,
                ));
                None
            }
        }
    }

    pub(crate) fn infer_binary(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if is_numeric(left_type.kind) && is_numeric(right_type.kind) {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        numeric_result_kind(op, left_type.kind, right_type.kind),
                    ))
                } else {
                    self.operator_error(op, left_type, right_type, span);
                    None
                }
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Gt
            | BinaryOp::Gte
            | BinaryOp::Lt
            | BinaryOp::Lte => Some(PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::Bool,
            )),
            BinaryOp::And | BinaryOp::Or => {
                if left_type.kind == ValueKind::Bool && right_type.kind == ValueKind::Bool {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    ))
                } else {
                    self.operator_error(op, left_type, right_type, span);
                    None
                }
            }
        }
    }

    pub(crate) fn operator_error(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            "E_OPERATOR_TYPE",
            format!(
                "operator {:?} does not accept {:?} {:?} and {:?} {:?}",
                op, left_type.qualifier, left_type.kind, right_type.qualifier, right_type.kind
            ),
            span,
        ));
    }

    pub(crate) fn expect_bool(&mut self, pine_type: PineType, span: Span) {
        if pine_type.kind != ValueKind::Bool {
            self.diagnostics.push(Diagnostic::error(
                "E_CONDITION_TYPE",
                format!(
                    "condition must be bool, got {:?} {:?}",
                    pine_type.qualifier, pine_type.kind
                ),
                span,
            ));
        }
    }

    pub(crate) fn expect_int(&mut self, pine_type: PineType, span: Span) {
        if pine_type.kind != ValueKind::Int {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RANGE_TYPE",
                format!(
                    "for loop range must be int, got {:?} {:?}",
                    pine_type.qualifier, pine_type.kind
                ),
                span,
            ));
        }
    }

    pub(crate) fn expect_non_zero_loop_step(&mut self, step: &Expr) {
        if const_int_value(step) == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_STEP",
                "for loop step cannot be zero",
                step.span,
            ));
        }
    }

    pub(crate) fn merge_branch_types(
        &mut self,
        condition_type: PineType,
        then_type: PineType,
        else_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        let Some(kind) = common_kind(then_type.kind, else_type.kind) else {
            self.diagnostics.push(Diagnostic::error(
                "E_BRANCH_TYPE",
                format!(
                    "ternary branches have incompatible types {:?} and {:?}",
                    then_type.kind, else_type.kind
                ),
                span,
            ));
            return None;
        };

        Some(PineType::new(
            strongest_qualifier(
                condition_type.qualifier,
                strongest_qualifier(then_type.qualifier, else_type.qualifier),
            ),
            kind,
        ))
    }

    pub(crate) fn type_of_expr(&self, expr: &Expr) -> Option<PineType> {
        self.type_of_expr_with_params(expr, &HashMap::new())
    }

    pub(crate) fn type_of_expr_with_params(
        &self,
        expr: &Expr,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        if let Some(pine_type) = self.expr_types.get(&span_key(expr.span)) {
            return Some(*pine_type);
        }
        match &expr.kind {
            ExprKind::Literal(literal) => Some(literal_type(literal)),
            ExprKind::Identifier(name) => param_types
                .get(name)
                .copied()
                .or_else(|| {
                    self.bound_symbol(name, expr.span)
                        .map(|symbol| symbol.pine_type)
                })
                .or_else(|| self.scope.resolve(name).map(|symbol| symbol.pine_type)),
            ExprKind::QualifiedName(parts) => {
                if let Some(pine_type) = self.type_of_bound_user_type_field_access(parts, expr.span)
                {
                    return Some(pine_type);
                }
                if let Some(pine_type) = self.type_of_user_type_field_access(parts) {
                    return Some(pine_type);
                }
                let name = expr_name(expr)?;
                pine_builtins::named_color(&name)
                    .map(|_| PineType::new(Qualifier::Const, ValueKind::Color))
                    .or_else(|| {
                        pine_builtins::named_float_constant(&name)
                            .map(|_| PineType::new(Qualifier::Const, ValueKind::Float))
                    })
                    .or_else(|| {
                        pine_builtins::named_int_constant(&name)
                            .map(|_| PineType::new(Qualifier::Const, ValueKind::Int))
                    })
                    .or_else(|| {
                        pine_builtins::named_string_constant(&name)
                            .map(|_| PineType::new(Qualifier::Const, ValueKind::String))
                    })
                    .or_else(|| pine_builtins::builtin_series_value_type(&name))
            }
            ExprKind::Unary { expr, .. } => self.type_of_expr_with_params(expr, param_types),
            ExprKind::Binary { op, left, right } => {
                let left_type = self.type_of_expr_with_params(left, param_types)?;
                let right_type = self.type_of_expr_with_params(right, param_types)?;
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        numeric_result_kind(*op, left_type.kind, right_type.kind),
                    )),
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Gt
                    | BinaryOp::Gte
                    | BinaryOp::Lt
                    | BinaryOp::Lte
                    | BinaryOp::And
                    | BinaryOp::Or => Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    )),
                }
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_type = self.type_of_expr_with_params(condition, param_types)?;
                let then_type = self.type_of_expr_with_params(then_expr, param_types)?;
                let else_type = self.type_of_expr_with_params(else_expr, param_types)?;
                Some(PineType::new(
                    strongest_qualifier(
                        condition_type.qualifier,
                        strongest_qualifier(then_type.qualifier, else_type.qualifier),
                    ),
                    common_kind(then_type.kind, else_type.kind)?,
                ))
            }
            ExprKind::Switch { selector, arms } => {
                self.type_of_switch_expr_with_params(selector.as_deref(), arms, param_types)
            }
            ExprKind::For { body, .. } => {
                let last = body.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.type_of_expr_with_params(expr, param_types)
            }
            ExprKind::Tuple(items) => {
                for item in items {
                    self.type_of_expr_with_params(item, param_types)?;
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => {
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| self.type_of_expr_with_params(&arg.value, param_types))
                    .collect();
                let name = expr_name(callee)?;
                if let Some(pine_type) =
                    self.type_of_user_type_constructor_with_params(&name, args, param_types)
                {
                    Some(pine_type)
                } else if let Some(signature) = pine_builtins::get_phase_1_builtin(&name) {
                    if is_ta_vwap_bands_call(&name, args) {
                        return Some(pine_builtins::tuple_return_type());
                    }
                    match signature.returns {
                        ReturnSpec::Fixed(pine_type) => Some(pine_type),
                        ReturnSpec::Tuple(_) => Some(pine_builtins::tuple_return_type()),
                        ReturnSpec::SameAsArg(index) => arg_types.get(index).copied().flatten(),
                        ReturnSpec::BoolFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(pine_builtins::fallback_bool_for_arg),
                        ReturnSpec::ColorFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(pine_builtins::color_return_for_arg),
                        ReturnSpec::PromotedColor => promoted_color_type(&arg_types),
                        ReturnSpec::PromotedBool => promoted_bool_type(&arg_types),
                        ReturnSpec::PromotedInt => promoted_int_type(&arg_types),
                        ReturnSpec::PromotedString => promoted_string_type(&arg_types),
                        ReturnSpec::FloatFromStringArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(float_return_for_arg),
                        ReturnSpec::PromotedNumeric => promoted_numeric_type(&arg_types),
                        ReturnSpec::ArrayElement(index) => {
                            array_element_return_type(&arg_types, index)
                        }
                        ReturnSpec::ArrayNumeric(index) => {
                            array_numeric_return_type(&arg_types, index)
                        }
                        ReturnSpec::ArrayFromArgs => array_from_return_type(&arg_types),
                        ReturnSpec::IntFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(int_return_for_arg),
                        ReturnSpec::FloatFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(float_return_for_arg),
                        ReturnSpec::SeriesFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .and_then(series_return_for_arg),
                        ReturnSpec::ChangeFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .and_then(pine_builtins::change_return_for_arg),
                        ReturnSpec::PromotedFloat => promoted_float_type(&arg_types),
                        ReturnSpec::Round => round_return_type(&arg_types),
                        ReturnSpec::InputFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .and_then(pine_builtins::input_return_for_arg),
                    }
                } else if let Some((receiver_name, method_name)) = method_call_parts(callee) {
                    self.type_of_method_call_with_params(
                        receiver_name,
                        method_name,
                        callee.span,
                        &arg_types,
                        param_types,
                    )
                } else {
                    let function = self.functions.get(&name)?;
                    let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
                    let mut nested_param_types = HashMap::new();
                    for (arg_type, param_index) in arg_types.into_iter().zip(arg_indices) {
                        let param = &function.params[param_index];
                        nested_param_types.insert(param.clone(), arg_type?);
                    }
                    self.type_of_function_body_with_params(&function.body, &nested_param_types)
                }
            }
            ExprKind::History { expr, .. } => self
                .type_of_expr_with_params(expr, param_types)
                .map(|pine_type| PineType::new(Qualifier::Series, pine_type.kind)),
        }
    }

    pub(crate) fn type_of_method_call_with_params(
        &self,
        receiver_name: &str,
        method_name: &str,
        receiver_span: Span,
        arg_types: &[Option<PineType>],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let receiver_type = param_types
            .get(receiver_name)
            .copied()
            .or_else(|| {
                self.bound_symbol(receiver_name, receiver_span)
                    .map(|symbol| symbol.pine_type)
            })
            .or_else(|| {
                self.scope
                    .resolve(receiver_name)
                    .map(|symbol| symbol.pine_type)
            })?;
        let signature = if let Some(builtin_name) =
            drawing_method_builtin_name(receiver_type.kind, method_name)
        {
            pine_builtins::get_phase_1_builtin(&builtin_name)?
        } else if is_array_kind(receiver_type.kind) {
            pine_builtins::get_phase_1_builtin(array_method_builtin_name(method_name)?)?
        } else {
            return None;
        };
        let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
        method_arg_types.push(Some(receiver_type));
        method_arg_types.extend(arg_types.iter().copied());
        self.return_type(signature, &method_arg_types)
    }

    pub(crate) fn type_of_switch_expr_with_params(
        &self,
        selector: Option<&Expr>,
        arms: &[SwitchArm],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let selector_type = match selector {
            Some(selector) => Some(self.type_of_expr_with_params(selector, param_types)?),
            None => None,
        };
        let mut condition_qualifier = selector_type.map_or(Qualifier::Const, |ty| ty.qualifier);
        let mut result_type = None;

        for arm in arms {
            if let Some(condition) = &arm.condition {
                let condition_type = self.type_of_expr_with_params(condition, param_types)?;
                condition_qualifier =
                    strongest_qualifier(condition_qualifier, condition_type.qualifier);
            }
            let arm_type = self.type_of_expr_with_params(&arm.result, param_types)?;
            result_type = Some(merge_result_types(result_type, arm_type)?);
        }

        result_type.map(|pine_type| {
            PineType::new(
                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                pine_type.kind,
            )
        })
    }

    pub(crate) fn type_of_function_body_with_params(
        &self,
        body: &FunctionBody,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        match body {
            FunctionBody::Expr(expr) => self.type_of_expr_with_params(expr, param_types),
            FunctionBody::Block(statements) => {
                let last = statements.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.type_of_expr_with_params(expr, param_types)
            }
        }
    }

    pub(crate) fn tuple_element_types(&self, expr: &Expr) -> Option<Vec<PineType>> {
        match &expr.kind {
            ExprKind::Tuple(items) => items
                .iter()
                .map(|item| self.type_of_expr(item))
                .collect::<Option<_>>(),
            ExprKind::Call { callee, args } => {
                let name = expr_name(callee)?;
                if is_ta_vwap_bands_call(&name, args) {
                    let series_float = PineType::new(Qualifier::Series, ValueKind::Float);
                    return Some(vec![series_float, series_float, series_float]);
                }
                let signature = pine_builtins::get_phase_1_builtin(&expr_name(callee)?)?;
                match signature.returns {
                    ReturnSpec::Tuple(types) => Some(types.to_vec()),
                    _ => None,
                }
            }
            ExprKind::For { body, .. } => {
                let last = body.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.tuple_element_types(expr)
            }
            _ => None,
        }
    }
}
