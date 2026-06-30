use crate::prelude::*;

mod declarations;
mod for_in;

impl Analyzer {
    pub(crate) fn analyze_program(&mut self, program: &Program) {
        self.register_user_types(program);
        self.register_methods(program);
        self.register_functions(program);
        for statement in &program.statements {
            self.analyze_stmt(statement);
        }
    }

    pub(crate) fn analyze_stmt(&mut self, statement: &Stmt) {
        match &statement.kind {
            StmtKind::Expr(expr) => {
                self.analyze_expr(expr);
            }
            StmtKind::Import(_) => {
                self.compatibility.supported.push(FeatureUse {
                    feature: "import".to_owned(),
                    span: statement.span,
                });
            }
            StmtKind::Library(_) => {
                self.unsupported(
                    "library",
                    unsupported_syntax_reason("library"),
                    statement.span,
                );
            }
            StmtKind::Export(_) => {
                self.unsupported(
                    "export",
                    unsupported_syntax_reason("export"),
                    statement.span,
                );
            }
            StmtKind::UserType(_) => {
                if self.block_depth > 0 || self.function_depth > 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "E_UDT_DECL_LOCATION",
                        "user-defined type declarations must be top-level",
                        statement.span,
                    ));
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "user-defined types".to_owned(),
                    span: statement.span,
                });
            }
            StmtKind::Method(_) => {
                if self.block_depth > 0 || self.function_depth > 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "E_METHOD_DECL_LOCATION",
                        "user-defined method declarations must be top-level",
                        statement.span,
                    ));
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "user-defined methods".to_owned(),
                    span: statement.span,
                });
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
            StmtKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => {
                self.analyze_for_in_stmt(index.as_deref(), value, iterable, body, statement.span);
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
            StmtKind::Decl {
                mode,
                declared_type,
                name,
                value,
            } => {
                let value_type = self.analyze_expr(value).unwrap_or(UNKNOWN);
                if value_type.kind == ValueKind::Void {
                    self.diagnostics.push(Diagnostic::error(
                        "E_DECL_VALUE",
                        "declarations must be initialized with a value-producing expression",
                        value.span,
                    ));
                }
                let declared_pine_type =
                    self.declared_pine_type(declared_type.as_ref(), statement.span);
                let declared_user_type_name = declared_type
                    .as_ref()
                    .and_then(DeclaredType::named_type)
                    .filter(|type_name| self.is_known_user_type_name(type_name))
                    .map(str::to_owned);
                let declared_user_type_array_name = declared_type
                    .as_ref()
                    .and_then(|declared_type| self.declared_user_type_array_name(declared_type));
                let inferred_varip_user_type_name = (matches!(mode, pine_syntax::DeclMode::Varip)
                    && declared_user_type_name.is_none())
                .then(|| self.direct_user_type_constructor_name(value))
                .flatten();
                if let Some(target_type) = declared_pine_type {
                    self.validate_typed_declaration(name, target_type, value_type, statement.span);
                    if let Some(target_user_type_name) = declared_user_type_name.as_deref() {
                        self.validate_user_type_value_assignment(
                            name,
                            target_user_type_name,
                            value,
                            value_type,
                            statement.span,
                        );
                    }
                    if let Some(target_type_name) = declared_user_type_array_name.as_deref() {
                        self.validate_user_type_array_value_assignment(
                            name,
                            target_type_name,
                            value,
                            value_type,
                            statement.span,
                        );
                    }
                    self.compatibility.supported.push(FeatureUse {
                        feature: format!(
                            "{} typed declarations",
                            declared_type
                                .as_ref()
                                .map_or_else(|| "typed".to_owned(), DeclaredType::canonical_name)
                        ),
                        span: statement.span,
                    });
                }
                let symbol_type = declared_pine_type.unwrap_or(value_type);
                let (persistence, var_slot_id) = self.declaration_persistence(
                    *mode,
                    symbol_type,
                    declared_user_type_name
                        .as_deref()
                        .or(inferred_varip_user_type_name.as_deref()),
                    statement.span,
                );
                let symbol = if self.block_depth > 0 || self.function_depth > 0 {
                    self.define_local_symbol_with_persistence(
                        name,
                        symbol_type,
                        persistence,
                        var_slot_id,
                        self.function_depth == 0,
                    )
                } else {
                    self.define_symbol_with_persistence(name, symbol_type, persistence, var_slot_id)
                };
                if let Some(type_name) = declared_user_type_name {
                    self.mark_symbol_user_type(symbol, type_name);
                } else if let Some(type_name) = declared_user_type_array_name {
                    self.mark_symbol_user_type_array(symbol, type_name);
                } else if let Some(type_name) = self.user_type_name_of_expr(value) {
                    self.mark_symbol_user_type(symbol, type_name);
                }
                if symbol_type.kind == ValueKind::UserTypeArray
                    && let Some(type_name) = self.user_type_array_name_of_expr(value)
                {
                    self.mark_symbol_user_type_array(symbol, type_name);
                }
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
                    if target_type.kind == ValueKind::UserType
                        && let Some(symbol) = self.scope.resolve(name)
                        && let Some(target_type_name) = self.symbol_user_types.get(&symbol.id)
                    {
                        let target_type_name = target_type_name.clone();
                        self.validate_user_type_value_assignment(
                            name,
                            &target_type_name,
                            value,
                            value_type,
                            statement.span,
                        );
                    }
                    if target_type.kind == ValueKind::UserTypeArray
                        && let Some(symbol) = self.scope.resolve(name)
                        && let Some(target_type_name) = self.symbol_user_type_arrays.get(&symbol.id)
                    {
                        let target_type_name = target_type_name.clone();
                        self.validate_user_type_array_value_assignment(
                            name,
                            &target_type_name,
                            value,
                            value_type,
                            statement.span,
                        );
                    }
                    if can_assign(target_type, value_type) {
                        self.update_symbol_type(
                            name,
                            reassigned_symbol_type(target_type, value_type),
                        );
                    }
                }
                if let Some(symbol) = self.scope.resolve(name) {
                    self.bind_symbol(name, statement.span, symbol);
                }
            }
            StmtKind::FieldReassign {
                receiver,
                field,
                value,
            } => {
                let target = if let Some(target) =
                    self.resolve_chart_point_field_mutation(receiver, field, statement.span)
                {
                    Some((target.pine_type, None, "chart.point field mutation", None))
                } else {
                    self.resolve_user_type_field_mutation(receiver, field, statement.span)
                        .map(|target| {
                            (
                                target.pine_type,
                                target.user_type_name,
                                "user-defined type field mutation",
                                Some(target.receiver_symbol),
                            )
                        })
                };
                let receiver_is_global = self.scope.resolves_to_global(receiver);
                let receiver_is_function_param = target
                    .as_ref()
                    .and_then(|(_, _, _, receiver_symbol)| receiver_symbol.as_ref())
                    .is_some_and(|symbol| {
                        self.function_param_symbols
                            .last()
                            .is_some_and(|params| params.contains(&symbol.id))
                    });
                let is_method_context = self
                    .function_context_is_method
                    .last()
                    .copied()
                    .unwrap_or(false);
                let allowed_function_local_udt_mutation =
                    target
                        .as_ref()
                        .is_some_and(|(_, _, feature, receiver_symbol)| {
                            *feature == "user-defined type field mutation"
                                && !is_method_context
                                && receiver_symbol.is_some()
                                && !receiver_is_global
                                && !receiver_is_function_param
                        });
                if self.function_depth > 0 && !allowed_function_local_udt_mutation {
                    let reason = match target.as_ref().map(|(_, _, feature, _)| *feature) {
                        Some("user-defined type field mutation") if is_method_context => {
                            "mutating user-defined type fields inside methods is not supported"
                        }
                        Some("user-defined type field mutation") if receiver_is_function_param => {
                            "mutating user-defined type parameter fields inside user-defined functions is not supported"
                        }
                        Some("user-defined type field mutation") if receiver_is_global => {
                            "mutating fields on global user-defined type values inside user-defined functions is not supported"
                        }
                        Some("user-defined type field mutation") => {
                            "mutating user-defined type fields inside user-defined functions is not supported"
                        }
                        Some("chart.point field mutation") => {
                            "mutating chart.point fields inside user-defined functions or methods is not supported"
                        }
                        _ => {
                            "mutating object fields inside user-defined functions or methods is not supported"
                        }
                    };
                    self.unsupported("function_side_effect", reason, statement.span);
                }
                let value_type = self.analyze_expr(value);
                if let (Some((target_type, target_user_type, feature, _)), Some(value_type)) =
                    (target, value_type)
                {
                    let name = format!("{receiver}.{field}");
                    if let Some(target_user_type) = target_user_type {
                        self.validate_user_type_field_assignment(
                            &name,
                            &target_user_type,
                            value,
                            value_type,
                            statement.span,
                        );
                    } else {
                        self.validate_assignment(&name, target_type, value_type, statement.span);
                    }
                    self.compatibility.supported.push(FeatureUse {
                        feature: feature.to_owned(),
                        span: statement.span,
                    });
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
}

fn reassigned_symbol_type(target_type: PineType, value_type: PineType) -> PineType {
    PineType::new(
        strongest_qualifier(target_type.qualifier, value_type.qualifier),
        common_kind(target_type.kind, value_type.kind).unwrap_or(target_type.kind),
    )
}
