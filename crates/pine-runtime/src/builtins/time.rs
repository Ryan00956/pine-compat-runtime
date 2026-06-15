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

struct BarTimeFunctionArgs {
    timeframe: String,
    session: Option<String>,
    timezone: String,
    bars_back: i64,
    timeframe_bars_back: i64,
}

impl Default for BarTimeFunctionArgs {
    fn default() -> Self {
        Self {
            timeframe: String::new(),
            session: None,
            timezone: "UTC".to_owned(),
            bars_back: 0,
            timeframe_bars_back: 0,
        }
    }
}

struct TimeSession {
    periods: Vec<TimeSessionPeriod>,
    days: [bool; 8],
}

struct TimeSessionPeriod {
    start_minute: i64,
    end_minute: i64,
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
            "time" => self.eval_bar_time_function(args, false),
            "time_close" => self.eval_bar_time_function(args, true),
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

    pub(crate) fn eval_bar_time_function(
        &mut self,
        args: &[HirCallArg],
        close_time: bool,
    ) -> Result<PineValue, RuntimeError> {
        let name = if close_time { "time_close" } else { "time" };
        let Some(args) = self.eval_bar_time_function_args(args)? else {
            return Ok(PineValue::Na);
        };
        if args.bars_back < -500 {
            return Err(RuntimeError {
                message: format!("{name} bars_back cannot reference more than 500 future bars"),
            });
        }
        if args.timeframe_bars_back < -500 {
            return Err(RuntimeError {
                message: format!(
                    "{name} timeframe_bars_back cannot reference more than 500 future bars"
                ),
            });
        }
        if !is_supported_utc_timezone(&args.timezone) {
            return Err(RuntimeError {
                message: format!("{name} unsupported timezone `{}`", args.timezone),
            });
        }
        let session = match args.session.as_deref() {
            Some("") | None => None,
            Some(session) => Some(parse_time_session(session).ok_or_else(|| RuntimeError {
                message: format!("{name} unsupported session `{session}`"),
            })?),
        };
        let timeframe = if args.timeframe.is_empty() {
            DEFAULT_CHART_TIMEFRAME
        } else {
            args.timeframe.trim()
        };
        let Some(seconds) = timeframe_seconds(timeframe) else {
            return Err(RuntimeError {
                message: format!("{name} unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(chart_seconds) = timeframe_seconds(DEFAULT_CHART_TIMEFRAME) else {
            return Err(RuntimeError {
                message: format!("unsupported default chart timeframe `{DEFAULT_CHART_TIMEFRAME}`"),
            });
        };
        if seconds < chart_seconds {
            return Err(RuntimeError {
                message: format!("{name} unsupported lower timeframe `{timeframe}`"),
            });
        }
        if session.is_none()
            && seconds == chart_seconds
            && args.bars_back == 0
            && args.timeframe_bars_back == 0
        {
            return Ok(self
                .current_builtin_i64(name)
                .map(PineValue::Int)
                .unwrap_or(PineValue::Na));
        }

        let Some(current_time) = self.current_builtin_i64("time") else {
            return Ok(PineValue::Na);
        };
        let Some(chart_duration_ms) = chart_seconds.checked_mul(1000) else {
            return Err(RuntimeError {
                message: format!("{name} unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(offset_ms) = args.bars_back.checked_mul(chart_duration_ms) else {
            return Err(RuntimeError {
                message: format!("{name} bars_back timestamp is out of range"),
            });
        };
        let Some(base_time) = current_time.checked_sub(offset_ms) else {
            return Err(RuntimeError {
                message: format!("{name} bars_back timestamp is out of range"),
            });
        };
        let Some(duration_ms) = seconds.checked_mul(1000) else {
            return Err(RuntimeError {
                message: format!("{name} unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(bucket) = timeframe_bucket(base_time, seconds) else {
            return Err(RuntimeError {
                message: format!("{name} unsupported timeframe `{timeframe}`"),
            });
        };
        let Some(bucket) = bucket.checked_sub(args.timeframe_bars_back) else {
            return Err(RuntimeError {
                message: format!("{name} timeframe_bars_back timestamp is out of range"),
            });
        };
        let Some(open_time) = bucket.checked_mul(duration_ms) else {
            return Err(RuntimeError {
                message: format!("{name} timestamp is out of range for timeframe `{timeframe}`"),
            });
        };
        let Some(close_timestamp) = bucket
            .checked_add(1)
            .and_then(|value| value.checked_mul(duration_ms))
        else {
            return Err(RuntimeError {
                message: format!("{name} timestamp is out of range for timeframe `{timeframe}`"),
            });
        };
        if let Some(session) = session {
            let Some(session_close) =
                session_close_for_bar_open(open_time, close_timestamp, &session)?
            else {
                return Ok(PineValue::Na);
            };
            return Ok(PineValue::Int(if close_time {
                session_close
            } else {
                open_time
            }));
        }

        Ok(PineValue::Int(if close_time {
            close_timestamp
        } else {
            open_time
        }))
    }

    fn eval_bar_time_function_args(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<Option<BarTimeFunctionArgs>, RuntimeError> {
        let mut parsed = BarTimeFunctionArgs::default();
        let mut positional = Vec::new();
        let mut saw_timeframe = false;
        for arg in args {
            if let Some(name) = arg.name.as_deref() {
                match name {
                    "timeframe" => {
                        let Some(value) = self.eval_time_function_string_arg(arg)? else {
                            return Ok(None);
                        };
                        parsed.timeframe = value;
                        saw_timeframe = true;
                    }
                    "session" => {
                        let Some(value) = self.eval_time_function_string_arg(arg)? else {
                            return Ok(None);
                        };
                        parsed.session = Some(value);
                    }
                    "timezone" => {
                        let Some(value) = self.eval_time_function_string_arg(arg)? else {
                            return Ok(None);
                        };
                        parsed.timezone = value;
                    }
                    "bars_back" => {
                        let Some(value) = self.eval_time_function_int_arg(arg)? else {
                            return Ok(None);
                        };
                        parsed.bars_back = value;
                    }
                    "timeframe_bars_back" => {
                        let Some(value) = self.eval_time_function_int_arg(arg)? else {
                            return Ok(None);
                        };
                        parsed.timeframe_bars_back = value;
                    }
                    _ => {}
                }
            } else {
                positional.push(arg);
            }
        }

        if let Some(timeframe_arg) = positional.first() {
            let Some(timeframe) = self.eval_time_function_string_arg(timeframe_arg)? else {
                return Ok(None);
            };
            parsed.timeframe = timeframe;
            saw_timeframe = true;
        }
        if !saw_timeframe {
            return Ok(None);
        }

        let mut second_is_bars_back = false;
        let mut third_is_timezone = false;
        if let Some(arg) = positional.get(1) {
            match self.eval_expr(&arg.value)? {
                PineValue::String(value) => parsed.session = Some(value),
                PineValue::Int(value) => {
                    parsed.bars_back = value;
                    second_is_bars_back = true;
                }
                PineValue::Na => return Ok(None),
                _ => return Ok(None),
            }
        }
        if let Some(arg) = positional.get(2) {
            if second_is_bars_back {
                let Some(value) = self.eval_time_function_int_arg(arg)? else {
                    return Ok(None);
                };
                parsed.timeframe_bars_back = value;
            } else {
                match self.eval_expr(&arg.value)? {
                    PineValue::String(value) => {
                        parsed.timezone = value;
                        third_is_timezone = true;
                    }
                    PineValue::Int(value) => parsed.bars_back = value,
                    PineValue::Na => return Ok(None),
                    _ => return Ok(None),
                }
            }
        }
        if let Some(arg) = positional.get(3) {
            let Some(value) = self.eval_time_function_int_arg(arg)? else {
                return Ok(None);
            };
            if third_is_timezone {
                parsed.bars_back = value;
            } else {
                parsed.timeframe_bars_back = value;
            }
        }
        if let Some(arg) = positional.get(4) {
            let Some(value) = self.eval_time_function_int_arg(arg)? else {
                return Ok(None);
            };
            parsed.timeframe_bars_back = value;
        }

        Ok(Some(parsed))
    }

    fn eval_time_function_string_arg(
        &mut self,
        arg: &HirCallArg,
    ) -> Result<Option<String>, RuntimeError> {
        Ok(match self.eval_expr(&arg.value)? {
            PineValue::String(value) => Some(value),
            PineValue::Na => None,
            _ => None,
        })
    }

    fn eval_time_function_int_arg(
        &mut self,
        arg: &HirCallArg,
    ) -> Result<Option<i64>, RuntimeError> {
        Ok(match self.eval_expr(&arg.value)? {
            PineValue::Int(value) => Some(value),
            PineValue::Na => None,
            _ => None,
        })
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

fn parse_time_session(session: &str) -> Option<TimeSession> {
    if session == "24x7" {
        return Some(TimeSession {
            periods: vec![TimeSessionPeriod {
                start_minute: 0,
                end_minute: 0,
            }],
            days: all_session_days(),
        });
    }

    let (periods, days) = match session.split_once(':') {
        Some((periods, days)) => (periods, parse_session_days(days)?),
        None => (session, all_session_days()),
    };
    if periods.is_empty() {
        return None;
    }
    let periods = periods
        .split(',')
        .map(parse_session_period)
        .collect::<Option<Vec<_>>>()?;
    if periods.is_empty() {
        return None;
    }

    Some(TimeSession { periods, days })
}

fn all_session_days() -> [bool; 8] {
    [false, true, true, true, true, true, true, true]
}

fn parse_session_days(days: &str) -> Option<[bool; 8]> {
    if days.is_empty() {
        return None;
    }
    let mut parsed = [false; 8];
    for ch in days.chars() {
        let day = ch.to_digit(10)?;
        if !(1..=7).contains(&day) {
            return None;
        }
        parsed[day as usize] = true;
    }
    Some(parsed)
}

fn parse_session_period(period: &str) -> Option<TimeSessionPeriod> {
    let (start, end) = period.split_once('-')?;
    Some(TimeSessionPeriod {
        start_minute: parse_session_hhmm(start)?,
        end_minute: parse_session_hhmm(end)?,
    })
}

fn parse_session_hhmm(value: &str) -> Option<i64> {
    if value.len() != 4 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let hour = value[..2].parse::<i64>().ok()?;
    let minute = value[2..].parse::<i64>().ok()?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        return None;
    }
    Some(hour * 60 + minute)
}

fn session_close_for_bar_open(
    open_time: i64,
    default_close: i64,
    session: &TimeSession,
) -> Result<Option<i64>, RuntimeError> {
    let datetime = utc_datetime_from_millis(open_time)?;
    let minute = i64::from(datetime.hour()) * 60 + i64::from(datetime.minute());
    let day = dayofweek_value(datetime) as usize;
    let midnight = utc_midnight_millis(datetime)?;

    for period in &session.periods {
        let Some(close_time) =
            session_period_close_for_open(minute, day, midnight, period, &session.days)?
        else {
            continue;
        };
        return Ok(Some(default_close.min(close_time)));
    }

    Ok(None)
}

fn session_period_close_for_open(
    minute: i64,
    day: usize,
    midnight: i64,
    period: &TimeSessionPeriod,
    days: &[bool; 8],
) -> Result<Option<i64>, RuntimeError> {
    const DAY_MS: i64 = 86_400_000;

    if period.start_minute == period.end_minute {
        if days[day] {
            return Ok(Some(midnight.checked_add(DAY_MS).ok_or_else(|| {
                RuntimeError {
                    message: "time session timestamp is out of range".to_owned(),
                }
            })?));
        }
        return Ok(None);
    }

    if period.start_minute < period.end_minute {
        if days[day] && minute >= period.start_minute && minute < period.end_minute {
            return Ok(Some(session_end_timestamp(midnight, period.end_minute)?));
        }
        return Ok(None);
    }

    if minute >= period.start_minute {
        let session_day = if day == 7 { 1 } else { day + 1 };
        if !days[session_day] {
            return Ok(None);
        }
        let next_midnight = midnight.checked_add(DAY_MS).ok_or_else(|| RuntimeError {
            message: "time session timestamp is out of range".to_owned(),
        })?;
        return Ok(Some(session_end_timestamp(
            next_midnight,
            period.end_minute,
        )?));
    }

    if minute < period.end_minute && days[day] {
        return Ok(Some(session_end_timestamp(midnight, period.end_minute)?));
    }

    Ok(None)
}

fn session_end_timestamp(midnight: i64, end_minute: i64) -> Result<i64, RuntimeError> {
    midnight
        .checked_add(end_minute.checked_mul(60_000).ok_or_else(|| RuntimeError {
            message: "time session timestamp is out of range".to_owned(),
        })?)
        .ok_or_else(|| RuntimeError {
            message: "time session timestamp is out of range".to_owned(),
        })
}

fn utc_midnight_millis(datetime: DateTime<Utc>) -> Result<i64, RuntimeError> {
    let Some(midnight) = Utc
        .with_ymd_and_hms(datetime.year(), datetime.month(), datetime.day(), 0, 0, 0)
        .single()
    else {
        return Err(RuntimeError {
            message: "time session timestamp is out of range".to_owned(),
        });
    };
    Ok(midnight.timestamp_millis())
}
