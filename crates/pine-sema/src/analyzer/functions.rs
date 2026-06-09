use crate::prelude::*;

pub(crate) fn resolve_udf_arg_indices(
    params: &[String],
    args: &[CallArg],
) -> Result<Vec<usize>, UdfArgError> {
    let mut used = vec![false; params.len()];
    let mut indices = Vec::with_capacity(args.len());
    let mut next_positional = 0;
    let mut saw_named = false;

    for arg in args {
        if let Some(name) = &arg.name {
            saw_named = true;
            let Some(param_index) = params.iter().position(|param| param == name) else {
                return Err(UdfArgError::UnknownName {
                    name: name.clone(),
                    span: arg.span,
                });
            };
            if used[param_index] {
                return Err(UdfArgError::Duplicate {
                    name: name.clone(),
                    span: arg.span,
                });
            }
            used[param_index] = true;
            indices.push(param_index);
        } else {
            if saw_named {
                return Err(UdfArgError::PositionalAfterNamed { span: arg.span });
            }
            while next_positional < used.len() && used[next_positional] {
                next_positional += 1;
            }
            if next_positional >= params.len() {
                return Err(UdfArgError::TooMany { span: arg.span });
            }
            used[next_positional] = true;
            indices.push(next_positional);
            next_positional += 1;
        }
    }

    if let Some(missing_index) = used.iter().position(|used| !*used) {
        return Err(UdfArgError::Missing {
            param: params[missing_index].clone(),
        });
    }

    Ok(indices)
}
pub(crate) fn contains_output_or_declaration_call(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            let name = expr_name(callee);
            name.as_deref().is_some_and(|name| {
                is_output_or_declaration_builtin(name)
                    || is_array_mutation_builtin(name)
                    || is_array_mutation_method_call_name(name)
            }) || args
                .iter()
                .any(|arg| contains_output_or_declaration_call(&arg.value))
        }
        ExprKind::Unary { expr, .. } | ExprKind::History { expr, .. } => {
            contains_output_or_declaration_call(expr)
        }
        ExprKind::Binary { left, right, .. } => {
            contains_output_or_declaration_call(left) || contains_output_or_declaration_call(right)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            contains_output_or_declaration_call(condition)
                || contains_output_or_declaration_call(then_expr)
                || contains_output_or_declaration_call(else_expr)
        }
        ExprKind::Switch { selector, arms } => {
            selector
                .as_deref()
                .is_some_and(contains_output_or_declaration_call)
                || arms.iter().any(|arm| {
                    arm.condition
                        .as_ref()
                        .is_some_and(contains_output_or_declaration_call)
                        || contains_output_or_declaration_call(&arm.result)
                })
        }
        ExprKind::For {
            from,
            to,
            step,
            body,
            ..
        } => {
            contains_output_or_declaration_call(from)
                || contains_output_or_declaration_call(to)
                || step
                    .as_deref()
                    .is_some_and(contains_output_or_declaration_call)
                || body.iter().any(|statement| match &statement.kind {
                    StmtKind::Expr(expr) => contains_output_or_declaration_call(expr),
                    StmtKind::Decl { value, .. }
                    | StmtKind::Reassign { value, .. }
                    | StmtKind::FieldReassign { value, .. }
                    | StmtKind::TupleDecl { value, .. } => {
                        contains_output_or_declaration_call(value)
                    }
                    StmtKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    } => {
                        contains_output_or_declaration_call(condition)
                            || then_branch.iter().any(|statement| {
                                statement_contains_output_or_declaration_call(statement)
                            })
                            || else_branch.iter().any(|statement| {
                                statement_contains_output_or_declaration_call(statement)
                            })
                    }
                    StmtKind::For { .. } | StmtKind::While { .. } => true,
                    StmtKind::Break | StmtKind::Continue | StmtKind::Function { .. } => false,
                    StmtKind::Import(_)
                    | StmtKind::Library(_)
                    | StmtKind::Export(_)
                    | StmtKind::UserType(_)
                    | StmtKind::Method(_) => false,
                    StmtKind::Unsupported { .. } => false,
                })
        }
        ExprKind::Tuple(items) => items.iter().any(contains_output_or_declaration_call),
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => false,
    }
}
pub(crate) fn statement_contains_output_or_declaration_call(statement: &Stmt) -> bool {
    match &statement.kind {
        StmtKind::Expr(expr) => contains_output_or_declaration_call(expr),
        StmtKind::Decl { value, .. }
        | StmtKind::Reassign { value, .. }
        | StmtKind::FieldReassign { value, .. }
        | StmtKind::TupleDecl { value, .. } => contains_output_or_declaration_call(value),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_output_or_declaration_call(condition)
                || then_branch
                    .iter()
                    .any(statement_contains_output_or_declaration_call)
                || else_branch
                    .iter()
                    .any(statement_contains_output_or_declaration_call)
        }
        StmtKind::For { .. } | StmtKind::While { .. } => true,
        StmtKind::Break | StmtKind::Continue | StmtKind::Function { .. } => false,
        StmtKind::Import(_)
        | StmtKind::Library(_)
        | StmtKind::Export(_)
        | StmtKind::UserType(_)
        | StmtKind::Method(_) => false,
        StmtKind::Unsupported { .. } => false,
    }
}
pub(crate) fn has_duplicate_param(params: &[String]) -> bool {
    for (index, param) in params.iter().enumerate() {
        if params[index + 1..].iter().any(|other| other == param) {
            return true;
        }
    }
    false
}

impl Analyzer {
    pub(crate) fn register_functions(&mut self, program: &Program) {
        for statement in &program.statements {
            let StmtKind::Function { name, params, body } = &statement.kind else {
                continue;
            };
            if self.functions.contains_key(name) {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_DUPLICATE",
                    format!("function `{name}` is already defined"),
                    statement.span,
                ));
                continue;
            }
            if pine_builtins::is_phase_1_builtin(name)
                || INITIAL_SYMBOLS
                    .iter()
                    .any(|(symbol_name, _)| symbol_name == name)
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_NAME",
                    format!("function `{name}` conflicts with an existing symbol"),
                    statement.span,
                ));
                continue;
            }
            if has_duplicate_param(params) {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_PARAM",
                    format!("function `{name}` has duplicate parameter names"),
                    statement.span,
                ));
                continue;
            }
            self.functions.insert(
                name.clone(),
                FunctionInfo {
                    params: params.clone(),
                    body: body.clone(),
                    span: statement.span,
                },
            );
        }
    }

    pub(crate) fn analyze_udf_call(
        &mut self,
        name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
        let function = self.functions.get(name)?.clone();
        if self.function_stack.iter().any(|active| active == name) {
            self.diagnostics.push(Diagnostic::error(
                "E_RECURSIVE_FUNCTION",
                format!("recursive function `{name}` is not supported"),
                span,
            ));
            return None;
        }
        if self.function_stack.len() >= MAX_FUNCTION_CALL_DEPTH {
            self.diagnostics.push(Diagnostic::error(
                "E_FUNCTION_CALL_DEPTH",
                "user-defined function call chain is too deep",
                span,
            ));
            return None;
        }
        for arg in args {
            if contains_output_or_declaration_call(&arg.value) {
                self.unsupported(
                    "function_side_effect",
                    "side-effecting calls cannot be passed as user-defined function arguments",
                    arg.span,
                );
            }
        }
        let arg_indices = match resolve_udf_arg_indices(&function.params, args) {
            Ok(arg_indices) => arg_indices,
            Err(error) => {
                self.report_udf_arg_error(name, span, function.params.len(), args.len(), error);
                return None;
            }
        };

        self.compatibility.supported.push(FeatureUse {
            feature: "function".to_owned(),
            span: function.span,
        });
        let mut resolved_arg_types = vec![None; function.params.len()];
        let mut resolved_arg_user_types = vec![None; function.params.len()];
        for (arg_index, param_index) in arg_indices.iter().copied().enumerate() {
            resolved_arg_types[param_index] = arg_types.get(arg_index).copied().flatten();
            resolved_arg_user_types[param_index] = args
                .get(arg_index)
                .and_then(|arg| self.user_type_name_of_expr(&arg.value));
        }
        self.scope.push_scope();
        for (param, (arg_type, arg_user_type)) in function
            .params
            .iter()
            .zip(resolved_arg_types.into_iter().zip(resolved_arg_user_types))
        {
            let symbol = self.define_local_symbol(param, arg_type.unwrap_or(UNKNOWN), None, false);
            if let Some(type_name) = arg_user_type {
                self.mark_symbol_user_type(symbol, type_name);
            }
        }
        self.function_stack.push(name.to_owned());
        self.function_depth += 1;
        let return_type = self.analyze_function_body(&function.body, function.span);
        if return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::UserType) {
            let returned_type_name = self
                .user_type_name_of_function_body(&function.body)
                .or_else(|| {
                    let FunctionBody::Expr(expr) = &function.body else {
                        return None;
                    };
                    let ExprKind::Identifier(returned_param) = &expr.kind else {
                        return None;
                    };
                    let param_index = function
                        .params
                        .iter()
                        .position(|param| param == returned_param)?;
                    let arg_index = arg_indices
                        .iter()
                        .position(|mapped_param_index| *mapped_param_index == param_index)?;
                    self.user_type_name_of_expr(&args[arg_index].value)
                });
            if let Some(type_name) = returned_type_name {
                self.mark_expr_user_type(call_span, type_name.clone());
                self.mark_expr_user_type(span, type_name);
            }
        }
        self.function_depth -= 1;
        self.function_stack.pop();
        self.scope.pop_scope();

        return_type
    }

    pub(crate) fn analyze_function_body(
        &mut self,
        body: &FunctionBody,
        span: Span,
    ) -> Option<PineType> {
        match body {
            FunctionBody::Expr(expr) => self.analyze_expr(expr),
            FunctionBody::Block(statements) => {
                let Some((last, prefix)) = statements.split_last() else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_FUNCTION_RETURN",
                        "user-defined function block must end with an expression",
                        span,
                    ));
                    return None;
                };
                for statement in prefix {
                    self.analyze_stmt(statement);
                }
                match &last.kind {
                    StmtKind::Expr(expr) => self.analyze_expr(expr),
                    _ => {
                        self.analyze_stmt(last);
                        self.diagnostics.push(Diagnostic::error(
                            "E_FUNCTION_RETURN",
                            "user-defined function block must end with an expression",
                            last.span,
                        ));
                        None
                    }
                }
            }
        }
    }

    pub(crate) fn report_udf_arg_error(
        &mut self,
        function_name: &str,
        call_span: Span,
        expected: usize,
        got: usize,
        error: UdfArgError,
    ) {
        match error {
            UdfArgError::UnknownName { name, span } => {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_ARG_NAME",
                    format!("`{function_name}` has no argument named `{name}`"),
                    span,
                ));
            }
            UdfArgError::Duplicate { name, span } => {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_ARG_DUPLICATE",
                    format!("`{function_name}` argument `{name}` is provided more than once"),
                    span,
                ));
            }
            UdfArgError::PositionalAfterNamed { span } => {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_ARG_ORDER",
                    "positional arguments cannot follow named arguments in user-defined function calls",
                    span,
                ));
            }
            UdfArgError::TooMany { span } => {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_ARITY",
                    format!("`{function_name}` expects {expected} argument(s), got {got}"),
                    span,
                ));
            }
            UdfArgError::Missing { param } => {
                self.diagnostics.push(Diagnostic::error(
                    "E_FUNCTION_ARITY",
                    format!("`{function_name}` is missing argument `{param}`"),
                    call_span,
                ));
            }
        }
    }
}
