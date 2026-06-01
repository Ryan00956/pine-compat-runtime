use std::{fs, path::PathBuf};

use pine_syntax::{LineCol, SourceFile, parse_source};

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn parses_phase_1_basic_fixture() {
    let (source, parsed) = parse_fixture("tests/fixtures/syntax/phase1_basic.pine");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        parsed.program.version.map(|version| version.version),
        Some(5)
    );
    assert_eq!(parsed.program.statements.len(), 6);
    assert_eq!(
        source.name(),
        workspace_fixture("tests/fixtures/syntax/phase1_basic.pine")
            .display()
            .to_string()
    );
}

#[test]
fn parses_soft_keyword_export_identifier_fixture() {
    let (_, parsed) = parse_fixture("tests/fixtures/syntax/soft_keyword_export_identifier.pine");

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.statements.len(), 3);
}

#[test]
fn recovers_after_parse_error_fixture() {
    let (_, parsed) = parse_fixture("tests/fixtures/syntax/parse_error_recovery.pine");

    assert!(has_diagnostic(&parsed.diagnostics, "E_PARSE_EXPR"));
    assert_eq!(parsed.program.statements.len(), 3);
}

#[test]
fn reports_malformed_number_fixture_and_recovers() {
    let (_, parsed) = parse_fixture("tests/fixtures/syntax/malformed_number_recovery.pine");

    assert!(has_diagnostic(&parsed.diagnostics, "E_LEX_INT"));
    assert_eq!(parsed.program.statements.len(), 3);
}

#[test]
fn reports_utf8_diagnostic_columns_in_characters() {
    let (source, parsed) = parse_fixture("tests/fixtures/syntax/utf8_diagnostic_column.pine");
    let diagnostic = parsed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E_LEX_CHAR")
        .expect("unexpected-character diagnostic");

    assert_eq!(
        source.line_col(diagnostic.span.start),
        LineCol { line: 3, column: 9 }
    );
}

#[test]
fn rejects_deep_expression_limit_fixture() {
    let (_, parsed) = parse_fixture("tests/fixtures/syntax/deep_expression_limit.pine");

    assert!(has_diagnostic(&parsed.diagnostics, "E_PARSE_EXPR_DEPTH"));
}

fn parse_fixture(path: &str) -> (SourceFile, pine_syntax::Parse) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let parsed = parse_source(&source);
    (source, parsed)
}

fn has_diagnostic(diagnostics: &[pine_syntax::Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.code == code)
}
