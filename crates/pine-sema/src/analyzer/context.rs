use std::collections::{HashMap, HashSet};

mod history_offsets;

use pine_ir::{
    CallSiteId, DrawingSettings, PersistenceKind, PineType, Qualifier, ScriptMode, SeriesId,
    StrategySettings, SymbolId, VarSlotId,
};
use pine_syntax::{Diagnostic, Expr, FunctionBody, Program, Severity, Span};

use crate::analysis::Analysis;
use crate::compatibility::CompatibilityReport;
use crate::modules::ImportedUserTypeInfo;
use crate::prelude::{UserTypeIdentity, UserTypeInfo};
use crate::resolver::{BindingKey, ScopeResolver, SymbolInfo};
use crate::types::{
    const_color_value, const_int_value, const_numeric_value, const_string_value, is_collection_kind,
};

pub(crate) const MAX_SEMA_EXPR_DEPTH: u32 = 128;
pub(crate) const MAX_FUNCTION_CALL_DEPTH: usize = 64;
pub(crate) const MAX_LOWERING_INLINE_DEPTH: u32 = 64;
pub(crate) const MAX_LOWERING_HIR_NODES: u32 = 65_536;
pub(crate) const MAX_LOWERING_TEMP_SYMBOLS: u32 = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoweringLimits {
    pub(crate) max_inline_depth: u32,
    pub(crate) max_hir_nodes: u32,
    pub(crate) max_temp_symbols: u32,
}

impl Default for LoweringLimits {
    fn default() -> Self {
        Self {
            max_inline_depth: MAX_LOWERING_INLINE_DEPTH,
            max_hir_nodes: MAX_LOWERING_HIR_NODES,
            max_temp_symbols: MAX_LOWERING_TEMP_SYMBOLS,
        }
    }
}

pub(crate) struct Analyzer {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) compatibility: CompatibilityReport,
    pub(crate) scope: ScopeResolver,
    pub(crate) bindings: HashMap<BindingKey, SymbolInfo>,
    pub(crate) lower_symbol_overrides: Vec<HashMap<SymbolId, SymbolInfo>>,
    pub(crate) lower_reassigned_symbols: HashSet<SymbolId>,
    pub(crate) functions: HashMap<String, FunctionInfo>,
    pub(crate) methods: HashMap<(String, String), MethodInfo>,
    pub(crate) imported_user_types: HashMap<String, ImportedUserTypeInfo>,
    pub(crate) user_types: HashMap<String, UserTypeInfo>,
    pub(crate) symbol_user_types: HashMap<SymbolId, String>,
    pub(crate) symbol_user_type_identities: HashMap<SymbolId, UserTypeIdentity>,
    pub(crate) symbol_init_exprs: HashMap<SymbolId, Expr>,
    pub(crate) typed_na_scalar_symbols: HashSet<SymbolId>,
    pub(crate) non_scalar_udt_varip_symbols: HashSet<SymbolId>,
    pub(crate) symbol_user_type_arrays: HashMap<SymbolId, String>,
    pub(crate) symbol_maps: HashMap<SymbolId, MapTypeInfo>,
    pub(crate) const_int_symbols: HashMap<SymbolId, i64>,
    pub(crate) const_numeric_symbols: HashMap<SymbolId, f64>,
    pub(crate) const_string_symbols: HashMap<SymbolId, String>,
    pub(crate) const_bool_symbols: HashMap<SymbolId, bool>,
    pub(crate) const_color_symbols: HashMap<SymbolId, u32>,
    pub(crate) expr_user_types: HashMap<(usize, usize), String>,
    pub(crate) expr_user_type_identities: HashMap<(usize, usize), UserTypeIdentity>,
    pub(crate) expr_user_type_arrays: HashMap<(usize, usize), String>,
    pub(crate) expr_maps: HashMap<(usize, usize), MapTypeInfo>,
    pub(crate) expr_types: HashMap<(usize, usize), PineType>,
    pub(crate) pure_expr_series_ids: HashMap<String, SeriesId>,
    pub(crate) script_declaration: Option<(ScriptMode, Span)>,
    pub(crate) strategy_settings: StrategySettings,
    pub(crate) drawing_settings: DrawingSettings,
    pub(crate) function_stack: Vec<String>,
    pub(crate) function_param_symbols: Vec<HashSet<SymbolId>>,
    pub(crate) function_param_const_switch_keys: Vec<HashMap<String, ConstSwitchKey>>,
    pub(crate) function_context_is_method: Vec<bool>,
    pub(crate) next_symbol_id: u32,
    pub(crate) next_series_id: u32,
    pub(crate) next_call_site_id: u32,
    pub(crate) next_var_slot_id: u32,
    pub(crate) block_depth: u32,
    pub(crate) function_depth: u32,
    pub(crate) loop_depth: u32,
    pub(crate) expr_depth: u32,
    pub(crate) assignment_qualifier_context: Vec<Qualifier>,
    pub(crate) lowering_limits: LoweringLimits,
    pub(crate) lowering_inline_depth: u32,
    pub(crate) lowered_hir_nodes: u32,
    pub(crate) lowered_temp_symbols: u32,
    pub(crate) lowering_budget_reported: bool,
}

#[derive(Default)]
struct HistoryOffsetIntEnv {
    symbol_visiting: Vec<SymbolId>,
    locals: HashMap<String, pine_syntax::Expr>,
    local_visiting: Vec<String>,
    shadowed_locals: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub(crate) params: Vec<String>,
    pub(crate) param_types: Vec<Option<FunctionParamInfo>>,
    pub(crate) body: FunctionBody,
    pub(crate) span: Span,
}
#[derive(Debug, Clone)]
pub(crate) struct FunctionParamInfo {
    pub(crate) pine_type: PineType,
    pub(crate) user_type_name: Option<String>,
    pub(crate) span: Span,
}
#[derive(Debug, Clone)]
pub(crate) struct MethodInfo {
    pub(crate) receiver_type: String,
    pub(crate) receiver_name: String,
    pub(crate) params: Vec<MethodParamInfo>,
    pub(crate) body: FunctionBody,
    pub(crate) span: Span,
}
#[derive(Debug, Clone)]
pub(crate) struct MethodParamInfo {
    pub(crate) name: String,
    pub(crate) pine_type: PineType,
    pub(crate) user_type_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UdfArgError {
    UnknownName { name: String, span: Span },
    Duplicate { name: String, span: Span },
    PositionalAfterNamed { span: Span },
    TooMany { span: Span },
    Missing { param: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodResolution {
    NotMethod,
    Resolved(Option<PineType>),
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstSwitchKey {
    Bool(bool),
    Numeric(f64),
    String(String),
    Color(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MapTypeInfo {
    pub(crate) key_kind: pine_ir::ValueKind,
    pub(crate) value_kind: pine_ir::ValueKind,
}

impl Analyzer {
    pub(crate) fn define_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
    ) -> SymbolInfo {
        self.define_symbol_with_persistence(
            name,
            pine_type,
            persistence_kind_for_slot(var_slot_id),
            var_slot_id,
        )
    }

    pub(crate) fn define_symbol_with_persistence(
        &mut self,
        name: &str,
        pine_type: PineType,
        persistence: PersistenceKind,
        var_slot_id: Option<VarSlotId>,
    ) -> SymbolInfo {
        if let Some(existing) = self.scope.resolve(name) {
            let persistence = if persistence != PersistenceKind::None {
                persistence
            } else {
                existing.persistence
            };
            let updated = SymbolInfo {
                pine_type,
                series_id: existing
                    .series_id
                    .or_else(|| self.series_id_for_type(pine_type)),
                persistence,
                var_slot_id: existing.var_slot_id.or(var_slot_id),
                ..existing
            };
            self.scope.update(name, updated);
            return updated;
        }

        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            persistence,
            var_slot_id,
        };
        self.scope.define_global(name, info);
        info
    }

    pub(crate) fn define_local_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
        lower: bool,
    ) -> SymbolInfo {
        self.define_local_symbol_with_persistence(
            name,
            pine_type,
            persistence_kind_for_slot(var_slot_id),
            var_slot_id,
            lower,
        )
    }

    pub(crate) fn define_local_symbol_with_persistence(
        &mut self,
        name: &str,
        pine_type: PineType,
        persistence: PersistenceKind,
        var_slot_id: Option<VarSlotId>,
        lower: bool,
    ) -> SymbolInfo {
        let series_id = self.series_id_for_type(pine_type);
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id,
            persistence,
            var_slot_id,
        };
        self.scope.define_local(name, info, lower);
        info
    }

    pub(crate) fn fresh_lower_symbol(&mut self, name: &str, original: SymbolInfo) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type: original.pine_type,
            series_id: self.series_id_for_type(original.pine_type),
            persistence: original.persistence,
            var_slot_id: original.var_slot_id.map(|_| self.alloc_var_slot()),
        };
        self.scope.add_lower_symbol(name, info);
        info
    }

    pub(crate) fn fresh_temp_symbol(&mut self, name: &str, pine_type: PineType) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            persistence: PersistenceKind::None,
            var_slot_id: None,
        };
        self.scope.add_lower_symbol(name, info);
        info
    }

    pub(crate) fn update_symbol_type(&mut self, name: &str, pine_type: PineType) {
        if let Some(mut symbol) = self.scope.resolve(name) {
            symbol.pine_type = pine_type;
            if pine_type.qualifier == Qualifier::Series || is_collection_kind(pine_type.kind) {
                if symbol.series_id.is_none() {
                    symbol.series_id = Some(self.alloc_series());
                }
            } else {
                symbol.series_id = None;
            }
            self.scope.update(name, symbol);
        }
    }

    pub(crate) fn known_const_int_value(&self, expr: &pine_syntax::Expr) -> Option<i64> {
        const_int_value(expr).or_else(|| self.known_const_int_value_from_symbols(expr))
    }

    pub(crate) fn known_history_offset_int_value(&self, expr: &pine_syntax::Expr) -> Option<i64> {
        self.known_history_offset_int_value_inner(expr, &mut HistoryOffsetIntEnv::default())
    }

    fn known_history_offset_int_value_inner(
        &self,
        expr: &pine_syntax::Expr,
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        const_int_value(expr)
            .or_else(|| self.known_history_offset_int_value_from_symbols(expr, env))
    }

    fn known_history_offset_int_value_from_symbols(
        &self,
        expr: &pine_syntax::Expr,
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        match &expr.kind {
            pine_syntax::ExprKind::Identifier(name) => {
                if let Some(local) = env.locals.get(name).cloned() {
                    if env.local_visiting.contains(name) {
                        return None;
                    }
                    env.local_visiting.push(name.clone());
                    let result = self.known_history_offset_int_value_inner(&local, env);
                    env.local_visiting.pop();
                    return result;
                }
                if env.shadowed_locals.contains(name) {
                    return None;
                }

                let symbol = self.scope.resolve(name)?;
                if let Some(value) = self.const_int_symbols.get(&symbol.id) {
                    return Some(*value);
                }
                if env.symbol_visiting.contains(&symbol.id) {
                    return None;
                }
                let init_expr = self.symbol_init_exprs.get(&symbol.id)?;
                env.symbol_visiting.push(symbol.id);
                let result = self.known_history_offset_int_value_inner(init_expr, env);
                env.symbol_visiting.pop();
                result
            }
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Plus,
                expr,
            } => self.known_history_offset_int_value_inner(expr, env),
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Minus,
                expr,
            } => self
                .known_history_offset_int_value_inner(expr, env)
                .and_then(i64::checked_neg),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Add,
                left,
                right,
            } => self
                .known_history_offset_int_value_inner(left, env)?
                .checked_add(self.known_history_offset_int_value_inner(right, env)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Sub,
                left,
                right,
            } => self
                .known_history_offset_int_value_inner(left, env)?
                .checked_sub(self.known_history_offset_int_value_inner(right, env)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mul,
                left,
                right,
            } => self
                .known_history_offset_int_value_inner(left, env)?
                .checked_mul(self.known_history_offset_int_value_inner(right, env)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mod,
                left,
                right,
            } => self
                .known_history_offset_int_value_inner(left, env)?
                .checked_rem(self.known_history_offset_int_value_inner(right, env)?),
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => match self.known_history_offset_bool_value_inner(condition, env) {
                Some(true) => self.known_history_offset_int_value_inner(then_expr, env),
                Some(false) => self.known_history_offset_int_value_inner(else_expr, env),
                None => {
                    let then_value = self.known_history_offset_int_value_inner(then_expr, env)?;
                    let else_value = self.known_history_offset_int_value_inner(else_expr, env)?;
                    (then_value == else_value).then_some(then_value)
                }
            },
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => match self.known_history_offset_bool_value_inner(condition, env) {
                Some(true) => self.known_history_offset_int_branch_result(then_branch, env),
                Some(false) => self.known_history_offset_int_branch_result(else_branch, env),
                None => {
                    let then_value =
                        self.known_history_offset_int_branch_result(then_branch, env)?;
                    let else_value =
                        self.known_history_offset_int_branch_result(else_branch, env)?;
                    (then_value == else_value).then_some(then_value)
                }
            },
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_history_offset_int_switch_result(selector.as_deref(), arms, env)
            }
            pine_syntax::ExprKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                self.known_history_offset_int_value_inner(from, env)?;
                self.known_history_offset_int_value_inner(to, env)?;
                if let Some(step) = step
                    && self.known_history_offset_int_value_inner(step, env)? == 0
                {
                    return None;
                }
                self.known_history_offset_int_branch_result(body, env)
            }
            pine_syntax::ExprKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.known_history_offset_for_in_branch_result(
                index,
                value,
                iterable,
                body,
                env,
                Self::known_history_offset_int_branch_result,
            ),
            _ => None,
        }
    }
}

impl Analyzer {
    fn known_history_offset_int_branch_result(
        &self,
        statements: &[pine_syntax::Stmt],
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        let saved_locals = env.locals.clone();
        let result = self.known_history_offset_int_branch_result_inner(statements, env);
        env.locals = saved_locals;
        result
    }

    fn known_history_offset_int_branch_result_inner(
        &self,
        statements: &[pine_syntax::Stmt],
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        let (last, prefix) = statements.split_last()?;
        for statement in prefix {
            match &statement.kind {
                pine_syntax::StmtKind::Expr(_) => {}
                pine_syntax::StmtKind::Decl {
                    mode: pine_syntax::DeclMode::Normal,
                    name,
                    value,
                    ..
                } => {
                    env.locals.insert(name.clone(), value.clone());
                }
                pine_syntax::StmtKind::TupleDecl { names, value } => {
                    let pine_syntax::ExprKind::Tuple(values) = &value.kind else {
                        return None;
                    };
                    if names.len() != values.len() {
                        return None;
                    }
                    for (name, value) in names.iter().zip(values) {
                        env.locals.insert(name.clone(), value.clone());
                    }
                }
                pine_syntax::StmtKind::Reassign { name, .. } => {
                    env.locals.remove(name);
                }
                _ => return None,
            }
        }

        match &last.kind {
            pine_syntax::StmtKind::Expr(expr) => {
                self.known_history_offset_int_value_inner(expr, env)
            }
            _ => None,
        }
    }

    fn known_history_offset_int_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        if let Some(selector) = selector {
            let Some(selector_key) = self.known_history_offset_switch_key(selector, env) else {
                return self.known_history_offset_int_all_switch_results_with_default(arms, env);
            };
            for (index, arm) in arms.iter().enumerate() {
                match &arm.condition {
                    Some(condition) => match self.known_history_offset_switch_key(condition, env) {
                        Some(condition_key) => {
                            if condition_key == selector_key {
                                return self
                                    .known_history_offset_int_switch_arm_result(&arm.result, env);
                            }
                        }
                        None => {
                            return self.known_history_offset_int_all_switch_results_with_default(
                                &arms[index..],
                                env,
                            );
                        }
                    },
                    None => {
                        return self.known_history_offset_int_switch_arm_result(&arm.result, env);
                    }
                }
            }
            return None;
        }

        for (index, arm) in arms.iter().enumerate() {
            match &arm.condition {
                Some(condition) => match self.known_history_offset_bool_value_inner(condition, env)
                {
                    Some(true) => {
                        return self.known_history_offset_int_switch_arm_result(&arm.result, env);
                    }
                    Some(false) => {}
                    None => {
                        return self.known_history_offset_int_all_switch_results_with_default(
                            &arms[index..],
                            env,
                        );
                    }
                },
                None => {
                    return self.known_history_offset_int_switch_arm_result(&arm.result, env);
                }
            }
        }
        None
    }

    fn known_history_offset_int_all_switch_results_with_default(
        &self,
        arms: &[pine_syntax::SwitchArm],
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        if !arms.iter().any(|arm| arm.condition.is_none()) {
            return None;
        }

        let mut expected = None;
        for arm in arms {
            let value = self.known_history_offset_int_switch_arm_result(&arm.result, env)?;
            match expected {
                Some(expected) if expected != value => return None,
                Some(_) => {}
                None => expected = Some(value),
            }
        }
        expected
    }

    fn known_history_offset_int_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
        env: &mut HistoryOffsetIntEnv,
    ) -> Option<i64> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => {
                self.known_history_offset_int_value_inner(expr, env)
            }
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_history_offset_int_branch_result(statements, env)
            }
        }
    }

    fn known_const_int_value_from_symbols(&self, expr: &pine_syntax::Expr) -> Option<i64> {
        match &expr.kind {
            pine_syntax::ExprKind::Identifier(name) => {
                let symbol = self.scope.resolve(name)?;
                self.const_int_symbols.get(&symbol.id).copied()
            }
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Plus,
                expr,
            } => self.known_const_int_value(expr),
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Minus,
                expr,
            } => self.known_const_int_value(expr).and_then(i64::checked_neg),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Add,
                left,
                right,
            } => self
                .known_const_int_value(left)?
                .checked_add(self.known_const_int_value(right)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Sub,
                left,
                right,
            } => self
                .known_const_int_value(left)?
                .checked_sub(self.known_const_int_value(right)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mul,
                left,
                right,
            } => self
                .known_const_int_value(left)?
                .checked_mul(self.known_const_int_value(right)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mod,
                left,
                right,
            } => self
                .known_const_int_value(left)?
                .checked_rem(self.known_const_int_value(right)?),
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => match self.known_const_bool_value(condition) {
                Some(true) => self.known_const_int_value(then_expr),
                Some(false) => self.known_const_int_value(else_expr),
                None => {
                    let then_value = self.known_const_int_value(then_expr)?;
                    let else_value = self.known_const_int_value(else_expr)?;
                    (then_value == else_value).then_some(then_value)
                }
            },
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => match self.known_const_bool_value(condition) {
                Some(true) => self.known_const_int_branch_result(then_branch),
                Some(false) => self.known_const_int_branch_result(else_branch),
                None => {
                    let then_value = self.known_const_int_branch_result(then_branch)?;
                    let else_value = self.known_const_int_branch_result(else_branch)?;
                    (then_value == else_value).then_some(then_value)
                }
            },
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_const_int_switch_result(selector.as_deref(), arms)
            }
            _ => None,
        }
    }

    fn known_const_int_branch_result(&self, statements: &[pine_syntax::Stmt]) -> Option<i64> {
        match &statements.last()?.kind {
            pine_syntax::StmtKind::Expr(expr) => self.known_const_int_value(expr),
            _ => None,
        }
    }

    fn known_const_int_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<i64> {
        if let Some(selector) = selector {
            let Some(selector_key) = self.known_const_switch_key(selector) else {
                return self.known_const_int_all_switch_results_with_default(arms);
            };
            for (index, arm) in arms.iter().enumerate() {
                match &arm.condition {
                    Some(condition) => match self.known_const_switch_key(condition) {
                        Some(condition_key) => {
                            if condition_key == selector_key {
                                return self.known_const_int_switch_arm_result(&arm.result);
                            }
                        }
                        None => {
                            return self
                                .known_const_int_all_switch_results_with_default(&arms[index..]);
                        }
                    },
                    None => return self.known_const_int_switch_arm_result(&arm.result),
                }
            }
            return None;
        }

        for (index, arm) in arms.iter().enumerate() {
            match &arm.condition {
                Some(condition) => match self.known_const_bool_value(condition) {
                    Some(true) => return self.known_const_int_switch_arm_result(&arm.result),
                    Some(false) => {}
                    None => {
                        return self
                            .known_const_int_all_switch_results_with_default(&arms[index..]);
                    }
                },
                None => return self.known_const_int_switch_arm_result(&arm.result),
            }
        }
        None
    }

    fn known_const_int_all_switch_results_with_default(
        &self,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<i64> {
        if !arms.iter().any(|arm| arm.condition.is_none()) {
            return None;
        }

        let mut expected = None;
        for arm in arms {
            let value = self.known_const_int_switch_arm_result(&arm.result)?;
            match expected {
                Some(expected) if expected != value => return None,
                Some(_) => {}
                None => expected = Some(value),
            }
        }
        expected
    }

    fn known_const_int_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
    ) -> Option<i64> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => self.known_const_int_value(expr),
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_const_int_branch_result(statements)
            }
        }
    }

    pub(crate) fn known_const_string_value(&self, expr: &pine_syntax::Expr) -> Option<String> {
        const_string_value(expr).or_else(|| self.known_const_string_value_from_symbols(expr))
    }

    fn known_const_string_value_from_symbols(&self, expr: &pine_syntax::Expr) -> Option<String> {
        match &expr.kind {
            pine_syntax::ExprKind::Identifier(name) => {
                let symbol = self.scope.resolve(name)?;
                self.const_string_symbols.get(&symbol.id).cloned()
            }
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_string_value(then_expr)
                } else {
                    self.known_const_string_value(else_expr)
                }
            }
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_string_branch_result(then_branch)
                } else {
                    self.known_const_string_branch_result(else_branch)
                }
            }
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_const_string_switch_result(selector.as_deref(), arms)
            }
            _ => None,
        }
    }

    fn known_const_string_branch_result(&self, statements: &[pine_syntax::Stmt]) -> Option<String> {
        match &statements.last()?.kind {
            pine_syntax::StmtKind::Expr(expr) => self.known_const_string_value(expr),
            _ => None,
        }
    }

    fn known_const_string_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<String> {
        if let Some(selector) = selector {
            let selector_key = self.known_const_switch_key(selector)?;
            for arm in arms {
                match &arm.condition {
                    Some(condition) => {
                        if self.known_const_switch_key(condition)? == selector_key {
                            return self.known_const_string_switch_arm_result(&arm.result);
                        }
                    }
                    None => return self.known_const_string_switch_arm_result(&arm.result),
                }
            }
            return None;
        }

        for arm in arms {
            match &arm.condition {
                Some(condition) => {
                    if self.known_const_bool_value(condition)? {
                        return self.known_const_string_switch_arm_result(&arm.result);
                    }
                }
                None => return self.known_const_string_switch_arm_result(&arm.result),
            }
        }
        None
    }

    fn known_const_string_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
    ) -> Option<String> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => self.known_const_string_value(expr),
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_const_string_branch_result(statements)
            }
        }
    }

    pub(crate) fn known_const_color_value(&self, expr: &pine_syntax::Expr) -> Option<u32> {
        const_color_value(expr).or_else(|| self.known_const_color_value_from_symbols(expr))
    }

    fn known_const_color_value_from_symbols(&self, expr: &pine_syntax::Expr) -> Option<u32> {
        match &expr.kind {
            pine_syntax::ExprKind::Identifier(name) => {
                let symbol = self.scope.resolve(name)?;
                self.const_color_symbols.get(&symbol.id).copied()
            }
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_color_value(then_expr)
                } else {
                    self.known_const_color_value(else_expr)
                }
            }
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_color_branch_result(then_branch)
                } else {
                    self.known_const_color_branch_result(else_branch)
                }
            }
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_const_color_switch_result(selector.as_deref(), arms)
            }
            _ => None,
        }
    }

    fn known_const_color_branch_result(&self, statements: &[pine_syntax::Stmt]) -> Option<u32> {
        match &statements.last()?.kind {
            pine_syntax::StmtKind::Expr(expr) => self.known_const_color_value(expr),
            _ => None,
        }
    }

    fn known_const_color_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<u32> {
        if let Some(selector) = selector {
            let selector_key = self.known_const_switch_key(selector)?;
            for arm in arms {
                match &arm.condition {
                    Some(condition) => {
                        if self.known_const_switch_key(condition)? == selector_key {
                            return self.known_const_color_switch_arm_result(&arm.result);
                        }
                    }
                    None => return self.known_const_color_switch_arm_result(&arm.result),
                }
            }
            return None;
        }

        for arm in arms {
            match &arm.condition {
                Some(condition) => {
                    if self.known_const_bool_value(condition)? {
                        return self.known_const_color_switch_arm_result(&arm.result);
                    }
                }
                None => return self.known_const_color_switch_arm_result(&arm.result),
            }
        }
        None
    }

    fn known_const_color_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
    ) -> Option<u32> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => self.known_const_color_value(expr),
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_const_color_branch_result(statements)
            }
        }
    }

    pub(crate) fn known_const_numeric_value(&self, expr: &pine_syntax::Expr) -> Option<f64> {
        const_numeric_value(expr).or_else(|| self.known_const_numeric_value_from_symbols(expr))
    }

    fn known_const_numeric_value_from_symbols(&self, expr: &pine_syntax::Expr) -> Option<f64> {
        match &expr.kind {
            pine_syntax::ExprKind::Identifier(name) => {
                let symbol = self.scope.resolve(name)?;
                self.const_numeric_symbols.get(&symbol.id).copied()
            }
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Plus,
                expr,
            } => self.known_const_numeric_value(expr),
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Minus,
                expr,
            } => self.known_const_numeric_value(expr).map(|value| -value),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Add,
                left,
                right,
            } => Some(
                self.known_const_numeric_value(left)? + self.known_const_numeric_value(right)?,
            ),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Sub,
                left,
                right,
            } => Some(
                self.known_const_numeric_value(left)? - self.known_const_numeric_value(right)?,
            ),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mul,
                left,
                right,
            } => Some(
                self.known_const_numeric_value(left)? * self.known_const_numeric_value(right)?,
            ),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Div,
                left,
                right,
            } => {
                let value = self.known_const_numeric_value(left)?
                    / self.known_const_numeric_value(right)?;
                value.is_finite().then_some(value)
            }
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Mod,
                left,
                right,
            } => {
                let value = self.known_const_numeric_value(left)?
                    % self.known_const_numeric_value(right)?;
                value.is_finite().then_some(value)
            }
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_numeric_value(then_expr)
                } else {
                    self.known_const_numeric_value(else_expr)
                }
            }
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_numeric_branch_result(then_branch)
                } else {
                    self.known_const_numeric_branch_result(else_branch)
                }
            }
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_const_numeric_switch_result(selector.as_deref(), arms)
            }
            _ => None,
        }
    }

    fn known_const_numeric_branch_result(&self, statements: &[pine_syntax::Stmt]) -> Option<f64> {
        match &statements.last()?.kind {
            pine_syntax::StmtKind::Expr(expr) => self.known_const_numeric_value(expr),
            _ => None,
        }
    }

    fn known_const_numeric_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<f64> {
        if let Some(selector) = selector {
            let selector_key = self.known_const_switch_key(selector)?;
            for arm in arms {
                match &arm.condition {
                    Some(condition) => {
                        if self.known_const_switch_key(condition)? == selector_key {
                            return self.known_const_numeric_switch_arm_result(&arm.result);
                        }
                    }
                    None => return self.known_const_numeric_switch_arm_result(&arm.result),
                }
            }
            return None;
        }

        for arm in arms {
            match &arm.condition {
                Some(condition) => {
                    if self.known_const_bool_value(condition)? {
                        return self.known_const_numeric_switch_arm_result(&arm.result);
                    }
                }
                None => return self.known_const_numeric_switch_arm_result(&arm.result),
            }
        }
        None
    }

    fn known_const_numeric_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
    ) -> Option<f64> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => self.known_const_numeric_value(expr),
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_const_numeric_branch_result(statements)
            }
        }
    }

    pub(crate) fn known_const_bool_value(&self, expr: &pine_syntax::Expr) -> Option<bool> {
        match &expr.kind {
            pine_syntax::ExprKind::Literal(pine_syntax::Literal::Bool(value)) => Some(*value),
            pine_syntax::ExprKind::Identifier(name) => {
                if let Some(ConstSwitchKey::Bool(value)) = self
                    .function_param_const_switch_keys
                    .iter()
                    .rev()
                    .find_map(|keys| keys.get(name))
                {
                    return Some(*value);
                }
                let symbol = self.scope.resolve(name)?;
                self.const_bool_symbols.get(&symbol.id).copied()
            }
            pine_syntax::ExprKind::Unary {
                op: pine_syntax::UnaryOp::Not,
                expr,
            } => self.known_const_bool_value(expr).map(|value| !value),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::And,
                left,
                right,
            } => Some(self.known_const_bool_value(left)? && self.known_const_bool_value(right)?),
            pine_syntax::ExprKind::Binary {
                op: pine_syntax::BinaryOp::Or,
                left,
                right,
            } => Some(self.known_const_bool_value(left)? || self.known_const_bool_value(right)?),
            pine_syntax::ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_bool_value(then_expr)
                } else {
                    self.known_const_bool_value(else_expr)
                }
            }
            pine_syntax::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.known_const_bool_value(condition)? {
                    self.known_const_bool_branch_result(then_branch)
                } else {
                    self.known_const_bool_branch_result(else_branch)
                }
            }
            pine_syntax::ExprKind::Switch { selector, arms } => {
                self.known_const_bool_switch_result(selector.as_deref(), arms)
            }
            pine_syntax::ExprKind::Binary {
                op:
                    op @ (pine_syntax::BinaryOp::Eq
                    | pine_syntax::BinaryOp::NotEq
                    | pine_syntax::BinaryOp::Gt
                    | pine_syntax::BinaryOp::Gte
                    | pine_syntax::BinaryOp::Lt
                    | pine_syntax::BinaryOp::Lte),
                left,
                right,
            } => self
                .known_const_numeric_comparison(*op, left, right)
                .or_else(|| self.known_const_bool_comparison(*op, left, right))
                .or_else(|| self.known_const_string_comparison(*op, left, right))
                .or_else(|| self.known_const_color_comparison(*op, left, right)),
            _ => None,
        }
    }

    fn known_const_bool_branch_result(&self, statements: &[pine_syntax::Stmt]) -> Option<bool> {
        match &statements.last()?.kind {
            pine_syntax::StmtKind::Expr(expr) => self.known_const_bool_value(expr),
            _ => None,
        }
    }

    fn known_const_bool_switch_result(
        &self,
        selector: Option<&pine_syntax::Expr>,
        arms: &[pine_syntax::SwitchArm],
    ) -> Option<bool> {
        if let Some(selector) = selector {
            let selector_key = self.known_const_switch_key(selector)?;
            for arm in arms {
                match &arm.condition {
                    Some(condition) => {
                        if self.known_const_switch_key(condition)? == selector_key {
                            return self.known_const_bool_switch_arm_result(&arm.result);
                        }
                    }
                    None => return self.known_const_bool_switch_arm_result(&arm.result),
                }
            }
            return None;
        }

        for arm in arms {
            match &arm.condition {
                Some(condition) => {
                    if self.known_const_bool_value(condition)? {
                        return self.known_const_bool_switch_arm_result(&arm.result);
                    }
                }
                None => return self.known_const_bool_switch_arm_result(&arm.result),
            }
        }
        None
    }

    fn known_const_bool_switch_arm_result(
        &self,
        result: &pine_syntax::SwitchArmResult,
    ) -> Option<bool> {
        match result {
            pine_syntax::SwitchArmResult::Expr(expr) => self.known_const_bool_value(expr),
            pine_syntax::SwitchArmResult::Block(statements) => {
                self.known_const_bool_branch_result(statements)
            }
        }
    }

    pub(crate) fn known_const_switch_key(
        &self,
        expr: &pine_syntax::Expr,
    ) -> Option<ConstSwitchKey> {
        if let pine_syntax::ExprKind::Identifier(name) = &expr.kind
            && let Some(key) = self
                .function_param_const_switch_keys
                .iter()
                .rev()
                .find_map(|keys| keys.get(name).cloned())
        {
            return Some(key);
        }
        self.known_const_bool_value(expr)
            .map(ConstSwitchKey::Bool)
            .or_else(|| {
                self.known_const_string_value(expr)
                    .map(ConstSwitchKey::String)
            })
            .or_else(|| {
                self.known_const_color_value(expr)
                    .map(ConstSwitchKey::Color)
            })
            .or_else(|| {
                self.known_const_numeric_value(expr)
                    .map(ConstSwitchKey::Numeric)
            })
    }

    fn known_const_numeric_comparison(
        &self,
        op: pine_syntax::BinaryOp,
        left: &pine_syntax::Expr,
        right: &pine_syntax::Expr,
    ) -> Option<bool> {
        let left = self.known_const_numeric_value(left)?;
        let right = self.known_const_numeric_value(right)?;
        Some(match op {
            pine_syntax::BinaryOp::Eq => left == right,
            pine_syntax::BinaryOp::NotEq => left != right,
            pine_syntax::BinaryOp::Gt => left > right,
            pine_syntax::BinaryOp::Gte => left >= right,
            pine_syntax::BinaryOp::Lt => left < right,
            pine_syntax::BinaryOp::Lte => left <= right,
            _ => return None,
        })
    }

    fn known_const_bool_comparison(
        &self,
        op: pine_syntax::BinaryOp,
        left: &pine_syntax::Expr,
        right: &pine_syntax::Expr,
    ) -> Option<bool> {
        let left = self.known_const_bool_value(left)?;
        let right = self.known_const_bool_value(right)?;
        match op {
            pine_syntax::BinaryOp::Eq => Some(left == right),
            pine_syntax::BinaryOp::NotEq => Some(left != right),
            _ => None,
        }
    }

    fn known_const_string_comparison(
        &self,
        op: pine_syntax::BinaryOp,
        left: &pine_syntax::Expr,
        right: &pine_syntax::Expr,
    ) -> Option<bool> {
        let left = self.known_const_string_value(left)?;
        let right = self.known_const_string_value(right)?;
        match op {
            pine_syntax::BinaryOp::Eq => Some(left == right),
            pine_syntax::BinaryOp::NotEq => Some(left != right),
            _ => None,
        }
    }

    fn known_const_color_comparison(
        &self,
        op: pine_syntax::BinaryOp,
        left: &pine_syntax::Expr,
        right: &pine_syntax::Expr,
    ) -> Option<bool> {
        let left = self.known_const_color_value(left)?;
        let right = self.known_const_color_value(right)?;
        match op {
            pine_syntax::BinaryOp::Eq => Some(left == right),
            pine_syntax::BinaryOp::NotEq => Some(left != right),
            _ => None,
        }
    }

    pub(crate) fn series_id_for_type(&mut self, pine_type: PineType) -> Option<SeriesId> {
        if pine_type.qualifier == Qualifier::Series || is_collection_kind(pine_type.kind) {
            Some(self.alloc_series())
        } else {
            None
        }
    }

    pub(crate) fn alloc_symbol(&mut self) -> SymbolId {
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    pub(crate) fn alloc_series(&mut self) -> SeriesId {
        let id = SeriesId(self.next_series_id);
        self.next_series_id += 1;
        id
    }

    pub(crate) fn alloc_call_site(&mut self) -> CallSiteId {
        let id = CallSiteId(self.next_call_site_id);
        self.next_call_site_id += 1;
        id
    }

    pub(crate) fn alloc_var_slot(&mut self) -> VarSlotId {
        let id = VarSlotId(self.next_var_slot_id);
        self.next_var_slot_id += 1;
        id
    }

    pub(crate) fn finish(mut self, program: &Program) -> Analysis {
        let hir = if self.has_errors() {
            None
        } else {
            self.lower_program(program)
        };

        Analysis {
            diagnostics: self.diagnostics,
            compatibility: self.compatibility,
            hir,
        }
    }

    pub(crate) fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

fn persistence_kind_for_slot(var_slot_id: Option<VarSlotId>) -> PersistenceKind {
    if var_slot_id.is_some() {
        PersistenceKind::Var
    } else {
        PersistenceKind::None
    }
}
