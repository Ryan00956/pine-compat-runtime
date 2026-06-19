use pine_ir::{HirExpr, HirHistoryOffset};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_history(
        &mut self,
        expr: &HirExpr,
        offset: &HirHistoryOffset,
    ) -> Result<PineValue, RuntimeError> {
        let Some(offset) = self.eval_history_offset(offset)? else {
            return Ok(PineValue::Na);
        };

        if offset == 0 {
            return self.eval_expr(expr);
        }

        self.eval_expr(expr)?;
        if let Some(series_id) = expr.series_id {
            let value = self.series_store.read(series_id, offset);
            self.clone_array_history_value(value)
        } else {
            Ok(PineValue::Na)
        }
    }

    pub(crate) fn eval_history_offset(
        &mut self,
        offset: &HirHistoryOffset,
    ) -> Result<Option<usize>, RuntimeError> {
        let value = match offset {
            HirHistoryOffset::Constant(offset) => return Ok(Some(*offset as usize)),
            HirHistoryOffset::Dynamic(expr) => self.eval_expr(expr)?,
        };

        match value {
            PineValue::Int(value) if value >= 0 => {
                usize::try_from(value).map(Some).map_err(|_| RuntimeError {
                    message: "history offset is too large".to_owned(),
                })
            }
            PineValue::Int(_) => Err(RuntimeError {
                message: "history offset must be non-negative".to_owned(),
            }),
            PineValue::Float(value) if value >= 0.0 && value.fract() == 0.0 => {
                if value > usize::MAX as f64 {
                    Err(RuntimeError {
                        message: "history offset is too large".to_owned(),
                    })
                } else {
                    Ok(Some(value as usize))
                }
            }
            PineValue::Float(value) if value < 0.0 => Err(RuntimeError {
                message: "history offset must be non-negative".to_owned(),
            }),
            PineValue::Na => Ok(None),
            _ => Err(RuntimeError {
                message: "history offset must be an int".to_owned(),
            }),
        }
    }

    pub(crate) fn clone_array_history_value(
        &mut self,
        value: PineValue,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Array(id) = value else {
            return Ok(value);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_values_clone(id)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.new_array_from_values(kind, values))
    }
}
