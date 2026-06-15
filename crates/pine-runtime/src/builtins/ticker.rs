use pine_ir::HirCallArg;

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_ticker_call(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        if !callee.starts_with("ticker.") {
            return None;
        }

        Some(match callee {
            "ticker.standard" => self.eval_ticker_standard(args),
            _ => return None,
        })
    }

    fn eval_ticker_standard(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(symbol) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::String(standard_ticker_id(&symbol)))
    }
}

fn standard_ticker_id(symbol: &str) -> String {
    extract_json_symbol_field(symbol).unwrap_or_else(|| symbol.to_owned())
}

fn extract_json_symbol_field(value: &str) -> Option<String> {
    let marker = "\"symbol\"";
    let after_marker = value.split_once(marker)?.1.trim_start();
    let after_colon = after_marker.strip_prefix(':')?.trim_start();
    let mut chars = after_colon.strip_prefix('"')?.chars();
    let mut symbol = String::new();
    let mut escaped = false;

    for ch in chars.by_ref() {
        if escaped {
            symbol.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(symbol),
            _ => symbol.push(ch),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_ticker_extracts_known_symbol_field_shape() {
        assert_eq!(
            standard_ticker_id(r#"{"session":"extended","symbol":"NASDAQ:AAPL"}"#),
            "NASDAQ:AAPL"
        );
        assert_eq!(
            standard_ticker_id(r#""settlement-as-close":true,"symbol":"COMEX:GC1!""#),
            "COMEX:GC1!"
        );
        assert_eq!(standard_ticker_id("NASDAQ:AAPL"), "NASDAQ:AAPL");
    }
}
