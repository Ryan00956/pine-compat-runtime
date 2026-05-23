use pine_ir::{CallSiteId, HirCallArg, HirExpr};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_request_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "request.security" => self.eval_request_security(call_site_id, args),
            _ => return None,
        })
    }

    fn eval_request_security(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError {
                message: format!("request.security expects 3 argument(s), got {}", args.len()),
            });
        }

        let PineValue::String(symbol) = self.eval_expr(&args[0].value)? else {
            return Err(RuntimeError {
                message: "request.security symbol must evaluate to string".to_owned(),
            });
        };
        let PineValue::String(timeframe) = self.eval_expr(&args[1].value)? else {
            return Err(RuntimeError {
                message: "request.security timeframe must evaluate to string".to_owned(),
            });
        };

        let chart = self.request_environment.chart();
        if timeframe != chart.timeframe().value() {
            return Err(RuntimeError {
                message: format!(
                    "request.security supports only current chart symbol `{}` timeframe `{}`",
                    chart.symbol(),
                    chart.timeframe().value()
                ),
            });
        }

        if symbol != chart.symbol() {
            return self.eval_provider_security(call_site_id, &symbol, &timeframe, &args[2].value);
        }

        self.eval_expr(&args[2].value)
    }

    fn eval_provider_security(
        &mut self,
        call_site_id: CallSiteId,
        symbol: &str,
        timeframe: &str,
        expression: &HirExpr,
    ) -> Result<PineValue, RuntimeError> {
        let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| RuntimeError {
            message: err.to_string(),
        })?;
        let key = RequestKey::new(symbol, timeframe);
        let current_time = self
            .current_bar
            .map(|bar| bar.time)
            .ok_or_else(|| RuntimeError {
                message: "request.security has no current chart bar".to_owned(),
            })?;

        let cache_key = RequestCacheKey::new(
            call_site_id,
            key.symbol(),
            key.timeframe().value(),
            format!("{:?}", expression.kind),
        );
        if !self.request_cache.contains_key(&cache_key) {
            let requested_bars = self
                .request_environment
                .provider()
                .bars(&key)
                .map_err(|err| RuntimeError {
                    message: err.to_string(),
                })?;
            let requested_values = self.evaluate_requested_values(requested_bars, expression)?;
            self.request_cache
                .insert(cache_key.clone(), requested_values);
        }

        Ok(self
            .request_cache
            .get(&cache_key)
            .and_then(|requested_values| {
                requested_values
                    .iter()
                    .find(|(time, _)| *time == current_time)
                    .map(|(_, value)| value.clone())
            })
            .unwrap_or(PineValue::Na))
    }

    fn evaluate_requested_values(
        &self,
        requested_bars: &[Bar],
        expression: &HirExpr,
    ) -> Result<Vec<(i64, PineValue)>, RuntimeError> {
        let mut runtime = HistoricalRuntime::with_request_environment(
            self.program,
            self.request_environment.clone(),
        );
        let mut values = Vec::with_capacity(requested_bars.len());
        for bar in requested_bars {
            values.push((
                bar.time,
                runtime.eval_requested_bar_expression(*bar, expression)?,
            ));
        }
        Ok(values)
    }

    fn eval_requested_bar_expression(
        &mut self,
        bar: Bar,
        expression: &HirExpr,
    ) -> Result<PineValue, RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        self.current_bar = Some(bar);
        self.series_store.set_current_bar(bar_index);
        self.current_symbols.clear();
        self.current_series.clear();
        self.set_builtin_symbols(&bar, bar_index)?;

        let value = self.eval_expr(expression)?;
        self.commit_current_series()?;
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        self.current_bar = None;
        Ok(value)
    }
}
