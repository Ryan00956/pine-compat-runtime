use std::collections::{HashMap, HashSet};

use pine_syntax::{
    Diagnostic, ExportItem, Expr, ExprKind, FunctionBody, Program, Span, Stmt, StmtKind,
    parse_source,
};

use crate::source_graph::{AnalysisInput, SourceId};

#[derive(Debug)]
pub(crate) struct ModuleValidation {
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
struct ModuleInfo {
    id: SourceId,
    key: Option<String>,
    program: Program,
    exports: HashMap<String, Span>,
    private_symbols: HashSet<String>,
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
    modules.push(ModuleInfo {
        id: graph.root().id(),
        key: None,
        program: root_parse.program,
        exports: HashMap::new(),
        private_symbols: HashSet::new(),
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

    ModuleValidation { diagnostics }
}

fn collect_library_declarations(module: &mut ModuleInfo, diagnostics: &mut Vec<Diagnostic>) {
    let mut library_declarations = 0;
    for statement in &module.program.statements {
        match &statement.kind {
            StmtKind::Library(_) => library_declarations += 1,
            StmtKind::Export(export) => {
                if let Some((name, span)) = export_name(&export.item)
                    && let Some(existing) = module.exports.insert(name.clone(), span)
                {
                    diagnostics.push(Diagnostic::error(
                        "E_IMPORT_DUPLICATE_EXPORT",
                        format!("duplicate export `{name}`"),
                        existing.merge(span),
                    ));
                }
            }
            StmtKind::Function { name, .. } | StmtKind::Decl { name, .. } => {
                module.private_symbols.insert(name.clone());
            }
            _ => {}
        }
    }

    if module.key.is_some() && library_declarations != 1 {
        diagnostics.push(Diagnostic::error(
            "E_IMPORT_INVALID_LIBRARY",
            "library source must contain exactly one library declaration",
            first_statement_span(&module.program).unwrap_or_else(|| Span::new(0, 0)),
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
        if let Some((alias, alias_span)) = &import.alias
            && let Some(previous) = aliases.insert(alias.clone(), *alias_span)
        {
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
            if parts.len() != 2 {
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
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNSUPPORTED_EXECUTION",
                    format!(
                        "imported symbol `{}` is not executable yet",
                        parts.join(".")
                    ),
                    expr.span,
                ));
            } else if module.private_symbols.contains(symbol) {
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

fn imports_in_program(program: &Program) -> Vec<ImportRef> {
    program
        .statements
        .iter()
        .filter_map(|statement| {
            let StmtKind::Import(import) = &statement.kind else {
                return None;
            };
            Some(ImportRef {
                key: import.key.clone(),
                alias: import
                    .alias
                    .as_ref()
                    .map(|alias| (alias.name.clone(), alias.span)),
                span: statement.span,
            })
        })
        .collect()
}

fn export_name(item: &ExportItem) -> Option<(String, Span)> {
    match item {
        ExportItem::Function { name, span, .. } | ExportItem::Const { name, span, .. } => {
            Some((name.clone(), *span))
        }
        ExportItem::Unknown { .. } => None,
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
