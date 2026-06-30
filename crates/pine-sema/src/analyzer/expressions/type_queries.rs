use crate::prelude::*;

impl Analyzer {
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
                if let Some(pine_type) =
                    self.type_of_bound_chart_point_field_access(parts, expr.span)
                {
                    return Some(pine_type);
                }
                if let Some(pine_type) = self.type_of_chart_point_field_access(parts) {
                    return Some(pine_type);
                }
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
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.type_of_if_expr_with_params(condition, then_branch, else_branch, param_types),
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
            ExprKind::While { body, .. } => {
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
                } else if let Some(pine_type) = self
                    .type_of_imported_user_type_constructor_with_params(&name, args, param_types)
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
        } else if let Some(builtin_name) =
            matrix_method_builtin_name(receiver_type.kind, method_name)
        {
            pine_builtins::get_phase_1_builtin(builtin_name)?
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

    pub(crate) fn type_of_if_expr_with_params(
        &self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &[Stmt],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let condition_type = self.type_of_expr_with_params(condition, param_types)?;
        let then_type = self.type_of_branch_return_with_params(then_branch, param_types)?;
        let else_type = self.type_of_branch_return_with_params(else_branch, param_types)?;
        Some(PineType::new(
            strongest_qualifier(
                condition_type.qualifier,
                strongest_qualifier(then_type.qualifier, else_type.qualifier),
            ),
            common_kind(then_type.kind, else_type.kind)?,
        ))
    }

    fn type_of_branch_return_with_params(
        &self,
        branch: &[Stmt],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let last = branch.last()?;
        let StmtKind::Expr(expr) = &last.kind else {
            return None;
        };
        self.type_of_expr_with_params(expr, param_types)
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
            let arm_type = self.type_of_switch_arm_result_with_params(&arm.result, param_types)?;
            result_type = Some(merge_result_types(result_type, arm_type)?);
        }

        result_type.map(|pine_type| {
            PineType::new(
                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                pine_type.kind,
            )
        })
    }

    fn type_of_switch_arm_result_with_params(
        &self,
        result: &SwitchArmResult,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        match result {
            SwitchArmResult::Expr(expr) => self.type_of_expr_with_params(expr, param_types),
            SwitchArmResult::Block(statements) => {
                self.type_of_branch_return_with_params(statements, param_types)
            }
        }
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
                if name == "request.security" && args.len() == 3 {
                    return self.tuple_element_types(&args[2].value);
                }
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
            ExprKind::While { body, .. } => {
                let last = body.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.tuple_element_types(expr)
            }
            ExprKind::Switch { selector, arms } => {
                let mut condition_qualifier = selector
                    .as_deref()
                    .and_then(|selector| self.type_of_expr(selector))
                    .map_or(Qualifier::Const, |ty| ty.qualifier);
                let mut result_types: Option<Vec<PineType>> = None;
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        let condition_type = self.type_of_expr(condition)?;
                        condition_qualifier =
                            strongest_qualifier(condition_qualifier, condition_type.qualifier);
                    }
                    let arm_types = self.tuple_element_types_of_switch_arm_result(&arm.result)?;
                    result_types = Some(merge_tuple_element_types(result_types, arm_types)?);
                }
                result_types.map(|types| {
                    types
                        .into_iter()
                        .map(|pine_type| {
                            PineType::new(
                                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                                pine_type.kind,
                            )
                        })
                        .collect()
                })
            }
            _ => None,
        }
    }

    fn tuple_element_types_of_switch_arm_result(
        &self,
        result: &SwitchArmResult,
    ) -> Option<Vec<PineType>> {
        match result {
            SwitchArmResult::Expr(expr) => self.tuple_element_types(expr),
            SwitchArmResult::Block(statements) => {
                let last = statements.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.tuple_element_types(expr)
            }
        }
    }
}

fn merge_tuple_element_types(
    current: Option<Vec<PineType>>,
    next: Vec<PineType>,
) -> Option<Vec<PineType>> {
    let Some(current) = current else {
        return Some(next);
    };
    if current.len() != next.len() {
        return None;
    }
    current
        .into_iter()
        .zip(next)
        .map(|(current, next)| merge_result_types(Some(current), next))
        .collect()
}
