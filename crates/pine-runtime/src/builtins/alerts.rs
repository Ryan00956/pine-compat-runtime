use pine_ir::{CallSiteId, HirCallArg};

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_alert_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "alertcondition" => self.eval_alertcondition(call_site_id, args),
            _ => return None,
        })
    }

    fn eval_alertcondition(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(condition_arg) = call_arg_expr(args, 0, "condition") else {
            return Err(RuntimeError {
                message: "alertcondition missing condition argument".to_owned(),
            });
        };
        let condition = self.eval_expr(condition_arg)?;
        if !matches!(condition, PineValue::Bool(true)) {
            return Ok(PineValue::Void);
        }

        let source = self.alert_string_arg(args, 1, "title")?;
        let message = self.alert_string_arg(args, 2, "message")?;
        let time = self.current_bar.map_or(0, |bar| bar.time);
        self.alerts.push(AlertEvent {
            id: call_site_id.0,
            bar_index: self.bars,
            time,
            message,
            source,
        });
        Ok(PineValue::Void)
    }

    fn alert_string_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<String, RuntimeError> {
        let Some(expr) = call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("alertcondition missing {name} argument"),
            });
        };
        match self.eval_expr(expr)? {
            PineValue::String(value) => Ok(value),
            value => Err(RuntimeError {
                message: format!("alertcondition {name} evaluated to {value:?}"),
            }),
        }
    }
}
