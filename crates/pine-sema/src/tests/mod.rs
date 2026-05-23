use crate::{Analysis, CompileCache, CompileCacheStats, analyze_source};
use pine_ir::{HirStmtKind, PersistenceKind, VarSlotId};
use pine_syntax::SourceFile;

fn analyze(text: &str) -> Analysis {
    analyze_source(&SourceFile::new("test.pine", text))
}

mod compatibility;
mod lowering;
mod scopes;
mod type_arrays;
mod type_core;
mod type_inputs_outputs;
mod type_ta;
