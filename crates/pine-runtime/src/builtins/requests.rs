use pine_ir::{HirCallArg, HirExpr, HirExprKind};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_request_call(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "request.security" => self.eval_request_security(args),
            _ => return None,
        })
    }

    fn eval_request_security(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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
            return self.eval_provider_security(&symbol, &timeframe, &args[2].value);
        }

        self.eval_expr(&args[2].value)
    }

    fn eval_provider_security(
        &mut self,
        symbol: &str,
        timeframe: &str,
        expression: &HirExpr,
    ) -> Result<PineValue, RuntimeError> {
        let source_name = self
            .request_source_name(expression)
            .ok_or_else(|| RuntimeError {
                message:
                    "request.security provider execution supports only direct OHLCV expressions"
                        .to_owned(),
            })?;
        let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| RuntimeError {
            message: err.to_string(),
        })?;
        let key = RequestKey::new(symbol, timeframe);
        let requested_bars = self
            .request_environment
            .provider()
            .bars(&key)
            .map_err(|err| RuntimeError {
                message: err.to_string(),
            })?;
        let current_time = self
            .current_bar
            .map(|bar| bar.time)
            .ok_or_else(|| RuntimeError {
                message: "request.security has no current chart bar".to_owned(),
            })?;
        let Some(requested_bar) = requested_bars.iter().find(|bar| bar.time == current_time) else {
            return Ok(PineValue::Na);
        };
        Ok(match source_name.as_str() {
            "open" => PineValue::Float(requested_bar.open),
            "high" => PineValue::Float(requested_bar.high),
            "low" => PineValue::Float(requested_bar.low),
            "close" => PineValue::Float(requested_bar.close),
            "volume" => PineValue::Float(requested_bar.volume),
            "time" => PineValue::Int(requested_bar.time),
            _ => PineValue::Na,
        })
    }

    fn request_source_name(&self, expression: &HirExpr) -> Option<String> {
        match &expression.kind {
            HirExprKind::Symbol(symbol_id) => self
                .program
                .symbols
                .iter()
                .find(|symbol| symbol.id == *symbol_id)
                .map(|symbol| symbol.name.as_str())
                .filter(|name| is_request_source_name(name))
                .map(str::to_owned),
            HirExprKind::Builtin(name) if is_request_source_name(name) => Some(name.to_owned()),
            _ => None,
        }
    }
}

fn is_request_source_name(name: &str) -> bool {
    matches!(name, "open" | "high" | "low" | "close" | "volume" | "time")
}
