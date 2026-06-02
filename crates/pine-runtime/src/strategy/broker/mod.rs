mod accounting;
mod entries;
mod exits;
mod fills;

use pine_ir::DEFAULT_STRATEGY_INITIAL_CAPITAL;

use entries::{PendingEntryBook, PendingEntryKind};
use exits::{
    PendingExit, PendingExitBook, PendingExitSide, PendingExitTrigger, PendingTrailingUpdate,
};
pub(crate) use exits::{TrailPointsExitSpec, TrailPriceExitSpec};

use crate::{
    RuntimeDiagnostic, StrategyEquitySnapshot, StrategyOrderEvent, StrategyPositionSnapshot,
    StrategyResult, StrategyTrade,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BrokerState {
    initial_capital: f64,
    cash: f64,
    position_size: f64,
    avg_price: f64,
    entry_id: Option<String>,
    entry_bar_index: Option<usize>,
    entry_time: Option<i64>,
    orders: Vec<StrategyOrderEvent>,
    trades: Vec<StrategyTrade>,
    position: Vec<StrategyPositionSnapshot>,
    equity: Vec<StrategyEquitySnapshot>,
    diagnostics: Vec<RuntimeDiagnostic>,
    pending_entries: PendingEntryBook,
    pending_exits: PendingExitBook,
}

impl Default for BrokerState {
    fn default() -> Self {
        Self::new(DEFAULT_STRATEGY_INITIAL_CAPITAL)
    }
}

impl BrokerState {
    #[must_use]
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            cash: initial_capital,
            position_size: 0.0,
            avg_price: 0.0,
            entry_id: None,
            entry_bar_index: None,
            entry_time: None,
            orders: Vec::new(),
            trades: Vec::new(),
            position: Vec::new(),
            equity: Vec::new(),
            diagnostics: Vec::new(),
            pending_entries: PendingEntryBook::new(),
            pending_exits: PendingExitBook::new(),
        }
    }

    pub(crate) fn entry_long(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) {
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_QTY".to_owned(),
                message: "`strategy.entry` quantity must be positive".to_owned(),
            });
            return;
        }
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` fill price must be finite".to_owned(),
            });
            return;
        }
        if self.position_size > 0.0 {
            return;
        }

        self.position_size = qty;
        self.avg_price = price;
        self.cash -= qty * price;
        self.entry_id = Some(id.clone());
        self.entry_bar_index = Some(bar_index);
        self.entry_time = Some(time);
        self.orders.push(StrategyOrderEvent {
            id,
            bar_index,
            time,
            direction: "strategy.long".to_owned(),
            qty,
            price,
        });
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: qty,
            avg_price: Some(price),
        });
    }

    pub(crate) fn cancel_exit_for_entry(&mut self, entry_id: &str) {
        self.pending_exits.clear_for_entry(entry_id);
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_market_long_entry(
        &mut self,
        id: String,
        qty: f64,
        created_bar_index: usize,
    ) {
        if self.position_size > 0.0 {
            return;
        }
        self.pending_entries
            .place_market_long(id, qty, created_bar_index, &mut self.diagnostics);
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_limit_long_entry(
        &mut self,
        id: String,
        qty: f64,
        limit: f64,
        created_bar_index: usize,
    ) {
        if self.position_size > 0.0 {
            return;
        }
        self.pending_entries.place_limit_long(
            id,
            qty,
            limit,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_stop_long_entry(
        &mut self,
        id: String,
        qty: f64,
        stop: f64,
        created_bar_index: usize,
    ) {
        if self.position_size > 0.0 {
            return;
        }
        self.pending_entries.place_stop_long(
            id,
            qty,
            stop,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    fn pending_entry_count(&self) -> usize {
        self.pending_entries.count()
    }

    pub(crate) fn fill_pending_market_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        fill_price: f64,
    ) {
        if self.position_size > 0.0 {
            self.pending_entries.clear_all();
            return;
        }
        let Some(pending_entry) = self
            .pending_entries
            .take_first_eligible_market_long(bar_index)
        else {
            return;
        };

        self.entry_long(
            pending_entry.id,
            bar_index,
            time,
            fill_price,
            pending_entry.quantity,
        );
        self.pending_entries.clear_all();
    }

    pub(crate) fn fill_pending_limit_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        low: f64,
    ) {
        if self.position_size > 0.0 {
            self.pending_entries.clear_all();
            return;
        }
        let Some(pending_entry) = self
            .pending_entries
            .take_first_eligible_limit_long(bar_index, low)
        else {
            return;
        };

        let PendingEntryKind::Limit { price } = pending_entry.kind else {
            return;
        };
        self.entry_long(
            pending_entry.id,
            bar_index,
            time,
            price,
            pending_entry.quantity,
        );
        self.pending_entries.clear_all();
    }

    pub(crate) fn fill_pending_stop_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
    ) {
        if self.position_size > 0.0 {
            self.pending_entries.clear_all();
            return;
        }
        let Some(pending_entry) = self
            .pending_entries
            .take_first_eligible_stop_long(bar_index, high)
        else {
            return;
        };

        let PendingEntryKind::Stop { price } = pending_entry.kind else {
            return;
        };
        self.entry_long(
            pending_entry.id,
            bar_index,
            time,
            price,
            pending_entry.quantity,
        );
        self.pending_entries.clear_all();
    }

    fn has_pending_entry(&self, id: &str) -> bool {
        self.pending_entries.quantity_for_id(id).is_some()
    }

    pub(crate) fn reject_entry_relative_exit_for_pending_entry(
        &mut self,
        from_entry: &str,
    ) -> bool {
        if self.position_size > 0.0 || self.pending_entries.quantity_for_id(from_entry).is_none() {
            return false;
        }

        self.diagnostics.push(RuntimeDiagnostic {
            code: "E_STRATEGY_EXIT_ENTRY".to_owned(),
            message: "`strategy.exit` from_entry must match the current long entry".to_owned(),
        });
        true
    }

    fn pending_exit(&self) -> Option<&PendingExit> {
        self.pending_exits.current()
    }

    #[allow(dead_code)]
    fn pending_exit_mut(&mut self) -> Option<&mut PendingExit> {
        self.pending_exits.current_mut()
    }

    #[allow(dead_code)]
    fn pending_exit_count(&self) -> usize {
        self.pending_exits.count()
    }

    #[allow(dead_code)]
    fn pending_exit_by_identity(&self, id: &str, from_entry: &str) -> Option<&PendingExit> {
        self.pending_exits.find_by_identity(id, from_entry)
    }

    #[allow(dead_code)]
    fn pending_exits_in_placement_order(&self) -> impl Iterator<Item = &PendingExit> {
        self.pending_exits.iter()
    }

    pub(crate) fn evaluate_pending_exits(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if self.pending_exit_count() > 1 {
            self.evaluate_multiple_pending_exits(bar_index, time, high, low);
            return;
        }

        let Some(mut pending_exit) = self.pending_exit().cloned() else {
            return;
        };
        if pending_exit.last_update_bar_index >= bar_index {
            return;
        }
        if self.position_size <= 0.0
            || self.entry_id.as_deref() != Some(pending_exit.from_entry.as_str())
        {
            if self.position_size <= 0.0 && self.has_pending_entry(&pending_exit.from_entry) {
                return;
            }
            self.pending_exits.clear_for_entry(&pending_exit.from_entry);
            return;
        }
        let triggered_price = match &mut pending_exit.trigger {
            PendingExitTrigger::Stop(price) if low <= *price => Some(*price),
            PendingExitTrigger::Limit(price) if high >= *price => Some(*price),
            PendingExitTrigger::Bracket { downside, upside } => {
                if low <= *downside {
                    Some(*downside)
                } else if high >= *upside {
                    Some(*upside)
                } else {
                    None
                }
            }
            PendingExitTrigger::Trailing(trailing) => match trailing.evaluate_update(high, low) {
                PendingTrailingUpdate::NoChange => return,
                PendingTrailingUpdate::Persist(updated_trailing) => {
                    pending_exit.trigger = PendingExitTrigger::Trailing(updated_trailing);
                    self.pending_exits.replace_all(pending_exit);
                    return;
                }
                PendingTrailingUpdate::Candidate(touch) => Some(touch.exit_price),
            },
            _ => None,
        };
        if let Some(exit_price) = triggered_price {
            let from_entry = pending_exit.from_entry.clone();
            self.fill_pending_exit(pending_exit, bar_index, time, exit_price);
            if self.position_size <= 0.0 {
                self.pending_exits.clear_all();
            } else {
                self.pending_exits.clear_for_entry(&from_entry);
            }
        }
    }

    fn evaluate_multiple_pending_exits(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        let pending_exits: Vec<PendingExit> = self.pending_exits.iter().cloned().collect();
        let Some(first_pending_exit) = pending_exits.first() else {
            return;
        };
        if self.position_size <= 0.0
            || self.entry_id.as_deref() != Some(first_pending_exit.from_entry.as_str())
        {
            if self.position_size <= 0.0 {
                let attached_pending_entry_ids: Vec<String> = pending_exits
                    .iter()
                    .filter(|pending_exit| self.has_pending_entry(&pending_exit.from_entry))
                    .map(|pending_exit| pending_exit.from_entry.clone())
                    .collect();
                if !attached_pending_entry_ids.is_empty() {
                    for pending_exit in pending_exits {
                        if !attached_pending_entry_ids
                            .iter()
                            .any(|entry_id| entry_id == &pending_exit.from_entry)
                        {
                            self.pending_exits.clear_for_entry(&pending_exit.from_entry);
                        }
                    }
                    return;
                }
            }
            self.pending_exits
                .clear_for_entry(&first_pending_exit.from_entry);
            return;
        }

        let mut touched_candidates = Vec::new();
        let mut state_updates = Vec::new();
        for pending_exit in pending_exits {
            if pending_exit.last_update_bar_index >= bar_index {
                continue;
            }
            if self.entry_id.as_deref() != Some(pending_exit.from_entry.as_str()) {
                self.pending_exits.clear_for_entry(&pending_exit.from_entry);
                continue;
            }

            match pending_exit.trigger.clone() {
                PendingExitTrigger::Trailing(trailing) => {
                    match trailing.evaluate_update(high, low) {
                        PendingTrailingUpdate::NoChange => {}
                        PendingTrailingUpdate::Persist(updated_trailing) => {
                            let mut updated_pending_exit = pending_exit;
                            updated_pending_exit.trigger =
                                PendingExitTrigger::Trailing(updated_trailing);
                            state_updates.push(updated_pending_exit);
                        }
                        PendingTrailingUpdate::Candidate(touch) => {
                            touched_candidates.push((pending_exit, touch.exit_price, touch.side));
                        }
                    }
                }
                _ => {
                    if let Some(touch) = pending_exit.trigger.touched_candidate(high, low) {
                        touched_candidates.push((pending_exit, touch.exit_price, touch.side));
                    }
                }
            }
        }

        let winning_side = if touched_candidates
            .iter()
            .any(|(_, _, side)| *side == PendingExitSide::Stop)
        {
            PendingExitSide::Stop
        } else if touched_candidates
            .iter()
            .any(|(_, _, side)| *side == PendingExitSide::Limit)
        {
            PendingExitSide::Limit
        } else {
            for updated_pending_exit in state_updates {
                self.pending_exits.replace_or_append(updated_pending_exit);
            }
            return;
        };

        let mut filled_identities = Vec::new();
        for (pending_exit, exit_price, side) in touched_candidates {
            if side != winning_side {
                continue;
            }
            if self.position_size <= 0.0 {
                break;
            }
            if self.entry_id.as_deref() != Some(pending_exit.from_entry.as_str()) {
                self.pending_exits.clear_for_entry(&pending_exit.from_entry);
                continue;
            }
            filled_identities.push((pending_exit.id.clone(), pending_exit.from_entry.clone()));
            self.fill_pending_exit(pending_exit, bar_index, time, exit_price);
        }

        if self.position_size <= 0.0 {
            self.pending_exits.clear_all();
        } else {
            for updated_pending_exit in state_updates {
                self.pending_exits.replace_or_append(updated_pending_exit);
            }
            self.pending_exits.remove_identities(&filled_identities);
        }
    }

    #[must_use]
    pub fn result(&self) -> StrategyResult {
        StrategyResult {
            orders: self.orders.clone(),
            trades: self.trades.clone(),
            position: self.position.clone(),
            equity: self.equity.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
