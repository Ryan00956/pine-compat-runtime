use std::collections::HashMap;

use pine_syntax::{
    CallArg, ExportDecl, ExportItem, Expr, ExprKind, FunctionBody, Program, Stmt, StmtKind,
    SwitchArm,
};

use crate::analyzer::calls::expr_name;

#[derive(Default)]
pub(super) struct RewriteContext {
    pub(super) constants: HashMap<String, Expr>,
    pub(super) function_targets: HashMap<String, String>,
}

pub(super) fn rewrite_program(program: &Program, context: &RewriteContext) -> Program {
    Program {
        version: program.version,
        statements: program
            .statements
            .iter()
            .map(|statement| rewrite_stmt(statement, context))
            .collect(),
    }
}

fn rewrite_stmt(statement: &Stmt, context: &RewriteContext) -> Stmt {
    let kind = match &statement.kind {
        StmtKind::Expr(expr) => StmtKind::Expr(rewrite_expr(expr, context)),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => StmtKind::If {
            condition: rewrite_expr(condition, context),
            then_branch: then_branch
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
            else_branch: else_branch
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
        },
        StmtKind::For {
            counter,
            from,
            to,
            step,
            body,
        } => StmtKind::For {
            counter: counter.clone(),
            from: rewrite_expr(from, context),
            to: rewrite_expr(to, context),
            step: step.as_ref().map(|step| rewrite_expr(step, context)),
            body: body
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition: rewrite_expr(condition, context),
            body: body
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
        },
        StmtKind::Decl { mode, name, value } => StmtKind::Decl {
            mode: *mode,
            name: name.clone(),
            value: rewrite_expr(value, context),
        },
        StmtKind::Reassign { name, value } => StmtKind::Reassign {
            name: name.clone(),
            value: rewrite_expr(value, context),
        },
        StmtKind::TupleDecl { names, value } => StmtKind::TupleDecl {
            names: names.clone(),
            value: rewrite_expr(value, context),
        },
        StmtKind::Export(export) => {
            let item = match &export.item {
                ExportItem::Function {
                    name,
                    params,
                    body,
                    span,
                } => ExportItem::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: rewrite_function_body(body, context),
                    span: *span,
                },
                ExportItem::Const { name, value, span } => ExportItem::Const {
                    name: name.clone(),
                    value: rewrite_expr(value, context),
                    span: *span,
                },
                ExportItem::Unknown { span } => ExportItem::Unknown { span: *span },
            };
            StmtKind::Export(ExportDecl { item })
        }
        StmtKind::Function { name, params, body } => StmtKind::Function {
            name: name.clone(),
            params: params.clone(),
            body: rewrite_function_body(body, context),
        },
        StmtKind::Import(_)
        | StmtKind::Library(_)
        | StmtKind::UserType(_)
        | StmtKind::Method(_)
        | StmtKind::Break
        | StmtKind::Continue
        | StmtKind::Unsupported { .. } => statement.kind.clone(),
    };
    Stmt {
        kind,
        span: statement.span,
    }
}

pub(super) fn rewrite_function_body(body: &FunctionBody, context: &RewriteContext) -> FunctionBody {
    match body {
        FunctionBody::Expr(expr) => FunctionBody::Expr(rewrite_expr(expr, context)),
        FunctionBody::Block(statements) => FunctionBody::Block(
            statements
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
        ),
    }
}

pub(super) fn rewrite_expr(expr: &Expr, context: &RewriteContext) -> Expr {
    if let Some(name) = expr_name(expr)
        && let Some(value) = context.constants.get(&name)
    {
        return value.clone();
    }

    let kind = match &expr.kind {
        ExprKind::Call { callee, args } => {
            let callee = if let Some(name) = expr_name(callee)
                && let Some(target) = context.function_targets.get(&name)
            {
                Expr {
                    kind: ExprKind::Identifier(target.clone()),
                    span: callee.span,
                }
            } else {
                rewrite_expr(callee, context)
            };
            ExprKind::Call {
                callee: Box::new(callee),
                args: args
                    .iter()
                    .map(|arg| CallArg {
                        name: arg.name.clone(),
                        value: rewrite_expr(&arg.value, context),
                        span: arg.span,
                    })
                    .collect(),
            }
        }
        ExprKind::Unary { op, expr } => ExprKind::Unary {
            op: *op,
            expr: Box::new(rewrite_expr(expr, context)),
        },
        ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op: *op,
            left: Box::new(rewrite_expr(left, context)),
            right: Box::new(rewrite_expr(right, context)),
        },
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => ExprKind::Ternary {
            condition: Box::new(rewrite_expr(condition, context)),
            then_expr: Box::new(rewrite_expr(then_expr, context)),
            else_expr: Box::new(rewrite_expr(else_expr, context)),
        },
        ExprKind::For {
            counter,
            from,
            to,
            step,
            body,
        } => ExprKind::For {
            counter: counter.clone(),
            from: Box::new(rewrite_expr(from, context)),
            to: Box::new(rewrite_expr(to, context)),
            step: step
                .as_ref()
                .map(|step| Box::new(rewrite_expr(step, context))),
            body: body
                .iter()
                .map(|statement| rewrite_stmt(statement, context))
                .collect(),
        },
        ExprKind::Switch { selector, arms } => ExprKind::Switch {
            selector: selector
                .as_ref()
                .map(|selector| Box::new(rewrite_expr(selector, context))),
            arms: arms
                .iter()
                .map(|arm| SwitchArm {
                    condition: arm
                        .condition
                        .as_ref()
                        .map(|condition| rewrite_expr(condition, context)),
                    result: rewrite_expr(&arm.result, context),
                })
                .collect(),
        },
        ExprKind::Tuple(items) => ExprKind::Tuple(
            items
                .iter()
                .map(|item| rewrite_expr(item, context))
                .collect(),
        ),
        ExprKind::History { expr, offset } => ExprKind::History {
            expr: Box::new(rewrite_expr(expr, context)),
            offset: Box::new(rewrite_expr(offset, context)),
        },
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => {
            expr.kind.clone()
        }
    };
    Expr {
        kind,
        span: expr.span,
    }
}
