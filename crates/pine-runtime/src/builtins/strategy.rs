use pine_ir::{CallSiteId, HirCallArg};

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_strategy_call(
        &mut self,
        callee: &str,
        _call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "strategy.entry" => self.eval_strategy_entry(args),
            "strategy.close" => self.eval_strategy_close(args),
            "strategy.exit" => self.eval_strategy_exit(args),
            _ => return None,
        })
    }

    fn eval_strategy_entry(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.entry` requires an active bar".to_owned(),
            });
        };
        let Some(id_expr) = call_arg_expr(args, 0, "id") else {
            return Ok(PineValue::Void);
        };
        let Some(direction_expr) = call_arg_expr(args, 1, "direction") else {
            return Ok(PineValue::Void);
        };

        let id = match self.eval_expr(id_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };
        let direction = self.eval_expr(direction_expr)?;
        if direction != PineValue::String("strategy.long".to_owned()) {
            return Ok(PineValue::Void);
        }
        let qty = if let Some(qty_expr) = call_arg_expr(args, 2, "qty") {
            self.eval_expr(qty_expr)?.as_f64().unwrap_or(f64::NAN)
        } else {
            self.program
                .strategy_settings
                .default_entry_qty()
                .unwrap_or(f64::NAN)
        };

        self.strategy_broker
            .entry_long(id, self.bars, bar.time, bar.close, qty);
        Ok(PineValue::Void)
    }

    fn eval_strategy_close(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.close` requires an active bar".to_owned(),
            });
        };
        let Some(id_expr) = call_arg_expr(args, 0, "id") else {
            return Ok(PineValue::Void);
        };
        let id = match self.eval_expr(id_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };

        self.strategy_broker
            .close_long(id, self.bars, bar.time, bar.close);
        Ok(PineValue::Void)
    }

    fn eval_strategy_exit(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(id_expr) = call_arg_expr(args, 0, "id") else {
            return Ok(PineValue::Void);
        };
        let Some(from_entry_expr) = call_arg_expr(args, 1, "from_entry") else {
            return Ok(PineValue::Void);
        };
        let Some(stop_expr) = call_arg_expr(args, 2, "stop") else {
            return Ok(PineValue::Void);
        };

        let _ = self.eval_expr(id_expr)?;
        let _ = self.eval_expr(from_entry_expr)?;
        let _ = self.eval_expr(stop_expr)?;
        self.strategy_broker.diagnose_exit_placeholder();
        Ok(PineValue::Void)
    }
}
