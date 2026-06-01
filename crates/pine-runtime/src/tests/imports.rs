use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::SourceFile;

use super::*;

fn analyze_import(root: &str, library: &str) -> pine_sema::Analysis {
    let input = AnalysisInput::with_library_sources(
        SourceFile::new("root.pine", root),
        vec![(
            "user/lib/1".to_owned(),
            SourceFile::new("lib.pine", library),
        )],
    )
    .expect("analysis input");
    analyze_input(&input)
}

#[test]
fn runs_imported_constants_and_pure_functions() {
    let analysis = analyze_import(
        r#"indicator("imports")
import user/lib/1 as lib
plot(lib.scale(close) + lib.offset)
"#,
        r#"library("lib")
export offset = 2
export scale(value) => value * offset
"#,
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
}

#[test]
fn imported_function_locals_shadow_exported_constants() {
    let analysis = analyze_import(
        r#"indicator("import shadow")
import user/lib/1 as lib
plot(lib.scale(close))
"#,
        r#"library("lib")
export offset = 2
export scale(value) =>
    offset = 5
    value + offset
"#,
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[6.0, 7.0, 8.0]);
}

#[test]
fn imported_function_callsite_state_is_independent() {
    let analysis = analyze_import(
        r#"indicator("imported state")
import user/lib/1 as lib
plot(lib.counter() + lib.counter())
"#,
        r#"library("lib")
export counter() =>
    var value = 0
    value := value + 1
    value
"#,
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}

#[test]
fn branch_skipped_imported_calls_do_not_advance_state() {
    let analysis = analyze_import(
        r#"indicator("imported branch state")
import user/lib/1 as lib
out = 0
if close >= 2
    out := lib.counter()
plot(out)
"#,
        r#"library("lib")
export counter() =>
    var value = 0
    value := value + 1
    value
"#,
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(
        &analysis.hir.expect("HIR"),
        &[bar(1.0), bar(2.0), bar(3.0), bar(4.0)],
    )
    .expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[0.0, 1.0, 2.0, 3.0]);
}

#[test]
fn repeated_imports_of_same_source_share_exports_without_state_collision() {
    let input = AnalysisInput::with_library_sources(
        SourceFile::new(
            "root.pine",
            r#"indicator("repeated imports")
import user/lib/1 as one
import user/lib/1 as two
plot(one.counter() + two.counter())
"#,
        ),
        vec![(
            "user/lib/1".to_owned(),
            SourceFile::new(
                "lib.pine",
                r#"library("lib")
export counter() =>
    var value = 0
    value := value + 1
    value
"#,
            ),
        )],
    )
    .expect("analysis input");
    let analysis = analyze_input(&input);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}
