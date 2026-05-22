use pine_ir::{CallSiteId, HirCallArg};

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_drawing_call(
        &mut self,
        callee: &str,
        _call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "label.new" => self.eval_label_new(args),
            _ => return None,
        })
    }

    fn eval_label_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(x_arg) = call_arg_expr(args, 0, "x") else {
            return Err(RuntimeError {
                message: "label.new missing x argument".to_owned(),
            });
        };
        let Some(y_arg) = call_arg_expr(args, 1, "y") else {
            return Err(RuntimeError {
                message: "label.new missing y argument".to_owned(),
            });
        };
        let Some(text_arg) = call_arg_expr(args, 2, "text") else {
            return Err(RuntimeError {
                message: "label.new missing text argument".to_owned(),
            });
        };

        let x = self.eval_expr(x_arg)?;
        let y = self.eval_expr(y_arg)?;
        let text = self.eval_expr(text_arg)?;
        let id = self.next_label_id;
        self.next_label_id = self
            .next_label_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "label id limit exceeded".to_owned(),
            })?;
        self.labels.push(LabelOutput {
            id,
            snapshots: vec![LabelSnapshot {
                bar_index: self.bars,
                exists: true,
                x,
                y,
                text,
            }],
        });
        Ok(PineValue::Label(id))
    }
}
