use super::{BrokerState, ClosedTradeMetrics, exits::PendingExit};
use crate::{RuntimeDiagnostic, StrategyOrderEvent, StrategyPositionSnapshot, StrategyTrade};

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn closed_trade_profit_percent(entry_price: f64, qty: f64, profit: f64) -> f64 {
    let denominator = entry_price * qty;
    if !profit.is_finite() || !denominator.is_finite() || denominator <= 0.0 {
        return 0.0;
    }
    normalize_zero(profit / denominator * 100.0)
}

impl BrokerState {
    pub(crate) fn evaluate_margin_call_long(
        &mut self,
        bar_index: usize,
        time: i64,
        current_price: f64,
    ) {
        if self.position_size <= 0.0 || !self.margin_long.is_active() || !current_price.is_finite()
        {
            return;
        }
        let margin_ratio = self.margin_long.value_percent / 100.0;
        if !margin_ratio.is_finite() || margin_ratio <= 0.0 || current_price <= 0.0 {
            return;
        }
        let margin_required = self.position_size * current_price * margin_ratio;
        let available_funds = self.equity_value(current_price) - margin_required;
        if !available_funds.is_finite() || available_funds >= 0.0 {
            return;
        }
        let cover_amount = (available_funds / margin_ratio / current_price).trunc();
        let qty = (cover_amount * 4.0).abs().min(self.position_size);
        if !qty.is_finite() || qty <= 0.0 {
            return;
        }

        let entry_id = self
            .entry_id
            .clone()
            .unwrap_or_else(|| "Margin Call".to_owned());
        let exit_id = "Margin Call".to_owned();
        let entry_price = self.avg_price;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        let entry_commission = self.entry_commission_for_closed_quantity(qty);
        let exit_commission = self.exit_commission_for_fill(qty, current_price);
        let commission = entry_commission + exit_commission;
        let profit = (current_price - entry_price) * qty - commission;

        self.pending_exits.clear_for_entry(&entry_id);
        self.orders.push(StrategyOrderEvent {
            id: exit_id.clone(),
            bar_index,
            time,
            direction: "strategy.short".to_owned(),
            qty,
            price: current_price,
        });
        self.trades.push(StrategyTrade {
            id: entry_id,
            exit_id,
            entry_bar_index,
            exit_bar_index: bar_index,
            entry_time,
            exit_time: time,
            entry_price,
            exit_price: current_price,
            qty,
            profit,
        });
        self.closed_trade_metrics.push(ClosedTradeMetrics {
            commission,
            profit_percent: closed_trade_profit_percent(entry_price, qty, profit),
            max_runup: self.current_open_trade_max_runup_for_quantity(qty),
            max_drawdown: self.current_open_trade_max_drawdown_for_quantity(qty),
        });

        self.cash += qty * current_price - exit_commission;
        if qty >= self.position_size {
            self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
            self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
            self.position_size = 0.0;
            self.avg_price = 0.0;
            self.entry_id = None;
            self.entry_bar_index = None;
            self.entry_time = None;
            self.open_entry_commission = 0.0;
            self.open_trade_max_high = None;
            self.open_trade_min_low = None;
            self.open_trade_equity_on_entry = None;
            self.open_trade_min_equity_before_entry = None;
            self.open_trade_max_equity_before_entry = None;
            self.position.push(StrategyPositionSnapshot {
                bar_index,
                size: 0.0,
                avg_price: None,
            });
            return;
        }

        self.position_size -= qty;
        self.open_entry_commission -= entry_commission;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: self.position_size,
            avg_price: Some(self.avg_price),
        });
    }

    pub(crate) fn close_all_long(&mut self, bar_index: usize, time: i64, price: f64) {
        let Some(id) = self.entry_id.clone() else {
            return;
        };
        self.close_long(id, bar_index, time, price);
    }

    pub(crate) fn close_long(&mut self, id: String, bar_index: usize, time: i64, price: f64) {
        if self.position_size <= 0.0 || self.entry_id.as_deref() != Some(id.as_str()) {
            return;
        }
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.close` fill price must be finite".to_owned(),
            });
            return;
        }

        let price = self.long_exit_fill_price(price);
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.close` slipped fill price must be finite".to_owned(),
            });
            return;
        }

        let qty = self.position_size;
        let entry_price = self.avg_price;
        let entry_commission = self.entry_commission_for_closed_quantity(qty);
        let exit_commission = self.exit_commission_for_fill(qty, price);
        let commission = entry_commission + exit_commission;
        let profit = (price - entry_price) * qty - commission;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        self.cancel_exit_for_entry(&id);
        self.trades.push(StrategyTrade {
            exit_id: id.clone(),
            id,
            entry_bar_index,
            exit_bar_index: bar_index,
            entry_time,
            exit_time: time,
            entry_price,
            exit_price: price,
            qty,
            profit,
        });
        self.closed_trade_metrics.push(ClosedTradeMetrics {
            commission,
            profit_percent: closed_trade_profit_percent(entry_price, qty, profit),
            max_runup: self.current_open_trade_max_runup_for_quantity(qty),
            max_drawdown: self.current_open_trade_max_drawdown_for_quantity(qty),
        });

        self.cash += qty * price - exit_commission;
        self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
        self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
        self.position_size = 0.0;
        self.avg_price = 0.0;
        self.entry_id = None;
        self.entry_bar_index = None;
        self.entry_time = None;
        self.open_entry_commission = 0.0;
        self.open_trade_max_high = None;
        self.open_trade_min_low = None;
        self.open_trade_equity_on_entry = None;
        self.open_trade_min_equity_before_entry = None;
        self.open_trade_max_equity_before_entry = None;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: 0.0,
            avg_price: None,
        });
    }

    pub(super) fn fill_pending_exit(
        &mut self,
        pending_exit: PendingExit,
        bar_index: usize,
        time: i64,
        exit_price: f64,
    ) {
        let qty = pending_exit.reserved_quantity.min(self.position_size);
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_QTY".to_owned(),
                message: "`strategy.exit` quantity must be finite and positive".to_owned(),
            });
            return;
        }
        let exit_price = self.long_exit_fill_price(exit_price);
        if !exit_price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.exit` slipped fill price must be finite".to_owned(),
            });
            return;
        }
        let entry_price = self.avg_price;
        let entry_commission = self.entry_commission_for_closed_quantity(qty);
        let exit_commission = self.exit_commission_for_fill(qty, exit_price);
        let commission = entry_commission + exit_commission;
        let profit = (exit_price - entry_price) * qty - commission;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        let exit_id = pending_exit.id;
        let entry_id = pending_exit.from_entry;

        self.orders.push(StrategyOrderEvent {
            id: exit_id.clone(),
            bar_index,
            time,
            direction: "strategy.exit".to_owned(),
            qty,
            price: exit_price,
        });
        self.trades.push(StrategyTrade {
            id: entry_id,
            exit_id,
            entry_bar_index,
            exit_bar_index: bar_index,
            entry_time,
            exit_time: time,
            entry_price,
            exit_price,
            qty,
            profit,
        });
        self.closed_trade_metrics.push(ClosedTradeMetrics {
            commission,
            profit_percent: closed_trade_profit_percent(entry_price, qty, profit),
            max_runup: self.current_open_trade_max_runup_for_quantity(qty),
            max_drawdown: self.current_open_trade_max_drawdown_for_quantity(qty),
        });

        self.cash += qty * exit_price - exit_commission;
        if qty >= self.position_size {
            self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
            self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
            self.position_size = 0.0;
            self.avg_price = 0.0;
            self.entry_id = None;
            self.entry_bar_index = None;
            self.entry_time = None;
            self.open_entry_commission = 0.0;
            self.open_trade_max_high = None;
            self.open_trade_min_low = None;
            self.open_trade_equity_on_entry = None;
            self.open_trade_min_equity_before_entry = None;
            self.open_trade_max_equity_before_entry = None;
            self.position.push(StrategyPositionSnapshot {
                bar_index,
                size: 0.0,
                avg_price: None,
            });
            return;
        }

        self.position_size -= qty;
        self.open_entry_commission -= entry_commission;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: self.position_size,
            avg_price: Some(self.avg_price),
        });
    }
}
