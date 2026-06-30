use std::collections::HashMap;

use crate::prelude::*;

use super::prepend_block_statements;

fn branch_return_expr(branch: &[Stmt]) -> Option<(&[Stmt], &Expr)> {
    let (last, prefix) = branch.split_last()?;
    let StmtKind::Expr(expr) = &last.kind else {
        return None;
    };
    Some((prefix, expr))
}

fn for_statement_expr(
    counter: &str,
    from: &Expr,
    to: &Expr,
    step: &Option<Expr>,
    body: &[Stmt],
    span: Span,
) -> Expr {
    Expr {
        span,
        kind: ExprKind::For {
            counter: counter.to_owned(),
            from: Box::new(from.clone()),
            to: Box::new(to.clone()),
            step: step.clone().map(Box::new),
            body: body.to_vec(),
        },
    }
}

impl Analyzer {
    pub(crate) fn lower_function_body(
        &mut self,
        body: &FunctionBody,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        match body {
            FunctionBody::Expr(expr) => {
                self.lower_symbol_overrides.push(HashMap::new());
                let result = self.lower_expr_with_params(expr, param_exprs, param_types);
                self.lower_symbol_overrides.pop();
                result
            }
            FunctionBody::Block(statements) => {
                let (last, prefix) = statements.split_last()?;
                let result = match &last.kind {
                    StmtKind::Expr(result) => result,
                    StmtKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    } if branch_return_expr(then_branch).is_some()
                        && branch_return_expr(else_branch).is_some() =>
                    {
                        return self.lower_function_if_return(
                            prefix,
                            condition,
                            then_branch,
                            else_branch,
                            param_exprs,
                            param_types,
                        );
                    }
                    StmtKind::For {
                        counter,
                        from,
                        to,
                        step,
                        body,
                    } => {
                        let expr = for_statement_expr(counter, from, to, step, body, last.span);
                        return self.lower_function_return_expr(
                            prefix,
                            &expr,
                            param_exprs,
                            param_types,
                        );
                    }
                    _ => return None,
                };
                self.lower_function_return_expr(prefix, result, param_exprs, param_types)
            }
        }
    }

    fn lower_function_return_expr(
        &mut self,
        prefix: &[Stmt],
        result: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        self.lower_symbol_overrides.push(HashMap::new());
        let lowered_statements = prefix
            .iter()
            .map(|statement| self.lower_stmt_with_params(statement, param_exprs, param_types))
            .collect::<Option<Vec<_>>>();
        let result = lowered_statements.and_then(|statements| {
            Some((
                statements,
                self.lower_expr_with_params(result, param_exprs, param_types)?,
            ))
        });
        self.lower_symbol_overrides.pop();
        let (statements, result) = result?;
        Some(HirExpr {
            pine_type: result.pine_type,
            series_id: result.series_id,
            kind: HirExprKind::Block {
                statements,
                result: Box::new(result),
            },
        })
    }

    pub(super) fn lower_expr_branch_return(
        &mut self,
        branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let (last, prefix) = branch.split_last()?;
        let StmtKind::Expr(result) = &last.kind else {
            return None;
        };
        let statements = prefix
            .iter()
            .map(|statement| self.lower_stmt_with_params(statement, param_exprs, param_types))
            .collect::<Option<Vec<_>>>()?;
        let result = self.lower_expr_with_params(result, param_exprs, param_types)?;
        if statements.is_empty() {
            Some(result)
        } else {
            Some(prepend_block_statements(statements, result))
        }
    }

    pub(super) fn lower_switch_arm_result_with_params(
        &mut self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        match result {
            SwitchArmResult::Expr(expr) => {
                self.lower_expr_with_params(expr, param_exprs, param_types)
            }
            SwitchArmResult::Block(statements) => {
                self.lower_expr_branch_return(statements, param_exprs, param_types)
            }
        }
    }

    fn lower_function_if_return(
        &mut self,
        prefix: &[Stmt],
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        self.lower_symbol_overrides.push(HashMap::new());
        let lowered_statements = prefix
            .iter()
            .map(|statement| self.lower_stmt_with_params(statement, param_exprs, param_types))
            .collect::<Option<Vec<_>>>();
        let condition_expr = self.lower_expr_with_params(condition, param_exprs, param_types);
        let then_expr = self.lower_function_branch_return(then_branch, param_exprs, param_types);
        let else_expr = self.lower_function_branch_return(else_branch, param_exprs, param_types);
        self.lower_symbol_overrides.pop();

        let condition_expr = condition_expr?;
        let then_expr = then_expr?;
        let else_expr = else_expr?;
        let pine_type = PineType::new(
            strongest_qualifier(
                condition_expr.pine_type.qualifier,
                strongest_qualifier(then_expr.pine_type.qualifier, else_expr.pine_type.qualifier),
            ),
            common_kind(then_expr.pine_type.kind, else_expr.pine_type.kind)?,
        );
        let series_id =
            if pine_type.qualifier == Qualifier::Series && pine_type.kind != ValueKind::Tuple {
                Some(self.alloc_series())
            } else {
                None
            };
        let result = HirExpr {
            pine_type,
            series_id,
            kind: HirExprKind::Ternary {
                condition: Box::new(condition_expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            },
        };
        Some(prepend_block_statements(lowered_statements?, result))
    }

    fn lower_function_branch_return(
        &mut self,
        branch: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let (prefix, result) = branch_return_expr(branch)?;
        let lowered_statements = prefix
            .iter()
            .map(|statement| self.lower_stmt_with_params(statement, param_exprs, param_types))
            .collect::<Option<Vec<_>>>()?;
        let result = self.lower_expr_with_params(result, param_exprs, param_types)?;
        if lowered_statements.is_empty() {
            Some(result)
        } else {
            Some(prepend_block_statements(lowered_statements, result))
        }
    }
}
