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

    pub(crate) fn eval_builtin_value(&self, name: &str) -> PineValue {
        if name == "barstate.isfirst" {
            return PineValue::Bool(self.bars == 0);
        }
        if name == "barstate.islast" {
            let is_last = match self.current_bar_update_kind {
                BarUpdateKind::Historical => self
                    .historical_end
                    .is_none_or(|historical_end| self.bars + 1 == historical_end),
                BarUpdateKind::Forming | BarUpdateKind::Confirmed => true,
            };
            return PineValue::Bool(is_last);
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
        if name == "syminfo.tickerid" {
            return PineValue::String(self.request_environment.chart().symbol().to_owned());
        }
        if name == "timeframe.period" {
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
        if name == "strategy.openprofit" {
            return self.current_bar.map_or(PineValue::Na, |bar| {
                PineValue::Float(self.strategy_broker.open_profit(bar.close))
            });
        }
        if name == "strategy.netprofit" {
            return PineValue::Float(self.strategy_broker.realized_profit());
        }
        if name == "strategy.grossprofit" {
            return PineValue::Float(self.strategy_broker.gross_profit());
        }
        if name == "strategy.grossloss" {
            return PineValue::Float(self.strategy_broker.gross_loss());
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
