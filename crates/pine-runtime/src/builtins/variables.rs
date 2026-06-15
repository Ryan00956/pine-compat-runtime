use pine_ir::{CallSiteId, HirCallArg};

use crate::*;

pub(crate) fn eval_static_builtin_value(name: &str) -> PineValue {
    if let Some(color) = pine_builtins::named_color(name) {
        return PineValue::Color(color);
    }
    if let Some(value) = pine_builtins::named_float_constant(name) {
        return PineValue::Float(value);
    }
    if let Some(value) = pine_builtins::named_int_constant(name) {
        return PineValue::Int(value);
    }
    pine_builtins::named_string_constant(name)
        .map(|constant| PineValue::String(constant.to_owned()))
        .unwrap_or(PineValue::Void)
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_variable_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "indicator" | "strategy" => Ok(PineValue::Void),
            "input" | "input.int" | "input.float" | "input.bool" | "input.color"
            | "input.string" | "input.price" | "input.time" | "input.symbol"
            | "input.timeframe" | "input.session" | "input.text_area" | "input.source" => {
                self.eval_input(args)
            }
            "na" => self.eval_na(args),
            "nz" => self.eval_nz(args),
            "fixnan" => self.eval_fixnan(call_site_id, args),
            _ => return None,
        })
    }

    pub(crate) fn eval_input(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        self.eval_expr(&args[0].value)
    }

    pub(crate) fn eval_na(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        Ok(PineValue::Bool(value.is_na()))
    }

    pub(crate) fn eval_nz(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        if value.is_na() {
            if let Some(replacement) = args.get(1) {
                self.eval_expr(&replacement.value)
            } else {
                Ok(PineValue::Int(0))
            }
        } else {
            Ok(value)
        }
    }

    pub(crate) fn eval_builtin_value(&mut self, name: &str) -> PineValue {
        if name == "barstate.isfirst" {
            return PineValue::Bool(self.bars == 0);
        }
        if name == "barstate.islast" {
            return PineValue::Bool(self.is_latest_known_bar());
        }
        if name == "barstate.islastconfirmedhistory" {
            return PineValue::Bool(self.is_last_confirmed_history_bar());
        }
        if name == "barstate.isnew" {
            return PineValue::Bool(self.current_bar_is_new);
        }
        if name == "barstate.isconfirmed" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Historical | BarUpdateKind::Confirmed
            ));
        }
        if name == "barstate.ishistory" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Historical
            ));
        }
        if name == "barstate.isrealtime" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Forming | BarUpdateKind::Confirmed
            ));
        }
        if name == "session.ismarket" {
            return PineValue::Bool(true);
        }
        if name == "session.ispremarket" || name == "session.ispostmarket" {
            return PineValue::Bool(false);
        }
        if matches!(name, "session.isfirstbar" | "session.isfirstbar_regular") {
            return PineValue::Bool(self.bars == 0);
        }
        if matches!(name, "session.islastbar" | "session.islastbar_regular") {
            return PineValue::Bool(self.is_latest_known_bar());
        }
        if name == "last_bar_index" {
            return self
                .last_bar_index
                .map_or(PineValue::Na, |index| PineValue::Int(index as i64));
        }
        if name == "last_bar_time" {
            return self.last_bar_time.map_or(PineValue::Na, PineValue::Int);
        }
        if name == "syminfo.tickerid" {
            return PineValue::String(self.request_environment.chart().symbol().to_owned());
        }
        if matches!(name, "timeframe.period" | "timeframe.main_period") {
            return PineValue::String(
                self.request_environment
                    .chart()
                    .timeframe()
                    .value()
                    .to_owned(),
            );
        }
        if name == "timeframe.isseconds" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isminutes" {
            return PineValue::Bool(true);
        }
        if name == "timeframe.isintraday" {
            return PineValue::Bool(true);
        }
        if name == "timeframe.isdaily" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isweekly" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.ismonthly" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isdwm" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.multiplier" {
            return PineValue::Int(1);
        }
        if name == "chart.left_visible_bar_time" {
            return self
                .chart_visible_left_time
                .map_or(PineValue::Na, PineValue::Int);
        }
        if name == "chart.right_visible_bar_time" {
            return self
                .chart_visible_right_time
                .map_or(PineValue::Na, PineValue::Int);
        }
        if name == "chart.bg_color" {
            return PineValue::Color(0xFFFFFF);
        }
        if name == "chart.fg_color" {
            return PineValue::Color(0x000000);
        }
        if name == "chart.is_standard" {
            return PineValue::Bool(true);
        }
        if matches!(
            name,
            "chart.is_heikinashi"
                | "chart.is_kagi"
                | "chart.is_linebreak"
                | "chart.is_pnf"
                | "chart.is_range"
                | "chart.is_renko"
        ) {
            return PineValue::Bool(false);
        }
        if name == "label.all" {
            let labels = self
                .labels
                .iter()
                .filter(|label| {
                    label
                        .snapshots
                        .last()
                        .is_some_and(|snapshot| snapshot.exists)
                })
                .map(|label| PineValue::Label(label.id))
                .collect();
            return self.new_array_from_values(ArrayElementKind::Label, labels);
        }
        if name == "strategy.position_size" {
            return PineValue::Float(self.strategy_broker.position_size());
        }
        if name == "strategy.position_avg_price" {
            return self.strategy_broker.position_avg_price_value();
        }
        if name == "strategy.closedtrades" {
            return PineValue::Int(self.strategy_broker.closed_trade_count());
        }
        if name == "strategy.wintrades" {
            return PineValue::Int(self.strategy_broker.winning_trade_count());
        }
        if name == "strategy.losstrades" {
            return PineValue::Int(self.strategy_broker.losing_trade_count());
        }
        if name == "strategy.eventrades" {
            return PineValue::Int(self.strategy_broker.even_trade_count());
        }
        if name == "strategy.opentrades" {
            return PineValue::Int(self.strategy_broker.open_trade_count());
        }
        if name == "strategy.opentrades.capital_held" {
            return self.current_bar.map_or(PineValue::Na, |bar| {
                self.strategy_broker
                    .open_trade_capital_held(bar.close)
                    .map_or(PineValue::Na, PineValue::Float)
            });
        }
        if name == "strategy.openprofit" {
            return self.current_bar.map_or(PineValue::Na, |bar| {
                PineValue::Float(self.strategy_broker.open_profit(bar.close))
            });
        }
        if name == "strategy.netprofit" {
            return PineValue::Float(self.strategy_broker.realized_profit());
        }
        if name == "strategy.netprofit_percent" {
            return PineValue::Float(self.strategy_broker.realized_profit_percent());
        }
        if name == "strategy.grossprofit" {
            return PineValue::Float(self.strategy_broker.gross_profit());
        }
        if name == "strategy.grossprofit_percent" {
            return PineValue::Float(self.strategy_broker.gross_profit_percent());
        }
        if name == "strategy.grossloss" {
            return PineValue::Float(self.strategy_broker.gross_loss());
        }
        if name == "strategy.grossloss_percent" {
            return PineValue::Float(self.strategy_broker.gross_loss_percent());
        }
        if name == "strategy.avg_trade" {
            return self
                .strategy_broker
                .average_trade()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.avg_trade_percent" {
            return self
                .strategy_broker
                .average_trade_percent()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.avg_winning_trade" {
            return self
                .strategy_broker
                .average_winning_trade()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.avg_winning_trade_percent" {
            return self
                .strategy_broker
                .average_winning_trade_percent()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.avg_losing_trade" {
            return self
                .strategy_broker
                .average_losing_trade()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.avg_losing_trade_percent" {
            return self
                .strategy_broker
                .average_losing_trade_percent()
                .map_or(PineValue::Na, PineValue::Float);
        }
        if name == "strategy.max_runup" {
            return PineValue::Float(self.strategy_broker.max_runup());
        }
        if name == "strategy.max_runup_percent" {
            return PineValue::Float(self.strategy_broker.max_runup_percent());
        }
        if name == "strategy.max_drawdown" {
            return PineValue::Float(self.strategy_broker.max_drawdown());
        }
        if name == "strategy.max_drawdown_percent" {
            return PineValue::Float(self.strategy_broker.max_drawdown_percent());
        }
        if name == "strategy.max_contracts_held_all" {
            return PineValue::Float(self.strategy_broker.max_contracts_held_all());
        }
        if name == "strategy.max_contracts_held_long" {
            return PineValue::Float(self.strategy_broker.max_contracts_held_long());
        }
        if name == "strategy.max_contracts_held_short" {
            return PineValue::Float(self.strategy_broker.max_contracts_held_short());
        }
        if name == "strategy.equity" {
            return self.current_bar.map_or(PineValue::Na, |bar| {
                PineValue::Float(self.strategy_broker.equity_value(bar.close))
            });
        }
        if name == "ta.accdist" {
            return self.accdist_current.clone();
        }
        if name == "ta.iii" {
            return self.iii_current.clone();
        }
        if name == "ta.nvi" {
            return self.nvi_current.clone();
        }
        if name == "ta.obv" {
            return self.obv_current.clone();
        }
        if name == "ta.pvi" {
            return self.pvi_current.clone();
        }
        if name == "ta.pvt" {
            return self.pvt_current.clone();
        }
        if name == "ta.tr" {
            return self.true_range(false);
        }
        if name == "ta.vwap" {
            return self.vwap_current.clone();
        }
        if name == "ta.wad" {
            return self.wad_current.clone();
        }
        if name == "ta.wvad" {
            return self.wvad_current.clone();
        }
        eval_static_builtin_value(name)
    }

    fn is_latest_known_bar(&self) -> bool {
        match self.current_bar_update_kind {
            BarUpdateKind::Historical => self
                .historical_end
                .is_none_or(|historical_end| self.bars + 1 == historical_end),
            BarUpdateKind::Forming | BarUpdateKind::Confirmed => true,
        }
    }

    fn is_last_confirmed_history_bar(&self) -> bool {
        match self.current_bar_update_kind {
            BarUpdateKind::Historical => self.is_latest_known_bar(),
            BarUpdateKind::Forming | BarUpdateKind::Confirmed => false,
        }
    }
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_fixnan(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        if value.is_na() {
            Ok(self
                .call_state
                .get(&call_site_id)
                .cloned()
                .unwrap_or(PineValue::Na))
        } else {
            self.call_state.insert(call_site_id, value.clone());
            Ok(value)
        }
    }
}
