use std::collections::HashMap;

use pine_ir::{
    CallSiteId, DrawingSettings, PersistenceKind, PineType, Qualifier, ScriptMode, SeriesId,
    StrategySettings, SymbolId, VarSlotId,
};
use pine_syntax::{Diagnostic, FunctionBody, Program, Severity, Span};

use crate::analysis::Analysis;
use crate::compatibility::CompatibilityReport;
use crate::prelude::UserTypeInfo;
use crate::resolver::{BindingKey, ScopeResolver, SymbolInfo};

pub(crate) const MAX_SEMA_EXPR_DEPTH: u32 = 128;
pub(crate) const MAX_FUNCTION_CALL_DEPTH: usize = 64;
pub(crate) const MAX_LOWERING_INLINE_DEPTH: u32 = 64;
pub(crate) const MAX_LOWERING_HIR_NODES: u32 = 65_536;
pub(crate) const MAX_LOWERING_TEMP_SYMBOLS: u32 = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoweringLimits {
    pub(crate) max_inline_depth: u32,
    pub(crate) max_hir_nodes: u32,
    pub(crate) max_temp_symbols: u32,
}

impl Default for LoweringLimits {
    fn default() -> Self {
        Self {
            max_inline_depth: MAX_LOWERING_INLINE_DEPTH,
            max_hir_nodes: MAX_LOWERING_HIR_NODES,
            max_temp_symbols: MAX_LOWERING_TEMP_SYMBOLS,
        }
    }
}

pub(crate) struct Analyzer {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) compatibility: CompatibilityReport,
    pub(crate) scope: ScopeResolver,
    pub(crate) bindings: HashMap<BindingKey, SymbolInfo>,
    pub(crate) lower_symbol_overrides: Vec<HashMap<SymbolId, SymbolInfo>>,
    pub(crate) functions: HashMap<String, FunctionInfo>,
    pub(crate) methods: HashMap<(String, String), MethodInfo>,
    pub(crate) user_types: HashMap<String, UserTypeInfo>,
    pub(crate) symbol_user_types: HashMap<SymbolId, String>,
    pub(crate) expr_user_types: HashMap<(usize, usize), String>,
    pub(crate) expr_types: HashMap<(usize, usize), PineType>,
    pub(crate) script_declaration: Option<(ScriptMode, Span)>,
    pub(crate) strategy_settings: StrategySettings,
    pub(crate) drawing_settings: DrawingSettings,
    pub(crate) function_stack: Vec<String>,
    pub(crate) next_symbol_id: u32,
    pub(crate) next_series_id: u32,
    pub(crate) next_call_site_id: u32,
    pub(crate) next_var_slot_id: u32,
    pub(crate) block_depth: u32,
    pub(crate) function_depth: u32,
    pub(crate) loop_depth: u32,
    pub(crate) expr_depth: u32,
    pub(crate) lowering_limits: LoweringLimits,
    pub(crate) lowering_inline_depth: u32,
    pub(crate) lowered_hir_nodes: u32,
    pub(crate) lowered_temp_symbols: u32,
    pub(crate) lowering_budget_reported: bool,
}
#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub(crate) params: Vec<String>,
    pub(crate) body: FunctionBody,
    pub(crate) span: Span,
}
#[derive(Debug, Clone)]
pub(crate) struct MethodInfo {
    pub(crate) receiver_type: String,
    pub(crate) receiver_name: String,
    pub(crate) params: Vec<MethodParamInfo>,
    pub(crate) body: FunctionBody,
    pub(crate) span: Span,
}
#[derive(Debug, Clone)]
pub(crate) struct MethodParamInfo {
    pub(crate) name: String,
    pub(crate) pine_type: PineType,
    pub(crate) user_type_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UdfArgError {
    UnknownName { name: String, span: Span },
    Duplicate { name: String, span: Span },
    PositionalAfterNamed { span: Span },
    TooMany { span: Span },
    Missing { param: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodResolution {
    NotMethod,
    Resolved(Option<PineType>),
}

impl Analyzer {
    pub(crate) fn define_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
    ) -> SymbolInfo {
        self.define_symbol_with_persistence(
            name,
            pine_type,
            persistence_kind_for_slot(var_slot_id),
            var_slot_id,
        )
    }

    pub(crate) fn define_symbol_with_persistence(
        &mut self,
        name: &str,
        pine_type: PineType,
        persistence: PersistenceKind,
        var_slot_id: Option<VarSlotId>,
    ) -> SymbolInfo {
        if let Some(existing) = self.scope.resolve(name) {
            let persistence = if persistence != PersistenceKind::None {
                persistence
            } else {
                existing.persistence
            };
            let updated = SymbolInfo {
                pine_type,
                series_id: existing
                    .series_id
                    .or_else(|| self.series_id_for_type(pine_type)),
                persistence,
                var_slot_id: existing.var_slot_id.or(var_slot_id),
                ..existing
            };
            self.scope.update(name, updated);
            return updated;
        }

        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            persistence,
            var_slot_id,
        };
        self.scope.define_global(name, info);
        info
    }

    pub(crate) fn define_local_symbol(
        &mut self,
        name: &str,
        pine_type: PineType,
        var_slot_id: Option<VarSlotId>,
        lower: bool,
    ) -> SymbolInfo {
        self.define_local_symbol_with_persistence(
            name,
            pine_type,
            persistence_kind_for_slot(var_slot_id),
            var_slot_id,
            lower,
        )
    }

    pub(crate) fn define_local_symbol_with_persistence(
        &mut self,
        name: &str,
        pine_type: PineType,
        persistence: PersistenceKind,
        var_slot_id: Option<VarSlotId>,
        lower: bool,
    ) -> SymbolInfo {
        let series_id = self.series_id_for_type(pine_type);
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id,
            persistence,
            var_slot_id,
        };
        self.scope.define_local(name, info, lower);
        info
    }

    pub(crate) fn fresh_lower_symbol(&mut self, name: &str, original: SymbolInfo) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type: original.pine_type,
            series_id: self.series_id_for_type(original.pine_type),
            persistence: original.persistence,
            var_slot_id: original.var_slot_id.map(|_| self.alloc_var_slot()),
        };
        self.scope.add_lower_symbol(name, info);
        info
    }

    pub(crate) fn fresh_temp_symbol(&mut self, name: &str, pine_type: PineType) -> SymbolInfo {
        let info = SymbolInfo {
            id: self.alloc_symbol(),
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            persistence: PersistenceKind::None,
            var_slot_id: None,
        };
        self.scope.add_lower_symbol(name, info);
        info
    }

    pub(crate) fn update_symbol_type(&mut self, name: &str, pine_type: PineType) {
        if let Some(mut symbol) = self.scope.resolve(name) {
            symbol.pine_type = pine_type;
            if symbol.series_id.is_none() {
                symbol.series_id = self.series_id_for_type(pine_type);
            }
            self.scope.update(name, symbol);
        }
    }

    pub(crate) fn series_id_for_type(&mut self, pine_type: PineType) -> Option<SeriesId> {
        if pine_type.qualifier == Qualifier::Series {
            Some(self.alloc_series())
        } else {
            None
        }
    }

    pub(crate) fn alloc_symbol(&mut self) -> SymbolId {
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    pub(crate) fn alloc_series(&mut self) -> SeriesId {
        let id = SeriesId(self.next_series_id);
        self.next_series_id += 1;
        id
    }

    pub(crate) fn alloc_call_site(&mut self) -> CallSiteId {
        let id = CallSiteId(self.next_call_site_id);
        self.next_call_site_id += 1;
        id
    }

    pub(crate) fn alloc_var_slot(&mut self) -> VarSlotId {
        let id = VarSlotId(self.next_var_slot_id);
        self.next_var_slot_id += 1;
        id
    }

    pub(crate) fn finish(mut self, program: &Program) -> Analysis {
        let hir = if self.has_errors() {
            None
        } else {
            self.lower_program(program)
        };

        Analysis {
            diagnostics: self.diagnostics,
            compatibility: self.compatibility,
            hir,
        }
    }

    pub(crate) fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

fn persistence_kind_for_slot(var_slot_id: Option<VarSlotId>) -> PersistenceKind {
    if var_slot_id.is_some() {
        PersistenceKind::Var
    } else {
        PersistenceKind::None
    }
}
