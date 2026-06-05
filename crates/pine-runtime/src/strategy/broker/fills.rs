use super::{
    BrokerState, ClosedTradeMetrics,
    exits::PendingExit,
    ledger::{OpenTrade, TradeAllocation},
};
use crate::{RuntimeDiagnostic, StrategyOrderEvent, StrategyTrade};

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct AllocatedEntryFill {
    entry_price: f64,
    entry_bar_index: usize,
    entry_time: i64,
    entry_commission: f64,
}

impl AllocatedEntryFill {
    fn from_allocations(
        allocations: &[TradeAllocation],
        fallback_entry_price: f64,
        fallback_entry_bar_index: usize,
        fallback_entry_time: i64,
        fallback_entry_commission: f64,
    ) -> Self {
        let Some(first_allocation) = allocations.first() else {
            return Self {
                entry_price: fallback_entry_price,
                entry_bar_index: fallback_entry_bar_index,
                entry_time: fallback_entry_time,
                entry_commission: fallback_entry_commission,
            };
        };

        Self {
            entry_price: first_allocation.entry_price,
            entry_bar_index: first_allocation.entry_bar_index,
            entry_time: first_allocation.entry_time,
            entry_commission: allocations
                .iter()
                .map(|allocation| allocation.entry_commission)
                .sum(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ClosedTradeFill {
    entry_id: String,
    exit_id: String,
    entry_fill: AllocatedEntryFill,
    exit_bar_index: usize,
    exit_time: i64,
    exit_price: f64,
    qty: f64,
    profit: f64,
    commission: f64,
}

impl BrokerState {
    pub(super) fn record_open_long_legacy_state(&mut self, trade: &OpenTrade) {
        self.position_size = trade.quantity;
        self.max_contracts_held_long = self.max_contracts_held_long.max(trade.quantity);
        self.avg_price = trade.entry_price;
        self.open_entry_commission = trade.entry_commission;
        self.cash -= trade.quantity * trade.entry_price + trade.entry_commission;
        self.entry_id = Some(trade.id.clone());
        self.entry_bar_index = Some(trade.entry_bar_index);
        self.entry_time = Some(trade.entry_time);
        self.open_trade_max_high = trade.max_high;
        self.open_trade_min_low = trade.min_low;
        self.open_trade_equity_on_entry = trade.equity_on_entry;
        self.open_trade_min_equity_before_entry = trade.min_equity_before_entry;
        self.open_trade_max_equity_before_entry = trade.max_equity_before_entry;
    }

    fn clear_open_long_legacy_state(&mut self) {
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
    }

    pub(super) fn record_position_snapshot(&mut self, bar_index: usize) {
        self.position.push(crate::StrategyPositionSnapshot {
            bar_index,
            size: self.position_size,
            avg_price: (self.position_size > 0.0).then_some(self.avg_price),
        });
    }

    pub(super) fn record_order_event(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        direction: &str,
        qty: f64,
        price: f64,
    ) {
        self.orders.push(StrategyOrderEvent {
            id,
            bar_index,
            time,
            direction: direction.to_owned(),
            qty,
            price,
        });
    }

    fn record_closed_trade_fill(&mut self, fill: ClosedTradeFill) {
        self.trades.push(StrategyTrade {
            id: fill.entry_id,
            exit_id: fill.exit_id,
            entry_bar_index: fill.entry_fill.entry_bar_index,
            exit_bar_index: fill.exit_bar_index,
            entry_time: fill.entry_fill.entry_time,
            exit_time: fill.exit_time,
            entry_price: fill.entry_fill.entry_price,
            exit_price: fill.exit_price,
            qty: fill.qty,
            profit: fill.profit,
        });
        self.closed_trade_metrics.push(ClosedTradeMetrics {
            commission: fill.commission,
            profit_percent: closed_trade_profit_percent(
                fill.entry_fill.entry_price,
                fill.qty,
                fill.profit,
            ),
            max_runup: self.current_open_trade_max_runup_for_quantity(fill.qty),
            max_drawdown: self.current_open_trade_max_drawdown_for_quantity(fill.qty),
        });
    }

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
        let allocations = self.trade_ledger.allocate_exit_fifo(None, qty);
        let entry_fill = AllocatedEntryFill::from_allocations(
            &allocations,
            self.avg_price,
            self.entry_bar_index.unwrap_or(bar_index),
            self.entry_time.unwrap_or(time),
            self.entry_commission_for_closed_quantity(qty),
        );
        let exit_commission = self.exit_commission_for_fill(qty, current_price);
        let commission = entry_fill.entry_commission + exit_commission;
        let profit = (current_price - entry_fill.entry_price) * qty - commission;

        self.order_book.exits_mut().clear_for_entry(&entry_id);
        self.record_order_event(
            exit_id.clone(),
            bar_index,
            time,
            "strategy.short",
            qty,
            current_price,
        );
        self.record_closed_trade_fill(ClosedTradeFill {
            entry_id,
            exit_id,
            entry_fill,
            exit_bar_index: bar_index,
            exit_time: time,
            exit_price: current_price,
            qty,
            profit,
            commission,
        });

        self.cash += qty * current_price - exit_commission;
        if qty >= self.position_size {
            self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
            self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
            self.clear_open_long_legacy_state();
            self.apply_trade_allocations_and_sync_position(&allocations);
            if allocations.is_empty() {
                self.trade_ledger.clear_open_trade();
                self.sync_aggregate_position_from_ledger();
            }
            self.record_position_snapshot(bar_index);
            return;
        }

        self.open_entry_commission -= entry_fill.entry_commission;
        self.apply_trade_allocations_and_sync_position(&allocations);
        self.record_position_snapshot(bar_index);
    }

    pub(crate) fn close_all_long(&mut self, bar_index: usize, time: i64, price: f64) {
        if self.position_size <= 0.0 {
            return;
        }
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.close_all` fill price must be finite".to_owned(),
            });
            return;
        }

        let price = self.long_exit_fill_price(price);
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.close_all` slipped fill price must be finite".to_owned(),
            });
            return;
        }

        let qty = self.position_size;
        let allocations = self.trade_ledger.allocate_exit_fifo(None, qty);
        if allocations.is_empty() {
            return;
        }

        let exit_commission = self.exit_commission_for_fill(qty, price);
        for allocation in &allocations {
            let allocated_exit_commission = exit_commission * (allocation.quantity / qty);
            let commission = allocation.entry_commission + allocated_exit_commission;
            let profit = (price - allocation.entry_price) * allocation.quantity - commission;
            self.record_closed_trade_fill(ClosedTradeFill {
                entry_id: allocation.entry_id.clone(),
                exit_id: allocation.entry_id.clone(),
                entry_fill: AllocatedEntryFill {
                    entry_price: allocation.entry_price,
                    entry_bar_index: allocation.entry_bar_index,
                    entry_time: allocation.entry_time,
                    entry_commission: allocation.entry_commission,
                },
                exit_bar_index: bar_index,
                exit_time: time,
                exit_price: price,
                qty: allocation.quantity,
                profit,
                commission,
            });
        }

        self.cash += qty * price - exit_commission;
        self.order_book.exits_mut().clear_all();
        self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
        self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
        self.clear_open_long_legacy_state();
        self.apply_trade_allocations_and_sync_position(&allocations);
        self.record_position_snapshot(bar_index);
    }

    pub(crate) fn close_long(&mut self, id: String, bar_index: usize, time: i64, price: f64) {
        self.close_long_quantity(id, bar_index, time, price, None);
    }

    pub(crate) fn close_long_qty(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) {
        self.close_long_quantity(id, bar_index, time, price, Some(qty));
    }

    pub(crate) fn close_long_qty_percent(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty_percent: f64,
    ) {
        let matching_position_size = self.trade_ledger.open_quantity_for_entry(&id);
        if self.position_size <= 0.0 || matching_position_size <= 0.0 {
            return;
        }
        if !qty_percent.is_finite() || qty_percent <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_CLOSE_QTY_PERCENT".to_owned(),
                message: "`strategy.close` percent quantity must be finite and positive".to_owned(),
            });
            return;
        }

        let qty = matching_position_size * qty_percent / 100.0;
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_CLOSE_QTY_PERCENT".to_owned(),
                message: "`strategy.close` percent quantity must be finite and positive".to_owned(),
            });
            return;
        }
        self.close_long_quantity(id, bar_index, time, price, Some(qty));
    }

    fn close_long_quantity(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        requested_qty: Option<f64>,
    ) {
        let matching_position_size = self.trade_ledger.open_quantity_for_entry(&id);
        if self.position_size <= 0.0 || matching_position_size <= 0.0 {
            return;
        }
        if let Some(requested_qty) = requested_qty
            && (!requested_qty.is_finite() || requested_qty <= 0.0)
        {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_CLOSE_QTY".to_owned(),
                message: "`strategy.close` quantity must be finite and positive".to_owned(),
            });
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

        let qty = requested_qty.map_or(matching_position_size, |qty| {
            qty.min(matching_position_size)
        });
        let allocations = self.trade_ledger.allocate_exit_fifo(Some(&id), qty);
        let entry_fill = AllocatedEntryFill::from_allocations(
            &allocations,
            self.avg_price,
            self.entry_bar_index.unwrap_or(bar_index),
            self.entry_time.unwrap_or(time),
            self.entry_commission_for_closed_quantity(qty),
        );
        let exit_commission = self.exit_commission_for_fill(qty, price);
        let commission = entry_fill.entry_commission + exit_commission;
        let profit = (price - entry_fill.entry_price) * qty - commission;
        self.record_closed_trade_fill(ClosedTradeFill {
            entry_id: id.clone(),
            exit_id: id.clone(),
            entry_fill,
            exit_bar_index: bar_index,
            exit_time: time,
            exit_price: price,
            qty,
            profit,
            commission,
        });

        self.cash += qty * price - exit_commission;
        if qty >= self.position_size {
            self.cancel_exit_for_entry(&id);
            self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
            self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
            self.clear_open_long_legacy_state();
            self.apply_trade_allocations_and_sync_position(&allocations);
            if allocations.is_empty() {
                self.trade_ledger.clear_open_trade();
                self.sync_aggregate_position_from_ledger();
            }
            self.record_position_snapshot(bar_index);
            return;
        }

        self.open_entry_commission -= entry_fill.entry_commission;
        self.apply_trade_allocations_and_sync_position(&allocations);
        self.record_position_snapshot(bar_index);
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
        let allocations = self
            .trade_ledger
            .allocate_exit_fifo(Some(&pending_exit.from_entry), qty);
        let entry_fill = AllocatedEntryFill::from_allocations(
            &allocations,
            self.avg_price,
            self.entry_bar_index.unwrap_or(bar_index),
            self.entry_time.unwrap_or(time),
            self.entry_commission_for_closed_quantity(qty),
        );
        let exit_commission = self.exit_commission_for_fill(qty, exit_price);
        let commission = entry_fill.entry_commission + exit_commission;
        let profit = (exit_price - entry_fill.entry_price) * qty - commission;
        let exit_id = pending_exit.id;
        let entry_id = pending_exit.from_entry;

        self.record_order_event(
            exit_id.clone(),
            bar_index,
            time,
            "strategy.exit",
            qty,
            exit_price,
        );
        self.record_closed_trade_fill(ClosedTradeFill {
            entry_id,
            exit_id,
            entry_fill,
            exit_bar_index: bar_index,
            exit_time: time,
            exit_price,
            qty,
            profit,
            commission,
        });

        self.cash += qty * exit_price - exit_commission;
        if qty >= self.position_size {
            self.min_equity_before_open_trade = self.min_equity_before_open_trade.min(self.cash);
            self.max_equity_before_open_trade = self.max_equity_before_open_trade.max(self.cash);
            self.clear_open_long_legacy_state();
            self.apply_trade_allocations_and_sync_position(&allocations);
            if allocations.is_empty() {
                self.trade_ledger.clear_open_trade();
                self.sync_aggregate_position_from_ledger();
            }
            self.record_position_snapshot(bar_index);
            return;
        }

        self.open_entry_commission -= entry_fill.entry_commission;
        self.apply_trade_allocations_and_sync_position(&allocations);
        self.record_position_snapshot(bar_index);
    }
}
