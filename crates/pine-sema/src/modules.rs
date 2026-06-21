use std::collections::{HashMap, HashSet};

use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{
    BinaryOp, Diagnostic, ExportItem, Expr, ExprKind, FunctionBody, ImportDecl, Program, Span,
    Stmt, StmtKind, UnaryOp, parse_source,
};

use crate::analyzer::context::FunctionInfo;
use crate::analyzer::functions::{
    contains_output_or_declaration_call, statement_contains_output_or_declaration_call,
};
use crate::source_graph::{AnalysisInput, SourceId};

#[path = "modules_rewrite.rs"]
mod modules_rewrite;

use modules_rewrite::{RewriteContext, rewrite_expr, rewrite_function_body, rewrite_program};

#[derive(Debug)]
pub(crate) struct ModuleValidation {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) root_program: Program,
    pub(crate) imported_functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug)]
struct ModuleInfo {
    id: SourceId,
    key: Option<String>,
    program: Program,
    exports: HashMap<String, ExportInfo>,
    private_symbols: HashSet<String>,
    functions: HashMap<String, FunctionInfo>,
    constants: HashMap<String, Expr>,
}

#[derive(Debug, Clone)]
enum ExportInfo {
    Function { span: Span },
    Const { value: Expr, span: Span },
}

impl ExportInfo {
    fn span(&self) -> Span {
        match self {
            ExportInfo::Function { span } | ExportInfo::Const { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
struct ImportRef {
    key: String,
    alias: Option<(String, Span)>,
    span: Span,
}

pub(crate) fn validate_modules(input: &AnalysisInput) -> ModuleValidation {
    let graph = input.source_graph();
    let mut diagnostics = Vec::new();
    let mut modules = Vec::with_capacity(graph.libraries().len() + 1);
    let root_parse = parse_source(graph.root().source());
    diagnostics.extend(root_parse.diagnostics.clone());
    let root_program = root_parse.program;
    modules.push(ModuleInfo {
        id: graph.root().id(),
        key: None,
        program: root_program.clone(),
        exports: HashMap::new(),
        private_symbols: HashSet::new(),
        functions: HashMap::new(),
        constants: HashMap::new(),
    });

    let mut library_index = HashMap::new();
    for library in graph.libraries() {
        let parsed = parse_source(library.source());
        diagnostics.extend(parsed.diagnostics.clone());
        let mut module = ModuleInfo {
            id: library.id(),
            key: library.import_key().map(str::to_owned),
            program: parsed.program,
            exports: HashMap::new(),
            private_symbols: HashSet::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        collect_library_declarations(&mut module, &mut diagnostics);
        if let Some(key) = &module.key {
            library_index.insert(key.clone(), modules.len());
        }
        modules.push(module);
    }

    validate_root_imports(&modules, &library_index, &mut diagnostics);
    validate_library_imports(&modules, &library_index, &mut diagnostics);
    detect_import_cycles(&modules, &library_index, &mut diagnostics);

    let import_plan = build_import_plan(&modules, &library_index, &mut diagnostics);
    let root_program = rewrite_program(&root_program, &import_plan.root_rewrites);

    ModuleValidation {
        diagnostics,
        root_program,
        imported_functions: import_plan.imported_functions,
    }
}

fn collect_library_declarations(module: &mut ModuleInfo, diagnostics: &mut Vec<Diagnostic>) {
    let mut library_declarations = 0;
    // Temporarily move the statements out so we can iterate by reference
    // without cloning the entire AST; the loop only mutates other fields of
    // `module`, never `module.program.statements`.
    let statements = std::mem::take(&mut module.program.statements);
    for statement in &statements {
        match &statement.kind {
            StmtKind::Library(_) => library_declarations += 1,
            StmtKind::Export(export) => match &export.item {
                ExportItem::Function {
                    name,
                    params,
                    body,
                    span,
                } => {
                    register_export(
                        module,
                        name,
                        ExportInfo::Function { span: *span },
                        diagnostics,
                    );
                    if function_body_has_side_effect(body) {
                        diagnostics.push(Diagnostic::error(
                            "E_IMPORT_FUNCTION_SIDE_EFFECT",
                            format!("exported function `{name}` contains unsupported side effects"),
                            *span,
                        ));
                    }
                    module.functions.insert(
                        name.clone(),
                        FunctionInfo {
                            params: params.clone(),
                            body: body.clone(),
                            span: *span,
                        },
                    );
                }
                ExportItem::Const { name, value, span } => {
                    register_export(
                        module,
                        name,
                        ExportInfo::Const {
                            value: value.clone(),
                            span: *span,
                        },
                        diagnostics,
                    );
                    if !is_const_import_expr(value) {
                        diagnostics.push(Diagnostic::error(
                            "E_IMPORT_CONST_VALUE",
                            format!("exported constant `{name}` must be a const expression"),
                            value.span,
                        ));
                    }
                    module.constants.insert(name.clone(), value.clone());
                }
                ExportItem::Unknown { .. } => {}
            },
            StmtKind::Function { name, params, body } => {
                module.private_symbols.insert(name.clone());
                module.functions.insert(
                    name.clone(),
                    FunctionInfo {
                        params: params.clone(),
                        body: body.clone(),
                        span: statement.span,
                    },
                );
            }
            StmtKind::Decl { name, value, .. } => {
                module.private_symbols.insert(name.clone());
                if is_const_import_expr(value) {
                    module.constants.insert(name.clone(), value.clone());
                }
            }
            _ => {}
        }
    }
    // Restore the statements that were moved out above.
    module.program.statements = statements;

    if module.key.is_some() && library_declarations != 1 {
        diagnostics.push(Diagnostic::error(
            "E_IMPORT_INVALID_LIBRARY",
            "library source must contain exactly one library declaration",
            first_statement_span(&module.program).unwrap_or_else(|| Span::new(0, 0)),
        ));
    }
}

fn register_export(
    module: &mut ModuleInfo,
    name: &str,
    export: ExportInfo,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(existing) = module.exports.insert(name.to_owned(), export.clone()) {
        diagnostics.push(Diagnostic::error(
            "E_IMPORT_DUPLICATE_EXPORT",
            format!("duplicate export `{name}`"),
            existing.span().merge(export.span()),
        ));
    }
}

fn validate_root_imports(
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let root = &modules[0];
    let imports = imports_in_program(&root.program);
    let mut aliases = HashMap::new();
    for import in &imports {
        if !library_index.contains_key(&import.key) {
            diagnostics.push(Diagnostic::error(
                "E_IMPORT_MISSING_LIBRARY",
                format!("missing library source for import `{}`", import.key),
                import.span,
            ));
        }
        let Some((alias, alias_span)) = &import.alias else {
            diagnostics.push(Diagnostic::error(
                "E_IMPORT_ALIAS_REQUIRED",
                format!(
                    "import `{}` requires an alias in the supported subset",
                    import.key
                ),
                import.span,
            ));
            continue;
        };
        if let Some(previous) = aliases.insert(alias.clone(), *alias_span) {
            diagnostics.push(Diagnostic::error(
                "E_IMPORT_DUPLICATE_ALIAS",
                format!("duplicate import alias `{alias}`"),
                previous.merge(*alias_span),
            ));
        }
    }

    validate_alias_access(root, &imports, modules, library_index, diagnostics);
}

fn validate_library_imports(
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for module in modules.iter().skip(1) {
        for import in imports_in_program(&module.program) {
            if !library_index.contains_key(&import.key) {
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_MISSING_LIBRARY",
                    format!("missing library source for import `{}`", import.key),
                    import.span,
                ));
            }
        }
    }
}

fn validate_alias_access(
    root: &ModuleInfo,
    imports: &[ImportRef],
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let aliases: HashMap<_, _> = imports
        .iter()
        .filter_map(|import| {
            let (alias, _) = import.alias.as_ref()?;
            Some((alias.as_str(), import.key.as_str()))
        })
        .collect();
    if aliases.is_empty() {
        return;
    }

    for statement in &root.program.statements {
        visit_statement_exprs(statement, &mut |expr| {
            let ExprKind::QualifiedName(parts) = &expr.kind else {
                return;
            };
            if parts.len() < 2 {
                return;
            }
            let Some(key) = aliases.get(parts[0].as_str()) else {
                return;
            };
            let Some(module_index) = library_index.get(*key) else {
                return;
            };
            let module = &modules[*module_index];
            let symbol = &parts[1];
            if module.exports.contains_key(symbol) {
                return;
            }
            if module.private_symbols.contains(symbol) || module.functions.contains_key(symbol) {
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_PRIVATE_SYMBOL",
                    format!("`{}` is not exported by `{}`", parts.join("."), key),
                    expr.span,
                ));
            } else {
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNKNOWN_EXPORT",
                    format!("unknown export `{symbol}` in `{key}`"),
                    expr.span,
                ));
            }
        });
    }
}

fn detect_import_cycles(
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for module in modules.iter().skip(1) {
        visit_module(
            module,
            modules,
            library_index,
            &mut visiting,
            &mut visited,
            diagnostics,
        );
    }
}

fn visit_module(
    module: &ModuleInfo,
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    visiting: &mut HashSet<SourceId>,
    visited: &mut HashSet<SourceId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if visited.contains(&module.id) {
        return;
    }
    if !visiting.insert(module.id) {
        return;
    }

    for import in imports_in_program(&module.program) {
        let Some(next_index) = library_index.get(&import.key) else {
            continue;
        };
        let next = &modules[*next_index];
        if visiting.contains(&next.id) {
            diagnostics.push(Diagnostic::error(
                "E_IMPORT_CYCLE",
                format!("import cycle includes `{}`", import.key),
                import.span,
            ));
            continue;
        }
        visit_module(next, modules, library_index, visiting, visited, diagnostics);
    }

    visiting.remove(&module.id);
    visited.insert(module.id);
}

#[derive(Default)]
struct ImportPlan {
    root_rewrites: RewriteContext,
    imported_functions: HashMap<String, FunctionInfo>,
}

fn build_import_plan(
    modules: &[ModuleInfo],
    library_index: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ImportPlan {
    let mut plan = ImportPlan::default();
    for import in imports_in_program(&modules[0].program) {
        let Some((alias, _)) = import.alias else {
            continue;
        };
        let Some(module_index) = library_index.get(&import.key) else {
            continue;
        };
        let module = &modules[*module_index];
        let module_context = rewrite_context_for_module(&alias, module);

        for (name, export) in &module.exports {
            match export {
                ExportInfo::Const { value, .. } => {
                    plan.root_rewrites.constants.insert(
                        format!("{alias}.{name}"),
                        rewrite_expr(value, &module_context),
                    );
                }
                ExportInfo::Function { .. } => {
                    plan.root_rewrites
                        .function_targets
                        .insert(format!("{alias}.{name}"), format!("{alias}.{name}"));
                }
            }
        }

        for (name, function) in &module.functions {
            let key = module_function_key(&alias, module, name);
            let body = rewrite_function_body(&function.body, &function.params, &module_context);
            if name_is_exported_function(module, name) || module.private_symbols.contains(name) {
                plan.imported_functions.insert(
                    key,
                    FunctionInfo {
                        params: function.params.clone(),
                        body,
                        span: function.span,
                    },
                );
            } else {
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNKNOWN_EXPORT",
                    format!("unknown imported function `{name}`"),
                    function.span,
                ));
            }
        }
    }
    plan
}

fn rewrite_context_for_module(alias: &str, module: &ModuleInfo) -> RewriteContext {
    let mut context = RewriteContext::default();
    for (name, value) in &module.constants {
        context.constants.insert(name.clone(), value.clone());
        context
            .constants
            .insert(format!("{alias}.{name}"), value.clone());
    }
    for name in module.functions.keys() {
        context
            .function_targets
            .insert(name.clone(), module_function_key(alias, module, name));
        context.function_targets.insert(
            format!("{alias}.{name}"),
            module_function_key(alias, module, name),
        );
    }
    context
}

fn module_function_key(alias: &str, module: &ModuleInfo, name: &str) -> String {
    if name_is_exported_function(module, name) {
        format!("{alias}.{name}")
    } else {
        format!("__import_{alias}_{name}")
    }
}

fn name_is_exported_function(module: &ModuleInfo, name: &str) -> bool {
    matches!(module.exports.get(name), Some(ExportInfo::Function { .. }))
}

fn is_const_import_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) => true,
        ExprKind::QualifiedName(parts) => const_qualified_type(&parts.join(".")).is_some(),
        ExprKind::Unary { op, expr } => {
            matches!(op, UnaryOp::Plus | UnaryOp::Minus | UnaryOp::Not)
                && is_const_import_expr(expr)
        }
        ExprKind::Binary { op, left, right } => {
            matches!(
                op,
                BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Gt
                    | BinaryOp::Gte
                    | BinaryOp::Lt
                    | BinaryOp::Lte
                    | BinaryOp::And
                    | BinaryOp::Or
            ) && is_const_import_expr(left)
                && is_const_import_expr(right)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            is_const_import_expr(condition)
                && is_const_import_expr(then_expr)
                && is_const_import_expr(else_expr)
        }
        ExprKind::Call { .. }
        | ExprKind::If { .. }
        | ExprKind::For { .. }
        | ExprKind::Switch { .. }
        | ExprKind::Tuple(_)
        | ExprKind::History { .. }
        | ExprKind::Identifier(_) => false,
    }
}

fn const_qualified_type(name: &str) -> Option<PineType> {
    if pine_builtins::named_color(name).is_some() {
        return Some(PineType::new(Qualifier::Const, ValueKind::Color));
    }
    if pine_builtins::named_float_constant(name).is_some() {
        return Some(PineType::new(Qualifier::Const, ValueKind::Float));
    }
    if pine_builtins::named_int_constant(name).is_some() {
        return Some(PineType::new(Qualifier::Const, ValueKind::Int));
    }
    if pine_builtins::named_string_constant(name).is_some() {
        return Some(PineType::new(Qualifier::Const, ValueKind::String));
    }
    None
}

fn function_body_has_side_effect(body: &FunctionBody) -> bool {
    match body {
        FunctionBody::Expr(expr) => contains_output_or_declaration_call(expr),
        FunctionBody::Block(statements) => statements
            .iter()
            .any(statement_contains_output_or_declaration_call),
    }
}

fn imports_in_program(program: &Program) -> Vec<ImportRef> {
    program
        .statements
        .iter()
        .filter_map(|statement| {
            let StmtKind::Import(import) = &statement.kind else {
                return None;
            };
            Some(import_ref(import, statement.span))
        })
        .collect()
}

fn import_ref(import: &ImportDecl, span: Span) -> ImportRef {
    ImportRef {
        key: import.key.clone(),
        alias: import
            .alias
            .as_ref()
            .map(|alias| (alias.name.clone(), alias.span)),
        span,
    }
}

fn first_statement_span(program: &Program) -> Option<Span> {
    program.statements.first().map(|statement| statement.span)
}

fn visit_statement_exprs(statement: &Stmt, visitor: &mut impl FnMut(&Expr)) {
    match &statement.kind {
        StmtKind::Expr(expr)
        | StmtKind::Decl { value: expr, .. }
        | StmtKind::Reassign { value: expr, .. }
        | StmtKind::FieldReassign { value: expr, .. }
        | StmtKind::TupleDecl { value: expr, .. } => visit_expr(expr, visitor),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visit_expr(condition, visitor);
            for statement in then_branch.iter().chain(else_branch) {
                visit_statement_exprs(statement, visitor);
            }
        }
        StmtKind::For {
            from,
            to,
            step,
            body,
            ..
        } => {
            visit_expr(from, visitor);
            visit_expr(to, visitor);
            if let Some(step) = step {
                visit_expr(step, visitor);
            }
            for statement in body {
                visit_statement_exprs(statement, visitor);
            }
        }
        StmtKind::While { condition, body } => {
            visit_expr(condition, visitor);
            for statement in body {
                visit_statement_exprs(statement, visitor);
            }
        }
        StmtKind::Export(export) => match &export.item {
            ExportItem::Const { value, .. } => visit_expr(value, visitor),
            ExportItem::Function { body, .. } => visit_function_body(body, visitor),
            ExportItem::Unknown { .. } => {}
        },
        StmtKind::Method(method) => visit_function_body(&method.body, visitor),
        StmtKind::Import(_)
        | StmtKind::Library(_)
        | StmtKind::UserType(_)
        | StmtKind::Break
        | StmtKind::Continue
        | StmtKind::Function { .. }
        | StmtKind::Unsupported { .. } => {}
    }
}

fn visit_function_body(body: &FunctionBody, visitor: &mut impl FnMut(&Expr)) {
    match body {
        FunctionBody::Expr(expr) => visit_expr(expr, visitor),
        FunctionBody::Block(statements) => {
            for statement in statements {
                visit_statement_exprs(statement, visitor);
            }
        }
    }
}

fn visit_expr(expr: &Expr, visitor: &mut impl FnMut(&Expr)) {
    visitor(expr);
    match &expr.kind {
        ExprKind::Unary { expr, .. } | ExprKind::History { expr, .. } => visit_expr(expr, visitor),
        ExprKind::Binary { left, right, .. } => {
            visit_expr(left, visitor);
            visit_expr(right, visitor);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            visit_expr(condition, visitor);
            visit_expr(then_expr, visitor);
            visit_expr(else_expr, visitor);
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visit_expr(condition, visitor);
            for statement in then_branch.iter().chain(else_branch) {
                visit_statement_exprs(statement, visitor);
            }
        }
        ExprKind::For {
            from,
            to,
            step,
            body,
            ..
        } => {
            visit_expr(from, visitor);
            visit_expr(to, visitor);
            if let Some(step) = step {
                visit_expr(step, visitor);
            }
            for statement in body {
                visit_statement_exprs(statement, visitor);
            }
        }
        ExprKind::Switch { selector, arms } => {
            if let Some(selector) = selector {
                visit_expr(selector, visitor);
            }
            for arm in arms {
                if let Some(condition) = &arm.condition {
                    visit_expr(condition, visitor);
                }
                visit_expr(&arm.result, visitor);
            }
        }
        ExprKind::Tuple(items) => {
            for item in items {
                visit_expr(item, visitor);
            }
        }
        ExprKind::Call { callee, args } => {
            visit_expr(callee, visitor);
            for arg in args {
                visit_expr(&arg.value, visitor);
            }
        }
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => {}
    }
}
