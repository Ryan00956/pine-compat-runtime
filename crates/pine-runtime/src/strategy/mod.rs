use crate::StrategyResult;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrokerState;

impl BrokerState {
    #[must_use]
    pub fn empty_result(&self) -> StrategyResult {
        StrategyResult::default()
    }
}
