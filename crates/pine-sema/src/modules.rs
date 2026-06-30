use std::collections::{HashMap, HashSet};

use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{
    BinaryOp, Diagnostic, ExportItem, Expr, ExprKind, FunctionBody, Program, Span, Stmt, StmtKind,
    SwitchArmResult, UnaryOp, parse_source,
};

use crate::analyzer::context::FunctionInfo;
use crate::analyzer::functions::{
    contains_output_or_declaration_call, statement_contains_output_or_declaration_call,
};
use crate::source_graph::AnalysisInput;

mod alias_access;
mod imports;
mod model;
#[path = "modules_rewrite.rs"]
mod modules_rewrite;

use imports::{
    detect_import_cycles, imports_in_program, validate_library_imports, validate_root_imports,
};
use model::{
    ExportInfo, ModuleInfo, ModuleMethodInfo, ModuleUserTypeFieldInfo, ModuleUserTypeIdentity,
    ModuleUserTypeInfo, imported_user_type_scalar_field_type, module_user_type_fields_match,
};
pub(crate) use model::{
    ImportedUserTypeFieldInfo, ImportedUserTypeIdentity, ImportedUserTypeInfo, ModuleValidation,
};
use modules_rewrite::{RewriteContext, rewrite_expr, rewrite_function_body, rewrite_program};

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
        user_types: HashMap::new(),
        methods: HashMap::new(),
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
            user_types: HashMap::new(),
            methods: HashMap::new(),
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
        imported_user_types: import_plan.imported_user_types,
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
                ExportItem::UserType { decl, span } => {
                    let identity = ModuleUserTypeIdentity {
                        source_id: module.id,
                        name: decl.name.clone(),
                    };
                    let fields = decl
                        .fields
                        .iter()
                        .map(|field| ModuleUserTypeFieldInfo {
                            name: field.name.clone(),
                            type_name: field.type_name.clone(),
                            pine_type: imported_user_type_scalar_field_type(&field.type_name),
                            span: field.span,
                        })
                        .collect::<Vec<_>>();
                    register_export(
                        module,
                        &decl.name,
                        ExportInfo::UserType {
                            identity: identity.clone(),
                            fields: fields.clone(),
                            span: *span,
                        },
                        diagnostics,
                    );
                    module.user_types.insert(
                        decl.name.clone(),
                        ModuleUserTypeInfo {
                            identity,
                            fields,
                            span: *span,
                        },
                    );
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
            StmtKind::UserType(user_type) => {
                module.private_symbols.insert(user_type.name.clone());
            }
            StmtKind::Method(_) => {}
            _ => {}
        }
    }
    for statement in &statements {
        let StmtKind::Method(method) = &statement.kind else {
            continue;
        };
        let receiver_type_name = method
            .params
            .first()
            .map(|receiver| receiver.type_name.clone());
        let receiver_identity = receiver_type_name
            .as_deref()
            .and_then(|type_name| module.user_types.get(type_name))
            .map(|user_type| user_type.identity.clone());
        module.methods.insert(
            method.name.clone(),
            ModuleMethodInfo {
                receiver_type_name,
                receiver_identity,
            },
        );
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

#[derive(Default)]
struct ImportPlan {
    root_rewrites: RewriteContext,
    imported_functions: HashMap<String, FunctionInfo>,
    imported_user_types: HashMap<String, ImportedUserTypeInfo>,
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
                ExportInfo::UserType {
                    identity,
                    fields,
                    span,
                } => {
                    debug_assert!(module.user_types.get(name).is_some_and(|user_type| {
                        module_user_type_fields_match(fields, &user_type.fields)
                    }));
                    plan.imported_user_types.insert(
                        format!("{alias}.{name}"),
                        ImportedUserTypeInfo {
                            identity: ImportedUserTypeIdentity {
                                source_id: identity.source_id,
                                name: identity.name.clone(),
                            },
                            fields: fields
                                .iter()
                                .map(|field| ImportedUserTypeFieldInfo {
                                    name: field.name.clone(),
                                    type_name: field.type_name.clone(),
                                    pine_type: field.pine_type,
                                    span: field.span,
                                })
                                .collect(),
                            span: *span,
                        },
                    );
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
        | ExprKind::While { .. }
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
        StmtKind::ForIn { iterable, body, .. } => {
            visit_expr(iterable, visitor);
            for statement in body {
                visit_statement_exprs(statement, visitor);
            }
        }
        StmtKind::Export(export) => match &export.item {
            ExportItem::Const { value, .. } => visit_expr(value, visitor),
            ExportItem::Function { body, .. } => visit_function_body(body, visitor),
            ExportItem::UserType { .. } => {}
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
        ExprKind::While { condition, body } => {
            visit_expr(condition, visitor);
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
                match &arm.result {
                    SwitchArmResult::Expr(result) => visit_expr(result, visitor),
                    SwitchArmResult::Block(statements) => {
                        for statement in statements {
                            visit_statement_exprs(statement, visitor);
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_graph::SourceId;
    use pine_syntax::SourceFile;

    fn parsed_program(text: &str) -> Program {
        parse_source(&SourceFile::new("library.pine", text)).program
    }

    #[test]
    fn exported_user_type_records_identity_and_fields() {
        let mut module = ModuleInfo {
            id: SourceId::library(7),
            key: Some("user/identity/1".to_owned()),
            program: parsed_program(
                r#"
library("identity")
export type Point
    float x
"#,
            ),
            exports: HashMap::new(),
            private_symbols: HashSet::new(),
            user_types: HashMap::new(),
            methods: HashMap::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        let mut diagnostics = Vec::new();

        collect_library_declarations(&mut module, &mut diagnostics);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let ExportInfo::UserType {
            identity, fields, ..
        } = module.exports.get("Point").expect("exported UDT")
        else {
            panic!("Point should be a UDT export");
        };
        assert_eq!(identity.source_id, SourceId::library(7));
        assert_eq!(identity.name, "Point");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "x");
        assert_eq!(fields[0].type_name, "float");
        assert_eq!(
            fields[0].pine_type,
            Some(PineType::new(Qualifier::Series, ValueKind::Float))
        );

        let user_type = module.user_types.get("Point").expect("UDT table entry");
        assert_eq!(user_type.identity, *identity);
        assert_eq!(user_type.fields.len(), 1);
        assert_eq!(user_type.fields[0].name, fields[0].name);
        assert_eq!(user_type.fields[0].type_name, fields[0].type_name);
        assert_eq!(user_type.fields[0].pine_type, fields[0].pine_type);
        assert_eq!(user_type.fields[0].span, fields[0].span);
    }

    #[test]
    fn import_plan_records_alias_qualified_user_type_metadata() {
        let root = ModuleInfo {
            id: SourceId::root(),
            key: None,
            program: parse_source(&SourceFile::new(
                "root.pine",
                r#"import user/identity/1 as lib
"#,
            ))
            .program,
            exports: HashMap::new(),
            private_symbols: HashSet::new(),
            user_types: HashMap::new(),
            methods: HashMap::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        let mut library = ModuleInfo {
            id: SourceId::library(0),
            key: Some("user/identity/1".to_owned()),
            program: parsed_program(
                r#"
library("identity")
export type Point
    float x
"#,
            ),
            exports: HashMap::new(),
            private_symbols: HashSet::new(),
            user_types: HashMap::new(),
            methods: HashMap::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        let mut diagnostics = Vec::new();
        collect_library_declarations(&mut library, &mut diagnostics);
        let modules = vec![root, library];
        let library_index = HashMap::from([("user/identity/1".to_owned(), 1)]);

        let plan = build_import_plan(&modules, &library_index, &mut diagnostics);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let point = plan
            .imported_user_types
            .get("lib.Point")
            .expect("alias-qualified imported UDT");
        assert_eq!(point.identity.source_id, SourceId::library(0));
        assert_eq!(point.identity.name, "Point");
        assert_eq!(point.fields.len(), 1);
        assert_eq!(point.fields[0].name, "x");
        assert_eq!(point.fields[0].type_name, "float");
        assert_eq!(
            point.fields[0].pine_type,
            Some(PineType::new(Qualifier::Series, ValueKind::Float))
        );
        assert_eq!(
            point.span,
            modules[1]
                .user_types
                .get("Point")
                .expect("library UDT metadata")
                .span
        );
    }

    #[test]
    fn library_method_records_receiver_identity_metadata() {
        let mut module = ModuleInfo {
            id: SourceId::library(3),
            key: Some("user/methods/1".to_owned()),
            program: parsed_program(
                r#"
library("methods")
export type Point
    float x

method shift(Point p, float delta) => p.x + delta
"#,
            ),
            exports: HashMap::new(),
            private_symbols: HashSet::new(),
            user_types: HashMap::new(),
            methods: HashMap::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        let mut diagnostics = Vec::new();

        collect_library_declarations(&mut module, &mut diagnostics);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let method = module.methods.get("shift").expect("library method");
        assert_eq!(method.receiver_type_name.as_deref(), Some("Point"));
        assert_eq!(
            method.receiver_identity,
            Some(ModuleUserTypeIdentity {
                source_id: SourceId::library(3),
                name: "Point".to_owned(),
            })
        );
    }
}
