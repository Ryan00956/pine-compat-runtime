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
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DeferredRelativeExitTrigger {
    ProfitTicks {
        ticks: f64,
        mintick: f64,
    },
    LossTicks {
        ticks: f64,
        mintick: f64,
    },
    TrailPoints {
        activation_ticks: f64,
        offset_ticks: f64,
        mintick: f64,
    },
    Bracket {
        downside: DeferredBracketLeg,
        upside: DeferredBracketLeg,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DeferredBracketLeg {
    Absolute(f64),
    RelativeProfit { ticks: f64, mintick: f64 },
    RelativeLoss { ticks: f64, mintick: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum PendingExitQuantity {
    Full,
    Fixed(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ExitQuantityRequest {
    Full,
    Fixed(f64),
    Percent(f64),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TrailPriceExitSpec {
    pub(crate) activation_price: f64,
    pub(crate) offset_ticks: f64,
    pub(crate) mintick: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TrailPointsExitSpec {
    pub(crate) activation_ticks: f64,
    pub(crate) offset_ticks: f64,
    pub(crate) mintick: f64,
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

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingTrailingUpdate {
    NoChange,
    Persist(PendingTrailingExit),
    Candidate(PendingExitTouch),
}

impl PendingTrailingActivation {
    pub(super) fn price(&self) -> f64 {
        match self {
            Self::Price(price) | Self::Points { price, .. } => *price,
        }
    }
}

impl PendingTrailingExit {
    pub(super) fn evaluate_update(&self, high: f64, low: f64) -> PendingTrailingUpdate {
        match self.state {
            PendingTrailingState::Inactive => {
                if high >= self.spec.activation.price() {
                    return PendingTrailingUpdate::Persist(Self {
                        spec: self.spec.clone(),
                        state: PendingTrailingState::Active {
                            stop_price: high - self.spec.offset_price_distance,
                        },
                    });
                }
                PendingTrailingUpdate::NoChange
            }
            PendingTrailingState::Active { stop_price } => {
                if low <= stop_price {
                    return PendingTrailingUpdate::Candidate(PendingExitTouch {
                        exit_price: stop_price,
                        side: PendingExitSide::Stop,
                    });
                }

                let next_stop = high - self.spec.offset_price_distance;
                if next_stop > stop_price {
                    PendingTrailingUpdate::Persist(Self {
                        spec: self.spec.clone(),
                        state: PendingTrailingState::Active {
                            stop_price: next_stop,
                        },
                    })
                } else {
                    PendingTrailingUpdate::NoChange
                }
            }
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

    pub(super) fn single_trigger_side(&self) -> Option<PendingExitSide> {
        match self {
            Self::Stop(_) => Some(PendingExitSide::Stop),
            Self::Limit(_) => Some(PendingExitSide::Limit),
            Self::Bracket { .. } | Self::Trailing(_) => None,
        }
    }

    pub(super) fn reservation_family(&self) -> PendingExitReservationFamily {
        if self.single_trigger_side().is_some() {
            return PendingExitReservationFamily::SingleTrigger;
        }
        if self.is_trailing_reservation_candidate() {
            return PendingExitReservationFamily::Trailing;
        }
        match self {
            Self::Bracket { .. } => PendingExitReservationFamily::Bracket,
            Self::Stop(_) | Self::Limit(_) | Self::Trailing(_) => {
                unreachable!("single and trailing triggers returned above")
            }
        }
    }

    pub(super) fn is_trailing_reservation_candidate(&self) -> bool {
        matches!(self, Self::Trailing(_))
    }

    pub(super) fn touched_candidate(
        &self,
        high: f64,
        low: f64,
        limit_verification_offset: f64,
    ) -> Option<PendingExitTouch> {
        match self {
            Self::Stop(price) if low <= *price => Some(PendingExitTouch {
                exit_price: *price,
                side: PendingExitSide::Stop,
            }),
            Self::Limit(price) if high >= *price + limit_verification_offset => {
                Some(PendingExitTouch {
                    exit_price: *price,
                    side: PendingExitSide::Limit,
                })
            }
            Self::Bracket { downside, upside } => {
                if low <= *downside {
                    Some(PendingExitTouch {
                        exit_price: *downside,
                        side: PendingExitSide::Stop,
                    })
                } else if high >= *upside + limit_verification_offset {
                    Some(PendingExitTouch {
                        exit_price: *upside,
                        side: PendingExitSide::Limit,
                    })
                } else {
                    None
                }
            }
            Self::Trailing(_) | Self::Stop(_) | Self::Limit(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PendingExitReservationFamily {
    SingleTrigger,
    Bracket,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PendingExitSide {
    Stop,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PendingExitTouch {
    pub(super) exit_price: f64,
    pub(super) side: PendingExitSide,
}

impl PendingExitQuantity {
    fn is_valid(self) -> bool {
        match self {
            Self::Full => true,
            Self::Fixed(qty) => qty.is_finite() && qty > 0.0,
        }
    }
}

impl ExitQuantityRequest {
    fn has_invalid_fixed_quantity(self) -> bool {
        matches!(self, Self::Fixed(qty) if !PendingExitQuantity::Fixed(qty).is_valid())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingExit {
    pub(super) id: String,
    pub(super) from_entry: String,
    pub(super) trigger: PendingExitTrigger,
    pub(super) quantity: PendingExitQuantity,
    pub(super) reserved_quantity: f64,
    pub(super) multiple_reservation: bool,
    pub(super) last_update_bar_index: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(super) struct DeferredRelativeExit {
    pub(super) id: String,
    pub(super) from_entry: String,
    pub(super) trigger: DeferredRelativeExitTrigger,
    pub(super) quantity: ExitQuantityRequest,
    pub(super) last_update_bar_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct PendingExitBook {
    exits: Vec<PendingExit>,
    deferred_relative_exits: Vec<DeferredRelativeExit>,
}

impl PendingExitBook {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn current(&self) -> Option<&PendingExit> {
        self.exits.first()
    }

    #[allow(dead_code)]
    pub(super) fn current_mut(&mut self) -> Option<&mut PendingExit> {
        self.exits.first_mut()
    }

    #[allow(dead_code)]
    pub(super) fn iter(&self) -> impl Iterator<Item = &PendingExit> {
        self.exits.iter()
    }

    #[allow(dead_code)]
    pub(super) fn count(&self) -> usize {
        self.exits.len()
    }

    #[allow(dead_code)]
    pub(super) fn deferred_relative_count(&self) -> usize {
        self.deferred_relative_exits.len()
    }

    #[allow(dead_code)]
    pub(super) fn find_by_identity(&self, id: &str, from_entry: &str) -> Option<&PendingExit> {
        self.exits
            .iter()
            .find(|pending_exit| pending_exit.id == id && pending_exit.from_entry == from_entry)
    }

    #[allow(dead_code)]
    pub(super) fn find_deferred_relative_by_identity(
        &self,
        id: &str,
        from_entry: &str,
    ) -> Option<&DeferredRelativeExit> {
        self.deferred_relative_exits
            .iter()
            .find(|pending_exit| pending_exit.id == id && pending_exit.from_entry == from_entry)
    }

    pub(super) fn other_exits_are_supported_reservations(
        &self,
        from_entry: &str,
        released_identity: Option<(&str, &str)>,
    ) -> bool {
        self.exits
            .iter()
            .filter(|pending_exit| pending_exit.from_entry == from_entry)
            .filter(|pending_exit| {
                released_identity.is_none_or(|(id, from_entry)| {
                    pending_exit.id != id || pending_exit.from_entry != from_entry
                })
            })
            .all(|pending_exit| {
                pending_exit.multiple_reservation
                    && matches!(
                        pending_exit.trigger.reservation_family(),
                        PendingExitReservationFamily::SingleTrigger
                            | PendingExitReservationFamily::Bracket
                            | PendingExitReservationFamily::Trailing
                    )
            })
    }

    #[allow(dead_code)]
    pub(super) fn total_reserved_for_entry(
        &self,
        entry_id: &str,
        released_identity: Option<(&str, &str)>,
    ) -> f64 {
        self.exits
            .iter()
            .filter(|pending_exit| pending_exit.from_entry == entry_id)
            .filter(|pending_exit| {
                released_identity.is_none_or(|(id, from_entry)| {
                    pending_exit.id != id || pending_exit.from_entry != from_entry
                })
            })
            .map(|pending_exit| pending_exit.reserved_quantity)
            .filter(|reserved_quantity| reserved_quantity.is_finite() && *reserved_quantity > 0.0)
            .sum()
    }

    #[allow(dead_code)]
    pub(super) fn available_unreserved_quantity(
        &self,
        position_size: f64,
        entry_id: &str,
        released_identity: Option<(&str, &str)>,
    ) -> f64 {
        if !position_size.is_finite() || position_size <= 0.0 {
            return 0.0;
        }
        (position_size - self.total_reserved_for_entry(entry_id, released_identity)).max(0.0)
    }

    pub(super) fn replace_all(&mut self, pending_exit: PendingExit) {
        self.exits.clear();
        self.exits.push(pending_exit);
    }

    pub(super) fn replace_or_append(&mut self, pending_exit: PendingExit) {
        if let Some(existing) = self.exits.iter_mut().find(|existing| {
            existing.id == pending_exit.id && existing.from_entry == pending_exit.from_entry
        }) {
            *existing = pending_exit;
            return;
        }
        self.exits.push(pending_exit);
    }

    #[allow(dead_code)]
    pub(super) fn replace_or_append_deferred_relative(
        &mut self,
        pending_exit: DeferredRelativeExit,
    ) {
        if let Some(existing) = self.deferred_relative_exits.iter_mut().find(|existing| {
            existing.id == pending_exit.id && existing.from_entry == pending_exit.from_entry
        }) {
            *existing = pending_exit;
            return;
        }
        self.deferred_relative_exits.push(pending_exit);
    }

    pub(super) fn take_deferred_relative_for_entry(
        &mut self,
        entry_id: &str,
    ) -> Vec<DeferredRelativeExit> {
        let mut matching = Vec::new();
        let mut retained = Vec::new();
        for pending_exit in self.deferred_relative_exits.drain(..) {
            if pending_exit.from_entry == entry_id {
                matching.push(pending_exit);
            } else {
                retained.push(pending_exit);
            }
        }
        self.deferred_relative_exits = retained;
        matching
    }

    pub(super) fn remove_identities(&mut self, identities: &[(String, String)]) {
        self.exits.retain(|pending_exit| {
            !identities.iter().any(|(id, from_entry)| {
                pending_exit.id == *id && pending_exit.from_entry == *from_entry
            })
        });
    }

    pub(super) fn cancel_id(&mut self, id: &str) {
        self.exits.retain(|pending_exit| pending_exit.id != id);
        self.deferred_relative_exits
            .retain(|pending_exit| pending_exit.id != id);
    }

    pub(super) fn clear_all(&mut self) {
        self.exits.clear();
        self.deferred_relative_exits.clear();
    }

    pub(super) fn clear_for_entry(&mut self, entry_id: &str) {
        self.exits
            .retain(|pending_exit| pending_exit.from_entry != entry_id);
        self.deferred_relative_exits
            .retain(|pending_exit| pending_exit.from_entry != entry_id);
    }
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
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

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
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_stop_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_stop_quantity(
            id,
            from_entry,
            stop_price,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_stop_quantity(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        quantity: ExitQuantityRequest,
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
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

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
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_limit_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_limit_quantity(
            id,
            from_entry,
            limit_price,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_limit_quantity(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        quantity: ExitQuantityRequest,
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
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

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
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_profit_ticks_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_profit_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_profit_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        if self.position_size <= 0.0 && self.has_pending_entry(&from_entry) {
            self.place_deferred_relative_profit_exit(
                id, from_entry, ticks, mintick, quantity, bar_index,
            );
            return;
        }
        if self.reject_entry_relative_exit_for_pending_entry(&from_entry) {
            return;
        }
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

    fn place_deferred_relative_profit_exit(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        let Some(pending_entry_quantity) = self.order_book.entries().quantity_for_id(&from_entry)
        else {
            return;
        };
        if self.exit_tick_price_offset(ticks, mintick).is_none() {
            return;
        }
        if self
            .resolve_exit_quantity_request_for_available(
                quantity,
                pending_entry_quantity,
                pending_entry_quantity,
            )
            .is_none()
        {
            return;
        }

        self.order_book
            .exits_mut()
            .replace_or_append_deferred_relative(DeferredRelativeExit {
                id,
                from_entry,
                trigger: DeferredRelativeExitTrigger::ProfitTicks { ticks, mintick },
                quantity,
                last_update_bar_index: bar_index,
            });
    }

    pub(crate) fn resolve_deferred_relative_exits_for_entry(&mut self, entry_id: &str) {
        let deferred_exits = self
            .order_book
            .exits_mut()
            .take_deferred_relative_for_entry(entry_id);
        for deferred_exit in deferred_exits {
            let DeferredRelativeExit {
                id,
                from_entry,
                trigger,
                quantity,
                last_update_bar_index,
            } = deferred_exit;
            match trigger {
                DeferredRelativeExitTrigger::ProfitTicks { ticks, mintick } => {
                    let Some(limit_price) = self.exit_profit_price_from_ticks(ticks, mintick)
                    else {
                        continue;
                    };
                    self.place_exit(
                        id,
                        from_entry,
                        PendingExitTrigger::Limit(limit_price),
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::LossTicks { ticks, mintick } => {
                    let Some(stop_price) = self.exit_loss_price_from_ticks(ticks, mintick) else {
                        continue;
                    };
                    self.place_exit(
                        id,
                        from_entry,
                        PendingExitTrigger::Stop(stop_price),
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::TrailPoints {
                    activation_ticks,
                    offset_ticks,
                    mintick,
                } => {
                    let Some(activation_price_offset) =
                        self.exit_tick_price_offset(activation_ticks, mintick)
                    else {
                        continue;
                    };
                    let Some(offset_price_distance) =
                        self.exit_tick_price_offset(offset_ticks, mintick)
                    else {
                        continue;
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
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::Bracket { .. } => {
                    continue;
                }
            }
        }
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
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

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
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_loss_ticks_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_loss_ticks_quantity(
            id,
            from_entry,
            ticks,
            mintick,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_loss_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        if self.position_size <= 0.0 && self.has_pending_entry(&from_entry) {
            self.place_deferred_relative_loss_exit(
                id, from_entry, ticks, mintick, quantity, bar_index,
            );
            return;
        }
        if self.reject_entry_relative_exit_for_pending_entry(&from_entry) {
            return;
        }
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

    fn place_deferred_relative_loss_exit(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        let Some(pending_entry_quantity) = self.order_book.entries().quantity_for_id(&from_entry)
        else {
            return;
        };
        if self.exit_tick_price_offset(ticks, mintick).is_none() {
            return;
        }
        if self
            .resolve_exit_quantity_request_for_available(
                quantity,
                pending_entry_quantity,
                pending_entry_quantity,
            )
            .is_none()
        {
            return;
        }

        self.order_book
            .exits_mut()
            .replace_or_append_deferred_relative(DeferredRelativeExit {
                id,
                from_entry,
                trigger: DeferredRelativeExitTrigger::LossTicks { ticks, mintick },
                quantity,
                last_update_bar_index: bar_index,
            });
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
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

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
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_bracket_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_bracket_quantity(
            id,
            from_entry,
            downside_price,
            upside_price,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        quantity: ExitQuantityRequest,
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
            TrailPriceExitSpec {
                activation_price,
                offset_ticks,
                mintick,
            },
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_price_qty(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPriceExitSpec,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_price_quantity(
            id,
            from_entry,
            spec,
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_price_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPriceExitSpec,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_price_quantity(
            id,
            from_entry,
            spec,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_trail_price_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPriceExitSpec,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        let Some(offset_price_distance) =
            self.exit_tick_price_offset(spec.offset_ticks, spec.mintick)
        else {
            return;
        };
        self.place_exit_trailing(
            id,
            from_entry,
            PendingTrailingActivation::Price(spec.activation_price),
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
            TrailPointsExitSpec {
                activation_ticks,
                offset_ticks,
                mintick,
            },
            ExitQuantityRequest::Full,
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_points_qty(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPointsExitSpec,
        qty: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_points_quantity(
            id,
            from_entry,
            spec,
            ExitQuantityRequest::Fixed(qty),
            bar_index,
        );
    }

    pub(crate) fn place_exit_trail_points_qty_percent(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPointsExitSpec,
        qty_percent: f64,
        bar_index: usize,
    ) {
        self.place_exit_trail_points_quantity(
            id,
            from_entry,
            spec,
            ExitQuantityRequest::Percent(qty_percent),
            bar_index,
        );
    }

    fn place_exit_trail_points_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPointsExitSpec,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        if self.position_size <= 0.0 && self.has_pending_entry(&from_entry) {
            self.place_deferred_relative_trail_points_exit(
                id, from_entry, spec, quantity, bar_index,
            );
            return;
        }
        if self.reject_entry_relative_exit_for_pending_entry(&from_entry) {
            return;
        }
        let Some(activation_price_offset) =
            self.exit_tick_price_offset(spec.activation_ticks, spec.mintick)
        else {
            return;
        };
        let Some(offset_price_distance) =
            self.exit_tick_price_offset(spec.offset_ticks, spec.mintick)
        else {
            return;
        };
        self.place_exit_trailing(
            id,
            from_entry,
            PendingTrailingActivation::Points {
                ticks: spec.activation_ticks,
                price: self.avg_price + activation_price_offset,
            },
            offset_price_distance,
            quantity,
            bar_index,
        );
    }

    fn place_deferred_relative_trail_points_exit(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPointsExitSpec,
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        let Some(pending_entry_quantity) = self.order_book.entries().quantity_for_id(&from_entry)
        else {
            return;
        };
        if self
            .exit_tick_price_offset(spec.activation_ticks, spec.mintick)
            .is_none()
        {
            return;
        }
        if self
            .exit_tick_price_offset(spec.offset_ticks, spec.mintick)
            .is_none()
        {
            return;
        }
        if self
            .resolve_exit_quantity_request_for_available(
                quantity,
                pending_entry_quantity,
                pending_entry_quantity,
            )
            .is_none()
        {
            return;
        }

        self.order_book
            .exits_mut()
            .replace_or_append_deferred_relative(DeferredRelativeExit {
                id,
                from_entry,
                trigger: DeferredRelativeExitTrigger::TrailPoints {
                    activation_ticks: spec.activation_ticks,
                    offset_ticks: spec.offset_ticks,
                    mintick: spec.mintick,
                },
                quantity,
                last_update_bar_index: bar_index,
            });
    }

    fn place_exit_trailing(
        &mut self,
        id: String,
        from_entry: String,
        activation: PendingTrailingActivation,
        offset_price_distance: f64,
        quantity: ExitQuantityRequest,
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
        quantity: ExitQuantityRequest,
        bar_index: usize,
    ) {
        if !trigger.prices_are_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_PRICE".to_owned(),
                message: "`strategy.exit` price must be finite".to_owned(),
            });
            return;
        }
        if quantity.has_invalid_fixed_quantity() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_QTY".to_owned(),
                message: "`strategy.exit` quantity must be finite and positive".to_owned(),
            });
            return;
        }
        let target_position_size = if self.position_size > 0.0
            && self.entry_id.as_deref() == Some(from_entry.as_str())
        {
            self.position_size
        } else if let Some(pending_entry_quantity) =
            self.order_book.entries().quantity_for_id(&from_entry)
        {
            pending_entry_quantity
        } else {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_ENTRY".to_owned(),
                message: "`strategy.exit` from_entry must match the current long entry".to_owned(),
            });
            return;
        };

        let multiple_reservation_family = match (quantity, trigger.reservation_family()) {
            (
                ExitQuantityRequest::Fixed(_) | ExitQuantityRequest::Percent(_),
                PendingExitReservationFamily::SingleTrigger,
            ) => Some(PendingExitReservationFamily::SingleTrigger),
            (
                ExitQuantityRequest::Fixed(_) | ExitQuantityRequest::Percent(_),
                PendingExitReservationFamily::Bracket,
            ) => Some(PendingExitReservationFamily::Bracket),
            (
                ExitQuantityRequest::Fixed(_) | ExitQuantityRequest::Percent(_),
                PendingExitReservationFamily::Trailing,
            ) => Some(PendingExitReservationFamily::Trailing),
            _ => None,
        };
        let released_identity =
            multiple_reservation_family.map(|_| (id.as_str(), from_entry.as_str()));
        let other_exits_are_supported_reservations = multiple_reservation_family.is_some()
            && self
                .order_book
                .exits()
                .other_exits_are_supported_reservations(&from_entry, released_identity);
        let available_quantity = if other_exits_are_supported_reservations {
            self.order_book.exits_mut().available_unreserved_quantity(
                target_position_size,
                &from_entry,
                released_identity,
            )
        } else {
            target_position_size
        };

        let Some((quantity, reserved_quantity)) = self.resolve_exit_quantity_request_for_available(
            quantity,
            target_position_size,
            available_quantity,
        ) else {
            return;
        };

        if self
            .order_book
            .exits()
            .find_by_identity(&id, &from_entry)
            .is_some_and(|pending_exit| {
                pending_exit.id == id
                    && pending_exit.from_entry == from_entry
                    && pending_exit.trigger.placement_equivalent(&trigger)
                    && pending_exit.quantity == quantity
                    && pending_exit.reserved_quantity == reserved_quantity
            })
        {
            return;
        }

        let pending_exit = PendingExit {
            id,
            from_entry,
            trigger,
            quantity,
            reserved_quantity,
            multiple_reservation: multiple_reservation_family.is_some(),
            last_update_bar_index: bar_index,
        };
        if multiple_reservation_family.is_some() && other_exits_are_supported_reservations {
            self.order_book.exits_mut().replace_or_append(pending_exit);
        } else {
            self.order_book.exits_mut().replace_all(pending_exit);
        }
    }

    pub(super) fn resolve_exit_quantity_request_for_available(
        &mut self,
        quantity: ExitQuantityRequest,
        target_quantity: f64,
        available_quantity: f64,
    ) -> Option<(PendingExitQuantity, f64)> {
        if !target_quantity.is_finite() || target_quantity <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_QTY".to_owned(),
                message: "`strategy.exit` target quantity must be positive".to_owned(),
            });
            return None;
        }
        if !available_quantity.is_finite() || available_quantity <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_EXIT_QTY".to_owned(),
                message: "`strategy.exit` reserved quantity must be positive".to_owned(),
            });
            return None;
        }

        match quantity {
            ExitQuantityRequest::Full => Some((PendingExitQuantity::Full, available_quantity)),
            ExitQuantityRequest::Fixed(qty) => {
                if !PendingExitQuantity::Fixed(qty).is_valid() {
                    self.diagnostics.push(RuntimeDiagnostic {
                        code: "E_STRATEGY_EXIT_QTY".to_owned(),
                        message: "`strategy.exit` quantity must be finite and positive".to_owned(),
                    });
                    return None;
                }
                Some((PendingExitQuantity::Fixed(qty), qty.min(available_quantity)))
            }
            ExitQuantityRequest::Percent(qty_percent) => {
                if !qty_percent.is_finite() || qty_percent <= 0.0 {
                    self.diagnostics.push(RuntimeDiagnostic {
                        code: "E_STRATEGY_EXIT_QTY_PERCENT".to_owned(),
                        message: "`strategy.exit` qty_percent must be finite and positive"
                            .to_owned(),
                    });
                    return None;
                }
                let requested_quantity = target_quantity * qty_percent / 100.0;
                Some((
                    PendingExitQuantity::Fixed(requested_quantity),
                    requested_quantity.min(available_quantity),
                ))
            }
        }
    }
}
