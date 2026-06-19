use std::collections::HashMap;

use pine_ir::HirProgram;
use pine_syntax::{Diagnostic, SourceFile};

use crate::analyzer::context::Analyzer;
use crate::compatibility::CompatibilityReport;
use crate::modules::validate_modules;
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
    let module_validation = validate_modules(input);
    let mut analyzer = Analyzer {
        diagnostics: module_validation.diagnostics,
        compatibility: CompatibilityReport {
            language_version: module_validation
                .root_program
                .version
                .map(|version| version.version),
            ..CompatibilityReport::default()
        },
        scope: ScopeResolver::new(initial_symbols(), initial_symbol_order()),
        bindings: HashMap::new(),
        lower_symbol_overrides: Vec::new(),
        functions: module_validation.imported_functions,
        methods: HashMap::new(),
        user_types: HashMap::new(),
        symbol_user_types: HashMap::new(),
        expr_user_types: HashMap::new(),
        expr_types: HashMap::new(),
        script_declaration: None,
        strategy_settings: Default::default(),
        drawing_settings: Default::default(),
        function_stack: Vec::new(),
        next_symbol_id: initial_symbol_count(),
        next_series_id: initial_series_count(),
        next_call_site_id: 0,
        next_var_slot_id: 0,
        block_depth: 0,
        function_depth: 0,
        loop_depth: 0,
        expr_depth: 0,
        lowering_limits: Default::default(),
        lowering_inline_depth: 0,
        lowered_hir_nodes: 0,
        lowered_temp_symbols: 0,
        lowering_budget_reported: false,
    };
    analyzer.analyze_program(&module_validation.root_program);
    analyzer.finish(&module_validation.root_program)
}
