use crate::prelude::*;

mod type_queries;

fn for_in_expr_value_kind(
    iterable_type: PineType,
    analyzer: &Analyzer,
    iterable: &Expr,
) -> Option<(ValueKind, Option<String>)> {
    match iterable_type.kind {
        ValueKind::IntArray => Some((ValueKind::Int, None)),
        ValueKind::FloatArray => Some((ValueKind::Float, None)),
        ValueKind::BoolArray => Some((ValueKind::Bool, None)),
        ValueKind::StringArray => Some((ValueKind::String, None)),
        ValueKind::ColorArray => Some((ValueKind::Color, None)),
        ValueKind::LabelArray => Some((ValueKind::Label, None)),
        ValueKind::LineArray => Some((ValueKind::Line, None)),
        ValueKind::LineFillArray => Some((ValueKind::LineFill, None)),
        ValueKind::PolylineArray => Some((ValueKind::Polyline, None)),
        ValueKind::BoxArray => Some((ValueKind::Box, None)),
        ValueKind::TableArray => Some((ValueKind::Table, None)),
        ValueKind::ChartPointArray => Some((ValueKind::ChartPoint, None)),
        ValueKind::UserTypeArray => analyzer
            .user_type_array_name_of_expr(iterable)
            .map(|type_name| (ValueKind::UserType, Some(type_name))),
        ValueKind::FloatMatrix => Some((ValueKind::FloatArray, None)),
        ValueKind::IntMatrix => Some((ValueKind::IntArray, None)),
        ValueKind::BoolMatrix => Some((ValueKind::BoolArray, None)),
        ValueKind::StringMatrix => Some((ValueKind::StringArray, None)),
        ValueKind::ColorMatrix => Some((ValueKind::ColorArray, None)),
        _ => None,
    }
}

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
                if let Some(field_type) = self.resolve_chart_point_field_access(parts, expr.span) {
                    return Some(field_type);
                }
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
                                "ternary user-defined type branches must resolve to the same UDT identity",
                                expr.span,
                            ));
                            return None;
                        }
                        Some(pine_type)
                    }
                    _ => None,
                }
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.analyze_if_expr(condition, then_branch, else_branch, expr.span),
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
            ExprKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.analyze_for_in_expr(index.as_deref(), value, iterable, body, expr.span),
            ExprKind::While { condition, body } => {
                self.analyze_while_expr(condition, body, expr.span)
            }
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
            ExprKind::History {
                expr: value_expr,
                offset,
            } => {
                let value_type = self.analyze_expr(value_expr);
                let offset_type = self.analyze_expr(offset);
                self.validate_history_offset(offset, offset_type);
                if matches!(
                    value_type.map(|pine_type| pine_type.kind),
                    Some(ValueKind::UserType)
                ) {
                    if let Some(type_name) = self
                        .user_type_name_of_expr(value_expr)
                        .filter(|type_name| self.imported_user_type_history_is_supported(type_name))
                    {
                        self.mark_expr_user_type(expr.span, type_name);
                        return value_type
                            .map(|value_type| PineType::new(Qualifier::Series, value_type.kind));
                    }
                    self.unsupported(
                        "user-defined type history",
                        "history references on user-defined type values are not supported in the current UDT subset",
                        value_expr.span,
                    );
                    return None;
                }
                if matches!(
                    value_type.map(|pine_type| pine_type.kind),
                    Some(ValueKind::Map)
                ) && let Some(info) = self.map_type_of_expr(value_expr)
                {
                    self.mark_expr_map(expr.span, info);
                }
                if matches!(
                    value_type.map(|pine_type| pine_type.kind),
                    Some(ValueKind::UserTypeArray)
                ) && let Some(type_name) = self.user_type_array_name_of_expr(value_expr)
                {
                    self.mark_expr_user_type_array(expr.span, type_name);
                }
                value_type.map(|value_type| PineType::new(Qualifier::Series, value_type.kind))
            }
        }
    }

    pub(crate) fn analyze_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &[Stmt],
        span: Span,
    ) -> Option<PineType> {
        let condition_type = self.analyze_expr(condition);
        if let Some(condition_type) = condition_type {
            self.expect_bool(condition_type, condition.span);
        }

        self.compatibility.supported.push(FeatureUse {
            feature: "if".to_owned(),
            span,
        });

        self.block_depth += 1;
        let then_type = self.analyze_expr_branch_return(then_branch, "if", span);
        let else_type = self.analyze_expr_branch_return(else_branch, "if", span);
        self.block_depth -= 1;

        match (condition_type, then_type, else_type) {
            (Some(condition_type), Some(then_type), Some(else_type)) => {
                let pine_type =
                    self.merge_branch_types(condition_type, then_type, else_type, span)?;
                if pine_type.kind == ValueKind::UserType {
                    let type_name = self.user_type_name_of_if_branches(then_branch, else_branch);
                    if let Some(type_name) = type_name {
                        self.mark_expr_user_type(span, type_name);
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "E_BRANCH_TYPE",
                            "if user-defined type branches must resolve to the same UDT identity",
                            span,
                        ));
                        return None;
                    }
                }
                Some(pine_type)
            }
            _ => None,
        }
    }

    fn analyze_expr_branch_return(
        &mut self,
        branch: &[Stmt],
        keyword: &str,
        span: Span,
    ) -> Option<PineType> {
        let Some((last, prefix)) = branch.split_last() else {
            self.diagnostics.push(Diagnostic::error(
                "E_BRANCH_RETURN",
                format!("{keyword} expression branches must end with an expression"),
                span,
            ));
            return None;
        };

        self.scope.push_scope();
        for statement in prefix {
            self.analyze_stmt(statement);
        }
        let pine_type = match &last.kind {
            StmtKind::Expr(expr) => {
                let pine_type = self.analyze_expr(expr);
                if matches!(
                    pine_type,
                    Some(PineType {
                        kind: ValueKind::Void,
                        ..
                    })
                ) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_BRANCH_RETURN",
                        format!("{keyword} expression branches must end with a value-producing expression"),
                        expr.span,
                    ));
                    None
                } else {
                    pine_type
                }
            }
            _ => {
                self.analyze_stmt(last);
                self.diagnostics.push(Diagnostic::error(
                    "E_BRANCH_RETURN",
                    format!("{keyword} expression branches must end with an expression"),
                    last.span,
                ));
                None
            }
        };
        self.scope.pop_scope();
        pine_type
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

            if let Some(arm_type) = self.analyze_switch_arm_result(&arm.result, span) {
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
                    "switch user-defined type arms must resolve to the same UDT identity",
                    span,
                ));
                return None;
            }
            Some(pine_type)
        })
    }

    fn analyze_switch_arm_result(
        &mut self,
        result: &SwitchArmResult,
        span: Span,
    ) -> Option<PineType> {
        match result {
            SwitchArmResult::Expr(expr) => self.analyze_expr(expr),
            SwitchArmResult::Block(statements) => {
                self.block_depth += 1;
                let result = self.analyze_expr_branch_return(statements, "switch", span);
                self.block_depth -= 1;
                result
            }
        }
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
                StmtKind::Expr(expr) => {
                    let pine_type = self.analyze_expr(expr);
                    if matches!(
                        pine_type,
                        Some(PineType {
                            kind: ValueKind::Void,
                            ..
                        })
                    ) {
                        self.diagnostics.push(Diagnostic::error(
                            "E_LOOP_RETURN",
                            "for expression body must end with a value-producing expression",
                            expr.span,
                        ));
                        None
                    } else {
                        pine_type
                    }
                }
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

    pub(crate) fn analyze_for_in_expr(
        &mut self,
        index: Option<&str>,
        value: &str,
        iterable: &Expr,
        body: &[Stmt],
        span: Span,
    ) -> Option<PineType> {
        let Some(iterable_type) = self.analyze_expr(iterable) else {
            self.unsupported(
                "for...in expression",
                "for...in expressions currently support array<int>, array<float>, array<bool>, array<string>, array<color>, array<label>, array<line>, array<linefill>, array<polyline>, array<box>, array<table>, array<chart.point>, same-local or same-imported scalar-field UDT array, and matrix iterables only",
                span,
            );
            return None;
        };
        let Some((value_kind, user_type_name)) =
            for_in_expr_value_kind(iterable_type, self, iterable)
        else {
            self.unsupported(
                "for...in expression",
                "for...in expressions currently support array<int>, array<float>, array<bool>, array<string>, array<color>, array<label>, array<line>, array<linefill>, array<polyline>, array<box>, array<table>, array<chart.point>, same-local or same-imported scalar-field UDT array, and matrix iterables only",
                span,
            );
            return None;
        };

        self.compatibility.supported.push(FeatureUse {
            feature: "for".to_owned(),
            span,
        });

        self.block_depth += 1;
        self.loop_depth += 1;
        self.scope.push_scope();
        if let Some(index) = index {
            let index_symbol = self.define_local_symbol(
                index,
                PineType::new(Qualifier::Series, ValueKind::Int),
                None,
                self.function_depth == 0,
            );
            self.bind_symbol(index, span, index_symbol);
        }
        let value_symbol = self.define_local_symbol(
            value,
            PineType::new(Qualifier::Series, value_kind),
            None,
            self.function_depth == 0,
        );
        self.bind_symbol(value, span, value_symbol);
        if let Some(type_name) = user_type_name {
            self.mark_symbol_id_user_type(value_symbol.id, type_name);
        }

        let return_type = if let Some((last, prefix)) = body.split_last() {
            for statement in prefix {
                self.analyze_stmt(statement);
            }
            match &last.kind {
                StmtKind::Expr(expr) => {
                    let pine_type = self.analyze_expr(expr);
                    if matches!(
                        pine_type,
                        Some(PineType {
                            kind: ValueKind::Void,
                            ..
                        })
                    ) {
                        self.diagnostics.push(Diagnostic::error(
                            "E_LOOP_RETURN",
                            "for expression body must end with a value-producing expression",
                            expr.span,
                        ));
                        None
                    } else if matches!(
                        pine_type,
                        Some(PineType {
                            kind: ValueKind::UserType,
                            ..
                        })
                    ) {
                        if let Some(type_name) = self.user_type_name_of_expr(expr) {
                            self.mark_expr_user_type(span, type_name);
                            pine_type
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "E_LOOP_RETURN",
                                "for...in expression user-defined type result must resolve to a known same-local UDT identity",
                                expr.span,
                            ));
                            None
                        }
                    } else {
                        pine_type
                    }
                }
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

    pub(crate) fn analyze_while_expr(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
        span: Span,
    ) -> Option<PineType> {
        let condition_type = self.analyze_expr(condition);
        if let Some(condition_type) = condition_type {
            self.expect_bool(condition_type, condition.span);
        }
        self.compatibility.supported.push(FeatureUse {
            feature: "while".to_owned(),
            span,
        });

        self.block_depth += 1;
        self.loop_depth += 1;
        let return_type = self.analyze_expr_branch_return(body, "while", span);
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
}
