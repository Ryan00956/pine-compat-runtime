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
p = Point.new(y = 1)
plot(p.z)
"#,
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_CONSTRUCTOR_ARG" })
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
fn rejects_user_type_field_mutation() {
    let analysis = analyze(
        r#"type Point
    float x
p = Point.new(close)
p.x := 1
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| { feature.feature == "user-defined type field mutation" })
    );
    assert!(analysis.hir.is_none());
}
