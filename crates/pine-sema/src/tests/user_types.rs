use super::*;

#[test]
fn accepts_local_scalar_user_type_construction_and_field_reads() {
    let analysis = analyze(
        r#"indicator("udt")
type Point
    float x
    int y
p = Point.new(close, 2)
plot(p.x + p.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "user-defined types")
    );
}

#[test]
fn rejects_duplicate_user_type_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    int x
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_FIELD_DUPLICATE" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_unknown_user_type_fields_and_constructor_fields() {
    let analysis = analyze(
        r#"type Point
    float x
ok = Point.new(1)
p = Point.new(y = 1)
plot(ok.z)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_CONSTRUCTOR_ARG" })
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_UNKNOWN_FIELD" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_user_type_history_references() {
    let analysis = analyze(
        r#"type Point
    float x
p = Point.new(close)
prior = p[1]
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| { feature.feature == "user-defined type history" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_block_local_user_type_declarations() {
    let analysis = analyze(
        r#"if close > open
    type Point
        float x
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_DECL_LOCATION" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_varip_user_type_declarations() {
    let analysis = analyze(
        r#"type Point
    float x
varip p = Point.new(close)
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| { feature.feature == "varip" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_user_type_field_mutation() {
    let analysis = analyze(
        r#"type Point
    float x
p = Point.new(close)
p.x := 1
plot(p.x)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| { feature.feature == "user-defined type field mutation" })
    );
}

#[test]
fn accepts_user_type_constructor_return_from_user_function() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
make(x, y) => Point.new(x, y)
p = make(close, open)
plot(p.x + p.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) => Point.new(p.x, p.y)
p = Point.new(close, open)
copy = cloneFrom(p)
plot(copy.x + copy.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) => Point.new(y=p.y, x=p.x)
p = Point.new(close, open)
copy = cloneFrom(p)
plot(copy.x + copy.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_constructor_return_from_udf_udt_param_field_aliases() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) =>
    ax = p.x
    ay = p.y
    Point.new(ax, ay)
p = Point.new(close, open)
copy = cloneFrom(p)
plot(copy.x + copy.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_constructor_return_from_udf_udt_param_field_aliases() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) =>
    ax = p.x
    ay = p.y
    Point.new(y=ay, x=ax)
p = Point.new(close, open)
copy = cloneFrom(p)
plot(copy.x + copy.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_constructor_return_from_udf_udt_block_alias_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) =>
    copy = p
    Point.new(copy.x, copy.y)
p = Point.new(close, open)
made = cloneFrom(p)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_constructor_return_from_udf_udt_block_alias_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p) =>
    copy = p
    Point.new(y=copy.y, x=copy.x)
p = Point.new(close, open)
made = cloneFrom(p)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_nested_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
make(x, y) => Point.new(x, y)
cloneFrom(p) => make(p.x, p.y)
p = Point.new(close, open)
made = cloneFrom(p)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_nested_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
makeNamed(x, y) => Point.new(y=y, x=x)
cloneFrom(p) => makeNamed(p.x, p.y)
p = Point.new(close, open)
made = cloneFrom(p)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_ternary_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, flip) => flip ? Point.new(p.x, p.y) : Point.new(p.x + 10, p.y)
p = Point.new(close, open)
made = cloneFrom(p, bar_index < 2)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_switch_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, mode) =>
    switch mode
        0 => Point.new(p.x, p.y)
        => Point.new(p.x + 10, p.y)
p = Point.new(close, open)
made = cloneFrom(p, bar_index)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_final_if_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, flip) =>
    if flip
        Point.new(p.x, p.y)
    else
        Point.new(p.x + 10, p.y)
p = Point.new(close, open)
made = cloneFrom(p, bar_index < 2)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_final_for_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, count) =>
    for i = 0 to count
        Point.new(p.x + i, p.y)
p = Point.new(close, open)
made = cloneFrom(p, 2)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_for_expression_constructor_assignment() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
p = Point.new(close, open)
made = for i = 0 to 1
    Point.new(p.x + i, p.y)
plot(made.x + made.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn rejects_mismatched_user_type_ternary_constructor_branches() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
value = close > open ? Point.new(close) : Other.new(open)
plot(value.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_BRANCH_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_mismatched_user_type_switch_constructor_arms() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
value = switch bar_index
    0 => Point.new(close)
    => Other.new(open)
plot(value.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_BRANCH_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_mismatched_user_type_final_if_constructor_branches() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
choose(flip) =>
    if flip
        Point.new(close)
    else
        Other.new(open)
value = choose(close > open)
plot(value.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_BRANCH_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_user_type_constructor_return_from_user_function_scalar_alias() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
make(x, y) =>
    ax = x
    ay = y
    Point.new(ax, ay)
p = make(close, open)
plot(p.x + p.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_constructor_return_from_user_function() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
make(x, y) => Point.new(y=y, x=x)
p = make(close, open)
plot(p.x + p.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_user_type_named_constructor_return_from_user_function_scalar_alias() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
make(x, y) =>
    ax = x
    ay = y
    Point.new(y=ay, x=ax)
p = make(close, open)
plot(p.x + p.y)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn rejects_user_type_field_mutation_inside_function() {
    let analysis = analyze(
        r#"type Point
    float x
p = Point.new(close)
touch() =>
    p.x := 1
    p.x
plot(touch())
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| { feature.feature == "function_side_effect" }),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}
