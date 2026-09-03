use super::{
    BrokerState,
    pending_entries::{PendingEntry, PendingEntryDirection, PendingEntryKind},
    types::{EntryFill, EntryPyramidingMode, InternalOrderKey, OcaPeerEffects},
};
use std::collections::{HashMap, HashSet};

impl BrokerState {
    pub(crate) fn fill_pending_market_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        fill_price: f64,
    ) {
        let pending_entry = self
            .order_book
            .entries_mut()
            .take_first_eligible_market_long(bar_index);
        self.fill_pending_market_entries_from(pending_entry, bar_index, time, fill_price);
    }

    pub(crate) fn fill_same_bar_market_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        fill_price: f64,
    ) {
        let pending_entry = self
            .order_book
            .entries_mut()
            .take_first_same_bar_market(bar_index);
        self.fill_pending_market_entries_from(pending_entry, bar_index, time, fill_price);
    }

    fn fill_pending_market_entries_from(
        &mut self,
        pending_entry: Option<super::pending_entries::PendingEntry>,
        bar_index: usize,
        time: i64,
        fill_price: f64,
    ) {
        let Some(pending_entry) = pending_entry else {
            return;
        };
        if !pending_entry.enforce_pyramiding {
            let signed_quantity = match pending_entry.direction {
                PendingEntryDirection::Long => pending_entry.quantity,
                PendingEntryDirection::Short => -pending_entry.quantity,
            };
            let filled_key = pending_entry.key;
            let filled_qty = pending_entry.quantity;
            if self.apply_generic_market_order_netting(
                pending_entry.id,
                signed_quantity,
                bar_index,
                time,
                fill_price,
                pending_entry.metadata,
            ) {
                self.order_book.apply_oca_after_fill(filled_key, filled_qty);
            }
            self.order_book.entries_mut().clear_all();
            return;
        }
        if pending_entry.direction == PendingEntryDirection::Short {
            let entry_id = pending_entry.id;
            let filled = self.entry_short_internal(
                EntryFill {
                    id: entry_id.clone(),
                    bar_index,
                    time,
                    price: fill_price,
                    qty: pending_entry.quantity,
                    metadata: pending_entry.metadata,
                },
                EntryPyramidingMode::EnforceLimit,
            );
            if !filled {
                self.order_book.exits_mut().clear_for_entry(&entry_id);
            }
            self.order_book.entries_mut().clear_all();
            return;
        }

        let entry_id = pending_entry.id;
        let filled = self.entry_long_internal(
            EntryFill {
                id: entry_id.clone(),
                bar_index,
                time,
                price: fill_price,
                qty: pending_entry.quantity,
                metadata: pending_entry.metadata,
            },
            EntryPyramidingMode::EnforceLimit,
        );
        if filled {
            self.resolve_deferred_relative_exits_for_entry(&entry_id, bar_index);
            self.expand_persistent_all_entry_exit_for_new_entry(bar_index);
        } else {
            self.order_book.exits_mut().clear_for_entry(&entry_id);
        }
        self.order_book.entries_mut().clear_all();
    }

    pub(crate) fn fill_pending_limit_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        low: f64,
    ) {
        if self.same_side_long_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_limit_long_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_long_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Long);
            return;
        }
        let pending_entries = self.order_book.entries_mut().take_all_eligible_limit_long(
            bar_index,
            low,
            self.limit_verification_price_offset,
        );
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::Limit { price } => Some(price),
                _ => None,
            }
        });
    }

    pub(crate) fn fill_pending_stop_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
    ) {
        if self.same_side_long_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_stop_long_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_long_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Long);
            return;
        }
        let pending_entries = self
            .order_book
            .entries_mut()
            .take_all_eligible_stop_long(bar_index, high);
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::Stop { price } => Some(price),
                _ => None,
            }
        });
    }

    pub(crate) fn fill_pending_stop_limit_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if self.same_side_long_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_stop_limit_long_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_long_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Long);
            return;
        }
        self.order_book
            .entries_mut()
            .activate_stop_limit_long_entries(bar_index, high);
        let pending_entries = self
            .order_book
            .entries_mut()
            .take_all_eligible_stop_limit_long(
                bar_index,
                low,
                self.limit_verification_price_offset,
            );
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::StopLimit { limit_price, .. } => Some(limit_price),
                _ => None,
            }
        });
    }

    pub(crate) fn fill_pending_limit_short_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
    ) {
        if self.same_side_short_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_limit_short_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_short_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Short);
            return;
        }
        let pending_entries = self.order_book.entries_mut().take_all_eligible_limit_short(
            bar_index,
            high,
            self.limit_verification_price_offset,
        );
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::Limit { price } => Some(price),
                _ => None,
            }
        });
    }

    fn fill_taken_generic_or_entries(
        &mut self,
        pending_entries: Vec<PendingEntry>,
        bar_index: usize,
        time: i64,
        mut fill_price: impl FnMut(&PendingEntry) -> Option<f64>,
    ) {
        let mut cancelled = HashSet::new();
        let mut reduce_by: HashMap<InternalOrderKey, f64> = HashMap::new();
        let mut remaining_qty: HashMap<InternalOrderKey, f64> = HashMap::new();
        for mut pending_entry in pending_entries {
            if cancelled.contains(&pending_entry.key) {
                continue;
            }
            if let Some(&qty) = remaining_qty.get(&pending_entry.key) {
                pending_entry.quantity = qty;
            } else if let Some(&sub) = reduce_by.get(&pending_entry.key) {
                pending_entry.quantity = (pending_entry.quantity - sub).max(0.0);
            }
            if pending_entry.quantity <= 0.0 {
                self.order_book.clear_oca_order(pending_entry.key);
                continue;
            }
            let Some(price) = fill_price(&pending_entry) else {
                continue;
            };
            let filled_qty = pending_entry.quantity;
            let effects = self.fill_pending_generic_or_entry(pending_entry, bar_index, time, price);
            cancelled.extend(effects.cancelled);
            for (key, remaining) in effects.reduced {
                remaining_qty.insert(key, remaining);
                if remaining <= 0.0 {
                    cancelled.insert(key);
                }
            }
            for key in effects.reduce_taken {
                *reduce_by.entry(key).or_insert(0.0) += filled_qty;
            }
        }
    }

    fn fill_pending_generic_or_entry(
        &mut self,
        pending_entry: PendingEntry,
        bar_index: usize,
        time: i64,
        price: f64,
    ) -> OcaPeerEffects {
        if !pending_entry.enforce_pyramiding {
            let signed_quantity = match pending_entry.direction {
                PendingEntryDirection::Long => pending_entry.quantity,
                PendingEntryDirection::Short => -pending_entry.quantity,
            };
            let filled_key = pending_entry.key;
            let filled_qty = pending_entry.quantity;
            if self.apply_generic_market_order_netting(
                pending_entry.id,
                signed_quantity,
                bar_index,
                time,
                price,
                pending_entry.metadata,
            ) {
                return self.order_book.apply_oca_after_fill(filled_key, filled_qty);
            }
            return OcaPeerEffects::default();
        }

        let entry_id = pending_entry.id;
        let opposite = match pending_entry.direction {
            PendingEntryDirection::Long => self.position_size < 0.0,
            PendingEntryDirection::Short => self.position_size > 0.0,
        };
        let pyramiding_mode = if opposite {
            EntryPyramidingMode::EnforceLimit
        } else {
            EntryPyramidingMode::SameTickPriceException
        };
        let filled = match pending_entry.direction {
            PendingEntryDirection::Long => self.entry_long_internal(
                EntryFill {
                    id: entry_id.clone(),
                    bar_index,
                    time,
                    price,
                    qty: pending_entry.quantity,
                    metadata: pending_entry.metadata,
                },
                pyramiding_mode,
            ),
            PendingEntryDirection::Short => self.entry_short_internal(
                EntryFill {
                    id: entry_id.clone(),
                    bar_index,
                    time,
                    price,
                    qty: pending_entry.quantity,
                    metadata: pending_entry.metadata,
                },
                pyramiding_mode,
            ),
        };
        if filled {
            self.resolve_deferred_relative_exits_for_entry(&entry_id, bar_index);
            self.expand_persistent_all_entry_exit_for_new_entry(bar_index);
        } else {
            self.order_book.exits_mut().clear_for_entry(&entry_id);
        }
        OcaPeerEffects::default()
    }

    pub(crate) fn fill_pending_stop_short_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        low: f64,
    ) {
        if self.same_side_short_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_stop_short_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_short_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Short);
            return;
        }
        let pending_entries = self
            .order_book
            .entries_mut()
            .take_all_eligible_stop_short(bar_index, low);
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::Stop { price } => Some(price),
                _ => None,
            }
        });
    }

    pub(crate) fn fill_pending_stop_limit_short_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if self.same_side_short_entry_blocked()
            && !self
                .order_book
                .entries()
                .has_stop_limit_short_bypassing_pyramiding()
        {
            if self
                .order_book
                .entries()
                .has_price_based_short_bypassing_pyramiding()
            {
                return;
            }
            self.order_book
                .entries_mut()
                .clear_direction(PendingEntryDirection::Short);
            return;
        }
        self.order_book
            .entries_mut()
            .activate_stop_limit_short_entries(bar_index, low);
        let pending_entries = self
            .order_book
            .entries_mut()
            .take_all_eligible_stop_limit_short(
                bar_index,
                high,
                self.limit_verification_price_offset,
            );
        if pending_entries.is_empty() {
            return;
        }

        self.fill_taken_generic_or_entries(pending_entries, bar_index, time, |pending_entry| {
            match pending_entry.kind {
                PendingEntryKind::StopLimit { limit_price, .. } => Some(limit_price),
                _ => None,
            }
        });
    }
}
