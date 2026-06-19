use crate::{
    BinaryOp, DeclMode, ExprKind, FunctionBody, Literal, Parse, SourceFile, StmtKind, parse_source,
};

fn parse(text: &str) -> Parse {
    parse_source(&SourceFile::new("test.pine", text))
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
    assert_eq!(declared_type.as_deref(), Some("chart.point"));
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
    assert_eq!(declared_type.as_deref(), Some("chart.point"));
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
    assert_eq!(declared_type.as_deref(), Some("float"));
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
    assert_eq!(declared_type.as_deref(), Some("array<float>"));
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
    assert_eq!(declared_type.as_deref(), Some("array<chart.point>"));
    assert_eq!(name, "points");
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
    assert_eq!(declared_type.as_deref(), Some("array<float>"));
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
    assert_eq!(declared_type.as_deref(), Some("array<float>"));
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
    assert_eq!(declared_type.as_deref(), Some("array<int>"));
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
    assert_eq!(declared_type.as_deref(), Some("array<chart.point>"));
    assert_eq!(name, "points");
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
    assert_eq!(declared_type.as_deref(), Some("line"));
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
fn rejects_while_expression() {
    let parsed = parse("x = while close > open\n    close\nplot(x)\n");

    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_PARSE_WHILE_EXPR"),
        "{:?}",
        parsed.diagnostics
    );
}

#[test]
fn rejects_statement_block_switch_arm() {
    let parsed = parse("x = switch\n    close > open =>\n        high\n    => close\n");

    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_PARSE_SWITCH_BLOCK"),
        "{:?}",
        parsed.diagnostics
    );
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
