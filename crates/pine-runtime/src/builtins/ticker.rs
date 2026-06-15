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
            "ticker.new" => self.eval_ticker_new(args),
            "ticker.modify" => self.eval_ticker_modify(args),
            "ticker.standard" => self.eval_ticker_standard(args),
            _ => return None,
        })
    }

    fn eval_ticker_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(prefix) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(ticker) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };

        let symbol = format!("{prefix}:{ticker}");
        let Some(session_arg) = args.get(2) else {
            return Ok(PineValue::String(symbol));
        };
        let PineValue::String(session) = self.eval_expr(&session_arg.value)? else {
            return Ok(PineValue::Na);
        };
        let adjustment = if let Some(adjustment_arg) = args.get(3) {
            let PineValue::String(adjustment) = self.eval_expr(&adjustment_arg.value)? else {
                return Ok(PineValue::Na);
            };
            Some(adjustment)
        } else {
            None
        };

        Ok(PineValue::String(modified_ticker_id(
            &symbol,
            &session,
            adjustment.as_deref(),
        )))
    }

    fn eval_ticker_modify(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(tickerid) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        let Some(session_arg) = args.get(1) else {
            return Ok(PineValue::String(tickerid));
        };
        let PineValue::String(session) = self.eval_expr(&session_arg.value)? else {
            return Ok(PineValue::Na);
        };
        let adjustment = if let Some(adjustment_arg) = args.get(2) {
            let PineValue::String(adjustment) = self.eval_expr(&adjustment_arg.value)? else {
                return Ok(PineValue::Na);
            };
            Some(adjustment)
        } else {
            None
        };

        let symbol = standard_ticker_id(&tickerid);
        Ok(PineValue::String(modified_ticker_id(
            &symbol,
            &session,
            adjustment.as_deref(),
        )))
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

fn modified_ticker_id(symbol: &str, session: &str, adjustment: Option<&str>) -> String {
    let session = escape_json_string(session);
    let symbol = escape_json_string(symbol);
    match adjustment {
        Some(adjustment) => format!(
            r#"{{"session":"{}","adjustment":"{}","symbol":"{}"}}"#,
            session,
            escape_json_string(adjustment),
            symbol
        ),
        None => format!(r#"{{"session":"{}","symbol":"{}"}}"#, session, symbol),
    }
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str(r#"\\"#),
            '"' => escaped.push_str(r#"\""#),
            _ => escaped.push(ch),
        }
    }
    escaped
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

    #[test]
    fn modified_ticker_escapes_json_string_fields() {
        assert_eq!(
            modified_ticker_id(r#"TEST:Q\""#, r#"reg\"ular"#, None),
            r#"{"session":"reg\\\"ular","symbol":"TEST:Q\\\""}"#
        );
        assert_eq!(
            modified_ticker_id("NASDAQ:AAPL", "extended", Some("dividends")),
            r#"{"session":"extended","adjustment":"dividends","symbol":"NASDAQ:AAPL"}"#
        );
    }
}
