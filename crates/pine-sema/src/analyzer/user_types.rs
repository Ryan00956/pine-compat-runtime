use std::collections::HashMap;

use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{
    CallArg, Diagnostic, Expr, ExprKind, FunctionBody, Program, Span, Stmt, StmtKind, SwitchArm,
};

use crate::analyzer::calls::expr_name;
use crate::analyzer::context::{Analyzer, FunctionInfo};
use crate::analyzer::functions::resolve_udf_arg_indices;
use crate::compatibility::FeatureUse;
use crate::resolver::SymbolInfo;
use crate::types::{UNKNOWN, can_assign, strongest_qualifier};

#[derive(Debug, Clone)]
pub(crate) struct UserTypeInfo {
    pub(crate) name: String,
    pub(crate) fields: Vec<UserTypeFieldInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct UserTypeFieldInfo {
    pub(crate) name: String,
    pub(crate) pine_type: PineType,
}

#[derive(Debug, Clone)]
pub(crate) struct UdtConstructor {
    pub(crate) field_args: Vec<Expr>,
    pub(crate) pine_type: PineType,
}

#[derive(Debug, Clone)]
pub(crate) struct UdtFieldAccess {
    pub(crate) receiver: String,
    pub(crate) index: usize,
}

pub(crate) struct UdtFieldMutation {
    pub(crate) pine_type: PineType,
}

pub(crate) fn span_key(span: Span) -> (usize, usize) {
    (span.start, span.end)
}

impl Analyzer {
    pub(crate) fn register_user_types(&mut self, program: &Program) {
        for statement in &program.statements {
            let StmtKind::UserType(decl) = &statement.kind else {
                continue;
            };

            if self.user_types.contains_key(&decl.name) {
                self.diagnostics.push(Diagnostic::error(
                    "E_UDT_DUPLICATE",
                    format!("duplicate user-defined type `{}`", decl.name),
                    decl.name_span,
                ));
                continue;
            }

            let mut fields = Vec::new();
            let mut seen = HashMap::new();
            for field in &decl.fields {
                if let Some(existing) = seen.insert(field.name.clone(), field.span) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UDT_FIELD_DUPLICATE",
                        format!("duplicate field `{}` in `{}`", field.name, decl.name),
                        existing.merge(field.span),
                    ));
                    continue;
                }

                let Some(pine_type) = self.user_type_field_type(&field.type_name, field.span)
                else {
                    continue;
                };
                fields.push(UserTypeFieldInfo {
                    name: field.name.clone(),
                    pine_type,
                });
            }

            self.user_types.insert(
                decl.name.clone(),
                UserTypeInfo {
                    name: decl.name.clone(),
                    fields,
                },
            );
        }
    }

    pub(crate) fn user_type_constructor(
        &mut self,
        callee_name: &str,
        args: &[CallArg],
        span: Span,
    ) -> Option<UdtConstructor> {
        let type_name = callee_name.strip_suffix(".new")?;
        let user_type = self.user_types.get(type_name).cloned()?;

        let resolved = self.resolve_constructor_args(&user_type, args, span)?;
        let mut qualifier = Qualifier::Const;
        let mut field_args = Vec::with_capacity(resolved.len());
        for (field, arg) in user_type.fields.iter().zip(resolved) {
            let Some(arg) = arg else {
                self.diagnostics.push(Diagnostic::error(
                    "E_UDT_CONSTRUCTOR_ARG",
                    format!(
                        "missing field `{}` for `{}` constructor",
                        field.name, type_name
                    ),
                    span,
                ));
                return None;
            };
            let arg_type = self.analyze_expr(&arg.value).unwrap_or(UNKNOWN);
            if !can_assign(field.pine_type, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_UDT_CONSTRUCTOR_ARG",
                    format!(
                        "cannot assign {:?} {:?} to field `{}` of type {:?}",
                        arg_type.qualifier, arg_type.kind, field.name, field.pine_type.kind
                    ),
                    arg.span,
                ));
            }
            qualifier = strongest_qualifier(qualifier, arg_type.qualifier);
            field_args.push(arg.value.clone());
        }

        let pine_type = PineType::new(qualifier, ValueKind::UserType);
        self.expr_user_types
            .insert(span_key(span), user_type.name.clone());
        self.compatibility.supported.push(FeatureUse {
            feature: "user-defined types".to_owned(),
            span,
        });
        Some(UdtConstructor {
            field_args,
            pine_type,
        })
    }

    pub(crate) fn resolve_user_type_field_access(
        &mut self,
        parts: &[String],
        span: Span,
    ) -> Option<PineType> {
        if parts.len() != 2 {
            return None;
        }
        let receiver = &parts[0];
        let field_name = &parts[1];
        let symbol = self.scope.resolve(receiver)?;
        let type_name = self.symbol_user_types.get(&symbol.id)?.clone();
        let user_type = self.user_types.get(&type_name)?;
        let Some((_, field)) = user_type
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == *field_name)
        else {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_UNKNOWN_FIELD",
                format!("unknown field `{field_name}` on `{type_name}`"),
                span,
            ));
            // The receiver is a known user-defined type, so this is a field
            // access (not a namespace lookup); short-circuit with `UNKNOWN`
            // instead of fabricating an invalid field index.
            return Some(UNKNOWN);
        };
        let pine_type = PineType::new(symbol.pine_type.qualifier, field.pine_type.kind);
        self.bind_symbol(receiver, span, symbol);
        Some(pine_type)
    }

    pub(crate) fn type_of_user_type_field_access(&self, parts: &[String]) -> Option<PineType> {
        if parts.len() != 2 {
            return None;
        }
        let symbol = self.scope.resolve(&parts[0])?;
        let type_name = self.symbol_user_types.get(&symbol.id)?;
        let user_type = self.user_types.get(type_name)?;
        let field = user_type
            .fields
            .iter()
            .find(|field| field.name == parts[1])?;
        Some(PineType::new(
            symbol.pine_type.qualifier,
            field.pine_type.kind,
        ))
    }

    pub(crate) fn resolve_user_type_field_mutation(
        &mut self,
        receiver: &str,
        field_name: &str,
        span: Span,
    ) -> Option<UdtFieldMutation> {
        let Some(symbol) = self.scope.resolve(receiver) else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_SYMBOL",
                format!("cannot reassign unknown symbol `{receiver}`"),
                span,
            ));
            return None;
        };
        let Some(type_name) = self.symbol_user_types.get(&symbol.id).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_FIELD_MUTATION",
                format!("cannot mutate field `{field_name}` on non-UDT `{receiver}`"),
                span,
            ));
            return None;
        };
        let user_type = self.user_types.get(&type_name)?;
        let Some(field_kind) = user_type
            .fields
            .iter()
            .find(|field| field.name == field_name)
            .map(|field| field.pine_type.kind)
        else {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_UNKNOWN_FIELD",
                format!("unknown field `{field_name}` on `{type_name}`"),
                span,
            ));
            return None;
        };
        self.bind_symbol(receiver, span, symbol);
        Some(UdtFieldMutation {
            pine_type: PineType::new(symbol.pine_type.qualifier, field_kind),
        })
    }

    pub(crate) fn type_of_bound_user_type_field_access(
        &self,
        parts: &[String],
        span: Span,
    ) -> Option<PineType> {
        if parts.len() != 2 {
            return None;
        }
        let symbol = self
            .bound_symbol(&parts[0], span)
            .or_else(|| self.scope.resolve(&parts[0]))?;
        let type_name = self.symbol_user_types.get(&symbol.id)?;
        let user_type = self.user_types.get(type_name)?;
        let field = user_type
            .fields
            .iter()
            .find(|field| field.name == parts[1])?;
        Some(PineType::new(
            symbol.pine_type.qualifier,
            field.pine_type.kind,
        ))
    }

    pub(crate) fn type_of_user_type_constructor_with_params(
        &self,
        callee_name: &str,
        args: &[CallArg],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let type_name = callee_name.strip_suffix(".new")?;
        let user_type = self.user_types.get(type_name)?;
        if args.len() != user_type.fields.len() {
            return None;
        }
        let mut qualifier = Qualifier::Const;
        for arg in args {
            let arg_type = self.type_of_expr_with_params(&arg.value, param_types)?;
            qualifier = strongest_qualifier(qualifier, arg_type.qualifier);
        }
        Some(PineType::new(qualifier, ValueKind::UserType))
    }

    pub(crate) fn user_type_constructor_for_lowering(
        &self,
        callee_name: &str,
        args: &[CallArg],
        param_types: &HashMap<String, PineType>,
    ) -> Option<UdtConstructor> {
        let type_name = callee_name.strip_suffix(".new")?;
        let user_type = self.user_types.get(type_name)?;
        let mut field_args = vec![None; user_type.fields.len()];
        let mut next_positional = 0;
        for arg in args {
            let index = match &arg.name {
                Some(name) => user_type
                    .fields
                    .iter()
                    .position(|field| field.name == *name)?,
                None => {
                    let index = next_positional;
                    next_positional += 1;
                    index
                }
            };
            field_args[index] = Some(arg.value.clone());
        }
        Some(UdtConstructor {
            field_args: field_args.into_iter().collect::<Option<_>>()?,
            pine_type: self.type_of_user_type_constructor_with_params(
                callee_name,
                args,
                param_types,
            )?,
        })
    }

    pub(crate) fn user_type_field_access_for_lowering(
        &self,
        parts: &[String],
        span: Span,
    ) -> Option<UdtFieldAccess> {
        if parts.len() != 2 {
            return None;
        }
        let symbol = self
            .bound_symbol(&parts[0], span)
            .or_else(|| self.scope.resolve(&parts[0]))?;
        let type_name = self.symbol_user_types.get(&symbol.id)?;
        let user_type = self.user_types.get(type_name)?;
        let index = user_type
            .fields
            .iter()
            .position(|field| field.name == parts[1])?;
        Some(UdtFieldAccess {
            receiver: parts[0].clone(),
            index,
        })
    }

    pub(crate) fn expr_user_type_name(&self, expr: &Expr) -> Option<String> {
        self.expr_user_types.get(&span_key(expr.span)).cloned()
    }

    pub(crate) fn user_type_name_of_expr(&self, expr: &Expr) -> Option<String> {
        if let Some(type_name) = self.expr_user_type_name(expr) {
            return Some(type_name);
        }
        match &expr.kind {
            ExprKind::Identifier(name) => self
                .scope
                .resolve(name)
                .and_then(|symbol| self.symbol_user_types.get(&symbol.id).cloned()),
            ExprKind::QualifiedName(parts) if parts.len() == 1 => self
                .scope
                .resolve(&parts[0])
                .and_then(|symbol| self.symbol_user_types.get(&symbol.id).cloned()),
            ExprKind::Call { callee, args } => {
                self.user_type_name_of_udf_passthrough(expr_name(callee)?.as_str(), args)
            }
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => self.user_type_name_of_ternary_branches(then_expr, else_expr),
            ExprKind::Switch { arms, .. } => self.user_type_name_of_switch_arms(arms),
            ExprKind::For { body, .. } => self.user_type_name_of_for_body(body),
            _ => None,
        }
    }

    pub(crate) fn mark_ternary_user_type(
        &mut self,
        span: Span,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> bool {
        let Some(type_name) = self.user_type_name_of_ternary_branches(then_expr, else_expr) else {
            return false;
        };
        self.mark_expr_user_type(span, type_name);
        true
    }

    fn user_type_name_of_ternary_branches(
        &self,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Option<String> {
        match (
            self.user_type_name_of_expr(then_expr),
            self.user_type_name_of_expr(else_expr),
        ) {
            (Some(then_name), Some(else_name)) if then_name == else_name => Some(then_name),
            (Some(then_name), None) if is_na_expr(else_expr) => Some(then_name),
            (None, Some(else_name)) if is_na_expr(then_expr) => Some(else_name),
            _ => None,
        }
    }

    pub(crate) fn mark_switch_user_type(&mut self, span: Span, arms: &[SwitchArm]) -> bool {
        let Some(type_name) = self.user_type_name_of_switch_arms(arms) else {
            return false;
        };
        self.mark_expr_user_type(span, type_name);
        true
    }

    fn user_type_name_of_switch_arms(&self, arms: &[SwitchArm]) -> Option<String> {
        let mut resolved_type_name = None;
        for arm in arms {
            match self.user_type_name_of_expr(&arm.result) {
                Some(type_name) => match &resolved_type_name {
                    Some(resolved) if resolved != &type_name => return None,
                    Some(_) => {}
                    None => resolved_type_name = Some(type_name),
                },
                None if is_na_expr(&arm.result) => {}
                None => return None,
            }
        }
        resolved_type_name
    }

    pub(crate) fn user_type_name_of_if_branches(
        &self,
        then_branch: &[Stmt],
        else_branch: &[Stmt],
    ) -> Option<String> {
        let then_expr = single_expr_branch(then_branch)?;
        let else_expr = single_expr_branch(else_branch)?;
        match (
            self.user_type_name_of_expr(then_expr),
            self.user_type_name_of_expr(else_expr),
        ) {
            (Some(then_name), Some(else_name)) if then_name == else_name => Some(then_name),
            (Some(then_name), None) if is_na_expr(else_expr) => Some(then_name),
            (None, Some(else_name)) if is_na_expr(then_expr) => Some(else_name),
            _ => None,
        }
    }

    fn user_type_name_of_for_body(&self, body: &[Stmt]) -> Option<String> {
        let last = body.last()?;
        let StmtKind::Expr(expr) = &last.kind else {
            return None;
        };
        self.user_type_name_of_expr(expr)
    }

    pub(crate) fn user_type_name_of_udf_passthrough(
        &self,
        name: &str,
        args: &[CallArg],
    ) -> Option<String> {
        let function = self.functions.get(name)?;
        let param_index =
            returned_udf_param_index(&function.body, &function.params, &self.functions, 0)?;
        let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
        let arg_index = arg_indices
            .iter()
            .position(|mapped_param_index| *mapped_param_index == param_index)?;
        self.user_type_name_of_expr(&args[arg_index].value)
    }

    pub(crate) fn user_type_name_of_function_body(&self, body: &FunctionBody) -> Option<String> {
        match body {
            FunctionBody::Expr(expr) => self.user_type_name_of_expr(expr),
            FunctionBody::Block(statements) => {
                let last = statements.last()?;
                match &last.kind {
                    StmtKind::Expr(expr) => self.user_type_name_of_expr(expr),
                    StmtKind::If {
                        then_branch,
                        else_branch,
                        ..
                    } => self.user_type_name_of_if_branches(then_branch, else_branch),
                    _ => None,
                }
            }
        }
    }

    pub(crate) fn mark_expr_user_type(&mut self, span: Span, type_name: String) {
        self.expr_user_types.insert(span_key(span), type_name);
    }

    pub(crate) fn mark_symbol_user_type(&mut self, symbol: SymbolInfo, type_name: String) {
        self.symbol_user_types.insert(symbol.id, type_name);
    }

    fn resolve_constructor_args(
        &mut self,
        user_type: &UserTypeInfo,
        args: &[CallArg],
        span: Span,
    ) -> Option<Vec<Option<CallArg>>> {
        if args.len() > user_type.fields.len() {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_CONSTRUCTOR_ARG",
                format!(
                    "`{}.new` expects {} field argument(s), got {}",
                    user_type.name,
                    user_type.fields.len(),
                    args.len()
                ),
                span,
            ));
            return None;
        }

        let mut resolved = vec![None; user_type.fields.len()];
        let mut positional_open = true;
        let mut next_positional = 0;
        for arg in args {
            if let Some(name) = &arg.name {
                positional_open = false;
                let Some(index) = user_type
                    .fields
                    .iter()
                    .position(|field| field.name == *name)
                else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UDT_CONSTRUCTOR_ARG",
                        format!(
                            "unknown field `{name}` for `{}` constructor",
                            user_type.name
                        ),
                        arg.span,
                    ));
                    return None;
                };
                if resolved[index].is_some() {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UDT_CONSTRUCTOR_ARG",
                        format!(
                            "duplicate field `{name}` for `{}` constructor",
                            user_type.name
                        ),
                        arg.span,
                    ));
                    return None;
                }
                resolved[index] = Some(arg.clone());
            } else {
                if !positional_open {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UDT_CONSTRUCTOR_ARG",
                        "positional field argument cannot follow named field argument",
                        arg.span,
                    ));
                    return None;
                }
                resolved[next_positional] = Some(arg.clone());
                next_positional += 1;
            }
        }
        Some(resolved)
    }

    fn user_type_field_type(&mut self, name: &str, span: Span) -> Option<PineType> {
        let kind = match name {
            "int" => ValueKind::Int,
            "float" => ValueKind::Float,
            "bool" => ValueKind::Bool,
            "string" => ValueKind::String,
            "color" => ValueKind::Color,
            other if self.user_types.contains_key(other) => {
                self.unsupported(
                    "user-defined type fields",
                    "user-defined type fields cannot contain nested or recursive UDT values in the current UDT subset",
                    span,
                );
                return None;
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_UDT_FIELD_TYPE",
                    format!("unsupported or unknown user-defined type field type `{name}`"),
                    span,
                ));
                return None;
            }
        };
        Some(PineType::new(Qualifier::Series, kind))
    }
}

fn returned_udf_param_index(
    body: &FunctionBody,
    params: &[String],
    functions: &HashMap<String, FunctionInfo>,
    depth: usize,
) -> Option<usize> {
    if depth > params.len() + functions.len() {
        return None;
    }
    match body {
        FunctionBody::Expr(expr) => returned_expr_param_index(expr, params, functions, depth),
        FunctionBody::Block(statements) => {
            let (last, prefix) = statements.split_last()?;
            let StmtKind::Expr(expr) = &last.kind else {
                return None;
            };
            let mut aliases = HashMap::new();
            for statement in prefix {
                if let StmtKind::Decl { name, value, .. } = &statement.kind
                    && let Some(source_name) = identifier_name(value)
                {
                    aliases.insert(name.clone(), source_name.clone());
                }
            }
            returned_expr_param_index_with_aliases(expr, params, functions, &aliases, depth)
        }
    }
}

fn returned_expr_param_index(
    expr: &Expr,
    params: &[String],
    functions: &HashMap<String, FunctionInfo>,
    depth: usize,
) -> Option<usize> {
    returned_expr_param_index_with_aliases(expr, params, functions, &HashMap::new(), depth)
}

fn returned_expr_param_index_with_aliases(
    expr: &Expr,
    params: &[String],
    functions: &HashMap<String, FunctionInfo>,
    aliases: &HashMap<String, String>,
    depth: usize,
) -> Option<usize> {
    if let Some(returned_name) = identifier_name(expr) {
        return aliased_param_index(returned_name, params, aliases);
    }
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let callee_name = expr_name(callee)?;
    let function = functions.get(&callee_name)?;
    let returned_param_index =
        returned_udf_param_index(&function.body, &function.params, functions, depth + 1)?;
    let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
    let arg_index = arg_indices
        .iter()
        .position(|mapped_param_index| *mapped_param_index == returned_param_index)?;
    returned_expr_param_index_with_aliases(
        &args[arg_index].value,
        params,
        functions,
        aliases,
        depth,
    )
}

fn aliased_param_index(
    returned_name: &str,
    params: &[String],
    aliases: &HashMap<String, String>,
) -> Option<usize> {
    let mut name = returned_name.to_owned();
    for _ in 0..=aliases.len() {
        if let Some(index) = params.iter().position(|param| param == &name) {
            return Some(index);
        }
        name = aliases.get(&name)?.clone();
    }
    None
}

fn identifier_name(expr: &Expr) -> Option<&String> {
    let ExprKind::Identifier(name) = &expr.kind else {
        return None;
    };
    Some(name)
}

fn is_na_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => name == "na",
        ExprKind::QualifiedName(parts) if parts.len() == 1 => parts[0] == "na",
        _ => false,
    }
}

fn single_expr_branch(branch: &[Stmt]) -> Option<&Expr> {
    let [statement] = branch else {
        return None;
    };
    let StmtKind::Expr(expr) = &statement.kind else {
        return None;
    };
    Some(expr)
}
