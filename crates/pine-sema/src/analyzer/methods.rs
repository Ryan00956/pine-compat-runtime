use crate::analyzer::functions::contains_output_or_declaration_call;
use crate::prelude::*;

impl Analyzer {
    pub(crate) fn register_methods(&mut self, program: &Program) {
        for statement in &program.statements {
            let StmtKind::Method(method) = &statement.kind else {
                continue;
            };
            let Some(receiver) = method.params.first() else {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_PARAM",
                    format!("method `{}` must declare a receiver parameter", method.name),
                    statement.span,
                ));
                continue;
            };
            if !self.user_types.contains_key(&receiver.type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_RECEIVER_TYPE",
                    format!(
                        "method `{}` receiver type `{}` is not a known user-defined type",
                        method.name, receiver.type_name
                    ),
                    receiver.span,
                ));
                continue;
            }
            let key = (receiver.type_name.clone(), method.name.clone());
            if self.methods.contains_key(&key) {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_DUPLICATE",
                    format!(
                        "method `{}` is already defined for `{}`",
                        method.name, receiver.type_name
                    ),
                    method.name_span,
                ));
                continue;
            }
            let mut params = Vec::new();
            let mut names = vec![receiver.name.clone()];
            let mut valid = true;
            for param in method.params.iter().skip(1) {
                if names.iter().any(|name| name == &param.name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_METHOD_PARAM",
                        format!(
                            "method `{}` has duplicate parameter `{}`",
                            method.name, param.name
                        ),
                        param.span,
                    ));
                    valid = false;
                    continue;
                }
                names.push(param.name.clone());
                let Some((pine_type, user_type_name)) =
                    self.method_param_type(&param.type_name, param.span)
                else {
                    valid = false;
                    continue;
                };
                params.push(MethodParamInfo {
                    name: param.name.clone(),
                    pine_type,
                    user_type_name,
                });
            }
            if !valid {
                continue;
            }
            self.methods.insert(
                key,
                MethodInfo {
                    receiver_type: receiver.type_name.clone(),
                    receiver_name: receiver.name.clone(),
                    params,
                    body: method.body.clone(),
                    span: statement.span,
                },
            );
        }
    }

    pub(crate) fn analyze_user_method_call(
        &mut self,
        receiver_name: &str,
        method_name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let receiver_symbol = self.scope.resolve(receiver_name)?;
        let receiver_type_name = self.symbol_user_types.get(&receiver_symbol.id).cloned()?;
        let Some(method) = self
            .methods
            .get(&(receiver_type_name.clone(), method_name.to_owned()))
            .cloned()
        else {
            if self.imported_user_types.contains_key(&receiver_type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNSUPPORTED_METHOD",
                    format!(
                        "imported method `{method_name}` for receiver `{receiver_type_name}` is not supported; imported method dispatch requires imported UDT identity"
                    ),
                    span,
                ));
                return Some(None);
            }
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown method `{method_name}` for `{receiver_type_name}`"),
                span,
            ));
            return Some(None);
        };
        let stack_name = format!("method:{}.{}", method.receiver_type, method_name);
        if self
            .function_stack
            .iter()
            .any(|active| active == &stack_name)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_RECURSIVE_METHOD",
                format!("recursive method `{method_name}` is not supported"),
                span,
            ));
            return Some(None);
        }
        if self.function_stack.len() >= MAX_FUNCTION_CALL_DEPTH {
            self.diagnostics.push(Diagnostic::error(
                "E_FUNCTION_CALL_DEPTH",
                "user-defined method call chain is too deep",
                span,
            ));
            return Some(None);
        }
        for arg in args {
            if contains_output_or_declaration_call(&arg.value) {
                self.unsupported(
                    "function_side_effect",
                    "side-effecting calls cannot be passed as user-defined method arguments",
                    arg.span,
                );
            }
        }
        let params: Vec<_> = method
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let arg_indices = match resolve_udf_arg_indices(&params, args) {
            Ok(arg_indices) => arg_indices,
            Err(error) => {
                self.report_udf_arg_error(method_name, span, params.len(), args.len(), error);
                return Some(None);
            }
        };

        self.compatibility.supported.push(FeatureUse {
            feature: "user-defined methods".to_owned(),
            span: method.span,
        });
        self.scope.push_scope();
        let receiver = self.define_local_symbol(
            &method.receiver_name,
            receiver_symbol.pine_type,
            None,
            false,
        );
        self.mark_symbol_user_type(receiver, method.receiver_type.clone());

        let mut param_symbols = std::collections::HashSet::from([receiver.id]);
        let mut resolved_arg_types = vec![None; method.params.len()];
        let mut resolved_arg_user_types = vec![None; method.params.len()];
        for (arg_index, param_index) in arg_indices.iter().copied().enumerate() {
            resolved_arg_types[param_index] = arg_types.get(arg_index).copied().flatten();
            resolved_arg_user_types[param_index] = args
                .get(arg_index)
                .and_then(|arg| self.user_type_name_of_expr(&arg.value));
        }
        for (param, (arg_type, arg_user_type)) in method
            .params
            .iter()
            .zip(resolved_arg_types.into_iter().zip(resolved_arg_user_types))
        {
            let arg_type = arg_type.unwrap_or(UNKNOWN);
            let symbol = self.define_local_symbol(&param.name, arg_type, None, false);
            param_symbols.insert(symbol.id);
            if !can_assign(param.pine_type, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_ARG_TYPE",
                    format!(
                        "cannot pass {:?} {:?} to method parameter `{}` of type {:?}",
                        arg_type.qualifier, arg_type.kind, param.name, param.pine_type.kind
                    ),
                    span,
                ));
            }
            if let Some(expected_type_name) = &param.user_type_name {
                if arg_user_type.as_deref() == Some(expected_type_name.as_str()) {
                    self.mark_symbol_user_type(symbol, expected_type_name.clone());
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_METHOD_ARG_TYPE",
                        format!(
                            "cannot pass argument to method parameter `{}` of user-defined type `{}`",
                            param.name, expected_type_name
                        ),
                        span,
                    ));
                }
            }
        }
        self.function_stack.push(stack_name);
        self.function_param_symbols.push(param_symbols);
        self.function_context_is_method.push(true);
        self.function_depth += 1;
        let return_type = self.analyze_function_body(&method.body, method.span);
        if return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::UserType)
            && let Some(type_name) = self.user_type_name_of_function_body(&method.body)
        {
            self.mark_expr_user_type(call_span, type_name.clone());
            self.mark_expr_user_type(span, type_name);
        }
        self.function_depth -= 1;
        self.function_context_is_method.pop();
        self.function_param_symbols.pop();
        self.function_stack.pop();
        self.scope.pop_scope();
        Some(return_type)
    }

    pub(crate) fn analyze_alias_qualified_user_method_call(
        &mut self,
        name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let (alias, method_name) = alias_qualified_method_name(name)?;
        if !self.methods.keys().any(|(receiver_type, candidate)| {
            candidate == method_name && receiver_type.starts_with(&format!("{alias}."))
        }) {
            return None;
        }
        let Some(receiver_arg) = args.first() else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!("`{name}` expects a receiver argument"),
                span,
            ));
            return Some(None);
        };
        let ExprKind::Identifier(receiver_name) = &receiver_arg.value.kind else {
            self.diagnostics.push(Diagnostic::error(
                "E_IMPORT_UNSUPPORTED_METHOD",
                format!("alias-qualified imported method `{name}` requires an identifier receiver"),
                receiver_arg.span,
            ));
            return Some(None);
        };
        let receiver_type = arg_types.first().copied().flatten();
        let Some(receiver_user_type) = self.user_type_name_of_expr(&receiver_arg.value) else {
            if let Some(receiver_type) = receiver_type {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_ARG_TYPE",
                    format!(
                        "cannot pass {:?} {:?} as receiver to imported method `{name}`",
                        receiver_type.qualifier, receiver_type.kind
                    ),
                    receiver_arg.span,
                ));
            }
            return Some(None);
        };
        if !receiver_user_type.starts_with(&format!("{alias}.")) {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_ARG_TYPE",
                format!("cannot pass receiver `{receiver_user_type}` to imported method `{name}`"),
                receiver_arg.span,
            ));
            return Some(None);
        }
        if !self
            .methods
            .contains_key(&(receiver_user_type, method_name.to_owned()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown imported method `{name}` for receiver `{receiver_name}`"),
                span,
            ));
            return Some(None);
        }
        self.analyze_user_method_call(
            receiver_name,
            method_name,
            span,
            call_span,
            &args[1..],
            &arg_types[1..],
        )
    }

    fn method_param_type(&mut self, name: &str, span: Span) -> Option<(PineType, Option<String>)> {
        let kind = match name {
            "int" => ValueKind::Int,
            "float" => ValueKind::Float,
            "bool" => ValueKind::Bool,
            "string" => ValueKind::String,
            "color" => ValueKind::Color,
            _ if self.user_types.contains_key(name) => {
                return Some((
                    PineType::new(Qualifier::Series, ValueKind::UserType),
                    Some(name.to_owned()),
                ));
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_PARAM",
                    format!("unsupported or unknown method parameter type `{name}`"),
                    span,
                ));
                return None;
            }
        };
        Some((PineType::new(Qualifier::Series, kind), None))
    }
}
