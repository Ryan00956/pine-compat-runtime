use super::*;
use crate::modules::{ImportedUserTypeFieldInfo, ImportedUserTypeIdentity, ImportedUserTypeInfo};

fn field(name: &str, kind: ValueKind, user_type_name: Option<&str>) -> UserTypeFieldInfo {
    UserTypeFieldInfo {
        name: name.to_owned(),
        pine_type: PineType::new(Qualifier::Series, kind),
        user_type_name: user_type_name.map(str::to_owned),
    }
}

fn scalar_type(name: &str) -> UserTypeInfo {
    UserTypeInfo {
        identity: UserTypeIdentity {
            source_id: SourceId::root(),
            name: name.to_owned(),
        },
        name: name.to_owned(),
        fields: vec![
            field("x", ValueKind::Float, None),
            field("label", ValueKind::String, None),
            field("active", ValueKind::Bool, None),
            field("shade", ValueKind::Color, None),
        ],
    }
}

fn imported_type(
    source_id: SourceId,
    name: &str,
    fields: Vec<ImportedUserTypeFieldInfo>,
) -> ImportedUserTypeInfo {
    ImportedUserTypeInfo {
        identity: ImportedUserTypeIdentity {
            source_id,
            name: name.to_owned(),
        },
        fields,
        span: Span::new(100, 140),
    }
}

fn imported_field(
    name: &str,
    type_name: &str,
    pine_type: Option<PineType>,
) -> ImportedUserTypeFieldInfo {
    ImportedUserTypeFieldInfo {
        name: name.to_owned(),
        type_name: type_name.to_owned(),
        pine_type,
        span: Span::new(120, 130),
    }
}

fn analyzer() -> Analyzer {
    use crate::compatibility::CompatibilityReport;
    use crate::resolver::ScopeResolver;
    use crate::symbols::{
        initial_series_count, initial_symbol_count, initial_symbol_order, initial_symbols,
    };

    Analyzer {
        diagnostics: Vec::new(),
        compatibility: CompatibilityReport::default(),
        scope: ScopeResolver::new(initial_symbols(), initial_symbol_order()),
        bindings: HashMap::new(),
        lower_symbol_overrides: Vec::new(),
        functions: HashMap::new(),
        methods: HashMap::new(),
        imported_user_types: HashMap::new(),
        user_types: HashMap::new(),
        symbol_user_types: HashMap::new(),
        symbol_user_type_identities: HashMap::new(),
        symbol_user_type_arrays: HashMap::new(),
        symbol_maps: HashMap::new(),
        expr_user_types: HashMap::new(),
        expr_user_type_identities: HashMap::new(),
        expr_user_type_arrays: HashMap::new(),
        expr_maps: HashMap::new(),
        expr_types: HashMap::new(),
        script_declaration: None,
        strategy_settings: Default::default(),
        drawing_settings: Default::default(),
        function_stack: Vec::new(),
        function_param_symbols: Vec::new(),
        function_context_is_method: Vec::new(),
        next_symbol_id: initial_symbol_count(),
        next_series_id: initial_series_count(),
        next_call_site_id: 0,
        next_var_slot_id: 0,
        block_depth: 0,
        function_depth: 0,
        loop_depth: 0,
        expr_depth: 0,
        lowering_limits: Default::default(),
        lowering_inline_depth: 0,
        lowered_hir_nodes: 0,
        lowered_temp_symbols: 0,
        lowering_budget_reported: false,
    }
}

fn identifier(name: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_owned()),
        span,
    }
}

fn qualified_name(name: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::QualifiedName(vec![name.to_owned()]),
        span,
    }
}

fn call_arg(name: Option<&str>, value_name: &str) -> CallArg {
    CallArg {
        name: name.map(str::to_owned),
        value: identifier(value_name, Span::new(1, 2)),
        span: Span::new(1, 2),
    }
}

#[test]
fn resolves_imported_user_type_constructor_metadata_without_accepting_it() {
    let mut analyzer = analyzer();
    analyzer.imported_user_types.insert(
        "lib.Point".to_owned(),
        imported_type(
            SourceId::library(0),
            "Point",
            vec![imported_field(
                "x",
                "float",
                Some(PineType::new(Qualifier::Series, ValueKind::Float)),
            )],
        ),
    );

    let point = analyzer
        .imported_user_type_constructor_metadata("lib.Point.new")
        .expect("imported UDT constructor metadata");

    assert_eq!(point.identity.source_id, SourceId::library(0));
    assert_eq!(point.identity.name, "Point");
    assert!(analyzer.imported_user_type_has_scalar_fields(point));
    assert_eq!(
        analyzer.imported_user_type_constructor_has_scalar_fields("lib.Point.new"),
        Some(true)
    );
    assert!(
        analyzer
            .imported_user_type_constructor_metadata("Point.new")
            .is_none()
    );
    assert!(
        analyzer
            .imported_user_type_constructor_metadata("lib.Point")
            .is_none()
    );
}

#[test]
fn plans_imported_user_type_constructor_args_without_accepting_it() {
    let mut analyzer = analyzer();
    analyzer.imported_user_types.insert(
        "lib.Point".to_owned(),
        imported_type(
            SourceId::library(0),
            "Point",
            vec![
                imported_field(
                    "x",
                    "float",
                    Some(PineType::new(Qualifier::Series, ValueKind::Float)),
                ),
                imported_field(
                    "label",
                    "string",
                    Some(PineType::new(Qualifier::Series, ValueKind::String)),
                ),
            ],
        ),
    );

    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[call_arg(None, "close"), call_arg(Some("label"), "name")]
        ),
        Some(Ok(ImportedUdtConstructorArgPlan {
            scalar_fields: true,
            field_arg_indices: vec![0, 1],
        }))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[
                call_arg(Some("label"), "name"),
                call_arg(Some("x"), "close")
            ]
        ),
        Some(Ok(ImportedUdtConstructorArgPlan {
            scalar_fields: true,
            field_arg_indices: vec![1, 0],
        }))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[call_arg(Some("label"), "name")]
        ),
        Some(Err(ImportedUdtConstructorArgError::MissingField(
            "x".to_owned()
        )))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[call_arg(Some("missing"), "close")]
        ),
        Some(Err(ImportedUdtConstructorArgError::UnknownField(
            "missing".to_owned()
        )))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[call_arg(Some("x"), "close"), call_arg(Some("x"), "open")]
        ),
        Some(Err(ImportedUdtConstructorArgError::DuplicateField(
            "x".to_owned()
        )))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[call_arg(Some("x"), "close"), call_arg(None, "name")]
        ),
        Some(Err(ImportedUdtConstructorArgError::PositionalAfterNamed))
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Point.new",
            &[
                call_arg(None, "close"),
                call_arg(None, "name"),
                call_arg(None, "extra")
            ]
        ),
        Some(Err(ImportedUdtConstructorArgError::TooManyArgs {
            expected: 2,
            actual: 3,
        }))
    );
}

#[test]
fn detects_imported_user_type_constructor_with_deferred_field_family() {
    let mut analyzer = analyzer();
    analyzer.imported_user_types.insert(
        "lib.Wrapper".to_owned(),
        imported_type(
            SourceId::library(1),
            "Wrapper",
            vec![imported_field("nested", "Other", None)],
        ),
    );

    let wrapper = analyzer
        .imported_user_type_constructor_metadata("lib.Wrapper.new")
        .expect("imported UDT constructor metadata");

    assert!(!analyzer.imported_user_type_has_scalar_fields(wrapper));
    assert_eq!(
        analyzer.imported_user_type_constructor_has_scalar_fields("lib.Wrapper.new"),
        Some(false)
    );
    assert_eq!(
        analyzer.imported_user_type_constructor_arg_plan(
            "lib.Wrapper.new",
            &[call_arg(Some("nested"), "value")]
        ),
        Some(Ok(ImportedUdtConstructorArgPlan {
            scalar_fields: false,
            field_arg_indices: vec![0],
        }))
    );
}

#[test]
fn registers_local_user_type_identity_as_root_source() {
    let mut analyzer = analyzer();
    let program = pine_syntax::parse_source(&pine_syntax::SourceFile::new(
        "root.pine",
        r#"type Point
    float x
"#,
    ))
    .program;

    analyzer.register_user_types(&program);

    let point = analyzer.user_types.get("Point").expect("registered UDT");
    assert_eq!(point.identity.source_id, SourceId::root());
    assert_eq!(point.identity.name, "Point");
}

#[test]
fn mirrors_user_type_identity_for_symbols_and_expr_spans() {
    let mut analyzer = analyzer();
    analyzer
        .user_types
        .insert("Point".to_owned(), scalar_type("Point"));
    let symbol = analyzer.define_symbol(
        "point",
        PineType::new(Qualifier::Series, ValueKind::UserType),
        None,
    );

    analyzer.mark_symbol_user_type(symbol, "Point".to_owned());
    analyzer.mark_expr_user_type(Span::new(10, 20), "Point".to_owned());

    let symbol_identity = analyzer
        .symbol_user_type_identities
        .get(&symbol.id)
        .expect("symbol UDT identity");
    assert_eq!(symbol_identity.source_id, SourceId::root());
    assert_eq!(symbol_identity.name, "Point");

    let expr = identifier("point", Span::new(10, 20));
    let expr_identity = analyzer
        .expr_user_type_identity(&expr)
        .expect("expr UDT identity");
    assert_eq!(expr_identity.source_id, SourceId::root());
    assert_eq!(expr_identity.name, "Point");
    assert_eq!(
        analyzer.expr_user_type_name(&expr),
        Some("Point".to_owned())
    );
}

#[test]
fn mirrors_user_type_identity_when_marking_symbol_id_for_lowering() {
    let mut analyzer = analyzer();
    analyzer
        .user_types
        .insert("Point".to_owned(), scalar_type("Point"));
    let symbol = analyzer.define_symbol(
        "lowered_point",
        PineType::new(Qualifier::Series, ValueKind::UserType),
        None,
    );

    analyzer.mark_symbol_id_user_type(symbol.id, "Point".to_owned());

    assert_eq!(
        analyzer.symbol_user_types.get(&symbol.id),
        Some(&"Point".to_owned())
    );
    let identity = analyzer
        .symbol_user_type_identities
        .get(&symbol.id)
        .expect("lowered symbol UDT identity");
    assert_eq!(identity.source_id, SourceId::root());
    assert_eq!(identity.name, "Point");
}

#[test]
fn tracks_user_type_array_names_for_symbols_and_expr_spans() {
    let mut analyzer = analyzer();
    let symbol = analyzer.define_symbol(
        "points",
        PineType::new(Qualifier::Series, ValueKind::UserTypeArray),
        None,
    );
    analyzer.mark_symbol_user_type_array(symbol, "Point".to_owned());

    assert_eq!(
        analyzer.user_type_array_name_of_expr(&identifier("points", Span::new(1, 7))),
        Some("Point".to_owned())
    );
    assert_eq!(
        analyzer.user_type_array_name_of_expr(&qualified_name("points", Span::new(10, 16))),
        Some("Point".to_owned())
    );

    let array_from_expr = identifier("array.from", Span::new(20, 30));
    analyzer.mark_expr_user_type_array(array_from_expr.span, "Point".to_owned());

    assert_eq!(
        analyzer.expr_user_type_array_name(&array_from_expr),
        Some("Point".to_owned())
    );
    assert_eq!(
        analyzer.user_type_array_name_of_expr(&array_from_expr),
        Some("Point".to_owned())
    );

    let concat_expr = identifier("array.concat", Span::new(40, 52));
    analyzer.mark_expr_user_type_array(concat_expr.span, "Point".to_owned());

    assert_eq!(
        analyzer.user_type_array_name_of_expr(&concat_expr),
        Some("Point".to_owned())
    );
}

#[test]
fn classifies_same_scalar_local_user_type_array_elements() {
    let mut user_types = HashMap::new();
    user_types.insert("Point".to_owned(), scalar_type("Point"));
    assert_eq!(
        classify_user_type_array_element_names(
            &user_types,
            &["Point".to_owned(), "Point".to_owned()],
        ),
        Some(UserTypeArrayElementInference::SameScalarLocal(
            "Point".to_owned()
        ))
    );
}

#[test]
fn classifies_mixed_local_user_type_array_elements() {
    let mut user_types = HashMap::new();
    user_types.insert("Point".to_owned(), scalar_type("Point"));
    user_types.insert("Marker".to_owned(), scalar_type("Marker"));
    assert_eq!(
        classify_user_type_array_element_names(
            &user_types,
            &["Point".to_owned(), "Marker".to_owned()],
        ),
        Some(UserTypeArrayElementInference::MixedLocal)
    );
}

#[test]
fn classifies_user_type_array_elements_with_nested_fields() {
    let mut user_types = HashMap::new();
    user_types.insert("Point".to_owned(), scalar_type("Point"));
    user_types.insert(
        "Wrapper".to_owned(),
        UserTypeInfo {
            identity: UserTypeIdentity {
                source_id: SourceId::root(),
                name: "Wrapper".to_owned(),
            },
            name: "Wrapper".to_owned(),
            fields: vec![
                field("inner", ValueKind::UserType, Some("Point")),
                field("weight", ValueKind::Float, None),
            ],
        },
    );

    assert_eq!(
        classify_user_type_array_element_names(
            &user_types,
            &["Wrapper".to_owned(), "Wrapper".to_owned()],
        ),
        Some(UserTypeArrayElementInference::UnsupportedFieldType(
            "Wrapper".to_owned()
        ))
    );
}
