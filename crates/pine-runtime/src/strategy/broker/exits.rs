use super::{
    BrokerState,
    active_entry_brackets::DeferredLossProfitBracketSpec,
    pending_exits::{
        DeferredBracketLeg, DeferredRelativeExit, DeferredRelativeExitTrigger, ExitQuantityRequest,
        PendingExit, PendingExitQuantity, PendingExitReservationFamily, PendingExitTrigger,
        PendingTrailingActivation, PendingTrailingExit, PendingTrailingSpec, PendingTrailingState,
        TrailPointsExitSpec, TrailPriceExitSpec,
    },
};
use crate::RuntimeDiagnostic;

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

    pub(crate) fn place_all_entry_exit_profit_ticks(
        &mut self,
        id: String,
        ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        let Some(price_offset) = self.exit_tick_price_offset(ticks, mintick) else {
            return;
        };
        let mut pending_exits = Vec::new();
        for index in 0..self.trade_ledger.open_count() {
            let Some(open_trade) = self.trade_ledger.open_at(index) else {
                continue;
            };
            pending_exits.push(PendingExit {
                id: id.clone(),
                from_entry: open_trade.id.clone(),
                target_trade_key: Some(open_trade.key),
                trigger: PendingExitTrigger::Limit(open_trade.entry_price + price_offset),
                quantity: PendingExitQuantity::Full,
                reserved_quantity: open_trade.quantity,
                multiple_reservation: false,
                last_update_bar_index: bar_index,
            });
        }
        self.order_book
            .exits_mut()
            .replace_all_entry_deferred_relative(DeferredRelativeExit {
                id,
                from_entry: String::new(),
                trigger: DeferredRelativeExitTrigger::ProfitTicks { ticks, mintick },
                quantity: ExitQuantityRequest::Full,
                last_update_bar_index: bar_index,
            });
        if pending_exits.is_empty() {
            return;
        }
        self.order_book.exits_mut().replace_all_many(pending_exits);
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
        let Some(limit_price) =
            self.exit_profit_price_from_ticks_for_entry(&from_entry, ticks, mintick)
        else {
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

    pub(crate) fn resolve_deferred_relative_exits_for_entry(
        &mut self,
        entry_id: &str,
        bar_index: usize,
    ) {
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
                    let Some(limit_price) =
                        self.exit_profit_price_from_ticks_for_entry(&from_entry, ticks, mintick)
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
                    let Some(stop_price) =
                        self.exit_loss_price_from_ticks_for_entry(&from_entry, ticks, mintick)
                    else {
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
                    let Some(activation_price) = self.exit_trail_points_activation_price_for_entry(
                        &from_entry,
                        activation_ticks,
                        mintick,
                    ) else {
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
                            price: activation_price,
                        },
                        offset_price_distance,
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::Bracket {
                    downside: DeferredBracketLeg::Absolute(downside),
                    upside: DeferredBracketLeg::RelativeProfit { ticks, mintick },
                } => {
                    let Some(upside) =
                        self.exit_profit_price_from_ticks_for_entry(&from_entry, ticks, mintick)
                    else {
                        continue;
                    };
                    self.place_exit(
                        id,
                        from_entry,
                        PendingExitTrigger::Bracket { downside, upside },
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::Bracket {
                    downside: DeferredBracketLeg::RelativeLoss { ticks, mintick },
                    upside: DeferredBracketLeg::Absolute(upside),
                } => {
                    let Some(downside) =
                        self.exit_loss_price_from_ticks_for_entry(&from_entry, ticks, mintick)
                    else {
                        continue;
                    };
                    self.place_exit(
                        id,
                        from_entry,
                        PendingExitTrigger::Bracket { downside, upside },
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::Bracket {
                    downside:
                        DeferredBracketLeg::RelativeLoss {
                            ticks: loss_ticks,
                            mintick: loss_mintick,
                        },
                    upside:
                        DeferredBracketLeg::RelativeProfit {
                            ticks: profit_ticks,
                            mintick: profit_mintick,
                        },
                } => {
                    self.place_resolved_loss_profit_bracket(
                        id,
                        from_entry,
                        DeferredLossProfitBracketSpec {
                            loss_ticks,
                            loss_mintick,
                            profit_ticks,
                            profit_mintick,
                        },
                        quantity,
                        last_update_bar_index,
                    );
                }
                DeferredRelativeExitTrigger::Bracket { .. } => continue,
            }
        }

        let all_entry_deferred_exits = self.order_book.exits().all_entry_deferred_relative_exits();
        for deferred_exit in all_entry_deferred_exits {
            self.resolve_all_entry_deferred_relative_exit_for_entry(
                deferred_exit,
                entry_id,
                bar_index,
            );
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

    pub(crate) fn place_all_entry_exit_loss_ticks(
        &mut self,
        id: String,
        ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        let Some(price_offset) = self.exit_tick_price_offset(ticks, mintick) else {
            return;
        };
        let mut pending_exits = Vec::new();
        for index in 0..self.trade_ledger.open_count() {
            let Some(open_trade) = self.trade_ledger.open_at(index) else {
                continue;
            };
            pending_exits.push(PendingExit {
                id: id.clone(),
                from_entry: open_trade.id.clone(),
                target_trade_key: Some(open_trade.key),
                trigger: PendingExitTrigger::Stop(open_trade.entry_price - price_offset),
                quantity: PendingExitQuantity::Full,
                reserved_quantity: open_trade.quantity,
                multiple_reservation: false,
                last_update_bar_index: bar_index,
            });
        }
        if pending_exits.is_empty() {
            return;
        }
        self.order_book
            .exits_mut()
            .replace_all_entry_deferred_relative(DeferredRelativeExit {
                id,
                from_entry: String::new(),
                trigger: DeferredRelativeExitTrigger::LossTicks { ticks, mintick },
                quantity: ExitQuantityRequest::Full,
                last_update_bar_index: bar_index,
            });
        self.order_book.exits_mut().replace_all_many(pending_exits);
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
        let Some(stop_price) =
            self.exit_loss_price_from_ticks_for_entry(&from_entry, ticks, mintick)
        else {
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

    pub(crate) fn place_all_entry_exit_trail_points(
        &mut self,
        id: String,
        activation_ticks: f64,
        offset_ticks: f64,
        mintick: f64,
        bar_index: usize,
    ) {
        let Some(activation_price_offset) = self.exit_tick_price_offset(activation_ticks, mintick)
        else {
            return;
        };
        let Some(offset_price_distance) = self.exit_tick_price_offset(offset_ticks, mintick) else {
            return;
        };
        let mut seen_entry_ids: Vec<String> = Vec::new();
        let mut pending_exits = Vec::new();
        for index in 0..self.trade_ledger.open_count() {
            let Some(open_trade) = self.trade_ledger.open_at(index) else {
                continue;
            };
            if seen_entry_ids
                .iter()
                .any(|entry_id| entry_id == &open_trade.id)
            {
                return;
            }
            seen_entry_ids.push(open_trade.id.clone());
            pending_exits.push(PendingExit {
                id: id.clone(),
                from_entry: open_trade.id.clone(),
                target_trade_key: Some(open_trade.key),
                trigger: PendingExitTrigger::Trailing(PendingTrailingExit {
                    spec: PendingTrailingSpec {
                        activation: PendingTrailingActivation::Points {
                            ticks: activation_ticks,
                            price: open_trade.entry_price + activation_price_offset,
                        },
                        offset_price_distance,
                    },
                    state: PendingTrailingState::Inactive,
                }),
                quantity: PendingExitQuantity::Full,
                reserved_quantity: open_trade.quantity,
                multiple_reservation: false,
                last_update_bar_index: bar_index,
            });
        }
        if pending_exits.is_empty() {
            return;
        }
        self.order_book
            .exits_mut()
            .replace_all_entry_deferred_relative(DeferredRelativeExit {
                id,
                from_entry: String::new(),
                trigger: DeferredRelativeExitTrigger::TrailPoints {
                    activation_ticks,
                    offset_ticks,
                    mintick,
                },
                quantity: ExitQuantityRequest::Full,
                last_update_bar_index: bar_index,
            });
        self.order_book.exits_mut().replace_all_many(pending_exits);
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
        let Some(activation_price) = self.exit_trail_points_activation_price_for_entry(
            &from_entry,
            spec.activation_ticks,
            spec.mintick,
        ) else {
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
                price: activation_price,
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

    pub(crate) fn exit_profit_price_from_ticks_for_entry(
        &mut self,
        from_entry: &str,
        ticks: f64,
        mintick: f64,
    ) -> Option<f64> {
        let base_price = self
            .first_open_entry_price_for_entry(from_entry)
            .unwrap_or(self.avg_price);
        self.exit_tick_price_offset(ticks, mintick)
            .map(|price_offset| base_price + price_offset)
    }

    pub(crate) fn exit_loss_price_from_ticks_for_entry(
        &mut self,
        from_entry: &str,
        ticks: f64,
        mintick: f64,
    ) -> Option<f64> {
        let base_price = self
            .first_open_entry_price_for_entry(from_entry)
            .unwrap_or(self.avg_price);
        self.exit_tick_price_offset(ticks, mintick)
            .map(|price_offset| base_price - price_offset)
    }

    pub(super) fn exit_trail_points_activation_price_for_entry(
        &mut self,
        from_entry: &str,
        ticks: f64,
        mintick: f64,
    ) -> Option<f64> {
        let base_price = self
            .first_open_entry_price_for_entry(from_entry)
            .unwrap_or(self.avg_price);
        self.exit_tick_price_offset(ticks, mintick)
            .map(|price_offset| base_price + price_offset)
    }

    pub(super) fn exit_tick_price_offset(&mut self, ticks: f64, mintick: f64) -> Option<f64> {
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

    pub(super) fn place_exit(
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
        let open_entry_position_size = self.open_position_size_for_entry(&from_entry);
        let target_position_size = if self.position_size > 0.0 && open_entry_position_size > 0.0 {
            open_entry_position_size
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
            target_trade_key: None,
            trigger,
            quantity,
            reserved_quantity,
            multiple_reservation: multiple_reservation_family.is_some(),
            last_update_bar_index: bar_index,
        };
        if multiple_reservation_family.is_some() && other_exits_are_supported_reservations {
            self.order_book.exits_mut().replace_or_append(pending_exit);
        } else {
            if pending_exit.from_entry.is_empty()
                && pending_exit.quantity == PendingExitQuantity::Full
            {
                self.order_book
                    .exits_mut()
                    .clear_all_entry_deferred_relative();
            }
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
