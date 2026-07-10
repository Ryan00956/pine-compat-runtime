use super::*;

impl Analyzer {
    pub(super) fn user_type_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        self.user_type_name_of_expr_with_params_and_aliases(expr, param_exprs, &HashMap::new())
    }

    pub(super) fn user_type_array_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        if let Some(type_name) = self.user_type_array_name_of_expr(expr) {
            return Some(type_name);
        }
        if let ExprKind::History { expr, .. } = &expr.kind {
            return self.user_type_array_name_of_expr_with_params(expr, param_exprs);
        }
        let name = match &expr.kind {
            ExprKind::Identifier(name) => name,
            ExprKind::QualifiedName(parts) if parts.len() == 1 => &parts[0],
            _ => return None,
        };
        let HirExprKind::Symbol(symbol_id) = param_exprs.get(name)?.kind else {
            return None;
        };
        self.symbol_user_type_arrays.get(&symbol_id).cloned()
    }

    fn user_type_name_of_expr_with_params_and_aliases(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(type_name) = self.user_type_name_of_expr(expr) {
            return Some(type_name);
        }
        if let ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } = &expr.kind
        {
            return match (
                self.user_type_name_of_expr_with_params_and_aliases(
                    then_expr,
                    param_exprs,
                    aliases,
                ),
                self.user_type_name_of_expr_with_params_and_aliases(
                    else_expr,
                    param_exprs,
                    aliases,
                ),
            ) {
                (Some(then_name), Some(else_name)) if then_name == else_name => Some(then_name),
                _ => None,
            };
        }
        if let ExprKind::Switch { arms, .. } = &expr.kind {
            let mut resolved_type_name = None;
            for arm in arms {
                match self.user_type_name_of_switch_arm_result_with_params_and_aliases(
                    &arm.result,
                    param_exprs,
                    aliases,
                ) {
                    Some(type_name) => match &resolved_type_name {
                        Some(resolved) if resolved != &type_name => return None,
                        Some(_) => {}
                        None => resolved_type_name = Some(type_name),
                    },
                    None => return None,
                }
            }
            return resolved_type_name;
        }
        if let ExprKind::For { body, .. } = &expr.kind {
            let (last, prefix) = body.split_last()?;
            let StmtKind::Expr(result) = &last.kind else {
                return None;
            };
            let aliases = self.local_user_type_param_aliases(prefix, param_exprs, aliases);
            return self.user_type_name_of_expr_with_params_and_aliases(
                result,
                param_exprs,
                &aliases,
            );
        }
        if let ExprKind::ForIn { body, .. } = &expr.kind {
            let (last, prefix) = body.split_last()?;
            let StmtKind::Expr(result) = &last.kind else {
                return None;
            };
            let aliases = self.local_user_type_param_aliases(prefix, param_exprs, aliases);
            return self.user_type_name_of_expr_with_params_and_aliases(
                result,
                param_exprs,
                &aliases,
            );
        }
        let name = match &expr.kind {
            ExprKind::Identifier(name) => name,
            ExprKind::QualifiedName(parts) if parts.len() == 1 => &parts[0],
            _ => return None,
        };
        if let Some(type_name) = aliases.get(name) {
            return Some(type_name.clone());
        }
        let HirExprKind::Symbol(symbol_id) = param_exprs.get(name)?.kind else {
            return None;
        };
        self.symbol_user_types.get(&symbol_id).cloned()
    }

    fn local_user_type_param_aliases(
        &self,
        prefix: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        outer_aliases: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut aliases = outer_aliases.clone();
        for statement in prefix {
            if let StmtKind::Decl { name, value, .. } = &statement.kind
                && let Some(type_name) = self.user_type_name_of_expr_with_params_and_aliases(
                    value,
                    param_exprs,
                    &aliases,
                )
            {
                aliases.insert(name.clone(), type_name);
            }
        }
        aliases
    }

    fn user_type_name_of_switch_arm_result_with_params_and_aliases(
        &self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, String>,
    ) -> Option<String> {
        match result {
            SwitchArmResult::Expr(expr) => {
                self.user_type_name_of_expr_with_params_and_aliases(expr, param_exprs, aliases)
            }
            SwitchArmResult::Block(statements) => {
                let (last, prefix) = statements.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                let aliases = self.local_user_type_param_aliases(prefix, param_exprs, aliases);
                self.user_type_name_of_expr_with_params_and_aliases(result, param_exprs, &aliases)
            }
        }
    }
}
