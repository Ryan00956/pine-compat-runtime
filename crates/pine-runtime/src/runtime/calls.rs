use pine_ir::{CallSiteId, HirCallArg};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        if let Some(result) = self.eval_variable_call(callee, call_site_id, args) {
            return result;
        }
        if let Some(result) = self.eval_output_call(callee, call_site_id, args) {
            return result;
        }
        if let Some(result) = self.eval_color_call(callee, args) {
            return result;
        }
        if let Some(result) = self.eval_string_call(callee, args) {
            return result;
        }
        if let Some(result) = self.eval_time_call(callee, args) {
            return result;
        }
        if let Some(result) = self.eval_cast_call(callee, args) {
            return result;
        }
        if let Some(result) = self.eval_math_call(callee, call_site_id, args) {
            return result;
        }
        if let Some(result) = self.eval_ta_call(callee, call_site_id, args) {
            return result;
        }
        if let Some(result) = self.eval_array_call(callee, args) {
            return result;
        }

        Err(RuntimeError {
            message: format!("unsupported runtime call `{callee}`"),
        })
    }
}
