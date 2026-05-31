use super::BrokerState;
use crate::RuntimeDiagnostic;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingExitTrigger {
    Stop(f64),
    Limit(f64),
    Bracket { downside: f64, upside: f64 },
    Trailing(PendingTrailingExit),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum PendingExitQuantity {
    Full,
    Fixed(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingTrailingExit {
    pub(super) spec: PendingTrailingSpec,
    pub(super) state: PendingTrailingState,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingTrailingSpec {
    pub(super) activation: PendingTrailingActivation,
    pub(super) offset_price_distance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingTrailingActivation {
    Price(f64),
    Points { ticks: f64, price: f64 },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingTrailingState {
    Inactive,
    Active { stop_price: f64 },
}

impl PendingTrailingActivation {
    pub(super) fn price(&self) -> f64 {
        match self {
            Self::Price(price) | Self::Points { price, .. } => *price,
        }
    }
}

impl PendingExitTrigger {
    fn prices_are_finite(&self) -> bool {
        match self {
            Self::Stop(price) | Self::Limit(price) => price.is_finite(),
            Self::Bracket { downside, upside } => downside.is_finite() && upside.is_finite(),
            Self::Trailing(trailing) => {
                trailing.spec.activation.price().is_finite()
                    && trailing.spec.offset_price_distance.is_finite()
            }
        }
    }

    fn placement_equivalent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Trailing(left), Self::Trailing(right)) => left.spec == right.spec,
            _ => self == other,
        }
    }
}

impl PendingExitQuantity {
    fn is_valid(self) -> bool {
        match self {
            Self::Full => true,
            Self::Fixed(qty) => qty.is_finite() && qty > 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingExit {
    pub(super) id: String,
    pub(super) from_entry: String,
    pub(super) trigger: PendingExitTrigger,
    pub(super) quantity: PendingExitQuantity,
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
        self.place_exit_stop_quantity(
            id,
            from_entry,
            stop_price,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_stop_qty(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_stop_quantity(
            id,
            from_entry,
            stop_price,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_stop_quantity(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Stop(stop_price),
            quantity,
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
        self.place_exit_limit_quantity(
            id,
            from_entry,
            limit_price,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_limit_qty(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_limit_quantity(
            id,
            from_entry,
            limit_price,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_limit_quantity(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Limit(limit_price),
            quantity,
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
        self.place_exit_profit_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_profit_ticks_qty(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_profit_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_profit_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        let Some(limit_price) = self.exit_profit_price_from_ticks(ticks, mintick) else {
            return;
        };
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Limit(limit_price),
            quantity,
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
        self.place_exit_loss_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_loss_ticks_qty(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_loss_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_loss_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        let Some(stop_price) = self.exit_loss_price_from_ticks(ticks, mintick) else {
            return;
        };
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Stop(stop_price),
            quantity,
            bar_index,
        );
    }

    pub(crate) fn place_exit_bracket(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        bar_index: usize,
    ) {
        self.place_exit_bracket_quantity(
            id,
            from_entry,
            downside_price,
            upside_price,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_bracket_qty(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_bracket_quantity(
            id,
            from_entry,
            downside_price,
            upside_price,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Bracket {
                downside: downside_price,
                upside: upside_price,
            },
            quantity,
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_price(
        &mut self,
        id: String,
        from_entry: String,
        activation_price: f64,
        offset_ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_price_quantity(
            id,
            from_entry,
            activation_price,
            offset_ticks,
            mintick,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_trail_price_qty(
        &mut self,
        id: String,
        from_entry: String,
        activation_price: f64,
        offset_ticks: f64,
        mintick: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_price_quantity(
            id,
            from_entry,
            activation_price,
            offset_ticks,
            mintick,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_trail_price_quantity(
        &mut self,
        id: String,
        from_entry: String,
        activation_price: f64,
        offset_ticks: f64,
        mintick: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        let Some(offset_price_distance) = self.exit_tick_price_offset(offset_ticks, mintick) else {
            return;
        };
        self.place_exit_trailing(
            id,
            from_entry,
            PendingTrailingActivation::Price(activation_price),
            offset_price_distance,
            quantity,
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_points(
        &mut self,
        id: String,
        from_entry: String,
        activation_ticks: f64,
        offset_ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_points_quantity(
            id,
            from_entry,
            activation_ticks,
            offset_ticks,
            mintick,
            PendingExitQuantity::Full,
            bar_index,
        );
    }

    #[cfg(test)]
    pub(crate) fn place_exit_trail_points_qty(
        &mut self,
        id: String,
        from_entry: String,
        activation_ticks: f64,
        offset_ticks: f64,
        mintick: f64,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_points_quantity(
            id,
            from_entry,
            activation_ticks,
            offset_ticks,
            mintick,
            PendingExitQuantity::Fixed(qty),
            bar_index,
        );
    }

    fn place_exit_trail_points_quantity(
        &mut self,
        id: String,
        from_entry: String,
        activation_ticks: f64,
        offset_ticks: f64,
        mintick: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        let Some(activation_price_offset) = self.exit_tick_price_offset(activation_ticks, mintick)
        else {
            return;
        };
        let Some(offset_price_distance) = self.exit_tick_price_offset(offset_ticks, mintick) else {
            return;
        };
        self.place_exit_trailing(
            id,
            from_entry,
            PendingTrailingActivation::Points {
                ticks: activation_ticks,
                price: self.avg_price + activation_price_offset,
            },
            offset_price_distance,
            quantity,
            bar_index,
        );
    }

    fn place_exit_trailing(
        &mut self,
        id: String,
        from_entry: String,
        activation: PendingTrailingActivation,
        offset_price_distance: f64,
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        self.place_exit(
            id,
            from_entry,
            PendingExitTrigger::Trailing(PendingTrailingExit {
                spec: PendingTrailingSpec {
                    activation,
                    offset_price_distance,
                },
                state: PendingTrailingState::Inactive,
            }),
            quantity,
            bar_index,
        );
    }

    pub(crate) fn exit_profit_price_from_ticks(&mut self, ticks: f64, mintick: f64) -> Option<f64> {
        self.exit_tick_price_offset(ticks, mintick)
            .map(|price_offset| self.avg_price + price_offset)
    }

    pub(crate) fn exit_loss_price_from_ticks(&mut self, ticks: f64, mintick: f64) -> Option<f64> {
        self.exit_tick_price_offset(ticks, mintick)
            .map(|price_offset| self.avg_price - price_offset)
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
        quantity: PendingExitQuantity,
        bar_index: usize,
    ) {
        if !trigger.prices_are_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_PRICE".to_owned(),
                message: "`strategy.exit` price must be finite".to_owned(),
            });
            return;
        }
        if !quantity.is_valid() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_QTY".to_owned(),
                message: "`strategy.exit` quantity must be finite and positive".to_owned(),
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
                && pending_exit.trigger.placement_equivalent(&trigger)
                && pending_exit.quantity == quantity
        }) {
            return;
        }

        self.pending_exit = Some(PendingExit {
            id,
            from_entry,
            trigger,
            quantity,
            last_update_bar_index: bar_index,
        });
    }
}
