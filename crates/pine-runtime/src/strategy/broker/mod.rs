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
        };
        if triggered {
            self.fill_pending_exit(pending_exit, bar_index, time);
        }
    }

    #[cfg(test)]
    #[must_use]
    fn pending_exit_count(&self) -> usize {
        usize::from(self.pending_exit.is_some())
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
    fn trade_counts_start_flat() {
        let broker = BrokerState::new(100_000.0);

        assert_eq!(broker.closed_trade_count(), 0);
        assert_eq!(broker.open_trade_count(), 0);
    }

    #[test]
    fn trade_counts_track_long_entry_and_no_pyramiding_noop() {
        let mut broker = BrokerState::new(100_000.0);

        broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0);
        assert_eq!(broker.closed_trade_count(), 0);
        assert_eq!(broker.open_trade_count(), 1);

        broker.entry_long("L2".to_owned(), 1, 20, 105.0, 1.0);
        assert_eq!(broker.closed_trade_count(), 0);
        assert_eq!(broker.open_trade_count(), 1);
    }

    #[test]
    fn trade_counts_track_matching_close() {
        let mut broker = broker_with_long_entry();

        broker.close_long("L".to_owned(), 1, 20, 110.0);

        assert_eq!(broker.closed_trade_count(), 1);
        assert_eq!(broker.open_trade_count(), 0);
    }

    #[test]
    fn trade_counts_track_filled_pending_exit() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

        assert_eq!(broker.closed_trade_count(), 1);
        assert_eq!(broker.open_trade_count(), 0);
    }

    #[test]
    fn trade_counts_ignore_mismatched_close_and_exit() {
        let mut broker = broker_with_long_entry();

        broker.close_long("OTHER".to_owned(), 1, 20, 110.0);
        broker.place_exit_stop("XL".to_owned(), "OTHER".to_owned(), 95.0, 1);

        assert_eq!(broker.closed_trade_count(), 0);
        assert_eq!(broker.open_trade_count(), 1);
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
                trigger: PendingExitTrigger::Stop(95.0),
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
                trigger: PendingExitTrigger::Stop(90.0),
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
                    pending_exit.trigger.price(),
                )
            }),
            Some(("XL", "L", 95.0))
        );
    }

    #[test]
    fn pending_stop_is_not_eligible_on_creation_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.evaluate_pending_exits(0, 10, 100.0, 90.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert!(broker.trades.is_empty());
        assert_eq!(broker.position_size, 2.0);
    }

    #[test]
    fn pending_stop_fills_on_later_crossing_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

        broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

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

        broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 95.0);
    }

    #[test]
    fn changed_repeated_exit_replaces_price_and_delays_eligibility() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
        broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 90.0, 1);

        broker.evaluate_pending_exits(1, 20, 100.0, 89.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert!(broker.trades.is_empty());

        broker.evaluate_pending_exits(2, 30, 100.0, 89.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 90.0);
    }

    #[test]
    fn pending_limit_fills_on_later_crossing_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

        broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.orders.len(), 2);
        assert_eq!(broker.orders[1].direction, "strategy.exit");
        assert_eq!(broker.orders[1].price, 110.0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 110.0);
        assert_eq!(broker.trades[0].profit, 20.0);
    }

    #[test]
    fn profit_ticks_create_limit_from_average_entry_price() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 0.5, 0);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XP".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Limit(105.0),
                last_update_bar_index: 0,
            })
        );
    }

    #[test]
    fn loss_ticks_create_stop_from_average_entry_price() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_loss_ticks("XL".to_owned(), "L".to_owned(), 5.0, 0.5, 0);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XL".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Stop(97.5),
                last_update_bar_index: 0,
            })
        );
    }

    #[test]
    fn invalid_profit_ticks_record_diagnostic_without_changing_pending_exit() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 0.0, 0.01, 1);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XS".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Stop(95.0),
                last_update_bar_index: 0,
            })
        );
        assert_eq!(broker.diagnostics.len(), 1);
        assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
    }

    #[test]
    fn invalid_exit_mintick_records_diagnostic_without_changing_pending_exit() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

        broker.place_exit_loss_ticks("XS".to_owned(), "L".to_owned(), 5.0, f64::NAN, 1);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XL".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Limit(110.0),
                last_update_bar_index: 0,
            })
        );
        assert_eq!(broker.diagnostics.len(), 1);
        assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_MINTICK");
    }

    #[test]
    fn profit_ticks_without_matching_entry_record_diagnostic_without_pending_state() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_profit_ticks("XP".to_owned(), "OTHER".to_owned(), 10.0, 0.01, 0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.diagnostics.len(), 1);
        assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
    }

    #[test]
    fn unchanged_repeated_profit_ticks_keep_original_eligibility_bar() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 0);
        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 1);

        broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 110.0);
    }

    #[test]
    fn changed_repeated_profit_ticks_delay_eligibility() {
        let mut broker = broker_with_long_entry();
        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 0);
        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 11.0, 1.0, 1);

        broker.evaluate_pending_exits(1, 20, 112.0, 100.0);

        assert_eq!(broker.pending_exit_count(), 1);
        assert!(broker.trades.is_empty());

        broker.evaluate_pending_exits(2, 30, 112.0, 100.0);

        assert_eq!(broker.pending_exit_count(), 0);
        assert_eq!(broker.trades.len(), 1);
        assert_eq!(broker.trades[0].exit_price, 111.0);
    }

    #[test]
    fn profit_ticks_replace_stop_and_loss_ticks_replace_limit() {
        let mut broker = broker_with_long_entry();

        broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);
        broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 1);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XP".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Limit(110.0),
                last_update_bar_index: 1,
            })
        );

        broker.place_exit_loss_ticks("XL".to_owned(), "L".to_owned(), 5.0, 1.0, 2);

        assert_eq!(
            broker.pending_exit,
            Some(PendingExit {
                id: "XL".to_owned(),
                from_entry: "L".to_owned(),
                trigger: PendingExitTrigger::Stop(95.0),
                last_update_bar_index: 2,
            })
        );
    }
}
