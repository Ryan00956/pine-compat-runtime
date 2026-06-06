mod accounting;
mod active_entry_brackets;
mod all_entry_relative_exits;
mod entries;
mod exits;
mod fills;
mod ledger;
mod order_book;
mod pending_exits;

use pine_ir::{DEFAULT_STRATEGY_INITIAL_CAPITAL, StrategyCommission, StrategyMarginSetting};

pub(crate) use active_entry_brackets::{
    LossLimitBracketSpec, LossProfitBracketSpec, StopProfitBracketSpec,
};
use entries::PendingEntryKind;
#[cfg(test)]
use ledger::NetPosition;
use ledger::{OpenTrade, TradeAllocation, TradeDirection, TradeLedger};
use order_book::OrderBook;
use pending_exits::{
    PendingExit, PendingExitQuantity, PendingExitSide, PendingExitTrigger, PendingTrailingUpdate,
};
pub(crate) use pending_exits::{TrailPointsExitSpec, TrailPriceExitSpec};

use crate::{
    RuntimeDiagnostic, StrategyEquitySnapshot, StrategyOrderEvent, StrategyPositionSnapshot,
    StrategyResult, StrategyTrade,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BrokerState {
    initial_capital: f64,
    commission: Option<StrategyCommission>,
    pyramiding_limit: usize,
    margin_long: StrategyMarginSetting,
    margin_short: StrategyMarginSetting,
    open_entry_commission: f64,
    slippage_price_offset: f64,
    limit_verification_price_offset: f64,
    cash: f64,
    position_size: f64,
    avg_price: f64,
    entry_id: Option<String>,
    entry_bar_index: Option<usize>,
    entry_time: Option<i64>,
    open_trade_max_high: Option<f64>,
    open_trade_min_low: Option<f64>,
    open_trade_equity_on_entry: Option<f64>,
    open_trade_min_equity_before_entry: Option<f64>,
    open_trade_max_equity_before_entry: Option<f64>,
    min_equity_before_open_trade: f64,
    max_equity_before_open_trade: f64,
    max_runup: f64,
    max_runup_percent: f64,
    max_drawdown: f64,
    max_drawdown_percent: f64,
    max_contracts_held_long: f64,
    orders: Vec<StrategyOrderEvent>,
    trades: Vec<StrategyTrade>,
    closed_trade_metrics: Vec<ClosedTradeMetrics>,
    position: Vec<StrategyPositionSnapshot>,
    equity: Vec<StrategyEquitySnapshot>,
    diagnostics: Vec<RuntimeDiagnostic>,
    order_book: OrderBook,
    trade_ledger: TradeLedger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryPyramidingMode {
    EnforceLimit,
    SameTickPriceException,
}

impl Default for BrokerState {
    fn default() -> Self {
        Self::new(DEFAULT_STRATEGY_INITIAL_CAPITAL)
    }
}

impl BrokerState {
    #[must_use]
    pub fn new(initial_capital: f64) -> Self {
        Self::new_with_commission(initial_capital, None)
    }

    #[must_use]
    pub fn new_with_cash_per_contract_commission(
        initial_capital: f64,
        commission_per_contract: f64,
    ) -> Self {
        Self::new_with_commission(
            initial_capital,
            Some(StrategyCommission::CashPerContract(commission_per_contract)),
        )
    }

    #[must_use]
    pub fn new_with_commission(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
    ) -> Self {
        Self::new_with_commission_and_slippage(initial_capital, commission, 0.0)
    }

    #[must_use]
    pub fn new_with_commission_and_slippage(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
    ) -> Self {
        Self::new_with_commission_slippage_and_limit_verification(
            initial_capital,
            commission,
            slippage_price_offset,
            0.0,
        )
    }

    #[must_use]
    pub fn new_with_commission_slippage_and_limit_verification(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
        limit_verification_price_offset: f64,
    ) -> Self {
        Self::new_with_account_settings(
            initial_capital,
            commission,
            slippage_price_offset,
            limit_verification_price_offset,
            StrategyMarginSetting::default(),
            StrategyMarginSetting::default(),
        )
    }

    #[must_use]
    pub fn new_with_account_settings(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
        limit_verification_price_offset: f64,
        margin_long: StrategyMarginSetting,
        margin_short: StrategyMarginSetting,
    ) -> Self {
        Self::new_with_account_settings_and_pyramiding(
            initial_capital,
            commission,
            slippage_price_offset,
            limit_verification_price_offset,
            margin_long,
            margin_short,
            1,
        )
    }

    #[must_use]
    pub fn new_with_account_settings_and_pyramiding(
        initial_capital: f64,
        commission: Option<StrategyCommission>,
        slippage_price_offset: f64,
        limit_verification_price_offset: f64,
        margin_long: StrategyMarginSetting,
        margin_short: StrategyMarginSetting,
        pyramiding_limit: usize,
    ) -> Self {
        Self {
            initial_capital,
            commission,
            pyramiding_limit,
            margin_long,
            margin_short,
            open_entry_commission: 0.0,
            slippage_price_offset,
            limit_verification_price_offset,
            cash: initial_capital,
            position_size: 0.0,
            avg_price: 0.0,
            entry_id: None,
            entry_bar_index: None,
            entry_time: None,
            open_trade_max_high: None,
            open_trade_min_low: None,
            open_trade_equity_on_entry: None,
            open_trade_min_equity_before_entry: None,
            open_trade_max_equity_before_entry: None,
            min_equity_before_open_trade: initial_capital,
            max_equity_before_open_trade: initial_capital,
            max_runup: 0.0,
            max_runup_percent: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percent: 0.0,
            max_contracts_held_long: 0.0,
            orders: Vec::new(),
            trades: Vec::new(),
            closed_trade_metrics: Vec::new(),
            position: Vec::new(),
            equity: Vec::new(),
            diagnostics: Vec::new(),
            order_book: OrderBook::new(),
            trade_ledger: TradeLedger::default(),
        }
    }

    fn commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        match self.commission {
            Some(StrategyCommission::CashPerContract(value)) => qty * value,
            Some(StrategyCommission::CashPerOrder(value)) => value,
            Some(StrategyCommission::Percent(value)) => qty * price * (value / 100.0),
            None => 0.0,
        }
    }

    fn entry_commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        self.commission_for_fill(qty, price)
    }

    fn exit_commission_for_fill(&self, qty: f64, price: f64) -> f64 {
        self.commission_for_fill(qty, price)
    }

    fn entry_commission_for_closed_quantity(&self, qty: f64) -> f64 {
        if qty >= self.position_size {
            self.open_entry_commission
        } else {
            self.open_entry_commission * (qty / self.position_size)
        }
    }

    fn long_entry_fill_price(&self, price: f64) -> f64 {
        price + self.slippage_price_offset
    }

    fn long_exit_fill_price(&self, price: f64) -> f64 {
        price - self.slippage_price_offset
    }

    fn long_limit_exit_is_verified(&self, limit_price: f64, high: f64) -> bool {
        high >= limit_price + self.limit_verification_price_offset
    }

    pub(crate) fn entry_long(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) -> bool {
        self.entry_long_internal(
            id,
            bar_index,
            time,
            price,
            qty,
            EntryPyramidingMode::EnforceLimit,
        )
    }

    fn entry_long_from_price_based_same_tick_exception(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
    ) -> bool {
        self.entry_long_internal(
            id,
            bar_index,
            time,
            price,
            qty,
            EntryPyramidingMode::SameTickPriceException,
        )
    }

    fn entry_long_internal(
        &mut self,
        id: String,
        bar_index: usize,
        time: i64,
        price: f64,
        qty: f64,
        pyramiding_mode: EntryPyramidingMode,
    ) -> bool {
        if !qty.is_finite() || qty <= 0.0 {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_QTY".to_owned(),
                message: "`strategy.entry` quantity must be positive".to_owned(),
            });
            return false;
        }
        if !price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` fill price must be finite".to_owned(),
            });
            return false;
        }
        if pyramiding_mode == EntryPyramidingMode::EnforceLimit && !self.can_open_long_entry() {
            return false;
        }

        let fill_price = self.long_entry_fill_price(price);
        if !fill_price.is_finite() {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_PRICE".to_owned(),
                message: "`strategy.entry` slipped fill price must be finite".to_owned(),
            });
            return false;
        }
        if !self.can_afford_long_entry(qty, fill_price) {
            self.diagnostics.push(RuntimeDiagnostic {
                code: "E_STRATEGY_MARGIN".to_owned(),
                message: "`strategy.entry` requires more margin than available equity".to_owned(),
            });
            return false;
        }

        let equity_on_entry = self.cash;
        let min_equity_before_entry = self.min_equity_before_open_trade;
        let max_equity_before_entry = self.max_equity_before_open_trade;
        let open_trade = OpenTrade {
            key: 0,
            id: id.clone(),
            direction: TradeDirection::Long,
            quantity: qty,
            entry_price: fill_price,
            entry_bar_index: bar_index,
            entry_time: time,
            entry_commission: self.entry_commission_for_fill(qty, fill_price),
            max_high: Some(fill_price),
            min_low: Some(fill_price),
            equity_on_entry: Some(equity_on_entry),
            min_equity_before_entry: Some(min_equity_before_entry),
            max_equity_before_entry: Some(max_equity_before_entry),
        };
        if pyramiding_mode == EntryPyramidingMode::SameTickPriceException {
            self.record_open_long_trade_exceeding_pyramiding(open_trade);
        } else {
            self.record_open_long_trade(open_trade);
        }
        self.record_order_event(id, bar_index, time, "strategy.long", qty, fill_price);
        self.record_position_snapshot(bar_index);
        true
    }

    fn record_open_long_trade(&mut self, open_trade: OpenTrade) {
        self.record_open_long_legacy_state(&open_trade);
        if self.pyramiding_limit <= 1 {
            self.trade_ledger.open_long(open_trade);
        } else {
            self.trade_ledger.append_long(open_trade);
        }
        self.sync_aggregate_position_from_ledger();
    }

    fn record_open_long_trade_exceeding_pyramiding(&mut self, open_trade: OpenTrade) {
        self.record_open_long_legacy_state(&open_trade);
        self.trade_ledger.append_long(open_trade);
        self.sync_aggregate_position_from_ledger();
    }

    fn sync_aggregate_position_from_ledger(&mut self) {
        let net_position = self.trade_ledger.net_position();
        self.position_size = net_position.signed_size;
        self.avg_price = net_position.avg_price;
        self.max_contracts_held_long = self.max_contracts_held_long.max(self.position_size);
    }

    fn apply_trade_allocations_and_sync_position(&mut self, allocations: &[TradeAllocation]) {
        self.trade_ledger.apply_allocations(allocations);
        self.sync_aggregate_position_from_ledger();
    }

    fn expand_persistent_all_entry_exit_for_new_entry(&mut self, bar_index: usize) {
        let position_size = self.position_size;
        if !position_size.is_finite() || position_size <= 0.0 {
            return;
        }
        let Some(pending_exit) = self.order_book.exits_mut().current_mut() else {
            return;
        };
        if pending_exit.from_entry.is_empty()
            && pending_exit.quantity == PendingExitQuantity::Full
            && !pending_exit.multiple_reservation
            && matches!(
                pending_exit.trigger,
                PendingExitTrigger::Stop(_)
                    | PendingExitTrigger::Limit(_)
                    | PendingExitTrigger::Bracket { .. }
                    | PendingExitTrigger::Trailing(_)
            )
        {
            pending_exit.reserved_quantity = position_size;
            pending_exit.last_update_bar_index = bar_index;
        }
    }

    fn can_open_long_entry(&self) -> bool {
        self.trade_ledger.open_count() < self.pyramiding_limit
    }

    pub(crate) fn cancel_exit_for_entry(&mut self, entry_id: &str) {
        self.order_book.exits_mut().clear_for_entry(entry_id);
    }

    pub(crate) fn cancel_pending_order(&mut self, id: &str) {
        self.order_book.cancel_id(id);
    }

    pub(crate) fn cancel_all_pending_orders(&mut self) {
        self.order_book.clear_all();
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_market_long_entry(
        &mut self,
        id: String,
        qty: f64,
        created_bar_index: usize,
    ) {
        if !self.can_open_long_entry() {
            return;
        }
        self.order_book.entries_mut().place_market_long(
            id,
            qty,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_limit_long_entry(
        &mut self,
        id: String,
        qty: f64,
        limit: f64,
        created_bar_index: usize,
    ) {
        if !self.can_open_long_entry() {
            return;
        }
        self.order_book.entries_mut().place_limit_long(
            id,
            qty,
            limit,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_stop_long_entry(
        &mut self,
        id: String,
        qty: f64,
        stop: f64,
        created_bar_index: usize,
    ) {
        if !self.can_open_long_entry() {
            return;
        }
        self.order_book.entries_mut().place_stop_long(
            id,
            qty,
            stop,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn place_pending_stop_limit_long_entry(
        &mut self,
        id: String,
        qty: f64,
        stop: f64,
        limit: f64,
        created_bar_index: usize,
    ) {
        if !self.can_open_long_entry() {
            return;
        }
        self.order_book.entries_mut().place_stop_limit_long(
            id,
            qty,
            stop,
            limit,
            created_bar_index,
            &mut self.diagnostics,
        );
    }

    #[allow(dead_code)]
    fn pending_entry_count(&self) -> usize {
        self.order_book.entries().count()
    }

    pub(crate) fn fill_pending_market_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        fill_price: f64,
    ) {
        if !self.can_open_long_entry() {
            self.order_book.entries_mut().clear_all();
            return;
        }
        let Some(pending_entry) = self
            .order_book
            .entries_mut()
            .take_first_eligible_market_long(bar_index)
        else {
            return;
        };

        let entry_id = pending_entry.id;
        let filled = self.entry_long(
            entry_id.clone(),
            bar_index,
            time,
            fill_price,
            pending_entry.quantity,
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
        if !self.can_open_long_entry() {
            self.order_book.entries_mut().clear_all();
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

        for pending_entry in pending_entries {
            let PendingEntryKind::Limit { price } = pending_entry.kind else {
                continue;
            };
            let entry_id = pending_entry.id;
            let filled = self.entry_long_from_price_based_same_tick_exception(
                entry_id.clone(),
                bar_index,
                time,
                price,
                pending_entry.quantity,
            );
            if filled {
                self.resolve_deferred_relative_exits_for_entry(&entry_id, bar_index);
                self.expand_persistent_all_entry_exit_for_new_entry(bar_index);
            } else {
                self.order_book.exits_mut().clear_for_entry(&entry_id);
            }
        }
        self.order_book.entries_mut().clear_all();
    }

    pub(crate) fn fill_pending_stop_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
    ) {
        if !self.can_open_long_entry() {
            self.order_book.entries_mut().clear_all();
            return;
        }
        let pending_entries = self
            .order_book
            .entries_mut()
            .take_all_eligible_stop_long(bar_index, high);
        if pending_entries.is_empty() {
            return;
        }

        for pending_entry in pending_entries {
            let PendingEntryKind::Stop { price } = pending_entry.kind else {
                continue;
            };
            let entry_id = pending_entry.id;
            let filled = self.entry_long_from_price_based_same_tick_exception(
                entry_id.clone(),
                bar_index,
                time,
                price,
                pending_entry.quantity,
            );
            if filled {
                self.resolve_deferred_relative_exits_for_entry(&entry_id, bar_index);
                self.expand_persistent_all_entry_exit_for_new_entry(bar_index);
            } else {
                self.order_book.exits_mut().clear_for_entry(&entry_id);
            }
        }
        self.order_book.entries_mut().clear_all();
    }

    pub(crate) fn fill_pending_stop_limit_long_entries(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if !self.can_open_long_entry() {
            self.order_book.entries_mut().clear_all();
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

        for pending_entry in pending_entries {
            let PendingEntryKind::StopLimit { limit_price, .. } = pending_entry.kind else {
                continue;
            };
            let entry_id = pending_entry.id;
            let filled = self.entry_long_from_price_based_same_tick_exception(
                entry_id.clone(),
                bar_index,
                time,
                limit_price,
                pending_entry.quantity,
            );
            if filled {
                self.resolve_deferred_relative_exits_for_entry(&entry_id, bar_index);
                self.expand_persistent_all_entry_exit_for_new_entry(bar_index);
            } else {
                self.order_book.exits_mut().clear_for_entry(&entry_id);
            }
        }
        self.order_book.entries_mut().clear_all();
    }

    fn has_pending_entry(&self, id: &str) -> bool {
        self.order_book.entries().quantity_for_id(id).is_some()
    }

    fn open_position_size_for_entry(&self, id: &str) -> f64 {
        if id.is_empty() {
            return self.position_size;
        }
        self.trade_ledger.open_quantity_for_entry(id)
    }

    fn last_open_trade_key_and_price_for_entry(&self, id: &str) -> Option<(u64, f64)> {
        let mut result = None;
        for index in 0..self.trade_ledger.open_count() {
            let Some(open_trade) = self.trade_ledger.open_at(index) else {
                continue;
            };
            if open_trade.id == id {
                result = Some((open_trade.key, open_trade.entry_price));
            }
        }
        result
    }

    fn first_open_entry_price_for_entry(&self, id: &str) -> Option<f64> {
        self.trade_ledger.first_open_entry_price_for_entry(id)
    }

    fn has_open_position_for_entry(&self, id: &str) -> bool {
        if id.is_empty() {
            return self.position_size > 0.0;
        }
        self.open_position_size_for_entry(id) > 0.0
    }

    pub(crate) fn reject_entry_relative_exit_for_pending_entry(
        &mut self,
        from_entry: &str,
    ) -> bool {
        if self.position_size > 0.0
            || self
                .order_book
                .entries()
                .quantity_for_id(from_entry)
                .is_none()
        {
            return false;
        }

        self.diagnostics.push(RuntimeDiagnostic {
            code: "E_STRATEGY_EXIT_ENTRY".to_owned(),
            message: "`strategy.exit` from_entry must match the current long entry".to_owned(),
        });
        true
    }

    fn pending_exit(&self) -> Option<&PendingExit> {
        self.order_book.exits().current()
    }

    #[allow(dead_code)]
    fn pending_exit_mut(&mut self) -> Option<&mut PendingExit> {
        self.order_book.exits_mut().current_mut()
    }

    #[allow(dead_code)]
    fn pending_exit_count(&self) -> usize {
        self.order_book.exits().count()
    }

    #[allow(dead_code)]
    fn pending_exit_by_identity(&self, id: &str, from_entry: &str) -> Option<&PendingExit> {
        self.order_book.exits().find_by_identity(id, from_entry)
    }

    #[allow(dead_code)]
    fn pending_exit_by_identity_and_key(
        &self,
        id: &str,
        from_entry: &str,
        target_trade_key: Option<u64>,
    ) -> Option<&PendingExit> {
        self.order_book
            .exits()
            .find_by_identity_and_key(id, from_entry, target_trade_key)
    }

    #[allow(dead_code)]
    fn pending_exits_in_placement_order(&self) -> impl Iterator<Item = &PendingExit> {
        self.order_book.exits().iter()
    }

    pub(crate) fn evaluate_pending_exits(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        if self.pending_exit_count() > 1 {
            self.evaluate_multiple_pending_exits(bar_index, time, high, low);
            return;
        }

        let Some(mut pending_exit) = self.pending_exit().cloned() else {
            return;
        };
        if pending_exit.last_update_bar_index >= bar_index {
            return;
        }
        if self.position_size <= 0.0 || !self.has_open_position_for_entry(&pending_exit.from_entry)
        {
            if self.position_size <= 0.0 && self.has_pending_entry(&pending_exit.from_entry) {
                return;
            }
            self.order_book
                .exits_mut()
                .clear_for_entry(&pending_exit.from_entry);
            return;
        }
        let triggered_price = match &mut pending_exit.trigger {
            PendingExitTrigger::Stop(price) if low <= *price => Some(*price),
            PendingExitTrigger::Limit(price) if self.long_limit_exit_is_verified(*price, high) => {
                Some(*price)
            }
            PendingExitTrigger::Bracket { downside, upside } => {
                if low <= *downside {
                    Some(*downside)
                } else if self.long_limit_exit_is_verified(*upside, high) {
                    Some(*upside)
                } else {
                    None
                }
            }
            PendingExitTrigger::Trailing(trailing) => match trailing.evaluate_update(high, low) {
                PendingTrailingUpdate::NoChange => return,
                PendingTrailingUpdate::Persist(updated_trailing) => {
                    pending_exit.trigger = PendingExitTrigger::Trailing(updated_trailing);
                    self.order_book.exits_mut().replace_all(pending_exit);
                    return;
                }
                PendingTrailingUpdate::Candidate(touch) => Some(touch.exit_price),
            },
            _ => None,
        };
        if let Some(exit_price) = triggered_price {
            let from_entry = pending_exit.from_entry.clone();
            self.fill_pending_exit(pending_exit, bar_index, time, exit_price);
            if self.position_size <= 0.0 {
                self.order_book.exits_mut().clear_all();
            } else {
                self.order_book.exits_mut().clear_for_entry(&from_entry);
            }
        }
    }

    fn evaluate_multiple_pending_exits(
        &mut self,
        bar_index: usize,
        time: i64,
        high: f64,
        low: f64,
    ) {
        let pending_exits: Vec<PendingExit> = self.order_book.exits().iter().cloned().collect();
        if pending_exits.is_empty() {
            return;
        }
        if self.position_size <= 0.0 {
            if self.position_size <= 0.0 {
                let attached_pending_entry_ids: Vec<String> = pending_exits
                    .iter()
                    .filter(|pending_exit| self.has_pending_entry(&pending_exit.from_entry))
                    .map(|pending_exit| pending_exit.from_entry.clone())
                    .collect();
                if !attached_pending_entry_ids.is_empty() {
                    for pending_exit in pending_exits {
                        if !attached_pending_entry_ids
                            .iter()
                            .any(|entry_id| entry_id == &pending_exit.from_entry)
                        {
                            self.order_book
                                .exits_mut()
                                .clear_for_entry(&pending_exit.from_entry);
                        }
                    }
                    return;
                }
            }
            return;
        }

        let mut touched_candidates = Vec::new();
        let mut state_updates = Vec::new();
        for pending_exit in pending_exits {
            if pending_exit.last_update_bar_index >= bar_index {
                continue;
            }
            if !self.has_open_position_for_entry(&pending_exit.from_entry) {
                self.order_book
                    .exits_mut()
                    .clear_for_entry(&pending_exit.from_entry);
                continue;
            }

            match pending_exit.trigger.clone() {
                PendingExitTrigger::Trailing(trailing) => {
                    match trailing.evaluate_update(high, low) {
                        PendingTrailingUpdate::NoChange => {}
                        PendingTrailingUpdate::Persist(updated_trailing) => {
                            let mut updated_pending_exit = pending_exit;
                            updated_pending_exit.trigger =
                                PendingExitTrigger::Trailing(updated_trailing);
                            state_updates.push(updated_pending_exit);
                        }
                        PendingTrailingUpdate::Candidate(touch) => {
                            touched_candidates.push((pending_exit, touch.exit_price, touch.side));
                        }
                    }
                }
                _ => {
                    if let Some(touch) = pending_exit.trigger.touched_candidate(
                        high,
                        low,
                        self.limit_verification_price_offset,
                    ) {
                        touched_candidates.push((pending_exit, touch.exit_price, touch.side));
                    }
                }
            }
        }

        let winning_side = if touched_candidates
            .iter()
            .any(|(_, _, side)| *side == PendingExitSide::Stop)
        {
            PendingExitSide::Stop
        } else if touched_candidates
            .iter()
            .any(|(_, _, side)| *side == PendingExitSide::Limit)
        {
            PendingExitSide::Limit
        } else {
            for updated_pending_exit in state_updates {
                self.order_book
                    .exits_mut()
                    .replace_or_append(updated_pending_exit);
            }
            return;
        };

        let mut filled_identities = Vec::new();
        for (pending_exit, exit_price, side) in touched_candidates {
            if side != winning_side {
                continue;
            }
            if self.position_size <= 0.0 {
                break;
            }
            if !self.has_open_position_for_entry(&pending_exit.from_entry) {
                self.order_book
                    .exits_mut()
                    .clear_for_entry(&pending_exit.from_entry);
                continue;
            }
            filled_identities.push((
                pending_exit.id.clone(),
                pending_exit.from_entry.clone(),
                pending_exit.target_trade_key,
            ));
            self.fill_pending_exit(pending_exit, bar_index, time, exit_price);
        }

        if self.position_size <= 0.0 {
            self.order_book.exits_mut().clear_all();
        } else {
            for updated_pending_exit in state_updates {
                self.order_book
                    .exits_mut()
                    .replace_or_append(updated_pending_exit);
            }
            self.order_book
                .exits_mut()
                .remove_identities(&filled_identities);
        }
    }

    #[must_use]
    pub fn result(&self) -> StrategyResult {
        StrategyResult {
            orders: self.orders.clone(),
            trades: self.trades.clone(),
            position: self.position.clone(),
            equity: self.equity.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ClosedTradeMetrics {
    commission: f64,
    profit_percent: f64,
    max_runup: f64,
    max_drawdown: f64,
}

#[cfg(test)]
mod tests;
