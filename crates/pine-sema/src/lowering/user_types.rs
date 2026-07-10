use super::*;

#[derive(Clone)]
enum LoweredUserTypeArrayResult {
    Known(String),
    Na,
    Unknown,
}

impl Analyzer {
    pub(super) fn user_type_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        match self.user_type_result_with_params_and_aliases(expr, param_exprs, &HashMap::new()) {
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
                    ),
                    self.user_type_array_result_with_params_and_aliases(
                        else_expr,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
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
                    ),
                    self.user_type_array_branch_result_with_params_and_aliases(
                        else_branch,
                        param_exprs,
                        array_aliases,
                        user_type_aliases,
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
                    )
                }));
            }
            ExprKind::For { body, .. }
            | ExprKind::ForIn { body, .. }
            | ExprKind::While { body, .. } => {
                return self.user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
                );
            }
            ExprKind::Call { callee, args } => {
                let Some(name) = expr_name(callee) else {
                    return LoweredUserTypeArrayResult::Unknown;
                };
                if matches!(name.as_str(), "array.copy" | "array.slice" | "array.concat") {
                    return args
                        .first()
                        .map_or(LoweredUserTypeArrayResult::Unknown, |arg| {
                            self.user_type_array_result_with_params_and_aliases(
                                &arg.value,
                                param_exprs,
                                array_aliases,
                                user_type_aliases,
                            )
                        });
                }
                if name == "array.from" {
                    return Self::merge_lowered_user_type_array_results(args.iter().map(|arg| {
                        self.user_type_result_with_params_and_aliases(
                            &arg.value,
                            param_exprs,
                            user_type_aliases,
                        )
                    }));
                }
                if let ExprKind::QualifiedName(parts) = &callee.kind
                    && let [receiver, method] = parts.as_slice()
                    && matches!(method.as_str(), "copy" | "slice" | "concat")
                {
                    return self.user_type_array_named_result_with_params_and_aliases(
                        receiver,
                        callee.span,
                        param_exprs,
                        array_aliases,
                    );
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

    fn user_type_array_branch_result_with_params_and_aliases(
        &self,
        branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        outer_array_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
        outer_user_type_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
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
                );
                array_aliases.insert(name.clone(), result);
                let user_type_result = self.user_type_result_with_params_and_aliases(
                    value,
                    param_exprs,
                    &user_type_aliases,
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
                ),
                self.user_type_array_branch_result_with_params_and_aliases(
                    else_branch,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
                ),
            ]),
            StmtKind::For { body, .. }
            | StmtKind::ForIn { body, .. }
            | StmtKind::While { body, .. } => self
                .user_type_array_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    &array_aliases,
                    &user_type_aliases,
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
    ) -> LoweredUserTypeArrayResult {
        match result {
            SwitchArmResult::Expr(expr) => self.user_type_array_result_with_params_and_aliases(
                expr,
                param_exprs,
                array_aliases,
                user_type_aliases,
            ),
            SwitchArmResult::Block(statements) => self
                .user_type_array_branch_result_with_params_and_aliases(
                    statements,
                    param_exprs,
                    array_aliases,
                    user_type_aliases,
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
        aliases: &HashMap<String, LoweredUserTypeArrayResult>,
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
                && let Some(type_name) =
                    self.user_type_array_name_of_expr_with_params(&array_arg.value, param_exprs)
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
                        &HashMap::new(),
                    )
            {
                return LoweredUserTypeArrayResult::Known(type_name);
            }
        }

        match &expr.kind {
            ExprKind::History { expr, .. } => {
                return self.user_type_result_with_params_and_aliases(expr, param_exprs, aliases);
            }
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => {
                return Self::merge_lowered_user_type_array_results([
                    self.user_type_result_with_params_and_aliases(then_expr, param_exprs, aliases),
                    self.user_type_result_with_params_and_aliases(else_expr, param_exprs, aliases),
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
                        aliases,
                    ),
                    self.user_type_branch_result_with_params_and_aliases(
                        else_branch,
                        param_exprs,
                        aliases,
                    ),
                ]);
            }
            ExprKind::Switch { arms, .. } => {
                return Self::merge_lowered_user_type_array_results(arms.iter().map(|arm| {
                    self.user_type_switch_result_with_params_and_aliases(
                        &arm.result,
                        param_exprs,
                        aliases,
                    )
                }));
            }
            ExprKind::For { body, .. }
            | ExprKind::ForIn { body, .. }
            | ExprKind::While { body, .. } => {
                return self.user_type_branch_result_with_params_and_aliases(
                    body,
                    param_exprs,
                    aliases,
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
        outer_aliases: &HashMap<String, LoweredUserTypeArrayResult>,
    ) -> LoweredUserTypeArrayResult {
        let Some((last, prefix)) = branch.split_last() else {
            return LoweredUserTypeArrayResult::Unknown;
        };
        let mut aliases = outer_aliases.clone();
        for statement in prefix {
            if let StmtKind::Decl { name, value, .. } = &statement.kind {
                let result =
                    self.user_type_result_with_params_and_aliases(value, param_exprs, &aliases);
                aliases.insert(name.clone(), result);
            }
        }
        match &last.kind {
            StmtKind::Expr(expr) => {
                self.user_type_result_with_params_and_aliases(expr, param_exprs, &aliases)
            }
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => Self::merge_lowered_user_type_array_results([
                self.user_type_branch_result_with_params_and_aliases(
                    then_branch,
                    param_exprs,
                    &aliases,
                ),
                self.user_type_branch_result_with_params_and_aliases(
                    else_branch,
                    param_exprs,
                    &aliases,
                ),
            ]),
            StmtKind::For { body, .. }
            | StmtKind::ForIn { body, .. }
            | StmtKind::While { body, .. } => {
                self.user_type_branch_result_with_params_and_aliases(body, param_exprs, &aliases)
            }
            _ => LoweredUserTypeArrayResult::Unknown,
        }
    }

    fn user_type_switch_result_with_params_and_aliases(
        &self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, LoweredUserTypeArrayResult>,
    ) -> LoweredUserTypeArrayResult {
        match result {
            SwitchArmResult::Expr(expr) => {
                self.user_type_result_with_params_and_aliases(expr, param_exprs, aliases)
            }
            SwitchArmResult::Block(statements) => self
                .user_type_branch_result_with_params_and_aliases(statements, param_exprs, aliases),
        }
    }
}
