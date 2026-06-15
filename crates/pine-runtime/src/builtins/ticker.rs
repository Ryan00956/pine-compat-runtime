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
            "ticker.heikinashi" => self.eval_ticker_heikinashi(args),
            "ticker.kagi" => self.eval_ticker_kagi(args),
            "ticker.linebreak" => self.eval_ticker_linebreak(args),
            "ticker.new" => self.eval_ticker_new(args),
            "ticker.modify" => self.eval_ticker_modify(args),
            "ticker.renko" => self.eval_ticker_renko(args),
            "ticker.standard" => self.eval_ticker_standard(args),
            _ => return None,
        })
    }

    fn eval_ticker_heikinashi(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(tickerid) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        let symbol = standard_ticker_id(&tickerid);
        Ok(PineValue::String(non_standard_ticker_id(
            &symbol,
            "heikinashi",
        )))
    }

    fn eval_ticker_kagi(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(tickerid) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(style) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };
        let Some(param) = numeric_param_string(&self.eval_expr(&args[2].value)?) else {
            return Ok(PineValue::Na);
        };

        let symbol = standard_ticker_id(&tickerid);
        Ok(PineValue::String(kagi_ticker_id(&symbol, &style, &param)))
    }

    fn eval_ticker_linebreak(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(tickerid) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::Int(number_of_lines) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };

        let symbol = standard_ticker_id(&tickerid);
        Ok(PineValue::String(linebreak_ticker_id(
            &symbol,
            number_of_lines,
        )))
    }

    fn eval_ticker_renko(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(tickerid) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(style) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };
        let Some(param) = numeric_param_string(&self.eval_expr(&args[2].value)?) else {
            return Ok(PineValue::Na);
        };

        let symbol = standard_ticker_id(&tickerid);
        Ok(PineValue::String(renko_ticker_id(&symbol, &style, &param)))
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

fn non_standard_ticker_id(symbol: &str, chart: &str) -> String {
    format!(
        r#"{{"chart":"{}","symbol":"{}"}}"#,
        escape_json_string(chart),
        escape_json_string(symbol)
    )
}

fn linebreak_ticker_id(symbol: &str, number_of_lines: i64) -> String {
    format!(
        r#"{{"chart":"linebreak","lines":{},"symbol":"{}"}}"#,
        number_of_lines,
        escape_json_string(symbol)
    )
}

fn kagi_ticker_id(symbol: &str, style: &str, param: &str) -> String {
    format!(
        r#"{{"chart":"kagi","style":"{}","param":{},"symbol":"{}"}}"#,
        escape_json_string(style),
        param,
        escape_json_string(symbol)
    )
}

fn renko_ticker_id(symbol: &str, style: &str, param: &str) -> String {
    format!(
        r#"{{"chart":"renko","style":"{}","param":{},"symbol":"{}"}}"#,
        escape_json_string(style),
        param,
        escape_json_string(symbol)
    )
}

fn numeric_param_string(value: &PineValue) -> Option<String> {
    match value {
        PineValue::Int(value) => Some(value.to_string()),
        PineValue::Float(value) if value.is_finite() => Some(value.to_string()),
        _ => None,
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

    #[test]
    fn non_standard_ticker_preserves_escaped_symbol_field() {
        assert_eq!(
            non_standard_ticker_id(r#"TEST:Q\""#, "heikinashi"),
            r#"{"chart":"heikinashi","symbol":"TEST:Q\\\""}"#
        );
    }

    #[test]
    fn linebreak_ticker_preserves_symbol_and_line_count_fields() {
        assert_eq!(
            linebreak_ticker_id(r#"TEST:Q\""#, 3),
            r#"{"chart":"linebreak","lines":3,"symbol":"TEST:Q\\\""}"#
        );
    }

    #[test]
    fn kagi_ticker_preserves_symbol_and_numeric_param_fields() {
        assert_eq!(
            kagi_ticker_id(r#"TEST:Q\""#, r#"AT\"R"#, "10"),
            r#"{"chart":"kagi","style":"AT\\\"R","param":10,"symbol":"TEST:Q\\\""}"#
        );
    }

    #[test]
    fn renko_ticker_preserves_symbol_and_numeric_param_fields() {
        assert_eq!(
            renko_ticker_id(r#"TEST:Q\""#, r#"AT\"R"#, "10"),
            r#"{"chart":"renko","style":"AT\\\"R","param":10,"symbol":"TEST:Q\\\""}"#
        );
        assert_eq!(numeric_param_string(&PineValue::Int(10)), Some("10".into()));
        assert_eq!(
            numeric_param_string(&PineValue::Float(2.5)),
            Some("2.5".into())
        );
        assert_eq!(numeric_param_string(&PineValue::Na), None);
    }
}
