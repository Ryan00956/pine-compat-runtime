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
