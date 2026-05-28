use super::model::RuntimeDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StrategyResult {
    pub orders: Vec<StrategyOrderEvent>,
    pub trades: Vec<StrategyTrade>,
    pub position: Vec<StrategyPositionSnapshot>,
    pub equity: Vec<StrategyEquitySnapshot>,
    pub diagnostics: Vec<RuntimeDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyOrderEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyTrade;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyPositionSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyEquitySnapshot;
