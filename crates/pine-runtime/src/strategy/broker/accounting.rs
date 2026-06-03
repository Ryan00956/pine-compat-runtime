use super::BrokerState;
use crate::{PineValue, StrategyEquitySnapshot, StrategyTrade};

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

impl BrokerState {
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
    pub fn closed_trade_count(&self) -> i64 {
        i64::try_from(self.trades.len()).unwrap_or(i64::MAX)
    }

    #[must_use]
    pub(crate) fn closed_trade(&self, trade_num: i64) -> Option<&StrategyTrade> {
        let index = usize::try_from(trade_num).ok()?;
        self.trades.get(index)
    }

    #[must_use]
    pub fn winning_trade_count(&self) -> i64 {
        i64::try_from(
            self.trades
                .iter()
                .filter(|trade| trade.profit > 0.0)
                .count(),
        )
        .unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn losing_trade_count(&self) -> i64 {
        i64::try_from(
            self.trades
                .iter()
                .filter(|trade| trade.profit < 0.0)
                .count(),
        )
        .unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn even_trade_count(&self) -> i64 {
        i64::try_from(
            self.trades
                .iter()
                .filter(|trade| normalize_zero(trade.profit) == 0.0)
                .count(),
        )
        .unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn open_trade_count(&self) -> i64 {
        if self.position_size > 0.0 && self.entry_id.is_some() {
            1
        } else {
            0
        }
    }

    #[must_use]
    pub(crate) fn open_trade_entry_price(&self, trade_num: i64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            Some(self.avg_price)
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn position_avg_price_value(&self) -> PineValue {
        if self.position_size > 0.0 {
            PineValue::Float(self.avg_price)
        } else {
            PineValue::Na
        }
    }
}
