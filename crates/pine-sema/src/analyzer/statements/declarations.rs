use crate::analyzer::user_types::{
    UserTypeArrayElementInference, classify_user_type_array_element_names,
};
use crate::prelude::*;
use crate::types::is_scalar_array_kind;

impl Analyzer {
    pub(super) fn declaration_persistence(
        &mut self,
        mode: pine_syntax::DeclMode,
        value_type: PineType,
        declared_user_type_name: Option<&str>,
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
                if value_type.kind == ValueKind::UserType {
                    if declared_user_type_name
                        .is_some_and(|type_name| self.is_scalar_field_user_type(type_name))
                    {
                        self.compatibility.supported.push(FeatureUse {
                            feature: "varip".to_owned(),
                            span,
                        });
                        return (PersistenceKind::Varip, Some(self.alloc_var_slot()));
                    }
                    self.unsupported("varip", VARIP_UDT_UNSUPPORTED_REASON, span);
                    return (PersistenceKind::None, None);
                }
                if !is_supported_varip_value(value_type.kind) {
                    self.unsupported(
                        "varip",
                        unsupported_varip_value_reason(value_type.kind),
                        span,
                    );
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

    fn is_scalar_field_user_type(&self, type_name: &str) -> bool {
        if let Some(user_type) = self.imported_user_types.get(type_name) {
            return self.imported_user_type_has_scalar_fields(user_type);
        }
        matches!(
            classify_user_type_array_element_names(&self.user_types, &[type_name.to_owned()]),
            Some(UserTypeArrayElementInference::SameScalarLocal(_))
        )
    }

    pub(super) fn declared_pine_type(
        &mut self,
        declared_type: Option<&DeclaredType>,
        span: Span,
    ) -> Option<PineType> {
        match declared_type {
            Some(DeclaredType::Named(type_name)) => match type_name.as_str() {
                "int" => Some(PineType::new(Qualifier::Series, ValueKind::Int)),
                "float" => Some(PineType::new(Qualifier::Series, ValueKind::Float)),
                "bool" => Some(PineType::new(Qualifier::Series, ValueKind::Bool)),
                "string" => Some(PineType::new(Qualifier::Series, ValueKind::String)),
                "color" => Some(PineType::new(Qualifier::Series, ValueKind::Color)),
                "label" => Some(PineType::new(Qualifier::Series, ValueKind::Label)),
                "line" => Some(PineType::new(Qualifier::Series, ValueKind::Line)),
                "linefill" => Some(PineType::new(Qualifier::Series, ValueKind::LineFill)),
                "polyline" => Some(PineType::new(Qualifier::Series, ValueKind::Polyline)),
                "box" => Some(PineType::new(Qualifier::Series, ValueKind::Box)),
                "table" => Some(PineType::new(Qualifier::Series, ValueKind::Table)),
                "chart.point" => Some(PineType::new(Qualifier::Series, ValueKind::ChartPoint)),
                _ if self.user_types.contains_key(type_name) => {
                    Some(PineType::new(Qualifier::Series, ValueKind::UserType))
                }
                _ if self.imported_user_types.contains_key(type_name) => {
                    Some(PineType::new(Qualifier::Series, ValueKind::UserType))
                }
                _ => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_DECL_TYPE",
                        format!("typed declaration `{type_name}` is not supported"),
                        span,
                    ));
                    None
                }
            },
            Some(declared_type @ DeclaredType::Array { element_type }) => {
                if let Some(kind) = array_kind_from_element_type_name(element_type) {
                    Some(PineType::new(Qualifier::Series, kind))
                } else if let Some(inference) = classify_user_type_array_element_names(
                    &self.user_types,
                    std::slice::from_ref(element_type),
                ) {
                    match inference {
                        UserTypeArrayElementInference::SameScalarLocal(_) => {
                            Some(PineType::new(Qualifier::Series, ValueKind::UserTypeArray))
                        }
                        UserTypeArrayElementInference::UnsupportedFieldType(_) => {
                            self.diagnostics.push(Diagnostic::error(
                                "E_DECL_TYPE",
                                format!(
                                    "typed declaration `{}` does not support UDT arrays with non-scalar fields",
                                    declared_type.canonical_name()
                                ),
                                span,
                            ));
                            None
                        }
                        _ => {
                            self.diagnostics.push(Diagnostic::error(
                                "E_DECL_TYPE",
                                format!(
                                    "typed declaration `{}` is not supported",
                                    declared_type.canonical_name()
                                ),
                                span,
                            ));
                            None
                        }
                    }
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_DECL_TYPE",
                        format!(
                            "typed declaration `{}` is not supported",
                            declared_type.canonical_name()
                        ),
                        span,
                    ));
                    None
                }
            }
            Some(declared_type @ DeclaredType::Matrix { element_type }) => {
                if element_type == "float" {
                    Some(PineType::new(Qualifier::Series, ValueKind::FloatMatrix))
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_DECL_TYPE",
                        format!(
                            "typed declaration `{}` is not supported",
                            declared_type.canonical_name()
                        ),
                        span,
                    ));
                    None
                }
            }
            None => None,
        }
    }

    pub(super) fn declared_user_type_array_name(
        &self,
        declared_type: &DeclaredType,
    ) -> Option<String> {
        let element_type = declared_type.array_element_type()?;
        match classify_user_type_array_element_names(
            &self.user_types,
            std::slice::from_ref(&element_type.to_owned()),
        ) {
            Some(UserTypeArrayElementInference::SameScalarLocal(type_name)) => Some(type_name),
            _ => None,
        }
    }

    pub(super) fn is_known_user_type_name(&self, type_name: &str) -> bool {
        self.user_types.contains_key(type_name) || self.imported_user_types.contains_key(type_name)
    }

    pub(super) fn validate_typed_declaration(
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

    pub(super) fn validate_user_type_field_assignment(
        &mut self,
        name: &str,
        target_user_type: &str,
        value: &pine_syntax::Expr,
        value_type: PineType,
        span: Span,
    ) {
        if self
            .user_type_name_of_expr(value)
            .is_some_and(|actual_user_type| actual_user_type == target_user_type)
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_ASSIGN_TYPE",
            format!(
                "cannot assign {:?} {:?} to `{name}` of user-defined type `{target_user_type}`",
                value_type.qualifier, value_type.kind
            ),
            span,
        ));
    }

    pub(super) fn validate_user_type_value_assignment(
        &mut self,
        name: &str,
        target_user_type: &str,
        value: &pine_syntax::Expr,
        value_type: PineType,
        span: Span,
    ) {
        if value_type.kind == ValueKind::Na
            || self
                .user_type_name_of_expr(value)
                .is_some_and(|actual_user_type| actual_user_type == target_user_type)
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_UDT_ASSIGN_TYPE",
            format!("cannot assign a different user-defined type to `{name}`"),
            span,
        ));
    }

    pub(super) fn validate_user_type_array_value_assignment(
        &mut self,
        name: &str,
        target_user_type: &str,
        value: &pine_syntax::Expr,
        value_type: PineType,
        span: Span,
    ) {
        if value_type.kind == ValueKind::Na
            || self
                .user_type_array_name_of_expr(value)
                .is_some_and(|actual_user_type| actual_user_type == target_user_type)
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_UDT_ASSIGN_TYPE",
            format!("cannot assign a different user-defined type array to `{name}`"),
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

fn unsupported_varip_value_reason(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::UserType => VARIP_UDT_UNSUPPORTED_REASON,
        ValueKind::UserTypeArray => VARIP_UDT_ARRAY_UNSUPPORTED_REASON,
        ValueKind::FloatMatrix => VARIP_MATRIX_UNSUPPORTED_REASON,
        _ => VARIP_VALUE_UNSUPPORTED_REASON,
    }
}

fn typed_declaration_name(kind: ValueKind) -> String {
    if let Some(element_kind) = kind.array_element_kind()
        && let Some(element_name) = typed_value_kind_name(element_kind)
    {
        return format!("array<{element_name}>");
    }
    typed_value_kind_name(kind).unwrap_or("typed").to_owned()
}

fn typed_value_kind_name(kind: ValueKind) -> Option<&'static str> {
    match kind {
        ValueKind::Int => Some("int"),
        ValueKind::Float => Some("float"),
        ValueKind::Bool => Some("bool"),
        ValueKind::String => Some("string"),
        ValueKind::Color => Some("color"),
        ValueKind::Label => Some("label"),
        ValueKind::Line => Some("line"),
        ValueKind::LineFill => Some("linefill"),
        ValueKind::Polyline => Some("polyline"),
        ValueKind::Box => Some("box"),
        ValueKind::Table => Some("table"),
        ValueKind::ChartPoint => Some("chart.point"),
        ValueKind::FloatMatrix => Some("matrix<float>"),
        _ => None,
    }
}

fn is_supported_varip_array(kind: ValueKind) -> bool {
    is_scalar_array_kind(kind)
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
