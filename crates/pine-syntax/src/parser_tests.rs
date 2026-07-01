use crate::{
    BinaryOp, DeclMode, DeclaredType, ExprKind, FunctionBody, Literal, Parse, SourceFile, StmtKind,
    SwitchArmResult, parse_source,
};

fn parse(text: &str) -> Parse {
    parse_source(&SourceFile::new("test.pine", text))
}

fn declared_type_name(declared_type: &Option<DeclaredType>) -> Option<String> {
    declared_type.as_ref().map(DeclaredType::canonical_name)
}

fn first_declared_type(parsed: &Parse) -> Option<String> {
    let StmtKind::Decl { declared_type, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    declared_type_name(declared_type)
}

#[test]
fn declared_type_canonical_names_match_existing_ast_strings() {
    assert_eq!(
        DeclaredType::Named("chart.point".to_owned()).into_canonical_name(),
        "chart.point"
    );
    assert_eq!(
        DeclaredType::Array {
            element_type: "chart.point".to_owned()
        }
        .into_canonical_name(),
        "array<chart.point>"
    );
}

#[test]
fn parses_version_and_indicator() {
    let parsed = parse("//@version=5\nindicator(\"Demo\", overlay=true)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        parsed.program.version.map(|version| version.version),
        Some(5)
    );
    assert_eq!(parsed.program.statements.len(), 1);
}

#[test]
fn parses_declaration_with_history_call() {
    let parsed = parse("x = ta.sma(close, 20)[1]\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 1);
    assert!(matches!(
        parsed.program.statements[0].kind,
        StmtKind::Decl { .. }
    ));
}

#[test]
fn parses_chart_point_array_new_template_call() {
    let parsed = parse("points = array.new<chart.point>(2, chart.point.now(close))\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("array.new<chart.point>".to_owned())
    );
    assert_eq!(args.len(), 2);
}

#[test]
fn parses_scalar_array_new_template_call() {
    let parsed = parse("values = array.new<float>(2, close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("array.new_float".to_owned())
    );
    assert_eq!(args.len(), 2);
}

#[test]
fn parses_udt_array_new_template_call() {
    let parsed = parse("points = array.new<Point>(2, Point.new(close))\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("array.new<Point>".to_owned())
    );
    assert_eq!(args.len(), 2);
}

#[test]
fn parses_matrix_new_template_call() {
    let parsed = parse("values = matrix.new<float>(2, 2, close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("matrix.new<float>".to_owned())
    );
    assert_eq!(args.len(), 3);
}

#[test]
fn parses_deferred_matrix_new_template_call() {
    let parsed = parse("points = matrix.new<chart.point>(2, 2, chart.point.now(close))\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("matrix.new<chart.point>".to_owned())
    );
    assert_eq!(args.len(), 3);
}

#[test]
fn parses_map_new_template_call() {
    let parsed = parse("values = map.new<string, float>()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("map.new<string,float>".to_owned())
    );
    assert_eq!(args.len(), 0);
}

#[test]
fn parses_dotted_map_new_template_call() {
    let parsed = parse("values = map.new<chart.point, chart.point>()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("map.new<chart.point,chart.point>".to_owned())
    );
    assert_eq!(args.len(), 0);
}

#[test]
fn parses_object_array_new_template_call() {
    let parsed = parse("labels = array.new<label>(1, label.new(bar_index, close))\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("array.new_label".to_owned())
    );
    assert_eq!(args.len(), 2);
}

#[test]
fn parses_polyline_array_new_template_call() {
    let parsed = parse("paths = array.new<polyline>(1, polyline.new(points))\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(
        callee.kind,
        ExprKind::Identifier("array.new_polyline".to_owned())
    );
    assert_eq!(args.len(), 2);
}

#[test]
fn parses_chart_point_typed_declaration() {
    let parsed = parse("chart.point p = chart.point.now(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("chart.point".to_owned())
    );
    assert_eq!(name, "p");
}

#[test]
fn parses_dotted_named_typed_declaration() {
    let parsed = parse("lib.Point p = lib.Point.new(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("lib.Point".to_owned())
    );
    assert_eq!(name, "p");
}

#[test]
fn parses_var_chart_point_typed_declaration() {
    let parsed = parse("var chart.point p = na\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Var);
    assert_eq!(
        declared_type_name(declared_type),
        Some("chart.point".to_owned())
    );
    assert_eq!(name, "p");
}

#[test]
fn parses_scalar_typed_declaration() {
    let parsed = parse("float price = close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(declared_type_name(declared_type), Some("float".to_owned()));
    assert_eq!(name, "price");
}

#[test]
fn parses_array_typed_declaration() {
    let parsed = parse("array<float> prices = array.new_float()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<float>".to_owned())
    );
    assert_eq!(name, "prices");
}

#[test]
fn parses_dotted_array_typed_declaration() {
    let parsed = parse("array<chart.point> points = array.new<chart.point>()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<chart.point>".to_owned())
    );
    assert_eq!(name, "points");
}

#[test]
fn parses_polyline_array_typed_declaration() {
    let parsed = parse("array<polyline> paths = array.new<polyline>()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<polyline>".to_owned())
    );
    assert_eq!(name, "paths");
}

#[test]
fn parses_matrix_typed_declaration() {
    let parsed = parse("matrix<float> values = matrix.new<float>(1, 1)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("matrix<float>".to_owned())
    );
    assert_eq!(name, "values");
}

#[test]
fn parses_map_typed_declaration() {
    let parsed = parse("map<string, float> values = na\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("map<string,float>".to_owned())
    );
    assert_eq!(name, "values");
}

#[test]
fn parses_array_type_alias_declaration() {
    let parsed = parse("float[] prices = array.new_float()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<float>".to_owned())
    );
    assert_eq!(name, "prices");
}

#[test]
fn parses_var_array_type_alias_declaration() {
    let parsed = parse("var float[] prices = array.new_float()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Var);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<float>".to_owned())
    );
    assert_eq!(name, "prices");
}

#[test]
fn parses_varip_array_type_alias_declaration() {
    let parsed = parse("varip int[] counts = array.new_int()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Varip);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<int>".to_owned())
    );
    assert_eq!(name, "counts");
}

#[test]
fn parses_dotted_array_type_alias_declaration() {
    let parsed = parse("chart.point[] points = array.new<chart.point>()\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        mode,
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(*mode, DeclMode::Normal);
    assert_eq!(
        declared_type_name(declared_type),
        Some("array<chart.point>".to_owned())
    );
    assert_eq!(name, "points");
}

#[test]
fn canonicalizes_array_template_and_alias_declarations() {
    for source in [
        "array < float > prices = array.new_float()\n",
        "float [] prices = array.new_float()\n",
    ] {
        let parsed = parse(source);

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(
            first_declared_type(&parsed),
            Some("array<float>".to_owned())
        );
    }
}

#[test]
fn canonicalizes_dotted_array_template_and_alias_declarations() {
    for source in [
        "array < chart.point > points = array.new<chart.point>()\n",
        "chart.point [] points = array.new<chart.point>()\n",
    ] {
        let parsed = parse(source);

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(
            first_declared_type(&parsed),
            Some("array<chart.point>".to_owned())
        );
    }
}

#[test]
fn parses_unknown_typed_declaration_for_semantic_diagnostic() {
    let parsed = parse("line id = na\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl {
        declared_type,
        name,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected declaration");
    };
    assert_eq!(declared_type_name(declared_type), Some("line".to_owned()));
    assert_eq!(name, "id");
}

#[test]
fn parses_array_new_comparison_without_template_rewrite() {
    let parsed = parse("is_less = array.new < close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    assert!(matches!(
        value.kind,
        ExprKind::Binary {
            op: BinaryOp::Lt,
            ..
        }
    ));
}

#[test]
fn parses_reassignment() {
    let parsed = parse("x := x + 1\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.program.statements[0].kind,
        StmtKind::Reassign { .. }
    ));
}

#[test]
fn recovers_after_bad_declaration() {
    let parsed = parse("x =\ny = close\n");

    assert_eq!(parsed.program.statements.len(), 1);
    assert!(!parsed.diagnostics.is_empty());
    assert!(matches!(
        parsed.program.statements[0].kind,
        StmtKind::Decl { .. }
    ));
}

#[test]
fn parses_tuple_declaration() {
    let parsed = parse("[macd, signal, hist] = ta.macd(close, 12, 26, 9)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.program.statements[0].kind,
        StmtKind::TupleDecl { .. }
    ));
}

#[test]
fn parses_tuple_expression() {
    let parsed = parse("x = [close, open]\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.program.statements[0].kind,
        StmtKind::Decl { .. }
    ));
}

#[test]
fn parses_if_statement() {
    let parsed = parse("if close > open\n    plot(close)\nelse\n    plot(open)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 1);
    let StmtKind::If {
        then_branch,
        else_branch,
        ..
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected if statement");
    };
    assert_eq!(then_branch.len(), 1);
    assert_eq!(else_branch.len(), 1);
}

#[test]
fn parses_if_expression_declaration() {
    let parsed = parse("x = if close > open\n    high\nelse\n    low\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::If {
        condition,
        then_branch,
        else_branch,
    } = &value.kind
    else {
        panic!("expected if expression");
    };
    assert!(matches!(condition.kind, ExprKind::Binary { .. }));
    assert_eq!(then_branch.len(), 1);
    assert_eq!(else_branch.len(), 1);
    assert!(matches!(then_branch[0].kind, StmtKind::Expr(_)));
    assert!(matches!(else_branch[0].kind, StmtKind::Expr(_)));
}

#[test]
fn rejects_if_expression_without_else() {
    let parsed = parse("x = if close > open\n    high\nplot(x)\n");

    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_PARSE_IF_EXPR"),
        "{:?}",
        parsed.diagnostics
    );
}

#[test]
fn parses_function_declaration() {
    let parsed = parse("double(x) => x * 2\nplot(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 2);
    let StmtKind::Function { name, params, .. } = &parsed.program.statements[0].kind else {
        panic!("expected function statement");
    };
    assert_eq!(name, "double");
    assert_eq!(params, &vec!["x".to_owned()]);
}

#[test]
fn parses_block_function_declaration() {
    let parsed = parse("double(x) =>\n    y = x * 2\n    y\nplot(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 2);
    let StmtKind::Function { body, .. } = &parsed.program.statements[0].kind else {
        panic!("expected function statement");
    };
    let FunctionBody::Block(statements) = body else {
        panic!("expected block function body");
    };
    assert_eq!(statements.len(), 2);
}

#[test]
fn parses_for_statement() {
    let parsed = parse("for i = 0 to 10\n    plot(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 1);
    let StmtKind::For {
        counter,
        from,
        to,
        step,
        body,
    } = &parsed.program.statements[0].kind
    else {
        panic!("expected for statement");
    };
    assert_eq!(counter, "i");
    assert!(matches!(from.kind, ExprKind::Literal(Literal::Int(0))));
    assert!(matches!(to.kind, ExprKind::Literal(Literal::Int(10))));
    assert!(step.is_none());
    assert_eq!(body.len(), 1);
}

#[test]
fn parses_for_statement_with_step() {
    let parsed = parse("for i = 0 to 10 by 2\n    plot(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 1);
    let StmtKind::For { step, .. } = &parsed.program.statements[0].kind else {
        panic!("expected for statement");
    };
    let Some(step) = step else {
        panic!("expected for step");
    };
    assert!(matches!(step.kind, ExprKind::Literal(Literal::Int(2))));
}

#[test]
fn parses_while_statement() {
    let parsed = parse("while close > open\n    plot(close)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 1);
    let StmtKind::While { condition, body } = &parsed.program.statements[0].kind else {
        panic!("expected while statement");
    };
    assert!(matches!(condition.kind, ExprKind::Binary { .. }));
    assert_eq!(body.len(), 1);
}

#[test]
fn parses_for_expression_declaration() {
    let parsed = parse("x = for i = 0 to 2\n    i * 2\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::For { counter, body, .. } = &value.kind else {
        panic!("expected for expression");
    };
    assert_eq!(counter, "i");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::Expr(_)));
}

#[test]
fn parses_for_in_expression_declaration() {
    let parsed = parse("x = for value in values\n    value + 1\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::ForIn {
        index,
        value: loop_value,
        body,
        ..
    } = &value.kind
    else {
        panic!("expected for-in expression");
    };
    assert_eq!(index, &None);
    assert_eq!(loop_value, "value");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::Expr(_)));
}

#[test]
fn parses_for_in_expression_index_value_declaration() {
    let parsed = parse("x = for index, value in values\n    index + value\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::ForIn {
        index,
        value: loop_value,
        body,
        ..
    } = &value.kind
    else {
        panic!("expected for-in expression");
    };
    assert_eq!(index.as_deref(), Some("index"));
    assert_eq!(loop_value, "value");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::Expr(_)));
}

#[test]
fn parses_for_in_expression_bracket_pair_declaration() {
    let parsed = parse("x = for [key, value] in values\n    value\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::ForIn {
        index,
        value: loop_value,
        body,
        ..
    } = &value.kind
    else {
        panic!("expected for-in expression");
    };
    assert_eq!(index.as_deref(), Some("key"));
    assert_eq!(loop_value, "value");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::Expr(_)));
}

#[test]
fn parses_condition_switch_expression_declaration() {
    let parsed = parse(
        "x = switch\n    close > open => high\n    close < open => low\n    => close\nplot(x)\n",
    );

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Switch { selector, arms } = &value.kind else {
        panic!("expected switch expression");
    };
    assert!(selector.is_none());
    assert_eq!(arms.len(), 3);
    assert!(arms[0].condition.is_some());
    assert!(arms[2].condition.is_none());
}

#[test]
fn parses_selector_switch_expression_declaration() {
    let parsed = parse("x = switch direction\n    1 => high\n    -1 => low\n    => close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Switch { selector, arms } = &value.kind else {
        panic!("expected switch expression");
    };
    assert!(selector.is_some());
    assert_eq!(arms.len(), 3);
}

#[test]
fn parses_while_expression() {
    let parsed = parse("x = while close > open\n    close\nplot(x)\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::While { condition, body } = &value.kind else {
        panic!("expected while expression AST");
    };
    assert!(matches!(condition.kind, ExprKind::Binary { .. }));
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::Expr(_)));
}

#[test]
fn parses_condition_switch_statement_block_arm() {
    let parsed = parse("x = switch\n    close > open =>\n        high\n    => close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Switch { arms, .. } = &value.kind else {
        panic!("expected switch expression");
    };
    assert!(matches!(arms[0].result, SwitchArmResult::Block(_)));
    assert!(matches!(arms[1].result, SwitchArmResult::Expr(_)));
}

#[test]
fn parses_selector_switch_statement_block_arm() {
    let parsed = parse("x = switch direction\n    1 =>\n        high\n    => close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Switch { arms, .. } = &value.kind else {
        panic!("expected switch expression");
    };
    assert!(matches!(arms[0].result, SwitchArmResult::Block(_)));
    assert!(matches!(arms[1].result, SwitchArmResult::Expr(_)));
}

#[test]
fn parses_default_statement_block_switch_arm() {
    let parsed = parse("x = switch\n    close > open => high\n    =>\n        close\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::Decl { value, .. } = &parsed.program.statements[0].kind else {
        panic!("expected declaration");
    };
    let ExprKind::Switch { arms, .. } = &value.kind else {
        panic!("expected switch expression");
    };
    assert!(matches!(arms[0].result, SwitchArmResult::Expr(_)));
    assert!(matches!(arms[1].result, SwitchArmResult::Block(_)));
}

#[test]
fn rejects_expression_nesting_past_depth_limit() {
    let depth = 257;
    let source = format!("x = {}close{}\n", "(".repeat(depth), ")".repeat(depth));
    let parsed = parse(&source);

    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_PARSE_EXPR_DEPTH"),
        "{:?}",
        parsed.diagnostics
    );
}

#[test]
fn parses_loop_control_statements() {
    let parsed = parse("for i = 0 to 10\n    if i == 2\n        break\n    continue\n");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let StmtKind::For { body, .. } = &parsed.program.statements[0].kind else {
        panic!("expected for statement");
    };
    let StmtKind::If { then_branch, .. } = &body[0].kind else {
        panic!("expected if statement");
    };
    assert!(matches!(then_branch[0].kind, StmtKind::Break));
    assert!(matches!(body[1].kind, StmtKind::Continue));
}
