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

    pub(crate) fn update_open_trade_extremes(&mut self, high: f64, low: f64) {
        if self.open_trade_count() != 1 {
            return;
        }
        if high.is_finite() {
            self.open_trade_max_high = Some(
                self.open_trade_max_high
                    .map_or(high, |current| current.max(high)),
            );
        }
        if low.is_finite() {
            self.open_trade_min_low = Some(
                self.open_trade_min_low
                    .map_or(low, |current| current.min(low)),
            );
        }
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
    pub(crate) fn gross_profit(&self) -> f64 {
        normalize_zero(
            self.trades
                .iter()
                .filter(|trade| trade.profit > 0.0)
                .map(|trade| trade.profit)
                .sum(),
        )
    }

    #[must_use]
    pub(crate) fn gross_loss(&self) -> f64 {
        normalize_zero(
            self.trades
                .iter()
                .filter(|trade| trade.profit < 0.0)
                .map(|trade| -trade.profit)
                .sum(),
        )
    }

    #[must_use]
    pub(crate) fn average_trade(&self) -> Option<f64> {
        if self.trades.is_empty() {
            None
        } else {
            Some(normalize_zero(
                self.realized_profit() / self.trades.len() as f64,
            ))
        }
    }

    #[must_use]
    pub(crate) fn average_winning_trade(&self) -> Option<f64> {
        let mut count = 0usize;
        let mut total = 0.0;
        for trade in &self.trades {
            if trade.profit > 0.0 {
                count += 1;
                total += trade.profit;
            }
        }
        if count == 0 {
            None
        } else {
            Some(normalize_zero(total / count as f64))
        }
    }

    #[must_use]
    pub(crate) fn average_losing_trade(&self) -> Option<f64> {
        let mut count = 0usize;
        let mut total = 0.0;
        for trade in &self.trades {
            if trade.profit < 0.0 {
                count += 1;
                total += -trade.profit;
            }
        }
        if count == 0 {
            None
        } else {
            Some(normalize_zero(total / count as f64))
        }
    }

    #[must_use]
    pub(crate) fn max_drawdown(&self, current_equity: f64) -> f64 {
        let mut peak = self.initial_capital;
        let mut max_drawdown = 0.0;
        for equity in self
            .equity
            .iter()
            .map(|snapshot| snapshot.equity)
            .chain(std::iter::once(current_equity))
        {
            peak = peak.max(equity);
            max_drawdown = f64::max(max_drawdown, peak - equity);
        }
        normalize_zero(max_drawdown)
    }

    #[must_use]
    pub(crate) fn equity_value(&self, close: f64) -> f64 {
        normalize_zero(self.cash + self.position_size * close)
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
    pub(crate) fn closed_trade_max_runup(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_runup)
    }

    #[must_use]
    pub(crate) fn closed_trade_commission(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.commission)
    }

    #[must_use]
    pub(crate) fn closed_trade_max_drawdown(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_drawdown)
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
    pub(crate) fn open_trade_entry_id(&self, trade_num: i64) -> Option<&str> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            self.entry_id.as_deref()
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_entry_bar_index(&self, trade_num: i64) -> Option<usize> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            self.entry_bar_index
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_entry_time(&self, trade_num: i64) -> Option<i64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            self.entry_time
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_size(&self, trade_num: i64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            Some(self.position_size)
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_profit(&self, trade_num: i64, close: f64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            Some(self.open_profit(close))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_commission(&self, trade_num: i64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            Some(normalize_zero(self.open_entry_commission))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_max_runup(&self, trade_num: i64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            let max_high = self.open_trade_max_high?;
            Some(normalize_zero(
                (max_high - self.avg_price).max(0.0) * self.position_size,
            ))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn open_trade_max_drawdown(&self, trade_num: i64) -> Option<f64> {
        if trade_num == 0 && self.open_trade_count() == 1 {
            let min_low = self.open_trade_min_low?;
            Some(normalize_zero(
                (self.avg_price - min_low).max(0.0) * self.position_size,
            ))
        } else {
            None
        }
    }

    #[must_use]
    pub(super) fn current_open_trade_max_runup_for_quantity(&self, qty: f64) -> f64 {
        let Some(max_high) = self.open_trade_max_high else {
            return 0.0;
        };
        normalize_zero((max_high - self.avg_price).max(0.0) * qty)
    }

    #[must_use]
    pub(super) fn current_open_trade_max_drawdown_for_quantity(&self, qty: f64) -> f64 {
        let Some(min_low) = self.open_trade_min_low else {
            return 0.0;
        };
        normalize_zero((self.avg_price - min_low).max(0.0) * qty)
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
