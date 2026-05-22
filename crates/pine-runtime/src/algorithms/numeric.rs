use crate::*;

pub(crate) fn finite_float_or_na(value: f64) -> PineValue {
    if value.is_finite() {
        PineValue::Float(value)
    } else {
        PineValue::Na
    }
}
