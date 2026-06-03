use super::{BrokerState, exits::PendingExit};
use crate::{RuntimeDiagnostic, StrategyOrderEvent, StrategyPositionSnapshot, StrategyTrade};

impl BrokerState {
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

        let qty = self.position_size;
        let entry_price = self.avg_price;
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
        let entry_price = self.avg_price;
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
            profit: (exit_price - entry_price) * qty,
        });

        self.cash += qty * exit_price;
        if qty >= self.position_size {
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
            return;
        }

        self.position_size -= qty;
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: self.position_size,
            avg_price: Some(self.avg_price),
        });
    }
}
