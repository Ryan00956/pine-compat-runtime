use pine_ir::HirCallArg;

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
        if symbol != chart.symbol() || timeframe != chart.timeframe().value() {
            return Err(RuntimeError {
                message: format!(
                    "request.security supports only current chart symbol `{}` timeframe `{}`",
                    chart.symbol(),
                    chart.timeframe().value()
                ),
            });
        }

        self.eval_expr(&args[2].value)
    }
}
