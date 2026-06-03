mod accounting;
mod entries;
mod exits;
mod fills;

use pine_ir::{DEFAULT_STRATEGY_INITIAL_CAPITAL, StrategyCommission, StrategyMarginSetting};

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
    commission: Option<StrategyCommission>,
    margin_long: StrategyMarginSetting,
    margin_short: StrategyMarginSetting,
    open_entry_commission: f64,
    slippage_price_offset: f64,
    limit_verification_price_offset: f64,
    cash: f64,
    position_size: f64,
    avg_price: f64,
    entry_id: Option<String>,
    entry_bar_index: Option<usize>,
    entry_time: Option<i64>,
    open_trade_max_high: Option<f64>,
    open_trade_min_low: Option<f64>,
    open_trade_equity_on_entry: Option<f64>,
    open_trade_min_equity_before_entry: Option<f64>,
    open_trade_max_equity_before_entry: Option<f64>,
    min_equity_before_open_trade: f64,
    max_equity_before_open_trade: f64,
    max_runup: f64,
    max_runup_percent: f64,
    max_drawdown: f64,
    max_drawdown_percent: f64,
    max_contracts_held_long: f64,
    orders: Vec<StrategyOrderEvent>,
    trades: Vec<StrategyTrade>,
    closed_trade_metrics: Vec<ClosedTradeMetrics>,
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
        Self::new_with_commission(initial_capital, None)
    }

    #[must_use]
    pub fn new_with_cash_per_contract_commission(
        initial_capital: f64,
        commission_per_contract: f64,
    ) -> Self {
        Self::new_with_commission(
            initial_capital,
            Some(StrategyCommission::CashPerContract(commission_per_contract)),
        )
    }

    #[must_use]
    pub fn new_with_commission(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
    ) -> Self {
        Self::new_with_commission_and_slippage(initial_capital, commission, 0.0)
    }

    #[must_use]
    pub fn new_with_commission_and_slippage(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
    ) -> Self {
        Self::new_with_commission_slippage_and_limit_verification(
            initial_capital,
            commission,
            slippage_price_offset,
            0.0,
        )
    }

    #[must_use]
    pub fn new_with_commission_slippage_and_limit_verification(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
        limit_verification_price_offset: f64,
    ) -> Self {
        Self::new_with_account_settings(
            initial_capital,
            commission,
            slippage_price_offset,
            limit_verification_price_offset,
            StrategyMarginSetting::default(),
            StrategyMarginSetting::default(),
        )
    }

    #[must_use]
    pub fn new_with_account_settings(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
        limit_verification_price_offset: f64,
        margin_long: StrategyMarginSetting,
        margin_short: StrategyMarginSetting,
    ) -> Self {
        Self {
            initial_capital,
            commission,
            margin_long,
            margin_short,
            open_entry_commission: 0.0,
            slippage_price_offset,
            limit_verification_price_offset,
            cash: initial_capital,
            position_size: 0.0,
            avg_price: 0.0,
            entry_id: None,
            entry_bar_index: None,
            entry_time: None,
            open_trade_max_high: None,
            open_trade_min_low: None,
            open_trade_equity_on_entry: None,
            open_trade_min_equity_before_entry: None,
            open_trade_max_equity_before_entry: None,
            min_equity_before_open_trade: initial_capital,
            max_equity_before_open_trade: initial_capital,
            max_runup: 0.0,
            max_runup_percent: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percent: 0.0,
            max_contracts_held_long: 0.0,
            orders: Vec::new(),
            trades: Vec::new(),
            closed_trade_metrics: Vec::new(),
            position: Vec::new(),
            equity: Vec::new(),
            diagnostics: Vec::new(),
            pending_entries: PendingEntryBook::new(),
            pending_exits: PendingExitBook::new(),
        }
    }

    fn commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        match self.commission {
            Some(StrategyCommission::CashPerContract(value)) => qty * value,
            Some(StrategyCommission::CashPerOrder(value)) => value,
            Some(StrategyCommission::Percent(value)) => qty * price * (value / 100.0),
            None => 0.0,
        }
    }

    fn entry_commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        self.commission_for_fill(qty, price)
    }

    fn exit_commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        self.commission_for_fill(qty, price)
    }

    fn entry_commission_for_closed_quantity(&self, qty: f64) -> f64 {
        if qty >= self.position_size {
            self.open_entry_commission
        } else {
            self.open_entry_commission * (qty / self.position_size)
        }
    }

    fn long_entry_fill_price(&self, price: f64) -> f64 {
        price + self.slippage_price_offset
    }

    fn long_exit_fill_price(&self, price: f64) -> f64 {
        price - self.slippage_price_offset
    }

    fn long_limit_exit_is_verified(&self, limit_price: f64, high: f64) -> bool {
        high >= limit_price + self.limit_verification_price_offset
    }

    pub(crate) fn entry_long(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) -> bool {
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_QTY".to_owned(),
                message: "`strategy.entry` quantity must be positive".to_owned(),
            });
            return false;
        }
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` fill price must be finite".to_owned(),
            });
            return false;
        }
        if self.position_size > 0.0 {
            return false;
        }

        let fill_price = self.long_entry_fill_price(price);
        if !fill_price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` slipped fill price must be finite".to_owned(),
            });
            return false;
        }
        if !self.can_afford_long_entry(qty, fill_price) {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_MARGIN".to_owned(),
                message: "`strategy.entry` requires more margin than available equity".to_owned(),
            });
            return false;
        }

        let equity_on_entry = self.cash;
        let min_equity_before_entry = self.min_equity_before_open_trade;
        let max_equity_before_entry = self.max_equity_before_open_trade;
        self.position_size = qty;
        self.max_contracts_held_long = self.max_contracts_held_long.max(qty);
        self.avg_price = fill_price;
        self.open_entry_commission = self.entry_commission_for_fill(qty, fill_price);
        self.cash -= qty * fill_price + self.open_entry_commission;
        self.entry_id = Some(id.clone());
        self.entry_bar_index = Some(bar_index);
        self.entry_time = Some(time);
        self.open_trade_max_high = Some(fill_price);
        self.open_trade_min_low = Some(fill_price);
        self.open_trade_equity_on_entry = Some(equity_on_entry);
        self.open_trade_min_equity_before_entry = Some(min_equity_before_entry);
        self.open_trade_max_equity_before_entry = Some(max_equity_before_entry);
        self.orders.push(StrategyOrderEvent {
            id,
            bar_index,
            time,
            direction: "strategy.long".to_owned(),
            qty,
            price: fill_price,
        });
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: qty,
            avg_price: Some(fill_price),
        });
        true
    }

    pub(crate) fn cancel_exit_for_entry(&mut self, entry_id: &str) {
        self.pending_exits.clear_for_entry(entry_id);
    }

    pub(crate) fn cancel_pending_order(&mut self, id: &str) {
        self.pending_entries.cancel_id(id);
        self.pending_exits.cancel_id(id);
    }

    pub(crate) fn cancel_all_pending_orders(&mut self) {
        self.pending_entries.clear_all();
        self.pending_exits.clear_all();
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
    pub(crate) fn place_pending_stop_limit_long_entry(
        &mut self,
        id: String,
        qty: f64,
        stop: f64,
        limit: f64,
        created_bar_index: usize,
    ) {
        if self.position_size > 0.0 {
            return;
        }
        self.pending_entries.place_stop_limit_long(
            id,
            qty,
            stop,
            limit,
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

        let entry_id = pending_entry.id;
        let filled = self.entry_long(
            entry_id.clone(),
            bar_index,
            time,
            fill_price,
            pending_entry.quantity,
        );
        if !filled {
            self.pending_exits.clear_for_entry(&entry_id);
        }
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
        let Some(pending_entry) = self.pending_entries.take_first_eligible_limit_long(
            bar_index,
            low,
            self.limit_verification_price_offset,
        ) else {
            return;
        };

        let PendingEntryKind::Limit { price } = pending_entry.kind else {
            return;
        };
        let entry_id = pending_entry.id;
        let filled = self.entry_long(
            entry_id.clone(),
            bar_index,
            time,
            price,
            pending_entry.quantity,
        );
        if !filled {
            self.pending_exits.clear_for_entry(&entry_id);
        }
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
        let entry_id = pending_entry.id;
        let filled = self.entry_long(
            entry_id.clone(),
            bar_index,
            time,
            price,
            pending_entry.quantity,
        );
        if !filled {
            self.pending_exits.clear_for_entry(&entry_id);
        }
        self.pending_entries.clear_all();
    }

    pub(crate) fn fill_pending_stop_limit_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if self.position_size > 0.0 {
            self.pending_entries.clear_all();
            return;
        }
        self.pending_entries
            .activate_stop_limit_long_entries(bar_index, high);
        let Some(pending_entry) = self.pending_entries.take_first_eligible_stop_limit_long(
            bar_index,
            low,
            self.limit_verification_price_offset,
        ) else {
            return;
        };

        let PendingEntryKind::StopLimit { limit_price, .. } = pending_entry.kind else {
            return;
        };
        let entry_id = pending_entry.id;
        let filled = self.entry_long(
            entry_id.clone(),
            bar_index,
            time,
            limit_price,
            pending_entry.quantity,
        );
        if !filled {
            self.pending_exits.clear_for_entry(&entry_id);
        }
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
            PendingExitTrigger::Limit(price) if self.long_limit_exit_is_verified(*price, high) => {
                Some(*price)
            }
            PendingExitTrigger::Bracket { downside, upside } => {
                if low <= *downside {
                    Some(*downside)
                } else if self.long_limit_exit_is_verified(*upside, high) {
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
                    if let Some(touch) = pending_exit.trigger.touched_candidate(
                        high,
                        low,
                        self.limit_verification_price_offset,
                    ) {
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

#[derive(Debug, Clone, PartialEq)]
struct ClosedTradeMetrics {
    commission: f64,
    profit_percent: f64,
    max_runup: f64,
    max_drawdown: f64,
}

#[cfg(test)]
mod tests;
