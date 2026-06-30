use std::collections::HashMap;

use pine_syntax::{Diagnostic, ExprKind, Span};

use super::model::{
    ExportInfo, ImportRef, ModuleInfo, ModuleMethodInfo, ModuleUserTypeFieldInfo,
    ModuleUserTypeIdentity, module_user_type_fields_match,
};
use super::visit_statement_exprs;
use crate::source_graph::SourceId;

pub(super) fn validate_alias_access(
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
            if let Some(export) = module.exports.get(symbol) {
                match export {
                    ExportInfo::Function { .. } | ExportInfo::Const { .. } => return,
                    ExportInfo::UserType {
                        identity, fields, ..
                    } => {
                        if imported_udt_constructor_is_supported(parts, fields) {
                            return;
                        }
                        if let Some(user_type) = module.user_types.get(symbol) {
                            debug_assert!(module_user_type_fields_match(fields, &user_type.fields));
                        }
                        diagnostics.push(imported_udt_unsupported_diagnostic(
                            parts[0].as_str(),
                            module.id,
                            identity,
                            fields,
                            expr.span,
                        ));
                        return;
                    }
                }
            }
            if let Some(user_type) = module.user_types.get(symbol) {
                if imported_udt_constructor_is_supported(parts, &user_type.fields) {
                    return;
                }
                debug_assert!(user_type.span.start <= user_type.span.end);
                debug_assert!(user_type.fields.iter().all(|field| {
                    !field.name.is_empty()
                        && !field.type_name.is_empty()
                        && field.span.start <= field.span.end
                }));
                diagnostics.push(imported_udt_unsupported_diagnostic(
                    parts[0].as_str(),
                    module.id,
                    &user_type.identity,
                    &user_type.fields,
                    expr.span,
                ));
            } else if let Some(method) = module.methods.get(symbol) {
                diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNSUPPORTED_METHOD",
                    imported_method_unsupported_message(parts[0].as_str(), symbol, method),
                    expr.span,
                ));
            } else if module.private_symbols.contains(symbol)
                || module.functions.contains_key(symbol)
            {
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

fn imported_method_unsupported_message(
    alias: &str,
    method_name: &str,
    method: &ModuleMethodInfo,
) -> String {
    if let Some(identity) = &method.receiver_identity {
        return format!(
            "imported method `{alias}.{method_name}` for receiver `{alias}.{}` is not supported; imported method dispatch requires imported UDT identity",
            identity.name
        );
    }
    if let Some(receiver_type_name) = &method.receiver_type_name {
        return format!(
            "imported method `{alias}.{method_name}` for receiver `{receiver_type_name}` is not supported; imported method dispatch requires imported UDT identity"
        );
    }
    format!(
        "imported method `{alias}.{method_name}` is not supported; imported method dispatch requires imported UDT identity"
    )
}

fn imported_udt_unsupported_diagnostic(
    alias: &str,
    module_id: SourceId,
    identity: &ModuleUserTypeIdentity,
    fields: &[ModuleUserTypeFieldInfo],
    span: Span,
) -> Diagnostic {
    debug_assert_eq!(identity.source_id, module_id);
    let field_note = if fields.iter().all(|field| field.pine_type.is_some()) {
        "scalar-field metadata is available, but constructor/value identity is not implemented"
    } else {
        "non-scalar or deferred field metadata remains unsupported"
    };
    Diagnostic::error(
        "E_IMPORT_UNSUPPORTED_UDT",
        format!(
            "imported UDT `{}.{}` is not supported; {field_note}",
            alias, identity.name
        ),
        span,
    )
}

fn imported_udt_constructor_is_supported(
    parts: &[String],
    fields: &[ModuleUserTypeFieldInfo],
) -> bool {
    parts.len() == 3 && parts[2] == "new" && fields.iter().all(|field| field.pine_type.is_some())
}
