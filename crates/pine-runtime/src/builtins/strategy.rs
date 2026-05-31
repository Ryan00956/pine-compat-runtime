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
        let Some(_bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.exit` requires an active bar".to_owned(),
            });
        };
        let Some(id_expr) = call_arg_expr(args, 0, "id") else {
            return Ok(PineValue::Void);
        };
        let Some(from_entry_expr) = call_arg_expr(args, 1, "from_entry") else {
            return Ok(PineValue::Void);
        };
        let stop_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("stop"))
            .or_else(|| args.get(2).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);
        let limit_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("limit"))
            .or_else(|| args.get(3).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);
        let profit_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("profit"))
            .map(|arg| &arg.value);
        let loss_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("loss"))
            .map(|arg| &arg.value);

        let id = match self.eval_expr(id_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };
        let from_entry = match self.eval_expr(from_entry_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };
        let has_downside = stop_expr.is_some() || loss_expr.is_some();
        let has_upside = limit_expr.is_some() || profit_expr.is_some();

        if has_downside && has_upside {
            let downside_price = if let Some(stop_expr) = stop_expr {
                let stop_price = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
                if !stop_price.is_finite() {
                    self.strategy_broker.place_exit_bracket(
                        id,
                        from_entry,
                        stop_price,
                        f64::NAN,
                        self.bars,
                    );
                    return Ok(PineValue::Void);
                }
                stop_price
            } else if let Some(loss_expr) = loss_expr {
                let loss_ticks = self.eval_expr(loss_expr)?.as_f64().unwrap_or(f64::NAN);
                let mintick =
                    pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
                let Some(loss_price) = self
                    .strategy_broker
                    .exit_loss_price_from_ticks(loss_ticks, mintick)
                else {
                    return Ok(PineValue::Void);
                };
                loss_price
            } else {
                return Ok(PineValue::Void);
            };

            let upside_price = if let Some(limit_expr) = limit_expr {
                let limit_price = self.eval_expr(limit_expr)?.as_f64().unwrap_or(f64::NAN);
                if !limit_price.is_finite() {
                    self.strategy_broker.place_exit_bracket(
                        id,
                        from_entry,
                        downside_price,
                        limit_price,
                        self.bars,
                    );
                    return Ok(PineValue::Void);
                }
                limit_price
            } else if let Some(profit_expr) = profit_expr {
                let profit_ticks = self.eval_expr(profit_expr)?.as_f64().unwrap_or(f64::NAN);
                let mintick =
                    pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
                let Some(profit_price) = self
                    .strategy_broker
                    .exit_profit_price_from_ticks(profit_ticks, mintick)
                else {
                    return Ok(PineValue::Void);
                };
                profit_price
            } else {
                return Ok(PineValue::Void);
            };

            self.strategy_broker.place_exit_bracket(
                id,
                from_entry,
                downside_price,
                upside_price,
                self.bars,
            );
        } else if let Some(stop_expr) = stop_expr {
            let stop_price = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .place_exit_stop(id, from_entry, stop_price, self.bars);
        } else if let Some(limit_expr) = limit_expr {
            let limit_price = self.eval_expr(limit_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .place_exit_limit(id, from_entry, limit_price, self.bars);
        } else if let Some(profit_expr) = profit_expr {
            let profit_ticks = self.eval_expr(profit_expr)?.as_f64().unwrap_or(f64::NAN);
            let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
            self.strategy_broker.place_exit_profit_ticks(
                id,
                from_entry,
                profit_ticks,
                mintick,
                self.bars,
            );
        } else if let Some(loss_expr) = loss_expr {
            let loss_ticks = self.eval_expr(loss_expr)?.as_f64().unwrap_or(f64::NAN);
            let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
            self.strategy_broker
                .place_exit_loss_ticks(id, from_entry, loss_ticks, mintick, self.bars);
        }
        Ok(PineValue::Void)
    }
}
