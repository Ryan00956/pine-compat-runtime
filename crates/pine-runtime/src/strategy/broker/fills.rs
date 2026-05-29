use super::{BrokerState, exits::PendingExit};
use crate::{StrategyOrderEvent, StrategyPositionSnapshot, StrategyTrade};

impl BrokerState {
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

    pub(super) fn fill_pending_exit(
        &mut self,
        pending_exit: PendingExit,
        bar_index: usize,
        time: i64,
    ) {
        let qty = self.position_size;
        let entry_price = self.avg_price;
        let entry_bar_index = self.entry_bar_index.unwrap_or(bar_index);
        let entry_time = self.entry_time.unwrap_or(time);
        let exit_price = pending_exit.trigger.price();

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
}
