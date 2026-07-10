use std::collections::HashMap;

use pine_ir::{
    HirBinaryOp, HirExpr, HirExprKind, HirLiteral, HirStmt, HirStmtKind, HirSwitchArm,
    HirSwitchStmtArm, HirUnaryOp, SymbolId,
};

pub(crate) type ConstSymbolEnv<'a> = HashMap<SymbolId, &'a HirExpr>;

pub(crate) fn constant_hir_int_with_symbols(
    expr: &HirExpr,
    env: &ConstSymbolEnv<'_>,
) -> Option<i64> {
    constant_hir_int_with_env(expr, Some(env), &mut Vec::new())
}

fn constant_hir_int_with_env(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<i64> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Int(value)) => Some(*value),
        HirExprKind::Builtin(name) => pine_builtins::named_int_constant(name),
        HirExprKind::Symbol(symbol) => {
            with_symbol_value(*symbol, env, visiting, constant_hir_int_with_env)
        }
        HirExprKind::Unary {
            op: HirUnaryOp::Plus,
            expr,
        } => constant_hir_int_with_env(expr, env, visiting),
        HirExprKind::Unary {
            op: HirUnaryOp::Minus,
            expr,
        } => constant_hir_int_with_env(expr, env, visiting).and_then(i64::checked_neg),
        HirExprKind::Binary {
            op: HirBinaryOp::Add,
            left,
            right,
        } => constant_hir_int_with_env(left, env, visiting)?
            .checked_add(constant_hir_int_with_env(right, env, visiting)?),
        HirExprKind::Binary {
            op: HirBinaryOp::Sub,
            left,
            right,
        } => constant_hir_int_with_env(left, env, visiting)?
            .checked_sub(constant_hir_int_with_env(right, env, visiting)?),
        HirExprKind::Binary {
            op: HirBinaryOp::Mul,
            left,
            right,
        } => constant_hir_int_with_env(left, env, visiting)?
            .checked_mul(constant_hir_int_with_env(right, env, visiting)?),
        HirExprKind::Binary {
            op: HirBinaryOp::Mod,
            left,
            right,
        } => constant_hir_int_with_env(left, env, visiting)?
            .checked_rem(constant_hir_int_with_env(right, env, visiting)?),
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => constant_hir_int_with_env(then_expr, env, visiting),
            Some(false) => constant_hir_int_with_env(else_expr, env, visiting),
            None => {
                let then_value = constant_hir_int_with_env(then_expr, env, visiting)?;
                let else_value = constant_hir_int_with_env(else_expr, env, visiting)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_int_with_env(result, env, visiting);
            };
            constant_hir_int_with_env(result, Some(&block_env), visiting)
        }
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            constant_hir_int_with_env,
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            constant_hir_int_with_env,
        ),
        HirExprKind::FieldAccess { value, index } => constant_hir_field_value_with_env(
            value,
            *index,
            env,
            visiting,
            constant_hir_int_with_env,
        ),
        _ => None,
    }
}

fn with_symbol_value<T>(
    symbol: SymbolId,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
    value_of: fn(&HirExpr, Option<&ConstSymbolEnv<'_>>, &mut Vec<SymbolId>) -> Option<T>,
) -> Option<T> {
    if visiting.contains(&symbol) {
        return None;
    }
    let value = env?.get(&symbol)?;
    visiting.push(symbol);
    let result = value_of(value, env, visiting);
    visiting.pop();
    result
}

fn constant_hir_bool_with_env(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<bool> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Bool(value)) => Some(*value),
        HirExprKind::Symbol(symbol) => {
            with_symbol_value(*symbol, env, visiting, constant_hir_bool_with_env)
        }
        HirExprKind::Unary {
            op: HirUnaryOp::Not,
            expr,
        } => constant_hir_bool_with_env(expr, env, visiting).map(|value| !value),
        HirExprKind::Binary {
            op: HirBinaryOp::And,
            left,
            right,
        } => match constant_hir_bool_with_env(left, env, visiting)? {
            false => Some(false),
            true => constant_hir_bool_with_env(right, env, visiting),
        },
        HirExprKind::Binary {
            op: HirBinaryOp::Or,
            left,
            right,
        } => match constant_hir_bool_with_env(left, env, visiting)? {
            true => Some(true),
            false => constant_hir_bool_with_env(right, env, visiting),
        },
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => constant_hir_bool_with_env(then_expr, env, visiting),
            Some(false) => constant_hir_bool_with_env(else_expr, env, visiting),
            None => {
                let then_value = constant_hir_bool_with_env(then_expr, env, visiting)?;
                let else_value = constant_hir_bool_with_env(else_expr, env, visiting)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_bool_with_env(result, env, visiting);
            };
            constant_hir_bool_with_env(result, Some(&block_env), visiting)
        }
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            constant_hir_bool_with_env,
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            constant_hir_bool_with_env,
        ),
        HirExprKind::FieldAccess { value, index } => constant_hir_field_value_with_env(
            value,
            *index,
            env,
            visiting,
            constant_hir_bool_with_env,
        ),
        HirExprKind::Binary {
            op:
                op @ (HirBinaryOp::Eq
                | HirBinaryOp::NotEq
                | HirBinaryOp::Gt
                | HirBinaryOp::Gte
                | HirBinaryOp::Lt
                | HirBinaryOp::Lte),
            left,
            right,
        } => constant_hir_numeric_comparison_with_env(*op, left, right, env, visiting)
            .or_else(|| {
                constant_hir_bool_comparison(
                    *op,
                    constant_hir_bool_with_env(left, env, visiting)?,
                    constant_hir_bool_with_env(right, env, visiting)?,
                )
            })
            .or_else(|| {
                constant_hir_string_comparison(
                    *op,
                    &constant_hir_string_with_env(left, env, visiting)?,
                    &constant_hir_string_with_env(right, env, visiting)?,
                )
            })
            .or_else(|| {
                constant_hir_color_comparison(
                    *op,
                    constant_hir_color_with_env(left, env, visiting)?,
                    constant_hir_color_with_env(right, env, visiting)?,
                )
            }),
        _ => None,
    }
}

fn constant_hir_numeric_with_env(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<f64> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Int(value)) => Some(*value as f64),
        HirExprKind::Literal(HirLiteral::Float(value)) => Some(*value),
        HirExprKind::Builtin(name) => named_hir_numeric_constant(name),
        HirExprKind::Symbol(symbol) => {
            with_symbol_value(*symbol, env, visiting, constant_hir_numeric_with_env)
        }
        HirExprKind::Unary {
            op: HirUnaryOp::Plus,
            expr,
        } => constant_hir_numeric_with_env(expr, env, visiting),
        HirExprKind::Unary {
            op: HirUnaryOp::Minus,
            expr,
        } => constant_hir_numeric_with_env(expr, env, visiting).map(|value| -value),
        HirExprKind::Binary {
            op: HirBinaryOp::Add,
            left,
            right,
        } => Some(
            constant_hir_numeric_with_env(left, env, visiting)?
                + constant_hir_numeric_with_env(right, env, visiting)?,
        ),
        HirExprKind::Binary {
            op: HirBinaryOp::Sub,
            left,
            right,
        } => Some(
            constant_hir_numeric_with_env(left, env, visiting)?
                - constant_hir_numeric_with_env(right, env, visiting)?,
        ),
        HirExprKind::Binary {
            op: HirBinaryOp::Mul,
            left,
            right,
        } => Some(
            constant_hir_numeric_with_env(left, env, visiting)?
                * constant_hir_numeric_with_env(right, env, visiting)?,
        ),
        HirExprKind::Binary {
            op: HirBinaryOp::Div,
            left,
            right,
        } => finite_hir_numeric(
            constant_hir_numeric_with_env(left, env, visiting)?
                / constant_hir_numeric_with_env(right, env, visiting)?,
        ),
        HirExprKind::Binary {
            op: HirBinaryOp::Mod,
            left,
            right,
        } => finite_hir_numeric(
            constant_hir_numeric_with_env(left, env, visiting)?
                % constant_hir_numeric_with_env(right, env, visiting)?,
        ),
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => constant_hir_numeric_with_env(then_expr, env, visiting),
            Some(false) => constant_hir_numeric_with_env(else_expr, env, visiting),
            None => {
                let then_value = constant_hir_numeric_with_env(then_expr, env, visiting)?;
                let else_value = constant_hir_numeric_with_env(else_expr, env, visiting)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_numeric_with_env(result, env, visiting);
            };
            constant_hir_numeric_with_env(result, Some(&block_env), visiting)
        }
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            constant_hir_numeric_with_env,
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            constant_hir_numeric_with_env,
        ),
        HirExprKind::FieldAccess { value, index } => constant_hir_field_value_with_env(
            value,
            *index,
            env,
            visiting,
            constant_hir_numeric_with_env,
        ),
        _ => None,
    }
}

fn named_hir_numeric_constant(name: &str) -> Option<f64> {
    pine_builtins::named_float_constant(name)
        .or_else(|| pine_builtins::named_int_constant(name).map(|value| value as f64))
}

fn finite_hir_numeric(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn block_const_symbol_env<'a>(
    statements: &'a [HirStmt],
    outer: Option<&ConstSymbolEnv<'a>>,
) -> Option<ConstSymbolEnv<'a>> {
    let mut env = outer.cloned().unwrap_or_default();
    for statement in statements {
        match &statement.kind {
            HirStmtKind::Expr(_) => {}
            HirStmtKind::Decl { symbol, value } => {
                env.insert(*symbol, value);
            }
            HirStmtKind::TupleDecl { symbols, value } => {
                let HirExprKind::Tuple(values) = &value.kind else {
                    return None;
                };
                if symbols.len() != values.len() {
                    return None;
                }
                for (symbol, value) in symbols.iter().zip(values) {
                    env.insert(*symbol, value);
                }
            }
            HirStmtKind::Reassign { symbol, .. } | HirStmtKind::FieldReassign { symbol, .. } => {
                env.remove(symbol);
            }
            HirStmtKind::ArrayFieldReassign { array, .. } => {
                if let HirExprKind::Symbol(symbol) = array.kind {
                    env.remove(&symbol);
                }
            }
            statement @ (HirStmtKind::If { .. }
            | HirStmtKind::Switch { .. }
            | HirStmtKind::For { .. }
            | HirStmtKind::ForIn { .. }
            | HirStmtKind::While { .. }) => {
                remove_reassigned_symbols_from_env(&mut env, statement);
            }
            HirStmtKind::Break | HirStmtKind::Continue => return None,
        }
    }
    Some(env)
}

pub(crate) fn remove_reassigned_symbols_from_env(
    env: &mut ConstSymbolEnv<'_>,
    statement: &HirStmtKind,
) {
    match statement {
        HirStmtKind::Reassign { symbol, .. } | HirStmtKind::FieldReassign { symbol, .. } => {
            env.remove(symbol);
        }
        HirStmtKind::ArrayFieldReassign { array, .. } => {
            if let HirExprKind::Symbol(symbol) = array.kind {
                env.remove(&symbol);
            }
        }
        HirStmtKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            for statement in then_branch.iter().chain(else_branch) {
                remove_reassigned_symbols_from_env(env, &statement.kind);
            }
        }
        HirStmtKind::Switch { arms, .. } => {
            for statement in switch_stmt_arm_bodies(arms) {
                remove_reassigned_symbols_from_env(env, &statement.kind);
            }
        }
        HirStmtKind::For { counter, body, .. } => {
            env.remove(counter);
            for statement in body {
                remove_reassigned_symbols_from_env(env, &statement.kind);
            }
        }
        HirStmtKind::ForIn {
            index, value, body, ..
        } => {
            if let Some(index) = index {
                env.remove(index);
            }
            env.remove(value);
            for statement in body {
                remove_reassigned_symbols_from_env(env, &statement.kind);
            }
        }
        HirStmtKind::While { body, .. } => {
            for statement in body {
                remove_reassigned_symbols_from_env(env, &statement.kind);
            }
        }
        HirStmtKind::Expr(_)
        | HirStmtKind::Decl { .. }
        | HirStmtKind::TupleDecl { .. }
        | HirStmtKind::Break
        | HirStmtKind::Continue => {}
    }
}

fn constant_hir_string_with_env(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<String> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::String(value)) => Some(value.clone()),
        HirExprKind::Builtin(name) => pine_builtins::named_string_constant(name).map(str::to_owned),
        HirExprKind::Symbol(symbol) => {
            with_symbol_value(*symbol, env, visiting, constant_hir_string_with_env)
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => constant_hir_string_with_env(then_expr, env, visiting),
            Some(false) => constant_hir_string_with_env(else_expr, env, visiting),
            None => {
                let then_value = constant_hir_string_with_env(then_expr, env, visiting)?;
                let else_value = constant_hir_string_with_env(else_expr, env, visiting)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_string_with_env(result, env, visiting);
            };
            constant_hir_string_with_env(result, Some(&block_env), visiting)
        }
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            constant_hir_string_with_env,
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            constant_hir_string_with_env,
        ),
        HirExprKind::FieldAccess { value, index } => constant_hir_field_value_with_env(
            value,
            *index,
            env,
            visiting,
            constant_hir_string_with_env,
        ),
        _ => None,
    }
}

fn constant_hir_color_with_env(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<u32> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::ColorHex(value)) => parse_color_hex(value),
        HirExprKind::Builtin(name) => pine_builtins::named_color(name),
        HirExprKind::Symbol(symbol) => {
            with_symbol_value(*symbol, env, visiting, constant_hir_color_with_env)
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => constant_hir_color_with_env(then_expr, env, visiting),
            Some(false) => constant_hir_color_with_env(else_expr, env, visiting),
            None => {
                let then_value = constant_hir_color_with_env(then_expr, env, visiting)?;
                let else_value = constant_hir_color_with_env(else_expr, env, visiting)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_color_with_env(result, env, visiting);
            };
            constant_hir_color_with_env(result, Some(&block_env), visiting)
        }
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            constant_hir_color_with_env,
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            constant_hir_color_with_env,
        ),
        HirExprKind::FieldAccess { value, index } => constant_hir_field_value_with_env(
            value,
            *index,
            env,
            visiting,
            constant_hir_color_with_env,
        ),
        _ => None,
    }
}

fn constant_hir_field_value_with_env<T>(
    value: &HirExpr,
    index: usize,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
    value_of: fn(&HirExpr, Option<&ConstSymbolEnv<'_>>, &mut Vec<SymbolId>) -> Option<T>,
) -> Option<T>
where
    T: PartialEq,
{
    match &value.kind {
        HirExprKind::UserTypeConstruct { fields, .. } => {
            value_of(fields.get(index)?, env, visiting)
        }
        HirExprKind::Symbol(symbol) => {
            if visiting.contains(symbol) {
                return None;
            }
            let value = env?.get(symbol)?;
            visiting.push(*symbol);
            let result = constant_hir_field_value_with_env(value, index, env, visiting, value_of);
            visiting.pop();
            result
        }
        HirExprKind::FieldAccess {
            value,
            index: inner_index,
        } => {
            let value = constant_hir_field_expr_with_env(value, *inner_index, env, visiting)?;
            constant_hir_field_value_with_env(value, index, env, visiting, value_of)
        }
        HirExprKind::Block { statements, result } => {
            let Some(block_env) = block_const_symbol_env(statements, env) else {
                return constant_hir_field_value_with_env(result, index, env, visiting, value_of);
            };
            constant_hir_field_value_with_env(result, index, Some(&block_env), visiting, value_of)
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => match constant_hir_bool_with_env(condition, env, visiting) {
            Some(true) => {
                constant_hir_field_value_with_env(then_expr, index, env, visiting, value_of)
            }
            Some(false) => {
                constant_hir_field_value_with_env(else_expr, index, env, visiting, value_of)
            }
            None => {
                let then_value =
                    constant_hir_field_value_with_env(then_expr, index, env, visiting, value_of)?;
                let else_value =
                    constant_hir_field_value_with_env(else_expr, index, env, visiting, value_of)?;
                (then_value == else_value).then_some(then_value)
            }
        },
        HirExprKind::Switch { selector, arms } => constant_switch_result_with_env(
            selector.as_deref(),
            arms,
            env,
            visiting,
            |value, env, visiting| {
                constant_hir_field_value_with_env(value, index, env, visiting, value_of)
            },
        ),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => constant_for_result_with_env(
            HirForConstParts {
                from,
                to,
                step: step.as_deref(),
                statements,
                result,
            },
            env,
            visiting,
            |value, env, visiting| {
                constant_hir_field_value_with_env(value, index, env, visiting, value_of)
            },
        ),
        _ => None,
    }
}

fn constant_hir_field_expr_with_env<'a>(
    value: &'a HirExpr,
    index: usize,
    env: Option<&ConstSymbolEnv<'a>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<&'a HirExpr> {
    match &value.kind {
        HirExprKind::UserTypeConstruct { fields, .. } => fields.get(index),
        HirExprKind::Symbol(symbol) => {
            if visiting.contains(symbol) {
                return None;
            }
            let value = env?.get(symbol)?;
            visiting.push(*symbol);
            let result = constant_hir_field_expr_with_env(value, index, env, visiting);
            visiting.pop();
            result
        }
        HirExprKind::FieldAccess {
            value,
            index: inner_index,
        } => {
            let value = constant_hir_field_expr_with_env(value, *inner_index, env, visiting)?;
            constant_hir_field_expr_with_env(value, index, env, visiting)
        }
        _ => None,
    }
}

struct HirForConstParts<'a> {
    from: &'a HirExpr,
    to: &'a HirExpr,
    step: Option<&'a HirExpr>,
    statements: &'a [HirStmt],
    result: &'a HirExpr,
}

fn constant_for_result_with_env<T, F>(
    parts: HirForConstParts<'_>,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
    value_of: F,
) -> Option<T>
where
    F: Fn(&HirExpr, Option<&ConstSymbolEnv<'_>>, &mut Vec<SymbolId>) -> Option<T> + Copy,
{
    constant_hir_int_with_env(parts.from, env, visiting)?;
    constant_hir_int_with_env(parts.to, env, visiting)?;
    if let Some(step) = parts.step
        && constant_hir_int_with_env(step, env, visiting)? == 0
    {
        return None;
    }
    if contains_loop_control(parts.statements) {
        return None;
    }
    let block_env = block_const_symbol_env(parts.statements, env)?;
    value_of(parts.result, Some(&block_env), visiting)
}

fn contains_loop_control(statements: &[HirStmt]) -> bool {
    statements
        .iter()
        .any(|statement| statement_contains_loop_control(&statement.kind))
}

fn switch_stmt_arm_bodies(arms: &[HirSwitchStmtArm]) -> impl Iterator<Item = &HirStmt> {
    arms.iter().flat_map(|arm| arm.body.iter())
}

fn statement_contains_loop_control(statement: &HirStmtKind) -> bool {
    match statement {
        HirStmtKind::Break | HirStmtKind::Continue => true,
        HirStmtKind::If {
            then_branch,
            else_branch,
            ..
        } => contains_loop_control(then_branch) || contains_loop_control(else_branch),
        HirStmtKind::For { body, .. }
        | HirStmtKind::ForIn { body, .. }
        | HirStmtKind::While { body, .. } => contains_loop_control(body),
        HirStmtKind::Switch { arms, .. } => switch_stmt_arm_bodies(arms)
            .any(|statement| statement_contains_loop_control(&statement.kind)),
        HirStmtKind::Expr(_)
        | HirStmtKind::Decl { .. }
        | HirStmtKind::Reassign { .. }
        | HirStmtKind::FieldReassign { .. }
        | HirStmtKind::ArrayFieldReassign { .. }
        | HirStmtKind::TupleDecl { .. } => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ConstSwitchValue {
    Numeric(f64),
    Bool(bool),
    String(String),
    Color(u32),
}

fn constant_hir_switch_value(
    expr: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<ConstSwitchValue> {
    constant_hir_bool_with_env(expr, env, visiting)
        .map(ConstSwitchValue::Bool)
        .or_else(|| constant_hir_string_with_env(expr, env, visiting).map(ConstSwitchValue::String))
        .or_else(|| constant_hir_color_with_env(expr, env, visiting).map(ConstSwitchValue::Color))
        .or_else(|| {
            constant_hir_numeric_with_env(expr, env, visiting).map(ConstSwitchValue::Numeric)
        })
}

fn constant_switch_values_equal(
    left: &ConstSwitchValue,
    right: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<bool> {
    let right = constant_hir_switch_value(right, env, visiting)?;
    Some(match (left, right) {
        (ConstSwitchValue::Numeric(left), ConstSwitchValue::Numeric(right)) => *left == right,
        (ConstSwitchValue::Bool(left), ConstSwitchValue::Bool(right)) => *left == right,
        (ConstSwitchValue::String(left), ConstSwitchValue::String(right)) => *left == right,
        (ConstSwitchValue::Color(left), ConstSwitchValue::Color(right)) => *left == right,
        _ => false,
    })
}

fn constant_switch_result_with_env<T, F>(
    selector: Option<&HirExpr>,
    arms: &[HirSwitchArm],
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
    value_of: F,
) -> Option<T>
where
    T: PartialEq,
    F: Fn(&HirExpr, Option<&ConstSymbolEnv<'_>>, &mut Vec<SymbolId>) -> Option<T> + Copy,
{
    if let Some(selector) = selector {
        let Some(selector_value) = constant_hir_switch_value(selector, env, visiting) else {
            return constant_all_switch_results_with_default(arms, env, visiting, value_of);
        };
        for (index, arm) in arms.iter().enumerate() {
            match &arm.condition {
                Some(condition) => {
                    match constant_switch_values_equal(&selector_value, condition, env, visiting) {
                        Some(true) => return value_of(&arm.result, env, visiting),
                        Some(false) => {}
                        None => {
                            return constant_all_switch_results_with_default(
                                &arms[index..],
                                env,
                                visiting,
                                value_of,
                            );
                        }
                    }
                }
                None => return value_of(&arm.result, env, visiting),
            }
        }
        return None;
    }

    for (index, arm) in arms.iter().enumerate() {
        match &arm.condition {
            Some(condition) => match constant_hir_bool_with_env(condition, env, visiting) {
                Some(true) => return value_of(&arm.result, env, visiting),
                Some(false) => {}
                None => {
                    return constant_all_switch_results_with_default(
                        &arms[index..],
                        env,
                        visiting,
                        value_of,
                    );
                }
            },
            None => return value_of(&arm.result, env, visiting),
        }
    }
    None
}

fn constant_all_switch_results_with_default<T, F>(
    arms: &[HirSwitchArm],
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
    value_of: F,
) -> Option<T>
where
    T: PartialEq,
    F: Fn(&HirExpr, Option<&ConstSymbolEnv<'_>>, &mut Vec<SymbolId>) -> Option<T> + Copy,
{
    if !arms.iter().any(|arm| arm.condition.is_none()) {
        return None;
    }
    let mut expected = None;
    for arm in arms {
        let value = value_of(&arm.result, env, visiting)?;
        match &expected {
            Some(expected) if *expected != value => return None,
            Some(_) => {}
            None => expected = Some(value),
        }
    }
    expected
}

fn parse_color_hex(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches('#'), 16).ok()
}

fn constant_hir_numeric_comparison_with_env(
    op: HirBinaryOp,
    left: &HirExpr,
    right: &HirExpr,
    env: Option<&ConstSymbolEnv<'_>>,
    visiting: &mut Vec<SymbolId>,
) -> Option<bool> {
    let left = constant_hir_numeric_with_env(left, env, visiting)?;
    let right = constant_hir_numeric_with_env(right, env, visiting)?;
    Some(match op {
        HirBinaryOp::Eq => left == right,
        HirBinaryOp::NotEq => left != right,
        HirBinaryOp::Gt => left > right,
        HirBinaryOp::Gte => left >= right,
        HirBinaryOp::Lt => left < right,
        HirBinaryOp::Lte => left <= right,
        _ => return None,
    })
}

fn constant_hir_string_comparison(op: HirBinaryOp, left: &str, right: &str) -> Option<bool> {
    Some(match op {
        HirBinaryOp::Eq => left == right,
        HirBinaryOp::NotEq => left != right,
        _ => return None,
    })
}

fn constant_hir_color_comparison(op: HirBinaryOp, left: u32, right: u32) -> Option<bool> {
    Some(match op {
        HirBinaryOp::Eq => left == right,
        HirBinaryOp::NotEq => left != right,
        _ => return None,
    })
}

fn constant_hir_bool_comparison(op: HirBinaryOp, left: bool, right: bool) -> Option<bool> {
    Some(match op {
        HirBinaryOp::Eq => left == right,
        HirBinaryOp::NotEq => left != right,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_ir::{PineType, Qualifier, ValueKind};

    fn const_int_expr(kind: HirExprKind) -> HirExpr {
        HirExpr {
            kind,
            pine_type: PineType::new(Qualifier::Const, ValueKind::Int),
            series_id: None,
        }
    }

    #[test]
    fn resolves_alias_chain_int_symbols() {
        let base_symbol = SymbolId(1);
        let base = const_int_expr(HirExprKind::Literal(HirLiteral::Int(8)));
        let length = const_int_expr(HirExprKind::Binary {
            op: HirBinaryOp::Add,
            left: Box::new(const_int_expr(HirExprKind::Symbol(base_symbol))),
            right: Box::new(const_int_expr(HirExprKind::Literal(HirLiteral::Int(2)))),
        });
        let mut env = ConstSymbolEnv::new();
        env.insert(base_symbol, &base);

        assert_eq!(constant_hir_int_with_symbols(&length, &env), Some(10));
    }

    #[test]
    fn returns_none_for_cyclic_int_symbols() {
        let first_symbol = SymbolId(1);
        let second_symbol = SymbolId(2);
        let first = const_int_expr(HirExprKind::Symbol(second_symbol));
        let second = const_int_expr(HirExprKind::Symbol(first_symbol));
        let mut env = ConstSymbolEnv::new();
        env.insert(first_symbol, &first);
        env.insert(second_symbol, &second);

        assert_eq!(constant_hir_int_with_symbols(&first, &env), None);
    }
}
