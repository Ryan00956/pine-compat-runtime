use std::cmp::Ordering;

use pine_ir::{SeriesId, SymbolId, VarSlotId};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn set_builtin_symbols(
        &mut self,
        bar: &Bar,
        bar_index: usize,
    ) -> Result<(), RuntimeError> {
        let datetime = utc_datetime_from_millis(bar.time)?;
        let chart_duration_ms = timeframe_seconds(DEFAULT_CHART_TIMEFRAME)
            .and_then(|seconds| seconds.checked_mul(1000))
            .ok_or_else(|| RuntimeError {
                message: "default chart timeframe duration is invalid".to_owned(),
            })?;
        let time_close = bar
            .time
            .checked_add(chart_duration_ms)
            .ok_or_else(|| RuntimeError {
                message: format!("time_close timestamp is out of range: {}", bar.time),
            })?;
        let previous_close = self.price_flow_previous_close;
        let previous_volume = self.price_flow_previous_volume;
        self.accdist_current = self.next_accdist(bar);
        self.iii_current = Self::iii_value(bar);
        self.nvi_current = self.next_nvi(bar, previous_close, previous_volume);
        self.obv_current = self.next_obv(bar, previous_close);
        self.pvi_current = self.next_pvi(bar, previous_close, previous_volume);
        self.pvt_current = self.next_pvt(bar, previous_close);
        self.vwap_current = self.next_vwap(bar);
        self.wad_current = self.next_wad(bar, previous_close);
        self.wvad_current = Self::wvad_value(bar);
        self.price_flow_previous_close = Some(bar.close);
        self.price_flow_previous_volume = Some(bar.volume);
        let builtins = [
            ("open", PineValue::Float(bar.open)),
            ("high", PineValue::Float(bar.high)),
            ("low", PineValue::Float(bar.low)),
            ("close", PineValue::Float(bar.close)),
            ("volume", PineValue::Float(bar.volume)),
            ("time", PineValue::Int(bar.time)),
            ("time_close", PineValue::Int(time_close)),
            ("year", PineValue::Int(datetime.year() as i64)),
            ("month", PineValue::Int(datetime.month() as i64)),
            (
                "weekofyear",
                PineValue::Int(datetime.iso_week().week() as i64),
            ),
            ("dayofmonth", PineValue::Int(datetime.day() as i64)),
            ("dayofweek", PineValue::Int(dayofweek_value(datetime))),
            ("hour", PineValue::Int(datetime.hour() as i64)),
            ("minute", PineValue::Int(datetime.minute() as i64)),
            ("second", PineValue::Int(datetime.second() as i64)),
            ("hl2", PineValue::Float((bar.high + bar.low) / 2.0)),
            (
                "hlc3",
                PineValue::Float((bar.high + bar.low + bar.close) / 3.0),
            ),
            (
                "hlcc4",
                PineValue::Float((bar.high + bar.low + bar.close + bar.close) / 4.0),
            ),
            (
                "ohlc4",
                PineValue::Float((bar.open + bar.high + bar.low + bar.close) / 4.0),
            ),
            ("bar_index", PineValue::Int(bar_index as i64)),
        ];

        for (name, value) in builtins {
            let symbol = self
                .program
                .symbols
                .iter()
                .find(|symbol| symbol.name == name)
                .ok_or_else(|| RuntimeError {
                    message: format!("missing builtin symbol `{name}`"),
                })?;
            self.current_symbols.insert(symbol.id, value.clone());
            if let Some(series_id) = symbol.series_id {
                self.current_series.insert(series_id, value);
            }
        }

        Ok(())
    }

    pub(crate) fn next_accdist(&mut self, bar: &Bar) -> PineValue {
        let range = bar.high - bar.low;
        if range == 0.0 {
            self.accdist_state = PineValue::Na;
            return PineValue::Na;
        }

        let multiplier = ((bar.close - bar.low) - (bar.high - bar.close)) / range;
        let increment = multiplier * bar.volume;
        if !increment.is_finite() {
            self.accdist_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.accdist_state.as_f64().unwrap_or(0.0) + increment;
        self.accdist_state = finite_float_or_na(value);
        self.accdist_state.clone()
    }

    pub(crate) fn iii_value(bar: &Bar) -> PineValue {
        let denominator = (bar.high - bar.low) * bar.volume;
        if denominator == 0.0 {
            return PineValue::Na;
        }

        finite_float_or_na((2.0 * bar.close - bar.high - bar.low) / denominator)
    }

    pub(crate) fn next_nvi(
        &mut self,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
    ) -> PineValue {
        Self::next_volume_index(
            &mut self.nvi_state,
            bar,
            previous_close,
            previous_volume,
            |volume, previous_volume| volume < previous_volume,
        )
    }

    pub(crate) fn next_obv(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.obv_state = PineValue::Na;
            return PineValue::Na;
        };
        let signed_volume = match bar.close.partial_cmp(&previous_close) {
            Some(Ordering::Greater) => bar.volume,
            Some(Ordering::Less) => -bar.volume,
            Some(Ordering::Equal) => 0.0,
            None => {
                self.obv_state = PineValue::Na;
                return PineValue::Na;
            }
        };
        let value = self.obv_state.as_f64().unwrap_or(0.0) + signed_volume;
        self.obv_state = PineValue::Float(value);
        self.obv_state.clone()
    }

    pub(crate) fn next_pvi(
        &mut self,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
    ) -> PineValue {
        Self::next_volume_index(
            &mut self.pvi_state,
            bar,
            previous_close,
            previous_volume,
            |volume, previous_volume| volume > previous_volume,
        )
    }

    pub(crate) fn next_pvt(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        };
        if previous_close == 0.0 {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        }

        let increment = ((bar.close - previous_close) / previous_close) * bar.volume;
        if !increment.is_finite() {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.pvt_state.as_f64().unwrap_or(0.0) + increment;
        self.pvt_state = finite_float_or_na(value);
        self.pvt_state.clone()
    }

    pub(crate) fn next_vwap(&mut self, bar: &Bar) -> PineValue {
        let source = (bar.high + bar.low + bar.close) / 3.0;
        let weighted = source * bar.volume;
        if !source.is_finite() || !bar.volume.is_finite() || !weighted.is_finite() {
            self.vwap_weighted_sum = 0.0;
            self.vwap_volume_sum = 0.0;
            self.vwap_current = PineValue::Na;
            return PineValue::Na;
        }

        self.vwap_weighted_sum += weighted;
        self.vwap_volume_sum += bar.volume;
        if self.vwap_volume_sum == 0.0
            || !self.vwap_weighted_sum.is_finite()
            || !self.vwap_volume_sum.is_finite()
        {
            self.vwap_current = PineValue::Na;
            return PineValue::Na;
        }

        self.vwap_current = finite_float_or_na(self.vwap_weighted_sum / self.vwap_volume_sum);
        self.vwap_current.clone()
    }

    pub(crate) fn next_wad(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.wad_state = PineValue::Na;
            return PineValue::Na;
        };

        let momentum = bar.close - previous_close;
        let gain = match momentum.partial_cmp(&0.0) {
            Some(Ordering::Greater) => bar.close - bar.low.min(previous_close),
            Some(Ordering::Less) => bar.close - bar.high.max(previous_close),
            Some(Ordering::Equal) => 0.0,
            None => {
                self.wad_state = PineValue::Na;
                return PineValue::Na;
            }
        };
        if !gain.is_finite() {
            self.wad_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.wad_state.as_f64().unwrap_or(0.0) + gain;
        self.wad_state = finite_float_or_na(value);
        self.wad_state.clone()
    }

    pub(crate) fn next_volume_index(
        state: &mut PineValue,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
        should_update: impl FnOnce(f64, f64) -> bool,
    ) -> PineValue {
        let previous_value = state.as_f64().filter(|value| *value != 0.0).unwrap_or(1.0);
        let Some(previous_close) = previous_close else {
            *state = PineValue::Float(previous_value);
            return state.clone();
        };

        if bar.close == 0.0
            || previous_close == 0.0
            || !bar.close.is_finite()
            || !previous_close.is_finite()
            || !bar.volume.is_finite()
        {
            *state = PineValue::Float(previous_value);
            return state.clone();
        }

        let previous_volume = previous_volume
            .filter(|volume| volume.is_finite())
            .unwrap_or(0.0);
        if !should_update(bar.volume, previous_volume) {
            *state = PineValue::Float(previous_value);
            return state.clone();
        }

        let value =
            previous_value + ((bar.close - previous_close) / previous_close) * previous_value;
        *state = finite_float_or_na(value);
        state.clone()
    }

    pub(crate) fn wvad_value(bar: &Bar) -> PineValue {
        let range = bar.high - bar.low;
        if range == 0.0 {
            return PineValue::Na;
        }

        finite_float_or_na(((bar.close - bar.open) / range) * bar.volume)
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
        if name == "timeframe.period" {
            return PineValue::String(DEFAULT_CHART_TIMEFRAME.to_owned());
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

    pub(crate) fn eval_decl(
        &mut self,
        symbol: SymbolId,
        value: &HirExpr,
    ) -> Result<PineValue, RuntimeError> {
        let Some(var_slot_id) = self.var_slot_for_symbol(symbol) else {
            return self.eval_expr(value);
        };

        if let Some(value) = self.var_store.get(&var_slot_id).cloned() {
            Ok(value)
        } else {
            let value = self.eval_expr(value)?;
            self.var_store.insert(var_slot_id, value.clone());
            Ok(value)
        }
    }

    pub(crate) fn var_slot_for_symbol(&self, symbol_id: SymbolId) -> Option<VarSlotId> {
        self.program
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)
            .and_then(|symbol| symbol.var_slot_id)
    }

    pub(crate) fn series_id_for_symbol(&self, symbol_id: SymbolId) -> Option<SeriesId> {
        self.program
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)
            .and_then(|symbol| symbol.series_id)
    }

    pub(crate) fn set_symbol_value(&mut self, symbol: SymbolId, value: PineValue) {
        self.current_symbols.insert(symbol, value.clone());
        if let Some(series_id) = self.series_id_for_symbol(symbol) {
            self.current_series.insert(series_id, value);
        }
    }

    pub(crate) fn commit_current_series(&mut self) -> Result<(), RuntimeError> {
        if self.projected_series_values_after_commit() > MAX_SERIES_HISTORY_VALUES {
            return Err(RuntimeError {
                message: format!(
                    "series history limit exceeded: at most {MAX_SERIES_HISTORY_VALUES} committed values are retained"
                ),
            });
        }

        for raw_series_id in 0..self.program.next_series_id {
            let series_id = SeriesId(raw_series_id);
            let value = self
                .current_series
                .remove(&series_id)
                .unwrap_or(PineValue::Na);
            self.series_store.commit(
                series_id,
                value,
                self.series_retention.max_depth_for(series_id),
            );
        }
        Ok(())
    }

    pub(crate) fn projected_series_values_after_commit(&self) -> usize {
        let mut total = 0usize;
        for raw_series_id in 0..self.program.next_series_id {
            let series_id = SeriesId(raw_series_id);
            let next_len = self.series_store.len(series_id).saturating_add(1);
            let retained_len = self
                .series_retention
                .max_depth_for(series_id)
                .map_or(next_len, |max_depth| next_len.min(max_depth));
            total = total.saturating_add(retained_len);
        }
        total
    }

    pub(crate) fn current_builtin_f64(&self, name: &str) -> Option<f64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        self.current_symbols.get(&symbol.id)?.as_f64()
    }

    pub(crate) fn builtin_f64_at(&self, name: &str, offset: usize) -> Option<f64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        let series_id = symbol.series_id?;
        self.series_store.read(series_id, offset).as_f64()
    }

    pub(crate) fn previous_builtin_f64(&self, name: &str) -> Option<f64> {
        self.builtin_f64_at(name, 1)
    }

    pub(crate) fn current_builtin_i64(&self, name: &str) -> Option<i64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        self.current_symbols.get(&symbol.id)?.as_i64()
    }

    pub(crate) fn previous_close(&self) -> Option<f64> {
        self.previous_builtin_f64("close")
    }
}
