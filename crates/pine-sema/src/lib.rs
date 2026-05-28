//! Semantic analysis and compatibility gating scaffolding.

mod analysis;
mod analyzer;
mod cache;
mod compatibility;
mod history;
mod lowering;
mod modules;
mod resolver;
mod source_graph;
mod symbols;
mod types;

mod prelude {
    pub(crate) use std::collections::HashMap;

    pub(crate) use pine_builtins::{Accepts, BuiltinSignature, ReturnSpec};
    pub(crate) use pine_ir::{
        HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirLiteral, HirProgram,
        HirStmt, HirStmtKind, HirSwitchArm, HirSymbol, HirUnaryOp, PersistenceKind, PineType,
        Qualifier, ScriptMode, SymbolId, ValueKind,
    };
    pub(crate) use pine_syntax::{
        BinaryOp, CallArg, Diagnostic, Expr, ExprKind, FunctionBody, Literal, Program, Severity,
        Span, Stmt, StmtKind, SwitchArm, UnaryOp,
    };

    pub(crate) use crate::analyzer::calls::{
        array_method_builtin_name, expr_name, is_array_mutation_builtin,
        is_array_mutation_method_call_name, is_output_or_declaration_builtin,
        is_ta_vwap_bands_call, method_call_parts, receiver_call_arg,
    };
    pub(crate) use crate::analyzer::context::{
        Analyzer, FunctionInfo, MethodInfo, MethodParamInfo, MethodResolution, UdfArgError,
    };
    pub(crate) use crate::analyzer::functions::resolve_udf_arg_indices;
    pub(crate) use crate::analyzer::strategy::is_phase_l_strategy_state_variable;
    pub(crate) use crate::analyzer::unsupported::{
        STRATEGY_STATE_UNSUPPORTED_REASON, VARIP_DRAWING_UNSUPPORTED_REASON,
        VARIP_VALUE_UNSUPPORTED_REASON, unsupported_strategy_reason, unsupported_syntax_reason,
    };
    pub(crate) use crate::analyzer::user_types::{UserTypeInfo, span_key};
    pub(crate) use crate::compatibility::{FeatureUse, UnsupportedFeature};
    pub(crate) use crate::history::{infer_history_requirements, infer_max_bars_back};
    pub(crate) use crate::resolver::{SymbolInfo, binding_key};
    pub(crate) use crate::symbols::INITIAL_SYMBOLS;
    pub(crate) use crate::types::{
        UNKNOWN, accepts_type, array_element_return_type, array_from_return_type,
        array_numeric_return_type, can_assign, common_kind, const_int_value, const_numeric_value,
        const_string_value, float_return_for_arg, int_return_for_arg, is_array_kind, is_numeric,
        literal_type, merge_result_types, numeric_result_kind, promoted_bool_type,
        promoted_color_type, promoted_float_type, promoted_int_type, promoted_numeric_type,
        promoted_string_type, round_return_type, series_return_for_arg, strongest_qualifier,
    };
}

pub use analysis::{Analysis, analyze_input, analyze_source};
pub use cache::{CompileCache, CompileCacheStats};
pub use compatibility::{CompatibilityReport, FeatureUse, UnsupportedFeature};
pub use source_graph::{
    AnalysisInput, LibrarySource, SourceGraph, SourceGraphError, SourceId, SourceUnit,
};

#[cfg(test)]
mod tests;
