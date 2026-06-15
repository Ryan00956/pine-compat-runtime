use pine_ir::{HirCallArg, HirExpr};
use regex::Regex;

use crate::builtins::time::{
    format_datetime_with_offset, format_fixed_timezone_offset, format_utc_datetime,
    parse_fixed_timezone_offset, utc_datetime_from_millis,
};
use crate::*;

pub(crate) fn replace_nth_non_overlapping(
    source: &str,
    target: &str,
    replacement: &str,
    occurrence: usize,
) -> String {
    let Some((byte_index, _)) = source.match_indices(target).nth(occurrence) else {
        return source.to_owned();
    };
    let mut result = String::with_capacity(source.len() + replacement.len());
    result.push_str(&source[..byte_index]);
    result.push_str(replacement);
    result.push_str(&source[byte_index + target.len()..]);
    result
}

pub(crate) fn replace_zero_width_occurrence(
    source: &str,
    replacement: &str,
    occurrence: usize,
) -> String {
    let char_count = source.chars().count();
    if occurrence > char_count {
        return source.to_owned();
    }

    let mut result = String::with_capacity(source.len() + replacement.len());
    if occurrence == 0 {
        result.push_str(replacement);
    }
    for (index, ch) in source.chars().enumerate() {
        result.push(ch);
        if index + 1 == occurrence {
            result.push_str(replacement);
        }
    }
    result
}

pub(crate) fn replace_all_zero_width_boundaries(source: &str, replacement: &str) -> String {
    let mut result =
        String::with_capacity(source.len() + replacement.len() * (source.chars().count() + 1));
    result.push_str(replacement);
    for ch in source.chars() {
        result.push(ch);
        result.push_str(replacement);
    }
    result
}

pub(crate) fn is_pine_numeric_string(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (significand, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((significand, exponent)) => (significand, Some(exponent)),
        None => (unsigned, None),
    };
    if unsigned
        .chars()
        .filter(|ch| matches!(ch, 'e' | 'E'))
        .count()
        > 1
    {
        return false;
    }

    let mut saw_digit = false;
    let mut saw_decimal = false;
    for ch in significand.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
        } else if ch == '.' && !saw_decimal {
            saw_decimal = true;
        } else {
            return false;
        }
    }
    if !saw_digit {
        return false;
    }

    if let Some(exponent) = exponent {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        !exponent.is_empty() && exponent.chars().all(|ch| ch.is_ascii_digit())
    } else {
        true
    }
}

pub(crate) fn stringify_array(values: &[PineValue], format: &str) -> String {
    let mut result = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            result.push_str(", ");
        }
        result.push_str(&stringify_array_element(value, format));
    }
    result.push(']');
    result
}

pub(crate) fn stringify_array_element(value: &PineValue, format: &str) -> String {
    match value {
        PineValue::Int(value) => format_number(*value as f64, format),
        PineValue::Float(value) => format_number(*value, format),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => value.clone(),
        PineValue::Na => "NaN".to_owned(),
        _ => "NaN".to_owned(),
    }
}

pub(crate) fn stringify_array_join_element(value: &PineValue) -> String {
    match value {
        PineValue::Int(value) => format_number(*value as f64, "#.########"),
        PineValue::Float(value) => format_number(*value, "#.########"),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => value.clone(),
        PineValue::Color(value) => value.to_string(),
        PineValue::Na => "NaN".to_owned(),
        _ => "NaN".to_owned(),
    }
}

pub(crate) fn format_string_placeholders(
    format_string: &str,
    values: &[PineValue],
    runtime: &HistoricalRuntime<'_>,
) -> Result<String, RuntimeError> {
    let mut result = String::new();
    let mut chars = format_string.char_indices().peekable();
    while let Some((byte_index, ch)) = chars.next() {
        match ch {
            '\'' => {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                    result.push('\'');
                } else {
                    for (_, literal) in chars.by_ref() {
                        if literal == '\'' {
                            break;
                        }
                        result.push(literal);
                    }
                }
            }
            '{' => {
                let start = byte_index + ch.len_utf8();
                let Some((end, _)) = chars.find(|(_, next)| *next == '}') else {
                    return Err(RuntimeError {
                        message: "str.format has unmatched `{`".to_owned(),
                    });
                };
                let placeholder = &format_string[start..end];
                if let Some(formatted) = format_placeholder(placeholder, values, runtime)? {
                    result.push_str(&formatted);
                } else {
                    result.push('{');
                    result.push_str(placeholder);
                    result.push('}');
                }
            }
            '}' => {
                return Err(RuntimeError {
                    message: "str.format has unmatched `}`".to_owned(),
                });
            }
            _ => result.push(ch),
        }
    }
    Ok(result)
}

pub(crate) fn format_placeholder(
    placeholder: &str,
    values: &[PineValue],
    runtime: &HistoricalRuntime<'_>,
) -> Result<Option<String>, RuntimeError> {
    let mut parts = placeholder.splitn(3, ',').map(str::trim);
    let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
        return Ok(None);
    };
    let Some(value) = values.get(index) else {
        return Ok(None);
    };
    let Some(modifier) = parts.next() else {
        return Ok(Some(runtime.stringify_value(value, "#,###.###")));
    };

    if matches!(modifier, "date" | "time") {
        return format_datetime_placeholder(modifier, parts.next().map(str::trim), value, runtime)
            .map(Some);
    }

    if modifier != "number" {
        return Ok(Some(runtime.stringify_value(value, "#,###.###")));
    }

    let format = match parts.next().map(str::trim) {
        Some("integer") => "#,###",
        Some("percent") => "#.##%",
        Some("currency") => return Ok(Some(format_currency_placeholder(value, runtime))),
        Some(format) if !format.is_empty() => format,
        _ => "#,###.###",
    };
    Ok(Some(runtime.stringify_value(value, format)))
}

pub(crate) fn format_datetime_placeholder(
    modifier: &str,
    format: Option<&str>,
    value: &PineValue,
    runtime: &HistoricalRuntime<'_>,
) -> Result<String, RuntimeError> {
    let PineValue::Int(timestamp) = value else {
        return Ok(runtime.stringify_value(value, "#,###.###"));
    };
    let format = match (modifier, format.filter(|format| !format.is_empty())) {
        (_, Some(format)) => format,
        ("date", None) => "yyyy-MM-dd",
        ("time", None) => "HH:mm:ss",
        _ => unreachable!("validated str.format date/time modifier"),
    };
    let datetime = utc_datetime_from_millis(*timestamp).map_err(|_| RuntimeError {
        message: format!("str.format timestamp is out of range: {timestamp}"),
    })?;
    Ok(format_utc_datetime(datetime, format))
}

pub(crate) fn format_currency_placeholder(
    value: &PineValue,
    runtime: &HistoricalRuntime<'_>,
) -> String {
    let formatted = match value {
        PineValue::Int(value) => format_number(*value as f64, "#,###.00"),
        PineValue::Float(value) => format_number(*value, "#,###.00"),
        _ => return runtime.stringify_value(value, "#,###.00"),
    };
    if formatted == "NaN" {
        formatted
    } else if let Some(unsigned) = formatted.strip_prefix('-') {
        format!("-${unsigned}")
    } else {
        format!("${formatted}")
    }
}

pub(crate) fn format_number(value: f64, format: &str) -> String {
    if !value.is_finite() {
        return "NaN".to_owned();
    }

    let format = match format {
        "" | "format.mintick" | "format.price" => "#.########",
        "format.percent" => "#.##%",
        "format.volume" => "#.##",
        other => other,
    };
    let percent = format.ends_with('%');
    let pattern = format.strip_suffix('%').unwrap_or(format);
    let value = if percent { value * 100.0 } else { value };

    let (whole_pattern, fractional_pattern) = pattern.split_once('.').unwrap_or((pattern, ""));
    let decimal_places = fractional_pattern
        .chars()
        .filter(|ch| matches!(ch, '#' | '0'))
        .count();
    let required_fractional = fractional_pattern.chars().filter(|ch| *ch == '0').count();
    let min_integer_digits = whole_pattern.chars().filter(|ch| *ch == '0').count();
    let use_grouping = whole_pattern.contains(',');
    let rounded = if decimal_places == 0 {
        value.round()
    } else {
        let factor = 10_f64.powi(decimal_places.min(308) as i32);
        (value * factor).round() / factor
    };
    let negative = rounded.is_sign_negative() && rounded != 0.0;
    let abs_value = rounded.abs();
    let raw = format!("{abs_value:.decimal_places$}");
    let (whole, fractional) = raw.split_once('.').unwrap_or((raw.as_str(), ""));
    let mut whole = whole.to_owned();
    if whole.len() < min_integer_digits {
        whole = format!("{}{}", "0".repeat(min_integer_digits - whole.len()), whole);
    }
    if use_grouping {
        whole = group_integer_digits(&whole);
    }

    let mut fractional = fractional.to_owned();
    while fractional.len() > required_fractional && fractional.ends_with('0') {
        fractional.pop();
    }

    let mut result = String::new();
    if negative {
        result.push('-');
    }
    result.push_str(&whole);
    if !fractional.is_empty() {
        result.push('.');
        result.push_str(&fractional);
    }
    if percent {
        result.push('%');
    }
    result
}

pub(crate) fn group_integer_digits(value: &str) -> String {
    let mut result = String::new();
    for (index, ch) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringCase {
    Upper,
    Lower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringMatch {
    Contains,
    StartsWith,
    EndsWith,
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_string_call(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        if !callee.starts_with("str.") {
            return None;
        }

        Some(match callee {
            "str.length" => self.eval_str_length(args),
            "str.upper" => self.eval_str_case(args, StringCase::Upper),
            "str.lower" => self.eval_str_case(args, StringCase::Lower),
            "str.contains" => self.eval_str_match(args, StringMatch::Contains),
            "str.startswith" => self.eval_str_match(args, StringMatch::StartsWith),
            "str.endswith" => self.eval_str_match(args, StringMatch::EndsWith),
            "str.pos" => self.eval_str_pos(args),
            "str.substring" => self.eval_str_substring(args),
            "str.trim" => self.eval_str_trim(args),
            "str.repeat" => self.eval_str_repeat(args),
            "str.replace" => self.eval_str_replace(args),
            "str.replace_all" => self.eval_str_replace_all(args),
            "str.tonumber" => self.eval_str_tonumber(args),
            "str.tostring" => self.eval_str_tostring(args),
            "str.format" => self.eval_str_format(args),
            "str.match" => self.eval_str_match_regex(args),
            "str.split" => self.eval_str_split(args),
            "str.format_time" => self.eval_str_format_time(args),
            _ => return None,
        })
    }

    pub(crate) fn eval_str_length(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Int(value.chars().count() as i64))
    }

    pub(crate) fn eval_str_case(
        &mut self,
        args: &[HirCallArg],
        string_case: StringCase,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        let value = match string_case {
            StringCase::Upper => value.to_uppercase(),
            StringCase::Lower => value.to_lowercase(),
        };
        Ok(PineValue::String(value))
    }

    pub(crate) fn eval_str_match(
        &mut self,
        args: &[HirCallArg],
        string_match: StringMatch,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(pattern) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };

        let matched = match string_match {
            StringMatch::Contains => source.contains(&pattern),
            StringMatch::StartsWith => source.starts_with(&pattern),
            StringMatch::EndsWith => source.ends_with(&pattern),
        };
        Ok(PineValue::Bool(matched))
    }

    pub(crate) fn eval_str_pos(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let pattern = match self.eval_expr(&args[1].value)? {
            PineValue::String(pattern) => pattern,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        if pattern.is_empty() {
            return Ok(PineValue::Int(0));
        }

        Ok(source.find(&pattern).map_or(PineValue::Na, |byte_index| {
            PineValue::Int(source[..byte_index].chars().count() as i64)
        }))
    }

    pub(crate) fn eval_str_substring(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let begin = self.eval_optional_string_index(&args[1].value, 0)?;
        let chars: Vec<char> = source.chars().collect();
        let len = chars.len() as i64;
        if begin < 0 || begin > len {
            return Err(RuntimeError {
                message: format!("str.substring begin_pos {begin} is outside string length {len}"),
            });
        }

        let end = if let Some(arg) = args.get(2) {
            self.eval_optional_string_index(&arg.value, len)?
        } else {
            len
        }
        .min(len);
        if end < begin {
            return Err(RuntimeError {
                message: format!("str.substring end_pos {end} is less than begin_pos {begin}"),
            });
        }

        Ok(PineValue::String(
            chars[begin as usize..end as usize].iter().collect(),
        ))
    }

    pub(crate) fn eval_str_trim(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::String(
            value
                .trim_matches(|ch: char| ch.is_ascii_whitespace())
                .to_owned(),
        ))
    }

    pub(crate) fn eval_str_repeat(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let Some(repeat) = self.eval_string_index(&args[1].value)? else {
            return Ok(PineValue::Na);
        };
        let separator = if let Some(arg) = args.get(2) {
            let PineValue::String(separator) = self.eval_expr(&arg.value)? else {
                return Ok(PineValue::Na);
            };
            separator
        } else {
            String::new()
        };
        if repeat < 0 {
            return Err(RuntimeError {
                message: format!("str.repeat count cannot be negative: {repeat}"),
            });
        }

        let repeat = repeat as usize;
        let result_chars = repeat
            .saturating_mul(source.chars().count())
            .saturating_add(
                repeat
                    .saturating_sub(1)
                    .saturating_mul(separator.chars().count()),
            );
        if result_chars > MAX_STRING_CHARS {
            return Err(RuntimeError {
                message: format!("str.repeat result cannot exceed {MAX_STRING_CHARS} characters"),
            });
        }

        let mut result = String::new();
        for index in 0..repeat {
            if index > 0 {
                result.push_str(&separator);
            }
            result.push_str(&source);
        }
        Ok(PineValue::String(result))
    }

    pub(crate) fn eval_str_replace(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((source, target, replacement)) = self.eval_replace_strings(args)? else {
            return Ok(PineValue::Na);
        };
        let occurrence = if let Some(arg) = args.get(3) {
            self.eval_optional_string_index(&arg.value, 0)?
        } else {
            0
        };
        if occurrence < 0 {
            return Ok(PineValue::String(source));
        }

        let result = if target.is_empty() {
            replace_zero_width_occurrence(&source, &replacement, occurrence as usize)
        } else {
            replace_nth_non_overlapping(&source, &target, &replacement, occurrence as usize)
        };
        self.string_value_or_error(result, "str.replace")
    }

    pub(crate) fn eval_str_replace_all(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((source, target, replacement)) = self.eval_replace_strings(args)? else {
            return Ok(PineValue::Na);
        };
        let result = if target.is_empty() {
            replace_all_zero_width_boundaries(&source, &replacement)
        } else {
            source.replace(&target, &replacement)
        };
        self.string_value_or_error(result, "str.replace_all")
    }

    pub(crate) fn eval_str_tonumber(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        if !is_pine_numeric_string(&value) {
            return Ok(PineValue::Na);
        }

        Ok(value
            .parse::<f64>()
            .ok()
            .map_or(PineValue::Na, finite_float_or_na))
    }

    pub(crate) fn eval_str_tostring(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        let format = if let Some(arg) = args.get(1) {
            match self.eval_expr(&arg.value)? {
                PineValue::String(format) => format,
                PineValue::Na => "#.########".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            "#.########".to_owned()
        };
        let result = self.stringify_value(&value, &format);
        self.string_value_or_error(result, "str.tostring")
    }

    pub(crate) fn eval_str_format(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(format_string) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let mut values = Vec::with_capacity(args.len().saturating_sub(1));
        for arg in &args[1..] {
            values.push(self.eval_expr(&arg.value)?);
        }

        let result = format_string_placeholders(&format_string, &values, self)?;
        self.string_value_or_error(result, "str.format")
    }

    pub(crate) fn eval_str_match_regex(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(regex) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };
        let regex = Regex::new(&regex).map_err(|err| RuntimeError {
            message: format!("str.match invalid regex: {err}"),
        })?;

        Ok(PineValue::String(
            regex
                .find(&source)
                .map_or_else(String::new, |matched| matched.as_str().to_owned()),
        ))
    }

    pub(crate) fn eval_str_split(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::String(separator) = self.eval_expr(&args[1].value)? else {
            return Ok(PineValue::Na);
        };

        let parts: Vec<PineValue> = if separator.is_empty() {
            source
                .chars()
                .map(|ch| PineValue::String(ch.to_string()))
                .collect()
        } else {
            source
                .split(&separator)
                .map(|part| PineValue::String(part.to_owned()))
                .collect()
        };
        if parts.len() > MAX_ARRAY_ELEMENTS {
            return Err(RuntimeError {
                message: format!("str.split cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
            });
        }

        Ok(self.new_array_from_values(ArrayElementKind::String, parts))
    }

    pub(crate) fn eval_str_format_time(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let timestamp = match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => value,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        let format = if let Some(arg) = args.get(1) {
            match self.eval_expr(&arg.value)? {
                PineValue::String(format) => format,
                PineValue::Na => "yyyy-MM-dd'T'HH:mm:ssZ".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            "yyyy-MM-dd'T'HH:mm:ssZ".to_owned()
        };
        let timezone = if let Some(arg) = args.get(2) {
            match self.eval_expr(&arg.value)? {
                PineValue::String(timezone) => timezone,
                PineValue::Na => "UTC".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            "UTC".to_owned()
        };
        let timezone_offset_seconds =
            parse_fixed_timezone_offset(&timezone).ok_or_else(|| RuntimeError {
                message: format!("str.format_time unsupported timezone `{timezone}`"),
            })?;
        let datetime = utc_datetime_from_millis(timestamp).map_err(|_| RuntimeError {
            message: format!("str.format_time timestamp is out of range: {timestamp}"),
        })?;
        let Some(offset) = chrono::Duration::try_seconds(i64::from(timezone_offset_seconds)) else {
            return Err(RuntimeError {
                message: format!("str.format_time unsupported timezone `{timezone}`"),
            });
        };
        let Some(datetime) = datetime.checked_add_signed(offset) else {
            return Err(RuntimeError {
                message: format!("str.format_time timestamp is out of range: {timestamp}"),
            });
        };

        let timezone_offset = format_fixed_timezone_offset(timezone_offset_seconds);
        let result = format_datetime_with_offset(datetime, &format, &timezone_offset);
        self.string_value_or_error(result, "str.format_time")
    }

    pub(crate) fn stringify_value(&self, value: &PineValue, format: &str) -> String {
        match value {
            PineValue::Int(value) => format_number(*value as f64, format),
            PineValue::Float(value) => format_number(*value, format),
            PineValue::Bool(value) => value.to_string(),
            PineValue::String(value) => value.clone(),
            PineValue::Array(id) => self
                .array_store
                .get(id)
                .map(|values| stringify_array(values, format))
                .unwrap_or_else(|| "NaN".to_owned()),
            PineValue::Na => "NaN".to_owned(),
            _ => "NaN".to_owned(),
        }
    }

    pub(crate) fn eval_replace_strings(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<Option<(String, String, String)>, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(None);
        };
        let PineValue::String(target) = self.eval_expr(&args[1].value)? else {
            return Ok(None);
        };
        let PineValue::String(replacement) = self.eval_expr(&args[2].value)? else {
            return Ok(None);
        };
        Ok(Some((source, target, replacement)))
    }

    pub(crate) fn string_value_or_error(
        &self,
        value: String,
        function: &str,
    ) -> Result<PineValue, RuntimeError> {
        if value.chars().count() > MAX_STRING_CHARS {
            return Err(RuntimeError {
                message: format!("{function} result cannot exceed {MAX_STRING_CHARS} characters"),
            });
        }
        Ok(PineValue::String(value))
    }

    pub(crate) fn eval_string_index(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<i64>, RuntimeError> {
        Ok(match self.eval_expr(expr)? {
            PineValue::Int(value) => Some(value),
            PineValue::Float(value) if value.is_finite() => Some(value as i64),
            PineValue::Na => None,
            _ => None,
        })
    }

    pub(crate) fn eval_optional_string_index(
        &mut self,
        expr: &HirExpr,
        default: i64,
    ) -> Result<i64, RuntimeError> {
        Ok(self.eval_string_index(expr)?.unwrap_or(default))
    }
}
