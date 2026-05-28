use crate::{RuntimeDiagnostic, StrategyOrderEvent, StrategyPositionSnapshot, StrategyResult};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BrokerState {
    position_size: f64,
    avg_price: f64,
    orders: Vec<StrategyOrderEvent>,
    position: Vec<StrategyPositionSnapshot>,
    diagnostics: Vec<RuntimeDiagnostic>,
}

impl BrokerState {
    pub(crate) fn entry_long(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) {
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_QTY".to_owned(),
                message: "`strategy.entry` quantity must be positive".to_owned(),
            });
            return;
        }
        if self.position_size > 0.0 {
            return;
        }

        self.position_size = qty;
        self.avg_price = price;
        self.orders.push(StrategyOrderEvent {
            id,
            bar_index,
            time,
            direction: "strategy.long".to_owned(),
            qty,
            price,
        });
        self.position.push(StrategyPositionSnapshot {
            bar_index,
            size: qty,
            avg_price: price,
        });
    }

    #[must_use]
    pub fn result(&self) -> StrategyResult {
        StrategyResult {
            orders: self.orders.clone(),
            trades: Vec::new(),
            position: self.position.clone(),
            equity: Vec::new(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}
