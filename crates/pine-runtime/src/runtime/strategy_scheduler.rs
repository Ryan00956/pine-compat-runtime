use pine_ir::ScriptMode;

use super::historical::HistoricalRuntime;
use crate::{Bar, RuntimeError};

/// Internal extra-pass cap per bar. Extra `calc_on_order_fills` passes stop
/// with a runtime error when this bound is exceeded.
pub(crate) const DEFAULT_MAX_RECALCULATION_PASSES: u32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrategyExecutionIdentity {
    pub bar_index: usize,
    pub phase: StrategyBarPhase,
    pub fill_step: Option<HistoricalFillStep>,
    pub pass: u32,
}

impl Default for StrategyExecutionIdentity {
    fn default() -> Self {
        Self {
            bar_index: 0,
            phase: StrategyBarPhase::EligibleEntryFills,
            fill_step: None,
            pass: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StrategySchedulerState {
    pub(crate) identity: StrategyExecutionIdentity,
    max_recalculation_passes: u32,
    script_passes: usize,
    recalculation_passes: usize,
    max_passes_on_bar: u32,
    current_bar_script_passes: u32,
}

impl Default for StrategySchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategySchedulerState {
    pub(crate) fn new() -> Self {
        Self::with_max_recalculation_passes(DEFAULT_MAX_RECALCULATION_PASSES)
    }

    pub(crate) fn with_max_recalculation_passes(max_recalculation_passes: u32) -> Self {
        Self {
            identity: StrategyExecutionIdentity::default(),
            max_recalculation_passes,
            script_passes: 0,
            recalculation_passes: 0,
            max_passes_on_bar: 0,
            current_bar_script_passes: 0,
        }
    }

    pub(crate) fn script_passes(&self) -> usize {
        self.script_passes
    }

    pub(crate) fn recalculation_passes(&self) -> usize {
        self.recalculation_passes
    }

    pub(crate) fn max_passes_on_bar(&self) -> u32 {
        self.max_passes_on_bar
    }

    pub(crate) fn max_recalculation_passes(&self) -> u32 {
        self.max_recalculation_passes
    }

    #[cfg(test)]
    pub(crate) fn set_max_recalculation_passes(&mut self, max_recalculation_passes: u32) {
        self.max_recalculation_passes = max_recalculation_passes;
    }

    pub(crate) fn begin_bar(&mut self, bar_index: usize) {
        self.identity.bar_index = bar_index;
        self.identity.phase = StrategyBarPhase::EligibleEntryFills;
        self.identity.fill_step = None;
        self.identity.pass = 0;
        self.current_bar_script_passes = 0;
    }

    pub(crate) fn set_phase(&mut self, phase: StrategyBarPhase) {
        self.identity.phase = phase;
        if !matches!(
            phase,
            StrategyBarPhase::EligibleEntryFills
                | StrategyBarPhase::BarCloseMarketFills
                | StrategyBarPhase::CurrentTickMarketFills
        ) {
            self.identity.fill_step = None;
        }
    }

    pub(crate) fn set_fill_step(&mut self, step: HistoricalFillStep) {
        self.identity.fill_step = Some(step);
        self.identity.phase = if matches!(
            step,
            HistoricalFillStep::SameBarMarketClosesAtClose
                | HistoricalFillStep::SameBarMarketEntriesAtClose
        ) {
            StrategyBarPhase::BarCloseMarketFills
        } else {
            StrategyBarPhase::EligibleEntryFills
        };
    }

    pub(crate) fn begin_script_pass(&mut self) -> Result<(), RuntimeError> {
        if self.current_bar_script_passes == 0 {
            self.identity.pass = 0;
            self.identity.phase = StrategyBarPhase::ScriptStatements;
            self.identity.fill_step = None;
            self.current_bar_script_passes = 1;
            self.script_passes += 1;
            self.max_passes_on_bar = self.max_passes_on_bar.max(1);
            return Ok(());
        }

        let extra_pass = self.current_bar_script_passes;
        if extra_pass > self.max_recalculation_passes {
            return Err(RuntimeError {
                message: format!(
                    "strategy recalculation pass limit exceeded: bar {} {:?} {:?} pass {} (limit {} extra passes)",
                    self.identity.bar_index,
                    self.identity.phase,
                    self.identity.fill_step,
                    extra_pass,
                    self.max_recalculation_passes
                ),
            });
        }

        self.identity.pass = extra_pass;
        self.identity.phase = StrategyBarPhase::ScriptStatements;
        self.identity.fill_step = None;
        self.current_bar_script_passes += 1;
        self.script_passes += 1;
        self.recalculation_passes += 1;
        self.max_passes_on_bar = self.max_passes_on_bar.max(self.current_bar_script_passes);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrategyBarPhase {
    EligibleEntryFills,
    TradeExtremes,
    MarginCall,
    BuiltinRefresh,
    ScriptStatements,
    CurrentTickMarketFills,
    BarCloseMarketFills,
    ExitFills,
    Equity,
    OutputCommit,
}

/// Deterministic historical fill-path steps. Discriminant order is the path
/// tick used when several families are eligible on the same bar: market closes
/// and entries at open, then long price families, then short price families.
/// Same-tick pyramiding batches stay inside one step so eligible limit/stop
/// entries can still fill together. Host-owned bar-magnifier intrabars, when
/// later wired, reuse this path against each lower-timeframe tick instead of
/// adding a second broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HistoricalFillStep {
    MarketClosesAtOpen,
    MarketEntriesAtOpen,
    LimitLong,
    StopLong,
    StopLimitLong,
    LimitShort,
    StopShort,
    StopLimitShort,
    SameBarMarketClosesAtClose,
    SameBarMarketEntriesAtClose,
}

impl HistoricalFillStep {
    fn pre_script_path() -> &'static [Self] {
        &[
            Self::MarketClosesAtOpen,
            Self::MarketEntriesAtOpen,
            Self::LimitLong,
            Self::StopLong,
            Self::StopLimitLong,
            Self::LimitShort,
            Self::StopShort,
            Self::StopLimitShort,
        ]
    }

    fn bar_close_path() -> &'static [Self] {
        &[
            Self::SameBarMarketClosesAtClose,
            Self::SameBarMarketEntriesAtClose,
        ]
    }

    pub(crate) fn ordering_key(self) -> (u8, u8) {
        (self.phase_rank(), self as u8)
    }

    fn phase_rank(self) -> u8 {
        match self {
            Self::MarketClosesAtOpen
            | Self::MarketEntriesAtOpen
            | Self::LimitLong
            | Self::StopLong
            | Self::StopLimitLong
            | Self::LimitShort
            | Self::StopShort
            | Self::StopLimitShort => 0,
            Self::SameBarMarketClosesAtClose | Self::SameBarMarketEntriesAtClose => 1,
        }
    }
}

impl HistoricalRuntime<'_> {
    pub(crate) fn run_pre_script_strategy_phases(
        &mut self,
        bar_index: usize,
        bar: Bar,
    ) -> Result<(), RuntimeError> {
        if self.program.script_mode != ScriptMode::Strategy {
            return Ok(());
        }
        let timeframe_seconds =
            crate::builtins::time::timeframe_seconds(crate::DEFAULT_CHART_TIMEFRAME).unwrap_or(0);
        let mark = bar.open;
        let equity = self.strategy_broker.equity_value(mark);
        self.strategy_broker.reset_intraday_window(
            bar_index,
            bar.time,
            timeframe_seconds,
            equity,
            mark,
        );
        self.trace_strategy_phase(StrategyBarPhase::EligibleEntryFills);
        let mut steps: Vec<_> = HistoricalFillStep::pre_script_path().to_vec();
        steps.sort_by_key(|step| step.ordering_key());
        for step in steps {
            self.strategy_scheduler.set_fill_step(step);
            let filled = self.apply_historical_fill_step(step, bar_index, bar);
            if filled {
                self.strategy_broker
                    .flatten_if_risk_blocked(bar_index, bar.time, bar.open);
            }
            self.recalculate_after_fill(filled)?;
        }
        self.strategy_broker
            .evaluate_risk_equity_stops(bar_index, bar.time, bar.open);
        self.trace_strategy_phase(StrategyBarPhase::TradeExtremes);
        self.strategy_broker
            .update_open_trade_extremes(bar.high, bar.low);
        let adverse = if self.strategy_broker.position_size() > 0.0 {
            bar.low
        } else if self.strategy_broker.position_size() < 0.0 {
            bar.high
        } else {
            bar.close
        };
        self.strategy_broker
            .evaluate_risk_equity_stops(bar_index, bar.time, adverse);
        self.trace_strategy_phase(StrategyBarPhase::MarginCall);
        let before_margin = self.strategy_broker.public_order_event_count();
        self.strategy_broker
            .evaluate_margin_call_long(bar_index, bar.time, bar.low);
        self.strategy_broker
            .evaluate_margin_call_short(bar_index, bar.time, bar.high);
        self.strategy_broker
            .flatten_if_risk_blocked(bar_index, bar.time, adverse);
        self.recalculate_after_fill(
            self.strategy_broker.public_order_event_count() > before_margin,
        )?;
        self.strategy_broker
            .evaluate_risk_equity_stops(bar_index, bar.time, adverse);
        Ok(())
    }

    fn apply_historical_fill_step(
        &mut self,
        step: HistoricalFillStep,
        bar_index: usize,
        bar: Bar,
    ) -> bool {
        let before = self.strategy_broker.public_order_event_count();
        match step {
            HistoricalFillStep::MarketClosesAtOpen => {
                self.strategy_broker
                    .fill_pending_market_closes(bar_index, bar.time, bar.open);
            }
            HistoricalFillStep::MarketEntriesAtOpen => {
                self.strategy_broker
                    .fill_pending_market_entries(bar_index, bar.time, bar.open);
            }
            HistoricalFillStep::LimitLong => {
                self.strategy_broker
                    .fill_pending_limit_long_entries(bar_index, bar.time, bar.low);
            }
            HistoricalFillStep::StopLong => {
                self.strategy_broker
                    .fill_pending_stop_long_entries(bar_index, bar.time, bar.high);
            }
            HistoricalFillStep::StopLimitLong => {
                self.strategy_broker
                    .fill_pending_stop_limit_long_entries(bar_index, bar.time, bar.high, bar.low);
            }
            HistoricalFillStep::LimitShort => {
                self.strategy_broker
                    .fill_pending_limit_short_entries(bar_index, bar.time, bar.high);
            }
            HistoricalFillStep::StopShort => {
                self.strategy_broker
                    .fill_pending_stop_short_entries(bar_index, bar.time, bar.low);
            }
            HistoricalFillStep::StopLimitShort => {
                self.strategy_broker
                    .fill_pending_stop_limit_short_entries(bar_index, bar.time, bar.high, bar.low);
            }
            HistoricalFillStep::SameBarMarketClosesAtClose => {
                self.strategy_broker
                    .fill_same_bar_market_closes(bar_index, bar.time, bar.close);
            }
            HistoricalFillStep::SameBarMarketEntriesAtClose => {
                self.strategy_broker
                    .fill_same_bar_market_entries(bar_index, bar.time, bar.close);
            }
        }
        self.strategy_broker.public_order_event_count() > before
    }

    pub(crate) fn fill_current_tick_market_closes(&mut self) {
        let Some(bar) = self.current_bar else {
            return;
        };
        if self.program.script_mode != ScriptMode::Strategy {
            return;
        }
        self.trace_strategy_phase(StrategyBarPhase::CurrentTickMarketFills);
        self.strategy_broker
            .fill_immediate_market_closes(self.bars, bar.time, bar.close);
    }

    pub(crate) fn run_post_script_strategy_phases(
        &mut self,
        bar_index: usize,
        bar: Bar,
    ) -> Result<(), RuntimeError> {
        if self.program.script_mode != ScriptMode::Strategy {
            return Ok(());
        }
        if self.program.strategy_settings.process_orders_on_close {
            self.trace_strategy_phase(StrategyBarPhase::BarCloseMarketFills);
            let mut steps: Vec<_> = HistoricalFillStep::bar_close_path().to_vec();
            steps.sort_by_key(|step| step.ordering_key());
            for step in steps {
                self.strategy_scheduler.set_fill_step(step);
                let filled = self.apply_historical_fill_step(step, bar_index, bar);
                if filled {
                    self.strategy_broker
                        .flatten_if_risk_blocked(bar_index, bar.time, bar.close);
                }
                self.recalculate_after_fill(filled)?;
            }
        }
        self.trace_strategy_phase(StrategyBarPhase::ExitFills);
        let before_exits = self.strategy_broker.public_order_event_count();
        self.strategy_broker
            .evaluate_pending_exits(bar_index, bar.time, bar.high, bar.low);
        self.strategy_broker
            .flatten_if_risk_blocked(bar_index, bar.time, bar.close);
        self.recalculate_after_fill(
            self.strategy_broker.public_order_event_count() > before_exits,
        )?;
        self.strategy_broker
            .evaluate_risk_equity_stops(bar_index, bar.time, bar.close);
        self.trace_strategy_phase(StrategyBarPhase::Equity);
        self.strategy_broker.record_equity(bar_index, bar.close);
        Ok(())
    }

    pub(crate) fn trace_strategy_phase(&mut self, phase: StrategyBarPhase) {
        self.strategy_scheduler.set_phase(phase);
        #[cfg(test)]
        self.strategy_phase_trace.push(phase);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_bar_resets_pass_identity() {
        let mut scheduler = StrategySchedulerState::new();
        scheduler.begin_bar(3);
        scheduler.begin_script_pass().expect("initial pass");
        scheduler.begin_bar(4);
        assert_eq!(scheduler.identity.bar_index, 4);
        assert_eq!(scheduler.identity.pass, 0);
        assert_eq!(
            scheduler.identity.phase,
            StrategyBarPhase::EligibleEntryFills
        );
        assert_eq!(scheduler.identity.fill_step, None);
        assert_eq!(scheduler.script_passes(), 1);
    }

    #[test]
    fn fill_step_is_part_of_tick_identity() {
        let mut scheduler = StrategySchedulerState::new();
        scheduler.begin_bar(1);
        scheduler.set_fill_step(HistoricalFillStep::LimitLong);
        assert_eq!(
            scheduler.identity.fill_step,
            Some(HistoricalFillStep::LimitLong)
        );
        assert_eq!(
            scheduler.identity.phase,
            StrategyBarPhase::EligibleEntryFills
        );
        scheduler.set_fill_step(HistoricalFillStep::SameBarMarketClosesAtClose);
        assert_eq!(
            scheduler.identity.phase,
            StrategyBarPhase::BarCloseMarketFills
        );
        scheduler.set_phase(StrategyBarPhase::ScriptStatements);
        assert_eq!(scheduler.identity.fill_step, None);
    }

    #[test]
    fn default_historical_path_allows_one_script_pass_per_bar() {
        let mut scheduler = StrategySchedulerState::new();
        for bar_index in 0..4 {
            scheduler.begin_bar(bar_index);
            scheduler.begin_script_pass().expect("initial pass");
        }
        assert_eq!(scheduler.script_passes(), 4);
        assert_eq!(scheduler.recalculation_passes(), 0);
        assert_eq!(scheduler.max_passes_on_bar(), 1);
        assert_eq!(
            scheduler.max_recalculation_passes(),
            DEFAULT_MAX_RECALCULATION_PASSES
        );
        assert_eq!(scheduler.identity.bar_index, 3);
        assert_eq!(scheduler.identity.pass, 0);
    }

    #[test]
    fn bounded_self_triggering_order_loop_fails_closed() {
        let mut scheduler = StrategySchedulerState::with_max_recalculation_passes(3);
        scheduler.begin_bar(4);
        scheduler.begin_script_pass().expect("initial pass");
        for extra in 1..=3 {
            scheduler
                .begin_script_pass()
                .unwrap_or_else(|_| panic!("extra pass {extra} should stay under the limit"));
            assert_eq!(scheduler.identity.pass, extra);
            assert_eq!(scheduler.identity.phase, StrategyBarPhase::ScriptStatements);
        }
        let error = scheduler
            .begin_script_pass()
            .expect_err("self-triggering extra pass should hit the guardrail");
        assert!(
            error
                .message
                .contains("strategy recalculation pass limit exceeded"),
            "{}",
            error.message
        );
        assert!(error.message.contains("bar 4"), "{}", error.message);
        assert!(error.message.contains("pass 4"), "{}", error.message);
        assert!(
            error.message.contains("limit 3 extra passes"),
            "{}",
            error.message
        );
        assert_eq!(scheduler.script_passes(), 4);
        assert_eq!(scheduler.recalculation_passes(), 3);
        assert_eq!(scheduler.max_passes_on_bar(), 4);
        assert_eq!(scheduler.identity.pass, 3);
    }

    #[test]
    fn zero_recalculation_limit_rejects_any_extra_pass() {
        let mut scheduler = StrategySchedulerState::with_max_recalculation_passes(0);
        scheduler.begin_bar(0);
        scheduler.begin_script_pass().expect("initial pass");
        let error = scheduler
            .begin_script_pass()
            .expect_err("max 0 extra passes must reject the first recalculation");
        assert!(
            error
                .message
                .contains("strategy recalculation pass limit exceeded"),
            "{}",
            error.message
        );
        assert_eq!(scheduler.recalculation_passes(), 0);
        assert_eq!(scheduler.max_passes_on_bar(), 1);
    }

    #[test]
    fn max_recalculation_passes_is_configurable() {
        let mut scheduler = StrategySchedulerState::new();
        scheduler.set_max_recalculation_passes(1);
        scheduler.begin_bar(2);
        scheduler.begin_script_pass().expect("initial pass");
        scheduler.begin_script_pass().expect("one extra pass");
        scheduler
            .begin_script_pass()
            .expect_err("configured limit of 1 extra pass");
        assert_eq!(scheduler.max_recalculation_passes(), 1);
        assert_eq!(scheduler.recalculation_passes(), 1);
    }
}
