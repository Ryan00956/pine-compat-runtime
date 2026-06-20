use crate::prelude::*;

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
                let declared_pine_type =
                    self.declared_pine_type(declared_type.as_deref(), statement.span);
                if let Some(target_type) = declared_pine_type {
                    self.validate_typed_declaration(name, target_type, value_type, statement.span);
                    self.compatibility.supported.push(FeatureUse {
                        feature: format!(
                            "{} typed declarations",
                            declared_type.as_deref().unwrap_or("typed")
                        ),
                        span: statement.span,
                    });
                }
                let symbol_type = declared_pine_type.unwrap_or(value_type);
                let (persistence, var_slot_id) =
                    self.declaration_persistence(*mode, symbol_type, statement.span);
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
                if let Some(type_name) = self.user_type_name_of_expr(value) {
                    self.mark_symbol_user_type(symbol, type_name);
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
                        && self.expr_user_type_name(value).as_ref() != Some(target_type_name)
                    {
                        self.diagnostics.push(Diagnostic::error(
                            "E_UDT_ASSIGN_TYPE",
                            format!("cannot assign a different user-defined type to `{name}`"),
                            statement.span,
                        ));
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
                    Some((target.pine_type, "chart.point field mutation"))
                } else {
                    self.resolve_user_type_field_mutation(receiver, field, statement.span)
                        .map(|target| (target.pine_type, "user-defined type field mutation"))
                };
                if self.function_depth > 0 {
                    let reason = match target.as_ref().map(|(_, feature)| *feature) {
                        Some("user-defined type field mutation") => {
                            "mutating user-defined type fields inside user-defined functions or methods is not supported"
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
                if let (Some((target_type, feature)), Some(value_type)) = (target, value_type) {
                    self.validate_assignment(
                        &format!("{receiver}.{field}"),
                        target_type,
                        value_type,
                        statement.span,
                    );
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

    fn declaration_persistence(
        &mut self,
        mode: pine_syntax::DeclMode,
        value_type: PineType,
        span: Span,
    ) -> (PersistenceKind, Option<pine_ir::VarSlotId>) {
        match mode {
            pine_syntax::DeclMode::Normal => (PersistenceKind::None, None),
            pine_syntax::DeclMode::Var => (PersistenceKind::Var, Some(self.alloc_var_slot())),
            pine_syntax::DeclMode::Varip => {
                if is_drawing_id_value(value_type.kind) {
                    self.unsupported("varip", VARIP_DRAWING_UNSUPPORTED_REASON, span);
                    return (PersistenceKind::None, None);
                }
                if !is_supported_varip_value(value_type.kind) {
                    self.unsupported("varip", VARIP_VALUE_UNSUPPORTED_REASON, span);
                    return (PersistenceKind::None, None);
                }
                self.compatibility.supported.push(FeatureUse {
                    feature: "varip".to_owned(),
                    span,
                });
                (PersistenceKind::Varip, Some(self.alloc_var_slot()))
            }
        }
    }

    fn declared_pine_type(&mut self, declared_type: Option<&str>, span: Span) -> Option<PineType> {
        match declared_type {
            Some("int") => Some(PineType::new(Qualifier::Series, ValueKind::Int)),
            Some("float") => Some(PineType::new(Qualifier::Series, ValueKind::Float)),
            Some("bool") => Some(PineType::new(Qualifier::Series, ValueKind::Bool)),
            Some("string") => Some(PineType::new(Qualifier::Series, ValueKind::String)),
            Some("color") => Some(PineType::new(Qualifier::Series, ValueKind::Color)),
            Some("label") => Some(PineType::new(Qualifier::Series, ValueKind::Label)),
            Some("line") => Some(PineType::new(Qualifier::Series, ValueKind::Line)),
            Some("linefill") => Some(PineType::new(Qualifier::Series, ValueKind::LineFill)),
            Some("polyline") => Some(PineType::new(Qualifier::Series, ValueKind::Polyline)),
            Some("box") => Some(PineType::new(Qualifier::Series, ValueKind::Box)),
            Some("table") => Some(PineType::new(Qualifier::Series, ValueKind::Table)),
            Some("chart.point") => Some(PineType::new(Qualifier::Series, ValueKind::ChartPoint)),
            Some("array<int>") => Some(PineType::new(Qualifier::Series, ValueKind::IntArray)),
            Some("array<float>") => Some(PineType::new(Qualifier::Series, ValueKind::FloatArray)),
            Some("array<bool>") => Some(PineType::new(Qualifier::Series, ValueKind::BoolArray)),
            Some("array<string>") => Some(PineType::new(Qualifier::Series, ValueKind::StringArray)),
            Some("array<color>") => Some(PineType::new(Qualifier::Series, ValueKind::ColorArray)),
            Some("array<label>") => Some(PineType::new(Qualifier::Series, ValueKind::LabelArray)),
            Some("array<line>") => Some(PineType::new(Qualifier::Series, ValueKind::LineArray)),
            Some("array<linefill>") => {
                Some(PineType::new(Qualifier::Series, ValueKind::LineFillArray))
            }
            Some("array<polyline>") => {
                Some(PineType::new(Qualifier::Series, ValueKind::PolylineArray))
            }
            Some("array<box>") => Some(PineType::new(Qualifier::Series, ValueKind::BoxArray)),
            Some("array<table>") => Some(PineType::new(Qualifier::Series, ValueKind::TableArray)),
            Some("array<chart.point>") => {
                Some(PineType::new(Qualifier::Series, ValueKind::ChartPointArray))
            }
            Some(type_name) => {
                self.diagnostics.push(Diagnostic::error(
                    "E_DECL_TYPE",
                    format!("typed declaration `{type_name}` is not supported"),
                    span,
                ));
                None
            }
            None => None,
        }
    }

    fn validate_typed_declaration(
        &mut self,
        name: &str,
        target_type: PineType,
        value_type: PineType,
        span: Span,
    ) {
        if can_assign(target_type, value_type) || value_type.kind == ValueKind::Na {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_ASSIGN_TYPE",
            format!(
                "cannot initialize `{name}` of type {} with {:?} {:?}",
                typed_declaration_name(target_type.kind),
                value_type.qualifier,
                value_type.kind
            ),
            span,
        ));
    }

    pub(crate) fn analyze_tuple_decl(&mut self, statement: &pine_syntax::Stmt) {
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
}

fn is_supported_varip_value(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::Int
            | ValueKind::Float
            | ValueKind::Bool
            | ValueKind::String
            | ValueKind::Color
            | ValueKind::Na
    ) || is_supported_varip_array(kind)
}

fn reassigned_symbol_type(target_type: PineType, value_type: PineType) -> PineType {
    PineType::new(
        strongest_qualifier(target_type.qualifier, value_type.qualifier),
        common_kind(target_type.kind, value_type.kind).unwrap_or(target_type.kind),
    )
}

fn typed_declaration_name(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Int => "int",
        ValueKind::Float => "float",
        ValueKind::Bool => "bool",
        ValueKind::String => "string",
        ValueKind::Color => "color",
        ValueKind::Label => "label",
        ValueKind::Line => "line",
        ValueKind::LineFill => "linefill",
        ValueKind::Polyline => "polyline",
        ValueKind::Box => "box",
        ValueKind::Table => "table",
        ValueKind::ChartPoint => "chart.point",
        ValueKind::IntArray => "array<int>",
        ValueKind::FloatArray => "array<float>",
        ValueKind::BoolArray => "array<bool>",
        ValueKind::StringArray => "array<string>",
        ValueKind::ColorArray => "array<color>",
        ValueKind::LabelArray => "array<label>",
        ValueKind::LineArray => "array<line>",
        ValueKind::LineFillArray => "array<linefill>",
        ValueKind::PolylineArray => "array<polyline>",
        ValueKind::BoxArray => "array<box>",
        ValueKind::TableArray => "array<table>",
        ValueKind::ChartPointArray => "array<chart.point>",
        _ => "typed",
    }
}

fn is_supported_varip_array(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::FloatArray
            | ValueKind::IntArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
    )
}

fn is_drawing_id_value(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::Label
            | ValueKind::Line
            | ValueKind::LineFill
            | ValueKind::Polyline
            | ValueKind::Box
            | ValueKind::Table
    )
}
