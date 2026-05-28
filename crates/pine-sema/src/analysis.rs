use std::collections::HashMap;

use pine_ir::HirProgram;
use pine_syntax::{Diagnostic, SourceFile, parse_source};

use crate::analyzer::context::Analyzer;
use crate::compatibility::CompatibilityReport;
use crate::resolver::ScopeResolver;
use crate::source_graph::AnalysisInput;
use crate::symbols::{
    initial_series_count, initial_symbol_count, initial_symbol_order, initial_symbols,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
    pub compatibility: CompatibilityReport,
    pub hir: Option<HirProgram>,
}

pub fn analyze_source(source: &SourceFile) -> Analysis {
    analyze_input(&AnalysisInput::new(source.clone()))
}

pub fn analyze_input(input: &AnalysisInput) -> Analysis {
    let graph = input.source_graph();
    let source = graph.root().source();
    let parsed = parse_source(source);
    let mut analyzer = Analyzer {
        diagnostics: parsed.diagnostics,
        compatibility: CompatibilityReport {
            language_version: parsed.program.version.map(|version| version.version),
            ..CompatibilityReport::default()
        },
        scope: ScopeResolver::new(initial_symbols(), initial_symbol_order()),
        bindings: HashMap::new(),
        lower_symbol_overrides: Vec::new(),
        functions: HashMap::new(),
        function_stack: Vec::new(),
        next_symbol_id: initial_symbol_count(),
        next_series_id: initial_series_count(),
        next_call_site_id: 0,
        next_var_slot_id: 0,
        block_depth: 0,
        function_depth: 0,
        loop_depth: 0,
    };
    analyzer.analyze_program(&parsed.program);
    analyzer.finish(&parsed.program)
}
