use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use pine_ir::HirCallArg;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeComponent {
    Year,
    Month,
    WeekOfYear,
    DayOfMonth,
    DayOfWeek,
    Hour,
    Minute,
    Second,
}

impl TimeComponent {
    fn function_name(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::WeekOfYear => "weekofyear",
            Self::DayOfMonth => "dayofmonth",
            Self::DayOfWeek => "dayofweek",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
        }
    }

    fn value(self, datetime: DateTime<Utc>) -> i64 {
        match self {
            Self::Year => datetime.year() as i64,
            Self::Month => datetime.month() as i64,
            Self::WeekOfYear => datetime.iso_week().week() as i64,
            Self::DayOfMonth => datetime.day() as i64,
            Self::DayOfWeek => dayofweek_value(datetime),
            Self::Hour => datetime.hour() as i64,
            Self::Minute => datetime.minute() as i64,
            Self::Second => datetime.second() as i64,
        }
    }
}

pub(crate) fn dayofweek_value(datetime: DateTime<Utc>) -> i64 {
    i64::from(datetime.weekday().num_days_from_sunday()) + 1
}

pub(crate) fn is_supported_utc_timezone(timezone: &str) -> bool {
    matches!(
        timezone,
        "UTC"
            | "Etc/UTC"
            | "GMT"
            | "Z"
            | "+0000"
            | "-0000"
            | "+00:00"
            | "-00:00"
            | "UTC+0"
            | "UTC-0"
            | "UTC+00:00"
            | "UTC-00:00"
            | "GMT+0"
            | "GMT-0"
            | "GMT+00:00"
            | "GMT-00:00"
    )
}

pub(crate) fn timeframe_from_seconds(seconds: i64) -> Option<String> {
    if seconds <= 0 {
        return None;
    }
    if matches!(seconds, 1 | 5 | 10 | 15 | 30 | 45) {
        return Some(format!("{seconds}S"));
    }

    if seconds % 2_592_000 == 0 {
        let months = seconds / 2_592_000;
        if (1..=12).contains(&months) {
            return Some(if months == 1 {
                "M".to_owned()
            } else {
                format!("{months}M")
            });
        }
    }
    if seconds % 604_800 == 0 {
        let weeks = seconds / 604_800;
        if (1..=52).contains(&weeks) {
            return Some(if weeks == 1 {
                "W".to_owned()
            } else {
                format!("{weeks}W")
            });
        }
    }
    if seconds % 86_400 == 0 {
        let days = seconds / 86_400;
        if (1..=365).contains(&days) {
            return Some(if days == 1 {
                "D".to_owned()
            } else {
                format!("{days}D")
            });
        }
    }
    if seconds % 60 == 0 {
        let minutes = seconds / 60;
        if (1..=1440).contains(&minutes) {
            return Some(minutes.to_string());
        }
    }

    None
}

pub(crate) fn timeframe_bucket(timestamp_ms: i64, seconds: i64) -> Option<i64> {
    let duration_ms = seconds.checked_mul(1000)?;
    if duration_ms <= 0 {
        return None;
    }
    Some(timestamp_ms.div_euclid(duration_ms))
}

pub(crate) fn timeframe_seconds(timeframe: &str) -> Option<i64> {
    if timeframe.is_empty() {
        return timeframe_seconds(DEFAULT_CHART_TIMEFRAME);
    }

    let unit = timeframe
        .chars()
        .last()
        .filter(|ch| ch.is_ascii_alphabetic());
    let number = if unit.is_some() {
        &timeframe[..timeframe.len() - 1]
    } else {
        timeframe
    };
    let multiplier = if number.is_empty() {
        1
    } else {
        number.parse::<i64>().ok()?
    };
    if multiplier <= 0 {
        return None;
    }

    match unit {
        None if (1..=1440).contains(&multiplier) => multiplier.checked_mul(60),
        Some('S') if matches!(multiplier, 1 | 5 | 10 | 15 | 30 | 45) => Some(multiplier),
        Some('D') if (1..=365).contains(&multiplier) => multiplier.checked_mul(86_400),
        Some('W') if (1..=52).contains(&multiplier) => multiplier.checked_mul(604_800),
        Some('M') if (1..=12).contains(&multiplier) => multiplier.checked_mul(2_592_000),
        _ => None,
    }
}

pub(crate) fn timestamp_unsigned_parts(
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> Option<(u32, u32, u32, u32, u32)> {
    Some((
        u32::try_from(month).ok()?,
        u32::try_from(day).ok()?,
        u32::try_from(hour).ok()?,
        u32::try_from(minute).ok()?,
        u32::try_from(second).ok()?,
    ))
}

pub(crate) fn utc_datetime_from_millis(timestamp: i64) -> Result<DateTime<Utc>, RuntimeError> {
    Utc.timestamp_millis_opt(timestamp)
        .single()
        .ok_or_else(|| RuntimeError {
            message: format!("timestamp is out of range: {timestamp}"),
        })
}

pub(crate) fn format_utc_datetime(datetime: DateTime<Utc>, format: &str) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            for literal in chars.by_ref() {
                if literal == '\'' {
                    break;
                }
                result.push(literal);
            }
            continue;
        }

        let count = consume_same_chars(&mut chars, ch) + 1;
        match ch {
            'y' | 'Y' => {
                if count == 2 {
                    result.push_str(&format!("{:02}", datetime.year().rem_euclid(100)));
                } else {
                    result.push_str(&format!("{:04}", datetime.year()));
                }
            }
            'M' => result.push_str(&format_month(datetime.month(), count)),
            'd' => push_padded_or_plain(&mut result, datetime.day(), count),
            'H' => push_padded_or_plain(&mut result, datetime.hour(), count),
            'h' => {
                let hour = match datetime.hour() % 12 {
                    0 => 12,
                    hour => hour,
                };
                push_padded_or_plain(&mut result, hour, count);
            }
            'm' => push_padded_or_plain(&mut result, datetime.minute(), count),
            's' => push_padded_or_plain(&mut result, datetime.second(), count),
            'S' => result.push_str(&format_millis(datetime.timestamp_subsec_millis(), count)),
            'a' => result.push_str(if datetime.hour() < 12 { "AM" } else { "PM" }),
            'Z' => result.push_str("+0000"),
            other => {
                for _ in 0..count {
                    result.push(other);
                }
            }
        }
    }
    result
}

pub(crate) fn consume_same_chars(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ch: char,
) -> usize {
    let mut count = 0;
    while chars.peek().copied() == Some(ch) {
        chars.next();
        count += 1;
    }
    count
}

pub(crate) fn push_padded_or_plain(result: &mut String, value: u32, width: usize) {
    if width >= 2 {
        result.push_str(&format!("{value:0width$}"));
    } else {
        result.push_str(&value.to_string());
    }
}

pub(crate) fn format_month(month: u32, width: usize) -> String {
    const SHORT: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const LONG: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    match width {
        1 => month.to_string(),
        2 => format!("{month:02}"),
        3 => SHORT[(month - 1) as usize].to_owned(),
        _ => LONG[(month - 1) as usize].to_owned(),
    }
}

pub(crate) fn format_millis(millis: u32, width: usize) -> String {
    let value = format!("{millis:03}");
    value[..width.min(3)].to_owned()
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_time_call(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "year" => self.eval_time_component(args, TimeComponent::Year),
            "month" => self.eval_time_component(args, TimeComponent::Month),
            "weekofyear" => self.eval_time_component(args, TimeComponent::WeekOfYear),
            "dayofmonth" => self.eval_time_component(args, TimeComponent::DayOfMonth),
            "dayofweek" => self.eval_time_component(args, TimeComponent::DayOfWeek),
            "hour" => self.eval_time_component(args, TimeComponent::Hour),
            "minute" => self.eval_time_component(args, TimeComponent::Minute),
            "second" => self.eval_time_component(args, TimeComponent::Second),
            "timestamp" => self.eval_timestamp(args),
            "timeframe.in_seconds" => self.eval_timeframe_in_seconds(args),
            "timeframe.from_seconds" => self.eval_timeframe_from_seconds(args),
            "timeframe.change" => self.eval_timeframe_change(args),
            _ => return None,
        })
    }

    pub(crate) fn eval_time_component(
        &mut self,
        args: &[HirCallArg],
        component: TimeComponent,
    ) -> Result<PineValue, RuntimeError> {
        let timestamp = match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => value,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        let timezone = if let Some(arg) = args.get(1) {
            match self.eval_expr(&arg.value)? {
                PineValue::String(timezone) => timezone,
                PineValue::Na => "UTC".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            "UTC".to_owned()
        };
        if !is_supported_utc_timezone(&timezone) {
            return Err(RuntimeError {
                message: format!(
                    "{} unsupported timezone `{timezone}`",
                    component.function_name()
                ),
            });
        }
        let datetime = utc_datetime_from_millis(timestamp).map_err(|_| RuntimeError {
            message: format!(
                "{} timestamp is out of range: {timestamp}",
                component.function_name()
            ),
        })?;

        Ok(PineValue::Int(component.value(datetime)))
    }

    pub(crate) fn eval_timestamp(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(year) = self.eval_optional_timestamp_part(args, 0, 0)? else {
            return Ok(PineValue::Na);
        };
        let Some(month) = self.eval_optional_timestamp_part(args, 1, 0)? else {
            return Ok(PineValue::Na);
        };
        let Some(day) = self.eval_optional_timestamp_part(args, 2, 0)? else {
            return Ok(PineValue::Na);
        };
        let Some(hour) = self.eval_optional_timestamp_part(args, 3, 0)? else {
            return Ok(PineValue::Na);
        };
        let Some(minute) = self.eval_optional_timestamp_part(args, 4, 0)? else {
            return Ok(PineValue::Na);
        };
        let Some(second) = self.eval_optional_timestamp_part(args, 5, 0)? else {
            return Ok(PineValue::Na);
        };

        let Ok(year) = i32::try_from(year) else {
            return Err(RuntimeError {
                message: format!("timestamp year is out of range: {year}"),
            });
        };
        let Some((month, day, hour, minute, second)) =
            timestamp_unsigned_parts(month, day, hour, minute, second)
        else {
            return Err(RuntimeError {
                message: format!(
                    "timestamp invalid UTC datetime: {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
                ),
            });
        };
        let Some(datetime) = Utc
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
        else {
            return Err(RuntimeError {
                message: format!(
                    "timestamp invalid UTC datetime: {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
                ),
            });
        };

        Ok(PineValue::Int(datetime.timestamp_millis()))
    }

    pub(crate) fn eval_timeframe_in_seconds(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let timeframe = if let Some(arg) = args.first() {
            match self.eval_expr(&arg.value)? {
                PineValue::String(value) => value,
                PineValue::Na => return Ok(PineValue::Na),
                _ => return Ok(PineValue::Na),
            }
        } else {
            DEFAULT_CHART_TIMEFRAME.to_owned()
        };
        let timeframe = if timeframe.is_empty() {
            DEFAULT_CHART_TIMEFRAME
        } else {
            timeframe.trim()
        };
        let Some(seconds) = timeframe_seconds(timeframe) else {
            return Err(RuntimeError {
                message: format!("timeframe.in_seconds unsupported timeframe `{timeframe}`"),
            });
        };

        Ok(PineValue::Int(seconds))
    }

    pub(crate) fn eval_timeframe_from_seconds(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = args.first() else {
            return Ok(PineValue::Na);
        };
        let seconds = match self.eval_expr(&arg.value)? {
            PineValue::Int(value) => value,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        let Some(timeframe) = timeframe_from_seconds(seconds) else {
            return Err(RuntimeError {
                message: format!("timeframe.from_seconds unsupported seconds `{seconds}`"),
            });
        };

        Ok(PineValue::String(timeframe))
    }

    pub(crate) fn eval_timeframe_change(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = args.first() else {
            return Ok(PineValue::Na);
        };
        let timeframe = match self.eval_expr(&arg.value)? {
            PineValue::String(value) => value,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        let timeframe = if timeframe.is_empty() {
            DEFAULT_CHART_TIMEFRAME
        } else {
            timeframe.trim()
        };
        let Some(seconds) = timeframe_seconds(timeframe) else {
            return Err(RuntimeError {
                message: format!("timeframe.change unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(current_time) = self.current_builtin_i64("time") else {
            return Ok(PineValue::Na);
        };
        let Some(previous_time) = self.previous_bar_time else {
            return Ok(PineValue::Bool(true));
        };
        let Some(current_bucket) = timeframe_bucket(current_time, seconds) else {
            return Err(RuntimeError {
                message: format!("timeframe.change unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(previous_bucket) = timeframe_bucket(previous_time, seconds) else {
            return Err(RuntimeError {
                message: format!("timeframe.change unsupported timeframe `{timeframe}`"),
            });
        };

        Ok(PineValue::Bool(current_bucket != previous_bucket))
    }

    pub(crate) fn eval_optional_timestamp_part(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        default: i64,
    ) -> Result<Option<i64>, RuntimeError> {
        let Some(arg) = args.get(index) else {
            return Ok(Some(default));
        };
        let value = match self.eval_expr(&arg.value)? {
            PineValue::Int(value) => value,
            PineValue::Na => return Ok(None),
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
