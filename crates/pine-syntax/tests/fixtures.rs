use std::{fs, path::PathBuf};

use pine_syntax::{SourceFile, parse_source};

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn parses_phase_1_basic_fixture() {
    let path = workspace_fixture("tests/fixtures/syntax/phase1_basic.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let parsed = parse_source(&source);

    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        parsed.program.version.map(|version| version.version),
        Some(5)
    );
    assert_eq!(parsed.program.statements.len(), 4);
}
