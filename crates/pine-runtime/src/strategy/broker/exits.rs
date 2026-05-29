use super::BrokerState;
use crate::RuntimeDiagnostic;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingExitTrigger {
    Stop(f64),
    Limit(f64),
}

impl PendingExitTrigger {
    pub(super) fn price(&self) -> f64 {
        match self {
            Self::Stop(price) | Self::Limit(price) => *price,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingExit {
    pub(super) id: String,
    pub(super) from_entry: String,
    pub(super) trigger: PendingExitTrigger,
    pub(super) last_update_bar_index: usize,
}

impl BrokerState {
    pub(crate) fn place_exit_stop(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Stop(stop_price),
            bar_index,
        );
    }

    pub(crate) fn place_exit_limit(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Limit(limit_price),
            bar_index,
        );
    }

    pub(crate) fn place_exit_profit_ticks(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        let Some(price_offset) = self.exit_tick_price_offset(ticks, mintick) else {
            return;
        };
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Limit(self.avg_price + price_offset),
            bar_index,
        );
    }

    pub(crate) fn place_exit_loss_ticks(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        let Some(price_offset) = self.exit_tick_price_offset(ticks, mintick) else {
            return;
        };
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Stop(self.avg_price - price_offset),
            bar_index,
        );
    }

    fn exit_tick_price_offset(&mut self, ticks: f64, mintick: f64) -> Option<f64> {
        if !ticks.is_finite() || ticks <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_TICKS".to_owned(),
                message: "`strategy.exit` tick distance must be finite and positive".to_owned(),
            });
            return None;
        }
        if !mintick.is_finite() || mintick <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_MINTICK".to_owned(),
                message: "`strategy.exit` mintick must be finite and positive".to_owned(),
            });
            return None;
        }
        Some(ticks * mintick)
    }

    fn place_exit(
        &mut self,
        id: String,
        from_entry: String,
        trigger: PendingExitTrigger,
        bar_index: usize,
    ) {
        if !trigger.price().is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_PRICE".to_owned(),
                message: "`strategy.exit` price must be finite".to_owned(),
            });
            return;
        }
        if self.position_size <= 0.0 || self.entry_id.as_deref() != Some(from_entry.as_str()) {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_ENTRY".to_owned(),
                message: "`strategy.exit` from_entry must match the current long entry".to_owned(),
            });
            return;
        }

        if self.pending_exit.as_ref().is_some_and(|pending_exit| {
            pending_exit.id == id
                && pending_exit.from_entry == from_entry
                && pending_exit.trigger == trigger
        }) {
            return;
        }

        self.pending_exit = Some(PendingExit {
            id,
            from_entry,
            trigger,
            last_update_bar_index: bar_index,
        });
    }
}
