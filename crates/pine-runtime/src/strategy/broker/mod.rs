mod accounting;
mod exits;
mod fills;

use pine_ir::DEFAULT_STRATEGY_INITIAL_CAPITAL;

use exits::{PendingExit, PendingExitTrigger};

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
    pending_exit: Option<PendingExit>,
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
            pending_exit: None,
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
        if self
            .pending_exit
            .as_ref()
            .is_some_and(|pending_exit| pending_exit.from_entry == entry_id)
        {
            self.pending_exit = None;
        }
    }

    pub(crate) fn evaluate_pending_exits(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        let Some(pending_exit) = self.pending_exit.clone() else {
            return;
        };
        if pending_exit.last_update_bar_index >= bar_index {
            return;
        }
        if self.position_size <= 0.0
            || self.entry_id.as_deref() != Some(pending_exit.from_entry.as_str())
        {
            self.pending_exit = None;
            return;
        }
        let triggered = match pending_exit.trigger {
            PendingExitTrigger::Stop(price) => low <= price,
            PendingExitTrigger::Limit(price) => high >= price,
            PendingExitTrigger::Bracket { .. } => false,
        };
        if triggered {
            self.fill_pending_exit(pending_exit, bar_index, time);
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
