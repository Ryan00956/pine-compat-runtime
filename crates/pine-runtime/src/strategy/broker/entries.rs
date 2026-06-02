use crate::RuntimeDiagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PendingEntryDirection {
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum PendingEntryKind {
    Market,
    Limit {
        price: f64,
    },
    Stop {
        price: f64,
    },
    StopLimit {
        stop_price: f64,
        limit_price: f64,
        activated_bar_index: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingEntry {
    pub(super) id: String,
    pub(super) direction: PendingEntryDirection,
    pub(super) kind: PendingEntryKind,
    pub(super) quantity: f64,
    pub(super) created_bar_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct PendingEntryBook {
    entries: Vec<PendingEntry>,
}

impl PendingEntryBook {
    pub(super) fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub(super) fn count(&self) -> usize {
        self.entries.len()
    }

    #[allow(dead_code)]
    pub(super) fn current(&self) -> Option<&PendingEntry> {
        self.entries.first()
    }

    #[allow(dead_code)]
    pub(super) fn iter(&self) -> impl Iterator<Item = &PendingEntry> {
        self.entries.iter()
    }

    #[allow(dead_code)]
    pub(super) fn find_by_id(&self, id: &str) -> Option<&PendingEntry> {
        self.entries
            .iter()
            .find(|pending_entry| pending_entry.id == id)
    }

    pub(super) fn quantity_for_id(&self, id: &str) -> Option<f64> {
        self.find_by_id(id)
            .map(|pending_entry| pending_entry.quantity)
    }

    #[allow(dead_code)]
    pub(super) fn take_first_eligible_market_long(
        &mut self,
        bar_index: usize,
    ) -> Option<PendingEntry> {
        let position = self.entries.iter().position(|pending_entry| {
            pending_entry.direction == PendingEntryDirection::Long
                && pending_entry.kind == PendingEntryKind::Market
                && pending_entry.created_bar_index < bar_index
        })?;
        Some(self.entries.remove(position))
    }

    #[allow(dead_code)]
    pub(super) fn take_first_eligible_limit_long(
        &mut self,
        bar_index: usize,
        low: f64,
    ) -> Option<PendingEntry> {
        let position = self.entries.iter().position(|pending_entry| {
            pending_entry.direction == PendingEntryDirection::Long
                && matches!(pending_entry.kind, PendingEntryKind::Limit { price } if low <= price)
                && pending_entry.created_bar_index < bar_index
        })?;
        Some(self.entries.remove(position))
    }

    #[allow(dead_code)]
    pub(super) fn take_first_eligible_stop_long(
        &mut self,
        bar_index: usize,
        high: f64,
    ) -> Option<PendingEntry> {
        let position = self.entries.iter().position(|pending_entry| {
            pending_entry.direction == PendingEntryDirection::Long
                && matches!(pending_entry.kind, PendingEntryKind::Stop { price } if high >= price)
                && pending_entry.created_bar_index < bar_index
        })?;
        Some(self.entries.remove(position))
    }

    pub(super) fn activate_stop_limit_long_entries(&mut self, bar_index: usize, high: f64) {
        for pending_entry in &mut self.entries {
            if pending_entry.direction != PendingEntryDirection::Long
                || pending_entry.created_bar_index >= bar_index
            {
                continue;
            }
            let PendingEntryKind::StopLimit {
                stop_price,
                activated_bar_index,
                ..
            } = &mut pending_entry.kind
            else {
                continue;
            };
            if activated_bar_index.is_none() && high >= *stop_price {
                *activated_bar_index = Some(bar_index);
            }
        }
    }

    pub(super) fn take_first_eligible_stop_limit_long(
        &mut self,
        bar_index: usize,
        low: f64,
    ) -> Option<PendingEntry> {
        let position = self.entries.iter().position(|pending_entry| {
            pending_entry.direction == PendingEntryDirection::Long
                && matches!(
                    pending_entry.kind,
                    PendingEntryKind::StopLimit {
                        limit_price,
                        activated_bar_index: Some(activated_bar_index),
                        ..
                    } if activated_bar_index < bar_index && low <= limit_price
                )
        })?;
        Some(self.entries.remove(position))
    }

    #[allow(dead_code)]
    pub(super) fn clear_all(&mut self) {
        self.entries.clear();
    }

    #[allow(dead_code)]
    pub(super) fn place_market_long(
        &mut self,
        id: String,
        quantity: f64,
        created_bar_index: usize,
        diagnostics: &mut Vec<RuntimeDiagnostic>,
    ) {
        self.place_long(
            id,
            PendingEntryKind::Market,
            quantity,
            created_bar_index,
            diagnostics,
        );
    }

    pub(super) fn place_limit_long(
        &mut self,
        id: String,
        quantity: f64,
        price: f64,
        created_bar_index: usize,
        diagnostics: &mut Vec<RuntimeDiagnostic>,
    ) {
        if !price.is_finite() || price <= 0.0 {
            diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` limit price must be positive".to_owned(),
            });
            return;
        }
        self.place_long(
            id,
            PendingEntryKind::Limit { price },
            quantity,
            created_bar_index,
            diagnostics,
        );
    }

    pub(super) fn place_stop_long(
        &mut self,
        id: String,
        quantity: f64,
        price: f64,
        created_bar_index: usize,
        diagnostics: &mut Vec<RuntimeDiagnostic>,
    ) {
        if !price.is_finite() || price <= 0.0 {
            diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` stop price must be positive".to_owned(),
            });
            return;
        }
        self.place_long(
            id,
            PendingEntryKind::Stop { price },
            quantity,
            created_bar_index,
            diagnostics,
        );
    }

    pub(super) fn place_stop_limit_long(
        &mut self,
        id: String,
        quantity: f64,
        stop_price: f64,
        limit_price: f64,
        created_bar_index: usize,
        diagnostics: &mut Vec<RuntimeDiagnostic>,
    ) {
        if !stop_price.is_finite()
            || stop_price <= 0.0
            || !limit_price.is_finite()
            || limit_price <= 0.0
        {
            diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` stop-limit prices must be positive".to_owned(),
            });
            return;
        }
        self.place_long(
            id,
            PendingEntryKind::StopLimit {
                stop_price,
                limit_price,
                activated_bar_index: None,
            },
            quantity,
            created_bar_index,
            diagnostics,
        );
    }

    fn place_long(
        &mut self,
        id: String,
        kind: PendingEntryKind,
        quantity: f64,
        created_bar_index: usize,
        diagnostics: &mut Vec<RuntimeDiagnostic>,
    ) {
        if !quantity.is_finite() || quantity <= 0.0 {
            diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_QTY".to_owned(),
                message: "`strategy.entry` quantity must be positive".to_owned(),
            });
            return;
        }
        let pending_entry = PendingEntry {
            id,
            direction: PendingEntryDirection::Long,
            kind,
            quantity,
            created_bar_index,
        };
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|existing| existing.id == pending_entry.id)
        {
            *existing = pending_entry;
        } else {
            self.entries.push(pending_entry);
        }
    }
}
