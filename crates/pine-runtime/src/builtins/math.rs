use pine_ir::{CallSiteId, HirCallArg};

use crate::algorithms::random::mix_random_seed;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathExtreme {
    Max,
    Min,
}

pub(crate) fn math_extreme(left: f64, right: f64, mode: MathExtreme) -> f64 {
    match mode {
        MathExtreme::Max => left.max(right),
        MathExtreme::Min => left.min(right),
    }
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_math_abs(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(value
                .checked_abs()
                .map(PineValue::Int)
                .unwrap_or_else(|| PineValue::Float((value as f64).abs()))),
            PineValue::Float(value) => Ok(PineValue::Float(value.abs())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    pub(crate) fn eval_math_round(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        if args.len() == 1 {
            return match value {
                PineValue::Int(value) => Ok(PineValue::Int(value)),
                PineValue::Float(value) => Ok(PineValue::Float(value.round())),
                PineValue::Na => Ok(PineValue::Na),
                _ => Ok(PineValue::Na),
            };
        }

        let Some(value) = value.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(precision) = self.eval_expr(&args[1].value)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let precision = precision.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        let factor = 10_f64.powi(precision);
        Ok(finite_float_or_na((value * factor).round() / factor))
    }

    pub(crate) fn eval_math_round_to_mintick(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
        if !value.is_finite() || mintick <= 0.0 || !mintick.is_finite() {
            return Ok(PineValue::Na);
        }
        let rounded_ticks = (value / mintick + 0.5).floor();
        Ok(finite_float_or_na(rounded_ticks * mintick))
    }

    pub(crate) fn eval_math_random(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let min = match args.first() {
            Some(arg) => {
                let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                    return Ok(PineValue::Na);
                };
                value
            }
            None => 0.0,
        };
        let max = match args.get(1) {
            Some(arg) => {
                let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                    return Ok(PineValue::Na);
                };
                value
            }
            None => 1.0,
        };
        let seed = match args.get(2) {
            Some(arg) => self.eval_expr(&arg.value)?.as_i64(),
            None => None,
        };

        if !min.is_finite() || !max.is_finite() || min >= max {
            return Ok(PineValue::Na);
        }

        let initial_state = seed.map_or_else(
            || default_random_seed(call_site_id),
            |seed| mix_random_seed(seed as u64),
        );
        let state = self
            .random_state
            .entry(call_site_id)
            .or_insert(initial_state);
        *state = next_random_state(*state);
        let unit = random_unit_interval(*state);
        Ok(finite_float_or_na(min + (max - min) * unit))
    }

    pub(crate) fn eval_math_sum(
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
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.sum))
    }

    pub(crate) fn eval_math_floor(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.floor())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    pub(crate) fn eval_math_ceil(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.ceil())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    pub(crate) fn eval_math_trunc(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.trunc())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    pub(crate) fn eval_math_unary_float(
        &mut self,
        args: &[HirCallArg],
        op: impl FnOnce(f64) -> f64,
    ) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(op(value)))
    }

    pub(crate) fn eval_math_sign(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(PineValue::Float(if value > 0.0 {
            1.0
        } else if value < 0.0 {
            -1.0
        } else {
            0.0
        }))
    }

    pub(crate) fn eval_math_pow(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(base) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(exponent) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(base.powf(exponent)))
    }

    pub(crate) fn eval_math_hypot(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(left) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(right) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(left.hypot(right)))
    }

    pub(crate) fn eval_math_avg(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let mut total = 0.0;
        let mut count = 0.0;

        for arg in args {
            let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                return Ok(PineValue::Na);
            };
            total += value;
            count += 1.0;
        }

        if count == 0.0 {
            return Ok(PineValue::Na);
        }
        Ok(finite_float_or_na(total / count))
    }

    pub(crate) fn eval_math_extreme(
        &mut self,
        args: &[HirCallArg],
        mode: MathExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let mut current = 0.0;
        let mut has_value = false;
        let mut has_float = false;

        for arg in args {
            match self.eval_expr(&arg.value)? {
                PineValue::Int(value) => {
                    let value = value as f64;
                    current = if has_value {
                        math_extreme(current, value, mode)
                    } else {
                        value
                    };
                    has_value = true;
                }
                PineValue::Float(value) => {
                    current = if has_value {
                        math_extreme(current, value, mode)
                    } else {
                        value
                    };
                    has_value = true;
                    has_float = true;
                }
                PineValue::Na => return Ok(PineValue::Na),
                _ => return Ok(PineValue::Na),
            }
        }

        if !has_value {
            return Ok(PineValue::Na);
        }
        if has_float {
            Ok(PineValue::Float(current))
        } else {
            Ok(PineValue::Int(current as i64))
        }
    }
}
