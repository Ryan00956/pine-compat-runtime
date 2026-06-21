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
fn rejects_user_type_array_typed_declarations() {
    let analysis = analyze(
        r#"type Point
    float x
array<Point> points = na
plot(close)
"#,
    );

    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_DECL_TYPE" && diagnostic.message.contains("array<Point>")
        }),
        "{:?}",
        analysis.diagnostics
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
fn accepts_user_type_typed_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
Point p = na
p := Point.new(close, open)
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
fn accepts_user_type_typed_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
Point p = Point.new(close, open)
p := Point.new(high, low)
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
fn accepts_user_type_typed_declaration_with_ternary_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
Point p = bar_index < 2 ? Point.new(close, open) : Point.new(high, low)
p := bar_index == 3 ? Point.new(high, low) : Point.new(close, open)
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
fn accepts_user_type_typed_declaration_with_switch_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
Point p = switch bar_index
    0 => Point.new(close, open)
    1 => Point.new(high, low)
    => Point.new(close + 1, open)
p := switch bar_index
    3 => Point.new(high, low)
    => Point.new(close, open)
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
fn accepts_var_user_type_typed_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
var Point p = na
if bar_index == 0
    p := Point.new(close, open)
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
fn accepts_var_user_type_typed_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
var Point p = Point.new(close, open)
if bar_index == 2
    p := Point.new(high, low)
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
fn accepts_block_local_user_type_typed_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
if close > open
    Point p = na
    p := Point.new(close, open)
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
fn accepts_block_local_user_type_typed_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
if close > open
    Point p = Point.new(close, open)
    p := Point.new(high, low)
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
fn accepts_block_local_user_type_typed_declaration_with_ternary_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
if close > open
    Point p = bar_index < 2 ? Point.new(close, open) : Point.new(high, low)
    p := bar_index == 3 ? Point.new(high, low) : Point.new(close, open)
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
fn accepts_loop_local_user_type_typed_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 1
    Point p = na
    p := Point.new(close + i, open)
    sum := sum + p.x + p.y
while sum > 0
    Point p = na
    p := Point.new(close, open)
    sum := sum - p.x - p.y
plot(sum)
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
fn accepts_loop_local_user_type_typed_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 1
    Point p = Point.new(close + i, open)
    p := Point.new(high + i, low)
    sum := sum + p.x + p.y
while sum > 0
    Point p = Point.new(close, open)
    p := Point.new(high, low)
    sum := sum - p.x - p.y
plot(sum)
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
fn accepts_udf_local_user_type_typed_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
makeTyped(x, y) =>
    Point p = na
    p := Point.new(x, y)
    p
made = makeTyped(close, open)
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
fn accepts_udf_local_user_type_typed_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
makeTyped(x, y) =>
    Point p = Point.new(x, y)
    p := Point.new(x + 1, y)
    p
made = makeTyped(close, open)
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
fn rejects_mismatched_user_type_typed_declaration_reassignment() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
Point p = na
p := Other.new(close)
plot(close)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UDT_ASSIGN_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_mismatched_user_type_typed_declaration_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
Point p = Other.new(close)
plot(close)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UDT_ASSIGN_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_nested_user_type_fields_and_field_reads() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
type Wrapper
    Point inner
p = Point.new(close, open)
w = Wrapper.new(p)
plot(w.inner.x + w.inner.y)
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
fn accepts_typed_nested_user_type_declaration_with_constructor_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
type Wrapper
    Point inner
    float weight
p = Point.new(close, open)
Wrapper w = Wrapper.new(p, high)
w := Wrapper.new(Point.new(high, low), close)
plot(w.inner.x + w.inner.y + w.weight)
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
fn accepts_typed_nested_user_type_declaration_with_na_initializer() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
type Wrapper
    Point inner
    float weight
Wrapper w = na
w := Wrapper.new(Point.new(high, low), close)
plot(w.inner.x + w.inner.y + w.weight)
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
fn accepts_nested_user_type_field_replacement() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
type Wrapper
    Point inner
p = Point.new(close, open)
w = Wrapper.new(p)
w.inner := Point.new(high, low)
plot(w.inner.x + w.inner.y)
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
fn accepts_nested_user_type_field_replacement_in_control_flow() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
type Wrapper
    Point inner
p = Point.new(close, open)
w = Wrapper.new(p)
if bar_index < 2
    w.inner := Point.new(high, low)
else
    w.inner := Point.new(close, open)
for i = 0 to 1
    w.inner := Point.new(w.inner.x + i, w.inner.y)
while_i = 0
while while_i < 1
    w.inner := Point.new(w.inner.x, w.inner.y + 1)
    while_i := while_i + 1
plot(w.inner.x + w.inner.y)
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
fn rejects_mismatched_nested_user_type_field_replacement() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
type Wrapper
    Point inner
p = Point.new(close)
w = Wrapper.new(p)
w.inner := Other.new(high)
plot(w.inner.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_ASSIGN_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_nested_user_type_field_mutation() {
    let analysis = analyze(
        r#"type Point
    float x
type Wrapper
    Point inner
p = Point.new(close)
w = Wrapper.new(p)
w.inner.x := high
plot(w.inner.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UNSUPPORTED_FEATURE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "nested field mutation"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_mismatched_nested_user_type_field_constructors() {
    let analysis = analyze(
        r#"type Point
    float x
type Other
    float x
type Wrapper
    Point inner
w = Wrapper.new(Other.new(close))
plot(close)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UDT_CONSTRUCTOR_ARG"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
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
fn accepts_user_type_final_if_branch_alias_constructor_return_from_udf_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, flip) =>
    if flip
        ax = p.x + 1
        Point.new(ax, p.y)
    else
        ax = p.x + 10
        Point.new(ax, p.y)
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
fn accepts_user_type_final_if_branch_udt_alias_return_from_udf_udt_param() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, flip) =>
    if flip
        q = p
        q
    else
        q = p
        q
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
fn accepts_user_type_final_for_udt_alias_return_from_udf_udt_param() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
cloneFrom(p, count) =>
    for i = 0 to count
        q = p
        q
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
