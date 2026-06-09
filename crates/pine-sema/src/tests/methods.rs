use super::*;

#[test]
fn accepts_pure_user_method_on_local_udt() {
    let analysis = analyze(
        r#"type Point
    float x
method shift(Point p, float delta) => p.x + delta
p = Point.new(close)
plot(p.shift(2))
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
            .any(|feature| feature.feature == "user-defined methods")
    );
}

#[test]
fn accepts_udt_receiver_passthrough_user_method_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method keep(Point p) => p
p = Point.new(close, open)
same = p.keep()
plot(same.x + same.y)
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
fn accepts_udt_receiver_block_alias_user_method_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method keepBlock(Point p) =>
    copy = p
    copy
p = Point.new(close, open)
same = p.keepBlock()
plot(same.x + same.y)
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
fn accepts_udt_parameter_passthrough_user_method_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method choose(Point p, Point other) => other
p = Point.new(close, open)
q = Point.new(open, close)
same = p.choose(q)
plot(same.x + same.y)
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
fn accepts_udt_parameter_block_alias_user_method_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method chooseBlock(Point p, Point other) =>
    copy = other
    copy
p = Point.new(close, open)
q = Point.new(open, close)
same = p.chooseBlock(q)
plot(same.x + same.y)
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
fn accepts_nested_udt_parameter_passthrough_user_method_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method choose(Point p, Point other) => other
method wrap(Point p, Point other) => p.choose(other)
p = Point.new(close, open)
q = Point.new(open, close)
same = p.wrap(q)
plot(same.x + same.y)
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
fn accepts_udt_constructor_return_from_user_method() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method clone(Point p) => Point.new(p.x, p.y)
p = Point.new(close, open)
copy = p.clone()
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
fn accepts_udt_constructor_return_from_user_method_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method cloneOther(Point p, Point other) => Point.new(other.x, other.y)
p = Point.new(close, open)
q = Point.new(open, close)
copy = p.cloneOther(q)
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
fn accepts_udt_named_constructor_return_from_user_method_udt_param_fields() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method cloneOtherNamed(Point p, Point other) => Point.new(y=other.y, x=other.x)
p = Point.new(close, open)
q = Point.new(open, close)
copy = p.cloneOtherNamed(q)
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
fn accepts_udt_constructor_return_from_user_method_scalar_param() {
    let analysis = analyze(
        r#"type Point
    float x
method make(Point p, float x) => Point.new(x)
p = Point.new(close)
made = p.make(open)
plot(made.x + close)
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
fn accepts_udt_constructor_return_from_user_method_scalar_alias() {
    let analysis = analyze(
        r#"type Point
    float x
method makeBlock(Point p, float x) =>
    ax = x
    Point.new(ax)
p = Point.new(close)
made = p.makeBlock(open)
plot(made.x + close)
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
fn accepts_udt_named_constructor_return_from_user_method() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
method make(Point p, float x, float y) => Point.new(y=y, x=x)
p = Point.new(close, open)
made = p.make(close, open)
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
fn accepts_udt_named_constructor_return_from_user_method_scalar_alias() {
    let analysis = analyze(
        r#"type Point
    float x
method makeNamedBlock(Point p, float x) =>
    ax = x
    Point.new(x=ax)
p = Point.new(close)
made = p.makeNamedBlock(open)
plot(made.x + close)
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
fn accepts_udt_passthrough_user_function() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
identity(p) => p
p = Point.new(close, open)
same = identity(p)
plot(same.x - same.y)
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
fn accepts_udt_block_body_user_function_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
wrap(p) =>
    copy = p
    copy
p = Point.new(close, open)
wrapped = wrap(p)
plot(wrapped.x + wrapped.y)
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
fn accepts_udt_nested_passthrough_user_function_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
inner(p) => p
outer(p) => inner(p)
p = Point.new(close, open)
nested = outer(p)
plot(nested.x + nested.y)
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
fn accepts_udt_named_arg_passthrough_user_function_return() {
    let analysis = analyze(
        r#"type Point
    float x
    float y
choose(delta, p) => p
p = Point.new(close, open)
chosen = choose(p=p, delta=1)
plot(chosen.x + chosen.y)
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
fn rejects_unknown_user_method() {
    let analysis = analyze(
        r#"type Point
    float x
p = Point.new(close)
plot(p.missing())
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UNKNOWN_METHOD")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_wrong_udt_user_method_parameter() {
    let analysis = analyze(
        r#"type Point
    float x
type LabelInfo
    float x
method choose(Point p, Point other) => other
p = Point.new(close)
info = LabelInfo.new(open)
chosen = p.choose(info)
plot(chosen.x)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_ARG_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_duplicate_user_methods() {
    let analysis = analyze(
        r#"type Point
    float x
method shift(Point p) => p.x
method shift(Point p) => p.x + 1
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_DUPLICATE")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_side_effecting_user_methods() {
    let analysis = analyze(
        r#"type Point
    float x
method draw(Point p) => plot(p.x)
p = Point.new(close)
p.draw()
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| { feature.feature == "function_side_effect" })
    );
    assert!(analysis.hir.is_none());
}
