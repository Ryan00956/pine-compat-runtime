use std::fs;

use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use crate::usage;

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let [path] = args.as_slice() else {
        return Err(usage());
    };
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    let source = SourceFile::new(path, text);
    let analysis = analyze_source(&source);
    println!("diagnostics: {}", analysis.diagnostics.len());
    println!(
        "supported: {}, unsupported: {}",
        analysis.compatibility.supported.len(),
        analysis.compatibility.unsupported.len()
    );
    for diagnostic in analysis.diagnostics {
        let line_col = source.line_col(diagnostic.span.start);
        println!(
            "{}:{:?}:{}:{}: {}",
            diagnostic.code,
            diagnostic.severity,
            line_col.line,
            line_col.column,
            diagnostic.message
        );
    }
    Ok(())
}
