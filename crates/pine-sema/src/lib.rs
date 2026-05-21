//! Semantic analysis and compatibility gating scaffolding.

use std::collections::{BTreeMap, HashMap};

use pine_builtins::{Accepts, BuiltinSignature, ReturnSpec};
use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset,
    HirHistoryRequirements, HirLiteral, HirProgram, HirSeriesHistoryRequirement, HirStmt,
    HirStmtKind, HirSwitchArm, HirSymbol, HirUnaryOp, PineType, Qualifier, SeriesId, SymbolId,
    ValueKind, VarSlotId,
};
use pine_syntax::{
    BinaryOp, CallArg, Diagnostic, Expr, ExprKind, FunctionBody, Literal, Program, Severity,
    SourceFile, Span, Stmt, StmtKind, SwitchArm, UnaryOp, parse_source,
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
        bindings: HashMap::new(),
        lower_symbol_overrides: Vec::new(),
        functions: HashMap::new(),
        function_stack: Vec::new(),
        next_symbol_id: initial_symbol_count(),
        next_series_id: initial_series_count(),
        next_call_site_id: 0,
        next_var_slot_id: 0,
        block_depth: 0,
        function_depth: 0,
        loop_depth: 0,
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
    bindings: HashMap<BindingKey, SymbolInfo>,
    lower_symbol_overrides: Vec<HashMap<SymbolId, SymbolInfo>>,
    functions: HashMap<String, FunctionInfo>,
    function_stack: Vec<String>,
    next_symbol_id: u32,
    next_series_id: u32,
    next_call_site_id: u32,
    next_var_slot_id: u32,
    block_depth: u32,
    function_depth: u32,
    loop_depth: u32,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    params: Vec<String>,
    body: FunctionBody,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UdfArgError {
    UnknownName { name: String, span: Span },
    Duplicate { name: String, span: Span },
    PositionalAfterNamed { span: Span },
    TooMany { span: Span },
    Missing { param: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodResolution {
    NotMethod,
    Resolved(Option<PineType>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolInfo {
    id: SymbolId,
    pine_type: PineType,
    series_id: Option<SeriesId>,
    var_slot_id: Option<VarSlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BindingKey {
    span_start: usize,
    span_end: usize,
    name: String,
}

#[derive(Debug, Clone)]
struct ScopeResolver {
    scopes: Vec<HashMap<String, SymbolInfo>>,
    all_symbols: Vec<(String, SymbolInfo)>,
}

impl ScopeResolver {
    fn new(global_symbols: HashMap<String, SymbolInfo>, symbol_order: Vec<String>) -> Self {
        let all_symbols = symbol_order
            .iter()
            .filter_map(|name| {
                global_symbols
                    .get(name)
                    .copied()
                    .map(|symbol| (name.clone(), symbol))
            })
            .collect();
        Self {
            scopes: vec![global_symbols],
            all_symbols,
        }
    }

    fn resolve(&self, name: &str) -> Option<SymbolInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn resolves_to_global(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
            .is_some_and(|index| index == 0)
    }

    fn define_global(&mut self, name: &str, info: SymbolInfo) {
        let global_scope = self
            .scopes
            .first_mut()
            .expect("scope resolver always has a global scope");
        if !global_scope.contains_key(name) {
            self.all_symbols.push((name.to_owned(), info));
        } else if let Some((_, symbol)) = self
            .all_symbols
            .iter_mut()
            .find(|(_, symbol)| symbol.id == info.id)
        {
            *symbol = info;
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
        if let Some((_, symbol)) = self
            .all_symbols
            .iter_mut()
            .find(|(_, symbol)| symbol.id == info.id)
        {
            *symbol = info;
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define_local(&mut self, name: &str, info: SymbolInfo, lower: bool) {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope resolver always has a current scope");
        scope.insert(name.to_owned(), info);
        if lower {
            self.all_symbols.push((name.to_owned(), info));
        }
    }

    fn lower_symbols(&self) -> Vec<HirSymbol> {
        self.all_symbols
            .iter()
            .map(|(name, symbol)| HirSymbol {
                id: symbol.id,
                name: name.clone(),
                pine_type: symbol.pine_type,
                series_id: symbol.series_id,
                var_slot_id: symbol.var_slot_id,
            })
            .collect()
    }

    fn contains_lower_symbol(&self, id: SymbolId) -> bool {
        self.all_symbols.iter().any(|(_, symbol)| symbol.id == id)
    }

    fn add_lower_symbol(&mut self, name: &str, info: SymbolInfo) {
        self.all_symbols.push((name.to_owned(), info));
    }
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) {
        self.register_functions(program);
        for statement in &program.statements {
            self.analyze_stmt(statement);
        }
    }

    fn register_functions(&mut self, program: &Program) {
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

                self.block_depth += 1;
                self.scope.push_scope();
                for branch_statement in then_branch {
                    self.analyze_stmt(branch_statement);
                }
                self.scope.pop_scope();
                self.scope.push_scope();
                for branch_statement in else_branch {
                    self.analyze_stmt(branch_statement);
                }
                self.scope.pop_scope();
                self.block_depth -= 1;
            }
            StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                let from_type = self.analyze_expr(from);
                let to_type = self.analyze_expr(to);
                let step_type = step.as_ref().and_then(|step| self.analyze_expr(step));
                if let Some(from_type) = from_type {
                    self.expect_int(from_type, from.span);
                }
                if let Some(to_type) = to_type {
                    self.expect_int(to_type, to.span);
                }
                if let Some((step, step_type)) = step.as_ref().zip(step_type) {
                    self.expect_int(step_type, step.span);
                    self.expect_non_zero_loop_step(step);
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "for".to_owned(),
                    span: statement.span,
                });

                let counter_type = PineType::new(
                    strongest_qualifier(
                        from_type.unwrap_or(UNKNOWN).qualifier,
                        to_type.unwrap_or(UNKNOWN).qualifier,
                    ),
                    ValueKind::Int,
                );
                self.block_depth += 1;
                self.loop_depth += 1;
                self.scope.push_scope();
                let counter_symbol =
                    self.define_local_symbol(counter, counter_type, None, self.function_depth == 0);
                self.bind_symbol(counter, statement.span, counter_symbol);
                for body_statement in body {
                    self.analyze_stmt(body_statement);
                }
                self.scope.pop_scope();
                self.loop_depth -= 1;
                self.block_depth -= 1;
            }
            StmtKind::While { condition, body } => {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    self.expect_bool(condition_type, condition.span);
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "while".to_owned(),
                    span: statement.span,
                });

                self.block_depth += 1;
                self.loop_depth += 1;
                self.scope.push_scope();
                for body_statement in body {
                    self.analyze_stmt(body_statement);
                }
                self.scope.pop_scope();
                self.loop_depth -= 1;
                self.block_depth -= 1;
            }
            StmtKind::Break => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "E_LOOP_CONTROL",
                        "`break` can only be used inside a loop",
                        statement.span,
                    ));
                }
            }
            StmtKind::Continue => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "E_LOOP_CONTROL",
                        "`continue` can only be used inside a loop",
                        statement.span,
                    ));
                }
            }
            StmtKind::Function { .. } => {
                if self.block_depth > 0 || self.function_depth > 0 {
                    self.unsupported(
                        "block_local_function",
                        "nested function declarations are not supported",
                        statement.span,
                    );
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
                let symbol = if self.block_depth > 0 || self.function_depth > 0 {
                    self.define_local_symbol(
                        name,
                        value_type,
                        var_slot_id,
                        self.function_depth == 0,
                    )
                } else {
                    self.define_symbol(name, value_type, var_slot_id)
                };
                self.bind_symbol(name, statement.span, symbol);
            }
            StmtKind::Reassign { name, value } => {
                if self.scope.resolve(name).is_none() {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UNKNOWN_SYMBOL",
                        format!("cannot reassign unknown symbol `{name}`"),
                        statement.span,
                    ));
                } else if self.function_depth > 0 && self.scope.resolves_to_global(name) {
                    self.unsupported(
                        "function_side_effect",
                        "reassigning global variables inside user-defined functions is not supported",
                        statement.span,
                    );
                }
                let value_type = self.analyze_expr(value);
                if let (Some(target_type), Some(value_type)) = (
                    self.scope.resolve(name).map(|symbol| symbol.pine_type),
                    value_type,
                ) {
                    self.validate_assignment(name, target_type, value_type, statement.span);
                    self.update_symbol_type(name, value_type);
                }
                if let Some(symbol) = self.scope.resolve(name) {
                    self.bind_symbol(name, statement.span, symbol);
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

        if self.block_depth > 0 || self.function_depth > 0 {
            for (name, pine_type) in names.iter().zip(element_types) {
                let symbol =
                    self.define_local_symbol(name, pine_type, None, self.function_depth == 0);
                self.bind_symbol(name, statement.span, symbol);
            }
        } else {
            for (name, pine_type) in names.iter().zip(element_types) {
                self.define_symbol(name, pine_type, None);
                if let Some(symbol) = self.scope.resolve(name) {
                    self.bind_symbol(name, statement.span, symbol);
                }
            }
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
            ExprKind::Switch { selector, arms } => {
                self.analyze_switch_expr(selector.as_deref(), arms, expr.span)
            }
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.analyze_for_expr(counter, from, to, step.as_deref(), body, expr.span),
            ExprKind::Tuple(items) => {
                for item in items {
                    self.analyze_expr(item);
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => self.analyze_call(callee, args),
            ExprKind::History { expr, offset } => {
                let value_type = self.analyze_expr(expr);
                let offset_type = self.analyze_expr(offset);
                self.validate_history_offset(offset, offset_type);
                value_type.map(|value_type| PineType::new(Qualifier::Series, value_type.kind))
            }
        }
    }

    fn analyze_switch_expr(
        &mut self,
        selector: Option<&Expr>,
        arms: &[SwitchArm],
        span: Span,
    ) -> Option<PineType> {
        let selector_type = selector.and_then(|selector| self.analyze_expr(selector));
        let mut condition_qualifier = selector_type.map_or(Qualifier::Const, |ty| ty.qualifier);
        let mut result_type = None;
        let mut has_type_error = false;

        self.compatibility.supported.push(FeatureUse {
            feature: "switch".to_owned(),
            span,
        });

        for arm in arms {
            if let Some(condition) = &arm.condition {
                let condition_type = self.analyze_expr(condition);
                if let Some(condition_type) = condition_type {
                    condition_qualifier =
                        strongest_qualifier(condition_qualifier, condition_type.qualifier);
                    if selector.is_none() {
                        self.expect_bool(condition_type, condition.span);
                    }
                }
            }

            if let Some(arm_type) = self.analyze_expr(&arm.result) {
                match merge_result_types(result_type, arm_type) {
                    Some(merged) => result_type = Some(merged),
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            "E_BRANCH_TYPE",
                            format!(
                                "switch arms have incompatible types {:?} and {:?}",
                                result_type.unwrap_or(UNKNOWN).kind,
                                arm_type.kind
                            ),
                            span,
                        ));
                        has_type_error = true;
                    }
                }
            }
        }

        if has_type_error {
            return None;
        }

        result_type.map(|pine_type| {
            PineType::new(
                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                pine_type.kind,
            )
        })
    }

    fn analyze_for_expr(
        &mut self,
        counter: &str,
        from: &Expr,
        to: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
        span: Span,
    ) -> Option<PineType> {
        let from_type = self.analyze_expr(from);
        let to_type = self.analyze_expr(to);
        let step_type = step.and_then(|step| self.analyze_expr(step));
        if let Some(from_type) = from_type {
            self.expect_int(from_type, from.span);
        }
        if let Some(to_type) = to_type {
            self.expect_int(to_type, to.span);
        }
        if let Some((step, step_type)) = step.zip(step_type) {
            self.expect_int(step_type, step.span);
            self.expect_non_zero_loop_step(step);
        }

        self.compatibility.supported.push(FeatureUse {
            feature: "for".to_owned(),
            span,
        });

        let counter_type = PineType::new(
            strongest_qualifier(
                from_type.unwrap_or(UNKNOWN).qualifier,
                to_type.unwrap_or(UNKNOWN).qualifier,
            ),
            ValueKind::Int,
        );
        self.block_depth += 1;
        self.loop_depth += 1;
        self.scope.push_scope();
        let counter_symbol =
            self.define_local_symbol(counter, counter_type, None, self.function_depth == 0);
        self.bind_symbol(counter, span, counter_symbol);

        let return_type = if let Some((last, prefix)) = body.split_last() {
            for statement in prefix {
                self.analyze_stmt(statement);
            }
            match &last.kind {
                StmtKind::Expr(expr) => self.analyze_expr(expr),
                _ => {
                    self.analyze_stmt(last);
                    self.diagnostics.push(Diagnostic::error(
                        "E_LOOP_RETURN",
                        "for expression body must end with an expression",
                        last.span,
                    ));
                    None
                }
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RETURN",
                "for expression body must end with an expression",
                span,
            ));
            None
        };

        self.scope.pop_scope();
        self.loop_depth -= 1;
        self.block_depth -= 1;
        return_type
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

        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();

        if let Some(signature) = pine_builtins::get_phase_1_builtin(&name) {
            self.check_feature_name(&name, callee.span);
            if self.function_depth > 0 && is_output_or_declaration_builtin(&name) {
                self.unsupported(
                    "function_side_effect",
                    "indicator, input, plot, plotchar, plotshape, plotarrow, plotbar, plotcandle, hline, fill, bgcolor, and barcolor calls are not supported inside user-defined functions",
                    callee.span,
                );
            }
            if self.function_depth > 0 && is_array_mutation_builtin(&name) {
                self.unsupported(
                    "function_side_effect",
                    "array mutation is not supported inside user-defined functions",
                    callee.span,
                );
            }

            self.validate_call_args(signature, args, &arg_types);
            return self.return_type(signature, &arg_types);
        }

        match self.analyze_method_call(callee, args, &arg_types) {
            MethodResolution::Resolved(pine_type) => return pine_type,
            MethodResolution::NotMethod => {}
        }

        if self.functions.contains_key(&name) {
            return self.analyze_udf_call(&name, callee.span, args, &arg_types);
        }

        self.check_feature_name(&name, callee.span);
        self.diagnostics.push(Diagnostic::error(
            "E_UNKNOWN_FUNCTION",
            format!("unknown function `{name}`"),
            callee.span,
        ));
        None
    }

    fn analyze_method_call(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> MethodResolution {
        let Some((receiver_name, method_name)) = method_call_parts(callee) else {
            return MethodResolution::NotMethod;
        };
        if self.scope.resolve(receiver_name).is_none() {
            return MethodResolution::NotMethod;
        }

        let receiver_arg = receiver_call_arg(receiver_name, callee.span);
        let receiver_type = self.analyze_expr(&receiver_arg.value);
        let Some(receiver_type) = receiver_type else {
            return MethodResolution::Resolved(None);
        };
        if !is_array_kind(receiver_type.kind) {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_RECEIVER_TYPE",
                format!(
                    "method `{method_name}` is not supported for {:?} {:?}",
                    receiver_type.qualifier, receiver_type.kind
                ),
                callee.span,
            ));
            return MethodResolution::Resolved(None);
        }

        let builtin_name = array_method_builtin_name(method_name);
        let Some(signature) = builtin_name
            .and_then(|name| pine_builtins::get_phase_1_builtin(name).map(|sig| (name, sig)))
        else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown array method `{method_name}`"),
                callee.span,
            ));
            return MethodResolution::Resolved(None);
        };
        let (builtin_name, signature) = signature;
        self.check_feature_name(builtin_name, callee.span);

        if self.function_depth > 0 && is_array_mutation_builtin(builtin_name) {
            self.unsupported(
                "function_side_effect",
                "array mutation is not supported inside user-defined functions",
                callee.span,
            );
        }

        let mut method_args = Vec::with_capacity(args.len() + 1);
        method_args.push(receiver_arg);
        method_args.extend(args.iter().cloned());
        let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
        method_arg_types.push(Some(receiver_type));
        method_arg_types.extend(arg_types.iter().copied());

        self.validate_call_args(signature, &method_args, &method_arg_types);
        MethodResolution::Resolved(self.return_type(signature, &method_arg_types))
    }

    fn analyze_udf_call(
        &mut self,
        name: &str,
        span: Span,
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
        self.scope.push_scope();
        let mut resolved_arg_types = vec![None; function.params.len()];
        for (arg_index, param_index) in arg_indices.iter().copied().enumerate() {
            resolved_arg_types[param_index] = arg_types.get(arg_index).copied().flatten();
        }
        for (param, arg_type) in function.params.iter().zip(resolved_arg_types) {
            self.define_local_symbol(param, arg_type.unwrap_or(UNKNOWN), None, false);
        }
        self.function_stack.push(name.to_owned());
        self.function_depth += 1;
        let return_type = self.analyze_function_body(&function.body, function.span);
        self.function_depth -= 1;
        self.function_stack.pop();
        self.scope.pop_scope();

        return_type
    }

    fn analyze_function_body(&mut self, body: &FunctionBody, span: Span) -> Option<PineType> {
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

    fn report_udf_arg_error(
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

    fn validate_history_offset(&mut self, offset: &Expr, offset_type: Option<PineType>) {
        if let Some(value) = const_int_value(offset) {
            if value < 0 {
                self.unsupported(
                    "negative_history_offset",
                    "history offsets must be non-negative in the current supported subset",
                    offset.span,
                );
            }
            return;
        }

        let Some(offset_type) = offset_type else {
            self.unsupported(
                "dynamic_history_offset",
                "dynamic history offsets require an integer expression in the current supported subset",
                offset.span,
            );
            return;
        };

        if offset_type.kind == ValueKind::Int {
            return;
        }

        self.unsupported(
            "dynamic_history_offset",
            "dynamic history offsets require an integer expression in the current supported subset",
            offset.span,
        );
    }

    fn check_feature_expr(&mut self, expr: &Expr) {
        let Some(name) = expr_name(expr) else {
            return;
        };
        self.check_feature_name(&name, expr.span);
    }

    fn check_feature_name(&mut self, name: &str, span: Span) {
        let unsupported_reason = if pine_builtins::is_phase_1_builtin(name) {
            None
        } else if name.starts_with("strategy.") {
            Some("strategy backtesting and broker emulation are outside the current runtime scope")
        } else if name.starts_with("request.") {
            Some("multi-symbol and multi-timeframe data requests are not supported in Phase 1")
        } else if name.starts_with("array.") {
            Some("this array function is not supported in the current partial array subset")
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

        self.validate_array_value_args(signature, args, arg_types);
        self.validate_array_concat_args(signature, args, arg_types);
        self.validate_array_from_args(signature, args, arg_types);
        self.validate_indicator_args(signature, args);
    }

    fn validate_indicator_args(&mut self, signature: &BuiltinSignature, args: &[CallArg]) {
        if signature.name != "indicator" {
            return;
        }

        for (index, arg) in args.iter().enumerate() {
            let is_max_bars_back = arg.name.as_deref() == Some("max_bars_back")
                || (arg.name.is_none()
                    && signature
                        .params
                        .get(index)
                        .is_some_and(|param| param.name == "max_bars_back"));
            if !is_max_bars_back {
                continue;
            }

            if let Some(value) = const_int_value(&arg.value)
                && value < 0
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    "`indicator` argument `max_bars_back` must be non-negative",
                    arg.span,
                ));
            }
        }
    }

    fn validate_array_value_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        let value_index = match signature.name {
            "array.push"
            | "array.unshift"
            | "array.fill"
            | "array.includes"
            | "array.indexof"
            | "array.lastindexof"
            | "array.binary_search"
            | "array.binary_search_leftmost"
            | "array.binary_search_rightmost" => 1,
            "array.set" | "array.insert" => 2,
            _ => return,
        };
        let Some(array_type) = arg_types.first().copied().flatten() else {
            return;
        };
        let Some(value_type) = arg_types.get(value_index).copied().flatten() else {
            return;
        };
        let expected = match array_type.kind {
            ValueKind::FloatArray
                if matches!(
                    value_type.kind,
                    ValueKind::Int | ValueKind::Float | ValueKind::Na
                ) =>
            {
                return;
            }
            ValueKind::IntArray if matches!(value_type.kind, ValueKind::Int | ValueKind::Na) => {
                return;
            }
            ValueKind::BoolArray if matches!(value_type.kind, ValueKind::Bool | ValueKind::Na) => {
                return;
            }
            ValueKind::StringArray
                if matches!(value_type.kind, ValueKind::String | ValueKind::Na) =>
            {
                return;
            }
            ValueKind::ColorArray
                if matches!(value_type.kind, ValueKind::Color | ValueKind::Na) =>
            {
                return;
            }
            ValueKind::FloatArray => "float arrays",
            ValueKind::IntArray => "int arrays",
            ValueKind::BoolArray => "bool arrays",
            ValueKind::StringArray => "string arrays",
            ValueKind::ColorArray => "color arrays",
            _ => return,
        };

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            format!(
                "`{}` argument `value` does not accept {:?} {:?} for {expected}",
                signature.name, value_type.qualifier, value_type.kind,
            ),
            args.get(value_index)
                .map_or(Span::default(), |arg| arg.span),
        ));
    }

    fn validate_array_concat_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if signature.name != "array.concat" {
            return;
        }
        let Some(first_type) = arg_types.first().copied().flatten() else {
            return;
        };
        let Some(second_type) = arg_types.get(1).copied().flatten() else {
            return;
        };
        if !is_array_kind(first_type.kind)
            || !is_array_kind(second_type.kind)
            || first_type.kind == second_type.kind
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            format!(
                "`array.concat` argument `id2` does not accept {:?} {:?} for {:?} {:?}",
                second_type.qualifier, second_type.kind, first_type.qualifier, first_type.kind,
            ),
            args.get(1).map_or(Span::default(), |arg| arg.span),
        ));
    }

    fn validate_array_from_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if signature.name != "array.from" {
            return;
        }
        if array_from_return_type(arg_types).is_some() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            "`array.from` arguments must infer one supported array element kind",
            args.first().map_or(Span::default(), |arg| arg.span),
        ));
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
            ReturnSpec::PromotedColor => promoted_color_type(arg_types),
            ReturnSpec::PromotedBool => promoted_bool_type(arg_types),
            ReturnSpec::PromotedInt => promoted_int_type(arg_types),
            ReturnSpec::PromotedString => promoted_string_type(arg_types),
            ReturnSpec::FloatFromStringArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::PromotedNumeric => promoted_numeric_type(arg_types),
            ReturnSpec::ArrayElement(index) => array_element_return_type(arg_types, index),
            ReturnSpec::ArrayNumeric(index) => array_numeric_return_type(arg_types, index),
            ReturnSpec::ArrayFromArgs => array_from_return_type(arg_types),
            ReturnSpec::IntFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(int_return_for_arg),
            ReturnSpec::FloatFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::SeriesFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(series_return_for_arg),
            ReturnSpec::PromotedFloat => promoted_float_type(arg_types),
            ReturnSpec::Round => round_return_type(arg_types),
            ReturnSpec::InputFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(pine_builtins::input_return_for_arg),
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
        if pine_builtins::named_float_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Float));
        }
        if pine_builtins::named_string_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::String));
        }
        if let Some(pine_type) = pine_builtins::builtin_series_value_type(name) {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(pine_type);
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
            self.bind_symbol(name, span, symbol);
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
    ) -> SymbolInfo {
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
            return updated;
        }

        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            var_slot_id,
        };
        self.scope.define_global(name, info);
        info
    }

    fn define_local_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
        lower: bool,
    ) -> SymbolInfo {
        let series_id = self.series_id_for_type(pine_type);
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id,
            var_slot_id,
        };
        self.scope.define_local(name, info, lower);
        info
    }

    fn fresh_lower_symbol(&mut self, name: &str, original: SymbolInfo) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type: original.pine_type,
            series_id: self.series_id_for_type(original.pine_type),
            var_slot_id: original.var_slot_id.map(|_| self.alloc_var_slot()),
        };
        self.scope.add_lower_symbol(name, info);
        info
    }

    fn fresh_temp_symbol(&mut self, name: &str, pine_type: PineType) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            var_slot_id: None,
        };
        self.scope.add_lower_symbol(name, info);
        info
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

    fn expect_int(&mut self, pine_type: PineType, span: Span) {
        if pine_type.kind != ValueKind::Int {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RANGE_TYPE",
                format!(
                    "for loop range must be int, got {:?} {:?}",
                    pine_type.qualifier, pine_type.kind
                ),
                span,
            ));
        }
    }

    fn expect_non_zero_loop_step(&mut self, step: &Expr) {
        if const_int_value(step) == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_STEP",
                "for loop step cannot be zero",
                step.span,
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
            if matches!(statement.kind, StmtKind::Function { .. }) {
                continue;
            }
            statements.push(self.lower_stmt(statement)?);
        }

        let symbols = self.lower_symbols();
        let history = infer_history_requirements(&statements, &symbols);
        let max_bars_back = infer_max_bars_back(&statements);
        Some(HirProgram {
            symbols,
            statements,
            next_series_id: self.next_series_id,
            next_call_site_id: self.next_call_site_id,
            next_var_slot_id: self.next_var_slot_id,
            max_bars_back,
            history: history.program,
            series_history: history.series,
        })
    }

    fn lower_symbols(&self) -> Vec<HirSymbol> {
        self.scope.lower_symbols()
    }

    fn bind_symbol(&mut self, name: &str, span: Span, symbol: SymbolInfo) {
        self.bindings.insert(binding_key(name, span), symbol);
    }

    fn bound_symbol(&self, name: &str, span: Span) -> Option<SymbolInfo> {
        let symbol = self.bindings.get(&binding_key(name, span)).copied()?;
        self.lower_symbol_overrides
            .iter()
            .rev()
            .find_map(|overrides| overrides.get(&symbol.id).copied())
            .or(Some(symbol))
    }

    fn has_lower_symbol_override(&self, symbol_id: SymbolId) -> bool {
        self.lower_symbol_overrides
            .iter()
            .rev()
            .any(|overrides| overrides.contains_key(&symbol_id))
    }

    fn lower_decl_symbol(&mut self, name: &str, span: Span) -> Option<SymbolInfo> {
        let symbol = self.bindings.get(&binding_key(name, span)).copied()?;
        if self.lower_symbol_overrides.is_empty() || self.scope.contains_lower_symbol(symbol.id) {
            return Some(symbol);
        }
        if let Some(existing) = self
            .lower_symbol_overrides
            .iter()
            .rev()
            .find_map(|overrides| overrides.get(&symbol.id).copied())
        {
            return Some(existing);
        }
        let fresh = self.fresh_lower_symbol(name, symbol);
        self.lower_symbol_overrides
            .last_mut()
            .expect("override scope is active")
            .insert(symbol.id, fresh);
        Some(fresh)
    }

    fn lower_stmt(&mut self, statement: &pine_syntax::Stmt) -> Option<HirStmt> {
        self.lower_stmt_with_params(statement, &HashMap::new(), &HashMap::new())
    }

    fn lower_stmt_with_params(
        &mut self,
        statement: &pine_syntax::Stmt,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirStmt> {
        let kind = match &statement.kind {
            StmtKind::Expr(expr) => {
                HirStmtKind::Expr(self.lower_expr_with_params(expr, param_exprs, param_types)?)
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => HirStmtKind::If {
                condition: self.lower_expr_with_params(condition, param_exprs, param_types)?,
                then_branch: then_branch
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
                else_branch: else_branch
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => HirStmtKind::For {
                counter: self.lower_decl_symbol(counter, statement.span)?.id,
                from: self.lower_expr_with_params(from, param_exprs, param_types)?,
                to: self.lower_expr_with_params(to, param_exprs, param_types)?,
                step: match step {
                    Some(step) => {
                        Some(self.lower_expr_with_params(step, param_exprs, param_types)?)
                    }
                    None => None,
                },
                body: body
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::While { condition, body } => HirStmtKind::While {
                condition: self.lower_expr_with_params(condition, param_exprs, param_types)?,
                body: body
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::Break => HirStmtKind::Break,
            StmtKind::Continue => HirStmtKind::Continue,
            StmtKind::Decl { name, value, .. } => HirStmtKind::Decl {
                symbol: self.lower_decl_symbol(name, statement.span)?.id,
                value: self.lower_expr_with_params(value, param_exprs, param_types)?,
            },
            StmtKind::Reassign { name, value } => HirStmtKind::Reassign {
                symbol: self.bound_symbol(name, statement.span)?.id,
                value: self.lower_expr_with_params(value, param_exprs, param_types)?,
            },
            StmtKind::TupleDecl { names, value } => HirStmtKind::TupleDecl {
                symbols: names
                    .iter()
                    .map(|name| {
                        self.lower_decl_symbol(name, statement.span)
                            .map(|symbol| symbol.id)
                    })
                    .collect::<Option<_>>()?,
                value: self.lower_expr_with_params(value, param_exprs, param_types)?,
            },
            StmtKind::Function { .. } => return None,
            StmtKind::Unsupported { .. } => return None,
        };

        Some(HirStmt { kind })
    }

    fn lower_expr_with_params(
        &mut self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        if let ExprKind::Identifier(name) = &expr.kind
            && let Some(param_expr) = param_exprs.get(name)
            && self
                .bindings
                .get(&binding_key(name, expr.span))
                .is_none_or(|symbol| !self.has_lower_symbol_override(symbol.id))
        {
            return Some(param_expr.clone());
        }

        let pine_type = self.type_of_expr_with_params(expr, param_types)?;
        let series_id =
            if pine_type.qualifier == Qualifier::Series && pine_type.kind != ValueKind::Tuple {
                match &expr.kind {
                    ExprKind::Identifier(name) => self
                        .bound_symbol(name, expr.span)
                        .and_then(|symbol| symbol.series_id),
                    _ => Some(self.alloc_series()),
                }
            } else {
                None
            };

        let kind = match &expr.kind {
            ExprKind::Literal(literal) => HirExprKind::Literal(lower_literal(literal)),
            ExprKind::Identifier(name) => {
                HirExprKind::Symbol(self.bound_symbol(name, expr.span)?.id)
            }
            ExprKind::QualifiedName(parts) => HirExprKind::Builtin(parts.join(".")),
            ExprKind::Unary { op, expr } => HirExprKind::Unary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr_with_params(expr, param_exprs, param_types)?),
            },
            ExprKind::Binary { op, left, right } => HirExprKind::Binary {
                op: lower_binary_op(*op),
                left: Box::new(self.lower_expr_with_params(left, param_exprs, param_types)?),
                right: Box::new(self.lower_expr_with_params(right, param_exprs, param_types)?),
            },
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => HirExprKind::Ternary {
                condition: Box::new(self.lower_expr_with_params(
                    condition,
                    param_exprs,
                    param_types,
                )?),
                then_expr: Box::new(self.lower_expr_with_params(
                    then_expr,
                    param_exprs,
                    param_types,
                )?),
                else_expr: Box::new(self.lower_expr_with_params(
                    else_expr,
                    param_exprs,
                    param_types,
                )?),
            },
            ExprKind::Switch { selector, arms } => HirExprKind::Switch {
                selector: match selector {
                    Some(selector) => Some(Box::new(self.lower_expr_with_params(
                        selector,
                        param_exprs,
                        param_types,
                    )?)),
                    None => None,
                },
                arms: arms
                    .iter()
                    .map(|arm| {
                        Some(HirSwitchArm {
                            condition: match &arm.condition {
                                Some(condition) => Some(self.lower_expr_with_params(
                                    condition,
                                    param_exprs,
                                    param_types,
                                )?),
                                None => None,
                            },
                            result: self.lower_expr_with_params(
                                &arm.result,
                                param_exprs,
                                param_types,
                            )?,
                        })
                    })
                    .collect::<Option<_>>()?,
            },
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                let (last, prefix) = body.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                HirExprKind::For {
                    counter: self.lower_decl_symbol(counter, expr.span)?.id,
                    from: Box::new(self.lower_expr_with_params(from, param_exprs, param_types)?),
                    to: Box::new(self.lower_expr_with_params(to, param_exprs, param_types)?),
                    step: match step {
                        Some(step) => Some(Box::new(self.lower_expr_with_params(
                            step,
                            param_exprs,
                            param_types,
                        )?)),
                        None => None,
                    },
                    statements: prefix
                        .iter()
                        .map(|statement| {
                            self.lower_stmt_with_params(statement, param_exprs, param_types)
                        })
                        .collect::<Option<_>>()?,
                    result: Box::new(self.lower_expr_with_params(
                        result,
                        param_exprs,
                        param_types,
                    )?),
                }
            }
            ExprKind::Tuple(items) => HirExprKind::Tuple(
                items
                    .iter()
                    .map(|item| self.lower_expr_with_params(item, param_exprs, param_types))
                    .collect::<Option<_>>()?,
            ),
            ExprKind::Call { callee, args } => {
                let name = expr_name(callee)?;
                if self.functions.contains_key(&name) {
                    return self.lower_udf_call(&name, args, param_exprs, param_types);
                }
                if pine_builtins::get_phase_1_builtin(&name).is_none()
                    && let Some((receiver_name, method_name)) = method_call_parts(callee)
                    && let Some(builtin_name) = array_method_builtin_name(method_name)
                {
                    let mut lowered_args = Vec::with_capacity(args.len() + 1);
                    let receiver_arg = receiver_call_arg(receiver_name, callee.span);
                    lowered_args.push(HirCallArg {
                        name: None,
                        value: self.lower_expr_with_params(
                            &receiver_arg.value,
                            param_exprs,
                            param_types,
                        )?,
                    });
                    lowered_args.extend(
                        args.iter()
                            .map(|arg| {
                                Some(HirCallArg {
                                    name: arg.name.clone(),
                                    value: self.lower_expr_with_params(
                                        &arg.value,
                                        param_exprs,
                                        param_types,
                                    )?,
                                })
                            })
                            .collect::<Option<Vec<_>>>()?,
                    );
                    return Some(HirExpr {
                        pine_type,
                        series_id,
                        kind: HirExprKind::Call {
                            callee: builtin_name.to_owned(),
                            call_site_id: self.alloc_call_site(),
                            args: lowered_args,
                        },
                    });
                }
                HirExprKind::Call {
                    callee: name,
                    call_site_id: self.alloc_call_site(),
                    args: args
                        .iter()
                        .map(|arg| {
                            Some(HirCallArg {
                                name: arg.name.clone(),
                                value: self.lower_expr_with_params(
                                    &arg.value,
                                    param_exprs,
                                    param_types,
                                )?,
                            })
                        })
                        .collect::<Option<_>>()?,
                }
            }
            ExprKind::History { expr, offset } => {
                let offset = match constant_history_offset(offset) {
                    Some(offset) => HirHistoryOffset::Constant(offset),
                    None => HirHistoryOffset::Dynamic(Box::new(self.lower_expr_with_params(
                        offset,
                        param_exprs,
                        param_types,
                    )?)),
                };
                HirExprKind::History {
                    expr: Box::new(self.lower_expr_with_params(expr, param_exprs, param_types)?),
                    offset,
                }
            }
        };

        Some(HirExpr {
            kind,
            pine_type,
            series_id,
        })
    }

    fn lower_udf_call(
        &mut self,
        name: &str,
        args: &[CallArg],
        outer_param_exprs: &HashMap<String, HirExpr>,
        outer_param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let function = self.functions.get(name)?.clone();
        let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
        let mut resolved_args = vec![None; function.params.len()];
        for (arg, param_index) in args.iter().zip(arg_indices) {
            let arg_expr =
                self.lower_expr_with_params(&arg.value, outer_param_exprs, outer_param_types)?;
            let arg_type = self.type_of_expr_with_params(&arg.value, outer_param_types)?;
            resolved_args[param_index] = Some((arg_expr, arg_type));
        }

        let mut param_exprs = HashMap::new();
        let mut param_types = HashMap::new();
        let mut arg_statements = Vec::new();
        for (param, resolved_arg) in function.params.iter().zip(resolved_args) {
            let (arg_expr, arg_type) = resolved_arg?;
            let symbol = self.fresh_temp_symbol(&format!("{name}.{param}"), arg_type);
            arg_statements.push(HirStmt {
                kind: HirStmtKind::Decl {
                    symbol: symbol.id,
                    value: arg_expr,
                },
            });
            param_exprs.insert(
                param.clone(),
                HirExpr {
                    kind: HirExprKind::Symbol(symbol.id),
                    pine_type: arg_type,
                    series_id: symbol.series_id,
                },
            );
            param_types.insert(param.clone(), arg_type);
        }
        let body = self.lower_function_body(&function.body, &param_exprs, &param_types)?;
        Some(prepend_block_statements(arg_statements, body))
    }

    fn lower_function_body(
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
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                self.lower_symbol_overrides.push(HashMap::new());
                let lowered_statements = prefix
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
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
        }
    }

    fn type_of_expr(&self, expr: &Expr) -> Option<PineType> {
        self.type_of_expr_with_params(expr, &HashMap::new())
    }

    fn type_of_expr_with_params(
        &self,
        expr: &Expr,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        match &expr.kind {
            ExprKind::Literal(literal) => Some(literal_type(literal)),
            ExprKind::Identifier(name) => param_types
                .get(name)
                .copied()
                .or_else(|| {
                    self.bound_symbol(name, expr.span)
                        .map(|symbol| symbol.pine_type)
                })
                .or_else(|| self.scope.resolve(name).map(|symbol| symbol.pine_type)),
            ExprKind::QualifiedName(_) => {
                let name = expr_name(expr)?;
                pine_builtins::named_color(&name)
                    .map(|_| PineType::new(Qualifier::Const, ValueKind::Color))
                    .or_else(|| {
                        pine_builtins::named_float_constant(&name)
                            .map(|_| PineType::new(Qualifier::Const, ValueKind::Float))
                    })
                    .or_else(|| {
                        pine_builtins::named_string_constant(&name)
                            .map(|_| PineType::new(Qualifier::Const, ValueKind::String))
                    })
                    .or_else(|| pine_builtins::builtin_series_value_type(&name))
            }
            ExprKind::Unary { expr, .. } => self.type_of_expr_with_params(expr, param_types),
            ExprKind::Binary { op, left, right } => {
                let left_type = self.type_of_expr_with_params(left, param_types)?;
                let right_type = self.type_of_expr_with_params(right, param_types)?;
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
                let condition_type = self.type_of_expr_with_params(condition, param_types)?;
                let then_type = self.type_of_expr_with_params(then_expr, param_types)?;
                let else_type = self.type_of_expr_with_params(else_expr, param_types)?;
                Some(PineType::new(
                    strongest_qualifier(
                        condition_type.qualifier,
                        strongest_qualifier(then_type.qualifier, else_type.qualifier),
                    ),
                    common_kind(then_type.kind, else_type.kind)?,
                ))
            }
            ExprKind::Switch { selector, arms } => {
                self.type_of_switch_expr_with_params(selector.as_deref(), arms, param_types)
            }
            ExprKind::For { body, .. } => {
                let last = body.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.type_of_expr_with_params(expr, param_types)
            }
            ExprKind::Tuple(items) => {
                for item in items {
                    self.type_of_expr_with_params(item, param_types)?;
                }
                Some(pine_builtins::tuple_return_type())
            }
            ExprKind::Call { callee, args } => {
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| self.type_of_expr_with_params(&arg.value, param_types))
                    .collect();
                let name = expr_name(callee)?;
                if let Some(signature) = pine_builtins::get_phase_1_builtin(&name) {
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
                        ReturnSpec::PromotedColor => promoted_color_type(&arg_types),
                        ReturnSpec::PromotedBool => promoted_bool_type(&arg_types),
                        ReturnSpec::PromotedInt => promoted_int_type(&arg_types),
                        ReturnSpec::PromotedString => promoted_string_type(&arg_types),
                        ReturnSpec::FloatFromStringArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(float_return_for_arg),
                        ReturnSpec::PromotedNumeric => promoted_numeric_type(&arg_types),
                        ReturnSpec::ArrayElement(index) => {
                            array_element_return_type(&arg_types, index)
                        }
                        ReturnSpec::ArrayNumeric(index) => {
                            array_numeric_return_type(&arg_types, index)
                        }
                        ReturnSpec::ArrayFromArgs => array_from_return_type(&arg_types),
                        ReturnSpec::IntFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(int_return_for_arg),
                        ReturnSpec::FloatFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .map(float_return_for_arg),
                        ReturnSpec::SeriesFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .and_then(series_return_for_arg),
                        ReturnSpec::PromotedFloat => promoted_float_type(&arg_types),
                        ReturnSpec::Round => round_return_type(&arg_types),
                        ReturnSpec::InputFromArg(index) => arg_types
                            .get(index)
                            .copied()
                            .flatten()
                            .and_then(pine_builtins::input_return_for_arg),
                    }
                } else if let Some((receiver_name, method_name)) = method_call_parts(callee) {
                    self.type_of_method_call_with_params(
                        receiver_name,
                        method_name,
                        callee.span,
                        &arg_types,
                        param_types,
                    )
                } else {
                    let function = self.functions.get(&name)?;
                    let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
                    let mut nested_param_types = HashMap::new();
                    for (arg_type, param_index) in arg_types.into_iter().zip(arg_indices) {
                        let param = &function.params[param_index];
                        nested_param_types.insert(param.clone(), arg_type?);
                    }
                    self.type_of_function_body_with_params(&function.body, &nested_param_types)
                }
            }
            ExprKind::History { expr, .. } => self
                .type_of_expr_with_params(expr, param_types)
                .map(|pine_type| PineType::new(Qualifier::Series, pine_type.kind)),
        }
    }

    fn type_of_method_call_with_params(
        &self,
        receiver_name: &str,
        method_name: &str,
        receiver_span: Span,
        arg_types: &[Option<PineType>],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let receiver_type = param_types
            .get(receiver_name)
            .copied()
            .or_else(|| {
                self.bound_symbol(receiver_name, receiver_span)
                    .map(|symbol| symbol.pine_type)
            })
            .or_else(|| {
                self.scope
                    .resolve(receiver_name)
                    .map(|symbol| symbol.pine_type)
            })?;
        if !is_array_kind(receiver_type.kind) {
            return None;
        }

        let signature =
            pine_builtins::get_phase_1_builtin(array_method_builtin_name(method_name)?)?;
        let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
        method_arg_types.push(Some(receiver_type));
        method_arg_types.extend(arg_types.iter().copied());
        self.return_type(signature, &method_arg_types)
    }

    fn type_of_switch_expr_with_params(
        &self,
        selector: Option<&Expr>,
        arms: &[SwitchArm],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let selector_type = match selector {
            Some(selector) => Some(self.type_of_expr_with_params(selector, param_types)?),
            None => None,
        };
        let mut condition_qualifier = selector_type.map_or(Qualifier::Const, |ty| ty.qualifier);
        let mut result_type = None;

        for arm in arms {
            if let Some(condition) = &arm.condition {
                let condition_type = self.type_of_expr_with_params(condition, param_types)?;
                condition_qualifier =
                    strongest_qualifier(condition_qualifier, condition_type.qualifier);
            }
            let arm_type = self.type_of_expr_with_params(&arm.result, param_types)?;
            result_type = Some(merge_result_types(result_type, arm_type)?);
        }

        result_type.map(|pine_type| {
            PineType::new(
                strongest_qualifier(condition_qualifier, pine_type.qualifier),
                pine_type.kind,
            )
        })
    }

    fn type_of_function_body_with_params(
        &self,
        body: &FunctionBody,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        match body {
            FunctionBody::Expr(expr) => self.type_of_expr_with_params(expr, param_types),
            FunctionBody::Block(statements) => {
                let last = statements.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.type_of_expr_with_params(expr, param_types)
            }
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
            ExprKind::For { body, .. } => {
                let last = body.last()?;
                let StmtKind::Expr(expr) = &last.kind else {
                    return None;
                };
                self.tuple_element_types(expr)
            }
            _ => None,
        }
    }
}

fn unsupported_syntax_reason(feature: &str) -> &'static str {
    match feature {
        "import" => "library imports are not supported in Phase 1",
        "function" => "unsupported user-defined function syntax",
        "for" => "unsupported for loop syntax",
        _ => "syntax is not supported in Phase 1",
    }
}

fn binding_key(name: &str, span: Span) -> BindingKey {
    BindingKey {
        span_start: span.start,
        span_end: span.end,
        name: name.to_owned(),
    }
}

fn const_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::Literal(Literal::Int(value)) => Some(*value),
        ExprKind::Unary {
            op: UnaryOp::Plus,
            expr,
        } => const_int_value(expr),
        ExprKind::Unary {
            op: UnaryOp::Minus,
            expr,
        } => const_int_value(expr).and_then(i64::checked_neg),
        _ => None,
    }
}

fn expr_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::QualifiedName(parts) => Some(parts.join(".")),
        _ => None,
    }
}

fn method_call_parts(expr: &Expr) -> Option<(&str, &str)> {
    match &expr.kind {
        ExprKind::QualifiedName(parts) if parts.len() == 2 => {
            Some((parts[0].as_str(), parts[1].as_str()))
        }
        _ => None,
    }
}

fn receiver_call_arg(receiver_name: &str, span: Span) -> CallArg {
    CallArg {
        name: None,
        span,
        value: Expr {
            kind: ExprKind::Identifier(receiver_name.to_owned()),
            span,
        },
    }
}

fn array_method_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "size" => Some("array.size"),
        "push" => Some("array.push"),
        "get" => Some("array.get"),
        "set" => Some("array.set"),
        "insert" => Some("array.insert"),
        "pop" => Some("array.pop"),
        "remove" => Some("array.remove"),
        "shift" => Some("array.shift"),
        "unshift" => Some("array.unshift"),
        "fill" => Some("array.fill"),
        "first" => Some("array.first"),
        "last" => Some("array.last"),
        "copy" => Some("array.copy"),
        "slice" => Some("array.slice"),
        "concat" => Some("array.concat"),
        "includes" => Some("array.includes"),
        "every" => Some("array.every"),
        "some" => Some("array.some"),
        "indexof" => Some("array.indexof"),
        "lastindexof" => Some("array.lastindexof"),
        "binary_search" => Some("array.binary_search"),
        "binary_search_leftmost" => Some("array.binary_search_leftmost"),
        "binary_search_rightmost" => Some("array.binary_search_rightmost"),
        "abs" => Some("array.abs"),
        "min" => Some("array.min"),
        "max" => Some("array.max"),
        "sum" => Some("array.sum"),
        "avg" => Some("array.avg"),
        "range" => Some("array.range"),
        "median" => Some("array.median"),
        "mode" => Some("array.mode"),
        "percentile_nearest_rank" => Some("array.percentile_nearest_rank"),
        "percentile_linear_interpolation" => Some("array.percentile_linear_interpolation"),
        "percentrank" => Some("array.percentrank"),
        "covariance" => Some("array.covariance"),
        "standardize" => Some("array.standardize"),
        "variance" => Some("array.variance"),
        "stdev" => Some("array.stdev"),
        "sort" => Some("array.sort"),
        "sort_indices" => Some("array.sort_indices"),
        "reverse" => Some("array.reverse"),
        "join" => Some("array.join"),
        "clear" => Some("array.clear"),
        _ => None,
    }
}

fn is_output_or_declaration_builtin(name: &str) -> bool {
    matches!(
        name,
        "indicator"
            | "plot"
            | "hline"
            | "fill"
            | "bgcolor"
            | "barcolor"
            | "plotchar"
            | "plotshape"
            | "plotarrow"
            | "plotbar"
            | "plotcandle"
    ) || name == "input"
        || name.starts_with("input.")
}

fn is_array_mutation_builtin(name: &str) -> bool {
    matches!(
        name,
        "array.push"
            | "array.set"
            | "array.insert"
            | "array.pop"
            | "array.remove"
            | "array.shift"
            | "array.unshift"
            | "array.fill"
            | "array.clear"
            | "array.sort"
            | "array.reverse"
            | "array.concat"
    )
}

fn is_array_mutation_method_call_name(name: &str) -> bool {
    name.rsplit_once('.')
        .and_then(|(_, method_name)| array_method_builtin_name(method_name))
        .is_some_and(is_array_mutation_builtin)
}

fn resolve_udf_arg_indices(params: &[String], args: &[CallArg]) -> Result<Vec<usize>, UdfArgError> {
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

fn prepend_block_statements(mut prefix: Vec<HirStmt>, expr: HirExpr) -> HirExpr {
    match expr.kind {
        HirExprKind::Block { statements, result } => {
            prefix.extend(statements);
            HirExpr {
                kind: HirExprKind::Block {
                    statements: prefix,
                    result,
                },
                pine_type: expr.pine_type,
                series_id: expr.series_id,
            }
        }
        _ => HirExpr {
            pine_type: expr.pine_type,
            series_id: expr.series_id,
            kind: HirExprKind::Block {
                statements: prefix,
                result: Box::new(expr),
            },
        },
    }
}

#[derive(Debug, Default)]
struct InferredHistoryRequirements {
    program: HirHistoryRequirements,
    series: Vec<HirSeriesHistoryRequirement>,
}

#[derive(Debug, Default)]
struct HistoryRequirementCollector {
    program: HirHistoryRequirements,
    series: BTreeMap<SeriesId, HirHistoryRequirements>,
    builtin_series: HashMap<String, SeriesId>,
}

fn infer_history_requirements(
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

fn infer_max_bars_back(statements: &[HirStmt]) -> Option<u32> {
    statements.iter().find_map(max_bars_back_from_stmt)
}

fn max_bars_back_from_stmt(statement: &HirStmt) -> Option<u32> {
    match &statement.kind {
        HirStmtKind::Expr(expr)
        | HirStmtKind::Decl { value: expr, .. }
        | HirStmtKind::Reassign { value: expr, .. }
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

fn max_bars_back_from_expr(expr: &HirExpr) -> Option<u32> {
    match &expr.kind {
        HirExprKind::Call { callee, args, .. } if callee == "indicator" => args
            .iter()
            .enumerate()
            .find(|(index, arg)| {
                arg.name.as_deref() == Some("max_bars_back") || (arg.name.is_none() && *index == 3)
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
        HirExprKind::Tuple(items) => items.iter().find_map(max_bars_back_from_expr),
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

impl HistoryRequirementCollector {
    fn visit_stmt(&mut self, statement: &HirStmt) {
        match &statement.kind {
            HirStmtKind::Expr(expr)
            | HirStmtKind::Decl { value: expr, .. }
            | HirStmtKind::Reassign { value: expr, .. }
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
            HirExprKind::Literal(_) | HirExprKind::Symbol(_) | HirExprKind::Builtin(_) => {}
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
        match callee {
            "ta.tr" | "ta.atr" => self.record_builtin_history("close", 1),
            "ta.change" => self.record_optional_length_history(args),
            "ta.mom" | "ta.roc" => self.record_required_length_history(args),
            "ta.cross" | "ta.crossover" | "ta.crossunder" => self.record_cross_history(args),
            _ => {}
        }
    }

    fn record_optional_length_history(&mut self, args: &[HirCallArg]) {
        let series_id = args.first().and_then(|arg| arg.value.series_id);
        match args.get(1).and_then(|arg| constant_hir_int(&arg.value)) {
            Some(length) if length > 0 => self.record_constant_history(series_id, length as u32),
            Some(_) => {}
            None if args.len() > 1 => self.record_dynamic_history(series_id),
            None => self.record_constant_history(series_id, 1),
        }
    }

    fn record_required_length_history(&mut self, args: &[HirCallArg]) {
        let series_id = args.first().and_then(|arg| arg.value.series_id);
        match args.get(1).and_then(|arg| constant_hir_int(&arg.value)) {
            Some(length) if length > 0 => self.record_constant_history(series_id, length as u32),
            Some(_) => {}
            None => self.record_dynamic_history(series_id),
        }
    }

    fn record_cross_history(&mut self, args: &[HirCallArg]) {
        for arg in args.iter().take(2) {
            self.record_constant_history(arg.value.series_id, 1);
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

fn constant_hir_int(expr: &HirExpr) -> Option<i64> {
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

fn contains_output_or_declaration_call(expr: &Expr) -> bool {
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
                    StmtKind::Unsupported { .. } => false,
                })
        }
        ExprKind::Tuple(items) => items.iter().any(contains_output_or_declaration_call),
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => false,
    }
}

fn statement_contains_output_or_declaration_call(statement: &Stmt) -> bool {
    match &statement.kind {
        StmtKind::Expr(expr) => contains_output_or_declaration_call(expr),
        StmtKind::Decl { value, .. }
        | StmtKind::Reassign { value, .. }
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
        StmtKind::Unsupported { .. } => false,
    }
}

fn has_duplicate_param(params: &[String]) -> bool {
    for (index, param) in params.iter().enumerate() {
        if params[index + 1..].iter().any(|other| other == param) {
            return true;
        }
    }
    false
}

const UNKNOWN: PineType = PineType::new(Qualifier::Series, ValueKind::Na);

const INITIAL_SYMBOLS: &[(&str, PineType)] = &[
    ("open", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("high", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("low", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("close", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("volume", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("time", PineType::new(Qualifier::Series, ValueKind::Int)),
    ("year", PineType::new(Qualifier::Series, ValueKind::Int)),
    ("month", PineType::new(Qualifier::Series, ValueKind::Int)),
    (
        "dayofmonth",
        PineType::new(Qualifier::Series, ValueKind::Int),
    ),
    ("hour", PineType::new(Qualifier::Series, ValueKind::Int)),
    ("minute", PineType::new(Qualifier::Series, ValueKind::Int)),
    ("second", PineType::new(Qualifier::Series, ValueKind::Int)),
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
        Accepts::SeriesOrSimpleNumericOrBool => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Series)
                && (is_numeric(arg_type.kind) || arg_type.kind == ValueKind::Bool)
        }
        Accepts::SimpleInt => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && arg_type.kind == ValueKind::Int
        }
        Accepts::SimpleNumeric => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple) && is_numeric(arg_type.kind)
        }
        Accepts::SimpleBool => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && arg_type.kind == ValueKind::Bool
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
        Accepts::StringCompatible => {
            matches!(arg_type.kind, ValueKind::String | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::StringConvertible => {
            matches!(
                arg_type.kind,
                ValueKind::Int
                    | ValueKind::Float
                    | ValueKind::Bool
                    | ValueKind::String
                    | ValueKind::FloatArray
                    | ValueKind::IntArray
                    | ValueKind::BoolArray
                    | ValueKind::StringArray
                    | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::ValueWhenSource => {
            matches!(
                arg_type.kind,
                ValueKind::Int
                    | ValueKind::Float
                    | ValueKind::Bool
                    | ValueKind::Color
                    | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::IntCompatible => {
            matches!(arg_type.kind, ValueKind::Int | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::BoolCompatible => {
            matches!(arg_type.kind, ValueKind::Bool | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::PlotOrHLine => matches!(arg_type.kind, ValueKind::Plot | ValueKind::HLine),
        Accepts::Array => is_array_kind(arg_type.kind),
        Accepts::NumericArray => is_numeric_array_kind(arg_type.kind),
        Accepts::NumericOrBoolArray => {
            is_numeric_array_kind(arg_type.kind) || arg_type.kind == ValueKind::BoolArray
        }
        Accepts::NumericOrStringArray => {
            is_numeric_array_kind(arg_type.kind) || arg_type.kind == ValueKind::StringArray
        }
        Accepts::InputDefval => {
            arg_type.qualifier == Qualifier::Const
                && matches!(
                    arg_type.kind,
                    ValueKind::Int
                        | ValueKind::Float
                        | ValueKind::Bool
                        | ValueKind::String
                        | ValueKind::Color
                )
        }
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

fn merge_result_types(current: Option<PineType>, next: PineType) -> Option<PineType> {
    match current {
        Some(current) => Some(PineType::new(
            strongest_qualifier(current.qualifier, next.qualifier),
            common_kind(current.kind, next.kind)?,
        )),
        None => Some(next),
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

fn float_return_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Float)
}

fn int_return_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Int)
}

fn series_return_for_arg(arg_type: PineType) -> Option<PineType> {
    match arg_type.kind {
        ValueKind::Int | ValueKind::Float | ValueKind::Bool | ValueKind::Color | ValueKind::Na => {
            Some(PineType::new(Qualifier::Series, arg_type.kind))
        }
        _ => None,
    }
}

fn promoted_float_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        if !is_numeric(arg_type.kind) {
            return None;
        }
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Float))
}

fn promoted_color_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Color))
}

fn promoted_bool_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Bool))
}

fn promoted_int_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    promoted_kind_type(arg_types, ValueKind::Int)
}

fn promoted_string_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    promoted_kind_type(arg_types, ValueKind::String)
}

fn promoted_kind_type(arg_types: &[Option<PineType>], kind: ValueKind) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, kind))
}

fn round_return_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let number_type = arg_types.first().copied().flatten()?;
    if arg_types.len() > 1 {
        Some(PineType::new(number_type.qualifier, ValueKind::Float))
    } else {
        Some(number_type)
    }
}

fn array_element_return_type(arg_types: &[Option<PineType>], index: usize) -> Option<PineType> {
    let array_type = arg_types.get(index).copied().flatten()?;
    let kind = match array_type.kind {
        ValueKind::FloatArray => ValueKind::Float,
        ValueKind::IntArray => ValueKind::Int,
        ValueKind::BoolArray => ValueKind::Bool,
        ValueKind::StringArray => ValueKind::String,
        ValueKind::ColorArray => ValueKind::Color,
        _ => return None,
    };
    Some(PineType::new(Qualifier::Series, kind))
}

fn array_numeric_return_type(arg_types: &[Option<PineType>], index: usize) -> Option<PineType> {
    let array_type = arg_types.get(index).copied().flatten()?;
    let kind = match array_type.kind {
        ValueKind::FloatArray => ValueKind::Float,
        ValueKind::IntArray => ValueKind::Int,
        _ => return None,
    };
    Some(PineType::new(Qualifier::Series, kind))
}

fn array_from_return_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut inferred_kind: Option<ValueKind> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        let next_kind = match arg_type.kind {
            ValueKind::Na => continue,
            ValueKind::Int => ValueKind::IntArray,
            ValueKind::Float => ValueKind::FloatArray,
            ValueKind::Bool => ValueKind::BoolArray,
            ValueKind::String => ValueKind::StringArray,
            ValueKind::Color => ValueKind::ColorArray,
            _ => return None,
        };
        inferred_kind = Some(match (inferred_kind, next_kind) {
            (None, kind) => kind,
            (Some(ValueKind::IntArray), ValueKind::FloatArray)
            | (Some(ValueKind::FloatArray), ValueKind::IntArray)
            | (Some(ValueKind::FloatArray), ValueKind::FloatArray)
            | (Some(ValueKind::IntArray), ValueKind::IntArray) => {
                if matches!(next_kind, ValueKind::FloatArray)
                    || matches!(inferred_kind, Some(ValueKind::FloatArray))
                {
                    ValueKind::FloatArray
                } else {
                    ValueKind::IntArray
                }
            }
            (Some(current), kind) if current == kind => current,
            _ => return None,
        });
    }
    inferred_kind.map(|kind| PineType::new(Qualifier::Simple, kind))
}

fn is_array_kind(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::FloatArray
            | ValueKind::IntArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
    )
}

fn is_numeric_array_kind(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::FloatArray | ValueKind::IntArray)
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
    fn accepts_ta_window_statistics() {
        let analysis = analyze(
            "plot(ta.stdev(close, 3, false) + ta.variance(close, 3, true) + ta.range(close, 3) + ta.dev(close, 3) + ta.vwma(close, 3) + ta.wma(close, 3) + ta.hma(close, 4) + ta.swma(close) + ta.alma(close, 4, 0.85, 6, true) + ta.linreg(close, 3, 0) + ta.correlation(close, high, 3) + ta.covariance(close, high, 3) + ta.median(close, 3) + ta.mode(close, 3) + ta.percentile_nearest_rank(close, 3, 50) + ta.percentile_linear_interpolation(close, 3, 50) + ta.percentrank(close, 3))\n",
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
                .any(|feature| feature.feature == "ta.stdev")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.variance")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.range")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.dev")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.correlation")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.covariance")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.median")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.mode")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.percentile_nearest_rank")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.percentile_linear_interpolation")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.percentrank")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.vwma")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.wma")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.hma")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.swma")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.alma")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.linreg")
        );
    }

    #[test]
    fn accepts_ta_momentum_history_calls() {
        let analysis = analyze("plot(ta.mom(close, 2) + ta.roc(open, 2))\n");

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
                .any(|feature| feature.feature == "ta.mom")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.roc")
        );
        let hir = analysis.hir.expect("HIR");
        assert_eq!(hir.history.max_constant_offset, 2);
        assert!(!hir.history.has_dynamic_offsets);
        assert!(
            hir.series_history
                .iter()
                .any(|requirement| requirement.max_constant_offset == 2)
        );
    }

    #[test]
    fn accepts_ta_trend_window_calls() {
        let analysis = analyze("plot(ta.rising(close, 2) ? 1 : ta.falling(open, 2) ? -1 : 0)\n");

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
                .any(|feature| feature.feature == "ta.rising")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.falling")
        );
    }

    #[test]
    fn accepts_ta_extreme_bar_offsets() {
        let analysis = analyze("plot(ta.highestbars(close, 3) + ta.lowestbars(open, 3))\n");

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
                .any(|feature| feature.feature == "ta.highestbars")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.lowestbars")
        );
    }

    #[test]
    fn accepts_ta_barssince() {
        let analysis = analyze("plot(ta.barssince(close > open))\n");

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
                .any(|feature| feature.feature == "ta.barssince")
        );
    }

    #[test]
    fn accepts_ta_valuewhen() {
        let analysis = analyze(
            "price = ta.valuewhen(close > open, close, 0)\nflag = ta.valuewhen(close > open, close > high, 1)\nshade = ta.valuewhen(close > open, color.red, 0)\nplot(price + (flag ? 1 : 0) + (shade == color.red ? 1 : 0))\n",
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
                .any(|feature| feature.feature == "ta.valuewhen")
        );
    }

    #[test]
    fn accepts_ta_cum() {
        let analysis = analyze("plot(ta.cum(close) + ta.cum(bar_index))\n");

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
                .any(|feature| feature.feature == "ta.cum")
        );
    }

    #[test]
    fn accepts_ta_volume_flow_variables() {
        let analysis = analyze(
            "plot(ta.accdist + ta.iii + ta.nvi + ta.obv + ta.pvi + ta.pvt + ta.vwap + ta.vwap(close) + ta.wad + ta.wvad)\n",
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
                .any(|feature| feature.feature == "ta.accdist")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.iii")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.nvi")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.obv")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.pvi")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.pvt")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.vwap")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.wad")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "ta.wvad")
        );
    }

    #[test]
    fn accepts_input_string_in_conditions() {
        let analysis = analyze(
            "mode = input.string(\"SMA\", \"Mode\")\nplot(mode == \"SMA\" ? close : open)\n",
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
                .any(|feature| feature.feature == "input.string")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_additional_input_variants() {
        let analysis = analyze(
            "threshold = input.price(2.5, \"Price\")\nstart = input.time(0, \"Start\")\nsymbol = input.symbol(\"AAPL\", \"Symbol\")\ntimeframe = input.timeframe(\"D\", \"Timeframe\")\nplot(time >= start and symbol == \"AAPL\" and timeframe == \"D\" ? math.max(close, threshold) : open)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for name in [
            "input.price",
            "input.time",
            "input.symbol",
            "input.timeframe",
        ] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|feature| feature.feature == name),
                "{name} should be reported as supported"
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_generic_input_variants() {
        let analysis = analyze(
            "length = input(2, \"Length\")\nscale = input(1.5, \"Scale\")\nenabled = input(true, \"Enabled\")\nmode = input(\"SMA\", \"Mode\")\nshade = input(color.orange, \"Shade\")\nplot(enabled and mode == \"SMA\" ? ta.sma(close, length) * scale : open, color=color.new(shade, 10))\n",
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
                .any(|feature| feature.feature == "input")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_bgcolor_and_barcolor() {
        let analysis =
            analyze("bgcolor(close > open ? color.green : na)\nbarcolor(color.red)\nplot(close)\n");

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
                .any(|feature| feature.feature == "bgcolor")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "barcolor")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_plotchar() {
        let analysis =
            analyze("plotchar(close > open, char=\"x\", color=color.green)\nplot(close)\n");

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
                .any(|feature| feature.feature == "plotchar")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_plotshape() {
        let analysis = analyze(
            "plotshape(close > open, style=shape.triangleup, location=location.belowbar, color=color.green, text=\"Buy\", textcolor=color.white, size=size.small)\nplot(close)\n",
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
                .any(|feature| feature.feature == "plotshape")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "shape.triangleup")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_plotarrow() {
        let analysis = analyze(
            "plotarrow(close - open, colorup=color.green, colordown=color.red, minheight=5, maxheight=20)\nplot(close)\n",
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
                .any(|feature| feature.feature == "plotarrow")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_plotbar() {
        let analysis = analyze("plotbar(open, high, low, close, color=color.green)\nplot(close)\n");

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
                .any(|feature| feature.feature == "plotbar")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_plotcandle() {
        let analysis = analyze(
            "plotcandle(open, high, low, close, color=color.green, wickcolor=color.white, bordercolor=color.red)\nplot(close)\n",
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
                .any(|feature| feature.feature == "plotcandle")
        );
        assert!(analysis.hir.is_some());
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
    fn rejects_unknown_history_offset() {
        let analysis = analyze("x = close[len]\n");

        assert_eq!(analysis.compatibility.unsupported.len(), 1);
        assert_eq!(
            analysis.compatibility.unsupported[0].feature,
            "dynamic_history_offset"
        );
    }

    #[test]
    fn rejects_non_int_history_offset() {
        let analysis = analyze("x = close[close]\n");

        assert_eq!(analysis.compatibility.unsupported.len(), 1);
        assert_eq!(
            analysis.compatibility.unsupported[0].feature,
            "dynamic_history_offset"
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_input_history_offset() {
        let analysis = analyze("len = input.int(1, \"Length\")\nx = close[len]\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn infers_history_requirements() {
        let analysis =
            analyze("len = input.int(1, \"Length\")\nplot(close[3])\nplot((close + open)[len])\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        assert_eq!(hir.history.max_constant_offset, 3);
        assert!(hir.history.has_dynamic_offsets);
        assert!(
            hir.series_history
                .iter()
                .any(|requirement| requirement.max_constant_offset == 3),
            "{:?}",
            hir.series_history
        );
        assert!(
            hir.series_history
                .iter()
                .any(|requirement| requirement.has_dynamic_offsets),
            "{:?}",
            hir.series_history
        );
    }

    #[test]
    fn infers_implicit_builtin_history_requirements() {
        let analysis = analyze(
            "len = input.int(1, \"Length\")\nplot(ta.tr())\nplot(ta.change(open, 2))\nplot(ta.change(close, len))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        assert_eq!(hir.history.max_constant_offset, 2);
        assert!(hir.history.has_dynamic_offsets);
        assert!(
            hir.series_history
                .iter()
                .any(|requirement| requirement.max_constant_offset == 2),
            "{:?}",
            hir.series_history
        );
        assert!(
            hir.series_history
                .iter()
                .any(|requirement| requirement.has_dynamic_offsets),
            "{:?}",
            hir.series_history
        );
    }

    #[test]
    fn accepts_series_history_offset() {
        let analysis = analyze("x = close[bar_index]\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        let hir = analysis.hir.expect("HIR");
        assert!(hir.history.has_dynamic_offsets);
    }

    #[test]
    fn accepts_indicator_max_bars_back() {
        let analysis = analyze("indicator(\"Demo\", max_bars_back=10)\nplot(close[bar_index])\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        assert_eq!(hir.max_bars_back, Some(10));
    }

    #[test]
    fn rejects_negative_indicator_max_bars_back() {
        let analysis = analyze("indicator(\"Demo\", max_bars_back=-1)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_VALUE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_non_const_indicator_max_bars_back() {
        let analysis = analyze("indicator(\"Demo\", max_bars_back=bar_index)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_negative_history_offset() {
        let analysis = analyze("x = close[-1]\n");

        assert_eq!(analysis.compatibility.unsupported.len(), 1);
        assert_eq!(
            analysis.compatibility.unsupported[0].feature,
            "negative_history_offset"
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
    fn simple_int_params_accept_input_int_expressions() {
        let analysis =
            analyze("length = input.int(2, \"Length\") + 1\nplot(ta.sma(close, length))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
    }

    #[test]
    fn simple_int_params_reject_series_int() {
        let analysis = analyze("plot(ta.sma(close, bar_index))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
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
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
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
                .any(|feature| feature.feature == "color.rgb")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.r")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.g")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.b")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "color.t")
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
    fn accepts_string_helpers() {
        let analysis = analyze(
            r##"indicator("strings")
mode = input.string("sma", "Mode")
upper = str.upper(mode)
lower = str.lower(upper)
length = str.length(upper)
missing = str.length(na)
matched = str.contains(upper, "M") and str.startswith(upper, "S") and str.endswith(upper, "A")
empty_match = str.contains(upper, "") and str.startswith(upper, "") and str.endswith(upper, "")
missing_match = str.contains(na, "S")
mid = str.pos(upper, "M")
missing_pos = str.pos(upper, "Z")
empty_pos = str.pos(upper, "")
na_pos = str.pos(upper, na)
slice = str.substring(upper, mid, mid + 1)
tail = str.substring(upper, mid)
wide = str.substring(upper, 1, 99)
na_begin = str.substring(upper, na, 1)
trimmed = str.trim(" \tSMA\n")
repeated = str.repeat("ab", 2, "-")
empty_repeat = str.repeat("ab", 0)
missing_repeat = str.repeat("ab", na)
replace_first = str.replace("hello", "l", "1")
replace_second = str.replace("hello", "l", "1", 1)
replace_missing = str.replace("hello", "z", "1", 0)
replace_all = str.replace_all("hello", "l", "1")
replace_boundary = str.replace("ab", "", ".", 1)
replace_all_boundaries = str.replace_all("ab", "", ".")
missing_replace = str.replace(na, "x", "y")
number = str.tonumber("1234.50")
signed_number = str.tonumber("-.5")
invalid_number = str.tonumber("$1,234.50")
exponent_number = str.tonumber("1e3")
missing_number = str.tonumber(na)
text_int = str.tostring(42)
text_float = str.tostring(1.25)
text_round0 = str.tostring(1.25, "#")
text_round1 = str.tostring(1.25, "#.#")
text_zeros = str.tostring(1.25, "#.0000")
text_percent = str.tostring(0.1234, format.percent)
text_bool = str.tostring(true)
text_string = str.tostring("ok")
text_na = str.tostring(na)
values = array.new_float(3)
array.set(values, 0, 1.2)
array.set(values, 1, 2.6)
text_array = str.tostring(values, "#")
formatted = str.format("A={0}, B={1}, A2={0}", text_int, text_float)
formatted_missing = str.format("Missing {2}", text_int)
formatted_number = str.format("Rounded {0,number,#.00} Percent {1,number,percent}", 1.2, 0.0345)
formatted_array = str.format("Values {0}", values)
match_prefix = str.match("NASDAQ:AAPL", "^(?:BATS|NASDAQ|NYSE|AMEX):")
match_suffix = str.match("NASDAQ:AAPL", "AAPL$")
match_missing = str.match("NASDAQ:AAPL", "^NYSE:")
missing_match_regex = str.match(na, ".+")
split_words = str.split("A,B,,C", ",")
split_chars = str.split("xy", "")
split_missing = str.split(na, ",")
formatted_time_default = str.format_time(1609459200000)
formatted_time_date = str.format_time(1609459200000, "yyyy-MM-dd")
formatted_time_text = str.format_time(1609459200000, "HH:mm:ss 'on' MMM dd, yyyy", "UTC")
missing_format_time = str.format_time(na)
plot(upper == "SMA" and lower == "sma" ? length : 0)
plot(na(missing) ? 1 : 0)
plot(matched and empty_match ? 1 : 0)
plot(na(missing_match) ? 1 : 0)
plot(mid + empty_pos + na_pos)
plot(na(missing_pos) ? 1 : 0)
plot(slice == "M" and tail == "MA" and wide == "MA" and na_begin == "S" ? 1 : 0)
plot(trimmed == upper and repeated == "ab-ab" and empty_repeat == "" ? 1 : 0)
plot(na(missing_repeat) ? 1 : 0)
plot(replace_first == "he1lo" and replace_second == "hel1o" and replace_missing == "hello" ? 1 : 0)
plot(replace_all == "he11o" and replace_boundary == "a.b" and replace_all_boundaries == ".a.b." ? 1 : 0)
plot(na(missing_replace) ? 1 : 0)
plot(number == 1234.5 and signed_number == -0.5 ? 1 : 0)
plot(na(invalid_number) and na(exponent_number) and na(missing_number) ? 1 : 0)
plot(text_int == "42" and text_float == "1.25" and text_round0 == "1" and text_round1 == "1.3" ? 1 : 0)
plot(text_zeros == "1.2500" and text_percent == "12.34%" ? 1 : 0)
plot(text_bool == "true" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20 Percent 3.45%" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(na(missing_match_regex) ? 1 : 0)
plot(split_words.size() == 4 and split_words.get(0) == "A" and split_words.get(2) == "" and split_words.get(3) == "C" ? 1 : 0)
plot(split_chars.size() == 2 and split_chars.get(0) == "x" and split_chars.get(1) == "y" and na(split_missing) ? 1 : 0)
plot(formatted_time_default == "2021-01-01T00:00:00+0000" and formatted_time_date == "2021-01-01" ? 1 : 0)
plot(formatted_time_text == "00:00:00 on Jan 01, 2021" and na(missing_format_time) ? 1 : 0)
"##,
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in [
            "str.upper",
            "str.lower",
            "str.length",
            "str.contains",
            "str.startswith",
            "str.endswith",
            "str.pos",
            "str.substring",
            "str.trim",
            "str.repeat",
            "str.replace",
            "str.replace_all",
            "str.tonumber",
            "str.tostring",
            "str.format",
            "str.match",
            "str.split",
            "str.format_time",
        ] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} not reported as supported"
            );
        }
    }

    #[test]
    fn accepts_time_helpers() {
        let analysis = analyze(
            r#"indicator("time helpers")
ts = timestamp(2021, 2, 2, 3, 4, 5)
plot(year(ts) + month(ts, "UTC") + dayofmonth(ts) + hour(ts) + minute(ts) + second(ts))
"#,
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in [
            "timestamp",
            "year",
            "month",
            "dayofmonth",
            "hour",
            "minute",
            "second",
        ] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} not reported as supported"
            );
        }
    }

    #[test]
    fn accepts_selected_math_functions() {
        let analysis = analyze(
            r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
avg_value = math.avg(open, close, high, low)
rounded_precision = math.round(close / 3, 2)
z = math.floor(close / 2) + math.ceil(close / 2)
w = math.sqrt(close) + math.log(close) + math.pow(close, 2)
scale = math.log10(close) + math.exp(close)
trig = math.sin(close) + math.cos(close) + math.tan(close)
inverse_trig = math.acos(close - 2) + math.asin(close - 2) + math.atan(close)
angle_helpers = math.sign(close - 2) + math.todegrees(close) + math.toradians(close)
constants = math.pi + math.e + math.phi + math.rphi
sum_value = math.sum(close, 3)
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
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.avg")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.round")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.floor")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.ceil")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.sqrt")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.log")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.log10")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.exp")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.acos")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.asin")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.atan")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.sign")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.todegrees")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.toradians")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.pi")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.e")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.phi")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.rphi")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.pow")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.sin")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.cos")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.tan")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "math.sum")
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
    fn accepts_block_local_declaration_in_if() {
        let analysis = analyze("if close > open\n    x = high - low\n    plot(x)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_block_local_declaration_escape() {
        let analysis = analyze("if close > open\n    x = close\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_SYMBOL")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_if_reassignment_to_declared_symbol() {
        let analysis = analyze("x = close\nif close > open\n    x := high\nplot(x)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_if_tuple_declaration_shadowing_outer_symbols() {
        let analysis =
            analyze("x = close\ny = close\nif close > open\n    [x, y] = [high, low]\nplot(x)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_block_local_tuple_declaration_in_if() {
        let analysis = analyze("if close > open\n    [x, y] = [high, low]\n    plot(x - y)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_block_local_tuple_declaration_escape() {
        let analysis = analyze("if close > open\n    [x, y] = [high, low]\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_SYMBOL")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_if_branch_assignment_type_mismatch() {
        let analysis = analyze("x = close\nif close > open\n    x := true\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_ASSIGN_TYPE")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_condition_switch_expression() {
        let analysis = analyze(
            "x = switch\n    close > open => high\n    close < open => low\n    => close\nplot(x)\n",
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
                .any(|feature| feature.feature == "switch")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_selector_switch_expression() {
        let analysis = analyze(
            "direction = 1\nx = switch direction\n    1 => high\n    -1 => low\n    => close\nplot(x)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_non_bool_condition_switch_arm() {
        let analysis = analyze("x = switch\n    close => high\n    => low\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CONDITION_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_incompatible_switch_arm_results() {
        let analysis = analyze("x = switch\n    close > open => high\n    => true\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_BRANCH_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_statement_block_switch_arm() {
        let analysis = analyze("x = switch\n    close > open =>\n        high\n    => close\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_PARSE_SWITCH_BLOCK"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_expression_body_function() {
        let analysis = analyze("double(x) => x * 2\nplot(double(close))\n");

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
                .any(|feature| feature.feature == "function")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_named_function_arguments() {
        let analysis = analyze("spread(hi, lo) => hi - lo\nplot(spread(lo=low, hi=high))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_duplicate_named_function_argument() {
        let analysis = analyze("spread(hi, lo) => hi - lo\nplot(spread(high, hi=low))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_FUNCTION_ARG_DUPLICATE")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_positional_function_argument_after_named_argument() {
        let analysis = analyze("spread(hi, lo) => hi - lo\nplot(spread(hi=high, low))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_FUNCTION_ARG_ORDER")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_unknown_named_function_argument() {
        let analysis = analyze("double(x) => x * 2\nplot(double(src=close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_FUNCTION_ARG_NAME")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_block_body_function() {
        let analysis = analyze("double(x) =>\n    y = x * 2\n    y\nplot(double(close))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_if_reassignment_inside_block_body_function() {
        let analysis = analyze(
            "select(x, y) =>\n    result = y\n    if x > y\n        result := x\n    result\nplot(select(high, low))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_local_var_declarations_in_blocks_and_functions() {
        let analysis = analyze(
            "counter() =>\n    var value = 0\n    value := value + 1\n    value\nif close > open\n    var seen = 10\n    seen := seen + 1\n    plot(counter() + seen)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_function_local_declaration_shadowing_parameter() {
        let analysis = analyze("bump(x) =>\n    x = x + 1\n    x\nplot(bump(close))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_function_loop_counter_shadowing_parameter() {
        let analysis = analyze(
            "mix(x) =>\n    total = 0\n    for x = 0 to 2\n        total := total + x\n    total + x\nplot(mix(close))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_block_body_function_without_final_expression() {
        let analysis = analyze("double(x) =>\n    y = x * 2\nplot(double(close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_FUNCTION_RETURN")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_recursive_function() {
        let analysis = analyze("loop(x) => loop(x)\nplot(loop(close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_RECURSIVE_FUNCTION")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_wrong_function_arity() {
        let analysis = analyze("double(x) => x * 2\nplot(double(close, open))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_FUNCTION_ARITY")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_output_call_inside_function() {
        let analysis = analyze("draw(x) => plot(x)\ndraw(close)\n");

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_global_reassignment_inside_function() {
        let analysis = analyze("x = close\nbump(v) =>\n    x := v\n    x\nplot(bump(high))\n");

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_stateful_call_as_function_argument() {
        let analysis = analyze("double(x) => x * 2\nplot(double(ta.sma(close, 2)))\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_for_loop_statement() {
        let analysis = analyze("sum = 0\nfor i = 0 to 4 by 2\n    sum := sum + i\nplot(sum)\n");

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
                .any(|feature| feature.feature == "for")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_for_loop_with_series_bounds_and_signed_step() {
        let analysis = analyze(
            "sum = close > 0 ? 0 : 0\nlimit = close > 1 ? 3 : na\nfor i = 0 to limit by -2\n    sum := sum + i\nplot(sum)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_non_int_for_loop_range() {
        let analysis = analyze("for i = 0.5 to 2\n    plot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_LOOP_RANGE_TYPE")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_non_int_for_loop_step() {
        let analysis = analyze("for i = 0 to 2 by 0.5\n    plot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_LOOP_RANGE_TYPE")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_zero_for_loop_step() {
        let analysis = analyze("for i = 0 to 2 by 0\n    plot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_LOOP_STEP")
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_loop_control_inside_for_loop() {
        let analysis = analyze(
            "sum = 0\nfor i = 0 to 5\n    if i == 2\n        continue\n    if i == 4\n        break\n    sum := sum + i\nplot(sum)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_while_loop_statement() {
        let analysis =
            analyze("i = 0\nsum = 0\nwhile i < 5\n    i := i + 1\n    sum := sum + i\nplot(sum)\n");

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
                .any(|feature| feature.feature == "while")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_loop_control_inside_while_loop() {
        let analysis = analyze(
            "i = 0\nwhile i < 5\n    i := i + 1\n    if i == 2\n        continue\n    if i == 4\n        break\nplot(i)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_while_loop_with_na_condition_and_local_var() {
        let analysis = analyze(
            "i = 0\nsum = close > 0 ? 0 : 0\nwhile close > 1 ? i < 3 : na\n    var seen = 0\n    seen := seen + 1\n    sum := sum + seen\n    i := i + 1\nplot(sum)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_while_expression() {
        let analysis = analyze("x = while close > open\n    close\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_PARSE_WHILE_EXPR"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_branch_loop_interactions() {
        let analysis = analyze(
            "repeat(src, limit) =>\n    i = 0\n    total = src * 0.0\n    while i < limit\n        total := total + src\n        i := i + 1\n    total\nsum = close > 0 ? 0.0 : 0.0\nif close > 1\n    for i = 0 to 2\n        value = switch i\n            0 => close\n            1 => high\n            => low\n        sum := sum + value\nelse\n    j = 0\n    while j < 2\n        sum := sum + open\n        j := j + 1\nplot(sum + repeat(close, 2))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_non_bool_while_condition() {
        let analysis = analyze("while close\n    plot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CONDITION_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_float_array_operations() {
        let analysis = analyze(
            "values = array.new_float(2, close)\narray.push(values, high)\narray.set(values, 0, low)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first + last + array.size(values))\n",
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
                .any(|feature| feature.feature == "array.new_float")
        );
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == "array.size")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_float_array_method_calls() {
        let analysis = analyze(
            "values = array.new_float(2, close)\nvalues.push(high)\nvalues.set(0, low)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first + last + values.size())\n",
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
                .any(|feature| feature.feature == "array.push")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_int_array_operations() {
        let analysis = analyze(
            "values = array.new_int(2, bar_index)\narray.push(values, 10)\narray.set(values, 0, 3)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first + last + array.size(values))\n",
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
                .any(|feature| feature.feature == "array.new_int")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_int_array_method_calls() {
        let analysis = analyze(
            "values = array.new_int(2, bar_index)\nvalues.push(10)\nvalues.set(0, 3)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first + last + values.size())\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_bool_array_operations() {
        let analysis = analyze(
            "values = array.new_bool(2, close > open)\narray.push(values, true)\narray.set(values, 0, false)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot((first or last) ? 1 : array.size(values))\n",
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
                .any(|feature| feature.feature == "array.new_bool")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_bool_array_method_calls() {
        let analysis = analyze(
            "values = array.new_bool(2, close > open)\nvalues.push(true)\nvalues.set(0, false)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot((first or last) ? 1 : values.size())\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_string_array_operations() {
        let analysis = analyze(
            "values = array.new_string(2, \"seed\")\narray.push(values, \"tail\")\narray.set(values, 0, \"head\")\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first == \"head\" and last == \"tail\" ? array.size(values) : 0)\n",
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
                .any(|feature| feature.feature == "array.new_string")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_string_array_method_calls() {
        let analysis = analyze(
            "values = array.new_string(2, \"seed\")\nvalues.push(\"tail\")\nvalues.set(0, \"head\")\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first == \"head\" and last == \"tail\" ? values.size() : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_color_array_operations() {
        let analysis = analyze(
            "values = array.new_color(2, color.red)\narray.push(values, color.green)\narray.set(values, 0, color.blue)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first == color.blue and last == color.green ? array.size(values) : 0)\n",
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
                .any(|feature| feature.feature == "array.new_color")
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_color_array_method_calls() {
        let analysis = analyze(
            "values = array.new_color(2, color.red)\nvalues.push(color.green)\nvalues.set(0, color.blue)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first == color.blue and last == color.green ? values.size() : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_helper_operations() {
        let analysis = analyze(
            "values = array.new_int()\narray.unshift(values, 2)\narray.unshift(values, 1)\nfirst = array.first(values)\nlast = array.last(values)\nshifted = array.shift(values)\nplot(first + last + shifted + array.size(values))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in ["array.unshift", "array.first", "array.last", "array.shift"] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_insert_remove_operations() {
        let analysis = analyze(
            "values = array.new_int()\nvalues.push(1)\narray.insert(values, 1, 2)\nvalues.insert(-1, 3)\nremoved = values.remove(-2)\nplot(removed + values.get(-1))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in ["array.insert", "array.remove"] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_fill_operations() {
        let analysis = analyze(
            "values = array.new_string(3, \"a\")\narray.fill(values, \"b\", 1, 3)\nints = array.new_int(2, 1)\nints.fill(2)\nplot(values.get(1) == \"b\" and ints.get(0) == 2 ? 1 : 0)\n",
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
                .any(|supported| supported.feature == "array.fill"),
            "{:?}",
            analysis.compatibility.supported
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_from_operations() {
        let analysis = analyze(
            "ints = array.from(1, 2, 3)\nfloats = array.from(1, close, na)\nflags = array.from(true, false)\nwords = array.from(\"a\", \"b\")\ncolors = array.from(color.red, color.green)\nplot(ints.sum() + floats.avg() + (flags.get(0) ? 1 : 0) + (words.join(\"|\") == \"a|b\" ? 1 : 0) + (colors.get(0) == color.red ? 1 : 0))\n",
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
                .any(|supported| supported.feature == "array.from"),
            "{:?}",
            analysis.compatibility.supported
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_helper_method_calls() {
        let analysis = analyze(
            "values = array.new_string()\nvalues.unshift(\"tail\")\nvalues.unshift(\"head\")\nfirst = values.first()\nlast = values.last()\nshifted = values.shift()\nplot(first == \"head\" and last == \"tail\" and shifted == \"head\" ? values.size() : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_copy_operations() {
        let analysis = analyze(
            "source = array.new_int()\nalias = source\ncopy = array.copy(source)\nmethod_copy = source.copy()\narray.push(alias, 1)\narray.push(copy, 2)\nmethod_copy.push(3)\nplot(array.size(source) + array.size(copy) + method_copy.size())\n",
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
                .any(|feature| feature.feature == "array.copy"),
            "{:?}",
            analysis.compatibility.supported
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_search_operations() {
        let analysis = analyze(
            "values = array.new_string()\narray.push(values, \"a\")\narray.push(values, \"b\")\narray.push(values, \"a\")\nhas_a = array.includes(values, \"a\")\nfirst = array.indexof(values, \"a\")\nlast = array.lastindexof(values, \"a\")\nmissing = values.indexof(\"z\")\nnums = array.from(1, 2, 2, 4)\nfound = array.binary_search(nums, 2)\nleft = nums.binary_search_leftmost(3)\nright = nums.binary_search_rightmost(3)\nflags = array.from(true, false)\nplot(has_a and values.includes(\"b\") and flags.some() and not flags.every() ? first + last + missing + found + left + right : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in [
            "array.includes",
            "array.every",
            "array.some",
            "array.indexof",
            "array.lastindexof",
            "array.binary_search",
            "array.binary_search_leftmost",
            "array.binary_search_rightmost",
        ] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_numeric_array_statistics() {
        let analysis = analyze(
            "ints = array.new_int()\narray.push(ints, 1)\narray.push(ints, 3)\narray.push(ints, 3)\nabs_ints = ints.abs()\nstandard_ints = ints.standardize()\nfloats = array.new_float()\nfloats.push(close)\nfloats.push(high)\nplot(array.min(ints) + array.max(ints) + array.sum(ints) + ints.range() + ints.median() + array.mode(ints) + ints.percentile_nearest_rank(50) + array.percentile_linear_interpolation(ints, 75) + array.percentrank(ints, 1) + ints.covariance(standard_ints) + ints.variance(false) + array.avg(floats) + floats.max() + array.range(floats) + array.stdev(floats) + array.sum(abs_ints) + standard_ints.get(0))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in [
            "array.min",
            "array.max",
            "array.sum",
            "array.avg",
            "array.range",
            "array.median",
            "array.mode",
            "array.percentile_nearest_rank",
            "array.percentile_linear_interpolation",
            "array.percentrank",
            "array.covariance",
            "array.standardize",
            "array.variance",
            "array.stdev",
            "array.abs",
        ] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_ordering_operations() {
        let analysis = analyze(
            "values = array.new_int()\narray.push(values, 3)\narray.push(values, 1)\nindices = values.sort_indices(order.descending)\narray.sort(values, order.descending)\nvalues.reverse()\nwords = array.from(\"b\", \"a\")\nword_indices = words.sort_indices(order.ascending)\nwords.sort(order.ascending)\nplot(values.get(0) + values.get(1) + indices.get(0) + word_indices.get(0))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in ["array.sort", "array.sort_indices", "array.reverse"] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_join_operations() {
        let analysis = analyze(
            "values = array.new_string()\nvalues.push(\"a\")\nvalues.push(\"b\")\ntext = array.join(values, \"|\")\nints = array.new_int()\nints.push(1)\nints.push(2)\nplot(text == \"a|b\" and ints.join() == \"1,2\" ? 1 : 0)\n",
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
                .any(|supported| supported.feature == "array.join"),
            "{:?}",
            analysis.compatibility.supported
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_array_slice_concat_operations() {
        let analysis = analyze(
            "values = array.new_int()\nvalues.push(1)\nvalues.push(2)\nvalues.push(3)\npart = array.slice(values, 1, 3)\nmore = array.new_int()\nmore.push(4)\nreturned = values.concat(more)\nplot(part.size() + array.size(returned) + values.get(3))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        for feature in ["array.slice", "array.concat"] {
            assert!(
                analysis
                    .compatibility
                    .supported
                    .iter()
                    .any(|supported| supported.feature == feature),
                "{feature} missing from supported features: {:?}",
                analysis.compatibility.supported
            );
        }
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_float_value_for_int_array_mutation() {
        let analysis =
            analyze("values = array.new_int()\narray.push(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_bool_array_mutation() {
        let analysis =
            analyze("values = array.new_bool()\narray.push(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_bool_array_unshift() {
        let analysis =
            analyze("values = array.new_bool()\narray.unshift(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_bool_array_insert() {
        let analysis =
            analyze("values = array.new_bool()\narray.insert(values, 0, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_bool_array_fill() {
        let analysis =
            analyze("values = array.new_bool(2)\narray.fill(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_mixed_array_from_element_kinds() {
        let analysis = analyze("values = array.from(1, \"two\")\nplot(array.size(values))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_untyped_na_array_from() {
        let analysis = analyze("values = array.from(na, na)\nplot(array.size(values))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_bool_array_search() {
        let analysis = analyze("values = array.new_bool()\nplot(array.indexof(values, close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_bool_array_binary_search() {
        let analysis = analyze("values = array.new_bool()\nplot(array.binary_search(values, 1))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_float_value_for_int_array_binary_search() {
        let analysis = analyze("values = array.new_int()\nplot(values.binary_search(close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_string_array_truth_helpers() {
        let analysis = analyze("values = array.new_string()\nplot(array.every(values))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_bool_array_statistics() {
        let analysis = analyze("values = array.new_bool()\nplot(array.stdev(values))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_bool_array_sort() {
        let analysis =
            analyze("values = array.new_bool()\nvalues.push(true)\narray.sort(values)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_array_sort_order() {
        let analysis = analyze("values = array.new_int()\narray.sort(values, close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_bool_array_sort_indices() {
        let analysis =
            analyze("values = array.new_bool()\nvalues.push(true)\nvalues.sort_indices()\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_array_sort_indices_order() {
        let analysis = analyze("values = array.new_int()\narray.sort_indices(values, close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_separator_for_array_join() {
        let analysis = analyze("values = array.new_string()\nplot(array.join(values, close))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_mismatched_array_concat_kind() {
        let analysis = analyze(
            "ints = array.new_int()\nfloats = array.new_float()\nplot(array.size(array.concat(ints, floats)))\n",
        );

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_series_array_slice_index() {
        let analysis =
            analyze("values = array.new_string()\nplot(array.size(values.slice(0, bar_index)))\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_string_array_mutation() {
        let analysis =
            analyze("values = array.new_string()\narray.push(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_numeric_value_for_color_array_mutation() {
        let analysis =
            analyze("values = array.new_color()\narray.push(values, close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_array_method_call_on_namespace_like_variable_name() {
        let analysis =
            analyze("strategy = array.new_float()\nstrategy.push(close)\nplot(strategy.size())\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_unknown_float_array_method() {
        let analysis =
            analyze("values = array.new_float()\nvalues.unsupported(close)\nplot(close)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_METHOD"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_array_helper_mutation_inside_udf() {
        let analysis = analyze(
            "add(values, value) =>\n    values.unshift(value)\n    values.shift()\nvalues = array.new_float()\nplot(add(values, close))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_array_ordering_mutation_inside_udf() {
        let analysis = analyze(
            "order(values) =>\n    values.sort()\n    values.reverse()\n    values.size()\nvalues = array.new_float()\nplot(order(values))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_unsupported_array_function() {
        let analysis = analyze("values = array.new_line(0)\nplot(close)\n");

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "array.new_line"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_readonly_float_array_udf_parameter() {
        let analysis = analyze(
            "first(values) => array.get(values, 0)\nvalues = array.new_float(1, close)\nplot(first(values) + array.size(values))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_readonly_int_array_udf_parameter() {
        let analysis = analyze(
            "first(values) => array.get(values, 0)\nvalues = array.new_int(1, bar_index)\nplot(first(values) + array.size(values))\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_readonly_bool_array_udf_parameter() {
        let analysis = analyze(
            "first(values) => array.get(values, 0)\nvalues = array.new_bool(1, true)\nplot(first(values) ? array.size(values) : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_readonly_string_array_udf_parameter() {
        let analysis = analyze(
            "first(values) => array.get(values, 0)\nvalues = array.new_string(1, \"seed\")\nplot(first(values) == \"seed\" ? array.size(values) : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_readonly_color_array_udf_parameter() {
        let analysis = analyze(
            "first(values) => array.get(values, 0)\nvalues = array.new_color(1, color.red)\nplot(first(values) == color.red ? array.size(values) : 0)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_readonly_float_array_method_udf_parameter() {
        let analysis = analyze(
            "first(values) => values.get(0)\nvalues = array.new_float(1, close)\nplot(first(values) + values.size())\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_array_mutation_inside_udf() {
        let analysis = analyze(
            "add(values, value) =>\n    array.push(values, value)\n    array.size(values)\nvalues = array.new_float()\nplot(add(values, close))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_array_method_mutation_inside_udf() {
        let analysis = analyze(
            "add(values, value) =>\n    values.push(value)\n    values.size()\nvalues = array.new_float()\nplot(add(values, close))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_array_mutation_as_udf_argument() {
        let analysis = analyze(
            "identity(value) => value\nvalues = array.new_float(1, close)\nplot(identity(array.pop(values)))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_array_method_mutation_as_udf_argument() {
        let analysis = analyze(
            "identity(value) => value\nvalues = array.new_float(1, close)\nplot(identity(values.pop()))\n",
        );

        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|feature| feature.feature == "function_side_effect"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_loop_control_outside_for_loop() {
        let analysis = analyze("break\ncontinue\n");

        let loop_control_errors = analysis
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "E_LOOP_CONTROL")
            .count();
        assert_eq!(loop_control_errors, 2, "{:?}", analysis.diagnostics);
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_for_counter_escape() {
        let analysis = analyze("for i = 0 to 2\n    plot(close)\nplot(i)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_SYMBOL"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn rejects_for_body_local_declaration_escape() {
        let analysis = analyze("for i = 0 to 2\n    x = i\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_UNKNOWN_SYMBOL"),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn accepts_nested_for_counter_shadowing() {
        let analysis = analyze(
            "sum = 0\nfor i = 0 to 1\n    for i = 0 to 1\n        sum := sum + i\nplot(sum)\n",
        );

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_for_expression_result() {
        let analysis = analyze("x = for i = 0 to 2\n    i * 2\nplot(x)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn accepts_tuple_for_expression_result() {
        let analysis = analyze("[x, y] = for i = 0 to 2\n    [i, i * 2]\nplot(x + y)\n");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
    }

    #[test]
    fn rejects_for_expression_without_final_expression() {
        let analysis = analyze("x = for i = 0 to 2\n    y = i\nplot(x)\n");

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E_LOOP_RETURN"),
            "{:?}",
            analysis.diagnostics
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
