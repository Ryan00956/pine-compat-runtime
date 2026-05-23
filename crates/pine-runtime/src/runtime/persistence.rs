use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_decl(
        &mut self,
        symbol: SymbolId,
        value: &HirExpr,
    ) -> Result<PineValue, RuntimeError> {
        let Some((_persistence, var_slot_id)) = self.persistent_slot_for_symbol(symbol) else {
            return self.eval_expr(value);
        };

        if let Some(value) = self.var_store.get(&var_slot_id).cloned() {
            Ok(value)
        } else {
            let value = self.eval_expr(value)?;
            self.var_store.insert(var_slot_id, value.clone());
            Ok(value)
        }
    }

    pub(crate) fn assign_persistent_symbol(&mut self, symbol: SymbolId, value: PineValue) {
        if let Some((_persistence, var_slot_id)) = self.persistent_slot_for_symbol(symbol) {
            self.var_store.insert(var_slot_id, value);
        }
    }

    pub(crate) fn seed_intrabar_persistence_from(&mut self, previous: &Self) {
        for var_slot_id in self.persistent_slots_for_kind(PersistenceKind::Varip) {
            if let Some(value) = previous.var_store.get(&var_slot_id).cloned() {
                self.var_store.insert(var_slot_id, value);
            }
        }
    }

    pub(crate) fn persistent_slot_for_symbol(
        &self,
        symbol_id: SymbolId,
    ) -> Option<(PersistenceKind, VarSlotId)> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)?;

        match symbol.persistence {
            PersistenceKind::None => None,
            PersistenceKind::Var | PersistenceKind::Varip => symbol
                .var_slot_id
                .map(|var_slot_id| (symbol.persistence, var_slot_id)),
        }
    }

    fn persistent_slots_for_kind(&self, kind: PersistenceKind) -> Vec<VarSlotId> {
        self.program
            .symbols
            .iter()
            .filter(|symbol| symbol.persistence == kind)
            .filter_map(|symbol| symbol.var_slot_id)
            .collect()
    }
}
