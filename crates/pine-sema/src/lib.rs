//! Semantic analysis and compatibility gating scaffolding.

use std::collections::HashMap;

use pine_builtins::{Accepts, BuiltinSignature, ReturnSpec};
use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt,
    HirStmtKind, HirSymbol, HirUnaryOp, PineType, Qualifier, SeriesId, SymbolId, ValueKind,
    VarSlotId,
};
use pine_syntax::{
    BinaryOp, CallArg, Diagnostic, Expr, ExprKind, Literal, Program, Severity, SourceFile, Span,
    Stmt, StmtKind, UnaryOp, parse_source,
};

const VARIP_UNSUPPORTED_REASON: &str = "varip intrabar persistence is not implemented; forming-bar rollback currently supports var state only";

#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
    pub compatibility: CompatibilityReport,
    pub hir: Option<HirProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompatibilityReport {
    pub language_version: Option<u16>,
    pub supported: Vec<FeatureUse>,
    pub unsupported: Vec<UnsupportedFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureUse {
    pub feature: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedFeature {
    pub feature: String,
    pub reason: String,
    pub span: Span,
}

pub fn analyze_source(source: &SourceFile) -> Analysis {
    let parsed = parse_source(source);
    let mut analyzer = Analyzer {
        diagnostics: parsed.diagnostics,
        compatibility: CompatibilityReport {
            language_version: parsed.program.version.map(|version| version.version),
            ..CompatibilityReport::default()
        },
        scope: ScopeResolver::new(initial_symbols(), initial_symbol_order()),
        next_symbol_id: initial_symbol_count(),
        next_series_id: initial_series_count(),
        next_call_site_id: 0,
        next_var_slot_id: 0,
    };
    analyzer.analyze_program(&parsed.program);
    analyzer.finish(&parsed.program)
}

#[derive(Debug, Default, Clone)]
pub struct CompileCache {
    entries: HashMap<CompileCacheKey, Analysis>,
    hits: usize,
    misses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheStats {
    pub entries: usize,
    pub hits: usize,
    pub misses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompileCacheKey {
    name: String,
    text: String,
}

impl CompileCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, source: &SourceFile) -> Analysis {
        let key = CompileCacheKey::from_source(source);
        if let Some(analysis) = self.entries.get(&key) {
            self.hits += 1;
            return analysis.clone();
        }

        self.misses += 1;
        let analysis = analyze_source(source);
        self.entries.insert(key, analysis.clone());
        analysis
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    #[must_use]
    pub fn stats(&self) -> CompileCacheStats {
        CompileCacheStats {
            entries: self.entries.len(),
            hits: self.hits,
            misses: self.misses,
        }
    }
}

impl CompileCacheKey {
    fn from_source(source: &SourceFile) -> Self {
        Self {
            name: source.name().to_owned(),
            text: source.text().to_owned(),
        }
    }
}

struct Analyzer {
    diagnostics: Vec<Diagnostic>,
    compatibility: CompatibilityReport,
    scope: ScopeResolver,
    next_symbol_id: u32,
    next_series_id: u32,
    next_call_site_id: u32,
    next_var_slot_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolInfo {
    id: SymbolId,
    pine_type: PineType,
    series_id: Option<SeriesId>,
    var_slot_id: Option<VarSlotId>,
}

#[derive(Debug, Clone)]
struct ScopeResolver {
    scopes: Vec<HashMap<String, SymbolInfo>>,
    symbol_order: Vec<String>,
}

impl ScopeResolver {
    fn new(global_symbols: HashMap<String, SymbolInfo>, symbol_order: Vec<String>) -> Self {
        Self {
            scopes: vec![global_symbols],
            symbol_order,
        }
    }

    fn resolve(&self, name: &str) -> Option<SymbolInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn define_global(&mut self, name: &str, info: SymbolInfo) {
        let global_scope = self
            .scopes
            .first_mut()
            .expect("scope resolver always has a global scope");
        if !global_scope.contains_key(name) {
            self.symbol_order.push(name.to_owned());
        }
        global_scope.insert(name.to_owned(), info);
    }

    fn update(&mut self, name: &str, info: SymbolInfo) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            scope.insert(name.to_owned(), info);
        }
    }

    fn lower_symbols(&self) -> Vec<HirSymbol> {
        let global_scope = self
            .scopes
            .first()
            .expect("scope resolver always has a global scope");
        self.symbol_order
            .iter()
            .filter_map(|name| {
                global_scope.get(name).map(|symbol| HirSymbol {
                    id: symbol.id,
                    name: name.clone(),
                    pine_type: symbol.pine_type,
                    series_id: symbol.series_id,
                    var_slot_id: symbol.var_slot_id,
                })
            })
            .collect()
    }
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) {
        for statement in &program.statements {
            self.analyze_stmt(statement);
        }
    }

    fn analyze_stmt(&mut self, statement: &Stmt) {
        match &statement.kind {
            StmtKind::Expr(expr) => {
                self.analyze_expr(expr);
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    self.expect_bool(condition_type, condition.span);
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "if".to_owned(),
                    span: statement.span,
                });

                for branch_statement in then_branch.iter().chain(else_branch) {
                    self.analyze_stmt(branch_statement);
                }
            }
            StmtKind::Decl { mode, name, value } => {
                if matches!(mode, pine_syntax::DeclMode::Varip) {
                    self.unsupported("varip", VARIP_UNSUPPORTED_REASON, statement.span);
                }
                let value_type = self.analyze_expr(value).unwrap_or(UNKNOWN);
                let var_slot_id = if matches!(mode, pine_syntax::DeclMode::Var) {
                    Some(self.alloc_var_slot())
                } else {
                    None
                };
                self.define_symbol(name, value_type, var_slot_id);
            }
            StmtKind::Reassign { name, value } => {
                if self.scope.resolve(name).is_none() {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UNKNOWN_SYMBOL",
                        format!("cannot reassign unknown symbol `{name}`"),
                        statement.span,
                    ));
                }
                let value_type = self.analyze_expr(value);
                if let (Some(target_type), Some(value_type)) = (
                    self.scope.resolve(name).map(|symbol| symbol.pine_type),
                    value_type,
                ) {
                    self.validate_assignment(name, target_type, value_type, statement.span);
                    self.update_symbol_type(name, value_type);
                }
            }
            StmtKind::TupleDecl { .. } => {
                self.analyze_tuple_decl(statement);
            }
            StmtKind::Unsupported { feature } => {
                self.unsupported(feature, unsupported_syntax_reason(feature), statement.span);
            }
        }
    }

    fn analyze_tuple_decl(&mut self, statement: &pine_syntax::Stmt) {
        let StmtKind::TupleDecl { names, value } = &statement.kind else {
            return;
        };
        self.analyze_expr(value);

        let Some(element_types) = self.tuple_element_types(value) else {
            self.diagnostics.push(Diagnostic::error(
                "E_TUPLE_TYPE",
                "tuple assignment requires a tuple value",
                value.span,
            ));
            return;
        };

        if names.len() != element_types.len() {
            self.diagnostics.push(Diagnostic::error(
                "E_TUPLE_ARITY",
                format!(
                    "tuple assignment expects {} value(s), got {}",
                    names.len(),
                    element_types.len()
                ),
                statement.span,
            ));
            return;
        }

        for (name, pine_type) in names.iter().zip(element_types) {
            self.define_symbol(name, pine_type, None);
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Option<PineType> {
        match &expr.kind {
            ExprKind::Literal(literal) => Some(literal_type(literal)),
            ExprKind::Identifier(name) => {
                self.check_feature_expr(expr);
                self.resolve_symbol(name, expr.span)
            }
            ExprKind::QualifiedName(_) => {
                let name = expr_name(expr)?;
                self.resolve_qualified_value(&name, expr.span)
            }
            ExprKind::Unary { op, expr } => {
                let expr_type = self.analyze_expr(expr)?;
                self.infer_unary(*op, expr_type, expr.span)
            }
            ExprKind::Binary { op, left, right } => {
                let left_type = self.analyze_expr(left);
                let right_type = self.analyze_expr(right);
                match (left_type, right_type) {
                    (Some(left_type), Some(right_type)) => {
                        self.infer_binary(*op, left_type, right_type, expr.span)
                    }
                    _ => None,
                }
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    self.expect_bool(condition_type, condition.span);
                }
                let then_type = self.analyze_expr(then_expr);
                let else_type = self.analyze_expr(else_expr);
                match (condition_type, then_type, else_type) {
                    (Some(condition_type), Some(then_type), Some(else_type)) => {
                        self.merge_branch_types(condition_type, then_type, else_type, expr.span)
                    }
                    _ => None,
                }
            }
            ExprKind::Tuple(items) => {
                for item in items {
                    self.analyze_expr(item);
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => self.analyze_call(callee, args),
            ExprKind::History { expr, offset } => {
                let value_type = self.analyze_expr(expr);
                self.analyze_expr(offset);
                self.validate_history_offset(offset);
                value_type.map(|value_type| PineType::new(Qualifier::Series, value_type.kind))
            }
        }
    }

    fn analyze_call(&mut self, callee: &Expr, args: &[CallArg]) -> Option<PineType> {
        let Some(name) = expr_name(callee) else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_TARGET",
                "expected function name",
                callee.span,
            ));
            return None;
        };
        self.check_feature_name(&name, callee.span);

        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();

        let signature = pine_builtins::get_phase_1_builtin(&name)?;

        self.validate_call_args(signature, args, &arg_types);
        self.return_type(signature, &arg_types)
    }

    fn validate_history_offset(&mut self, offset: &Expr) {
        match &offset.kind {
            ExprKind::Literal(Literal::Int(value)) if *value >= 0 => {}
            ExprKind::Unary {
                op: UnaryOp::Minus,
                expr,
            } if matches!(expr.kind, ExprKind::Literal(Literal::Int(_))) => {
                self.unsupported(
                    "negative_history_offset",
                    "history offsets must be non-negative in Phase 1",
                    offset.span,
                );
            }
            _ => {
                self.unsupported(
                    "dynamic_history_offset",
                    "dynamic history offsets are not supported in Phase 1",
                    offset.span,
                );
            }
        }
    }

    fn check_feature_expr(&mut self, expr: &Expr) {
        let Some(name) = expr_name(expr) else {
            return;
        };
        self.check_feature_name(&name, expr.span);
    }

    fn check_feature_name(&mut self, name: &str, span: Span) {
        let unsupported_reason = if name.starts_with("strategy.") {
            Some("strategy backtesting and broker emulation are outside the current runtime scope")
        } else if name.starts_with("request.") {
            Some("multi-symbol and multi-timeframe data requests are not supported in Phase 1")
        } else if name.starts_with("array.") {
            Some("array storage and mutation are not supported in Phase 1")
        } else if matches!(name, "alert" | "alertcondition") {
            Some("alerts are not supported in Phase 1")
        } else if name.starts_with("label.")
            || name.starts_with("line.")
            || name.starts_with("box.")
            || name.starts_with("table.")
            || name.starts_with("polyline.")
        {
            Some("drawing object systems are not supported in Phase 1")
        } else {
            None
        };

        if let Some(reason) = unsupported_reason {
            self.unsupported(name, reason, span);
        } else if pine_builtins::is_phase_1_builtin(name) {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
        }
    }

    fn validate_call_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        let required_count = signature
            .params
            .iter()
            .filter(|param| !param.optional)
            .count();
        if args.len() < required_count {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{}` expects at least {} argument(s), got {}",
                    signature.name,
                    required_count,
                    args.len()
                ),
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
            return;
        }

        if !signature.variadic && args.len() > signature.params.len() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{}` expects at most {} argument(s), got {}",
                    signature.name,
                    signature.params.len(),
                    args.len()
                ),
                args[signature.params.len()].span,
            ));
        }

        for (index, arg) in args.iter().enumerate() {
            let Some(param) = self.resolve_param(signature, index, arg) else {
                continue;
            };
            let Some(arg_type) = arg_types.get(index).copied().flatten() else {
                continue;
            };

            if !accepts_type(param.accepts, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`{}` argument `{}` does not accept {:?} {:?}",
                        signature.name, param.name, arg_type.qualifier, arg_type.kind
                    ),
                    arg.span,
                ));
            }
        }
    }

    fn resolve_param<'a>(
        &mut self,
        signature: &'a BuiltinSignature,
        index: usize,
        arg: &CallArg,
    ) -> Option<&'a pine_builtins::BuiltinParam> {
        if let Some(name) = &arg.name {
            let param = signature.params.iter().find(|param| param.name == name);
            if param.is_none() {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    format!("`{}` has no argument named `{name}`", signature.name),
                    arg.span,
                ));
            }
            param
        } else {
            signature.params.get(index).or_else(|| {
                signature
                    .variadic
                    .then(|| signature.params.last())
                    .flatten()
            })
        }
    }

    fn return_type(
        &self,
        signature: &BuiltinSignature,
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
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
            ReturnSpec::PromotedNumeric => promoted_numeric_type(arg_types),
        }
    }

    fn resolve_qualified_value(&mut self, name: &str, span: Span) -> Option<PineType> {
        if pine_builtins::named_color(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Color));
        }

        self.check_feature_name(name, span);
        if name.starts_with("color.") {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_COLOR",
                format!("unknown named color `{name}`"),
                span,
            ));
        }
        None
    }

    fn resolve_symbol(&mut self, name: &str, span: Span) -> Option<PineType> {
        if let Some(symbol) = self.scope.resolve(name) {
            Some(symbol.pine_type)
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_SYMBOL",
                format!("unknown symbol `{name}`"),
                span,
            ));
            None
        }
    }

    fn define_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
    ) -> SymbolId {
        if let Some(existing) = self.scope.resolve(name) {
            let updated = SymbolInfo {
                pine_type,
                series_id: existing
                    .series_id
                    .or_else(|| self.series_id_for_type(pine_type)),
                var_slot_id: existing.var_slot_id.or(var_slot_id),
                ..existing
            };
            self.scope.update(name, updated);
            return existing.id;
        }

        let id = self.alloc_symbol();
        let info = SymbolInfo {
            id,
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            var_slot_id,
        };
        self.scope.define_global(name, info);
        id
    }

    fn update_symbol_type(&mut self, name: &str, pine_type: PineType) {
        if let Some(mut symbol) = self.scope.resolve(name) {
            symbol.pine_type = pine_type;
            if symbol.series_id.is_none() {
                symbol.series_id = self.series_id_for_type(pine_type);
            }
            self.scope.update(name, symbol);
        }
    }

    fn series_id_for_type(&mut self, pine_type: PineType) -> Option<SeriesId> {
        if pine_type.qualifier == Qualifier::Series {
            Some(self.alloc_series())
        } else {
            None
        }
    }

    fn alloc_symbol(&mut self) -> SymbolId {
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    fn alloc_series(&mut self) -> SeriesId {
        let id = SeriesId(self.next_series_id);
        self.next_series_id += 1;
        id
    }

    fn alloc_call_site(&mut self) -> CallSiteId {
        let id = CallSiteId(self.next_call_site_id);
        self.next_call_site_id += 1;
        id
    }

    fn alloc_var_slot(&mut self) -> VarSlotId {
        let id = VarSlotId(self.next_var_slot_id);
        self.next_var_slot_id += 1;
        id
    }

    fn validate_assignment(
        &mut self,
        name: &str,
        target_type: PineType,
        value_type: PineType,
        span: Span,
    ) {
        if !can_assign(target_type, value_type) {
            self.diagnostics.push(Diagnostic::error(
                "E_ASSIGN_TYPE",
                format!(
                    "cannot assign {:?} {:?} to `{}` of type {:?} {:?}",
                    value_type.qualifier,
                    value_type.kind,
                    name,
                    target_type.qualifier,
                    target_type.kind
                ),
                span,
            ));
        }
    }

    fn infer_unary(&mut self, op: UnaryOp, expr_type: PineType, span: Span) -> Option<PineType> {
        match op {
            UnaryOp::Plus | UnaryOp::Minus if is_numeric(expr_type.kind) => Some(expr_type),
            UnaryOp::Not if expr_type.kind == ValueKind::Bool => Some(expr_type),
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_OPERATOR_TYPE",
                    format!(
                        "operator {:?} does not accept {:?} {:?}",
                        op, expr_type.qualifier, expr_type.kind
                    ),
                    span,
                ));
                None
            }
        }
    }

    fn infer_binary(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if is_numeric(left_type.kind) && is_numeric(right_type.kind) {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        numeric_result_kind(op, left_type.kind, right_type.kind),
                    ))
                } else {
                    self.operator_error(op, left_type, right_type, span);
                    None
                }
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Gt
            | BinaryOp::Gte
            | BinaryOp::Lt
            | BinaryOp::Lte => Some(PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::Bool,
            )),
            BinaryOp::And | BinaryOp::Or => {
                if left_type.kind == ValueKind::Bool && right_type.kind == ValueKind::Bool {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    ))
                } else {
                    self.operator_error(op, left_type, right_type, span);
                    None
                }
            }
        }
    }

    fn operator_error(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            "E_OPERATOR_TYPE",
            format!(
                "operator {:?} does not accept {:?} {:?} and {:?} {:?}",
                op, left_type.qualifier, left_type.kind, right_type.qualifier, right_type.kind
            ),
            span,
        ));
    }

    fn expect_bool(&mut self, pine_type: PineType, span: Span) {
        if pine_type.kind != ValueKind::Bool {
            self.diagnostics.push(Diagnostic::error(
                "E_CONDITION_TYPE",
                format!(
                    "condition must be bool, got {:?} {:?}",
                    pine_type.qualifier, pine_type.kind
                ),
                span,
            ));
        }
    }

    fn merge_branch_types(
        &mut self,
        condition_type: PineType,
        then_type: PineType,
        else_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        let Some(kind) = common_kind(then_type.kind, else_type.kind) else {
            self.diagnostics.push(Diagnostic::error(
                "E_BRANCH_TYPE",
                format!(
                    "ternary branches have incompatible types {:?} and {:?}",
                    then_type.kind, else_type.kind
                ),
                span,
            ));
            return None;
        };

        Some(PineType::new(
            strongest_qualifier(
                condition_type.qualifier,
                strongest_qualifier(then_type.qualifier, else_type.qualifier),
            ),
            kind,
        ))
    }

    fn unsupported(&mut self, feature: &str, reason: &str, span: Span) {
        self.compatibility.unsupported.push(UnsupportedFeature {
            feature: feature.to_owned(),
            reason: reason.to_owned(),
            span,
        });
        self.diagnostics.push(Diagnostic {
            code: "E_UNSUPPORTED_FEATURE".to_owned(),
            severity: Severity::Error,
            message: format!("`{feature}` is not supported: {reason}"),
            span,
        });
    }

    fn finish(mut self, program: &Program) -> Analysis {
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

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    fn lower_program(&mut self, program: &Program) -> Option<HirProgram> {
        let mut statements = Vec::new();
        for statement in &program.statements {
            statements.push(self.lower_stmt(statement)?);
        }

        Some(HirProgram {
            symbols: self.lower_symbols(),
            statements,
            next_series_id: self.next_series_id,
            next_call_site_id: self.next_call_site_id,
            next_var_slot_id: self.next_var_slot_id,
        })
    }

    fn lower_symbols(&self) -> Vec<HirSymbol> {
        self.scope.lower_symbols()
    }

    fn lower_stmt(&mut self, statement: &pine_syntax::Stmt) -> Option<HirStmt> {
        let kind = match &statement.kind {
            StmtKind::Expr(expr) => HirStmtKind::Expr(self.lower_expr(expr)?),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => HirStmtKind::If {
                condition: self.lower_expr(condition)?,
                then_branch: then_branch
                    .iter()
                    .map(|statement| self.lower_stmt(statement))
                    .collect::<Option<_>>()?,
                else_branch: else_branch
                    .iter()
                    .map(|statement| self.lower_stmt(statement))
                    .collect::<Option<_>>()?,
            },
            StmtKind::Decl { name, value, .. } => HirStmtKind::Decl {
                symbol: self.scope.resolve(name)?.id,
                value: self.lower_expr(value)?,
            },
            StmtKind::Reassign { name, value } => HirStmtKind::Reassign {
                symbol: self.scope.resolve(name)?.id,
                value: self.lower_expr(value)?,
            },
            StmtKind::TupleDecl { names, value } => HirStmtKind::TupleDecl {
                symbols: names
                    .iter()
                    .map(|name| self.scope.resolve(name).map(|symbol| symbol.id))
                    .collect::<Option<_>>()?,
                value: self.lower_expr(value)?,
            },
            StmtKind::Unsupported { .. } => return None,
        };

        Some(HirStmt { kind })
    }

    fn lower_expr(&mut self, expr: &Expr) -> Option<HirExpr> {
        let pine_type = self.type_of_expr(expr)?;
        let series_id =
            if pine_type.qualifier == Qualifier::Series && pine_type.kind != ValueKind::Tuple {
                match &expr.kind {
                    ExprKind::Identifier(name) => {
                        self.scope.resolve(name).and_then(|symbol| symbol.series_id)
                    }
                    _ => Some(self.alloc_series()),
                }
            } else {
                None
            };

        let kind = match &expr.kind {
            ExprKind::Literal(literal) => HirExprKind::Literal(lower_literal(literal)),
            ExprKind::Identifier(name) => HirExprKind::Symbol(self.scope.resolve(name)?.id),
            ExprKind::QualifiedName(parts) => HirExprKind::Builtin(parts.join(".")),
            ExprKind::Unary { op, expr } => HirExprKind::Unary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr(expr)?),
            },
            ExprKind::Binary { op, left, right } => HirExprKind::Binary {
                op: lower_binary_op(*op),
                left: Box::new(self.lower_expr(left)?),
                right: Box::new(self.lower_expr(right)?),
            },
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => HirExprKind::Ternary {
                condition: Box::new(self.lower_expr(condition)?),
                then_expr: Box::new(self.lower_expr(then_expr)?),
                else_expr: Box::new(self.lower_expr(else_expr)?),
            },
            ExprKind::Tuple(items) => HirExprKind::Tuple(
                items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect::<Option<_>>()?,
            ),
            ExprKind::Call { callee, args } => HirExprKind::Call {
                callee: expr_name(callee)?,
                call_site_id: self.alloc_call_site(),
                args: args
                    .iter()
                    .map(|arg| {
                        Some(HirCallArg {
                            name: arg.name.clone(),
                            value: self.lower_expr(&arg.value)?,
                        })
                    })
                    .collect::<Option<_>>()?,
            },
            ExprKind::History { expr, offset } => HirExprKind::History {
                expr: Box::new(self.lower_expr(expr)?),
                offset: constant_history_offset(offset)?,
            },
        };

        Some(HirExpr {
            kind,
            pine_type,
            series_id,
        })
    }

    fn type_of_expr(&self, expr: &Expr) -> Option<PineType> {
        match &expr.kind {
            ExprKind::Literal(literal) => Some(literal_type(literal)),
            ExprKind::Identifier(name) => self.scope.resolve(name).map(|symbol| symbol.pine_type),
            ExprKind::QualifiedName(_) => {
                let name = expr_name(expr)?;
                pine_builtins::named_color(&name)
                    .map(|_| PineType::new(Qualifier::Const, ValueKind::Color))
            }
            ExprKind::Unary { expr, .. } => self.type_of_expr(expr),
            ExprKind::Binary { op, left, right } => {
                let left_type = self.type_of_expr(left)?;
                let right_type = self.type_of_expr(right)?;
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
                let condition_type = self.type_of_expr(condition)?;
                let then_type = self.type_of_expr(then_expr)?;
                let else_type = self.type_of_expr(else_expr)?;
                Some(PineType::new(
                    strongest_qualifier(
                        condition_type.qualifier,
                        strongest_qualifier(then_type.qualifier, else_type.qualifier),
                    ),
                    common_kind(then_type.kind, else_type.kind)?,
                ))
            }
            ExprKind::Tuple(items) => {
                for item in items {
                    self.type_of_expr(item)?;
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => {
                let signature = pine_builtins::get_phase_1_builtin(&expr_name(callee)?)?;
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| self.type_of_expr(&arg.value))
                    .collect();
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
                    ReturnSpec::PromotedNumeric => promoted_numeric_type(&arg_types),
                }
            }
            ExprKind::History { expr, .. } => self
                .type_of_expr(expr)
                .map(|pine_type| PineType::new(Qualifier::Series, pine_type.kind)),
        }
    }

    fn tuple_element_types(&self, expr: &Expr) -> Option<Vec<PineType>> {
        match &expr.kind {
            ExprKind::Tuple(items) => items
                .iter()
                .map(|item| self.type_of_expr(item))
                .collect::<Option<_>>(),
            ExprKind::Call { callee, .. } => {
                let signature = pine_builtins::get_phase_1_builtin(&expr_name(callee)?)?;
                match signature.returns {
                    ReturnSpec::Tuple(types) => Some(types.to_vec()),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn unsupported_syntax_reason(feature: &str) -> &'static str {
    match feature {
        "import" => "library imports are not supported in Phase 1",
        "if" => "block if syntax is parsed for compatibility reporting but not executable yet",
        "function" => {
            "user-defined functions are parsed for compatibility reporting but not executable yet"
        }
        "for" => "for loops are parsed for compatibility reporting but not executable yet",
        _ => "syntax is not supported in Phase 1",
    }
}

fn expr_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::QualifiedName(parts) => Some(parts.join(".")),
        _ => None,
    }
}

const UNKNOWN: PineType = PineType::new(Qualifier::Series, ValueKind::Na);

const INITIAL_SYMBOLS: &[(&str, PineType)] = &[
    ("open", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("high", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("low", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("close", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("volume", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("time", PineType::new(Qualifier::Series, ValueKind::Int)),
    ("hl2", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("hlc3", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ohlc4", PineType::new(Qualifier::Series, ValueKind::Float)),
    (
        "bar_index",
        PineType::new(Qualifier::Series, ValueKind::Int),
    ),
    ("na", PineType::new(Qualifier::Const, ValueKind::Na)),
];

fn initial_symbols() -> HashMap<String, SymbolInfo> {
    INITIAL_SYMBOLS
        .iter()
        .enumerate()
        .map(|(index, (name, pine_type))| {
            (
                (*name).to_owned(),
                SymbolInfo {
                    id: SymbolId(index as u32),
                    pine_type: *pine_type,
                    series_id: if pine_type.qualifier == Qualifier::Series {
                        Some(SeriesId(index as u32))
                    } else {
                        None
                    },
                    var_slot_id: None,
                },
            )
        })
        .collect()
}

fn initial_symbol_order() -> Vec<String> {
    INITIAL_SYMBOLS
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

fn initial_symbol_count() -> u32 {
    INITIAL_SYMBOLS.len() as u32
}

fn initial_series_count() -> u32 {
    INITIAL_SYMBOLS
        .iter()
        .filter(|(_, pine_type)| pine_type.qualifier == Qualifier::Series)
        .count() as u32
}

fn lower_literal(literal: &Literal) -> HirLiteral {
    match literal {
        Literal::Int(value) => HirLiteral::Int(*value),
        Literal::Float(value) => HirLiteral::Float(*value),
        Literal::Bool(value) => HirLiteral::Bool(*value),
        Literal::String(value) => HirLiteral::String(value.clone()),
        Literal::ColorHex(value) => HirLiteral::ColorHex(value.clone()),
    }
}

fn lower_unary_op(op: UnaryOp) -> HirUnaryOp {
    match op {
        UnaryOp::Plus => HirUnaryOp::Plus,
        UnaryOp::Minus => HirUnaryOp::Minus,
        UnaryOp::Not => HirUnaryOp::Not,
    }
}

fn lower_binary_op(op: BinaryOp) -> HirBinaryOp {
    match op {
        BinaryOp::Add => HirBinaryOp::Add,
        BinaryOp::Sub => HirBinaryOp::Sub,
        BinaryOp::Mul => HirBinaryOp::Mul,
        BinaryOp::Div => HirBinaryOp::Div,
        BinaryOp::Mod => HirBinaryOp::Mod,
        BinaryOp::Eq => HirBinaryOp::Eq,
        BinaryOp::NotEq => HirBinaryOp::NotEq,
        BinaryOp::Gt => HirBinaryOp::Gt,
        BinaryOp::Gte => HirBinaryOp::Gte,
        BinaryOp::Lt => HirBinaryOp::Lt,
        BinaryOp::Lte => HirBinaryOp::Lte,
        BinaryOp::And => HirBinaryOp::And,
        BinaryOp::Or => HirBinaryOp::Or,
    }
}

fn constant_history_offset(expr: &Expr) -> Option<u32> {
    match expr.kind {
        ExprKind::Literal(Literal::Int(value)) if value >= 0 => Some(value as u32),
        _ => None,
    }
}

fn literal_type(literal: &Literal) -> PineType {
    match literal {
        Literal::Int(_) => PineType::new(Qualifier::Const, ValueKind::Int),
        Literal::Float(_) => PineType::new(Qualifier::Const, ValueKind::Float),
        Literal::Bool(_) => PineType::new(Qualifier::Const, ValueKind::Bool),
        Literal::String(_) => PineType::new(Qualifier::Const, ValueKind::String),
        Literal::ColorHex(_) => PineType::new(Qualifier::Const, ValueKind::Color),
    }
}

fn accepts_type(accepts: Accepts, arg_type: PineType) -> bool {
    match accepts {
        Accepts::Any => true,
        Accepts::Exact(expected) => can_assign(expected, arg_type),
        Accepts::Kind(kind) => arg_type.kind == kind,
        Accepts::Numeric => is_numeric(arg_type.kind),
        Accepts::SeriesFloat => {
            arg_type.qualifier == Qualifier::Series && arg_type.kind == ValueKind::Float
        }
        Accepts::SeriesOrSimpleNumeric => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Series) && is_numeric(arg_type.kind)
        }
        Accepts::SimpleInt => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && arg_type.kind == ValueKind::Int
        }
        Accepts::ConstString => {
            arg_type.qualifier == Qualifier::Const && arg_type.kind == ValueKind::String
        }
        Accepts::ConstBool => {
            arg_type.qualifier == Qualifier::Const && arg_type.kind == ValueKind::Bool
        }
        Accepts::ConstOrInputFloat => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Input) && is_numeric(arg_type.kind)
        }
        Accepts::ColorCompatible => {
            matches!(arg_type.kind, ValueKind::Color | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::PlotOrHLine => matches!(arg_type.kind, ValueKind::Plot | ValueKind::HLine),
    }
}

fn can_assign(target: PineType, value: PineType) -> bool {
    if target.kind == value.kind {
        return qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series;
    }

    target.kind == ValueKind::Float
        && value.kind == ValueKind::Int
        && (qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series)
}

fn qualifier_at_most(actual: Qualifier, max: Qualifier) -> bool {
    qualifier_rank(actual) <= qualifier_rank(max)
}

fn strongest_qualifier(left: Qualifier, right: Qualifier) -> Qualifier {
    if qualifier_rank(left) >= qualifier_rank(right) {
        left
    } else {
        right
    }
}

fn qualifier_rank(qualifier: Qualifier) -> u8 {
    match qualifier {
        Qualifier::Const => 0,
        Qualifier::Input => 1,
        Qualifier::Simple => 2,
        Qualifier::Series => 3,
    }
}

fn is_numeric(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::Int | ValueKind::Float)
}

fn numeric_result_kind(op: BinaryOp, left: ValueKind, right: ValueKind) -> ValueKind {
    if op == BinaryOp::Div || left == ValueKind::Float || right == ValueKind::Float {
        ValueKind::Float
    } else {
        ValueKind::Int
    }
}

fn common_kind(left: ValueKind, right: ValueKind) -> Option<ValueKind> {
    if left == right {
        Some(left)
    } else if is_numeric(left) && is_numeric(right) {
        Some(ValueKind::Float)
    } else if left == ValueKind::Na {
        Some(right)
    } else if right == ValueKind::Na {
        Some(left)
    } else {
        None
    }
}

fn promoted_numeric_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut result: Option<PineType> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        if !is_numeric(arg_type.kind) {
            return None;
        }
        result = Some(match result {
            Some(current) => PineType::new(
                strongest_qualifier(current.qualifier, arg_type.qualifier),
                if current.kind == ValueKind::Float || arg_type.kind == ValueKind::Float {
                    ValueKind::Float
                } else {
                    ValueKind::Int
                },
            ),
            None => arg_type,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(text: &str) -> Analysis {
        analyze_source(&SourceFile::new("test.pine", text))
    }

    #[test]
    fn reports_supported_phase_1_calls() {
        let analysis = analyze("plot(ta.sma(close, 20))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "plot")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.sma")
        );
    }

    #[test]
    fn rejects_request_namespace() {
        let analysis = analyze("x = request.security(\"AAPL\", \"D\", close)\n");

        assert_eq!(analysis.compatibility.unsupported.len(), 1);
        assert_eq!(
            analysis.compatibility.unsupported[0].feature,
            "request.security"
        );
        assert_eq!(analysis.diagnostics[0].code, "E_UNSUPPORTED_FEATURE");
    }

    #[test]
    fn rejects_dynamic_history_offset() {
        let analysis = analyze("x = close[len]\n");

        assert_eq!(analysis.compatibility.unsupported.len(), 1);
        assert_eq!(
            analysis.compatibility.unsupported[0].feature,
            "dynamic_history_offset"
        );
    }

    #[test]
    fn accepts_constant_history_offset() {
        let analysis = analyze("x = close[1]\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
    }

    #[test]
    fn rejects_reassignment_to_unknown_symbol() {
        let analysis = analyze("x := x + 1\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_SYMBOL")
        );
    }

    #[test]
    fn accepts_reassignment_to_declared_symbol() {
        let analysis = analyze("x = close\nx := x + 1\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
    }

    #[test]
    fn rejects_wrong_builtin_argument_type() {
        let analysis = analyze("plot(ta.sma(close, close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE")
        );
    }

    #[test]
    fn rejects_missing_builtin_argument() {
        let analysis = analyze("plot()\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARITY")
        );
    }

    #[test]
    fn rejects_unknown_named_argument() {
        let analysis = analyze("indicator(\"Demo\", bogus=true)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_NAME")
        );
    }

    #[test]
    fn accepts_named_colors_and_color_new() {
        let analysis = analyze(
            r#"indicator("colors")
base = input.color(color.orange, "Base")
shade = color.new(base, 50)
plot(close, color=shade)
"#,
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.new")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.orange")
        );
    }

    #[test]
    fn rejects_unknown_named_color() {
        let analysis = analyze("plot(close, color=color.not_registered)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_COLOR")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_selected_math_functions() {
        let analysis = analyze(
            r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
plot(y)
"#,
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.max")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.min")
        );
    }

    #[test]
    fn lowers_if_statement_to_hir() {
        let analysis = analyze("if close > open\n    plot(close)\nelse\n    plot(open)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "if")
        );
        let hir = analysis.hir.expect("if statement should lower");
        assert!(matches!(hir.statements[0].kind, HirStmtKind::If { .. }));
    }

    #[test]
    fn reports_parse_only_function_as_unsupported() {
        let analysis = analyze("double(x) => x * 2\nplot(close)\n");

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn reports_parse_only_for_as_unsupported() {
        let analysis = analyze("for i = 0 to 10\nplot(close)\n");

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "for")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn lowers_valid_script_to_hir() {
        let analysis = analyze(
            r#"indicator("Demo", overlay=true)
length = input.int(20, "Length")
ma = ta.sma(close, length)
plot(ma)
"#,
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("valid script should lower to HIR");
        assert_eq!(hir.statements.len(), 4);
        assert!(hir.next_call_site_id >= 3);
        assert!(hir.next_series_id > 10);
    }

    #[test]
    fn lowers_var_declaration_to_var_slot() {
        let analysis = analyze("var x = 0\nx := x + 1\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("valid script should lower to HIR");
        let symbol = hir
            .symbols
            .iter()
            .find(|symbol| symbol.name == "x")
            .expect("x symbol should exist");
        assert_eq!(symbol.var_slot_id, Some(VarSlotId(0)));
    }

    #[test]
    fn skips_hir_when_semantic_errors_exist() {
        let analysis = analyze("plot()\n");

        assert!(analysis.hir.is_none());
    }

    #[test]
    fn lowers_tuple_assignment() {
        let analysis = analyze("[a, b] = [close, open]\nplot(a)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("valid tuple assignment should lower");
        assert!(
            hir.symbols
                .iter()
                .any(|symbol| symbol.name == "a" && symbol.series_id.is_some())
        );
    }

    #[test]
    fn compile_cache_reuses_analysis_for_identical_source() {
        let source = SourceFile::new("test.pine", "plot(close)\n");
        let mut cache = CompileCache::new();

        let first = cache.analyze(&source);
        let second = cache.analyze(&source);

        assert_eq!(first, second);
        assert_eq!(
            cache.stats(),
            CompileCacheStats {
                entries: 1,
                hits: 1,
                misses: 1,
            }
        );
    }

    #[test]
    fn compile_cache_keys_by_source_name_and_text() {
        let mut cache = CompileCache::new();

        cache.analyze(&SourceFile::new("one.pine", "plot(close)\n"));
        cache.analyze(&SourceFile::new("two.pine", "plot(close)\n"));
        cache.analyze(&SourceFile::new("one.pine", "plot(open)\n"));

        assert_eq!(
            cache.stats(),
            CompileCacheStats {
                entries: 3,
                hits: 0,
                misses: 3,
            }
        );
    }

    #[test]
    fn compile_cache_clear_drops_entries_and_stats() {
        let source = SourceFile::new("test.pine", "plot(close)\n");
        let mut cache = CompileCache::new();

        cache.analyze(&source);
        cache.analyze(&source);
        cache.clear();

        assert_eq!(
            cache.stats(),
            CompileCacheStats {
                entries: 0,
                hits: 0,
                misses: 0,
            }
        );
    }
}
