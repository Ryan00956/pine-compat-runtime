use std::{fs, path::PathBuf};

use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn reports_unsupported_request_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_request.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn reports_unsupported_varip_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_varip.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("intrabar persistence")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_strategy_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy.pine",
        "strategy.close",
        "broker emulation",
    );
}

#[test]
fn reports_unsupported_drawing_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_drawing.pine",
        "label.new",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_array_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array.pine",
        "array.new_float",
        "array storage",
    );
}

#[test]
fn reports_unsupported_import_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_import.pine",
        "import",
        "library imports",
    );
}

#[test]
fn reports_unsupported_alert_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert.pine",
        "alert",
        "alerts",
    );
}

#[test]
fn reports_unsupported_block_local_declaration_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_block_local_decl.pine",
        "block_local_declaration",
        "declarations inside if blocks",
    );
}

#[test]
fn reports_unsupported_recursive_function_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_recursive_function.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_RECURSIVE_FUNCTION"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_function_block_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_function_block.pine",
        "function_block",
        "multi-statement user-defined functions",
    );
}

fn assert_unsupported_fixture(path: &str, feature: &str, reason: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.compatibility.unsupported.iter().any(
            |unsupported| unsupported.feature == feature && unsupported.reason.contains(reason)
        ),
        "{} unsupported features: {:?}",
        path.display(),
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}
