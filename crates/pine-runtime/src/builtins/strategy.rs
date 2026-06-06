use pine_ir::{CallSiteId, HirCallArg};

use crate::builtins::args::call_arg_expr;
use crate::strategy::{
    LossLimitBracketSpec, LossProfitBracketSpec, StopProfitBracketSpec, TrailPointsExitSpec,
    TrailPriceExitSpec,
};
use crate::*;

#[derive(Clone, Copy)]
enum StrategyExitQuantityArg {
    Full,
    Fixed(f64),
    Percent(f64),
}

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
            "strategy.close_all" => self.eval_strategy_close_all(),
            "strategy.cancel" => self.eval_strategy_cancel(args),
            "strategy.cancel_all" => self.eval_strategy_cancel_all(),
            "strategy.exit" => self.eval_strategy_exit(args),
            "strategy.closedtrades.entry_price"
            | "strategy.closedtrades.entry_id"
            | "strategy.closedtrades.exit_price"
            | "strategy.closedtrades.exit_id"
            | "strategy.closedtrades.entry_bar_index"
            | "strategy.closedtrades.exit_bar_index"
            | "strategy.closedtrades.entry_time"
            | "strategy.closedtrades.exit_time"
            | "strategy.closedtrades.commission"
            | "strategy.closedtrades.size"
            | "strategy.closedtrades.profit"
            | "strategy.closedtrades.max_runup"
            | "strategy.closedtrades.max_drawdown" => {
                self.eval_strategy_closed_trade_field(callee, args)
            }
            "strategy.opentrades.entry_price"
            | "strategy.opentrades.entry_id"
            | "strategy.opentrades.entry_bar_index"
            | "strategy.opentrades.entry_time"
            | "strategy.opentrades.size"
            | "strategy.opentrades.profit"
            | "strategy.opentrades.commission"
            | "strategy.opentrades.max_runup"
            | "strategy.opentrades.max_drawdown" => {
                self.eval_strategy_open_trade_field(callee, args)
            }
            _ => return None,
        })
    }

    fn eval_strategy_closed_trade_field(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(trade_num_expr) = call_arg_expr(args, 0, "trade_num") else {
            return Ok(PineValue::Na);
        };
        let Some(trade_num) = self.eval_expr(trade_num_expr)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let Some(trade) = self.strategy_broker.closed_trade(trade_num) else {
            return Ok(PineValue::Na);
        };

        Ok(match callee {
            "strategy.closedtrades.entry_price" => PineValue::Float(trade.entry_price),
            "strategy.closedtrades.entry_id" => PineValue::String(trade.id.clone()),
            "strategy.closedtrades.exit_price" => PineValue::Float(trade.exit_price),
            "strategy.closedtrades.exit_id" => PineValue::String(trade.exit_id.clone()),
            "strategy.closedtrades.entry_bar_index" => {
                PineValue::Int(i64::try_from(trade.entry_bar_index).unwrap_or(i64::MAX))
            }
            "strategy.closedtrades.exit_bar_index" => {
                PineValue::Int(i64::try_from(trade.exit_bar_index).unwrap_or(i64::MAX))
            }
            "strategy.closedtrades.entry_time" => PineValue::Int(trade.entry_time),
            "strategy.closedtrades.exit_time" => PineValue::Int(trade.exit_time),
            "strategy.closedtrades.commission" => self
                .strategy_broker
                .closed_trade_commission(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.closedtrades.size" => PineValue::Float(trade.qty),
            "strategy.closedtrades.profit" => PineValue::Float(trade.profit),
            "strategy.closedtrades.max_runup" => self
                .strategy_broker
                .closed_trade_max_runup(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.closedtrades.max_drawdown" => self
                .strategy_broker
                .closed_trade_max_drawdown(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            _ => PineValue::Na,
        })
    }

    fn eval_strategy_open_trade_field(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(trade_num_expr) = call_arg_expr(args, 0, "trade_num") else {
            return Ok(PineValue::Na);
        };
        let Some(trade_num) = self.eval_expr(trade_num_expr)?.as_i64() else {
            return Ok(PineValue::Na);
        };

        Ok(match callee {
            "strategy.opentrades.entry_price" => self
                .strategy_broker
                .open_trade_entry_price(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.opentrades.entry_id" => self
                .strategy_broker
                .open_trade_entry_id(trade_num)
                .map(|value| PineValue::String(value.to_owned()))
                .unwrap_or(PineValue::Na),
            "strategy.opentrades.entry_bar_index" => self
                .strategy_broker
                .open_trade_entry_bar_index(trade_num)
                .map(|value| PineValue::Int(i64::try_from(value).unwrap_or(i64::MAX)))
                .unwrap_or(PineValue::Na),
            "strategy.opentrades.entry_time" => self
                .strategy_broker
                .open_trade_entry_time(trade_num)
                .map_or(PineValue::Na, PineValue::Int),
            "strategy.opentrades.size" => self
                .strategy_broker
                .open_trade_size(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.opentrades.profit" => {
                let Some(bar) = self.current_bar else {
                    return Ok(PineValue::Na);
                };
                self.strategy_broker
                    .open_trade_profit(trade_num, bar.close)
                    .map_or(PineValue::Na, PineValue::Float)
            }
            "strategy.opentrades.commission" => self
                .strategy_broker
                .open_trade_commission(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.opentrades.max_runup" => self
                .strategy_broker
                .open_trade_max_runup(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            "strategy.opentrades.max_drawdown" => self
                .strategy_broker
                .open_trade_max_drawdown(trade_num)
                .map_or(PineValue::Na, PineValue::Float),
            _ => PineValue::Na,
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
        let qty_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("qty"))
            .or_else(|| args.get(2).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);
        let limit_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("limit"))
            .or_else(|| args.get(3).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);
        let stop_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("stop"))
            .or_else(|| args.get(4).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);

        let qty = if let Some(qty_expr) = qty_expr {
            self.eval_expr(qty_expr)?.as_f64().unwrap_or(f64::NAN)
        } else {
            let equity = self.strategy_broker.equity_value(bar.close);
            self.program
                .strategy_settings
                .default_entry_qty(equity, bar.close)
                .unwrap_or(f64::NAN)
        };
        if let (Some(limit_expr), Some(stop_expr)) = (limit_expr, stop_expr) {
            let limit = self.eval_expr(limit_expr)?.as_f64().unwrap_or(f64::NAN);
            let stop = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .place_pending_stop_limit_long_entry(id, qty, stop, limit, self.bars);
            return Ok(PineValue::Void);
        }
        if let Some(limit_expr) = limit_expr {
            let limit = self.eval_expr(limit_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .place_pending_limit_long_entry(id, qty, limit, self.bars);
            return Ok(PineValue::Void);
        }
        if let Some(stop_expr) = stop_expr {
            let stop = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .place_pending_stop_long_entry(id, qty, stop, self.bars);
            return Ok(PineValue::Void);
        }

        self.strategy_broker
            .place_pending_market_long_entry(id, qty, self.bars);
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
        let qty_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("qty"))
            .map(|arg| &arg.value);
        let qty_percent_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("qty_percent"))
            .map(|arg| &arg.value);

        if let Some(qty_expr) = qty_expr {
            let qty = self.eval_expr(qty_expr)?.as_f64().unwrap_or(f64::NAN);
            self.strategy_broker
                .close_long_qty(id, self.bars, bar.time, bar.close, qty);
        } else if let Some(qty_percent_expr) = qty_percent_expr {
            let qty_percent = self
                .eval_expr(qty_percent_expr)?
                .as_f64()
                .unwrap_or(f64::NAN);
            self.strategy_broker.close_long_qty_percent(
                id,
                self.bars,
                bar.time,
                bar.close,
                qty_percent,
            );
        } else {
            self.strategy_broker
                .close_long(id, self.bars, bar.time, bar.close);
        }
        Ok(PineValue::Void)
    }

    fn eval_strategy_close_all(&mut self) -> Result<PineValue, RuntimeError> {
        let Some(bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.close_all` requires an active bar".to_owned(),
            });
        };

        self.strategy_broker
            .close_all_long(self.bars, bar.time, bar.close);
        Ok(PineValue::Void)
    }

    fn eval_strategy_cancel(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(_bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.cancel` requires an active bar".to_owned(),
            });
        };
        let Some(id_expr) = call_arg_expr(args, 0, "id") else {
            return Ok(PineValue::Void);
        };
        let id = match self.eval_expr(id_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };

        self.strategy_broker.cancel_pending_order(&id);
        Ok(PineValue::Void)
    }

    fn eval_strategy_cancel_all(&mut self) -> Result<PineValue, RuntimeError> {
        let Some(_bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.cancel_all` requires an active bar".to_owned(),
            });
        };

        self.strategy_broker.cancel_all_pending_orders();
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
        let from_entry_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("from_entry"))
            .or_else(|| args.get(1).filter(|arg| arg.name.is_none()))
            .map(|arg| &arg.value);
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
        let trail_price_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("trail_price"))
            .map(|arg| &arg.value);
        let trail_points_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("trail_points"))
            .map(|arg| &arg.value);
        let trail_offset_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("trail_offset"))
            .map(|arg| &arg.value);
        let qty_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("qty"))
            .map(|arg| &arg.value);
        let qty_percent_expr = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("qty_percent"))
            .map(|arg| &arg.value);

        let id = match self.eval_expr(id_expr)? {
            PineValue::String(value) => value,
            _ => return Ok(PineValue::Void),
        };
        let from_entry = match from_entry_expr {
            Some(expr) => match self.eval_expr(expr)? {
                PineValue::String(value) => value,
                _ => return Ok(PineValue::Void),
            },
            None => String::new(),
        };
        let qty = if let Some(qty_expr) = qty_expr {
            Some(self.eval_expr(qty_expr)?.as_f64().unwrap_or(f64::NAN))
        } else {
            None
        };
        let qty_percent = if let Some(qty_percent_expr) = qty_percent_expr {
            Some(
                self.eval_expr(qty_percent_expr)?
                    .as_f64()
                    .unwrap_or(f64::NAN),
            )
        } else {
            None
        };
        let quantity = match (qty, qty_percent) {
            (Some(qty), Some(_)) => StrategyExitQuantityArg::Fixed(qty),
            (Some(qty), None) => StrategyExitQuantityArg::Fixed(qty),
            (None, Some(qty_percent)) => StrategyExitQuantityArg::Percent(qty_percent),
            (None, None) => StrategyExitQuantityArg::Full,
        };
        let has_downside = stop_expr.is_some() || loss_expr.is_some();
        let has_upside = limit_expr.is_some() || profit_expr.is_some();
        let has_fixed_exit = has_downside || has_upside;
        let has_trailing_activation = trail_price_expr.is_some() || trail_points_expr.is_some();
        let has_trailing = has_trailing_activation || trail_offset_expr.is_some();
        let has_single_trailing_activation =
            trail_price_expr.is_some() != trail_points_expr.is_some();
        let is_trailing_only =
            !has_fixed_exit && has_single_trailing_activation && trail_offset_expr.is_some();
        let is_stop_profit_bracket = stop_expr.is_some()
            && profit_expr.is_some()
            && loss_expr.is_none()
            && limit_expr.is_none();
        let is_loss_limit_bracket = loss_expr.is_some()
            && limit_expr.is_some()
            && stop_expr.is_none()
            && profit_expr.is_none();
        let is_loss_profit_bracket = loss_expr.is_some()
            && profit_expr.is_some()
            && stop_expr.is_none()
            && limit_expr.is_none();
        let is_omitted_absolute_single = (stop_expr.is_some() != limit_expr.is_some())
            && profit_expr.is_none()
            && loss_expr.is_none()
            && !has_trailing;
        let is_omitted_absolute_bracket = stop_expr.is_some()
            && limit_expr.is_some()
            && profit_expr.is_none()
            && loss_expr.is_none()
            && !has_trailing
            && matches!(quantity, StrategyExitQuantityArg::Full);
        let is_omitted_relative_single = (profit_expr.is_some() != loss_expr.is_some())
            && stop_expr.is_none()
            && limit_expr.is_none()
            && !has_trailing
            && matches!(quantity, StrategyExitQuantityArg::Full);
        let is_omitted_loss_profit_bracket = is_loss_profit_bracket
            && !has_trailing
            && matches!(quantity, StrategyExitQuantityArg::Full);
        let is_omitted_stop_profit_bracket = is_stop_profit_bracket
            && !has_trailing
            && matches!(quantity, StrategyExitQuantityArg::Full);
        let is_omitted_loss_limit_bracket = is_loss_limit_bracket
            && !has_trailing
            && matches!(quantity, StrategyExitQuantityArg::Full);
        if from_entry.is_empty()
            && !(is_omitted_absolute_single
                || is_omitted_absolute_bracket
                || is_omitted_relative_single
                || is_omitted_loss_profit_bracket
                || is_omitted_stop_profit_bracket
                || is_omitted_loss_limit_bracket)
        {
            return Ok(PineValue::Void);
        }
        let has_unsupported_entry_relative_active_entry_exit = (trail_points_expr.is_some()
            && !is_trailing_only)
            || ((profit_expr.is_some() || loss_expr.is_some())
                && has_downside
                && has_upside
                && !is_stop_profit_bracket
                && !is_loss_limit_bracket
                && !is_loss_profit_bracket);
        if has_unsupported_entry_relative_active_entry_exit
            && self
                .strategy_broker
                .reject_entry_relative_exit_for_pending_entry(&from_entry)
        {
            return Ok(PineValue::Void);
        }

        if has_trailing {
            if !is_trailing_only {
                return Ok(PineValue::Void);
            }

            let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
            if let Some(trail_price_expr) = trail_price_expr {
                let activation_price = self
                    .eval_expr(trail_price_expr)?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let trail_offset_expr =
                    trail_offset_expr.expect("checked trailing offset presence");
                let trail_offset_ticks = self
                    .eval_expr(trail_offset_expr)?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                self.place_exit_trail_price_quantity(
                    id,
                    from_entry,
                    TrailPriceExitSpec {
                        activation_price,
                        offset_ticks: trail_offset_ticks,
                        mintick,
                    },
                    quantity,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }

            if let Some(trail_points_expr) = trail_points_expr {
                let activation_ticks = self
                    .eval_expr(trail_points_expr)?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let trail_offset_expr =
                    trail_offset_expr.expect("checked trailing offset presence");
                let trail_offset_ticks = self
                    .eval_expr(trail_offset_expr)?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                self.place_exit_trail_points_quantity(
                    id,
                    from_entry,
                    TrailPointsExitSpec {
                        activation_ticks,
                        offset_ticks: trail_offset_ticks,
                        mintick,
                    },
                    quantity,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }

            return Ok(PineValue::Void);
        }

        if has_downside && has_upside {
            if is_stop_profit_bracket {
                let stop_price = self
                    .eval_expr(stop_expr.expect("checked stop presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let profit_ticks = self
                    .eval_expr(profit_expr.expect("checked profit presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let mintick =
                    pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
                if from_entry.is_empty() {
                    self.strategy_broker
                        .place_all_entry_exit_stop_profit_bracket(
                            id,
                            StopProfitBracketSpec {
                                stop_price,
                                profit_ticks,
                                mintick,
                            },
                            self.bars,
                        );
                    return Ok(PineValue::Void);
                }
                self.place_exit_stop_profit_bracket_quantity(
                    id,
                    from_entry,
                    StopProfitBracketSpec {
                        stop_price,
                        profit_ticks,
                        mintick,
                    },
                    quantity,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }
            if is_loss_limit_bracket {
                let loss_ticks = self
                    .eval_expr(loss_expr.expect("checked loss presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let limit_price = self
                    .eval_expr(limit_expr.expect("checked limit presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let mintick =
                    pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
                if from_entry.is_empty() {
                    self.strategy_broker
                        .place_all_entry_exit_loss_limit_bracket(
                            id,
                            LossLimitBracketSpec {
                                loss_ticks,
                                limit_price,
                                mintick,
                            },
                            self.bars,
                        );
                    return Ok(PineValue::Void);
                }
                self.place_exit_loss_limit_bracket_quantity(
                    id,
                    from_entry,
                    LossLimitBracketSpec {
                        loss_ticks,
                        limit_price,
                        mintick,
                    },
                    quantity,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }
            if is_loss_profit_bracket {
                let loss_ticks = self
                    .eval_expr(loss_expr.expect("checked loss presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let profit_ticks = self
                    .eval_expr(profit_expr.expect("checked profit presence"))?
                    .as_f64()
                    .unwrap_or(f64::NAN);
                let mintick =
                    pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
                if from_entry.is_empty() {
                    self.strategy_broker
                        .place_all_entry_exit_loss_profit_bracket(
                            id,
                            LossProfitBracketSpec {
                                loss_ticks,
                                profit_ticks,
                                mintick,
                            },
                            self.bars,
                        );
                    return Ok(PineValue::Void);
                }
                self.place_exit_loss_profit_bracket_quantity(
                    id,
                    from_entry,
                    LossProfitBracketSpec {
                        loss_ticks,
                        profit_ticks,
                        mintick,
                    },
                    quantity,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }

            let downside_price = if let Some(stop_expr) = stop_expr {
                let stop_price = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
                if !stop_price.is_finite() {
                    self.place_exit_bracket_quantity(
                        id,
                        from_entry,
                        stop_price,
                        f64::NAN,
                        quantity,
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
                    self.place_exit_bracket_quantity(
                        id,
                        from_entry,
                        downside_price,
                        limit_price,
                        quantity,
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

            self.place_exit_bracket_quantity(
                id,
                from_entry,
                downside_price,
                upside_price,
                quantity,
                self.bars,
            );
        } else if let Some(stop_expr) = stop_expr {
            let stop_price = self.eval_expr(stop_expr)?.as_f64().unwrap_or(f64::NAN);
            self.place_exit_stop_quantity(id, from_entry, stop_price, quantity, self.bars);
        } else if let Some(limit_expr) = limit_expr {
            let limit_price = self.eval_expr(limit_expr)?.as_f64().unwrap_or(f64::NAN);
            self.place_exit_limit_quantity(id, from_entry, limit_price, quantity, self.bars);
        } else if let Some(profit_expr) = profit_expr {
            let profit_ticks = self.eval_expr(profit_expr)?.as_f64().unwrap_or(f64::NAN);
            let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
            if from_entry.is_empty() {
                self.strategy_broker.place_all_entry_exit_profit_ticks(
                    id,
                    profit_ticks,
                    mintick,
                    self.bars,
                );
                return Ok(PineValue::Void);
            }
            self.place_exit_profit_ticks_quantity(
                id,
                from_entry,
                profit_ticks,
                mintick,
                quantity,
                self.bars,
            );
        } else if let Some(loss_expr) = loss_expr {
            let loss_ticks = self.eval_expr(loss_expr)?.as_f64().unwrap_or(f64::NAN);
            let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
            if from_entry.is_empty() {
                self.strategy_broker
                    .place_all_entry_exit_loss_ticks(id, loss_ticks, mintick, self.bars);
                return Ok(PineValue::Void);
            }
            self.place_exit_loss_ticks_quantity(
                id, from_entry, loss_ticks, mintick, quantity, self.bars,
            );
        }
        Ok(PineValue::Void)
    }

    fn place_exit_stop_quantity(
        &mut self,
        id: String,
        from_entry: String,
        stop_price: f64,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_stop(id, from_entry, stop_price, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_stop_qty(id, from_entry, stop_price, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_stop_qty_percent(id, from_entry, stop_price, qty_percent, bar_index),
        }
    }

    fn place_exit_limit_quantity(
        &mut self,
        id: String,
        from_entry: String,
        limit_price: f64,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => {
                self.strategy_broker
                    .place_exit_limit(id, from_entry, limit_price, bar_index)
            }
            StrategyExitQuantityArg::Fixed(qty) => self.strategy_broker.place_exit_limit_qty(
                id,
                from_entry,
                limit_price,
                qty,
                bar_index,
            ),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_limit_qty_percent(id, from_entry, limit_price, qty_percent, bar_index),
        }
    }

    fn place_exit_profit_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_profit_ticks(id, from_entry, ticks, mintick, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_profit_ticks_qty(id, from_entry, ticks, mintick, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => {
                self.strategy_broker.place_exit_profit_ticks_qty_percent(
                    id,
                    from_entry,
                    ticks,
                    mintick,
                    qty_percent,
                    bar_index,
                )
            }
        }
    }

    fn place_exit_loss_ticks_quantity(
        &mut self,
        id: String,
        from_entry: String,
        ticks: f64,
        mintick: f64,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_loss_ticks(id, from_entry, ticks, mintick, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_loss_ticks_qty(id, from_entry, ticks, mintick, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => {
                self.strategy_broker.place_exit_loss_ticks_qty_percent(
                    id,
                    from_entry,
                    ticks,
                    mintick,
                    qty_percent,
                    bar_index,
                )
            }
        }
    }

    fn place_exit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        downside_price: f64,
        upside_price: f64,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self.strategy_broker.place_exit_bracket(
                id,
                from_entry,
                downside_price,
                upside_price,
                bar_index,
            ),
            StrategyExitQuantityArg::Fixed(qty) => self.strategy_broker.place_exit_bracket_qty(
                id,
                from_entry,
                downside_price,
                upside_price,
                qty,
                bar_index,
            ),
            StrategyExitQuantityArg::Percent(qty_percent) => {
                self.strategy_broker.place_exit_bracket_qty_percent(
                    id,
                    from_entry,
                    downside_price,
                    upside_price,
                    qty_percent,
                    bar_index,
                )
            }
        }
    }

    fn place_exit_stop_profit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: StopProfitBracketSpec,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_bracket_stop_profit_ticks(id, from_entry, spec, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_bracket_stop_profit_ticks_qty(id, from_entry, spec, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_bracket_stop_profit_ticks_qty_percent(
                    id,
                    from_entry,
                    spec,
                    qty_percent,
                    bar_index,
                ),
        }
    }

    fn place_exit_loss_limit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: LossLimitBracketSpec,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_bracket_loss_limit_ticks(id, from_entry, spec, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_bracket_loss_limit_ticks_qty(id, from_entry, spec, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_bracket_loss_limit_ticks_qty_percent(
                    id,
                    from_entry,
                    spec,
                    qty_percent,
                    bar_index,
                ),
        }
    }

    fn place_exit_loss_profit_bracket_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: LossProfitBracketSpec,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self
                .strategy_broker
                .place_exit_bracket_loss_profit_ticks(id, from_entry, spec, bar_index),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_bracket_loss_profit_ticks_qty(id, from_entry, spec, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_bracket_loss_profit_ticks_qty_percent(
                    id,
                    from_entry,
                    spec,
                    qty_percent,
                    bar_index,
                ),
        }
    }

    fn place_exit_trail_price_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPriceExitSpec,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self.strategy_broker.place_exit_trail_price(
                id,
                from_entry,
                spec.activation_price,
                spec.offset_ticks,
                spec.mintick,
                bar_index,
            ),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_trail_price_qty(id, from_entry, spec, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_trail_price_qty_percent(id, from_entry, spec, qty_percent, bar_index),
        }
    }

    fn place_exit_trail_points_quantity(
        &mut self,
        id: String,
        from_entry: String,
        spec: TrailPointsExitSpec,
        quantity: StrategyExitQuantityArg,
        bar_index: usize,
    ) {
        match quantity {
            StrategyExitQuantityArg::Full => self.strategy_broker.place_exit_trail_points(
                id,
                from_entry,
                spec.activation_ticks,
                spec.offset_ticks,
                spec.mintick,
                bar_index,
            ),
            StrategyExitQuantityArg::Fixed(qty) => self
                .strategy_broker
                .place_exit_trail_points_qty(id, from_entry, spec, qty, bar_index),
            StrategyExitQuantityArg::Percent(qty_percent) => self
                .strategy_broker
                .place_exit_trail_points_qty_percent(id, from_entry, spec, qty_percent, bar_index),
        }
    }
}
