use super::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_cum(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let Some(source) = source.as_f64() else {
            self.call_state.insert(call_site_id, PineValue::Na);
            return Ok(PineValue::Na);
        };

        let value = self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_f64)
            .unwrap_or(0.0)
            + source;
        let value = PineValue::Float(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    pub(crate) fn eval_all_time_extreme(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let Some(source) = source.as_f64() else {
            return Ok(self
                .call_state
                .get(&call_site_id)
                .cloned()
                .unwrap_or(PineValue::Na));
        };

        let value = match self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_f64)
        {
            Some(previous) => match mode {
                WindowExtreme::Highest => previous.max(source),
                WindowExtreme::Lowest => previous.min(source),
            },
            None => source,
        };
        let value = finite_float_or_na(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    pub(crate) fn eval_cci(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(current) = source.as_f64() else {
            self.update_rolling_window(call_site_id, source, length as usize);
            return Ok(PineValue::Na);
        };

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, PineValue::Float(current), length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let deviation = window.mean_absolute_deviation(length);
        if deviation == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            (current - window.mean(length)) / (0.015 * deviation),
        ))
    }

    pub(crate) fn eval_cog(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) || window.sum == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.center_of_gravity(length)))
    }

    pub(crate) fn eval_vwma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let Some(source) = source.as_f64() else {
            self.update_rolling_window_key(
                RollingWindowKey::VwmaWeighted(call_site_id),
                None,
                length,
            );
            self.update_rolling_window_key(
                RollingWindowKey::VwmaVolume(call_site_id),
                None,
                length,
            );
            return Ok(PineValue::Na);
        };
        let Some(volume) = self.current_builtin_f64("volume") else {
            self.update_rolling_window_key(
                RollingWindowKey::VwmaWeighted(call_site_id),
                None,
                length,
            );
            self.update_rolling_window_key(
                RollingWindowKey::VwmaVolume(call_site_id),
                None,
                length,
            );
            return Ok(PineValue::Na);
        };

        self.update_rolling_window_key(
            RollingWindowKey::VwmaWeighted(call_site_id),
            Some(source * volume),
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::VwmaVolume(call_site_id),
            Some(volume),
            length,
        );

        let weighted = self
            .rolling_windows
            .get(&RollingWindowKey::VwmaWeighted(call_site_id));
        let volumes = self
            .rolling_windows
            .get(&RollingWindowKey::VwmaVolume(call_site_id));
        let (Some(weighted), Some(volumes)) = (weighted, volumes) else {
            return Ok(PineValue::Na);
        };
        if !weighted.is_ready(length) || !volumes.is_ready(length) || volumes.sum == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(weighted.sum / volumes.sum))
    }

    pub(crate) fn eval_mfi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let Some(source) = source.as_f64() else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };
        let Some(volume) = self.current_builtin_f64("volume") else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };
        let Some(series_id) = args[0].value.series_id else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };

        let (positive_flow, negative_flow) = match self.series_store.read(series_id, 1).as_f64() {
            Some(previous) if source > previous => (Some(source * volume), Some(0.0)),
            Some(previous) if source < previous => (Some(0.0), Some(source * volume)),
            Some(_) | None => (Some(0.0), Some(0.0)),
        };
        self.update_mfi_windows(call_site_id, positive_flow, negative_flow, length);

        let positive_window = self
            .rolling_windows
            .get(&RollingWindowKey::MfiPositive(call_site_id));
        let negative_window = self
            .rolling_windows
            .get(&RollingWindowKey::MfiNegative(call_site_id));
        let (Some(positive_window), Some(negative_window)) = (positive_window, negative_window)
        else {
            return Ok(PineValue::Na);
        };
        if !positive_window.is_ready(length) || !negative_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let positive_sum = positive_window.sum;
        let negative_sum = negative_window.sum;
        if positive_sum == 0.0 && negative_sum == 0.0 {
            return Ok(PineValue::Na);
        }
        if negative_sum == 0.0 {
            return Ok(PineValue::Float(100.0));
        }

        Ok(finite_float_or_na(
            100.0 - 100.0 / (1.0 + positive_sum / negative_sum),
        ))
    }

    pub(crate) fn eval_vwap_source(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let has_bands = vwap_arg(args, 2, "stdev_mult").is_some();
        let source_arg = vwap_arg(args, 0, "source").ok_or_else(|| RuntimeError {
            message: "ta.vwap missing source argument".to_owned(),
        })?;
        let source = self.eval_expr(source_arg)?;
        let anchor = if let Some(arg) = vwap_arg(args, 1, "anchor") {
            matches!(self.eval_expr(arg)?, PineValue::Bool(true))
        } else {
            false
        };
        let stdev_mult = if let Some(arg) = vwap_arg(args, 2, "stdev_mult") {
            let Some(mult) = self.eval_expr(arg)?.as_f64() else {
                self.vwap_call_state.remove(&call_site_id);
                return Ok(vwap_result_na(has_bands));
            };
            Some(mult)
        } else {
            None
        };
        let (Some(source), Some(volume)) = (source.as_f64(), self.current_builtin_f64("volume"))
        else {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        };
        let weighted = source * volume;
        let weighted_square = source * source * volume;
        if !source.is_finite()
            || !volume.is_finite()
            || !weighted.is_finite()
            || !weighted_square.is_finite()
        {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        }
        if let Some(mult) = stdev_mult
            && !mult.is_finite()
        {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        }

        let state = self.vwap_call_state.entry(call_site_id).or_default();
        if anchor {
            *state = VwapState::default();
        }
        state.weighted_sum += weighted;
        state.weighted_square_sum += weighted_square;
        state.volume_sum += volume;
        if state.volume_sum == 0.0
            || !state.weighted_sum.is_finite()
            || !state.weighted_square_sum.is_finite()
            || !state.volume_sum.is_finite()
        {
            return Ok(vwap_result_na(has_bands));
        }

        let vwap = state.weighted_sum / state.volume_sum;
        let value = finite_float_or_na(vwap);
        let Some(mult) = stdev_mult else {
            return Ok(value);
        };
        let variance = (state.weighted_square_sum / state.volume_sum) - vwap * vwap;
        let deviation = variance.max(0.0).sqrt();
        let band = deviation * mult;
        Ok(PineValue::Tuple(vec![
            value,
            finite_float_or_na(vwap + band),
            finite_float_or_na(vwap - band),
        ]))
    }

    pub(crate) fn eval_stoch(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?.as_f64();
        let high = self.eval_expr(&args[1].value)?.as_f64();
        let low = self.eval_expr(&args[2].value)?.as_f64();
        let length = self.eval_expr(&args[3].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        self.update_rolling_window_key(RollingWindowKey::StochHigh(call_site_id), high, length);
        self.update_rolling_window_key(RollingWindowKey::StochLow(call_site_id), low, length);

        let high_window = self
            .rolling_windows
            .get(&RollingWindowKey::StochHigh(call_site_id));
        let low_window = self
            .rolling_windows
            .get(&RollingWindowKey::StochLow(call_site_id));
        let (Some(source), Some(high_window), Some(low_window)) = (source, high_window, low_window)
        else {
            return Ok(PineValue::Na);
        };
        if !high_window.is_ready(length) || !low_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let (Some(highest_high), Some(lowest_low)) = (
            high_window.extreme(WindowExtreme::Highest),
            low_window.extreme(WindowExtreme::Lowest),
        ) else {
            return Ok(PineValue::Na);
        };
        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(100.0 * (source - lowest_low) / range))
    }

    pub(crate) fn eval_wpr(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let close = self.current_builtin_f64("close");
        self.update_rolling_window_key(
            RollingWindowKey::WprHigh(call_site_id),
            self.current_builtin_f64("high"),
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::WprLow(call_site_id),
            self.current_builtin_f64("low"),
            length,
        );

        let high_window = self
            .rolling_windows
            .get(&RollingWindowKey::WprHigh(call_site_id));
        let low_window = self
            .rolling_windows
            .get(&RollingWindowKey::WprLow(call_site_id));
        let (Some(close), Some(high_window), Some(low_window)) = (close, high_window, low_window)
        else {
            return Ok(PineValue::Na);
        };
        if !high_window.is_ready(length) || !low_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let (Some(highest_high), Some(lowest_low)) = (
            high_window.extreme(WindowExtreme::Highest),
            low_window.extreme(WindowExtreme::Lowest),
        ) else {
            return Ok(PineValue::Na);
        };
        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(-100.0 * (highest_high - close) / range))
    }

    pub(crate) fn eval_ao(&mut self, call_site_id: CallSiteId) -> Result<PineValue, RuntimeError> {
        let source = match (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
        ) {
            (Some(high), Some(low)) => Some((high + low) / 2.0),
            _ => None,
        };

        self.update_rolling_window_key(RollingWindowKey::AoFast(call_site_id), source, 5);
        self.update_rolling_window_key(RollingWindowKey::AoSlow(call_site_id), source, 34);

        let fast_window = self
            .rolling_windows
            .get(&RollingWindowKey::AoFast(call_site_id));
        let slow_window = self
            .rolling_windows
            .get(&RollingWindowKey::AoSlow(call_site_id));
        let (Some(fast_window), Some(slow_window)) = (fast_window, slow_window) else {
            return Ok(PineValue::Na);
        };
        if !fast_window.is_ready(5) || !slow_window.is_ready(34) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            fast_window.mean(5) - slow_window.mean(34),
        ))
    }

    pub(crate) fn eval_bop(&self) -> Result<PineValue, RuntimeError> {
        let (Some(open), Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("open"),
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(PineValue::Na);
        };

        let range = high - low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na((close - open) / range))
    }

    pub(crate) fn eval_tr(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let handle_na = if let Some(arg) = args.first() {
            matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true))
        } else {
            true
        };

        Ok(self.true_range(handle_na))
    }

    pub(crate) fn eval_atr(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let true_range = self.true_range(true);
        let Some(true_range) = true_range.as_f64() else {
            return Ok(PineValue::Na);
        };
        let value = rma_next(
            self.call_state
                .get(&call_site_id)
                .and_then(PineValue::as_f64),
            true_range,
            length,
        );
        let value = PineValue::Float(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    pub(crate) fn eval_supertrend(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(factor) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(two_na_tuple());
        };
        let atr_period = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if atr_period <= 0 {
            return Ok(two_na_tuple());
        }

        let Some(true_range) = self.true_range(true).as_f64() else {
            return Ok(two_na_tuple());
        };
        let (Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(two_na_tuple());
        };

        let previous = supertrend_state(self.call_state.get(&call_site_id));
        let atr = rma_next(previous.map(|state| state.0), true_range, atr_period);
        let hl2 = (high + low) / 2.0;
        let basic_upper = hl2 + factor * atr;
        let basic_lower = hl2 - factor * atr;
        let previous_close = self.previous_close();

        let upper = match previous.zip(previous_close) {
            Some(((_, previous_upper, _, _), previous_close))
                if basic_upper >= previous_upper && previous_close <= previous_upper =>
            {
                previous_upper
            }
            _ => basic_upper,
        };
        let lower = match previous.zip(previous_close) {
            Some(((_, _, previous_lower, _), previous_close))
                if basic_lower <= previous_lower && previous_close >= previous_lower =>
            {
                previous_lower
            }
            _ => basic_lower,
        };

        let direction = match previous {
            None => 1.0,
            Some((_, previous_upper, _, previous_supertrend))
                if previous_supertrend == previous_upper && close > upper =>
            {
                -1.0
            }
            Some((_, previous_upper, _, previous_supertrend))
                if previous_supertrend == previous_upper =>
            {
                1.0
            }
            Some(_) if close < lower => 1.0,
            Some(_) => -1.0,
        };
        let supertrend = if direction < 0.0 { lower } else { upper };

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(atr),
                PineValue::Float(upper),
                PineValue::Float(lower),
                PineValue::Float(supertrend),
            ]),
        );

        Ok(PineValue::Tuple(vec![
            finite_float_or_na(supertrend),
            PineValue::Float(direction),
        ]))
    }

    pub(crate) fn eval_dmi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let di_length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        let adx_smoothing = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if di_length <= 0 || adx_smoothing <= 0 {
            return Ok(three_na_tuple());
        }

        let (Some(high), Some(low)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
        ) else {
            return Ok(three_na_tuple());
        };
        let Some(true_range) = self.true_range(true).as_f64() else {
            return Ok(three_na_tuple());
        };

        let (plus_dm, minus_dm) = match (
            self.previous_builtin_f64("high"),
            self.previous_builtin_f64("low"),
        ) {
            (Some(previous_high), Some(previous_low)) => {
                let up_move = high - previous_high;
                let down_move = previous_low - low;
                (
                    if up_move > down_move && up_move > 0.0 {
                        up_move
                    } else {
                        0.0
                    },
                    if down_move > up_move && down_move > 0.0 {
                        down_move
                    } else {
                        0.0
                    },
                )
            }
            _ => (0.0, 0.0),
        };

        let previous = dmi_state(self.call_state.get(&call_site_id));
        let smoothed_tr = rma_next(previous.map(|state| state.0), true_range, di_length);
        let smoothed_plus_dm = rma_next(previous.map(|state| state.1), plus_dm, di_length);
        let smoothed_minus_dm = rma_next(previous.map(|state| state.2), minus_dm, di_length);
        let (plus_di, minus_di) = if smoothed_tr.is_finite() && smoothed_tr != 0.0 {
            (
                100.0 * smoothed_plus_dm / smoothed_tr,
                100.0 * smoothed_minus_dm / smoothed_tr,
            )
        } else {
            (0.0, 0.0)
        };
        let di_sum = plus_di + minus_di;
        let dx = if di_sum.is_finite() && di_sum != 0.0 {
            100.0 * (plus_di - minus_di).abs() / di_sum
        } else {
            0.0
        };
        let adx = rma_next(previous.map(|state| state.3), dx, adx_smoothing);

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(smoothed_tr),
                PineValue::Float(smoothed_plus_dm),
                PineValue::Float(smoothed_minus_dm),
                PineValue::Float(adx),
            ]),
        );

        Ok(PineValue::Tuple(vec![
            finite_float_or_na(plus_di),
            finite_float_or_na(minus_di),
            finite_float_or_na(adx),
        ]))
    }

    pub(crate) fn eval_sar(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(start) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(increment) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(max_acceleration) = self.eval_expr(&args[2].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        if !start.is_finite() || !increment.is_finite() || !max_acceleration.is_finite() {
            return Ok(PineValue::Na);
        }

        let (Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(PineValue::Na);
        };

        let mut is_first_trend_bar = false;
        let (mut result, mut max_min, mut acceleration, mut is_below) =
            if let Some(state) = sar_state(self.call_state.get(&call_site_id)) {
                state
            } else {
                let (Some(previous_close), Some(previous_high), Some(previous_low)) = (
                    self.previous_builtin_f64("close"),
                    self.previous_builtin_f64("high"),
                    self.previous_builtin_f64("low"),
                ) else {
                    return Ok(PineValue::Na);
                };
                is_first_trend_bar = true;
                if close > previous_close {
                    (previous_low, high, start, true)
                } else {
                    (previous_high, low, start, false)
                }
            };

        result += acceleration * (max_min - result);
        if is_below {
            if result > low {
                is_first_trend_bar = true;
                is_below = false;
                result = high.max(max_min);
                max_min = low;
                acceleration = start;
            }
        } else if result < high {
            is_first_trend_bar = true;
            is_below = true;
            result = low.min(max_min);
            max_min = high;
            acceleration = start;
        }

        if !is_first_trend_bar {
            if is_below {
                if high > max_min {
                    max_min = high;
                    acceleration = (acceleration + increment).min(max_acceleration);
                }
            } else if low < max_min {
                max_min = low;
                acceleration = (acceleration + increment).min(max_acceleration);
            }
        }

        if is_below {
            if let Some(previous_low) = self.previous_builtin_f64("low") {
                result = result.min(previous_low);
            }
            if let Some(previous_previous_low) = self.builtin_f64_at("low", 2) {
                result = result.min(previous_previous_low);
            }
        } else {
            if let Some(previous_high) = self.previous_builtin_f64("high") {
                result = result.max(previous_high);
            }
            if let Some(previous_previous_high) = self.builtin_f64_at("high", 2) {
                result = result.max(previous_previous_high);
            }
        }

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(result),
                PineValue::Float(max_min),
                PineValue::Float(acceleration),
                PineValue::Bool(is_below),
            ]),
        );

        Ok(finite_float_or_na(result))
    }

    pub(crate) fn eval_change(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let current = self.eval_expr(&args[0].value)?;
        let length = if let Some(length_arg) = args.get(1) {
            self.eval_expr(&length_arg.value)?.as_i64().unwrap_or(1)
        } else {
            1
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(series_id) = args[0].value.series_id else {
            return Ok(PineValue::Na);
        };
        let previous = self.series_store.read(series_id, length as usize);

        match (current, previous) {
            (PineValue::Bool(current), PineValue::Bool(previous)) => {
                Ok(PineValue::Bool(current != previous))
            }
            (current, previous) => {
                let Some(current) = current.as_f64() else {
                    return Ok(PineValue::Na);
                };
                let Some(previous) = previous.as_f64() else {
                    return Ok(PineValue::Na);
                };
                Ok(PineValue::Float(current - previous))
            }
        }
    }

    pub(crate) fn eval_mom(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some((current, previous)) = self.current_and_previous(args)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Float(current - previous))
    }

    pub(crate) fn eval_roc(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some((current, previous)) = self.current_and_previous(args)? else {
            return Ok(PineValue::Na);
        };
        if previous == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(100.0 * (current - previous) / previous))
    }

    pub(crate) fn current_and_previous(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<Option<(f64, f64)>, RuntimeError> {
        let current = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(None);
        }

        let Some(current) = current.as_f64() else {
            return Ok(None);
        };
        let Some(series_id) = args[0].value.series_id else {
            return Ok(None);
        };
        let previous = self.series_store.read(series_id, length as usize);
        let Some(previous) = previous.as_f64() else {
            return Ok(None);
        };

        Ok(Some((current, previous)))
    }

    pub(crate) fn eval_rising_falling(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: RisingFallingMode,
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Bool(false));
        }

        let length = length as usize;
        let current = source.as_f64();
        let key = RollingWindowKey::RisingFalling(call_site_id);
        let value = if let Some(current) = current {
            self.rolling_windows
                .get(&key)
                .is_some_and(|window| window.is_ready(length) && window.trend(current, mode))
        } else {
            false
        };
        self.update_rolling_window_key(key, current, length);

        Ok(PineValue::Bool(value))
    }

    pub(crate) fn eval_cross(
        &mut self,
        args: &[HirCallArg],
        mode: CrossMode,
    ) -> Result<PineValue, RuntimeError> {
        let current_left = self.eval_expr(&args[0].value)?;
        let current_right = self.eval_expr(&args[1].value)?;
        let Some(left_series_id) = args[0].value.series_id else {
            return Ok(PineValue::Bool(false));
        };
        let previous_left = self.series_store.read(left_series_id, 1);
        let previous_right = if let Some(right_series_id) = args[1].value.series_id {
            self.series_store.read(right_series_id, 1)
        } else {
            current_right.clone()
        };

        let Some(current_left) = current_left.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(current_right) = current_right.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(previous_left) = previous_left.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(previous_right) = previous_right.as_f64() else {
            return Ok(PineValue::Bool(false));
        };

        let crossed_over = current_left > current_right && previous_left <= previous_right;
        let crossed_under = current_left < current_right && previous_left >= previous_right;
        Ok(PineValue::Bool(match mode {
            CrossMode::Any => crossed_over || crossed_under,
            CrossMode::Over => crossed_over,
            CrossMode::Under => crossed_under,
        }))
    }

    pub(crate) fn eval_barssince(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let condition = self.eval_expr(&args[0].value)?;
        let value = if matches!(condition, PineValue::Bool(true)) {
            PineValue::Int(0)
        } else if let Some(previous) = self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_i64)
        {
            PineValue::Int(previous + 1)
        } else {
            PineValue::Na
        };

        if matches!(value, PineValue::Int(_)) {
            self.call_state.insert(call_site_id, value.clone());
        }
        Ok(value)
    }

    pub(crate) fn eval_valuewhen(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let condition = self.eval_expr(&args[0].value)?;
        let source = self.eval_expr(&args[1].value)?;
        let occurrence = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(-1);
        if occurrence < 0 {
            return Ok(PineValue::Na);
        }

        let occurrence = occurrence as usize;
        if occurrence >= MAX_SERIES_HISTORY_VALUES {
            return Ok(PineValue::Na);
        }

        let values = self.valuewhen_state.entry(call_site_id).or_default();
        if matches!(condition, PineValue::Bool(true)) {
            values.push_front(source);
            values.truncate(occurrence + 1);
        }

        Ok(values.get(occurrence).cloned().unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_window_extreme(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let (source, length) = self.eval_extreme_source_length(args, mode)?;
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let value = window.extreme(mode).unwrap_or(f64::NAN);
        Ok(PineValue::Float(value))
    }

    pub(crate) fn eval_window_extreme_offset(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let (source, length) = self.eval_extreme_source_length(args, mode)?;
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(window
            .extreme_offset(mode)
            .map_or(PineValue::Na, |offset| PineValue::Int(offset as i64)))
    }

    pub(crate) fn eval_extreme_source_length(
        &mut self,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<(PineValue, i64), RuntimeError> {
        if args.len() == 1 {
            let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
            let source_name = match mode {
                WindowExtreme::Highest => "high",
                WindowExtreme::Lowest => "low",
            };
            let source = self
                .current_builtin_f64(source_name)
                .map_or(PineValue::Na, PineValue::Float);
            return Ok((source, length));
        }

        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        Ok((source, length))
    }

    pub(crate) fn update_rolling_window(
        &mut self,
        call_site_id: CallSiteId,
        source: PineValue,
        length: usize,
    ) -> &RollingWindowState {
        let source = source.as_f64();
        self.update_rolling_window_key(RollingWindowKey::Single(call_site_id), source, length)
    }

    pub(crate) fn update_mfi_windows(
        &mut self,
        call_site_id: CallSiteId,
        positive_flow: Option<f64>,
        negative_flow: Option<f64>,
        length: usize,
    ) {
        self.update_rolling_window_key(
            RollingWindowKey::MfiPositive(call_site_id),
            positive_flow,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::MfiNegative(call_site_id),
            negative_flow,
            length,
        );
    }

    pub(crate) fn update_cmo_windows(
        &mut self,
        call_site_id: CallSiteId,
        positive_change: Option<f64>,
        negative_change: Option<f64>,
        length: usize,
    ) {
        self.update_rolling_window_key(
            RollingWindowKey::CmoPositive(call_site_id),
            positive_change,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CmoNegative(call_site_id),
            negative_change,
            length,
        );
    }

    pub(crate) fn update_rolling_window_key(
        &mut self,
        key: RollingWindowKey,
        source: Option<f64>,
        length: usize,
    ) -> &RollingWindowState {
        let window = self.rolling_windows.entry(key).or_default();
        window.push(source, length);
        window
    }

    pub(crate) fn true_range(&self, handle_na: bool) -> PineValue {
        let Some(high) = self.current_builtin_f64("high") else {
            return PineValue::Na;
        };
        let Some(low) = self.current_builtin_f64("low") else {
            return PineValue::Na;
        };
        let high_low = high - low;
        let previous_close = self.previous_close();

        let Some(previous_close) = previous_close else {
            return if handle_na {
                PineValue::Float(high_low)
            } else {
                PineValue::Na
            };
        };

        PineValue::Float(
            high_low
                .max((high - previous_close).abs())
                .max((low - previous_close).abs()),
        )
    }

    pub(crate) fn eval_tsi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let short_length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let long_length = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        if short_length <= 0 || long_length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(series_id) = args[0].value.series_id else {
            return Ok(PineValue::Na);
        };
        let Some(previous_source) = self.series_store.read(series_id, 1).as_f64() else {
            return Ok(PineValue::Na);
        };

        let momentum = source - previous_source;
        let previous = tsi_state(self.call_state.get(&call_site_id));
        let short_momentum = ema_next(previous.map(|state| state.0), momentum, short_length);
        let long_momentum = ema_next(previous.map(|state| state.1), short_momentum, long_length);
        let short_abs_momentum =
            ema_next(previous.map(|state| state.2), momentum.abs(), short_length);
        let long_abs_momentum = ema_next(
            previous.map(|state| state.3),
            short_abs_momentum,
            long_length,
        );

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(short_momentum),
                PineValue::Float(long_momentum),
                PineValue::Float(short_abs_momentum),
                PineValue::Float(long_abs_momentum),
            ]),
        );

        if long_abs_momentum == 0.0 {
            return Ok(PineValue::Na);
        }
        Ok(finite_float_or_na(long_momentum / long_abs_momentum))
    }

    pub(crate) fn eval_cmo(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let (positive_change, negative_change) = match (source.as_f64(), args[0].value.series_id) {
            (Some(source), Some(series_id)) => {
                match self.series_store.read(series_id, 1).as_f64() {
                    Some(previous) => {
                        let change = source - previous;
                        (Some(change.max(0.0)), Some((-change).max(0.0)))
                    }
                    None => (None, None),
                }
            }
            _ => (None, None),
        };

        self.update_cmo_windows(call_site_id, positive_change, negative_change, length);

        let positive_window = self
            .rolling_windows
            .get(&RollingWindowKey::CmoPositive(call_site_id));
        let negative_window = self
            .rolling_windows
            .get(&RollingWindowKey::CmoNegative(call_site_id));
        let (Some(positive_window), Some(negative_window)) = (positive_window, negative_window)
        else {
            return Ok(PineValue::Na);
        };
        if !positive_window.is_ready(length) || !negative_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let positive_sum = positive_window.sum;
        let negative_sum = negative_window.sum;
        let denominator = positive_sum + negative_sum;
        if denominator == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            100.0 * (positive_sum - negative_sum) / denominator,
        ))
    }
}
