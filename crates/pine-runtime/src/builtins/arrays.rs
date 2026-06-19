use std::cmp::Ordering;

use pine_ir::{HirCallArg, HirExpr};

use crate::builtins::strings::stringify_array_join_element;
use crate::*;

mod calls;
mod constructors;
mod support;

pub(crate) use support::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_array_size(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        Ok(PineValue::Int(values.len() as i64))
    }

    pub(crate) fn eval_array_push(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.push cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.push(value);
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_get(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| {
                normalize_array_index(index, values.len()).and_then(|index| values.get(index))
            })
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_set(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[2].value, kind)?;
        if let Some(slot) = self.array_store.get_mut(&id).and_then(|values| {
            normalize_array_index(index, values.len()).and_then(|index| values.get_mut(index))
        }) {
            *slot = value;
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_insert(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[2].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            let Some(index) = normalize_array_insert_index(index, values.len()) else {
                return Ok(PineValue::Void);
            };
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.insert cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.insert(index, value);
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_pop(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(Vec::pop)
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_remove(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(|values| {
                normalize_array_index(index, values.len()).map(|index| values.remove(index))
            })
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_shift(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(|values| {
                if values.is_empty() {
                    None
                } else {
                    Some(values.remove(0))
                }
            })
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_unshift(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.unshift cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.insert(0, value);
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_fill(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            if let Some(index_from) = args.get(2) {
                let _ = self.eval_expr(&index_from.value)?;
            }
            if let Some(index_to) = args.get(3) {
                let _ = self.eval_expr(&index_to.value)?;
            }
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            if let Some(index_from) = args.get(2) {
                let _ = self.eval_expr(&index_from.value)?;
            }
            if let Some(index_to) = args.get(3) {
                let _ = self.eval_expr(&index_to.value)?;
            }
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        let index_from = if let Some(index_from) = args.get(2) {
            self.eval_expr(&index_from.value)?.as_i64()
        } else {
            Some(0)
        };
        let Some(index_from) = index_from else {
            return Ok(PineValue::Void);
        };
        let index_to = if let Some(index_to) = args.get(3) {
            self.eval_expr(&index_to.value)?.as_i64()
        } else {
            self.array_store.get(&id).map(|values| values.len() as i64)
        };
        let Some(index_to) = index_to else {
            return Ok(PineValue::Void);
        };
        if index_from < 0 || index_to < 0 || index_from > index_to {
            return Ok(PineValue::Void);
        }
        let index_from = index_from as usize;
        let index_to = index_to as usize;
        if let Some(values) = self.array_store.get_mut(&id) {
            if index_to > values.len() {
                return Ok(PineValue::Void);
            }
            for item in &mut values[index_from..index_to] {
                *item = value.clone();
            }
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_first(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| values.first())
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_last(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| values.last())
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_array_copy(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id).cloned() else {
            return Ok(PineValue::Na);
        };
        Ok(self.new_array_from_values(kind, values))
    }

    pub(crate) fn eval_array_slice(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index_from = self.eval_expr(&args[1].value)?.as_i64();
        let index_to = self.eval_expr(&args[2].value)?.as_i64();
        let (PineValue::Array(id), Some(index_from), Some(index_to)) = (id, index_from, index_to)
        else {
            return Ok(PineValue::Na);
        };
        if index_from < 0 || index_to < 0 || index_from > index_to {
            return Ok(PineValue::Na);
        }

        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let index_from = index_from as usize;
        let index_to = index_to as usize;
        if index_to > values.len() {
            return Ok(PineValue::Na);
        }
        let values = values[index_from..index_to].to_vec();

        Ok(self.new_array_from_values(kind, values))
    }

    pub(crate) fn eval_array_concat(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let target = self.eval_expr(&args[0].value)?;
        let source = self.eval_expr(&args[1].value)?;
        let (PineValue::Array(target_id), PineValue::Array(source_id)) = (target, source) else {
            return Ok(PineValue::Na);
        };
        let Some(target_kind) = self.array_kinds.get(&target_id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(source_kind) = self.array_kinds.get(&source_id).copied() else {
            return Ok(PineValue::Na);
        };
        if target_kind != source_kind {
            return Ok(PineValue::Na);
        }
        let Some(source_values) = self.array_store.get(&source_id).cloned() else {
            return Ok(PineValue::Na);
        };
        let Some(target_values) = self.array_store.get_mut(&target_id) else {
            return Ok(PineValue::Na);
        };
        if target_values.len() + source_values.len() > MAX_ARRAY_ELEMENTS {
            return Err(RuntimeError {
                message: format!("array.concat cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
            });
        }
        target_values.extend(source_values);
        Ok(PineValue::Array(target_id))
    }

    pub(crate) fn eval_array_includes(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let index = self.eval_array_search(args, ArraySearchMode::First)?;
        Ok(PineValue::Bool(index.is_some()))
    }

    pub(crate) fn eval_array_truth(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayTruthMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::Bool
        ) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let result = match mode {
            ArrayTruthMode::Every => values.iter().all(array_truthy_value),
            ArrayTruthMode::Some => values.iter().any(array_truthy_value),
        };
        Ok(PineValue::Bool(result))
    }

    pub(crate) fn eval_array_indexof(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let index = self
            .eval_array_search(args, ArraySearchMode::First)?
            .map_or(-1, |index| index as i64);
        Ok(PineValue::Int(index))
    }

    pub(crate) fn eval_array_lastindexof(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let index = self
            .eval_array_search(args, ArraySearchMode::Last)?
            .map_or(-1, |index| index as i64);
        Ok(PineValue::Int(index))
    }

    pub(crate) fn eval_array_search(
        &mut self,
        args: &[HirCallArg],
        mode: ArraySearchMode,
    ) -> Result<Option<usize>, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(None);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(None);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        let Some(values) = self.array_store.get(&id) else {
            return Ok(None);
        };
        let index = match mode {
            ArraySearchMode::First => values.iter().position(|item| values_equal(item, &value)),
            ArraySearchMode::Last => values.iter().rposition(|item| values_equal(item, &value)),
        };
        Ok(index)
    }

    pub(crate) fn eval_array_binary_search(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayBinarySearchMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        }
        let value = self.eval_array_value(&args[1].value, kind)?;
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Int(-1));
        };
        if values.is_empty() {
            return Ok(PineValue::Int(-1));
        }

        let lower = array_numeric_lower_bound(values, &value);
        let exact_match =
            lower < values.len() && compare_array_numeric_values(&values[lower], &value).is_eq();
        let index = match mode {
            ArrayBinarySearchMode::Exact => exact_match.then_some(lower),
            ArrayBinarySearchMode::Leftmost => {
                if exact_match || lower == 0 {
                    Some(lower.min(values.len() - 1))
                } else {
                    Some(lower - 1)
                }
            }
            ArrayBinarySearchMode::Rightmost => {
                if exact_match {
                    Some(array_numeric_upper_bound(values, &value) - 1)
                } else {
                    Some(lower.min(values.len() - 1))
                }
            }
        }
        .map_or(-1, |index| index as i64);

        Ok(PineValue::Int(index))
    }

    pub(crate) fn eval_array_numeric(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayNumericMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        match mode {
            ArrayNumericMode::Min | ArrayNumericMode::Max => {
                let mut current: Option<f64> = None;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    current = Some(match (mode, current) {
                        (_, None) => value,
                        (ArrayNumericMode::Min, Some(current)) => current.min(value),
                        (ArrayNumericMode::Max, Some(current)) => current.max(value),
                        _ => unreachable!("only min/max modes are handled here"),
                    });
                }
                let Some(current) = current else {
                    return Ok(PineValue::Na);
                };
                Ok(array_numeric_result(kind, current))
            }
            ArrayNumericMode::Range => {
                let mut min: Option<f64> = None;
                let mut max: Option<f64> = None;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    min = Some(min.map_or(value, |current| current.min(value)));
                    max = Some(max.map_or(value, |current| current.max(value)));
                }
                let (Some(min), Some(max)) = (min, max) else {
                    return Ok(PineValue::Na);
                };
                Ok(array_numeric_result(kind, max - min))
            }
            ArrayNumericMode::Sum | ArrayNumericMode::Avg => {
                let mut total = 0.0;
                let mut count = 0_usize;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    total += value;
                    count += 1;
                }
                if count == 0 {
                    return Ok(PineValue::Na);
                }
                if matches!(mode, ArrayNumericMode::Avg) {
                    Ok(finite_float_or_na(total / count as f64))
                } else {
                    Ok(array_numeric_result(kind, total))
                }
            }
            ArrayNumericMode::Median => {
                let mut numeric_values: Vec<_> =
                    values.iter().filter_map(PineValue::as_f64).collect();
                if numeric_values.is_empty() {
                    return Ok(PineValue::Na);
                }
                numeric_values
                    .sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
                let middle = numeric_values.len() / 2;
                let median = if numeric_values.len() % 2 == 0 {
                    (numeric_values[middle - 1] + numeric_values[middle]) / 2.0
                } else {
                    numeric_values[middle]
                };
                Ok(array_numeric_result(kind, median))
            }
            ArrayNumericMode::Mode => {
                let mut numeric_values: Vec<_> =
                    values.iter().filter_map(PineValue::as_f64).collect();
                if numeric_values.is_empty() {
                    return Ok(PineValue::Na);
                }
                numeric_values
                    .sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

                let mut best_value = numeric_values[0];
                let mut best_count = 0_usize;
                let mut current_value = numeric_values[0];
                let mut current_count = 0_usize;
                for value in numeric_values {
                    if (value - current_value).abs() < f64::EPSILON {
                        current_count += 1;
                    } else {
                        if current_count > best_count {
                            best_value = current_value;
                            best_count = current_count;
                        }
                        current_value = value;
                        current_count = 1;
                    }
                }
                if current_count > best_count {
                    best_value = current_value;
                    best_count = current_count;
                }
                if best_count < 2 {
                    return Ok(PineValue::Na);
                }
                Ok(array_numeric_result(kind, best_value))
            }
        }
    }

    pub(crate) fn eval_array_abs(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let values = values
            .iter()
            .map(|value| match (kind, value) {
                (_, PineValue::Na) => PineValue::Na,
                (ArrayElementKind::Int, PineValue::Int(value)) => value
                    .checked_abs()
                    .map(PineValue::Int)
                    .unwrap_or(PineValue::Na),
                (ArrayElementKind::Float, PineValue::Float(value)) => {
                    finite_float_or_na(value.abs())
                }
                (ArrayElementKind::Float, PineValue::Int(value)) => {
                    finite_float_or_na((*value as f64).abs())
                }
                _ => PineValue::Na,
            })
            .collect();

        Ok(self.new_array_from_values(kind, values))
    }

    pub(crate) fn eval_array_percentile(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayPercentileMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let percentage = self.eval_expr(&args[1].value)?.as_f64();
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(percentage) = percentage else {
            return Ok(PineValue::Na);
        };
        if !(0.0..=100.0).contains(&percentage) {
            return Ok(PineValue::Na);
        }
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let mut numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        if numeric_values.is_empty() {
            return Ok(PineValue::Na);
        }
        numeric_values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

        match mode {
            ArrayPercentileMode::NearestRank => {
                let rank = ((percentage / 100.0) * numeric_values.len() as f64).ceil();
                let index = (rank as usize)
                    .saturating_sub(1)
                    .min(numeric_values.len() - 1);
                Ok(array_numeric_result(kind, numeric_values[index]))
            }
            ArrayPercentileMode::LinearInterpolation => {
                if numeric_values.len() == 1 {
                    return Ok(finite_float_or_na(numeric_values[0]));
                }
                let rank = (percentage / 100.0) * (numeric_values.len() - 1) as f64;
                let lower = rank.floor() as usize;
                let upper = rank.ceil() as usize;
                let fraction = rank - lower as f64;
                let value = numeric_values[lower]
                    + (numeric_values[upper] - numeric_values[lower]) * fraction;
                Ok(finite_float_or_na(value))
            }
        }
    }

    pub(crate) fn eval_array_percentrank(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(index) = index else {
            return Ok(PineValue::Na);
        };
        if index < 0 {
            return Ok(PineValue::Na);
        }
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let Some(target) = values.get(index as usize).and_then(PineValue::as_f64) else {
            return Ok(PineValue::Na);
        };
        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        if numeric_values.is_empty() {
            return Ok(PineValue::Na);
        }
        let count = numeric_values
            .iter()
            .filter(|value| **value <= target || (**value - target).abs() < f64::EPSILON)
            .count();
        Ok(finite_float_or_na(
            count as f64 / numeric_values.len() as f64 * 100.0,
        ))
    }

    pub(crate) fn eval_array_standardize(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        let count = numeric_values.len();
        if count == 0 {
            return Ok(self.new_array_from_values(ArrayElementKind::Float, Vec::new()));
        }

        let mean = numeric_values.iter().sum::<f64>() / count as f64;
        let variance = numeric_values
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        let stdev = variance.sqrt();

        let values = values
            .iter()
            .map(|value| {
                let Some(value) = value.as_f64() else {
                    return PineValue::Na;
                };
                if stdev == 0.0 || !stdev.is_finite() {
                    PineValue::Na
                } else {
                    finite_float_or_na((value - mean) / stdev)
                }
            })
            .collect();

        Ok(self.new_array_from_values(ArrayElementKind::Float, values))
    }

    pub(crate) fn eval_array_covariance(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id1 = self.eval_expr(&args[0].value)?;
        let id2 = self.eval_expr(&args[1].value)?;
        let biased = match args.get(2) {
            Some(arg) => matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true)),
            None => true,
        };
        let (PineValue::Array(id1), PineValue::Array(id2)) = (id1, id2) else {
            return Ok(PineValue::Na);
        };
        let (Some(kind1), Some(kind2)) = (
            self.array_kinds.get(&id1).copied(),
            self.array_kinds.get(&id2).copied(),
        ) else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind1, ArrayElementKind::Float | ArrayElementKind::Int)
            || !matches!(kind2, ArrayElementKind::Float | ArrayElementKind::Int)
        {
            return Ok(PineValue::Na);
        }
        let (Some(values1), Some(values2)) =
            (self.array_store.get(&id1), self.array_store.get(&id2))
        else {
            return Ok(PineValue::Na);
        };
        if values1.len() != values2.len() {
            return Ok(PineValue::Na);
        }

        let pairs: Vec<_> = values1
            .iter()
            .zip(values2)
            .filter_map(|(left, right)| Some((left.as_f64()?, right.as_f64()?)))
            .collect();
        let count = pairs.len();
        if count == 0 || (!biased && count < 2) {
            return Ok(PineValue::Na);
        }

        let mean1 = pairs.iter().map(|(left, _)| left).sum::<f64>() / count as f64;
        let mean2 = pairs.iter().map(|(_, right)| right).sum::<f64>() / count as f64;
        let covariance_sum = pairs
            .iter()
            .map(|(left, right)| (left - mean1) * (right - mean2))
            .sum::<f64>();
        let denominator = if biased { count } else { count - 1 };
        Ok(finite_float_or_na(covariance_sum / denominator as f64))
    }

    pub(crate) fn eval_array_variance(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayVarianceMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let biased = match args.get(1) {
            Some(arg) => matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true)),
            None => true,
        };
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        let count = numeric_values.len();
        if count == 0 || (!biased && count < 2) {
            return Ok(PineValue::Na);
        }

        let mean = numeric_values.iter().sum::<f64>() / count as f64;
        let squared_diff_sum = numeric_values
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>();
        let denominator = if biased { count } else { count - 1 };
        let variance = squared_diff_sum / denominator as f64;
        let result = match mode {
            ArrayVarianceMode::Variance => variance,
            ArrayVarianceMode::Stdev => variance.sqrt(),
        };

        Ok(finite_float_or_na(result))
    }

    pub(crate) fn eval_array_sort(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let descending = self.eval_array_sort_descending(args, "array.sort")?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Void);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::String
        ) {
            return Ok(PineValue::Void);
        }
        if let Some(values) = self.array_store.get_mut(&id) {
            values.sort_by(|left, right| compare_array_sort_values(kind, left, right, descending));
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_sort_indices(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let descending = self.eval_array_sort_descending(args, "array.sort_indices")?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::String
        ) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let mut indices = (0..values.len()).collect::<Vec<_>>();
        indices.sort_by(|left, right| {
            compare_array_sort_values(kind, &values[*left], &values[*right], descending)
                .then_with(|| left.cmp(right))
        });
        let values = indices
            .into_iter()
            .map(|index| PineValue::Int(index as i64))
            .collect();

        Ok(self.new_array_from_values(ArrayElementKind::Int, values))
    }

    pub(crate) fn eval_array_sort_descending(
        &mut self,
        args: &[HirCallArg],
        callee: &str,
    ) -> Result<bool, RuntimeError> {
        match args.get(1) {
            Some(order) => match self.eval_expr(&order.value)? {
                PineValue::String(order) if order == "order.descending" => Ok(true),
                PineValue::String(order) if order == "order.ascending" => Ok(false),
                PineValue::String(order) => Err(RuntimeError {
                    message: format!("unsupported {callee} order `{order}`"),
                }),
                _ => Ok(false),
            },
            None => Ok(false),
        }
    }

    pub(crate) fn eval_array_reverse(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        if let Some(values) = self.array_store.get_mut(&id) {
            values.reverse();
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_join(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            if let Some(separator) = args.get(1) {
                let _ = self.eval_expr(&separator.value)?;
            }
            return Ok(PineValue::Na);
        };
        let separator = if let Some(separator) = args.get(1) {
            match self.eval_expr(&separator.value)? {
                PineValue::String(separator) => separator,
                PineValue::Na => ",".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            ",".to_owned()
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let mut result = String::new();
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                result.push_str(&separator);
            }
            result.push_str(&stringify_array_join_element(value));
        }
        self.string_value_or_error(result, "array.join")
    }

    pub(crate) fn eval_array_clear(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        if let Some(values) = self.array_store.get_mut(&id) {
            values.clear();
        }
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_array_value(
        &mut self,
        expr: &HirExpr,
        kind: ArrayElementKind,
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(expr)?;
        Ok(match (kind, value) {
            (ArrayElementKind::Float, PineValue::Int(value)) => PineValue::Float(value as f64),
            (ArrayElementKind::Float, PineValue::Float(value)) => PineValue::Float(value),
            (ArrayElementKind::Int, PineValue::Int(value)) => PineValue::Int(value),
            (ArrayElementKind::Bool, PineValue::Bool(value)) => PineValue::Bool(value),
            (ArrayElementKind::String, PineValue::String(value)) => PineValue::String(value),
            (ArrayElementKind::Color, PineValue::Color(value)) => PineValue::Color(value),
            (ArrayElementKind::Label, PineValue::Label(value)) => PineValue::Label(value),
            (ArrayElementKind::Line, PineValue::Line(value)) => PineValue::Line(value),
            (ArrayElementKind::LineFill, PineValue::LineFill(value)) => PineValue::LineFill(value),
            (ArrayElementKind::Box, PineValue::Box(value)) => PineValue::Box(value),
            (ArrayElementKind::Table, PineValue::Table(value)) => PineValue::Table(value),
            (_, PineValue::Na) => PineValue::Na,
            _ => PineValue::Na,
        })
    }
}
