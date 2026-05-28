use super::model::RuntimeDiagnostic;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StrategyResult {
    pub orders: Vec<StrategyOrderEvent>,
    pub trades: Vec<StrategyTrade>,
    pub position: Vec<StrategyPositionSnapshot>,
    pub equity: Vec<StrategyEquitySnapshot>,
    pub diagnostics: Vec<RuntimeDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrategyOrderEvent {
    pub id: String,
    pub bar_index: usize,
    pub time: i64,
    pub direction: String,
    pub qty: f64,
    pub price: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyTrade;

#[derive(Debug, Clone, PartialEq)]
pub struct StrategyPositionSnapshot {
    pub bar_index: usize,
    pub size: f64,
    pub avg_price: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyEquitySnapshot;
