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
                let Some(pine_type) = self.method_param_type(&param.type_name, param.span) else {
                    valid = false;
                    continue;
                };
                params.push(MethodParamInfo {
                    name: param.name.clone(),
                    pine_type,
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

        let mut resolved_arg_types = vec![None; method.params.len()];
        for (arg_index, param_index) in arg_indices.iter().copied().enumerate() {
            resolved_arg_types[param_index] = arg_types.get(arg_index).copied().flatten();
        }
        for (param, arg_type) in method.params.iter().zip(resolved_arg_types) {
            let arg_type = arg_type.unwrap_or(UNKNOWN);
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
            self.define_local_symbol(&param.name, arg_type, None, false);
        }
        self.function_stack.push(stack_name);
        self.function_depth += 1;
        let return_type = self.analyze_function_body(&method.body, method.span);
        self.function_depth -= 1;
        self.function_stack.pop();
        self.scope.pop_scope();
        Some(return_type)
    }

    fn method_param_type(&mut self, name: &str, span: Span) -> Option<PineType> {
        let kind = match name {
            "int" => ValueKind::Int,
            "float" => ValueKind::Float,
            "bool" => ValueKind::Bool,
            "string" => ValueKind::String,
            "color" => ValueKind::Color,
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_PARAM",
                    format!("unsupported or unknown method parameter type `{name}`"),
                    span,
                ));
                return None;
            }
        };
        Some(PineType::new(Qualifier::Series, kind))
    }
}
