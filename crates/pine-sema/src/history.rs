use std::collections::{BTreeMap, HashMap};

use pine_ir::{
    HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirHistoryRequirements, HirLiteral,
    HirSeriesHistoryRequirement, HirStmt, HirStmtKind, HirSymbol, HirUnaryOp, SeriesId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InferredHistoryRequirements {
    pub(crate) program: HirHistoryRequirements,
    pub(crate) series: Vec<HirSeriesHistoryRequirement>,
}
#[derive(Default)]
struct HistoryRequirementCollector {
    pub(crate) program: HirHistoryRequirements,
    pub(crate) series: BTreeMap<SeriesId, HirHistoryRequirements>,
    pub(crate) builtin_series: HashMap<String, SeriesId>,
}
pub(crate) fn infer_history_requirements(
    statements: &[HirStmt],
    symbols: &[HirSymbol],
) -> InferredHistoryRequirements {
    let mut collector = HistoryRequirementCollector {
        builtin_series: symbols
            .iter()
            .filter_map(|symbol| {
                symbol
                    .series_id
                    .map(|series_id| (symbol.name.clone(), series_id))
            })
            .collect(),
        ..HistoryRequirementCollector::default()
    };
    for statement in statements {
        collector.visit_stmt(statement);
    }
    InferredHistoryRequirements {
        program: collector.program,
        series: collector
            .series
            .into_iter()
            .map(|(series_id, requirements)| HirSeriesHistoryRequirement {
                series_id,
                max_constant_offset: requirements.max_constant_offset,
                has_dynamic_offsets: requirements.has_dynamic_offsets,
            })
            .collect(),
    }
}
pub(crate) fn infer_max_bars_back(statements: &[HirStmt]) -> Option<u32> {
    statements.iter().find_map(max_bars_back_from_stmt)
}
pub(crate) fn max_bars_back_from_stmt(statement: &HirStmt) -> Option<u32> {
    match &statement.kind {
        HirStmtKind::Expr(expr)
        | HirStmtKind::Decl { value: expr, .. }
        | HirStmtKind::Reassign { value: expr, .. }
        | HirStmtKind::FieldReassign { value: expr, .. }
        | HirStmtKind::TupleDecl { value: expr, .. } => max_bars_back_from_expr(expr),
        HirStmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => max_bars_back_from_expr(condition)
            .or_else(|| infer_max_bars_back(then_branch))
            .or_else(|| infer_max_bars_back(else_branch)),
        HirStmtKind::For {
            from,
            to,
            step,
            body,
            ..
        } => max_bars_back_from_expr(from)
            .or_else(|| max_bars_back_from_expr(to))
            .or_else(|| step.as_ref().and_then(max_bars_back_from_expr))
            .or_else(|| infer_max_bars_back(body)),
        HirStmtKind::While { condition, body } => {
            max_bars_back_from_expr(condition).or_else(|| infer_max_bars_back(body))
        }
        HirStmtKind::Break | HirStmtKind::Continue => None,
    }
}
pub(crate) fn max_bars_back_from_expr(expr: &HirExpr) -> Option<u32> {
    match &expr.kind {
        HirExprKind::Call { callee, args, .. } if callee == "indicator" => args
            .iter()
            .enumerate()
            .find(|(index, arg)| {
                arg.name.as_deref() == Some("max_bars_back") || (arg.name.is_none() && *index == 6)
            })
            .and_then(|(_, arg)| constant_hir_int(&arg.value))
            .and_then(|value| u32::try_from(value).ok()),
        HirExprKind::Call { args, .. } => args
            .iter()
            .find_map(|arg| max_bars_back_from_expr(&arg.value)),
        HirExprKind::Unary { expr, .. } => max_bars_back_from_expr(expr),
        HirExprKind::Binary { left, right, .. } => {
            max_bars_back_from_expr(left).or_else(|| max_bars_back_from_expr(right))
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => max_bars_back_from_expr(condition)
            .or_else(|| max_bars_back_from_expr(then_expr))
            .or_else(|| max_bars_back_from_expr(else_expr)),
        HirExprKind::Switch { selector, arms } => selector
            .as_deref()
            .and_then(max_bars_back_from_expr)
            .or_else(|| {
                arms.iter().find_map(|arm| {
                    arm.condition
                        .as_ref()
                        .and_then(max_bars_back_from_expr)
                        .or_else(|| max_bars_back_from_expr(&arm.result))
                })
            }),
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => max_bars_back_from_expr(from)
            .or_else(|| max_bars_back_from_expr(to))
            .or_else(|| step.as_deref().and_then(max_bars_back_from_expr))
            .or_else(|| infer_max_bars_back(statements))
            .or_else(|| max_bars_back_from_expr(result)),
        HirExprKind::Tuple(items) | HirExprKind::UserTypeConstruct { fields: items } => {
            items.iter().find_map(max_bars_back_from_expr)
        }
        HirExprKind::FieldAccess { value, .. } => max_bars_back_from_expr(value),
        HirExprKind::Block { statements, result } => {
            infer_max_bars_back(statements).or_else(|| max_bars_back_from_expr(result))
        }
        HirExprKind::History { expr, offset } => max_bars_back_from_expr(expr).or_else(|| {
            if let HirHistoryOffset::Dynamic(offset) = offset {
                max_bars_back_from_expr(offset)
            } else {
                None
            }
        }),
        HirExprKind::Literal(_) | HirExprKind::Symbol(_) | HirExprKind::Builtin(_) => None,
    }
}
pub(crate) fn constant_hir_int(expr: &HirExpr) -> Option<i64> {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Int(value)) => Some(*value),
        HirExprKind::Unary {
            op: HirUnaryOp::Plus,
            expr,
        } => constant_hir_int(expr),
        HirExprKind::Unary {
            op: HirUnaryOp::Minus,
            expr,
        } => constant_hir_int(expr).and_then(i64::checked_neg),
        _ => None,
    }
}
impl HistoryRequirementCollector {
    fn visit_stmt(&mut self, statement: &HirStmt) {
        match &statement.kind {
            HirStmtKind::Expr(expr)
            | HirStmtKind::Decl { value: expr, .. }
            | HirStmtKind::Reassign { value: expr, .. }
            | HirStmtKind::FieldReassign { value: expr, .. }
            | HirStmtKind::TupleDecl { value: expr, .. } => self.visit_expr(expr),
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(condition);
                self.visit_stmts(then_branch);
                self.visit_stmts(else_branch);
            }
            HirStmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                self.visit_expr(from);
                self.visit_expr(to);
                if let Some(step) = step {
                    self.visit_expr(step);
                }
                self.visit_stmts(body);
            }
            HirStmtKind::While { condition, body } => {
                self.visit_expr(condition);
                self.visit_stmts(body);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }

    fn visit_stmts(&mut self, statements: &[HirStmt]) {
        for statement in statements {
            self.visit_stmt(statement);
        }
    }

    fn visit_expr(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Literal(_) | HirExprKind::Symbol(_) => {}
            HirExprKind::Builtin(name) => {
                if let Some(requirement) = pine_builtins::builtin_history_requirement(name) {
                    self.record_builtin_history_requirement(requirement, &[]);
                }
            }
            HirExprKind::Unary { expr, .. } => self.visit_expr(expr),
            HirExprKind::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.visit_expr(condition);
                self.visit_expr(then_expr);
                self.visit_expr(else_expr);
            }
            HirExprKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    self.visit_expr(selector);
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        self.visit_expr(condition);
                    }
                    self.visit_expr(&arm.result);
                }
            }
            HirExprKind::For {
                from,
                to,
                step,
                statements,
                result,
                ..
            } => {
                self.visit_expr(from);
                self.visit_expr(to);
                if let Some(step) = step {
                    self.visit_expr(step);
                }
                self.visit_stmts(statements);
                self.visit_expr(result);
            }
            HirExprKind::Tuple(items) => {
                for item in items {
                    self.visit_expr(item);
                }
            }
            HirExprKind::UserTypeConstruct { fields } => {
                for field in fields {
                    self.visit_expr(field);
                }
            }
            HirExprKind::FieldAccess { value, .. } => self.visit_expr(value),
            HirExprKind::Block { statements, result } => {
                self.visit_stmts(statements);
                self.visit_expr(result);
            }
            HirExprKind::Call { callee, args, .. } => {
                for arg in args {
                    self.visit_expr(&arg.value);
                }
                self.record_call_history(callee, args);
            }
            HirExprKind::History { expr, offset } => {
                self.record_history(expr.series_id, offset);
                self.visit_expr(expr);
                if let HirHistoryOffset::Dynamic(offset) = offset {
                    self.visit_expr(offset);
                }
            }
        }
    }

    fn record_history(&mut self, series_id: Option<SeriesId>, offset: &HirHistoryOffset) {
        match offset {
            HirHistoryOffset::Constant(offset) => {
                self.record_constant_history(series_id, *offset);
            }
            HirHistoryOffset::Dynamic(_) => {
                self.record_dynamic_history(series_id);
            }
        }
    }

    fn record_call_history(&mut self, callee: &str, args: &[HirCallArg]) {
        if let Some(requirement) = pine_builtins::builtin_history_requirement(callee) {
            self.record_builtin_history_requirement(requirement, args);
        }
    }

    fn record_builtin_history_requirement(
        &mut self,
        requirement: pine_builtins::BuiltinHistoryRequirement,
        args: &[HirCallArg],
    ) {
        match requirement {
            pine_builtins::BuiltinHistoryRequirement::BuiltinSeries(requirements) => {
                for requirement in requirements {
                    self.record_builtin_history(requirement.symbol, requirement.offset);
                }
            }
            pine_builtins::BuiltinHistoryRequirement::SourceOffset { source_arg, offset } => self
                .record_constant_history(
                    args.get(source_arg).and_then(|arg| arg.value.series_id),
                    offset,
                ),
            pine_builtins::BuiltinHistoryRequirement::OptionalLengthOffset {
                source_arg,
                length_arg,
                default_offset,
            } => self.record_optional_length_history(args, source_arg, length_arg, default_offset),
            pine_builtins::BuiltinHistoryRequirement::RequiredLengthOffset {
                source_arg,
                length_arg,
            } => self.record_required_length_history(args, source_arg, length_arg),
            pine_builtins::BuiltinHistoryRequirement::Cross {
                args: count,
                offset,
            } => {
                self.record_cross_history(args, count, offset);
            }
        }
    }

    fn record_optional_length_history(
        &mut self,
        args: &[HirCallArg],
        source_arg: usize,
        length_arg: usize,
        default_offset: u32,
    ) {
        let series_id = args.get(source_arg).and_then(|arg| arg.value.series_id);
        match args
            .get(length_arg)
            .and_then(|arg| constant_hir_int(&arg.value))
        {
            Some(length) if length > 0 => self.record_constant_history(series_id, length as u32),
            Some(_) => {}
            None if args.len() > length_arg => self.record_dynamic_history(series_id),
            None => self.record_constant_history(series_id, default_offset),
        }
    }

    fn record_required_length_history(
        &mut self,
        args: &[HirCallArg],
        source_arg: usize,
        length_arg: usize,
    ) {
        let series_id = args.get(source_arg).and_then(|arg| arg.value.series_id);
        match args
            .get(length_arg)
            .and_then(|arg| constant_hir_int(&arg.value))
        {
            Some(length) if length > 0 => self.record_constant_history(series_id, length as u32),
            Some(_) => {}
            None => self.record_dynamic_history(series_id),
        }
    }

    fn record_cross_history(&mut self, args: &[HirCallArg], count: usize, offset: u32) {
        for arg in args.iter().take(count) {
            self.record_constant_history(arg.value.series_id, offset);
        }
    }

    fn record_builtin_history(&mut self, name: &str, offset: u32) {
        let series_id = self.builtin_series.get(name).copied();
        self.record_constant_history(series_id, offset);
    }

    fn record_constant_history(&mut self, series_id: Option<SeriesId>, offset: u32) {
        self.program.max_constant_offset = self.program.max_constant_offset.max(offset);
        if let Some(series_id) = series_id {
            let requirement = self.series.entry(series_id).or_default();
            requirement.max_constant_offset = requirement.max_constant_offset.max(offset);
        }
    }

    fn record_dynamic_history(&mut self, series_id: Option<SeriesId>) {
        self.program.has_dynamic_offsets = true;
        if let Some(series_id) = series_id {
            self.series
                .entry(series_id)
                .or_default()
                .has_dynamic_offsets = true;
        }
    }
}
