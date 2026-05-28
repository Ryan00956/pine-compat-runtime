use pine_ir::DEFAULT_STRATEGY_INITIAL_CAPITAL;

use crate::{
    PineValue, RuntimeDiagnostic, StrategyEquitySnapshot, StrategyOrderEvent,
    StrategyPositionSnapshot, StrategyResult, StrategyTrade,
};

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

#[derive(Debug, Clone, PartialEq)]
struct PendingExit {
    id: String,
    from_entry: String,
    stop_price: f64,
    last_update_bar_index: usize,
}

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

    pub(crate) fn close_long(&mut self, id: String, bar_index: usize, time: i64, price: f64) {
        if self.position_size <= 0.0 || self.entry_id.as_deref() != Some(id.as_str()) {
            return;
        }

        let qty = self.position_size;
        let entry_price = self.avg_price;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        self.cancel_exit_for_entry(&id);
        self.trades.push(StrategyTrade {
            id,
            entry_bar_index,
            exit_bar_index: bar_index,
            entry_time,
            exit_time: time,
            entry_price,
            exit_price: price,
            qty,
            profit: (price - entry_price) * qty,
        });

        self.cash += qty * price;
        self.position_size = 0.0;
        self.avg_price = 0.0;
        self.entry_id = None;
        self.entry_bar_index = None;
        self.entry_time = None;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: 0.0,
            avg_price: None,
        });
    }

    pub(crate) fn place_exit_stop(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        bar_index: usize,
    ) {
        if !stop_price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_PRICE".to_owned(),
                message: "`strategy.exit` stop price must be finite".to_owned(),
            });
            return;
        }
        if self.position_size <= 0.0 || self.entry_id.as_deref() != Some(from_entry.as_str()) {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_ENTRY".to_owned(),
                message: "`strategy.exit` from_entry must match the current long entry".to_owned(),
            });
            return;
        }

        if self.pending_exit.as_ref().is_some_and(|pending_exit| {
            pending_exit.id == id
                && pending_exit.from_entry == from_entry
                && pending_exit.stop_price == stop_price
        }) {
            return;
        }

        self.pending_exit = Some(PendingExit {
            id,
            from_entry,
            stop_price,
            last_update_bar_index: bar_index,
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

    pub(crate) fn evaluate_pending_exits(&mut self, bar_index: usize, time: i64, low: f64) {
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
        if low <= pending_exit.stop_price {
            self.fill_pending_stop_exit(pending_exit, bar_index, time);
        }
    }

    fn fill_pending_stop_exit(&mut self, pending_exit: PendingExit, bar_index: usize, time: i64) {
        let qty = self.position_size;
        let entry_price = self.avg_price;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        let exit_price = pending_exit.stop_price;

        self.orders.push(StrategyOrderEvent {
            id: pending_exit.id,
            bar_index,
            time,
            direction: "strategy.exit".to_owned(),
            qty,
            price: exit_price,
        });
        self.trades.push(StrategyTrade {
            id: pending_exit.from_entry,
            entry_bar_index,
            exit_bar_index: bar_index,
            entry_time,
            exit_time: time,
            entry_price,
            exit_price,
            qty,
            profit: (exit_price - entry_price) * qty,
        });

        self.cash += qty * exit_price;
        self.position_size = 0.0;
        self.avg_price = 0.0;
        self.entry_id = None;
        self.entry_bar_index = None;
        self.entry_time = None;
        self.pending_exit = None;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: 0.0,
            avg_price: None,
        });
    }

    #[cfg(test)]
    #[must_use]
    fn pending_exit_count(&self) -> usize {
        usize::from(self.pending_exit.is_some())
    }

    pub(crate) fn record_equity(&mut self, bar_index: usize, close: f64) {
        let market_value = self.position_size * close;
        let equity = self.cash + market_value;
        let net_profit = normalize_zero(equity - self.initial_capital);
        self.equity.push(StrategyEquitySnapshot {
            bar_index,
            cash: self.cash,
            market_value,
            equity,
            net_profit,
        });
    }

    #[must_use]
    pub(crate) fn open_profit(&self, close: f64) -> f64 {
        if self.position_size > 0.0 {
            normalize_zero((close - self.avg_price) * self.position_size)
        } else {
            0.0
        }
    }

    #[must_use]
    pub(crate) fn realized_profit(&self) -> f64 {
        normalize_zero(self.trades.iter().map(|trade| trade.profit).sum())
    }

    #[must_use]
    pub(crate) fn equity_value(&self, close: f64) -> f64 {
        normalize_zero(self.initial_capital + self.realized_profit() + self.open_profit(close))
    }

    #[must_use]
    pub(crate) fn position_size(&self) -> f64 {
        self.position_size
    }

    #[must_use]
    pub(crate) fn position_avg_price_value(&self) -> PineValue {
        if self.position_size > 0.0 {
            PineValue::Float(self.avg_price)
        } else {
            PineValue::Na
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
mod tests {
    use super::*;

    fn broker_with_long_entry() -> BrokerState {
        let mut broker = BrokerState::new(100_000.0);
        broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0);
        broker
    }

    #[test]
    fn place_exit_while_flat_records_diagnostic_without_pending_state() {
        let mut broker = BrokerState::new(100_000.0);

        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.diagnostics.len(), 1);
        assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
    }

    #[test]
    fn place_exit_while_long_records_pending_stop() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XL".to_owned(),
                from_entry: "L".to_owned(),
                stop_price: 95.0,
                last_update_bar_index: 0,
            })
        );
        assert!(broker.diagnostics.is_empty());
    }

    #[test]
    fn place_exit_replaces_existing_pending_stop() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
        broker.place_exit_stop("XL2".to_owned(), "L".to_owned(), 90.0, 1);

        assert_eq!(broker.pending_exit_count(), 1);
        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XL2".to_owned(),
                from_entry: "L".to_owned(),
                stop_price: 90.0,
                last_update_bar_index: 1,
            })
        );
    }

    #[test]
    fn close_long_cancels_matching_pending_exit() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.close_long("L".to_owned(), 1, 20, 110.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
    }

    #[test]
    fn mismatched_entry_id_records_diagnostic_without_pending_state() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_stop("XL".to_owned(), "OTHER".to_owned(), 95.0, 0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.diagnostics.len(), 1);
        assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
    }

    #[test]
    fn repeated_entry_noop_leaves_pending_exit_untouched() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.entry_long("L2".to_owned(), 1, 20, 105.0, 1.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert_eq!(
            broker.pending_exit.as_ref().map(|pending_exit| {
                (
                    pending_exit.id.as_str(),
                    pending_exit.from_entry.as_str(),
                    pending_exit.stop_price,
                )
            }),
            Some(("XL", "L", 95.0))
        );
    }

    #[test]
    fn pending_stop_is_not_eligible_on_creation_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.evaluate_pending_exits(0, 10, 90.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert!(broker.trades.is_empty());
        assert_eq!(broker.position_size, 2.0);
    }

    #[test]
    fn pending_stop_fills_on_later_crossing_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.evaluate_pending_exits(1, 20, 94.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.orders.len(), 2);
        assert_eq!(broker.orders[1].id, "XL");
        assert_eq!(broker.orders[1].direction, "strategy.exit");
        assert_eq!(broker.orders[1].price, 95.0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].id, "L");
        assert_eq!(broker.trades[0].exit_bar_index, 1);
        assert_eq!(broker.trades[0].exit_price, 95.0);
        assert_eq!(broker.trades[0].profit, -10.0);
        assert_eq!(broker.position_size, 0.0);
        assert_eq!(broker.position.last().unwrap().avg_price, None);
    }

    #[test]
    fn unchanged_repeated_exit_keeps_original_eligibility_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 1);

        broker.evaluate_pending_exits(1, 20, 94.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 95.0);
    }

    #[test]
    fn changed_repeated_exit_replaces_price_and_delays_eligibility() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 90.0, 1);

        broker.evaluate_pending_exits(1, 20, 89.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert!(broker.trades.is_empty());

        broker.evaluate_pending_exits(2, 30, 89.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 90.0);
    }
}
