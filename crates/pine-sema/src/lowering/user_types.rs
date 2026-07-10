use crate::source_graph::{SourceContextId, SourceId};

use super::*;

#[derive(Clone)]
enum LoweredUserTypeArrayResult {
    Known(String),
    Na,
    Unknown,
}

struct LoweredUserTypeCall {
    key: String,
    source_context_id: SourceContextId,
    body: FunctionBody,
    array_aliases: HashMap<String, LoweredUserTypeArrayResult>,
    user_type_aliases: HashMap<String, LoweredUserTypeArrayResult>,
    supports_array_return: bool,
}

enum LoweredUserTypeCallResolution {
    Resolved(Box<LoweredUserTypeCall>),
    Unresolved,
}

impl Analyzer {
    pub(super) fn user_type_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        match self.user_type_result_with_params_and_aliases(
            expr,
            param_exprs,
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        ) {
            LoweredUserTypeArrayResult::Known(type_name) => Some(type_name),
            LoweredUserTypeArrayResult::Na | LoweredUserTypeArrayResult::Unknown => None,
        }
    }

    pub(super) fn user_type_array_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        match self.user_type_array_result_with_params_and_aliases(
            expr,
            param_exprs,
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        ) {
            LoweredUserTypeArrayResult::Known(type_name) => Some(type_name),
            LoweredUserTypeArrayResult::Na | LoweredUserTypeArrayResult::Unknown => None,
        }
    }

    fn user_type_array_result_with_params_and_aliases(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        let identifier_name = match &expr.kind {
            ExprKind::Identifier(name) => Some(name),
            ExprKind::QualifiedName(parts) if parts.len() == 1 => Some(&parts[0]),
            _ => None,
        };
        if let Some(name) = identifier_name {
            if name == "na" {
                return LoweredUserTypeArrayResult::Na;
            }
            if let Some(result) = array_aliases.get(name) {
                return result.clone();
            }
            if let Some(HirExpr {
                kind: HirExprKind::Symbol(symbol_id),
                ..
            }) = param_exprs.get(name)
                && let Some(type_name) = self.symbol_user_type_arrays.get(symbol_id)
            {
                return LoweredUserTypeArrayResult::Known(type_name.clone());
            }
            if let Some(symbol) = self.bound_symbol(name, expr.span)
                && let Some(type_name) = self.symbol_user_type_arrays.get(&symbol.id)
            {
                return LoweredUserTypeArrayResult::Known(type_name.clone());
            }
            if let Some(type_name) = self.user_type_array_name_of_expr(expr) {
                return LoweredUserTypeArrayResult::Known(type_name);
            }
            if self
                .bound_symbol(name, expr.span)
                .is_some_and(|symbol| symbol.pine_type.kind == ValueKind::Na)
            {
                return LoweredUserTypeArrayResult::Na;
            }
            return LoweredUserTypeArrayResult::Unknown;
        }

        match &expr.kind {
            ExprKind::History { expr, .. } => {
                return self.user_type_array_result_with_params_and_aliases(
                    expr,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
            }
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => {
                return Self::merge_lowered_user_type_array_results([
                    self.user_type_array_result_with_params_and_aliases(
                        then_expr,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                    self.user_type_array_result_with_params_and_aliases(
                        else_expr,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                ]);
            }
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                return Self::merge_lowered_user_type_array_results([
                    self.user_type_array_branch_result_with_params_and_aliases(
                        then_branch,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                    self.user_type_array_branch_result_with_params_and_aliases(
                        else_branch,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                ]);
            }
            ExprKind::Switch { arms, .. } => {
                return Self::merge_lowered_user_type_array_results(arms.iter().map(|arm| {
                    self.user_type_array_switch_result_with_params_and_aliases(
                        &arm.result,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
                }));
            }
            ExprKind::ForIn {
                value,
                iterable,
                body,
                ..
            } => {
                let mut loop_user_type_aliases = user_type_aliases.clone();
                let element_result = self.user_type_array_result_with_params_and_aliases(
                    iterable,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
                loop_user_type_aliases.insert(value.clone(), element_result);
                return self.user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    array_aliases,
                    &loop_user_type_aliases,
                    call_stack,
                );
            }
            ExprKind::For { body, .. } | ExprKind::While { body, .. } => {
                return self.user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
            }
            ExprKind::Call { callee, args } => {
                let Some(name) = expr_name(callee) else {
                    return LoweredUserTypeArrayResult::Unknown;
                };
                if let Some(type_name) = name
                    .strip_prefix("array.new<")
                    .and_then(|name| name.strip_suffix('>'))
                    && (self.user_types.contains_key(type_name)
                        || self.imported_user_type_array_is_supported(type_name))
                {
                    return LoweredUserTypeArrayResult::Known(type_name.to_owned());
                }
                if matches!(name.as_str(), "array.copy" | "array.slice" | "array.concat") {
                    return args
                        .first()
                        .map_or(LoweredUserTypeArrayResult::Unknown, |arg| {
                            self.user_type_array_result_with_params_and_aliases(
                                &arg.value,
                                param_exprs,
                                array_aliases,
                                user_type_aliases,
                                call_stack,
                            )
                        });
                }
                if name == "array.from" {
                    return Self::merge_lowered_user_type_array_results(args.iter().map(|arg| {
                        self.user_type_result_with_params_and_aliases(
                            &arg.value,
                            param_exprs,
                            array_aliases,
                            user_type_aliases,
                            call_stack,
                        )
                    }));
                }
                if let ExprKind::QualifiedName(parts) = &callee.kind
                    && let [receiver, method] = parts.as_slice()
                    && matches!(method.as_str(), "copy" | "slice" | "concat")
                {
                    let result = self.user_type_array_named_result_with_params_and_aliases(
                        receiver,
                        callee.span,
                        param_exprs,
                        array_aliases,
                    );
                    if !matches!(result, LoweredUserTypeArrayResult::Unknown) {
                        return result;
                    }
                }
                if let Some(call) = self.lowered_user_type_call(
                    callee,
                    args,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                ) {
                    return match call {
                        LoweredUserTypeCallResolution::Resolved(call) => {
                            self.user_type_array_call_result(*call, call_stack)
                        }
                        LoweredUserTypeCallResolution::Unresolved => {
                            LoweredUserTypeArrayResult::Unknown
                        }
                    };
                }
            }
            _ => {}
        }

        self.user_type_array_name_of_expr(expr).map_or(
            LoweredUserTypeArrayResult::Unknown,
            LoweredUserTypeArrayResult::Known,
        )
    }

    fn user_type_array_named_result_with_params_and_aliases(
        &self,
        name: &str,
        span: Span,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, LoweredUserTypeArrayResult>,
    ) -> LoweredUserTypeArrayResult {
        if let Some(result) = aliases.get(name) {
            return result.clone();
        }
        if let Some(HirExpr {
            kind: HirExprKind::Symbol(symbol_id),
            ..
        }) = param_exprs.get(name)
            && let Some(type_name) = self.symbol_user_type_arrays.get(symbol_id)
        {
            return LoweredUserTypeArrayResult::Known(type_name.clone());
        }
        let Some(symbol) = self
            .bound_symbol(name, span)
            .or_else(|| self.scope.resolve(name))
        else {
            return LoweredUserTypeArrayResult::Unknown;
        };
        self.symbol_user_type_arrays
            .get(&symbol.id)
            .map_or(LoweredUserTypeArrayResult::Unknown, |type_name| {
                LoweredUserTypeArrayResult::Known(type_name.clone())
            })
    }

    fn user_type_named_result_with_params_and_aliases(
        &self,
        name: &str,
        span: Span,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, LoweredUserTypeArrayResult>,
    ) -> LoweredUserTypeArrayResult {
        if let Some(result) = aliases.get(name) {
            return result.clone();
        }
        if let Some(HirExpr {
            kind: HirExprKind::Symbol(symbol_id),
            ..
        }) = param_exprs.get(name)
            && let Some(type_name) = self.symbol_user_types.get(symbol_id)
        {
            return LoweredUserTypeArrayResult::Known(type_name.clone());
        }
        let Some(symbol) = self
            .bound_symbol(name, span)
            .or_else(|| self.scope.resolve(name))
        else {
            return LoweredUserTypeArrayResult::Unknown;
        };
        self.symbol_user_types
            .get(&symbol.id)
            .map_or(LoweredUserTypeArrayResult::Unknown, |type_name| {
                LoweredUserTypeArrayResult::Known(type_name.clone())
            })
    }

    fn lowered_user_type_call(
        &self,
        callee: &Expr,
        args: &[CallArg],
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> Option<LoweredUserTypeCallResolution> {
        let name = expr_name(callee)?;
        if let ExprKind::QualifiedName(parts) = &callee.kind
            && let [qualifier, method_name] = parts.as_slice()
        {
            if let Some(receiver_arg) = args.first()
                && let LoweredUserTypeArrayResult::Known(receiver_type) = self
                    .user_type_result_with_params_and_aliases(
                        &receiver_arg.value,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
                && (receiver_type == *qualifier
                    || receiver_type.starts_with(&format!("{qualifier}.")))
                && let Some(method) = self
                    .methods
                    .get(&(receiver_type.clone(), method_name.clone()))
                    .cloned()
            {
                return Some(
                    self.lowered_user_type_method_call(
                        receiver_type,
                        method_name,
                        method,
                        &args[1..],
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
                    .map_or(LoweredUserTypeCallResolution::Unresolved, |call| {
                        LoweredUserTypeCallResolution::Resolved(Box::new(call))
                    }),
                );
            }

            if let LoweredUserTypeArrayResult::Known(receiver_type) = self
                .user_type_named_result_with_params_and_aliases(
                    qualifier,
                    callee.span,
                    param_exprs,
                    user_type_aliases,
                )
                && let Some(method) = self
                    .methods
                    .get(&(receiver_type.clone(), method_name.clone()))
                    .cloned()
            {
                return Some(
                    self.lowered_user_type_method_call(
                        receiver_type,
                        method_name,
                        method,
                        args,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
                    .map_or(LoweredUserTypeCallResolution::Unresolved, |call| {
                        LoweredUserTypeCallResolution::Resolved(Box::new(call))
                    }),
                );
            }
        }

        let function = self.functions.get(&name)?.clone();
        Some(
            self.lowered_user_type_function_call(
                name,
                function,
                args,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            )
            .map_or(LoweredUserTypeCallResolution::Unresolved, |call| {
                LoweredUserTypeCallResolution::Resolved(Box::new(call))
            }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn lowered_user_type_function_call(
        &self,
        name: String,
        function: FunctionInfo,
        args: &[CallArg],
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> Option<LoweredUserTypeCall> {
        let (array_aliases, user_type_aliases) = self.lowered_user_type_call_aliases(
            &function.params,
            args,
            param_exprs,
            array_aliases,
            user_type_aliases,
            call_stack,
        )?;
        Some(LoweredUserTypeCall {
            key: format!("function:{name}"),
            source_context_id: function.source_context_id,
            body: function.body,
            array_aliases,
            user_type_aliases,
            supports_array_return: function.source_id == SourceId::root(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn lowered_user_type_method_call(
        &self,
        receiver_type: String,
        method_name: &str,
        method: MethodInfo,
        args: &[CallArg],
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> Option<LoweredUserTypeCall> {
        let param_names: Vec<_> = method
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let (array_aliases, mut user_type_aliases) = self.lowered_user_type_call_aliases(
            &param_names,
            args,
            param_exprs,
            array_aliases,
            user_type_aliases,
            call_stack,
        )?;
        user_type_aliases.insert(
            method.receiver_name,
            LoweredUserTypeArrayResult::Known(receiver_type.clone()),
        );
        Some(LoweredUserTypeCall {
            key: format!("method:{receiver_type}.{method_name}"),
            source_context_id: method.source_context_id,
            body: method.body,
            array_aliases,
            user_type_aliases,
            supports_array_return: method.source_id == SourceId::root(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn lowered_user_type_call_aliases(
        &self,
        params: &[String],
        args: &[CallArg],
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> Option<(
        HashMap<String, LoweredUserTypeArrayResult>,
        HashMap<String, LoweredUserTypeArrayResult>,
    )> {
        let arg_indices = resolve_udf_arg_indices(params, args).ok()?;
        let mut resolved_array_args = vec![LoweredUserTypeArrayResult::Unknown; params.len()];
        let mut resolved_user_type_args = vec![LoweredUserTypeArrayResult::Unknown; params.len()];
        for (arg, param_index) in args.iter().zip(arg_indices) {
            resolved_array_args[param_index] = self.user_type_array_result_with_params_and_aliases(
                &arg.value,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            );
            resolved_user_type_args[param_index] = self.user_type_result_with_params_and_aliases(
                &arg.value,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            );
        }
        Some((
            params.iter().cloned().zip(resolved_array_args).collect(),
            params
                .iter()
                .cloned()
                .zip(resolved_user_type_args)
                .collect(),
        ))
    }

    fn user_type_array_call_result(
        &self,
        call: LoweredUserTypeCall,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        if !call.supports_array_return
            || call_stack.len() >= MAX_FUNCTION_CALL_DEPTH
            || call_stack.contains(&call.key)
        {
            return LoweredUserTypeArrayResult::Unknown;
        }
        call_stack.push(call.key);
        let result =
            self.with_source_context_ref(call.source_context_id, |analyzer| match &call.body {
                FunctionBody::Expr(expr) => analyzer
                    .user_type_array_result_with_params_and_aliases(
                        expr,
                        &HashMap::new(),
                        &call.array_aliases,
                        &call.user_type_aliases,
                        call_stack,
                    ),
                FunctionBody::Block(statements) => analyzer
                    .user_type_array_branch_result_with_params_and_aliases(
                        statements,
                        &HashMap::new(),
                        &call.array_aliases,
                        &call.user_type_aliases,
                        call_stack,
                    ),
            });
        call_stack.pop();
        result
    }

    fn user_type_call_result(
        &self,
        call: LoweredUserTypeCall,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        if call_stack.len() >= MAX_FUNCTION_CALL_DEPTH || call_stack.contains(&call.key) {
            return LoweredUserTypeArrayResult::Unknown;
        }
        call_stack.push(call.key);
        let result =
            self.with_source_context_ref(call.source_context_id, |analyzer| match &call.body {
                FunctionBody::Expr(expr) => analyzer.user_type_result_with_params_and_aliases(
                    expr,
                    &HashMap::new(),
                    &call.array_aliases,
                    &call.user_type_aliases,
                    call_stack,
                ),
                FunctionBody::Block(statements) => analyzer
                    .user_type_branch_result_with_params_and_aliases(
                        statements,
                        &HashMap::new(),
                        &call.array_aliases,
                        &call.user_type_aliases,
                        call_stack,
                    ),
            });
        call_stack.pop();
        result
    }

    fn user_type_array_branch_result_with_params_and_aliases(
        &self,
        branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        outer_array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        outer_user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        let Some((last, prefix)) = branch.split_last() else {
            return LoweredUserTypeArrayResult::Unknown;
        };
        let mut array_aliases = outer_array_aliases.clone();
        let mut user_type_aliases = outer_user_type_aliases.clone();
        for statement in prefix {
            if let StmtKind::Decl { name, value, .. } = &statement.kind {
                let result = self.user_type_array_result_with_params_and_aliases(
                    value,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                array_aliases.insert(name.clone(), result);
                let user_type_result = self.user_type_result_with_params_and_aliases(
                    value,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                user_type_aliases.insert(name.clone(), user_type_result);
            }
        }
        match &last.kind {
            StmtKind::Expr(expr) => self.user_type_array_result_with_params_and_aliases(
                expr,
                param_exprs,
                &array_aliases,
                &user_type_aliases,
                call_stack,
            ),
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => Self::merge_lowered_user_type_array_results([
                self.user_type_array_branch_result_with_params_and_aliases(
                    then_branch,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
                self.user_type_array_branch_result_with_params_and_aliases(
                    else_branch,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
            ]),
            StmtKind::ForIn {
                value,
                iterable,
                body,
                ..
            } => {
                let mut loop_user_type_aliases = user_type_aliases.clone();
                let element_result = self.user_type_array_result_with_params_and_aliases(
                    iterable,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                loop_user_type_aliases.insert(value.clone(), element_result);
                self.user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    &array_aliases,
                    &loop_user_type_aliases,
                    call_stack,
                )
            }
            StmtKind::For { body, .. } | StmtKind::While { body, .. } => self
                .user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
            _ => LoweredUserTypeArrayResult::Unknown,
        }
    }

    fn user_type_array_switch_result_with_params_and_aliases(
        &self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        match result {
            SwitchArmResult::Expr(expr) => self.user_type_array_result_with_params_and_aliases(
                expr,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            ),
            SwitchArmResult::Block(statements) => self
                .user_type_array_branch_result_with_params_and_aliases(
                    statements,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                ),
        }
    }

    fn merge_lowered_user_type_array_results(
        results: impl IntoIterator<Item = LoweredUserTypeArrayResult>,
    ) -> LoweredUserTypeArrayResult {
        let mut resolved = None;
        for result in results {
            match result {
                LoweredUserTypeArrayResult::Known(type_name)
                    if resolved
                        .as_ref()
                        .is_some_and(|resolved| resolved != &type_name) =>
                {
                    return LoweredUserTypeArrayResult::Unknown;
                }
                LoweredUserTypeArrayResult::Known(type_name) => {
                    resolved.get_or_insert(type_name);
                }
                LoweredUserTypeArrayResult::Na => {}
                LoweredUserTypeArrayResult::Unknown => {
                    return LoweredUserTypeArrayResult::Unknown;
                }
            }
        }
        resolved.map_or(
            LoweredUserTypeArrayResult::Na,
            LoweredUserTypeArrayResult::Known,
        )
    }

    fn user_type_result_with_params_and_aliases(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        let identifier_name = match &expr.kind {
            ExprKind::Identifier(name) => Some(name),
            ExprKind::QualifiedName(parts) if parts.len() == 1 => Some(&parts[0]),
            _ => None,
        };
        if let Some(name) = identifier_name {
            if name == "na" {
                return LoweredUserTypeArrayResult::Na;
            }
            if let Some(result) = user_type_aliases.get(name) {
                return result.clone();
            }
            if let Some(HirExpr {
                kind: HirExprKind::Symbol(symbol_id),
                ..
            }) = param_exprs.get(name)
                && let Some(type_name) = self.symbol_user_types.get(symbol_id)
            {
                return LoweredUserTypeArrayResult::Known(type_name.clone());
            }
            if let Some(symbol) = self.bound_symbol(name, expr.span)
                && let Some(type_name) = self.symbol_user_types.get(&symbol.id)
            {
                return LoweredUserTypeArrayResult::Known(type_name.clone());
            }
            if let Some(type_name) = self.user_type_name_of_expr(expr) {
                return LoweredUserTypeArrayResult::Known(type_name);
            }
            if self
                .bound_symbol(name, expr.span)
                .is_some_and(|symbol| symbol.pine_type.kind == ValueKind::Na)
            {
                return LoweredUserTypeArrayResult::Na;
            }
            return LoweredUserTypeArrayResult::Unknown;
        }

        if let ExprKind::Call { callee, args } = &expr.kind
            && let Some(name) = expr_name(callee)
        {
            const ELEMENT_HELPERS: &[&str] = &["get", "pop", "remove", "shift", "first", "last"];
            if let Some(helper) = name.strip_prefix("array.")
                && ELEMENT_HELPERS.contains(&helper)
                && let Some(array_arg) = args.first()
                && let LoweredUserTypeArrayResult::Known(type_name) = self
                    .user_type_array_result_with_params_and_aliases(
                        &array_arg.value,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
            {
                return LoweredUserTypeArrayResult::Known(type_name);
            }
            if let ExprKind::QualifiedName(parts) = &callee.kind
                && let [receiver, method] = parts.as_slice()
                && ELEMENT_HELPERS.contains(&method.as_str())
                && let LoweredUserTypeArrayResult::Known(type_name) = self
                    .user_type_array_named_result_with_params_and_aliases(
                        receiver,
                        callee.span,
                        param_exprs,
                        array_aliases,
                    )
            {
                return LoweredUserTypeArrayResult::Known(type_name);
            }
            if let Some(call) = self.lowered_user_type_call(
                callee,
                args,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            ) {
                return match call {
                    LoweredUserTypeCallResolution::Resolved(call) => {
                        self.user_type_call_result(*call, call_stack)
                    }
                    LoweredUserTypeCallResolution::Unresolved => {
                        LoweredUserTypeArrayResult::Unknown
                    }
                };
            }
        }

        match &expr.kind {
            ExprKind::History { expr, .. } => {
                return self.user_type_result_with_params_and_aliases(
                    expr,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
            }
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => {
                return Self::merge_lowered_user_type_array_results([
                    self.user_type_result_with_params_and_aliases(
                        then_expr,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                    self.user_type_result_with_params_and_aliases(
                        else_expr,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                ]);
            }
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                return Self::merge_lowered_user_type_array_results([
                    self.user_type_branch_result_with_params_and_aliases(
                        then_branch,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                    self.user_type_branch_result_with_params_and_aliases(
                        else_branch,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    ),
                ]);
            }
            ExprKind::Switch { arms, .. } => {
                return Self::merge_lowered_user_type_array_results(arms.iter().map(|arm| {
                    self.user_type_switch_result_with_params_and_aliases(
                        &arm.result,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
                        call_stack,
                    )
                }));
            }
            ExprKind::ForIn {
                value,
                iterable,
                body,
                ..
            } => {
                let mut loop_user_type_aliases = user_type_aliases.clone();
                let element_result = self.user_type_array_result_with_params_and_aliases(
                    iterable,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
                loop_user_type_aliases.insert(value.clone(), element_result);
                return self.user_type_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    array_aliases,
                    &loop_user_type_aliases,
                    call_stack,
                );
            }
            ExprKind::For { body, .. } | ExprKind::While { body, .. } => {
                return self.user_type_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                );
            }
            _ => {}
        }

        self.user_type_name_of_expr(expr).map_or(
            LoweredUserTypeArrayResult::Unknown,
            LoweredUserTypeArrayResult::Known,
        )
    }

    fn user_type_branch_result_with_params_and_aliases(
        &self,
        branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        outer_array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        outer_user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        let Some((last, prefix)) = branch.split_last() else {
            return LoweredUserTypeArrayResult::Unknown;
        };
        let mut array_aliases = outer_array_aliases.clone();
        let mut user_type_aliases = outer_user_type_aliases.clone();
        for statement in prefix {
            if let StmtKind::Decl { name, value, .. } = &statement.kind {
                let array_result = self.user_type_array_result_with_params_and_aliases(
                    value,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                array_aliases.insert(name.clone(), array_result);
                let result = self.user_type_result_with_params_and_aliases(
                    value,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                user_type_aliases.insert(name.clone(), result);
            }
        }
        match &last.kind {
            StmtKind::Expr(expr) => self.user_type_result_with_params_and_aliases(
                expr,
                param_exprs,
                &array_aliases,
                &user_type_aliases,
                call_stack,
            ),
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => Self::merge_lowered_user_type_array_results([
                self.user_type_branch_result_with_params_and_aliases(
                    then_branch,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
                self.user_type_branch_result_with_params_and_aliases(
                    else_branch,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
            ]),
            StmtKind::ForIn {
                value,
                iterable,
                body,
                ..
            } => {
                let mut loop_user_type_aliases = user_type_aliases.clone();
                let element_result = self.user_type_array_result_with_params_and_aliases(
                    iterable,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                );
                loop_user_type_aliases.insert(value.clone(), element_result);
                self.user_type_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    &array_aliases,
                    &loop_user_type_aliases,
                    call_stack,
                )
            }
            StmtKind::For { body, .. } | StmtKind::While { body, .. } => self
                .user_type_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                    call_stack,
                ),
            _ => LoweredUserTypeArrayResult::Unknown,
        }
    }

    fn user_type_switch_result_with_params_and_aliases(
        &self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        call_stack: &mut Vec<String>,
    ) -> LoweredUserTypeArrayResult {
        match result {
            SwitchArmResult::Expr(expr) => self.user_type_result_with_params_and_aliases(
                expr,
                param_exprs,
                array_aliases,
                user_type_aliases,
                call_stack,
            ),
            SwitchArmResult::Block(statements) => self
                .user_type_branch_result_with_params_and_aliases(
                    statements,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                    call_stack,
                ),
        }
    }
}
