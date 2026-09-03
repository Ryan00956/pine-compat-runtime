use std::collections::BTreeMap;
use std::fmt;

use crate::{Bar, RuntimeDiagnostic};

/// Official TradingView bar-magnifier lower-timeframe cap.
pub const MAX_MAGNIFIER_INTRABARS: usize = 200_000;

/// Host-owned lower-timeframe bars for one chart bar.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnifierChartBarInput {
    pub chart_bar_index: usize,
    pub bars: Vec<Bar>,
}

/// Host-owned bar-magnifier input keyed by chart bar index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MagnifierInput {
    bars_by_chart_index: BTreeMap<usize, Vec<Bar>>,
}

impl MagnifierInput {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bars_by_chart_index.is_empty()
    }

    #[must_use]
    pub fn chart_bar_count(&self) -> usize {
        self.bars_by_chart_index.len()
    }

    #[must_use]
    pub fn intrabar_count(&self) -> usize {
        self.bars_by_chart_index
            .values()
            .map(Vec::len)
            .sum::<usize>()
    }

    #[must_use]
    pub fn bars_for_chart_bar(&self, chart_bar_index: usize) -> Option<&[Bar]> {
        self.bars_by_chart_index
            .get(&chart_bar_index)
            .map(Vec::as_slice)
    }
}

/// Explicit fallback when lower-timeframe data is unavailable for a chart bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnifierFallback {
    StandardOhlc,
}

/// Tick source used by the existing strategy scheduler fill path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnifierTickSource {
    StandardOhlc,
    Intrabars,
}

/// Host ticks for one chart bar. Later fill wiring reuses `HistoricalFillStep`
/// against each tick instead of adding a second broker path.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnifierHostTicks {
    pub chart_bar_index: usize,
    pub source: MagnifierTickSource,
    pub fallback: Option<MagnifierFallback>,
    pub ticks: Vec<Bar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MagnifierInputError {
    DuplicateChartBar {
        chart_bar_index: usize,
    },
    DuplicateTicks {
        chart_bar_index: usize,
        time: i64,
    },
    UnsortedTicks {
        chart_bar_index: usize,
        previous_time: i64,
        time: i64,
    },
    TooManyIntrabars {
        count: usize,
        limit: usize,
    },
}

impl fmt::Display for MagnifierInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateChartBar { chart_bar_index } => {
                write!(
                    formatter,
                    "duplicate magnifier input for chart bar {chart_bar_index}"
                )
            }
            Self::DuplicateTicks {
                chart_bar_index,
                time,
            } => write!(
                formatter,
                "duplicate magnifier tick time `{time}` on chart bar {chart_bar_index}"
            ),
            Self::UnsortedTicks {
                chart_bar_index,
                previous_time,
                time,
            } => write!(
                formatter,
                "magnifier ticks on chart bar {chart_bar_index} are not sorted: `{time}` follows `{previous_time}`"
            ),
            Self::TooManyIntrabars { count, limit } => write!(
                formatter,
                "magnifier input has {count} lower-timeframe bars; at most {limit} are allowed"
            ),
        }
    }
}

impl MagnifierInputError {
    #[must_use]
    pub fn diagnostic(&self) -> RuntimeDiagnostic {
        let code = match self {
            Self::DuplicateChartBar { .. } => "E_MAGNIFIER_DUPLICATE_CHART_BAR",
            Self::DuplicateTicks { .. } => "E_MAGNIFIER_DUPLICATE_TICK",
            Self::UnsortedTicks { .. } => "E_MAGNIFIER_UNSORTED_TICKS",
            Self::TooManyIntrabars { .. } => "E_MAGNIFIER_MAX_INTRABARS",
        };
        RuntimeDiagnostic {
            code: code.to_owned(),
            message: self.to_string(),
        }
    }
}

/// Build and validate host-owned magnifier input. Invalid host data fails closed.
pub fn magnifier_input_from_groups(
    groups: Vec<MagnifierChartBarInput>,
) -> Result<MagnifierInput, MagnifierInputError> {
    let mut input = MagnifierInput::new();
    let mut total = 0usize;
    for group in groups {
        if input
            .bars_by_chart_index
            .contains_key(&group.chart_bar_index)
        {
            return Err(MagnifierInputError::DuplicateChartBar {
                chart_bar_index: group.chart_bar_index,
            });
        }
        validate_intrabar_times(group.chart_bar_index, &group.bars)?;
        total = total.saturating_add(group.bars.len());
        if total > MAX_MAGNIFIER_INTRABARS {
            return Err(MagnifierInputError::TooManyIntrabars {
                count: total,
                limit: MAX_MAGNIFIER_INTRABARS,
            });
        }
        input
            .bars_by_chart_index
            .insert(group.chart_bar_index, group.bars);
    }
    Ok(input)
}

fn validate_intrabar_times(
    chart_bar_index: usize,
    bars: &[Bar],
) -> Result<(), MagnifierInputError> {
    let mut previous_time = None;
    for bar in bars {
        if let Some(previous_time) = previous_time {
            if bar.time == previous_time {
                return Err(MagnifierInputError::DuplicateTicks {
                    chart_bar_index,
                    time: bar.time,
                });
            }
            if bar.time < previous_time {
                return Err(MagnifierInputError::UnsortedTicks {
                    chart_bar_index,
                    previous_time,
                    time: bar.time,
                });
            }
        }
        previous_time = Some(bar.time);
    }
    Ok(())
}

#[must_use]
pub fn magnifier_absence_diagnostic(chart_bar_index: usize) -> RuntimeDiagnostic {
    RuntimeDiagnostic {
        code: "W_MAGNIFIER_FALLBACK".to_owned(),
        message: format!(
            "magnifier data absent for chart bar {chart_bar_index}; using standard OHLC path"
        ),
    }
}

#[must_use]
pub fn magnifier_gap_diagnostic(chart_bar_index: usize) -> RuntimeDiagnostic {
    RuntimeDiagnostic {
        code: "W_MAGNIFIER_GAP".to_owned(),
        message: format!(
            "magnifier data has a gap at chart bar {chart_bar_index}; using standard OHLC path"
        ),
    }
}

/// Plan host ticks for one chart bar. Missing or empty lower-timeframe data
/// falls back to the chart bar's standard OHLC path.
#[must_use]
pub fn magnifier_host_ticks(
    chart_bar_index: usize,
    chart_bar: Bar,
    input: &MagnifierInput,
) -> MagnifierHostTicks {
    match input.bars_for_chart_bar(chart_bar_index) {
        Some(bars) if !bars.is_empty() => MagnifierHostTicks {
            chart_bar_index,
            source: MagnifierTickSource::Intrabars,
            fallback: None,
            ticks: bars.to_vec(),
        },
        _ => MagnifierHostTicks {
            chart_bar_index,
            source: MagnifierTickSource::StandardOhlc,
            fallback: Some(MagnifierFallback::StandardOhlc),
            ticks: vec![chart_bar],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar_at(time: i64, close: f64) -> Bar {
        Bar {
            time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }

    #[test]
    fn accepts_ordered_intrabars_keyed_by_chart_bar() {
        let input = magnifier_input_from_groups(vec![
            MagnifierChartBarInput {
                chart_bar_index: 0,
                bars: vec![bar_at(1, 1.0), bar_at(2, 1.5)],
            },
            MagnifierChartBarInput {
                chart_bar_index: 1,
                bars: vec![bar_at(3, 2.0)],
            },
        ])
        .expect("valid input");
        assert_eq!(input.chart_bar_count(), 2);
        assert_eq!(input.intrabar_count(), 3);
        let ticks = magnifier_host_ticks(0, bar_at(10, 1.0), &input);
        assert_eq!(ticks.source, MagnifierTickSource::Intrabars);
        assert_eq!(ticks.fallback, None);
        assert_eq!(ticks.ticks.len(), 2);
    }

    #[test]
    fn absence_falls_back_to_standard_ohlc_path() {
        let input = MagnifierInput::new();
        let chart_bar = bar_at(10, 4.0);
        let ticks = magnifier_host_ticks(2, chart_bar, &input);
        assert_eq!(ticks.source, MagnifierTickSource::StandardOhlc);
        assert_eq!(ticks.fallback, Some(MagnifierFallback::StandardOhlc));
        assert_eq!(ticks.ticks, vec![chart_bar]);
        let diagnostic = magnifier_absence_diagnostic(2);
        assert_eq!(diagnostic.code, "W_MAGNIFIER_FALLBACK");
        assert!(diagnostic.message.contains("chart bar 2"));
        assert!(diagnostic.message.contains("standard OHLC path"));
    }

    #[test]
    fn gap_falls_back_and_reports_gap_diagnostic() {
        let input = magnifier_input_from_groups(vec![
            MagnifierChartBarInput {
                chart_bar_index: 0,
                bars: vec![bar_at(1, 1.0)],
            },
            MagnifierChartBarInput {
                chart_bar_index: 2,
                bars: vec![bar_at(3, 3.0)],
            },
        ])
        .expect("valid input");
        let ticks = magnifier_host_ticks(1, bar_at(2, 2.0), &input);
        assert_eq!(ticks.fallback, Some(MagnifierFallback::StandardOhlc));
        let diagnostic = magnifier_gap_diagnostic(1);
        assert_eq!(diagnostic.code, "W_MAGNIFIER_GAP");
        assert!(diagnostic.message.contains("chart bar 1"));
    }

    #[test]
    fn rejects_duplicate_chart_bar_keys() {
        let error = magnifier_input_from_groups(vec![
            MagnifierChartBarInput {
                chart_bar_index: 0,
                bars: vec![bar_at(1, 1.0)],
            },
            MagnifierChartBarInput {
                chart_bar_index: 0,
                bars: vec![bar_at(2, 2.0)],
            },
        ])
        .expect_err("duplicate chart bar");
        assert!(matches!(
            error,
            MagnifierInputError::DuplicateChartBar { chart_bar_index: 0 }
        ));
        assert_eq!(error.diagnostic().code, "E_MAGNIFIER_DUPLICATE_CHART_BAR");
    }

    #[test]
    fn rejects_duplicate_tick_timestamps() {
        let error = magnifier_input_from_groups(vec![MagnifierChartBarInput {
            chart_bar_index: 1,
            bars: vec![bar_at(5, 1.0), bar_at(5, 2.0)],
        }])
        .expect_err("duplicate ticks");
        assert!(matches!(
            error,
            MagnifierInputError::DuplicateTicks {
                chart_bar_index: 1,
                time: 5
            }
        ));
        assert_eq!(error.diagnostic().code, "E_MAGNIFIER_DUPLICATE_TICK");
    }

    #[test]
    fn rejects_unsorted_tick_timestamps() {
        let error = magnifier_input_from_groups(vec![MagnifierChartBarInput {
            chart_bar_index: 0,
            bars: vec![bar_at(4, 1.0), bar_at(3, 2.0)],
        }])
        .expect_err("unsorted ticks");
        assert!(matches!(
            error,
            MagnifierInputError::UnsortedTicks {
                chart_bar_index: 0,
                previous_time: 4,
                time: 3
            }
        ));
        assert_eq!(error.diagnostic().code, "E_MAGNIFIER_UNSORTED_TICKS");
    }

    #[test]
    fn host_ticks_are_chart_bars_consumed_by_the_existing_fill_path() {
        use crate::runtime::strategy_scheduler::HistoricalFillStep;
        let input = magnifier_input_from_groups(vec![MagnifierChartBarInput {
            chart_bar_index: 0,
            bars: vec![bar_at(1, 1.0), bar_at(2, 1.5), bar_at(3, 2.0)],
        }])
        .expect("valid input");
        let ticks = magnifier_host_ticks(0, bar_at(10, 2.0), &input);
        assert_eq!(ticks.ticks.len(), 3);
        let mut steps = [
            HistoricalFillStep::StopLong,
            HistoricalFillStep::MarketEntriesAtOpen,
            HistoricalFillStep::LimitLong,
        ];
        steps.sort_by_key(|step| step.ordering_key());
        assert_eq!(steps[0], HistoricalFillStep::MarketEntriesAtOpen);
        assert_eq!(steps[1], HistoricalFillStep::LimitLong);
        assert_eq!(steps[2], HistoricalFillStep::StopLong);
    }

    #[test]
    fn rejects_intrabar_count_above_official_limit() {
        let mut bars = Vec::with_capacity(MAX_MAGNIFIER_INTRABARS + 1);
        for index in 0..=MAX_MAGNIFIER_INTRABARS {
            bars.push(bar_at(index as i64, 1.0));
        }
        let error = magnifier_input_from_groups(vec![MagnifierChartBarInput {
            chart_bar_index: 0,
            bars,
        }])
        .expect_err("too many intrabars");
        assert!(matches!(
            error,
            MagnifierInputError::TooManyIntrabars {
                count,
                limit: MAX_MAGNIFIER_INTRABARS
            } if count == MAX_MAGNIFIER_INTRABARS + 1
        ));
        assert_eq!(error.diagnostic().code, "E_MAGNIFIER_MAX_INTRABARS");
    }
}
