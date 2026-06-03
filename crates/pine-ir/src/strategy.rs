pub const DEFAULT_STRATEGY_INITIAL_CAPITAL: f64 = 100_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyDefaultQuantity {
    Fixed(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyCommission {
    CashPerContract(f64),
    CashPerOrder(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrategySettings {
    pub initial_capital: f64,
    pub default_qty: Option<StrategyDefaultQuantity>,
    pub commission: Option<StrategyCommission>,
    pub slippage_ticks: f64,
}

impl Default for StrategySettings {
    fn default() -> Self {
        Self {
            initial_capital: DEFAULT_STRATEGY_INITIAL_CAPITAL,
            default_qty: Some(StrategyDefaultQuantity::Fixed(1.0)),
            commission: None,
            slippage_ticks: 0.0,
        }
    }
}

impl StrategySettings {
    #[must_use]
    pub fn default_entry_qty(self) -> Option<f64> {
        self.default_qty
            .map(|StrategyDefaultQuantity::Fixed(qty)| qty)
    }

    #[must_use]
    pub fn commission_cash_per_contract(self) -> f64 {
        match self.commission {
            Some(StrategyCommission::CashPerContract(value)) => value,
            Some(StrategyCommission::CashPerOrder(_)) | None => 0.0,
        }
    }
}
