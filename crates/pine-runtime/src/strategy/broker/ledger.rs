#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TradeDirection {
    Long,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct OpenTrade {
    pub(super) id: String,
    pub(super) direction: TradeDirection,
    pub(super) quantity: f64,
    pub(super) entry_price: f64,
    pub(super) entry_bar_index: usize,
    pub(super) entry_time: i64,
    pub(super) entry_commission: f64,
    pub(super) max_high: Option<f64>,
    pub(super) min_low: Option<f64>,
    pub(super) equity_on_entry: Option<f64>,
    pub(super) min_equity_before_entry: Option<f64>,
    pub(super) max_equity_before_entry: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NetPosition {
    pub(super) signed_size: f64,
    pub(super) avg_price: f64,
}

impl Default for NetPosition {
    fn default() -> Self {
        Self {
            signed_size: 0.0,
            avg_price: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct TradeLedger {
    open_trades: Vec<OpenTrade>,
    net_position: NetPosition,
}

impl TradeLedger {
    pub(super) fn open_long(&mut self, trade: OpenTrade) {
        self.open_trades.clear();
        self.open_trades.push(trade);
        self.rebuild_net_position();
    }

    pub(super) fn reduce_open_trade(&mut self, quantity: f64, entry_commission: f64) {
        if !quantity.is_finite() || quantity <= 0.0 {
            return;
        }
        let Some(open_trade) = self.open_trades.first_mut() else {
            return;
        };
        if quantity >= open_trade.quantity {
            self.clear_open_trade();
            return;
        }

        open_trade.quantity -= quantity;
        open_trade.entry_commission -= entry_commission;
        self.rebuild_net_position();
    }

    pub(super) fn clear_open_trade(&mut self) {
        self.open_trades.clear();
        self.net_position = NetPosition::default();
    }

    pub(super) fn update_extremes(&mut self, high: f64, low: f64) {
        let Some(open_trade) = self.open_trades.first_mut() else {
            return;
        };
        if high.is_finite() {
            open_trade.max_high = Some(
                open_trade
                    .max_high
                    .map_or(high, |current| current.max(high)),
            );
        }
        if low.is_finite() {
            open_trade.min_low = Some(open_trade.min_low.map_or(low, |current| current.min(low)));
        }
    }

    fn rebuild_net_position(&mut self) {
        let signed_size: f64 = self.open_trades.iter().map(|trade| trade.quantity).sum();
        if signed_size <= 0.0 {
            self.net_position = NetPosition::default();
            return;
        }

        let weighted_entry_value: f64 = self
            .open_trades
            .iter()
            .map(|trade| trade.quantity * trade.entry_price)
            .sum();
        self.net_position = NetPosition {
            signed_size,
            avg_price: weighted_entry_value / signed_size,
        };
    }

    #[cfg(test)]
    pub(super) fn open_trade(&self) -> Option<&OpenTrade> {
        self.open_trades.first()
    }

    #[cfg(test)]
    pub(super) fn open_trades(&self) -> &[OpenTrade] {
        &self.open_trades
    }

    #[cfg(test)]
    pub(super) fn net_position(&self) -> NetPosition {
        self.net_position
    }
}
