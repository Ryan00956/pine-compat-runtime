use std::cmp::Ordering;

use crate::{PineValue, finite_float_or_na};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayElementKind {
    Float,
    Int,
    Bool,
    String,
    Color,
    Label,
    Line,
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArraySearchMode {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayBinarySearchMode {
    Exact,
    Leftmost,
    Rightmost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayTruthMode {
    Every,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayNumericMode {
    Min,
    Max,
    Sum,
    Avg,
    Range,
    Median,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayVarianceMode {
    Variance,
    Stdev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayPercentileMode {
    NearestRank,
    LinearInterpolation,
}

pub(crate) fn infer_array_from_kind(values: &[PineValue]) -> Option<ArrayElementKind> {
    let mut inferred_kind: Option<ArrayElementKind> = None;
    for value in values {
        let next_kind = match value {
            PineValue::Na => continue,
            PineValue::Int(_) => ArrayElementKind::Int,
            PineValue::Float(_) => ArrayElementKind::Float,
            PineValue::Bool(_) => ArrayElementKind::Bool,
            PineValue::String(_) => ArrayElementKind::String,
            PineValue::Color(_) => ArrayElementKind::Color,
            PineValue::Label(_) => ArrayElementKind::Label,
            PineValue::Line(_) => ArrayElementKind::Line,
            PineValue::Box(_) => ArrayElementKind::Box,
            _ => return None,
        };
        inferred_kind = Some(match (inferred_kind, next_kind) {
            (None, kind) => kind,
            (Some(ArrayElementKind::Int), ArrayElementKind::Float)
            | (Some(ArrayElementKind::Float), ArrayElementKind::Int)
            | (Some(ArrayElementKind::Float), ArrayElementKind::Float)
            | (Some(ArrayElementKind::Int), ArrayElementKind::Int) => {
                if matches!(next_kind, ArrayElementKind::Float)
                    || matches!(inferred_kind, Some(ArrayElementKind::Float))
                {
                    ArrayElementKind::Float
                } else {
                    ArrayElementKind::Int
                }
            }
            (Some(current), kind) if current == kind => current,
            _ => return None,
        });
    }
    inferred_kind
}

pub(crate) fn array_truthy_value(value: &PineValue) -> bool {
    match value {
        PineValue::Bool(value) => *value,
        PineValue::Int(value) => *value != 0,
        PineValue::Float(value) => *value != 0.0,
        _ => false,
    }
}

pub(crate) fn normalize_array_index(index: i64, len: usize) -> Option<usize> {
    let len = i64::try_from(len).ok()?;
    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };
    if (0..len).contains(&index) {
        Some(index as usize)
    } else {
        None
    }
}

pub(crate) fn normalize_array_insert_index(index: i64, len: usize) -> Option<usize> {
    let len = i64::try_from(len).ok()?;
    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };
    if (0..=len).contains(&index) {
        Some(index as usize)
    } else {
        None
    }
}

pub(crate) fn array_numeric_result(kind: ArrayElementKind, value: f64) -> PineValue {
    match kind {
        ArrayElementKind::Int => PineValue::Int(value as i64),
        ArrayElementKind::Float => finite_float_or_na(value),
        _ => PineValue::Na,
    }
}

pub(crate) fn compare_array_numeric_values(left: &PineValue, right: &PineValue) -> Ordering {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(crate) fn compare_array_sort_values(
    kind: ArrayElementKind,
    left: &PineValue,
    right: &PineValue,
    descending: bool,
) -> Ordering {
    let left_is_special = is_array_sort_special(kind, left);
    let right_is_special = is_array_sort_special(kind, right);
    match (left_is_special, right_is_special) {
        (true, true) => return Ordering::Equal,
        (true, false) => {
            return if descending {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        (false, true) => {
            return if descending {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
        (false, false) => {}
    }

    let ordering = match kind {
        ArrayElementKind::Float | ArrayElementKind::Int => {
            compare_array_numeric_values(left, right)
        }
        ArrayElementKind::String => match (left, right) {
            (PineValue::String(left), PineValue::String(right)) => left.cmp(right),
            _ => Ordering::Equal,
        },
        _ => Ordering::Equal,
    };
    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

pub(crate) fn is_array_sort_special(kind: ArrayElementKind, value: &PineValue) -> bool {
    matches!(value, PineValue::Na)
        || matches!(value, PineValue::Float(value) if !value.is_finite())
        || matches!(
            (kind, value),
            (ArrayElementKind::String, PineValue::String(value)) if value.is_empty()
        )
}

pub(crate) fn array_numeric_lower_bound(values: &[PineValue], target: &PineValue) -> usize {
    let mut left = 0;
    let mut right = values.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if compare_array_numeric_values(&values[mid], target).is_lt() {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

pub(crate) fn array_numeric_upper_bound(values: &[PineValue], target: &PineValue) -> usize {
    let mut left = 0;
    let mut right = values.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if compare_array_numeric_values(&values[mid], target).is_le() {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}
