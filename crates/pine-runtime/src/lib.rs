//! Historical runtime scaffolding.

use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirLiteral,
    HirProgram, HirStmt, HirStmtKind, HirUnaryOp, SeriesId, SymbolId, VarSlotId,
};
use regex::Regex;

mod bar;
mod error;
mod retention;
mod series;
mod value;

pub use bar::{Bar, BarUpdate, BarUpdateKind};
pub use error::RuntimeError;
pub use retention::HistoryRetentionMode;
pub use series::SeriesStore;
pub use value::PineValue;

use retention::SeriesRetention;

const MAX_WHILE_ITERATIONS: usize = 100_000;
const MAX_ARRAY_ELEMENTS: usize = 100_000;
const MAX_STRING_CHARS: usize = 40_960;
const MAX_SERIES_HISTORY_VALUES: usize = 1_000_000;
const DEFAULT_CHART_TIMEFRAME: &str = "1";

pub const PUBLIC_OUTPUT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeResult {
    pub plots: Vec<PlotSeries>,
    pub plot_chars: Vec<PlotCharSeries>,
    pub plot_shapes: Vec<PlotShapeSeries>,
    pub plot_arrows: Vec<PlotArrowSeries>,
    pub plot_bars: Vec<PlotBarSeries>,
    pub plot_candles: Vec<PlotCandleSeries>,
    pub bg_colors: Vec<ColorSeries>,
    pub bar_colors: Vec<ColorSeries>,
    pub hlines: Vec<HLineOutput>,
    pub fills: Vec<FillOutput>,
    pub diagnostics: Vec<RuntimeDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeProfiledResult {
    pub result: RuntimeResult,
    pub profile: RuntimeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProfile {
    pub bars: usize,
    pub series_buffers: usize,
    pub series_values: usize,
    pub series_capacity: usize,
    pub max_series_depth: usize,
    pub history_retention_mode: HistoryRetentionMode,
    pub history_max_constant_offset: u32,
    pub history_max_bars_back: Option<u32>,
    pub history_has_dynamic_offsets: bool,
    pub symbol_slots: usize,
    pub symbol_capacity: usize,
    pub current_series_slots: usize,
    pub current_series_capacity: usize,
    pub var_slots: usize,
    pub var_capacity: usize,
    pub array_slots: usize,
    pub array_capacity: usize,
    pub array_values: usize,
    pub array_value_capacity: usize,
    pub call_state_slots: usize,
    pub call_state_capacity: usize,
    pub valuewhen_state_slots: usize,
    pub valuewhen_state_capacity: usize,
    pub valuewhen_state_values: usize,
    pub valuewhen_state_value_capacity: usize,
    pub rolling_window_slots: usize,
    pub rolling_window_capacity: usize,
    pub rolling_window_values: usize,
    pub rolling_window_value_capacity: usize,
    pub rsi_state_slots: usize,
    pub rsi_state_capacity: usize,
    pub macd_state_slots: usize,
    pub macd_state_capacity: usize,
    pub plots: usize,
    pub plot_values: usize,
    pub plot_capacity: usize,
    pub plot_chars: usize,
    pub plot_char_values: usize,
    pub plot_char_capacity: usize,
    pub plot_shapes: usize,
    pub plot_shape_values: usize,
    pub plot_shape_capacity: usize,
    pub plot_arrows: usize,
    pub plot_arrow_values: usize,
    pub plot_arrow_capacity: usize,
    pub plot_bars: usize,
    pub plot_bar_values: usize,
    pub plot_bar_capacity: usize,
    pub plot_candles: usize,
    pub plot_candle_values: usize,
    pub plot_candle_capacity: usize,
    pub bg_colors: usize,
    pub bg_color_values: usize,
    pub bg_color_capacity: usize,
    pub bar_colors: usize,
    pub bar_color_values: usize,
    pub bar_color_capacity: usize,
    pub hlines: usize,
    pub hline_capacity: usize,
    pub fills: usize,
    pub fill_capacity: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotSeries {
    pub id: u32,
    pub values: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorSeries {
    pub id: u32,
    pub values: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotCharSeries {
    pub id: u32,
    pub values: Vec<PineValue>,
    pub chars: Vec<PineValue>,
    pub colors: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotShapeSeries {
    pub id: u32,
    pub values: Vec<PineValue>,
    pub styles: Vec<PineValue>,
    pub locations: Vec<PineValue>,
    pub colors: Vec<PineValue>,
    pub texts: Vec<PineValue>,
    pub text_colors: Vec<PineValue>,
    pub sizes: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotArrowSeries {
    pub id: u32,
    pub values: Vec<PineValue>,
    pub color_ups: Vec<PineValue>,
    pub color_downs: Vec<PineValue>,
    pub min_heights: Vec<PineValue>,
    pub max_heights: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotBarSeries {
    pub id: u32,
    pub opens: Vec<PineValue>,
    pub highs: Vec<PineValue>,
    pub lows: Vec<PineValue>,
    pub closes: Vec<PineValue>,
    pub colors: Vec<PineValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotCandleSeries {
    pub id: u32,
    pub opens: Vec<PineValue>,
    pub highs: Vec<PineValue>,
    pub lows: Vec<PineValue>,
    pub closes: Vec<PineValue>,
    pub colors: Vec<PineValue>,
    pub wick_colors: Vec<PineValue>,
    pub border_colors: Vec<PineValue>,
}

trait SeriesOutput: Sized {
    fn new(id: u32, values: Vec<PineValue>) -> Self;
    fn id(&self) -> u32;
    fn values_mut(&mut self) -> &mut Vec<PineValue>;
}

impl SeriesOutput for PlotSeries {
    fn new(id: u32, values: Vec<PineValue>) -> Self {
        Self { id, values }
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn values_mut(&mut self) -> &mut Vec<PineValue> {
        &mut self.values
    }
}

impl SeriesOutput for ColorSeries {
    fn new(id: u32, values: Vec<PineValue>) -> Self {
        Self { id, values }
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn values_mut(&mut self) -> &mut Vec<PineValue> {
        &mut self.values
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HLineOutput {
    pub id: u32,
    pub price: PineValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillOutput {
    pub id: u32,
    pub first_id: u32,
    pub second_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDiagnostic {
    pub code: String,
    pub message: String,
}

pub fn public_runtime_result_json(result: &RuntimeResult) -> String {
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_OUTPUT_SCHEMA_VERSION);
    output.push_str("\"plots\":");
    output.push_str(&plots_json(&result.plots));
    output.push_str(",\"plotChars\":");
    output.push_str(&plot_chars_json(&result.plot_chars));
    output.push_str(",\"plotShapes\":");
    output.push_str(&plot_shapes_json(&result.plot_shapes));
    output.push_str(",\"plotArrows\":");
    output.push_str(&plot_arrows_json(&result.plot_arrows));
    output.push_str(",\"plotBars\":");
    output.push_str(&plot_bars_json(&result.plot_bars));
    output.push_str(",\"plotCandles\":");
    output.push_str(&plot_candles_json(&result.plot_candles));
    output.push_str(",\"bgColors\":");
    output.push_str(&colors_json(&result.bg_colors));
    output.push_str(",\"barColors\":");
    output.push_str(&colors_json(&result.bar_colors));
    output.push_str(",\"hlines\":");
    output.push_str(&hlines_json(&result.hlines));
    output.push_str(",\"fills\":");
    output.push_str(&fills_json(&result.fills));
    output.push_str(",\"diagnostics\":[]");
    output.push('}');
    output
}

pub fn public_runtime_profiled_result_json(
    result: &RuntimeResult,
    profile: &RuntimeProfile,
) -> String {
    let mut output = public_runtime_result_json(result);
    output.pop();
    output.push_str(",\"profile\":");
    output.push_str(&profile_json(profile));
    output.push('}');
    output
}

fn profile_json(profile: &RuntimeProfile) -> String {
    format!(
        concat!(
            "{{",
            "\"bars\":{},",
            "\"seriesBuffers\":{},",
            "\"seriesValues\":{},",
            "\"seriesCapacity\":{},",
            "\"maxSeriesDepth\":{},",
            "\"historyRetentionMode\":\"{}\",",
            "\"historyMaxConstantOffset\":{},",
            "\"historyMaxBarsBack\":{},",
            "\"historyHasDynamicOffsets\":{},",
            "\"symbolSlots\":{},",
            "\"symbolCapacity\":{},",
            "\"currentSeriesSlots\":{},",
            "\"currentSeriesCapacity\":{},",
            "\"varSlots\":{},",
            "\"varCapacity\":{},",
            "\"arraySlots\":{},",
            "\"arrayCapacity\":{},",
            "\"arrayValues\":{},",
            "\"arrayValueCapacity\":{},",
            "\"callStateSlots\":{},",
            "\"callStateCapacity\":{},",
            "\"valuewhenStateSlots\":{},",
            "\"valuewhenStateCapacity\":{},",
            "\"valuewhenStateValues\":{},",
            "\"valuewhenStateValueCapacity\":{},",
            "\"rollingWindowSlots\":{},",
            "\"rollingWindowCapacity\":{},",
            "\"rollingWindowValues\":{},",
            "\"rollingWindowValueCapacity\":{},",
            "\"rsiStateSlots\":{},",
            "\"rsiStateCapacity\":{},",
            "\"macdStateSlots\":{},",
            "\"macdStateCapacity\":{},",
            "\"plots\":{},",
            "\"plotValues\":{},",
            "\"plotCapacity\":{},",
            "\"plotChars\":{},",
            "\"plotCharValues\":{},",
            "\"plotCharCapacity\":{},",
            "\"plotShapes\":{},",
            "\"plotShapeValues\":{},",
            "\"plotShapeCapacity\":{},",
            "\"plotArrows\":{},",
            "\"plotArrowValues\":{},",
            "\"plotArrowCapacity\":{},",
            "\"plotBars\":{},",
            "\"plotBarValues\":{},",
            "\"plotBarCapacity\":{},",
            "\"plotCandles\":{},",
            "\"plotCandleValues\":{},",
            "\"plotCandleCapacity\":{},",
            "\"bgColors\":{},",
            "\"bgColorValues\":{},",
            "\"bgColorCapacity\":{},",
            "\"barColors\":{},",
            "\"barColorValues\":{},",
            "\"barColorCapacity\":{},",
            "\"hlines\":{},",
            "\"hlineCapacity\":{},",
            "\"fills\":{},",
            "\"fillCapacity\":{}",
            "}}"
        ),
        profile.bars,
        profile.series_buffers,
        profile.series_values,
        profile.series_capacity,
        profile.max_series_depth,
        history_retention_mode_json(profile.history_retention_mode),
        profile.history_max_constant_offset,
        option_u32_json(profile.history_max_bars_back),
        profile.history_has_dynamic_offsets,
        profile.symbol_slots,
        profile.symbol_capacity,
        profile.current_series_slots,
        profile.current_series_capacity,
        profile.var_slots,
        profile.var_capacity,
        profile.array_slots,
        profile.array_capacity,
        profile.array_values,
        profile.array_value_capacity,
        profile.call_state_slots,
        profile.call_state_capacity,
        profile.valuewhen_state_slots,
        profile.valuewhen_state_capacity,
        profile.valuewhen_state_values,
        profile.valuewhen_state_value_capacity,
        profile.rolling_window_slots,
        profile.rolling_window_capacity,
        profile.rolling_window_values,
        profile.rolling_window_value_capacity,
        profile.rsi_state_slots,
        profile.rsi_state_capacity,
        profile.macd_state_slots,
        profile.macd_state_capacity,
        profile.plots,
        profile.plot_values,
        profile.plot_capacity,
        profile.plot_chars,
        profile.plot_char_values,
        profile.plot_char_capacity,
        profile.plot_shapes,
        profile.plot_shape_values,
        profile.plot_shape_capacity,
        profile.plot_arrows,
        profile.plot_arrow_values,
        profile.plot_arrow_capacity,
        profile.plot_bars,
        profile.plot_bar_values,
        profile.plot_bar_capacity,
        profile.plot_candles,
        profile.plot_candle_values,
        profile.plot_candle_capacity,
        profile.bg_colors,
        profile.bg_color_values,
        profile.bg_color_capacity,
        profile.bar_colors,
        profile.bar_color_values,
        profile.bar_color_capacity,
        profile.hlines,
        profile.hline_capacity,
        profile.fills,
        profile.fill_capacity
    )
}

fn history_retention_mode_json(mode: HistoryRetentionMode) -> &'static str {
    match mode {
        HistoryRetentionMode::StaticTrimmed => "staticTrimmed",
        HistoryRetentionMode::DynamicFull => "dynamicFull",
        HistoryRetentionMode::MaxBarsBack => "maxBarsBack",
    }
}

fn option_u32_json(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn plots_json(plots: &[PlotSeries]) -> String {
    let mut output = String::from("[");
    for (plot_index, plot) in plots.iter().enumerate() {
        if plot_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot.id));
        values_json_into(&mut output, &plot.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn colors_json(colors: &[ColorSeries]) -> String {
    let mut output = String::from("[");
    for (color_index, colors) in colors.iter().enumerate() {
        if color_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", colors.id));
        values_json_into(&mut output, &colors.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_chars_json(plot_chars: &[PlotCharSeries]) -> String {
    let mut output = String::from("[");
    for (plot_char_index, plot_char) in plot_chars.iter().enumerate() {
        if plot_char_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_char.id));
        values_json_into(&mut output, &plot_char.values);
        output.push_str("],\"chars\":[");
        values_json_into(&mut output, &plot_char.chars);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_char.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_shapes_json(plot_shapes: &[PlotShapeSeries]) -> String {
    let mut output = String::from("[");
    for (plot_shape_index, plot_shape) in plot_shapes.iter().enumerate() {
        if plot_shape_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_shape.id));
        values_json_into(&mut output, &plot_shape.values);
        output.push_str("],\"styles\":[");
        values_json_into(&mut output, &plot_shape.styles);
        output.push_str("],\"locations\":[");
        values_json_into(&mut output, &plot_shape.locations);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_shape.colors);
        output.push_str("],\"texts\":[");
        values_json_into(&mut output, &plot_shape.texts);
        output.push_str("],\"textColors\":[");
        values_json_into(&mut output, &plot_shape.text_colors);
        output.push_str("],\"sizes\":[");
        values_json_into(&mut output, &plot_shape.sizes);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_arrows_json(plot_arrows: &[PlotArrowSeries]) -> String {
    let mut output = String::from("[");
    for (plot_arrow_index, plot_arrow) in plot_arrows.iter().enumerate() {
        if plot_arrow_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_arrow.id));
        values_json_into(&mut output, &plot_arrow.values);
        output.push_str("],\"colorUps\":[");
        values_json_into(&mut output, &plot_arrow.color_ups);
        output.push_str("],\"colorDowns\":[");
        values_json_into(&mut output, &plot_arrow.color_downs);
        output.push_str("],\"minHeights\":[");
        values_json_into(&mut output, &plot_arrow.min_heights);
        output.push_str("],\"maxHeights\":[");
        values_json_into(&mut output, &plot_arrow.max_heights);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_bars_json(plot_bars: &[PlotBarSeries]) -> String {
    let mut output = String::from("[");
    for (plot_bar_index, plot_bar) in plot_bars.iter().enumerate() {
        if plot_bar_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_bar.id));
        values_json_into(&mut output, &plot_bar.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_bar.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_bar.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_bar.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_bar.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_candles_json(plot_candles: &[PlotCandleSeries]) -> String {
    let mut output = String::from("[");
    for (plot_candle_index, plot_candle) in plot_candles.iter().enumerate() {
        if plot_candle_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_candle.id));
        values_json_into(&mut output, &plot_candle.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_candle.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_candle.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_candle.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_candle.colors);
        output.push_str("],\"wickColors\":[");
        values_json_into(&mut output, &plot_candle.wick_colors);
        output.push_str("],\"borderColors\":[");
        values_json_into(&mut output, &plot_candle.border_colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn values_json_into(output: &mut String, values: &[PineValue]) {
    for (value_index, value) in values.iter().enumerate() {
        if value_index > 0 {
            output.push(',');
        }
        output.push_str(&value_json(value));
    }
}

fn hlines_json(hlines: &[HLineOutput]) -> String {
    let mut output = String::from("[");
    for (index, hline) in hlines.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"price\":{}}}",
            hline.id,
            value_json(&hline.price)
        ));
    }
    output.push(']');
    output
}

fn fills_json(fills: &[FillOutput]) -> String {
    let mut output = String::from("[");
    for (index, fill) in fills.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"firstId\":{},\"secondId\":{}}}",
            fill.id, fill.first_id, fill.second_id
        ));
    }
    output.push(']');
    output
}

fn value_json(value: &PineValue) -> String {
    match value {
        PineValue::Int(value) => value.to_string(),
        PineValue::Float(value) => value.to_string(),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => format!("\"{}\"", json_escape(value)),
        PineValue::Color(value) => value.to_string(),
        PineValue::Plot(value) | PineValue::HLine(value) => value.to_string(),
        PineValue::Tuple(values) => {
            let mut output = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&value_json(value));
            }
            output.push(']');
            output
        }
        PineValue::Array(_) | PineValue::Na | PineValue::Void => "null".to_owned(),
    }
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn run_historical(program: &HirProgram, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::new(program).run(bars)
}

pub fn run_historical_profiled(
    program: &HirProgram,
    bars: &[Bar],
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::new(program).run_profiled(bars)
}

#[derive(Clone)]
pub struct HistoricalRuntime<'a> {
    program: &'a HirProgram,
    bars: usize,
    historical_end: Option<usize>,
    current_bar_update_kind: BarUpdateKind,
    current_bar_is_new: bool,
    series_store: SeriesStore,
    series_retention: SeriesRetention,
    current_symbols: HashMap<SymbolId, PineValue>,
    current_series: HashMap<SeriesId, PineValue>,
    var_store: HashMap<VarSlotId, PineValue>,
    array_store: HashMap<u32, Vec<PineValue>>,
    array_kinds: HashMap<u32, ArrayElementKind>,
    next_array_id: u32,
    call_state: HashMap<CallSiteId, PineValue>,
    valuewhen_state: HashMap<CallSiteId, VecDeque<PineValue>>,
    rolling_windows: HashMap<RollingWindowKey, RollingWindowState>,
    rsi_state: HashMap<CallSiteId, RsiState>,
    macd_state: HashMap<CallSiteId, MacdState>,
    vwap_call_state: HashMap<CallSiteId, VwapState>,
    pivot_point_state: HashMap<CallSiteId, PivotPointState>,
    random_state: HashMap<CallSiteId, u64>,
    previous_bar_time: Option<i64>,
    price_flow_previous_close: Option<f64>,
    price_flow_previous_volume: Option<f64>,
    accdist_state: PineValue,
    accdist_current: PineValue,
    iii_current: PineValue,
    nvi_state: PineValue,
    nvi_current: PineValue,
    obv_state: PineValue,
    obv_current: PineValue,
    pvi_state: PineValue,
    pvi_current: PineValue,
    pvt_state: PineValue,
    pvt_current: PineValue,
    vwap_weighted_sum: f64,
    vwap_volume_sum: f64,
    vwap_current: PineValue,
    wad_state: PineValue,
    wad_current: PineValue,
    wvad_current: PineValue,
    plots: Vec<PlotSeries>,
    plot_chars: Vec<PlotCharSeries>,
    plot_shapes: Vec<PlotShapeSeries>,
    plot_arrows: Vec<PlotArrowSeries>,
    plot_bars: Vec<PlotBarSeries>,
    plot_candles: Vec<PlotCandleSeries>,
    bg_colors: Vec<ColorSeries>,
    bar_colors: Vec<ColorSeries>,
    hlines: Vec<HLineOutput>,
    fills: Vec<FillOutput>,
}

pub struct RealtimeRuntime<'a> {
    confirmed: HistoricalRuntime<'a>,
    forming: Option<HistoricalRuntime<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RsiState {
    previous_source: f64,
    average_gain: Option<f64>,
    average_loss: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MacdState {
    fast_ema: Option<f64>,
    slow_ema: Option<f64>,
    signal_ema: Option<f64>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct VwapState {
    weighted_sum: f64,
    weighted_square_sum: f64,
    volume_sum: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PivotPointPeriod {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

impl PivotPointPeriod {
    fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    fn update(&mut self, high: f64, low: f64, close: f64) {
        self.high = self.high.max(high);
        self.low = self.low.min(low);
        self.close = close;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct PivotPointState {
    current: Option<PivotPointPeriod>,
    active_levels: Option<Vec<PineValue>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct RollingWindowState {
    values: VecDeque<Option<f64>>,
    sum: f64,
    sum_squares: f64,
    na_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RollingWindowKey {
    Single(CallSiteId),
    VwmaWeighted(CallSiteId),
    VwmaVolume(CallSiteId),
    MfiPositive(CallSiteId),
    MfiNegative(CallSiteId),
    CmoPositive(CallSiteId),
    CmoNegative(CallSiteId),
    AoFast(CallSiteId),
    AoSlow(CallSiteId),
    CorrelationLeft(CallSiteId),
    CorrelationRight(CallSiteId),
    CorrelationProduct(CallSiteId),
    CovarianceLeft(CallSiteId),
    CovarianceRight(CallSiteId),
    CovarianceProduct(CallSiteId),
    StochHigh(CallSiteId),
    StochLow(CallSiteId),
    WprHigh(CallSiteId),
    WprLow(CallSiteId),
    HmaHalf(CallSiteId),
    HmaFull(CallSiteId),
    HmaSmooth(CallSiteId),
    RisingFalling(CallSiteId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StmtControl {
    None,
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayElementKind {
    Float,
    Int,
    Bool,
    String,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArraySearchMode {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayBinarySearchMode {
    Exact,
    Leftmost,
    Rightmost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayTruthMode {
    Every,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayNumericMode {
    Min,
    Max,
    Sum,
    Avg,
    Range,
    Median,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayVarianceMode {
    Variance,
    Stdev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayPercentileMode {
    NearestRank,
    LinearInterpolation,
}

fn infer_array_from_kind(values: &[PineValue]) -> Option<ArrayElementKind> {
    let mut inferred_kind: Option<ArrayElementKind> = None;
    for value in values {
        let next_kind = match value {
            PineValue::Na => continue,
            PineValue::Int(_) => ArrayElementKind::Int,
            PineValue::Float(_) => ArrayElementKind::Float,
            PineValue::Bool(_) => ArrayElementKind::Bool,
            PineValue::String(_) => ArrayElementKind::String,
            PineValue::Color(_) => ArrayElementKind::Color,
            _ => return None,
        };
        inferred_kind = Some(match (inferred_kind, next_kind) {
            (None, kind) => kind,
            (Some(ArrayElementKind::Int), ArrayElementKind::Float)
            | (Some(ArrayElementKind::Float), ArrayElementKind::Int)
            | (Some(ArrayElementKind::Float), ArrayElementKind::Float)
            | (Some(ArrayElementKind::Int), ArrayElementKind::Int) => {
                if matches!(next_kind, ArrayElementKind::Float)
                    || matches!(inferred_kind, Some(ArrayElementKind::Float))
                {
                    ArrayElementKind::Float
                } else {
                    ArrayElementKind::Int
                }
            }
            (Some(current), kind) if current == kind => current,
            _ => return None,
        });
    }
    inferred_kind
}

impl<'a> HistoricalRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self {
            program,
            bars: 0,
            historical_end: None,
            current_bar_update_kind: BarUpdateKind::Historical,
            current_bar_is_new: true,
            series_store: SeriesStore::new(),
            series_retention: SeriesRetention::from_program(program),
            current_symbols: HashMap::new(),
            current_series: HashMap::new(),
            var_store: HashMap::new(),
            array_store: HashMap::new(),
            array_kinds: HashMap::new(),
            next_array_id: 0,
            call_state: HashMap::new(),
            valuewhen_state: HashMap::new(),
            rolling_windows: HashMap::new(),
            rsi_state: HashMap::new(),
            macd_state: HashMap::new(),
            vwap_call_state: HashMap::new(),
            pivot_point_state: HashMap::new(),
            random_state: HashMap::new(),
            previous_bar_time: None,
            price_flow_previous_close: None,
            price_flow_previous_volume: None,
            accdist_state: PineValue::Na,
            accdist_current: PineValue::Na,
            iii_current: PineValue::Na,
            nvi_state: PineValue::Na,
            nvi_current: PineValue::Na,
            obv_state: PineValue::Na,
            obv_current: PineValue::Na,
            pvi_state: PineValue::Na,
            pvi_current: PineValue::Na,
            pvt_state: PineValue::Na,
            pvt_current: PineValue::Na,
            vwap_weighted_sum: 0.0,
            vwap_volume_sum: 0.0,
            vwap_current: PineValue::Na,
            wad_state: PineValue::Na,
            wad_current: PineValue::Na,
            wvad_current: PineValue::Na,
            plots: Vec::new(),
            plot_chars: Vec::new(),
            plot_shapes: Vec::new(),
            plot_arrows: Vec::new(),
            plot_bars: Vec::new(),
            plot_candles: Vec::new(),
            bg_colors: Vec::new(),
            bar_colors: Vec::new(),
            hlines: Vec::new(),
            fills: Vec::new(),
        }
    }

    fn run(mut self, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
        self.append_bars(bars)?;

        Ok(self.result())
    }

    fn run_profiled(mut self, bars: &[Bar]) -> Result<RuntimeProfiledResult, RuntimeError> {
        self.append_bars(bars)?;

        Ok(RuntimeProfiledResult {
            result: self.result(),
            profile: self.profile(),
        })
    }

    pub fn append_bars(&mut self, bars: &[Bar]) -> Result<(), RuntimeError> {
        let previous_historical_end = self.historical_end;
        self.historical_end = Some(self.bars + bars.len());
        let result = (|| {
            for bar in bars {
                self.append_bar_with_kind(*bar, BarUpdateKind::Historical)?;
            }
            Ok(())
        })();
        self.historical_end = previous_historical_end;
        result
    }

    pub fn append_bar(&mut self, bar: Bar) -> Result<(), RuntimeError> {
        self.append_bar_with_kind(bar, BarUpdateKind::Historical)
    }

    fn append_bar_with_kind(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
    ) -> Result<(), RuntimeError> {
        self.append_bar_with_context(bar, update_kind, true)
    }

    fn append_bar_with_context(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
        is_new_bar: bool,
    ) -> Result<(), RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = update_kind;
        self.current_bar_is_new = is_new_bar;
        self.series_store.set_current_bar(bar_index);
        self.current_symbols.clear();
        self.current_series.clear();
        self.set_builtin_symbols(&bar, bar_index)?;

        for statement in &self.program.statements {
            match self.eval_stmt(statement)? {
                StmtControl::None => {}
                StmtControl::Break | StmtControl::Continue => {
                    return Err(RuntimeError {
                        message: "loop control escaped its enclosing loop".to_owned(),
                    });
                }
            }
        }

        self.finalize_series_outputs();
        self.commit_current_series()?;
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        Ok(())
    }

    fn eval_stmt(&mut self, statement: &HirStmt) -> Result<StmtControl, RuntimeError> {
        match &statement.kind {
            HirStmtKind::Expr(expr) => {
                self.eval_expr(expr)?;
            }
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let branch = match self.eval_expr(condition)? {
                    PineValue::Bool(true) => then_branch,
                    PineValue::Bool(false) | PineValue::Na => else_branch,
                    _ => return Ok(StmtControl::None),
                };
                for statement in branch {
                    match self.eval_stmt(statement)? {
                        StmtControl::None => {}
                        control => return Ok(control),
                    }
                }
            }
            HirStmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                self.eval_for_loop(*counter, from, to, step.as_ref(), body, None)?;
            }
            HirStmtKind::While { condition, body } => {
                self.eval_while_loop(condition, body)?;
            }
            HirStmtKind::Break => return Ok(StmtControl::Break),
            HirStmtKind::Continue => return Ok(StmtControl::Continue),
            HirStmtKind::Decl { symbol, value } => {
                let value = self.eval_decl(*symbol, value)?;
                self.set_symbol_value(*symbol, value);
            }
            HirStmtKind::Reassign { symbol, value } => {
                let value = self.eval_expr(value)?;
                if let Some(var_slot_id) = self.var_slot_for_symbol(*symbol) {
                    self.var_store.insert(var_slot_id, value.clone());
                }
                self.set_symbol_value(*symbol, value);
            }
            HirStmtKind::TupleDecl { symbols, value } => {
                let value = self.eval_expr(value)?;
                let PineValue::Tuple(values) = value else {
                    return Ok(StmtControl::None);
                };
                for (symbol, value) in symbols.iter().zip(values) {
                    self.set_symbol_value(*symbol, value);
                }
            }
        }

        Ok(StmtControl::None)
    }

    fn eval_for_loop(
        &mut self,
        counter: SymbolId,
        from: &HirExpr,
        to: &HirExpr,
        step: Option<&HirExpr>,
        body: &[HirStmt],
        result: Option<&HirExpr>,
    ) -> Result<PineValue, RuntimeError> {
        let Some(from) = self.eval_expr(from)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let Some(to) = self.eval_expr(to)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let step_size = if let Some(step) = step {
            let Some(step) = self.eval_expr(step)?.as_i64() else {
                return Ok(PineValue::Na);
            };
            if step == 0 {
                return Err(RuntimeError {
                    message: "for loop step cannot be zero".to_owned(),
                });
            }
            step.checked_abs().ok_or_else(|| RuntimeError {
                message: "for loop step is out of range".to_owned(),
            })?
        } else {
            1
        };
        let step = if from <= to { step_size } else { -step_size };
        let mut value = from;
        let mut loop_result = PineValue::Na;
        loop {
            if (step > 0 && value > to) || (step < 0 && value < to) {
                break;
            }
            self.set_symbol_value(counter, PineValue::Int(value));
            let mut control = StmtControl::None;
            for statement in body {
                match self.eval_stmt(statement)? {
                    StmtControl::None => {}
                    next_control => {
                        control = next_control;
                        break;
                    }
                }
            }
            match control {
                StmtControl::None => {
                    if let Some(result) = result {
                        loop_result = self.eval_expr(result)?;
                    }
                }
                StmtControl::Break => break,
                StmtControl::Continue => {}
            }
            let Some(next) = value.checked_add(step) else {
                break;
            };
            value = next;
        }
        Ok(loop_result)
    }

    fn eval_while_loop(
        &mut self,
        condition: &HirExpr,
        body: &[HirStmt],
    ) -> Result<(), RuntimeError> {
        let mut iterations = 0_usize;
        loop {
            match self.eval_expr(condition)? {
                PineValue::Bool(true) => {}
                PineValue::Bool(false) | PineValue::Na => break,
                _ => break,
            }

            if iterations >= MAX_WHILE_ITERATIONS {
                return Err(RuntimeError {
                    message: format!(
                        "while loop exceeded maximum iteration count of {MAX_WHILE_ITERATIONS}"
                    ),
                });
            }
            iterations += 1;

            for statement in body {
                match self.eval_stmt(statement)? {
                    StmtControl::None => {}
                    StmtControl::Break => return Ok(()),
                    StmtControl::Continue => break,
                }
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        RuntimeResult {
            plots: self.plots.clone(),
            plot_chars: self.plot_chars.clone(),
            plot_shapes: self.plot_shapes.clone(),
            plot_arrows: self.plot_arrows.clone(),
            plot_bars: self.plot_bars.clone(),
            plot_candles: self.plot_candles.clone(),
            bg_colors: self.bg_colors.clone(),
            bar_colors: self.bar_colors.clone(),
            hlines: self.hlines.clone(),
            fills: self.fills.clone(),
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn profile(&self) -> RuntimeProfile {
        let series_buffers = self.series_store.buffers.len();
        let series_values = self
            .series_store
            .buffers
            .values()
            .map(Vec::len)
            .sum::<usize>();
        let series_capacity = self
            .series_store
            .buffers
            .values()
            .map(Vec::capacity)
            .sum::<usize>();
        let plot_values = self
            .plots
            .iter()
            .map(|plot| plot.values.len())
            .sum::<usize>();
        let plot_capacity = self
            .plots
            .iter()
            .map(|plot| plot.values.capacity())
            .sum::<usize>();
        let plot_char_values = self
            .plot_chars
            .iter()
            .map(|plot_char| plot_char.values.len())
            .sum::<usize>();
        let plot_char_capacity = self
            .plot_chars
            .iter()
            .map(|plot_char| {
                plot_char.values.capacity()
                    + plot_char.chars.capacity()
                    + plot_char.colors.capacity()
            })
            .sum::<usize>();
        let plot_shape_values = self
            .plot_shapes
            .iter()
            .map(|plot_shape| plot_shape.values.len())
            .sum::<usize>();
        let plot_shape_capacity = self
            .plot_shapes
            .iter()
            .map(|plot_shape| {
                plot_shape.values.capacity()
                    + plot_shape.styles.capacity()
                    + plot_shape.locations.capacity()
                    + plot_shape.colors.capacity()
                    + plot_shape.texts.capacity()
                    + plot_shape.text_colors.capacity()
                    + plot_shape.sizes.capacity()
            })
            .sum::<usize>();
        let plot_arrow_values = self
            .plot_arrows
            .iter()
            .map(|plot_arrow| plot_arrow.values.len())
            .sum::<usize>();
        let plot_arrow_capacity = self
            .plot_arrows
            .iter()
            .map(|plot_arrow| {
                plot_arrow.values.capacity()
                    + plot_arrow.color_ups.capacity()
                    + plot_arrow.color_downs.capacity()
                    + plot_arrow.min_heights.capacity()
                    + plot_arrow.max_heights.capacity()
            })
            .sum::<usize>();
        let plot_bar_values = self
            .plot_bars
            .iter()
            .map(|plot_bar| plot_bar.opens.len())
            .sum::<usize>();
        let plot_bar_capacity = self
            .plot_bars
            .iter()
            .map(|plot_bar| {
                plot_bar.opens.capacity()
                    + plot_bar.highs.capacity()
                    + plot_bar.lows.capacity()
                    + plot_bar.closes.capacity()
                    + plot_bar.colors.capacity()
            })
            .sum::<usize>();
        let plot_candle_values = self
            .plot_candles
            .iter()
            .map(|plot_candle| plot_candle.opens.len())
            .sum::<usize>();
        let plot_candle_capacity = self
            .plot_candles
            .iter()
            .map(|plot_candle| {
                plot_candle.opens.capacity()
                    + plot_candle.highs.capacity()
                    + plot_candle.lows.capacity()
                    + plot_candle.closes.capacity()
                    + plot_candle.colors.capacity()
                    + plot_candle.wick_colors.capacity()
                    + plot_candle.border_colors.capacity()
            })
            .sum::<usize>();
        let bg_color_values = self
            .bg_colors
            .iter()
            .map(|colors| colors.values.len())
            .sum::<usize>();
        let bg_color_capacity = self
            .bg_colors
            .iter()
            .map(|colors| colors.values.capacity())
            .sum::<usize>();
        let bar_color_values = self
            .bar_colors
            .iter()
            .map(|colors| colors.values.len())
            .sum::<usize>();
        let bar_color_capacity = self
            .bar_colors
            .iter()
            .map(|colors| colors.values.capacity())
            .sum::<usize>();
        let rolling_window_values = self
            .rolling_windows
            .values()
            .map(|window| window.values.len())
            .sum::<usize>();
        let rolling_window_value_capacity = self
            .rolling_windows
            .values()
            .map(|window| window.values.capacity())
            .sum::<usize>();
        let valuewhen_state_values = self
            .valuewhen_state
            .values()
            .map(VecDeque::len)
            .sum::<usize>();
        let valuewhen_state_value_capacity = self
            .valuewhen_state
            .values()
            .map(VecDeque::capacity)
            .sum::<usize>();
        let array_values = self.array_store.values().map(Vec::len).sum::<usize>();
        let array_value_capacity = self.array_store.values().map(Vec::capacity).sum::<usize>();

        RuntimeProfile {
            bars: self.bars,
            series_buffers,
            series_values,
            series_capacity,
            max_series_depth: self.series_store.max_depth(),
            history_retention_mode: self.series_retention.mode(),
            history_max_constant_offset: self.program.history.max_constant_offset,
            history_max_bars_back: self.program.max_bars_back,
            history_has_dynamic_offsets: self.program.history.has_dynamic_offsets,
            symbol_slots: self.current_symbols.len(),
            symbol_capacity: self.current_symbols.capacity(),
            current_series_slots: self.current_series.len(),
            current_series_capacity: self.current_series.capacity(),
            var_slots: self.var_store.len(),
            var_capacity: self.var_store.capacity(),
            array_slots: self.array_store.len(),
            array_capacity: self.array_store.capacity(),
            array_values,
            array_value_capacity,
            call_state_slots: self.call_state.len(),
            call_state_capacity: self.call_state.capacity(),
            valuewhen_state_slots: self.valuewhen_state.len(),
            valuewhen_state_capacity: self.valuewhen_state.capacity(),
            valuewhen_state_values,
            valuewhen_state_value_capacity,
            rolling_window_slots: self.rolling_windows.len(),
            rolling_window_capacity: self.rolling_windows.capacity(),
            rolling_window_values,
            rolling_window_value_capacity,
            rsi_state_slots: self.rsi_state.len(),
            rsi_state_capacity: self.rsi_state.capacity(),
            macd_state_slots: self.macd_state.len(),
            macd_state_capacity: self.macd_state.capacity(),
            plots: self.plots.len(),
            plot_values,
            plot_capacity,
            plot_chars: self.plot_chars.len(),
            plot_char_values,
            plot_char_capacity,
            plot_shapes: self.plot_shapes.len(),
            plot_shape_values,
            plot_shape_capacity,
            plot_arrows: self.plot_arrows.len(),
            plot_arrow_values,
            plot_arrow_capacity,
            plot_bars: self.plot_bars.len(),
            plot_bar_values,
            plot_bar_capacity,
            plot_candles: self.plot_candles.len(),
            plot_candle_values,
            plot_candle_capacity,
            bg_colors: self.bg_colors.len(),
            bg_color_values,
            bg_color_capacity,
            bar_colors: self.bar_colors.len(),
            bar_color_values,
            bar_color_capacity,
            hlines: self.hlines.len(),
            hline_capacity: self.hlines.capacity(),
            fills: self.fills.len(),
            fill_capacity: self.fills.capacity(),
        }
    }

    fn set_builtin_symbols(&mut self, bar: &Bar, bar_index: usize) -> Result<(), RuntimeError> {
        let datetime = utc_datetime_from_millis(bar.time)?;
        let chart_duration_ms = timeframe_seconds(DEFAULT_CHART_TIMEFRAME)
            .and_then(|seconds| seconds.checked_mul(1000))
            .ok_or_else(|| RuntimeError {
                message: "default chart timeframe duration is invalid".to_owned(),
            })?;
        let time_close = bar
            .time
            .checked_add(chart_duration_ms)
            .ok_or_else(|| RuntimeError {
                message: format!("time_close timestamp is out of range: {}", bar.time),
            })?;
        let previous_close = self.price_flow_previous_close;
        let previous_volume = self.price_flow_previous_volume;
        self.accdist_current = self.next_accdist(bar);
        self.iii_current = Self::iii_value(bar);
        self.nvi_current = self.next_nvi(bar, previous_close, previous_volume);
        self.obv_current = self.next_obv(bar, previous_close);
        self.pvi_current = self.next_pvi(bar, previous_close, previous_volume);
        self.pvt_current = self.next_pvt(bar, previous_close);
        self.vwap_current = self.next_vwap(bar);
        self.wad_current = self.next_wad(bar, previous_close);
        self.wvad_current = Self::wvad_value(bar);
        self.price_flow_previous_close = Some(bar.close);
        self.price_flow_previous_volume = Some(bar.volume);
        let builtins = [
            ("open", PineValue::Float(bar.open)),
            ("high", PineValue::Float(bar.high)),
            ("low", PineValue::Float(bar.low)),
            ("close", PineValue::Float(bar.close)),
            ("volume", PineValue::Float(bar.volume)),
            ("time", PineValue::Int(bar.time)),
            ("time_close", PineValue::Int(time_close)),
            ("year", PineValue::Int(datetime.year() as i64)),
            ("month", PineValue::Int(datetime.month() as i64)),
            (
                "weekofyear",
                PineValue::Int(datetime.iso_week().week() as i64),
            ),
            ("dayofmonth", PineValue::Int(datetime.day() as i64)),
            ("dayofweek", PineValue::Int(dayofweek_value(datetime))),
            ("hour", PineValue::Int(datetime.hour() as i64)),
            ("minute", PineValue::Int(datetime.minute() as i64)),
            ("second", PineValue::Int(datetime.second() as i64)),
            ("hl2", PineValue::Float((bar.high + bar.low) / 2.0)),
            (
                "hlc3",
                PineValue::Float((bar.high + bar.low + bar.close) / 3.0),
            ),
            (
                "hlcc4",
                PineValue::Float((bar.high + bar.low + bar.close + bar.close) / 4.0),
            ),
            (
                "ohlc4",
                PineValue::Float((bar.open + bar.high + bar.low + bar.close) / 4.0),
            ),
            ("bar_index", PineValue::Int(bar_index as i64)),
        ];

        for (name, value) in builtins {
            let symbol = self
                .program
                .symbols
                .iter()
                .find(|symbol| symbol.name == name)
                .ok_or_else(|| RuntimeError {
                    message: format!("missing builtin symbol `{name}`"),
                })?;
            self.current_symbols.insert(symbol.id, value.clone());
            if let Some(series_id) = symbol.series_id {
                self.current_series.insert(series_id, value);
            }
        }

        Ok(())
    }

    fn next_accdist(&mut self, bar: &Bar) -> PineValue {
        let range = bar.high - bar.low;
        if range == 0.0 {
            self.accdist_state = PineValue::Na;
            return PineValue::Na;
        }

        let multiplier = ((bar.close - bar.low) - (bar.high - bar.close)) / range;
        let increment = multiplier * bar.volume;
        if !increment.is_finite() {
            self.accdist_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.accdist_state.as_f64().unwrap_or(0.0) + increment;
        self.accdist_state = finite_float_or_na(value);
        self.accdist_state.clone()
    }

    fn iii_value(bar: &Bar) -> PineValue {
        let denominator = (bar.high - bar.low) * bar.volume;
        if denominator == 0.0 {
            return PineValue::Na;
        }

        finite_float_or_na((2.0 * bar.close - bar.high - bar.low) / denominator)
    }

    fn next_nvi(
        &mut self,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
    ) -> PineValue {
        Self::next_volume_index(
            &mut self.nvi_state,
            bar,
            previous_close,
            previous_volume,
            |volume, previous_volume| volume < previous_volume,
        )
    }

    fn next_obv(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.obv_state = PineValue::Na;
            return PineValue::Na;
        };
        let signed_volume = match bar.close.partial_cmp(&previous_close) {
            Some(Ordering::Greater) => bar.volume,
            Some(Ordering::Less) => -bar.volume,
            Some(Ordering::Equal) => 0.0,
            None => {
                self.obv_state = PineValue::Na;
                return PineValue::Na;
            }
        };
        let value = self.obv_state.as_f64().unwrap_or(0.0) + signed_volume;
        self.obv_state = PineValue::Float(value);
        self.obv_state.clone()
    }

    fn next_pvi(
        &mut self,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
    ) -> PineValue {
        Self::next_volume_index(
            &mut self.pvi_state,
            bar,
            previous_close,
            previous_volume,
            |volume, previous_volume| volume > previous_volume,
        )
    }

    fn next_pvt(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        };
        if previous_close == 0.0 {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        }

        let increment = ((bar.close - previous_close) / previous_close) * bar.volume;
        if !increment.is_finite() {
            self.pvt_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.pvt_state.as_f64().unwrap_or(0.0) + increment;
        self.pvt_state = finite_float_or_na(value);
        self.pvt_state.clone()
    }

    fn next_vwap(&mut self, bar: &Bar) -> PineValue {
        let source = (bar.high + bar.low + bar.close) / 3.0;
        let weighted = source * bar.volume;
        if !source.is_finite() || !bar.volume.is_finite() || !weighted.is_finite() {
            self.vwap_weighted_sum = 0.0;
            self.vwap_volume_sum = 0.0;
            self.vwap_current = PineValue::Na;
            return PineValue::Na;
        }

        self.vwap_weighted_sum += weighted;
        self.vwap_volume_sum += bar.volume;
        if self.vwap_volume_sum == 0.0
            || !self.vwap_weighted_sum.is_finite()
            || !self.vwap_volume_sum.is_finite()
        {
            self.vwap_current = PineValue::Na;
            return PineValue::Na;
        }

        self.vwap_current = finite_float_or_na(self.vwap_weighted_sum / self.vwap_volume_sum);
        self.vwap_current.clone()
    }

    fn next_wad(&mut self, bar: &Bar, previous_close: Option<f64>) -> PineValue {
        let Some(previous_close) = previous_close else {
            self.wad_state = PineValue::Na;
            return PineValue::Na;
        };

        let momentum = bar.close - previous_close;
        let gain = match momentum.partial_cmp(&0.0) {
            Some(Ordering::Greater) => bar.close - bar.low.min(previous_close),
            Some(Ordering::Less) => bar.close - bar.high.max(previous_close),
            Some(Ordering::Equal) => 0.0,
            None => {
                self.wad_state = PineValue::Na;
                return PineValue::Na;
            }
        };
        if !gain.is_finite() {
            self.wad_state = PineValue::Na;
            return PineValue::Na;
        }

        let value = self.wad_state.as_f64().unwrap_or(0.0) + gain;
        self.wad_state = finite_float_or_na(value);
        self.wad_state.clone()
    }

    fn next_volume_index(
        state: &mut PineValue,
        bar: &Bar,
        previous_close: Option<f64>,
        previous_volume: Option<f64>,
        should_update: impl FnOnce(f64, f64) -> bool,
    ) -> PineValue {
        let previous_value = state.as_f64().filter(|value| *value != 0.0).unwrap_or(1.0);
        let Some(previous_close) = previous_close else {
            *state = PineValue::Float(previous_value);
            return state.clone();
        };

        if bar.close == 0.0
            || previous_close == 0.0
            || !bar.close.is_finite()
            || !previous_close.is_finite()
            || !bar.volume.is_finite()
        {
            *state = PineValue::Float(previous_value);
            return state.clone();
        }

        let previous_volume = previous_volume
            .filter(|volume| volume.is_finite())
            .unwrap_or(0.0);
        if !should_update(bar.volume, previous_volume) {
            *state = PineValue::Float(previous_value);
            return state.clone();
        }

        let value =
            previous_value + ((bar.close - previous_close) / previous_close) * previous_value;
        *state = finite_float_or_na(value);
        state.clone()
    }

    fn wvad_value(bar: &Bar) -> PineValue {
        let range = bar.high - bar.low;
        if range == 0.0 {
            return PineValue::Na;
        }

        finite_float_or_na(((bar.close - bar.open) / range) * bar.volume)
    }

    fn eval_builtin_value(&self, name: &str) -> PineValue {
        if name == "barstate.isfirst" {
            return PineValue::Bool(self.bars == 0);
        }
        if name == "barstate.islast" {
            let is_last = match self.current_bar_update_kind {
                BarUpdateKind::Historical => self
                    .historical_end
                    .is_none_or(|historical_end| self.bars + 1 == historical_end),
                BarUpdateKind::Forming | BarUpdateKind::Confirmed => true,
            };
            return PineValue::Bool(is_last);
        }
        if name == "barstate.isnew" {
            return PineValue::Bool(self.current_bar_is_new);
        }
        if name == "barstate.isconfirmed" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Historical | BarUpdateKind::Confirmed
            ));
        }
        if name == "barstate.ishistory" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Historical
            ));
        }
        if name == "barstate.isrealtime" {
            return PineValue::Bool(matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Forming | BarUpdateKind::Confirmed
            ));
        }
        if name == "session.ismarket" {
            return PineValue::Bool(true);
        }
        if name == "session.ispremarket" || name == "session.ispostmarket" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.period" {
            return PineValue::String(DEFAULT_CHART_TIMEFRAME.to_owned());
        }
        if name == "timeframe.isseconds" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isminutes" {
            return PineValue::Bool(true);
        }
        if name == "timeframe.isintraday" {
            return PineValue::Bool(true);
        }
        if name == "timeframe.isdaily" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isweekly" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.ismonthly" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.isdwm" {
            return PineValue::Bool(false);
        }
        if name == "timeframe.multiplier" {
            return PineValue::Int(1);
        }
        if name == "ta.accdist" {
            return self.accdist_current.clone();
        }
        if name == "ta.iii" {
            return self.iii_current.clone();
        }
        if name == "ta.nvi" {
            return self.nvi_current.clone();
        }
        if name == "ta.obv" {
            return self.obv_current.clone();
        }
        if name == "ta.pvi" {
            return self.pvi_current.clone();
        }
        if name == "ta.pvt" {
            return self.pvt_current.clone();
        }
        if name == "ta.tr" {
            return self.true_range(false);
        }
        if name == "ta.vwap" {
            return self.vwap_current.clone();
        }
        if name == "ta.wad" {
            return self.wad_current.clone();
        }
        if name == "ta.wvad" {
            return self.wvad_current.clone();
        }
        eval_static_builtin_value(name)
    }

    fn eval_decl(&mut self, symbol: SymbolId, value: &HirExpr) -> Result<PineValue, RuntimeError> {
        let Some(var_slot_id) = self.var_slot_for_symbol(symbol) else {
            return self.eval_expr(value);
        };

        if let Some(value) = self.var_store.get(&var_slot_id).cloned() {
            Ok(value)
        } else {
            let value = self.eval_expr(value)?;
            self.var_store.insert(var_slot_id, value.clone());
            Ok(value)
        }
    }

    fn var_slot_for_symbol(&self, symbol_id: SymbolId) -> Option<VarSlotId> {
        self.program
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)
            .and_then(|symbol| symbol.var_slot_id)
    }

    fn series_id_for_symbol(&self, symbol_id: SymbolId) -> Option<SeriesId> {
        self.program
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)
            .and_then(|symbol| symbol.series_id)
    }

    fn set_symbol_value(&mut self, symbol: SymbolId, value: PineValue) {
        self.current_symbols.insert(symbol, value.clone());
        if let Some(series_id) = self.series_id_for_symbol(symbol) {
            self.current_series.insert(series_id, value);
        }
    }

    fn commit_current_series(&mut self) -> Result<(), RuntimeError> {
        if self.projected_series_values_after_commit() > MAX_SERIES_HISTORY_VALUES {
            return Err(RuntimeError {
                message: format!(
                    "series history limit exceeded: at most {MAX_SERIES_HISTORY_VALUES} committed values are retained"
                ),
            });
        }

        for raw_series_id in 0..self.program.next_series_id {
            let series_id = SeriesId(raw_series_id);
            let value = self
                .current_series
                .remove(&series_id)
                .unwrap_or(PineValue::Na);
            self.series_store.commit(
                series_id,
                value,
                self.series_retention.max_depth_for(series_id),
            );
        }
        Ok(())
    }

    fn projected_series_values_after_commit(&self) -> usize {
        let mut total = 0usize;
        for raw_series_id in 0..self.program.next_series_id {
            let series_id = SeriesId(raw_series_id);
            let next_len = self.series_store.len(series_id).saturating_add(1);
            let retained_len = self
                .series_retention
                .max_depth_for(series_id)
                .map_or(next_len, |max_depth| next_len.min(max_depth));
            total = total.saturating_add(retained_len);
        }
        total
    }

    fn eval_expr(&mut self, expr: &HirExpr) -> Result<PineValue, RuntimeError> {
        let value = match &expr.kind {
            HirExprKind::Literal(literal) => eval_literal(literal),
            HirExprKind::Symbol(symbol) => self
                .current_symbols
                .get(symbol)
                .cloned()
                .unwrap_or(PineValue::Na),
            HirExprKind::Builtin(name) => self.eval_builtin_value(name),
            HirExprKind::Unary { op, expr } => {
                let value = self.eval_expr(expr)?;
                eval_unary(*op, value)
            }
            HirExprKind::Binary { op, left, right } => {
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                eval_binary(*op, left, right)
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => match self.eval_expr(condition)? {
                PineValue::Bool(true) => self.eval_expr(then_expr)?,
                PineValue::Bool(false) | PineValue::Na => self.eval_expr(else_expr)?,
                _ => PineValue::Na,
            },
            HirExprKind::Switch { selector, arms } => {
                self.eval_switch(selector.as_deref(), arms)?
            }
            HirExprKind::For {
                counter,
                from,
                to,
                step,
                statements,
                result,
            } => self.eval_for_loop(
                *counter,
                from,
                to,
                step.as_deref(),
                statements,
                Some(result),
            )?,
            HirExprKind::Tuple(items) => PineValue::Tuple(
                items
                    .iter()
                    .map(|item| self.eval_expr(item))
                    .collect::<Result<_, _>>()?,
            ),
            HirExprKind::Block { statements, result } => {
                for statement in statements {
                    match self.eval_stmt(statement)? {
                        StmtControl::None => {}
                        StmtControl::Break | StmtControl::Continue => {
                            return Err(RuntimeError {
                                message: "loop control escaped its enclosing loop".to_owned(),
                            });
                        }
                    }
                }
                self.eval_expr(result)?
            }
            HirExprKind::Call {
                callee,
                call_site_id,
                args,
            } => self.eval_call(callee, *call_site_id, args)?,
            HirExprKind::History { expr, offset } => self.eval_history(expr, offset)?,
        };

        if let Some(series_id) = expr.series_id {
            self.current_series.insert(series_id, value.clone());
        }

        Ok(value)
    }

    fn eval_history(
        &mut self,
        expr: &HirExpr,
        offset: &HirHistoryOffset,
    ) -> Result<PineValue, RuntimeError> {
        let Some(offset) = self.eval_history_offset(offset)? else {
            return Ok(PineValue::Na);
        };

        if offset == 0 {
            return self.eval_expr(expr);
        }

        self.eval_expr(expr)?;
        if let Some(series_id) = expr.series_id {
            Ok(self.series_store.read(series_id, offset))
        } else {
            Ok(PineValue::Na)
        }
    }

    fn eval_history_offset(
        &mut self,
        offset: &HirHistoryOffset,
    ) -> Result<Option<usize>, RuntimeError> {
        let value = match offset {
            HirHistoryOffset::Constant(offset) => return Ok(Some(*offset as usize)),
            HirHistoryOffset::Dynamic(expr) => self.eval_expr(expr)?,
        };

        match value {
            PineValue::Int(value) if value >= 0 => {
                usize::try_from(value).map(Some).map_err(|_| RuntimeError {
                    message: "history offset is too large".to_owned(),
                })
            }
            PineValue::Int(_) => Err(RuntimeError {
                message: "history offset must be non-negative".to_owned(),
            }),
            PineValue::Float(value) if value >= 0.0 && value.fract() == 0.0 => {
                if value > usize::MAX as f64 {
                    Err(RuntimeError {
                        message: "history offset is too large".to_owned(),
                    })
                } else {
                    Ok(Some(value as usize))
                }
            }
            PineValue::Float(value) if value < 0.0 => Err(RuntimeError {
                message: "history offset must be non-negative".to_owned(),
            }),
            PineValue::Na => Ok(None),
            _ => Err(RuntimeError {
                message: "history offset must be an int".to_owned(),
            }),
        }
    }

    fn eval_switch(
        &mut self,
        selector: Option<&HirExpr>,
        arms: &[pine_ir::HirSwitchArm],
    ) -> Result<PineValue, RuntimeError> {
        let selector_value = match selector {
            Some(selector) => Some(self.eval_expr(selector)?),
            None => None,
        };

        for arm in arms {
            let matches = match (&selector_value, &arm.condition) {
                (Some(selector_value), Some(case_expr)) => {
                    let case_value = self.eval_expr(case_expr)?;
                    matches!(
                        eval_binary(HirBinaryOp::Eq, selector_value.clone(), case_value),
                        PineValue::Bool(true)
                    )
                }
                (None, Some(condition)) => {
                    matches!(self.eval_expr(condition)?, PineValue::Bool(true))
                }
                (_, None) => true,
            };

            if matches {
                return self.eval_expr(&arm.result);
            }
        }

        Ok(PineValue::Na)
    }

    fn eval_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match callee {
            "indicator" => Ok(PineValue::Void),
            "input" | "input.int" | "input.float" | "input.bool" | "input.color"
            | "input.string" | "input.price" | "input.time" | "input.symbol"
            | "input.timeframe" | "input.session" | "input.text_area" | "input.source" => {
                self.eval_expr(&args[0].value)
            }
            "plot" => {
                let value = self.eval_expr(&args[0].value)?;
                push_series_value(&mut self.plots, self.bars, call_site_id.0, value);
                Ok(PineValue::Plot(call_site_id.0))
            }
            "plotchar" => {
                let Some(series_arg) = call_arg_expr(args, 0, "series") else {
                    return Err(RuntimeError {
                        message: "plotchar missing series argument".to_owned(),
                    });
                };
                let value = self.eval_expr(series_arg)?;
                let char_value = match call_arg_expr(args, 2, "char") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::String("*".to_owned()),
                };
                let color_value = match call_arg_expr(args, 3, "color") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                push_bar_aligned_output(
                    &mut self.plot_chars,
                    self.bars,
                    call_site_id.0,
                    PlotCharPoint {
                        value,
                        char_value,
                        color: color_value,
                    },
                );
                Ok(PineValue::Void)
            }
            "plotshape" => {
                let Some(series_arg) = call_arg_expr(args, 0, "series") else {
                    return Err(RuntimeError {
                        message: "plotshape missing series argument".to_owned(),
                    });
                };
                let value = self.eval_expr(series_arg)?;
                let style_value = match call_arg_expr(args, 2, "style") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::String("shape.xcross".to_owned()),
                };
                let location_value = match call_arg_expr(args, 3, "location") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::String("location.abovebar".to_owned()),
                };
                let color_value = match call_arg_expr(args, 4, "color") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                let text_value = match call_arg_expr(args, 6, "text") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::String(String::new()),
                };
                let text_color_value = match call_arg_expr(args, 7, "textcolor") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                let size_value = match call_arg_expr(args, 9, "size") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::String("size.auto".to_owned()),
                };
                push_bar_aligned_output(
                    &mut self.plot_shapes,
                    self.bars,
                    call_site_id.0,
                    PlotShapePoint {
                        value,
                        style: style_value,
                        location: location_value,
                        color: color_value,
                        text: text_value,
                        text_color: text_color_value,
                        size: size_value,
                    },
                );
                Ok(PineValue::Void)
            }
            "plotarrow" => {
                let Some(series_arg) = call_arg_expr(args, 0, "series") else {
                    return Err(RuntimeError {
                        message: "plotarrow missing series argument".to_owned(),
                    });
                };
                let value = self.eval_expr(series_arg)?;
                let color_up_value = match call_arg_expr(args, 2, "colorup") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Color(0x008000),
                };
                let color_down_value = match call_arg_expr(args, 3, "colordown") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Color(0xFF0000),
                };
                let min_height_value = match call_arg_expr(args, 5, "minheight") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Int(0),
                };
                let max_height_value = match call_arg_expr(args, 6, "maxheight") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Int(0),
                };
                push_bar_aligned_output(
                    &mut self.plot_arrows,
                    self.bars,
                    call_site_id.0,
                    PlotArrowPoint {
                        value,
                        color_up: color_up_value,
                        color_down: color_down_value,
                        min_height: min_height_value,
                        max_height: max_height_value,
                    },
                );
                Ok(PineValue::Void)
            }
            "plotbar" => {
                let Some(open_arg) = call_arg_expr(args, 0, "open") else {
                    return Err(RuntimeError {
                        message: "plotbar missing open argument".to_owned(),
                    });
                };
                let Some(high_arg) = call_arg_expr(args, 1, "high") else {
                    return Err(RuntimeError {
                        message: "plotbar missing high argument".to_owned(),
                    });
                };
                let Some(low_arg) = call_arg_expr(args, 2, "low") else {
                    return Err(RuntimeError {
                        message: "plotbar missing low argument".to_owned(),
                    });
                };
                let Some(close_arg) = call_arg_expr(args, 3, "close") else {
                    return Err(RuntimeError {
                        message: "plotbar missing close argument".to_owned(),
                    });
                };
                let open_value = self.eval_expr(open_arg)?;
                let high_value = self.eval_expr(high_arg)?;
                let low_value = self.eval_expr(low_arg)?;
                let close_value = self.eval_expr(close_arg)?;
                let color_value = match call_arg_expr(args, 5, "color") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                push_bar_aligned_output(
                    &mut self.plot_bars,
                    self.bars,
                    call_site_id.0,
                    PlotBarPoint {
                        open: open_value,
                        high: high_value,
                        low: low_value,
                        close: close_value,
                        color: color_value,
                    },
                );
                Ok(PineValue::Void)
            }
            "plotcandle" => {
                let Some(open_arg) = call_arg_expr(args, 0, "open") else {
                    return Err(RuntimeError {
                        message: "plotcandle missing open argument".to_owned(),
                    });
                };
                let Some(high_arg) = call_arg_expr(args, 1, "high") else {
                    return Err(RuntimeError {
                        message: "plotcandle missing high argument".to_owned(),
                    });
                };
                let Some(low_arg) = call_arg_expr(args, 2, "low") else {
                    return Err(RuntimeError {
                        message: "plotcandle missing low argument".to_owned(),
                    });
                };
                let Some(close_arg) = call_arg_expr(args, 3, "close") else {
                    return Err(RuntimeError {
                        message: "plotcandle missing close argument".to_owned(),
                    });
                };
                let open_value = self.eval_expr(open_arg)?;
                let high_value = self.eval_expr(high_arg)?;
                let low_value = self.eval_expr(low_arg)?;
                let close_value = self.eval_expr(close_arg)?;
                let color_value = match call_arg_expr(args, 5, "color") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                let wick_color_value = match call_arg_expr(args, 6, "wickcolor") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                let border_color_value = match call_arg_expr(args, 9, "bordercolor") {
                    Some(expr) => self.eval_expr(expr)?,
                    None => PineValue::Na,
                };
                push_bar_aligned_output(
                    &mut self.plot_candles,
                    self.bars,
                    call_site_id.0,
                    PlotCandlePoint {
                        open: open_value,
                        high: high_value,
                        low: low_value,
                        close: close_value,
                        color: color_value,
                        wick_color: wick_color_value,
                        border_color: border_color_value,
                    },
                );
                Ok(PineValue::Void)
            }
            "bgcolor" => {
                let value = self.eval_expr(&args[0].value)?;
                push_series_value(&mut self.bg_colors, self.bars, call_site_id.0, value);
                Ok(PineValue::Void)
            }
            "barcolor" => {
                let value = self.eval_expr(&args[0].value)?;
                push_series_value(&mut self.bar_colors, self.bars, call_site_id.0, value);
                Ok(PineValue::Void)
            }
            "hline" => {
                let price = self.eval_expr(&args[0].value)?;
                self.push_hline(call_site_id.0, price);
                Ok(PineValue::HLine(call_site_id.0))
            }
            "fill" => {
                let first = self.eval_expr(&args[0].value)?;
                let second = self.eval_expr(&args[1].value)?;
                self.push_fill(call_site_id.0, first, second);
                Ok(PineValue::Void)
            }
            "color.new" => self.eval_color_new(args),
            "color.rgb" => self.eval_color_rgb(args),
            "color.r" => self.eval_color_component(args, ColorComponent::Red),
            "color.g" => self.eval_color_component(args, ColorComponent::Green),
            "color.b" => self.eval_color_component(args, ColorComponent::Blue),
            "color.t" => self.eval_color_component(args, ColorComponent::Transparency),
            "color.from_gradient" => self.eval_color_from_gradient(args),
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
            "int" => self.eval_int_cast(args),
            "float" => self.eval_float_cast(args),
            "bool" => self.eval_bool_cast(args),
            "string" => self.eval_string_cast(args),
            "color" => self.eval_color_cast(args),
            "math.abs" => self.eval_math_abs(args),
            "math.max" => self.eval_math_extreme(args, MathExtreme::Max),
            "math.min" => self.eval_math_extreme(args, MathExtreme::Min),
            "math.avg" => self.eval_math_avg(args),
            "math.floor" => self.eval_math_floor(args),
            "math.ceil" => self.eval_math_ceil(args),
            "math.trunc" => self.eval_math_trunc(args),
            "math.sqrt" => self.eval_math_unary_float(args, f64::sqrt),
            "math.cbrt" => self.eval_math_unary_float(args, f64::cbrt),
            "math.log" => self.eval_math_unary_float(args, f64::ln),
            "math.log10" => self.eval_math_unary_float(args, f64::log10),
            "math.exp" => self.eval_math_unary_float(args, f64::exp),
            "math.acos" => self.eval_math_unary_float(args, f64::acos),
            "math.asin" => self.eval_math_unary_float(args, f64::asin),
            "math.atan" => self.eval_math_unary_float(args, f64::atan),
            "math.sign" => self.eval_math_sign(args),
            "math.todegrees" => self.eval_math_unary_float(args, f64::to_degrees),
            "math.toradians" => self.eval_math_unary_float(args, f64::to_radians),
            "math.sin" => self.eval_math_unary_float(args, f64::sin),
            "math.cos" => self.eval_math_unary_float(args, f64::cos),
            "math.tan" => self.eval_math_unary_float(args, f64::tan),
            "math.pow" => self.eval_math_pow(args),
            "math.hypot" => self.eval_math_hypot(args),
            "math.round" => self.eval_math_round(args),
            "math.round_to_mintick" => self.eval_math_round_to_mintick(args),
            "math.random" => self.eval_math_random(call_site_id, args),
            "math.sum" => self.eval_math_sum(call_site_id, args),
            "ta.sma" => self.eval_sma(call_site_id, args),
            "ta.ema" => self.eval_ema(call_site_id, args),
            "ta.dema" => self.eval_dema(call_site_id, args),
            "ta.tema" => self.eval_tema(call_site_id, args),
            "ta.rma" => self.eval_rma(call_site_id, args),
            "ta.rsi" => self.eval_rsi(call_site_id, args),
            "ta.macd" => self.eval_macd(call_site_id, args),
            "ta.tsi" => self.eval_tsi(call_site_id, args),
            "ta.cmo" => self.eval_cmo(call_site_id, args),
            "ta.cci" => self.eval_cci(call_site_id, args),
            "ta.cog" => self.eval_cog(call_site_id, args),
            "ta.ao" => self.eval_ao(call_site_id),
            "ta.bop" => self.eval_bop(),
            "ta.bb" => self.eval_bb(call_site_id, args),
            "ta.bbw" => self.eval_bbw(call_site_id, args),
            "ta.kc" => self.eval_kc(call_site_id, args),
            "ta.kcw" => self.eval_kcw(call_site_id, args),
            "ta.pivothigh" => self.eval_pivot(call_site_id, args, WindowExtreme::Highest),
            "ta.pivotlow" => self.eval_pivot(call_site_id, args, WindowExtreme::Lowest),
            "ta.pivot_point_levels" => self.eval_pivot_point_levels(call_site_id, args),
            "ta.cum" => self.eval_cum(call_site_id, args),
            "ta.max" => self.eval_all_time_extreme(call_site_id, args, WindowExtreme::Highest),
            "ta.min" => self.eval_all_time_extreme(call_site_id, args, WindowExtreme::Lowest),
            "ta.stdev" => self.eval_stdev(call_site_id, args),
            "ta.variance" => self.eval_variance(call_site_id, args),
            "ta.range" => self.eval_range(call_site_id, args),
            "ta.dev" => self.eval_dev(call_site_id, args),
            "ta.vwap" => self.eval_vwap_source(call_site_id, args),
            "ta.vwma" => self.eval_vwma(call_site_id, args),
            "ta.mfi" => self.eval_mfi(call_site_id, args),
            "ta.wma" => self.eval_wma(call_site_id, args),
            "ta.hma" => self.eval_hma(call_site_id, args),
            "ta.swma" => self.eval_swma(call_site_id, args),
            "ta.alma" => self.eval_alma(call_site_id, args),
            "ta.linreg" => self.eval_linreg(call_site_id, args),
            "ta.stoch" => self.eval_stoch(call_site_id, args),
            "ta.wpr" => self.eval_wpr(call_site_id, args),
            "ta.correlation" => self.eval_correlation(call_site_id, args),
            "ta.covariance" => self.eval_covariance(call_site_id, args),
            "ta.median" => self.eval_median(call_site_id, args),
            "ta.mode" => self.eval_mode(call_site_id, args),
            "ta.percentile_nearest_rank" => {
                self.eval_percentile(call_site_id, args, ArrayPercentileMode::NearestRank)
            }
            "ta.percentile_linear_interpolation" => {
                self.eval_percentile(call_site_id, args, ArrayPercentileMode::LinearInterpolation)
            }
            "ta.percentrank" => self.eval_percentrank(call_site_id, args),
            "ta.tr" => self.eval_tr(args),
            "ta.atr" => self.eval_atr(call_site_id, args),
            "ta.supertrend" => self.eval_supertrend(call_site_id, args),
            "ta.dmi" => self.eval_dmi(call_site_id, args),
            "ta.sar" => self.eval_sar(call_site_id, args),
            "ta.change" => self.eval_change(args),
            "ta.mom" => self.eval_mom(args),
            "ta.roc" => self.eval_roc(args),
            "ta.rising" => self.eval_rising_falling(call_site_id, args, RisingFallingMode::Rising),
            "ta.falling" => {
                self.eval_rising_falling(call_site_id, args, RisingFallingMode::Falling)
            }
            "ta.barssince" => self.eval_barssince(call_site_id, args),
            "ta.valuewhen" => self.eval_valuewhen(call_site_id, args),
            "ta.cross" => self.eval_cross(args, CrossMode::Any),
            "ta.crossover" => self.eval_cross(args, CrossMode::Over),
            "ta.crossunder" => self.eval_cross(args, CrossMode::Under),
            "ta.highest" => self.eval_window_extreme(call_site_id, args, WindowExtreme::Highest),
            "ta.lowest" => self.eval_window_extreme(call_site_id, args, WindowExtreme::Lowest),
            "ta.highestbars" => {
                self.eval_window_extreme_offset(call_site_id, args, WindowExtreme::Highest)
            }
            "ta.lowestbars" => {
                self.eval_window_extreme_offset(call_site_id, args, WindowExtreme::Lowest)
            }
            "na" => {
                let value = self.eval_expr(&args[0].value)?;
                Ok(PineValue::Bool(value.is_na()))
            }
            "nz" => {
                let value = self.eval_expr(&args[0].value)?;
                if value.is_na() {
                    if let Some(replacement) = args.get(1) {
                        self.eval_expr(&replacement.value)
                    } else {
                        Ok(PineValue::Int(0))
                    }
                } else {
                    Ok(value)
                }
            }
            "fixnan" => self.eval_fixnan(call_site_id, args),
            "array.new_float" => self.eval_array_new_float(args),
            "array.new_int" => self.eval_array_new_int(args),
            "array.new_bool" => self.eval_array_new_bool(args),
            "array.new_string" => self.eval_array_new_string(args),
            "array.new_color" => self.eval_array_new_color(args),
            "array.from" => self.eval_array_from(args),
            "array.size" => self.eval_array_size(args),
            "array.push" => self.eval_array_push(args),
            "array.get" => self.eval_array_get(args),
            "array.set" => self.eval_array_set(args),
            "array.insert" => self.eval_array_insert(args),
            "array.pop" => self.eval_array_pop(args),
            "array.remove" => self.eval_array_remove(args),
            "array.shift" => self.eval_array_shift(args),
            "array.unshift" => self.eval_array_unshift(args),
            "array.fill" => self.eval_array_fill(args),
            "array.first" => self.eval_array_first(args),
            "array.last" => self.eval_array_last(args),
            "array.copy" => self.eval_array_copy(args),
            "array.slice" => self.eval_array_slice(args),
            "array.concat" => self.eval_array_concat(args),
            "array.includes" => self.eval_array_includes(args),
            "array.every" => self.eval_array_truth(args, ArrayTruthMode::Every),
            "array.some" => self.eval_array_truth(args, ArrayTruthMode::Some),
            "array.indexof" => self.eval_array_indexof(args),
            "array.lastindexof" => self.eval_array_lastindexof(args),
            "array.binary_search" => {
                self.eval_array_binary_search(args, ArrayBinarySearchMode::Exact)
            }
            "array.binary_search_leftmost" => {
                self.eval_array_binary_search(args, ArrayBinarySearchMode::Leftmost)
            }
            "array.binary_search_rightmost" => {
                self.eval_array_binary_search(args, ArrayBinarySearchMode::Rightmost)
            }
            "array.abs" => self.eval_array_abs(args),
            "array.min" => self.eval_array_numeric(args, ArrayNumericMode::Min),
            "array.max" => self.eval_array_numeric(args, ArrayNumericMode::Max),
            "array.sum" => self.eval_array_numeric(args, ArrayNumericMode::Sum),
            "array.avg" => self.eval_array_numeric(args, ArrayNumericMode::Avg),
            "array.range" => self.eval_array_numeric(args, ArrayNumericMode::Range),
            "array.median" => self.eval_array_numeric(args, ArrayNumericMode::Median),
            "array.mode" => self.eval_array_numeric(args, ArrayNumericMode::Mode),
            "array.percentile_nearest_rank" => {
                self.eval_array_percentile(args, ArrayPercentileMode::NearestRank)
            }
            "array.percentile_linear_interpolation" => {
                self.eval_array_percentile(args, ArrayPercentileMode::LinearInterpolation)
            }
            "array.percentrank" => self.eval_array_percentrank(args),
            "array.covariance" => self.eval_array_covariance(args),
            "array.standardize" => self.eval_array_standardize(args),
            "array.variance" => self.eval_array_variance(args, ArrayVarianceMode::Variance),
            "array.stdev" => self.eval_array_variance(args, ArrayVarianceMode::Stdev),
            "array.sort" => self.eval_array_sort(args),
            "array.sort_indices" => self.eval_array_sort_indices(args),
            "array.reverse" => self.eval_array_reverse(args),
            "array.join" => self.eval_array_join(args),
            "array.clear" => self.eval_array_clear(args),
            _ => Err(RuntimeError {
                message: format!("unsupported runtime call `{callee}`"),
            }),
        }
    }

    fn eval_int_cast(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        Ok(match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => PineValue::Int(value),
            PineValue::Float(value) if value.is_finite() => PineValue::Int(value.trunc() as i64),
            PineValue::Bool(value) => PineValue::Int(i64::from(value)),
            PineValue::Na => PineValue::Na,
            _ => PineValue::Na,
        })
    }

    fn eval_float_cast(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        Ok(match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => PineValue::Float(value as f64),
            PineValue::Float(value) => finite_float_or_na(value),
            PineValue::Bool(value) => PineValue::Float(if value { 1.0 } else { 0.0 }),
            PineValue::Na => PineValue::Na,
            _ => PineValue::Na,
        })
    }

    fn eval_bool_cast(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        Ok(match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => PineValue::Bool(value != 0),
            PineValue::Float(value) => PineValue::Bool(value != 0.0 && !value.is_nan()),
            PineValue::Bool(value) => PineValue::Bool(value),
            PineValue::Na => PineValue::Bool(false),
            _ => PineValue::Bool(false),
        })
    }

    fn eval_string_cast(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        let result = match value {
            PineValue::Int(value) => value.to_string(),
            PineValue::Float(value) => format_number(value, "#.########"),
            PineValue::Bool(value) => value.to_string(),
            PineValue::String(value) => value,
            PineValue::Na => return Ok(PineValue::Na),
            _ => return Ok(PineValue::Na),
        };
        self.string_value_or_error(result, "string")
    }

    fn eval_color_cast(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        Ok(match self.eval_expr(&args[0].value)? {
            PineValue::Color(value) => PineValue::Color(value),
            PineValue::Na => PineValue::Na,
            _ => PineValue::Na,
        })
    }

    fn eval_fixnan(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        if value.is_na() {
            Ok(self
                .call_state
                .get(&call_site_id)
                .cloned()
                .unwrap_or(PineValue::Na))
        } else {
            self.call_state.insert(call_site_id, value.clone());
            Ok(value)
        }
    }

    fn eval_array_new_float(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(size) = self.eval_array_new_size(args, "array.new_float")? else {
            return Ok(PineValue::Na);
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            self.eval_array_value(&value_arg.value, ArrayElementKind::Float)?
        } else {
            PineValue::Na
        };

        Ok(self.new_array(ArrayElementKind::Float, size, initial_value))
    }

    fn eval_array_new_int(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(size) = self.eval_array_new_size(args, "array.new_int")? else {
            return Ok(PineValue::Na);
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            self.eval_array_value(&value_arg.value, ArrayElementKind::Int)?
        } else {
            PineValue::Na
        };

        Ok(self.new_array(ArrayElementKind::Int, size, initial_value))
    }

    fn eval_array_new_bool(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(size) = self.eval_array_new_size(args, "array.new_bool")? else {
            return Ok(PineValue::Na);
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            self.eval_array_value(&value_arg.value, ArrayElementKind::Bool)?
        } else {
            PineValue::Na
        };

        Ok(self.new_array(ArrayElementKind::Bool, size, initial_value))
    }

    fn eval_array_new_string(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(size) = self.eval_array_new_size(args, "array.new_string")? else {
            return Ok(PineValue::Na);
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            self.eval_array_value(&value_arg.value, ArrayElementKind::String)?
        } else {
            PineValue::Na
        };

        Ok(self.new_array(ArrayElementKind::String, size, initial_value))
    }

    fn eval_array_new_color(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(size) = self.eval_array_new_size(args, "array.new_color")? else {
            return Ok(PineValue::Na);
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            self.eval_array_value(&value_arg.value, ArrayElementKind::Color)?
        } else {
            PineValue::Na
        };

        Ok(self.new_array(ArrayElementKind::Color, size, initial_value))
    }

    fn eval_array_from(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        if args.len() > MAX_ARRAY_ELEMENTS {
            return Err(RuntimeError {
                message: format!("array.from cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
            });
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(&arg.value)?);
        }

        let Some(kind) = infer_array_from_kind(&values) else {
            return Ok(PineValue::Na);
        };
        for value in &mut values {
            if matches!(kind, ArrayElementKind::Float) {
                let int_value = match value {
                    PineValue::Int(int_value) => Some(*int_value),
                    _ => None,
                };
                if let Some(int_value) = int_value {
                    *value = PineValue::Float(int_value as f64);
                }
            }
        }
        Ok(self.new_array_from_values(kind, values))
    }

    fn eval_array_new_size(
        &mut self,
        args: &[HirCallArg],
        function_name: &str,
    ) -> Result<Option<usize>, RuntimeError> {
        if let Some(size_arg) = args.first() {
            let Some(size) = self.eval_expr(&size_arg.value)?.as_i64() else {
                return Ok(None);
            };
            if size < 0 {
                return Err(RuntimeError {
                    message: format!("{function_name} size cannot be negative"),
                });
            }
            let size = size as usize;
            if size > MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!(
                        "{function_name} size cannot exceed {MAX_ARRAY_ELEMENTS} elements"
                    ),
                });
            }
            Ok(Some(size))
        } else {
            Ok(Some(0))
        }
    }

    fn new_array(
        &mut self,
        kind: ArrayElementKind,
        size: usize,
        initial_value: PineValue,
    ) -> PineValue {
        let id = self.next_array_id;
        self.next_array_id += 1;
        self.array_store.insert(id, vec![initial_value; size]);
        self.array_kinds.insert(id, kind);
        PineValue::Array(id)
    }

    fn new_array_from_values(
        &mut self,
        kind: ArrayElementKind,
        values: Vec<PineValue>,
    ) -> PineValue {
        let id = self.next_array_id;
        self.next_array_id += 1;
        self.array_store.insert(id, values);
        self.array_kinds.insert(id, kind);
        PineValue::Array(id)
    }

    fn eval_array_size(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        Ok(PineValue::Int(values.len() as i64))
    }

    fn eval_array_push(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.push cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.push(value);
        }
        Ok(PineValue::Void)
    }

    fn eval_array_get(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| {
                normalize_array_index(index, values.len()).and_then(|index| values.get(index))
            })
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_set(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[2].value, kind)?;
        if let Some(slot) = self.array_store.get_mut(&id).and_then(|values| {
            normalize_array_index(index, values.len()).and_then(|index| values.get_mut(index))
        }) {
            *slot = value;
        }
        Ok(PineValue::Void)
    }

    fn eval_array_insert(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[2].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[2].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            let Some(index) = normalize_array_insert_index(index, values.len()) else {
                return Ok(PineValue::Void);
            };
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.insert cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.insert(index, value);
        }
        Ok(PineValue::Void)
    }

    fn eval_array_pop(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(Vec::pop)
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_remove(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(|values| {
                normalize_array_index(index, values.len()).map(|index| values.remove(index))
            })
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_shift(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get_mut(&id)
            .and_then(|values| {
                if values.is_empty() {
                    None
                } else {
                    Some(values.remove(0))
                }
            })
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_unshift(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        if let Some(values) = self.array_store.get_mut(&id) {
            if values.len() >= MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!("array.unshift cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
                });
            }
            values.insert(0, value);
        }
        Ok(PineValue::Void)
    }

    fn eval_array_fill(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            if let Some(index_from) = args.get(2) {
                let _ = self.eval_expr(&index_from.value)?;
            }
            if let Some(index_to) = args.get(3) {
                let _ = self.eval_expr(&index_to.value)?;
            }
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            if let Some(index_from) = args.get(2) {
                let _ = self.eval_expr(&index_from.value)?;
            }
            if let Some(index_to) = args.get(3) {
                let _ = self.eval_expr(&index_to.value)?;
            }
            return Ok(PineValue::Void);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        let index_from = if let Some(index_from) = args.get(2) {
            self.eval_expr(&index_from.value)?.as_i64()
        } else {
            Some(0)
        };
        let Some(index_from) = index_from else {
            return Ok(PineValue::Void);
        };
        let index_to = if let Some(index_to) = args.get(3) {
            self.eval_expr(&index_to.value)?.as_i64()
        } else {
            self.array_store.get(&id).map(|values| values.len() as i64)
        };
        let Some(index_to) = index_to else {
            return Ok(PineValue::Void);
        };
        if index_from < 0 || index_to < 0 || index_from > index_to {
            return Ok(PineValue::Void);
        }
        let index_from = index_from as usize;
        let index_to = index_to as usize;
        if let Some(values) = self.array_store.get_mut(&id) {
            if index_to > values.len() {
                return Ok(PineValue::Void);
            }
            for item in &mut values[index_from..index_to] {
                *item = value.clone();
            }
        }
        Ok(PineValue::Void)
    }

    fn eval_array_first(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| values.first())
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_last(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| values.last())
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_copy(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id).cloned() else {
            return Ok(PineValue::Na);
        };
        Ok(self.new_array_from_values(kind, values))
    }

    fn eval_array_slice(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index_from = self.eval_expr(&args[1].value)?.as_i64();
        let index_to = self.eval_expr(&args[2].value)?.as_i64();
        let (PineValue::Array(id), Some(index_from), Some(index_to)) = (id, index_from, index_to)
        else {
            return Ok(PineValue::Na);
        };
        if index_from < 0 || index_to < 0 || index_from > index_to {
            return Ok(PineValue::Na);
        }

        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let index_from = index_from as usize;
        let index_to = index_to as usize;
        if index_to > values.len() {
            return Ok(PineValue::Na);
        }
        let values = values[index_from..index_to].to_vec();

        Ok(self.new_array_from_values(kind, values))
    }

    fn eval_array_concat(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let target = self.eval_expr(&args[0].value)?;
        let source = self.eval_expr(&args[1].value)?;
        let (PineValue::Array(target_id), PineValue::Array(source_id)) = (target, source) else {
            return Ok(PineValue::Na);
        };
        let Some(target_kind) = self.array_kinds.get(&target_id).copied() else {
            return Ok(PineValue::Na);
        };
        let Some(source_kind) = self.array_kinds.get(&source_id).copied() else {
            return Ok(PineValue::Na);
        };
        if target_kind != source_kind {
            return Ok(PineValue::Na);
        }
        let Some(source_values) = self.array_store.get(&source_id).cloned() else {
            return Ok(PineValue::Na);
        };
        let Some(target_values) = self.array_store.get_mut(&target_id) else {
            return Ok(PineValue::Na);
        };
        if target_values.len() + source_values.len() > MAX_ARRAY_ELEMENTS {
            return Err(RuntimeError {
                message: format!("array.concat cannot exceed {MAX_ARRAY_ELEMENTS} elements"),
            });
        }
        target_values.extend(source_values);
        Ok(PineValue::Array(target_id))
    }

    fn eval_array_includes(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let index = self.eval_array_search(args, ArraySearchMode::First)?;
        Ok(PineValue::Bool(index.is_some()))
    }

    fn eval_array_truth(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayTruthMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::Bool
        ) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let result = match mode {
            ArrayTruthMode::Every => values.iter().all(array_truthy_value),
            ArrayTruthMode::Some => values.iter().any(array_truthy_value),
        };
        Ok(PineValue::Bool(result))
    }

    fn eval_array_indexof(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let index = self
            .eval_array_search(args, ArraySearchMode::First)?
            .map_or(-1, |index| index as i64);
        Ok(PineValue::Int(index))
    }

    fn eval_array_lastindexof(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let index = self
            .eval_array_search(args, ArraySearchMode::Last)?
            .map_or(-1, |index| index as i64);
        Ok(PineValue::Int(index))
    }

    fn eval_array_search(
        &mut self,
        args: &[HirCallArg],
        mode: ArraySearchMode,
    ) -> Result<Option<usize>, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(None);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(None);
        };
        let value = self.eval_array_value(&args[1].value, kind)?;
        let Some(values) = self.array_store.get(&id) else {
            return Ok(None);
        };
        let index = match mode {
            ArraySearchMode::First => values.iter().position(|item| values_equal(item, &value)),
            ArraySearchMode::Last => values.iter().rposition(|item| values_equal(item, &value)),
        };
        Ok(index)
    }

    fn eval_array_binary_search(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayBinarySearchMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            let _ = self.eval_expr(&args[1].value)?;
            return Ok(PineValue::Int(-1));
        }
        let value = self.eval_array_value(&args[1].value, kind)?;
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Int(-1));
        };
        if values.is_empty() {
            return Ok(PineValue::Int(-1));
        }

        let lower = array_numeric_lower_bound(values, &value);
        let exact_match =
            lower < values.len() && compare_array_numeric_values(&values[lower], &value).is_eq();
        let index = match mode {
            ArrayBinarySearchMode::Exact => exact_match.then_some(lower),
            ArrayBinarySearchMode::Leftmost => {
                if exact_match || lower == 0 {
                    Some(lower.min(values.len() - 1))
                } else {
                    Some(lower - 1)
                }
            }
            ArrayBinarySearchMode::Rightmost => {
                if exact_match {
                    Some(array_numeric_upper_bound(values, &value) - 1)
                } else {
                    Some(lower.min(values.len() - 1))
                }
            }
        }
        .map_or(-1, |index| index as i64);

        Ok(PineValue::Int(index))
    }

    fn eval_array_numeric(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayNumericMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        match mode {
            ArrayNumericMode::Min | ArrayNumericMode::Max => {
                let mut current: Option<f64> = None;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    current = Some(match (mode, current) {
                        (_, None) => value,
                        (ArrayNumericMode::Min, Some(current)) => current.min(value),
                        (ArrayNumericMode::Max, Some(current)) => current.max(value),
                        _ => unreachable!("only min/max modes are handled here"),
                    });
                }
                let Some(current) = current else {
                    return Ok(PineValue::Na);
                };
                Ok(array_numeric_result(kind, current))
            }
            ArrayNumericMode::Range => {
                let mut min: Option<f64> = None;
                let mut max: Option<f64> = None;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    min = Some(min.map_or(value, |current| current.min(value)));
                    max = Some(max.map_or(value, |current| current.max(value)));
                }
                let (Some(min), Some(max)) = (min, max) else {
                    return Ok(PineValue::Na);
                };
                Ok(array_numeric_result(kind, max - min))
            }
            ArrayNumericMode::Sum | ArrayNumericMode::Avg => {
                let mut total = 0.0;
                let mut count = 0_usize;
                for value in values.iter().filter_map(PineValue::as_f64) {
                    total += value;
                    count += 1;
                }
                if count == 0 {
                    return Ok(PineValue::Na);
                }
                if matches!(mode, ArrayNumericMode::Avg) {
                    Ok(finite_float_or_na(total / count as f64))
                } else {
                    Ok(array_numeric_result(kind, total))
                }
            }
            ArrayNumericMode::Median => {
                let mut numeric_values: Vec<_> =
                    values.iter().filter_map(PineValue::as_f64).collect();
                if numeric_values.is_empty() {
                    return Ok(PineValue::Na);
                }
                numeric_values
                    .sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
                let middle = numeric_values.len() / 2;
                let median = if numeric_values.len() % 2 == 0 {
                    (numeric_values[middle - 1] + numeric_values[middle]) / 2.0
                } else {
                    numeric_values[middle]
                };
                Ok(array_numeric_result(kind, median))
            }
            ArrayNumericMode::Mode => {
                let mut numeric_values: Vec<_> =
                    values.iter().filter_map(PineValue::as_f64).collect();
                if numeric_values.is_empty() {
                    return Ok(PineValue::Na);
                }
                numeric_values
                    .sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

                let mut best_value = numeric_values[0];
                let mut best_count = 0_usize;
                let mut current_value = numeric_values[0];
                let mut current_count = 0_usize;
                for value in numeric_values {
                    if (value - current_value).abs() < f64::EPSILON {
                        current_count += 1;
                    } else {
                        if current_count > best_count {
                            best_value = current_value;
                            best_count = current_count;
                        }
                        current_value = value;
                        current_count = 1;
                    }
                }
                if current_count > best_count {
                    best_value = current_value;
                    best_count = current_count;
                }
                if best_count < 2 {
                    return Ok(PineValue::Na);
                }
                Ok(array_numeric_result(kind, best_value))
            }
        }
    }

    fn eval_array_abs(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let values = values
            .iter()
            .map(|value| match (kind, value) {
                (_, PineValue::Na) => PineValue::Na,
                (ArrayElementKind::Int, PineValue::Int(value)) => value
                    .checked_abs()
                    .map(PineValue::Int)
                    .unwrap_or(PineValue::Na),
                (ArrayElementKind::Float, PineValue::Float(value)) => {
                    finite_float_or_na(value.abs())
                }
                (ArrayElementKind::Float, PineValue::Int(value)) => {
                    finite_float_or_na((*value as f64).abs())
                }
                _ => PineValue::Na,
            })
            .collect();

        Ok(self.new_array_from_values(kind, values))
    }

    fn eval_array_percentile(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayPercentileMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let percentage = self.eval_expr(&args[1].value)?.as_f64();
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(percentage) = percentage else {
            return Ok(PineValue::Na);
        };
        if !(0.0..=100.0).contains(&percentage) {
            return Ok(PineValue::Na);
        }
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let mut numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        if numeric_values.is_empty() {
            return Ok(PineValue::Na);
        }
        numeric_values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

        match mode {
            ArrayPercentileMode::NearestRank => {
                let rank = ((percentage / 100.0) * numeric_values.len() as f64).ceil();
                let index = (rank as usize)
                    .saturating_sub(1)
                    .min(numeric_values.len() - 1);
                Ok(array_numeric_result(kind, numeric_values[index]))
            }
            ArrayPercentileMode::LinearInterpolation => {
                if numeric_values.len() == 1 {
                    return Ok(finite_float_or_na(numeric_values[0]));
                }
                let rank = (percentage / 100.0) * (numeric_values.len() - 1) as f64;
                let lower = rank.floor() as usize;
                let upper = rank.ceil() as usize;
                let fraction = rank - lower as f64;
                let value = numeric_values[lower]
                    + (numeric_values[upper] - numeric_values[lower]) * fraction;
                Ok(finite_float_or_na(value))
            }
        }
    }

    fn eval_array_percentrank(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(index) = index else {
            return Ok(PineValue::Na);
        };
        if index < 0 {
            return Ok(PineValue::Na);
        }
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let Some(target) = values.get(index as usize).and_then(PineValue::as_f64) else {
            return Ok(PineValue::Na);
        };
        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        if numeric_values.is_empty() {
            return Ok(PineValue::Na);
        }
        let count = numeric_values
            .iter()
            .filter(|value| **value <= target || (**value - target).abs() < f64::EPSILON)
            .count();
        Ok(finite_float_or_na(
            count as f64 / numeric_values.len() as f64 * 100.0,
        ))
    }

    fn eval_array_standardize(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        let count = numeric_values.len();
        if count == 0 {
            return Ok(self.new_array_from_values(ArrayElementKind::Float, Vec::new()));
        }

        let mean = numeric_values.iter().sum::<f64>() / count as f64;
        let variance = numeric_values
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        let stdev = variance.sqrt();

        let values = values
            .iter()
            .map(|value| {
                let Some(value) = value.as_f64() else {
                    return PineValue::Na;
                };
                if stdev == 0.0 || !stdev.is_finite() {
                    PineValue::Na
                } else {
                    finite_float_or_na((value - mean) / stdev)
                }
            })
            .collect();

        Ok(self.new_array_from_values(ArrayElementKind::Float, values))
    }

    fn eval_array_covariance(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id1 = self.eval_expr(&args[0].value)?;
        let id2 = self.eval_expr(&args[1].value)?;
        let biased = match args.get(2) {
            Some(arg) => matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true)),
            None => true,
        };
        let (PineValue::Array(id1), PineValue::Array(id2)) = (id1, id2) else {
            return Ok(PineValue::Na);
        };
        let (Some(kind1), Some(kind2)) = (
            self.array_kinds.get(&id1).copied(),
            self.array_kinds.get(&id2).copied(),
        ) else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind1, ArrayElementKind::Float | ArrayElementKind::Int)
            || !matches!(kind2, ArrayElementKind::Float | ArrayElementKind::Int)
        {
            return Ok(PineValue::Na);
        }
        let (Some(values1), Some(values2)) =
            (self.array_store.get(&id1), self.array_store.get(&id2))
        else {
            return Ok(PineValue::Na);
        };
        if values1.len() != values2.len() {
            return Ok(PineValue::Na);
        }

        let pairs: Vec<_> = values1
            .iter()
            .zip(values2)
            .filter_map(|(left, right)| Some((left.as_f64()?, right.as_f64()?)))
            .collect();
        let count = pairs.len();
        if count == 0 || (!biased && count < 2) {
            return Ok(PineValue::Na);
        }

        let mean1 = pairs.iter().map(|(left, _)| left).sum::<f64>() / count as f64;
        let mean2 = pairs.iter().map(|(_, right)| right).sum::<f64>() / count as f64;
        let covariance_sum = pairs
            .iter()
            .map(|(left, right)| (left - mean1) * (right - mean2))
            .sum::<f64>();
        let denominator = if biased { count } else { count - 1 };
        Ok(finite_float_or_na(covariance_sum / denominator as f64))
    }

    fn eval_array_variance(
        &mut self,
        args: &[HirCallArg],
        mode: ArrayVarianceMode,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let biased = match args.get(1) {
            Some(arg) => matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true)),
            None => true,
        };
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(kind, ArrayElementKind::Float | ArrayElementKind::Int) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let numeric_values: Vec<_> = values.iter().filter_map(PineValue::as_f64).collect();
        let count = numeric_values.len();
        if count == 0 || (!biased && count < 2) {
            return Ok(PineValue::Na);
        }

        let mean = numeric_values.iter().sum::<f64>() / count as f64;
        let squared_diff_sum = numeric_values
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>();
        let denominator = if biased { count } else { count - 1 };
        let variance = squared_diff_sum / denominator as f64;
        let result = match mode {
            ArrayVarianceMode::Variance => variance,
            ArrayVarianceMode::Stdev => variance.sqrt(),
        };

        Ok(finite_float_or_na(result))
    }

    fn eval_array_sort(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let descending = self.eval_array_sort_descending(args, "array.sort")?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Void);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::String
        ) {
            return Ok(PineValue::Void);
        }
        if let Some(values) = self.array_store.get_mut(&id) {
            values.sort_by(|left, right| compare_array_sort_values(kind, left, right, descending));
        }
        Ok(PineValue::Void)
    }

    fn eval_array_sort_indices(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let descending = self.eval_array_sort_descending(args, "array.sort_indices")?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(kind) = self.array_kinds.get(&id).copied() else {
            return Ok(PineValue::Na);
        };
        if !matches!(
            kind,
            ArrayElementKind::Float | ArrayElementKind::Int | ArrayElementKind::String
        ) {
            return Ok(PineValue::Na);
        }
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };

        let mut indices = (0..values.len()).collect::<Vec<_>>();
        indices.sort_by(|left, right| {
            compare_array_sort_values(kind, &values[*left], &values[*right], descending)
                .then_with(|| left.cmp(right))
        });
        let values = indices
            .into_iter()
            .map(|index| PineValue::Int(index as i64))
            .collect();

        Ok(self.new_array_from_values(ArrayElementKind::Int, values))
    }

    fn eval_array_sort_descending(
        &mut self,
        args: &[HirCallArg],
        callee: &str,
    ) -> Result<bool, RuntimeError> {
        match args.get(1) {
            Some(order) => match self.eval_expr(&order.value)? {
                PineValue::String(order) if order == "order.descending" => Ok(true),
                PineValue::String(order) if order == "order.ascending" => Ok(false),
                PineValue::String(order) => Err(RuntimeError {
                    message: format!("unsupported {callee} order `{order}`"),
                }),
                _ => Ok(false),
            },
            None => Ok(false),
        }
    }

    fn eval_array_reverse(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        if let Some(values) = self.array_store.get_mut(&id) {
            values.reverse();
        }
        Ok(PineValue::Void)
    }

    fn eval_array_join(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            if let Some(separator) = args.get(1) {
                let _ = self.eval_expr(&separator.value)?;
            }
            return Ok(PineValue::Na);
        };
        let separator = if let Some(separator) = args.get(1) {
            match self.eval_expr(&separator.value)? {
                PineValue::String(separator) => separator,
                PineValue::Na => ",".to_owned(),
                _ => return Ok(PineValue::Na),
            }
        } else {
            ",".to_owned()
        };
        let Some(values) = self.array_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let mut result = String::new();
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                result.push_str(&separator);
            }
            result.push_str(&stringify_array_join_element(value));
        }
        self.string_value_or_error(result, "array.join")
    }

    fn eval_array_clear(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
        if let Some(values) = self.array_store.get_mut(&id) {
            values.clear();
        }
        Ok(PineValue::Void)
    }

    fn eval_array_value(
        &mut self,
        expr: &HirExpr,
        kind: ArrayElementKind,
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(expr)?;
        Ok(match (kind, value) {
            (ArrayElementKind::Float, PineValue::Int(value)) => PineValue::Float(value as f64),
            (ArrayElementKind::Float, PineValue::Float(value)) => PineValue::Float(value),
            (ArrayElementKind::Int, PineValue::Int(value)) => PineValue::Int(value),
            (ArrayElementKind::Bool, PineValue::Bool(value)) => PineValue::Bool(value),
            (ArrayElementKind::String, PineValue::String(value)) => PineValue::String(value),
            (ArrayElementKind::Color, PineValue::Color(value)) => PineValue::Color(value),
            (_, PineValue::Na) => PineValue::Na,
            _ => PineValue::Na,
        })
    }

    fn eval_sma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(PineValue::Float(window.mean(length)))
    }

    fn eval_bb(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let mult = self.eval_expr(&args[2].value)?.as_f64().unwrap_or(0.0);
        if length <= 0 {
            return Ok(PineValue::Tuple(vec![
                PineValue::Na,
                PineValue::Na,
                PineValue::Na,
            ]));
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Tuple(vec![
                PineValue::Na,
                PineValue::Na,
                PineValue::Na,
            ]));
        }

        let basis = window.mean(length);
        let variance = window.variance(length, true);
        let dev = mult * variance.sqrt();

        Ok(PineValue::Tuple(vec![
            PineValue::Float(basis),
            PineValue::Float(basis + dev),
            PineValue::Float(basis - dev),
        ]))
    }

    fn eval_bbw(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let mult = self.eval_expr(&args[2].value)?.as_f64().unwrap_or(0.0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let basis = window.mean(length);
        if basis == 0.0 {
            return Ok(PineValue::Na);
        }
        let dev = mult * window.variance(length, true).sqrt();

        Ok(finite_float_or_na((2.0 * dev) / basis))
    }

    fn eval_kc(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((basis, range_ema, mult)) = self.eval_kc_components(call_site_id, args)? else {
            return Ok(three_na_tuple());
        };

        Ok(PineValue::Tuple(vec![
            finite_float_or_na(basis),
            finite_float_or_na(basis + range_ema * mult),
            finite_float_or_na(basis - range_ema * mult),
        ]))
    }

    fn eval_kcw(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((basis, range_ema, mult)) = self.eval_kc_components(call_site_id, args)? else {
            return Ok(PineValue::Na);
        };
        if basis == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na((2.0 * range_ema * mult) / basis))
    }

    fn eval_kc_components(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<Option<(f64, f64, f64)>, RuntimeError> {
        let Some(source) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(None);
        };
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let Some(mult) = self.eval_expr(&args[2].value)?.as_f64() else {
            return Ok(None);
        };
        let use_true_range = if let Some(arg) = args.get(3) {
            match self.eval_expr(&arg.value)? {
                PineValue::Bool(value) => value,
                PineValue::Na => true,
                _ => false,
            }
        } else {
            true
        };
        if length <= 0 {
            return Ok(None);
        }

        let span = if use_true_range {
            self.true_range(true).as_f64()
        } else {
            match (
                self.current_builtin_f64("high"),
                self.current_builtin_f64("low"),
            ) {
                (Some(high), Some(low)) => Some(high - low),
                _ => None,
            }
        };
        let Some(span) = span else {
            return Ok(None);
        };

        let previous = kc_state(self.call_state.get(&call_site_id));
        let basis = ema_next(previous.map(|state| state.0), source, length);
        let range_ema = ema_next(previous.map(|state| state.1), span, length);
        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![PineValue::Float(basis), PineValue::Float(range_ema)]),
        );

        Ok(Some((basis, range_ema, mult)))
    }

    fn eval_pivot(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let (source, leftbars, rightbars) = self.eval_pivot_args(args, mode)?;
        if leftbars < 0 || rightbars < 0 {
            return Ok(PineValue::Na);
        }

        let leftbars = leftbars as usize;
        let rightbars = rightbars as usize;
        let length = leftbars + rightbars + 1;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let candidate_index = length - 1 - rightbars;
        let candidate = window.values.get(candidate_index).and_then(|value| *value);
        let Some(candidate) = candidate else {
            return Ok(PineValue::Na);
        };

        let is_pivot = window
            .values
            .iter()
            .flatten()
            .enumerate()
            .all(|(index, value)| {
                index == candidate_index
                    || match mode {
                        WindowExtreme::Highest => candidate > *value,
                        WindowExtreme::Lowest => candidate < *value,
                    }
            });
        if !is_pivot {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(candidate))
    }

    fn eval_pivot_args(
        &mut self,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<(PineValue, i64, i64), RuntimeError> {
        if args.len() == 2 {
            let leftbars = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(-1);
            let rightbars = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(-1);
            let source_name = match mode {
                WindowExtreme::Highest => "high",
                WindowExtreme::Lowest => "low",
            };
            let source = self
                .current_builtin_f64(source_name)
                .map_or(PineValue::Na, PineValue::Float);
            return Ok((source, leftbars, rightbars));
        }

        let source = self.eval_expr(&args[0].value)?;
        let leftbars = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(-1);
        let rightbars = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(-1);
        Ok((source, leftbars, rightbars))
    }

    fn eval_pivot_point_levels(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let type_arg = pivot_point_arg(args, 0, "type").ok_or_else(|| RuntimeError {
            message: "ta.pivot_point_levels missing type argument".to_owned(),
        })?;
        let anchor_arg = pivot_point_arg(args, 1, "anchor").ok_or_else(|| RuntimeError {
            message: "ta.pivot_point_levels missing anchor argument".to_owned(),
        })?;
        let PineValue::String(type_name) = self.eval_expr(type_arg)? else {
            return Ok(self.new_array_from_values(ArrayElementKind::Float, pivot_na_levels()));
        };
        let anchor = matches!(self.eval_expr(anchor_arg)?, PineValue::Bool(true));
        let developing = if let Some(arg) = pivot_point_arg(args, 2, "developing") {
            matches!(self.eval_expr(arg)?, PineValue::Bool(true))
        } else {
            false
        };

        let (Some(open), Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("open"),
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(self.new_array_from_values(ArrayElementKind::Float, pivot_na_levels()));
        };
        if !open.is_finite() || !high.is_finite() || !low.is_finite() || !close.is_finite() {
            return Ok(self.new_array_from_values(ArrayElementKind::Float, pivot_na_levels()));
        }

        let state = self.pivot_point_state.entry(call_site_id).or_default();
        if anchor {
            if let Some(previous) = state.current {
                state.active_levels = Some(pivot_point_levels(&type_name, previous, open));
            }
            state.current = Some(PivotPointPeriod::new(open, high, low, close));
        } else if let Some(current) = &mut state.current {
            current.update(high, low, close);
        } else {
            state.current = Some(PivotPointPeriod::new(open, high, low, close));
        }

        let levels = if developing {
            state
                .current
                .map(|current| pivot_point_levels(&type_name, current, current.open))
        } else {
            state.active_levels.clone()
        }
        .unwrap_or_else(pivot_na_levels);

        Ok(self.new_array_from_values(ArrayElementKind::Float, levels))
    }

    fn eval_cum(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let Some(source) = source.as_f64() else {
            self.call_state.insert(call_site_id, PineValue::Na);
            return Ok(PineValue::Na);
        };

        let value = self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_f64)
            .unwrap_or(0.0)
            + source;
        let value = PineValue::Float(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    fn eval_all_time_extreme(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let Some(source) = source.as_f64() else {
            return Ok(self
                .call_state
                .get(&call_site_id)
                .cloned()
                .unwrap_or(PineValue::Na));
        };

        let value = match self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_f64)
        {
            Some(previous) => match mode {
                WindowExtreme::Highest => previous.max(source),
                WindowExtreme::Lowest => previous.min(source),
            },
            None => source,
        };
        let value = finite_float_or_na(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    fn eval_stdev(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match self.eval_window_variance(call_site_id, args)? {
            PineValue::Float(value) => Ok(finite_float_or_na(value.sqrt())),
            value => Ok(value),
        }
    }

    fn eval_variance(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_window_variance(call_site_id, args)
    }

    fn eval_range(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(window.range().map_or(PineValue::Na, PineValue::Float))
    }

    fn eval_dev(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.mean_absolute_deviation(length)))
    }

    fn eval_cci(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(current) = source.as_f64() else {
            self.update_rolling_window(call_site_id, source, length as usize);
            return Ok(PineValue::Na);
        };

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, PineValue::Float(current), length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let deviation = window.mean_absolute_deviation(length);
        if deviation == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            (current - window.mean(length)) / (0.015 * deviation),
        ))
    }

    fn eval_cog(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) || window.sum == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.center_of_gravity(length)))
    }

    fn eval_vwma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let Some(source) = source.as_f64() else {
            self.update_rolling_window_key(
                RollingWindowKey::VwmaWeighted(call_site_id),
                None,
                length,
            );
            self.update_rolling_window_key(
                RollingWindowKey::VwmaVolume(call_site_id),
                None,
                length,
            );
            return Ok(PineValue::Na);
        };
        let Some(volume) = self.current_builtin_f64("volume") else {
            self.update_rolling_window_key(
                RollingWindowKey::VwmaWeighted(call_site_id),
                None,
                length,
            );
            self.update_rolling_window_key(
                RollingWindowKey::VwmaVolume(call_site_id),
                None,
                length,
            );
            return Ok(PineValue::Na);
        };

        self.update_rolling_window_key(
            RollingWindowKey::VwmaWeighted(call_site_id),
            Some(source * volume),
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::VwmaVolume(call_site_id),
            Some(volume),
            length,
        );

        let weighted = self
            .rolling_windows
            .get(&RollingWindowKey::VwmaWeighted(call_site_id));
        let volumes = self
            .rolling_windows
            .get(&RollingWindowKey::VwmaVolume(call_site_id));
        let (Some(weighted), Some(volumes)) = (weighted, volumes) else {
            return Ok(PineValue::Na);
        };
        if !weighted.is_ready(length) || !volumes.is_ready(length) || volumes.sum == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(weighted.sum / volumes.sum))
    }

    fn eval_mfi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let Some(source) = source.as_f64() else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };
        let Some(volume) = self.current_builtin_f64("volume") else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };
        let Some(series_id) = args[0].value.series_id else {
            self.update_mfi_windows(call_site_id, None, None, length);
            return Ok(PineValue::Na);
        };

        let (positive_flow, negative_flow) = match self.series_store.read(series_id, 1).as_f64() {
            Some(previous) if source > previous => (Some(source * volume), Some(0.0)),
            Some(previous) if source < previous => (Some(0.0), Some(source * volume)),
            Some(_) | None => (Some(0.0), Some(0.0)),
        };
        self.update_mfi_windows(call_site_id, positive_flow, negative_flow, length);

        let positive_window = self
            .rolling_windows
            .get(&RollingWindowKey::MfiPositive(call_site_id));
        let negative_window = self
            .rolling_windows
            .get(&RollingWindowKey::MfiNegative(call_site_id));
        let (Some(positive_window), Some(negative_window)) = (positive_window, negative_window)
        else {
            return Ok(PineValue::Na);
        };
        if !positive_window.is_ready(length) || !negative_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let positive_sum = positive_window.sum;
        let negative_sum = negative_window.sum;
        if positive_sum == 0.0 && negative_sum == 0.0 {
            return Ok(PineValue::Na);
        }
        if negative_sum == 0.0 {
            return Ok(PineValue::Float(100.0));
        }

        Ok(finite_float_or_na(
            100.0 - 100.0 / (1.0 + positive_sum / negative_sum),
        ))
    }

    fn eval_vwap_source(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let has_bands = vwap_arg(args, 2, "stdev_mult").is_some();
        let source_arg = vwap_arg(args, 0, "source").ok_or_else(|| RuntimeError {
            message: "ta.vwap missing source argument".to_owned(),
        })?;
        let source = self.eval_expr(source_arg)?;
        let anchor = if let Some(arg) = vwap_arg(args, 1, "anchor") {
            matches!(self.eval_expr(arg)?, PineValue::Bool(true))
        } else {
            false
        };
        let stdev_mult = if let Some(arg) = vwap_arg(args, 2, "stdev_mult") {
            let Some(mult) = self.eval_expr(arg)?.as_f64() else {
                self.vwap_call_state.remove(&call_site_id);
                return Ok(vwap_result_na(has_bands));
            };
            Some(mult)
        } else {
            None
        };
        let (Some(source), Some(volume)) = (source.as_f64(), self.current_builtin_f64("volume"))
        else {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        };
        let weighted = source * volume;
        let weighted_square = source * source * volume;
        if !source.is_finite()
            || !volume.is_finite()
            || !weighted.is_finite()
            || !weighted_square.is_finite()
        {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        }
        if let Some(mult) = stdev_mult
            && !mult.is_finite()
        {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(vwap_result_na(has_bands));
        }

        let state = self.vwap_call_state.entry(call_site_id).or_default();
        if anchor {
            *state = VwapState::default();
        }
        state.weighted_sum += weighted;
        state.weighted_square_sum += weighted_square;
        state.volume_sum += volume;
        if state.volume_sum == 0.0
            || !state.weighted_sum.is_finite()
            || !state.weighted_square_sum.is_finite()
            || !state.volume_sum.is_finite()
        {
            return Ok(vwap_result_na(has_bands));
        }

        let vwap = state.weighted_sum / state.volume_sum;
        let value = finite_float_or_na(vwap);
        let Some(mult) = stdev_mult else {
            return Ok(value);
        };
        let variance = (state.weighted_square_sum / state.volume_sum) - vwap * vwap;
        let deviation = variance.max(0.0).sqrt();
        let band = deviation * mult;
        Ok(PineValue::Tuple(vec![
            value,
            finite_float_or_na(vwap + band),
            finite_float_or_na(vwap - band),
        ]))
    }

    fn eval_wma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.weighted_mean(length)))
    }

    fn eval_correlation(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let left = self.eval_expr(&args[0].value)?;
        let right = self.eval_expr(&args[1].value)?;
        let length = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let left = left.as_f64();
        let right = right.as_f64();
        let product = left.zip(right).map(|(left, right)| left * right);
        self.update_rolling_window_key(
            RollingWindowKey::CorrelationLeft(call_site_id),
            left,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CorrelationRight(call_site_id),
            right,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CorrelationProduct(call_site_id),
            product,
            length,
        );

        let left = self
            .rolling_windows
            .get(&RollingWindowKey::CorrelationLeft(call_site_id));
        let right = self
            .rolling_windows
            .get(&RollingWindowKey::CorrelationRight(call_site_id));
        let product = self
            .rolling_windows
            .get(&RollingWindowKey::CorrelationProduct(call_site_id));
        let (Some(left), Some(right), Some(product)) = (left, right, product) else {
            return Ok(PineValue::Na);
        };
        if !left.is_ready(length) || !right.is_ready(length) || !product.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let left_variance = left.variance(length, true);
        let right_variance = right.variance(length, true);
        let denominator = (left_variance * right_variance).sqrt();
        if denominator == 0.0 || !denominator.is_finite() {
            return Ok(PineValue::Na);
        }

        let covariance = product.mean(length) - (left.mean(length) * right.mean(length));
        Ok(finite_float_or_na(covariance / denominator))
    }

    fn eval_covariance(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let left = self.eval_expr(&args[0].value)?;
        let right = self.eval_expr(&args[1].value)?;
        let length = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let left = left.as_f64();
        let right = right.as_f64();
        let product = left.zip(right).map(|(left, right)| left * right);
        self.update_rolling_window_key(
            RollingWindowKey::CovarianceLeft(call_site_id),
            left,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CovarianceRight(call_site_id),
            right,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CovarianceProduct(call_site_id),
            product,
            length,
        );

        let left = self
            .rolling_windows
            .get(&RollingWindowKey::CovarianceLeft(call_site_id));
        let right = self
            .rolling_windows
            .get(&RollingWindowKey::CovarianceRight(call_site_id));
        let product = self
            .rolling_windows
            .get(&RollingWindowKey::CovarianceProduct(call_site_id));
        let (Some(left), Some(right), Some(product)) = (left, right, product) else {
            return Ok(PineValue::Na);
        };
        if !left.is_ready(length) || !right.is_ready(length) || !product.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let covariance = product.mean(length) - (left.mean(length) * right.mean(length));
        Ok(finite_float_or_na(covariance))
    }

    fn eval_median(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let mut values: Vec<_> = window.values.iter().flatten().copied().collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        let middle = values.len() / 2;
        let median = if values.len() % 2 == 0 {
            (values[middle - 1] + values[middle]) / 2.0
        } else {
            values[middle]
        };
        Ok(finite_float_or_na(median))
    }

    fn eval_mode(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let mut values: Vec<_> = window.values.iter().flatten().copied().collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

        let mut best_value = values[0];
        let mut best_count = 0_usize;
        let mut current_value = values[0];
        let mut current_count = 0_usize;
        for value in values {
            if (value - current_value).abs() < f64::EPSILON {
                current_count += 1;
            } else {
                if current_count > best_count {
                    best_value = current_value;
                    best_count = current_count;
                }
                current_value = value;
                current_count = 1;
            }
        }
        if current_count > best_count {
            best_value = current_value;
        }

        Ok(finite_float_or_na(best_value))
    }

    fn eval_percentile(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: ArrayPercentileMode,
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let percentage = self.eval_expr(&args[2].value)?.as_f64();
        if length <= 0 {
            return Ok(PineValue::Na);
        }
        let Some(percentage) = percentage else {
            return Ok(PineValue::Na);
        };
        if !(0.0..=100.0).contains(&percentage) {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let mut values: Vec<_> = window.values.iter().flatten().copied().collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        match mode {
            ArrayPercentileMode::NearestRank => {
                let rank = ((percentage / 100.0) * values.len() as f64).ceil();
                let index = (rank as usize).saturating_sub(1).min(values.len() - 1);
                Ok(finite_float_or_na(values[index]))
            }
            ArrayPercentileMode::LinearInterpolation => {
                if values.len() == 1 {
                    return Ok(finite_float_or_na(values[0]));
                }
                let rank = (percentage / 100.0) * (values.len() - 1) as f64;
                let lower = rank.floor() as usize;
                let upper = rank.ceil() as usize;
                let fraction = rank - lower as f64;
                let value = values[lower] + (values[upper] - values[lower]) * fraction;
                Ok(finite_float_or_na(value))
            }
        }
    }

    fn eval_percentrank(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let target = source.as_f64();
        let window = self.update_rolling_window(call_site_id, source, length);
        let Some(target) = target else {
            return Ok(PineValue::Na);
        };
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let count = window
            .values
            .iter()
            .flatten()
            .filter(|value| **value <= target || (**value - target).abs() < f64::EPSILON)
            .count();
        Ok(finite_float_or_na(count as f64 / length as f64 * 100.0))
    }

    fn eval_hma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let half_length = (length / 2).max(1);
        let smooth_length = (length as f64).sqrt().round().max(1.0) as usize;
        let source = source.as_f64();

        self.update_rolling_window_key(
            RollingWindowKey::HmaHalf(call_site_id),
            source,
            half_length,
        );
        self.update_rolling_window_key(RollingWindowKey::HmaFull(call_site_id), source, length);

        let half = self
            .rolling_windows
            .get(&RollingWindowKey::HmaHalf(call_site_id));
        let full = self
            .rolling_windows
            .get(&RollingWindowKey::HmaFull(call_site_id));
        let diff = match (half, full) {
            (Some(half), Some(full)) if half.is_ready(half_length) && full.is_ready(length) => {
                Some(2.0 * half.weighted_mean(half_length) - full.weighted_mean(length))
            }
            _ => None,
        };

        let smooth = self.update_rolling_window_key(
            RollingWindowKey::HmaSmooth(call_site_id),
            diff,
            smooth_length,
        );
        if !smooth.is_ready(smooth_length) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(smooth.weighted_mean(smooth_length)))
    }

    fn eval_swma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = 4_usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let values: Vec<_> = window.values.iter().flatten().copied().collect();
        let value = (values[0] + 2.0 * values[1] + 2.0 * values[2] + values[3]) / 6.0;
        Ok(finite_float_or_na(value))
    }

    fn eval_alma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let offset = self.eval_expr(&args[2].value)?.as_f64();
        let sigma = self.eval_expr(&args[3].value)?.as_f64();
        let floor_center = args
            .get(4)
            .map(|arg| self.eval_expr(&arg.value))
            .transpose()?
            .is_some_and(|value| matches!(value, PineValue::Bool(true)));
        if length <= 0 {
            return Ok(PineValue::Na);
        }
        let (Some(offset), Some(sigma)) = (offset, sigma) else {
            return Ok(PineValue::Na);
        };
        if sigma <= 0.0 || !offset.is_finite() || !sigma.is_finite() {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let mut center = offset * (length as f64 - 1.0);
        if floor_center {
            center = center.floor();
        }
        let scale = length as f64 / sigma;
        if scale == 0.0 || !scale.is_finite() {
            return Ok(PineValue::Na);
        }

        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        for (index, value) in window.values.iter().flatten().copied().enumerate() {
            let distance = index as f64 - center;
            let weight = (-(distance * distance) / (2.0 * scale * scale)).exp();
            weighted_sum += value * weight;
            weight_sum += weight;
        }
        if weight_sum == 0.0 || !weight_sum.is_finite() {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(weighted_sum / weight_sum))
    }

    fn eval_linreg(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let offset = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let values: Vec<_> = window.values.iter().flatten().copied().collect();
        if values.len() == 1 {
            return Ok(finite_float_or_na(values[0]));
        }

        let n = length as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x_squared = 0.0;
        let mut sum_xy = 0.0;
        for (index, value) in values.iter().enumerate() {
            let x = index as f64;
            sum_x += x;
            sum_y += value;
            sum_x_squared += x * x;
            sum_xy += x * value;
        }

        let denominator = n * sum_x_squared - sum_x * sum_x;
        if denominator == 0.0 {
            return Ok(PineValue::Na);
        }
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        let value = intercept + slope * (length as f64 - 1.0 - offset as f64);
        Ok(finite_float_or_na(value))
    }

    fn eval_stoch(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?.as_f64();
        let high = self.eval_expr(&args[1].value)?.as_f64();
        let low = self.eval_expr(&args[2].value)?.as_f64();
        let length = self.eval_expr(&args[3].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        self.update_rolling_window_key(RollingWindowKey::StochHigh(call_site_id), high, length);
        self.update_rolling_window_key(RollingWindowKey::StochLow(call_site_id), low, length);

        let high_window = self
            .rolling_windows
            .get(&RollingWindowKey::StochHigh(call_site_id));
        let low_window = self
            .rolling_windows
            .get(&RollingWindowKey::StochLow(call_site_id));
        let (Some(source), Some(high_window), Some(low_window)) = (source, high_window, low_window)
        else {
            return Ok(PineValue::Na);
        };
        if !high_window.is_ready(length) || !low_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let (Some(highest_high), Some(lowest_low)) = (
            high_window.extreme(WindowExtreme::Highest),
            low_window.extreme(WindowExtreme::Lowest),
        ) else {
            return Ok(PineValue::Na);
        };
        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(100.0 * (source - lowest_low) / range))
    }

    fn eval_wpr(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let close = self.current_builtin_f64("close");
        self.update_rolling_window_key(
            RollingWindowKey::WprHigh(call_site_id),
            self.current_builtin_f64("high"),
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::WprLow(call_site_id),
            self.current_builtin_f64("low"),
            length,
        );

        let high_window = self
            .rolling_windows
            .get(&RollingWindowKey::WprHigh(call_site_id));
        let low_window = self
            .rolling_windows
            .get(&RollingWindowKey::WprLow(call_site_id));
        let (Some(close), Some(high_window), Some(low_window)) = (close, high_window, low_window)
        else {
            return Ok(PineValue::Na);
        };
        if !high_window.is_ready(length) || !low_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let (Some(highest_high), Some(lowest_low)) = (
            high_window.extreme(WindowExtreme::Highest),
            low_window.extreme(WindowExtreme::Lowest),
        ) else {
            return Ok(PineValue::Na);
        };
        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(-100.0 * (highest_high - close) / range))
    }

    fn eval_ao(&mut self, call_site_id: CallSiteId) -> Result<PineValue, RuntimeError> {
        let source = match (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
        ) {
            (Some(high), Some(low)) => Some((high + low) / 2.0),
            _ => None,
        };

        self.update_rolling_window_key(RollingWindowKey::AoFast(call_site_id), source, 5);
        self.update_rolling_window_key(RollingWindowKey::AoSlow(call_site_id), source, 34);

        let fast_window = self
            .rolling_windows
            .get(&RollingWindowKey::AoFast(call_site_id));
        let slow_window = self
            .rolling_windows
            .get(&RollingWindowKey::AoSlow(call_site_id));
        let (Some(fast_window), Some(slow_window)) = (fast_window, slow_window) else {
            return Ok(PineValue::Na);
        };
        if !fast_window.is_ready(5) || !slow_window.is_ready(34) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            fast_window.mean(5) - slow_window.mean(34),
        ))
    }

    fn eval_bop(&self) -> Result<PineValue, RuntimeError> {
        let (Some(open), Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("open"),
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(PineValue::Na);
        };

        let range = high - low;
        if range == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na((close - open) / range))
    }

    fn eval_window_variance(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let biased = if let Some(arg) = args.get(2) {
            matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true))
        } else {
            true
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) || (!biased && length < 2) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.variance(length, biased)))
    }

    fn eval_tr(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let handle_na = if let Some(arg) = args.first() {
            matches!(self.eval_expr(&arg.value)?, PineValue::Bool(true))
        } else {
            true
        };

        Ok(self.true_range(handle_na))
    }

    fn eval_atr(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let true_range = self.true_range(true);
        let Some(true_range) = true_range.as_f64() else {
            return Ok(PineValue::Na);
        };
        let value = rma_next(
            self.call_state
                .get(&call_site_id)
                .and_then(PineValue::as_f64),
            true_range,
            length,
        );
        let value = PineValue::Float(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    fn eval_supertrend(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(factor) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(two_na_tuple());
        };
        let atr_period = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if atr_period <= 0 {
            return Ok(two_na_tuple());
        }

        let Some(true_range) = self.true_range(true).as_f64() else {
            return Ok(two_na_tuple());
        };
        let (Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(two_na_tuple());
        };

        let previous = supertrend_state(self.call_state.get(&call_site_id));
        let atr = rma_next(previous.map(|state| state.0), true_range, atr_period);
        let hl2 = (high + low) / 2.0;
        let basic_upper = hl2 + factor * atr;
        let basic_lower = hl2 - factor * atr;
        let previous_close = self.previous_close();

        let upper = match previous.zip(previous_close) {
            Some(((_, previous_upper, _, _), previous_close))
                if basic_upper >= previous_upper && previous_close <= previous_upper =>
            {
                previous_upper
            }
            _ => basic_upper,
        };
        let lower = match previous.zip(previous_close) {
            Some(((_, _, previous_lower, _), previous_close))
                if basic_lower <= previous_lower && previous_close >= previous_lower =>
            {
                previous_lower
            }
            _ => basic_lower,
        };

        let direction = match previous {
            None => 1.0,
            Some((_, previous_upper, _, previous_supertrend))
                if previous_supertrend == previous_upper && close > upper =>
            {
                -1.0
            }
            Some((_, previous_upper, _, previous_supertrend))
                if previous_supertrend == previous_upper =>
            {
                1.0
            }
            Some(_) if close < lower => 1.0,
            Some(_) => -1.0,
        };
        let supertrend = if direction < 0.0 { lower } else { upper };

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(atr),
                PineValue::Float(upper),
                PineValue::Float(lower),
                PineValue::Float(supertrend),
            ]),
        );

        Ok(PineValue::Tuple(vec![
            finite_float_or_na(supertrend),
            PineValue::Float(direction),
        ]))
    }

    fn eval_dmi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let di_length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
        let adx_smoothing = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if di_length <= 0 || adx_smoothing <= 0 {
            return Ok(three_na_tuple());
        }

        let (Some(high), Some(low)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
        ) else {
            return Ok(three_na_tuple());
        };
        let Some(true_range) = self.true_range(true).as_f64() else {
            return Ok(three_na_tuple());
        };

        let (plus_dm, minus_dm) = match (
            self.previous_builtin_f64("high"),
            self.previous_builtin_f64("low"),
        ) {
            (Some(previous_high), Some(previous_low)) => {
                let up_move = high - previous_high;
                let down_move = previous_low - low;
                (
                    if up_move > down_move && up_move > 0.0 {
                        up_move
                    } else {
                        0.0
                    },
                    if down_move > up_move && down_move > 0.0 {
                        down_move
                    } else {
                        0.0
                    },
                )
            }
            _ => (0.0, 0.0),
        };

        let previous = dmi_state(self.call_state.get(&call_site_id));
        let smoothed_tr = rma_next(previous.map(|state| state.0), true_range, di_length);
        let smoothed_plus_dm = rma_next(previous.map(|state| state.1), plus_dm, di_length);
        let smoothed_minus_dm = rma_next(previous.map(|state| state.2), minus_dm, di_length);
        let (plus_di, minus_di) = if smoothed_tr.is_finite() && smoothed_tr != 0.0 {
            (
                100.0 * smoothed_plus_dm / smoothed_tr,
                100.0 * smoothed_minus_dm / smoothed_tr,
            )
        } else {
            (0.0, 0.0)
        };
        let di_sum = plus_di + minus_di;
        let dx = if di_sum.is_finite() && di_sum != 0.0 {
            100.0 * (plus_di - minus_di).abs() / di_sum
        } else {
            0.0
        };
        let adx = rma_next(previous.map(|state| state.3), dx, adx_smoothing);

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(smoothed_tr),
                PineValue::Float(smoothed_plus_dm),
                PineValue::Float(smoothed_minus_dm),
                PineValue::Float(adx),
            ]),
        );

        Ok(PineValue::Tuple(vec![
            finite_float_or_na(plus_di),
            finite_float_or_na(minus_di),
            finite_float_or_na(adx),
        ]))
    }

    fn eval_sar(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(start) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(increment) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(max_acceleration) = self.eval_expr(&args[2].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        if !start.is_finite() || !increment.is_finite() || !max_acceleration.is_finite() {
            return Ok(PineValue::Na);
        }

        let (Some(high), Some(low), Some(close)) = (
            self.current_builtin_f64("high"),
            self.current_builtin_f64("low"),
            self.current_builtin_f64("close"),
        ) else {
            return Ok(PineValue::Na);
        };

        let mut is_first_trend_bar = false;
        let (mut result, mut max_min, mut acceleration, mut is_below) =
            if let Some(state) = sar_state(self.call_state.get(&call_site_id)) {
                state
            } else {
                let (Some(previous_close), Some(previous_high), Some(previous_low)) = (
                    self.previous_builtin_f64("close"),
                    self.previous_builtin_f64("high"),
                    self.previous_builtin_f64("low"),
                ) else {
                    return Ok(PineValue::Na);
                };
                is_first_trend_bar = true;
                if close > previous_close {
                    (previous_low, high, start, true)
                } else {
                    (previous_high, low, start, false)
                }
            };

        result += acceleration * (max_min - result);
        if is_below {
            if result > low {
                is_first_trend_bar = true;
                is_below = false;
                result = high.max(max_min);
                max_min = low;
                acceleration = start;
            }
        } else if result < high {
            is_first_trend_bar = true;
            is_below = true;
            result = low.min(max_min);
            max_min = high;
            acceleration = start;
        }

        if !is_first_trend_bar {
            if is_below {
                if high > max_min {
                    max_min = high;
                    acceleration = (acceleration + increment).min(max_acceleration);
                }
            } else if low < max_min {
                max_min = low;
                acceleration = (acceleration + increment).min(max_acceleration);
            }
        }

        if is_below {
            if let Some(previous_low) = self.previous_builtin_f64("low") {
                result = result.min(previous_low);
            }
            if let Some(previous_previous_low) = self.builtin_f64_at("low", 2) {
                result = result.min(previous_previous_low);
            }
        } else {
            if let Some(previous_high) = self.previous_builtin_f64("high") {
                result = result.max(previous_high);
            }
            if let Some(previous_previous_high) = self.builtin_f64_at("high", 2) {
                result = result.max(previous_previous_high);
            }
        }

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(result),
                PineValue::Float(max_min),
                PineValue::Float(acceleration),
                PineValue::Bool(is_below),
            ]),
        );

        Ok(finite_float_or_na(result))
    }

    fn eval_change(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let current = self.eval_expr(&args[0].value)?;
        let length = if let Some(length_arg) = args.get(1) {
            self.eval_expr(&length_arg.value)?.as_i64().unwrap_or(1)
        } else {
            1
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(series_id) = args[0].value.series_id else {
            return Ok(PineValue::Na);
        };
        let previous = self.series_store.read(series_id, length as usize);

        match (current, previous) {
            (PineValue::Bool(current), PineValue::Bool(previous)) => {
                Ok(PineValue::Bool(current != previous))
            }
            (current, previous) => {
                let Some(current) = current.as_f64() else {
                    return Ok(PineValue::Na);
                };
                let Some(previous) = previous.as_f64() else {
                    return Ok(PineValue::Na);
                };
                Ok(PineValue::Float(current - previous))
            }
        }
    }

    fn eval_mom(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some((current, previous)) = self.current_and_previous(args)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Float(current - previous))
    }

    fn eval_roc(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some((current, previous)) = self.current_and_previous(args)? else {
            return Ok(PineValue::Na);
        };
        if previous == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(100.0 * (current - previous) / previous))
    }

    fn current_and_previous(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<Option<(f64, f64)>, RuntimeError> {
        let current = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(None);
        }

        let Some(current) = current.as_f64() else {
            return Ok(None);
        };
        let Some(series_id) = args[0].value.series_id else {
            return Ok(None);
        };
        let previous = self.series_store.read(series_id, length as usize);
        let Some(previous) = previous.as_f64() else {
            return Ok(None);
        };

        Ok(Some((current, previous)))
    }

    fn eval_rising_falling(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: RisingFallingMode,
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Bool(false));
        }

        let length = length as usize;
        let current = source.as_f64();
        let key = RollingWindowKey::RisingFalling(call_site_id);
        let value = if let Some(current) = current {
            self.rolling_windows
                .get(&key)
                .is_some_and(|window| window.is_ready(length) && window.trend(current, mode))
        } else {
            false
        };
        self.update_rolling_window_key(key, current, length);

        Ok(PineValue::Bool(value))
    }

    fn eval_cross(
        &mut self,
        args: &[HirCallArg],
        mode: CrossMode,
    ) -> Result<PineValue, RuntimeError> {
        let current_left = self.eval_expr(&args[0].value)?;
        let current_right = self.eval_expr(&args[1].value)?;
        let Some(left_series_id) = args[0].value.series_id else {
            return Ok(PineValue::Bool(false));
        };
        let previous_left = self.series_store.read(left_series_id, 1);
        let previous_right = if let Some(right_series_id) = args[1].value.series_id {
            self.series_store.read(right_series_id, 1)
        } else {
            current_right.clone()
        };

        let Some(current_left) = current_left.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(current_right) = current_right.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(previous_left) = previous_left.as_f64() else {
            return Ok(PineValue::Bool(false));
        };
        let Some(previous_right) = previous_right.as_f64() else {
            return Ok(PineValue::Bool(false));
        };

        let crossed_over = current_left > current_right && previous_left <= previous_right;
        let crossed_under = current_left < current_right && previous_left >= previous_right;
        Ok(PineValue::Bool(match mode {
            CrossMode::Any => crossed_over || crossed_under,
            CrossMode::Over => crossed_over,
            CrossMode::Under => crossed_under,
        }))
    }

    fn eval_barssince(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let condition = self.eval_expr(&args[0].value)?;
        let value = if matches!(condition, PineValue::Bool(true)) {
            PineValue::Int(0)
        } else if let Some(previous) = self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_i64)
        {
            PineValue::Int(previous + 1)
        } else {
            PineValue::Na
        };

        if matches!(value, PineValue::Int(_)) {
            self.call_state.insert(call_site_id, value.clone());
        }
        Ok(value)
    }

    fn eval_valuewhen(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let condition = self.eval_expr(&args[0].value)?;
        let source = self.eval_expr(&args[1].value)?;
        let occurrence = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(-1);
        if occurrence < 0 {
            return Ok(PineValue::Na);
        }

        let occurrence = occurrence as usize;
        if occurrence >= MAX_SERIES_HISTORY_VALUES {
            return Ok(PineValue::Na);
        }

        let values = self.valuewhen_state.entry(call_site_id).or_default();
        if matches!(condition, PineValue::Bool(true)) {
            values.push_front(source);
            values.truncate(occurrence + 1);
        }

        Ok(values.get(occurrence).cloned().unwrap_or(PineValue::Na))
    }

    fn eval_window_extreme(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let (source, length) = self.eval_extreme_source_length(args, mode)?;
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let value = window.extreme(mode).unwrap_or(f64::NAN);
        Ok(PineValue::Float(value))
    }

    fn eval_window_extreme_offset(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let (source, length) = self.eval_extreme_source_length(args, mode)?;
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(window
            .extreme_offset(mode)
            .map_or(PineValue::Na, |offset| PineValue::Int(offset as i64)))
    }

    fn eval_extreme_source_length(
        &mut self,
        args: &[HirCallArg],
        mode: WindowExtreme,
    ) -> Result<(PineValue, i64), RuntimeError> {
        if args.len() == 1 {
            let length = self.eval_expr(&args[0].value)?.as_i64().unwrap_or(0);
            let source_name = match mode {
                WindowExtreme::Highest => "high",
                WindowExtreme::Lowest => "low",
            };
            let source = self
                .current_builtin_f64(source_name)
                .map_or(PineValue::Na, PineValue::Float);
            return Ok((source, length));
        }

        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        Ok((source, length))
    }

    fn update_rolling_window(
        &mut self,
        call_site_id: CallSiteId,
        source: PineValue,
        length: usize,
    ) -> &RollingWindowState {
        let source = source.as_f64();
        self.update_rolling_window_key(RollingWindowKey::Single(call_site_id), source, length)
    }

    fn update_mfi_windows(
        &mut self,
        call_site_id: CallSiteId,
        positive_flow: Option<f64>,
        negative_flow: Option<f64>,
        length: usize,
    ) {
        self.update_rolling_window_key(
            RollingWindowKey::MfiPositive(call_site_id),
            positive_flow,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::MfiNegative(call_site_id),
            negative_flow,
            length,
        );
    }

    fn update_cmo_windows(
        &mut self,
        call_site_id: CallSiteId,
        positive_change: Option<f64>,
        negative_change: Option<f64>,
        length: usize,
    ) {
        self.update_rolling_window_key(
            RollingWindowKey::CmoPositive(call_site_id),
            positive_change,
            length,
        );
        self.update_rolling_window_key(
            RollingWindowKey::CmoNegative(call_site_id),
            negative_change,
            length,
        );
    }

    fn update_rolling_window_key(
        &mut self,
        key: RollingWindowKey,
        source: Option<f64>,
        length: usize,
    ) -> &RollingWindowState {
        let window = self.rolling_windows.entry(key).or_default();
        window.push(source, length);
        window
    }

    fn eval_color_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let color = self.eval_expr(&args[0].value)?;
        let transp = if let Some(arg) = args.get(1) {
            self.eval_expr(&arg.value)?.as_i64().unwrap_or(0)
        } else {
            0
        };
        let PineValue::Color(color) = color else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Color(apply_transparency(color, transp)))
    }

    fn eval_color_rgb(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(red) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(green) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(blue) = self.eval_expr(&args[2].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let transp = if let Some(arg) = args.get(3) {
            let Some(transp) = self.eval_expr(&arg.value)?.as_f64() else {
                return Ok(PineValue::Na);
            };
            transp.round() as i64
        } else {
            0
        };
        let color = (color_channel(red) << 16) | (color_channel(green) << 8) | color_channel(blue);
        Ok(PineValue::Color(apply_transparency(color, transp)))
    }

    fn eval_color_component(
        &mut self,
        args: &[HirCallArg],
        component: ColorComponent,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Color(color) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Float(color_component(color, component)))
    }

    fn eval_color_from_gradient(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(bottom_value) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(top_value) = self.eval_expr(&args[2].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let PineValue::Color(bottom_color) = self.eval_expr(&args[3].value)? else {
            return Ok(PineValue::Na);
        };
        let PineValue::Color(top_color) = self.eval_expr(&args[4].value)? else {
            return Ok(PineValue::Na);
        };

        let ratio = if (top_value - bottom_value).abs() < f64::EPSILON {
            1.0
        } else {
            ((value - bottom_value) / (top_value - bottom_value)).clamp(0.0, 1.0)
        };
        Ok(PineValue::Color(interpolate_color(
            bottom_color,
            top_color,
            ratio,
        )))
    }

    fn eval_str_length(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Int(value.chars().count() as i64))
    }

    fn eval_str_case(
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

    fn eval_str_match(
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

    fn eval_str_pos(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(source) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        let pattern = match self.eval_expr(&args[1].value)? {
            PineValue::String(pattern) => pattern,
            PineValue::Na => return Ok(PineValue::Int(0)),
            _ => return Ok(PineValue::Na),
        };
        if pattern.is_empty() {
            return Ok(PineValue::Int(0));
        }

        Ok(source.find(&pattern).map_or(PineValue::Na, |byte_index| {
            PineValue::Int(source[..byte_index].chars().count() as i64)
        }))
    }

    fn eval_str_substring(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_trim(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let PineValue::String(value) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::String(
            value
                .trim_matches(|ch: char| ch.is_ascii_whitespace())
                .to_owned(),
        ))
    }

    fn eval_str_repeat(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_replace(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_replace_all(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_tonumber(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_tostring(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_format(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_match_regex(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_split(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_str_format_time(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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
        if !is_supported_utc_timezone(&timezone) {
            return Err(RuntimeError {
                message: format!("str.format_time unsupported timezone `{timezone}`"),
            });
        }
        let datetime = utc_datetime_from_millis(timestamp).map_err(|_| RuntimeError {
            message: format!("str.format_time timestamp is out of range: {timestamp}"),
        })?;

        let result = format_utc_datetime(datetime, &format);
        self.string_value_or_error(result, "str.format_time")
    }

    fn eval_time_component(
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

    fn eval_timestamp(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_timeframe_in_seconds(
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

    fn eval_timeframe_from_seconds(
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

    fn eval_timeframe_change(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
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

    fn eval_optional_timestamp_part(
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

    fn stringify_value(&self, value: &PineValue, format: &str) -> String {
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

    fn eval_replace_strings(
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

    fn string_value_or_error(
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

    fn eval_string_index(&mut self, expr: &HirExpr) -> Result<Option<i64>, RuntimeError> {
        Ok(match self.eval_expr(expr)? {
            PineValue::Int(value) => Some(value),
            PineValue::Float(value) if value.is_finite() => Some(value as i64),
            PineValue::Na => None,
            _ => None,
        })
    }

    fn eval_optional_string_index(
        &mut self,
        expr: &HirExpr,
        default: i64,
    ) -> Result<i64, RuntimeError> {
        Ok(self.eval_string_index(expr)?.unwrap_or(default))
    }

    fn eval_math_abs(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(value
                .checked_abs()
                .map(PineValue::Int)
                .unwrap_or_else(|| PineValue::Float((value as f64).abs()))),
            PineValue::Float(value) => Ok(PineValue::Float(value.abs())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    fn eval_math_round(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        if args.len() == 1 {
            return match value {
                PineValue::Int(value) => Ok(PineValue::Int(value)),
                PineValue::Float(value) => Ok(PineValue::Float(value.round())),
                PineValue::Na => Ok(PineValue::Na),
                _ => Ok(PineValue::Na),
            };
        }

        let Some(value) = value.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(precision) = self.eval_expr(&args[1].value)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let precision = precision.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        let factor = 10_f64.powi(precision);
        Ok(finite_float_or_na((value * factor).round() / factor))
    }

    fn eval_math_round_to_mintick(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let mintick = pine_builtins::named_float_constant("syminfo.mintick").unwrap_or(0.01);
        if !value.is_finite() || mintick <= 0.0 || !mintick.is_finite() {
            return Ok(PineValue::Na);
        }
        let rounded_ticks = (value / mintick + 0.5).floor();
        Ok(finite_float_or_na(rounded_ticks * mintick))
    }

    fn eval_math_random(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let min = match args.first() {
            Some(arg) => {
                let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                    return Ok(PineValue::Na);
                };
                value
            }
            None => 0.0,
        };
        let max = match args.get(1) {
            Some(arg) => {
                let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                    return Ok(PineValue::Na);
                };
                value
            }
            None => 1.0,
        };
        let seed = match args.get(2) {
            Some(arg) => self.eval_expr(&arg.value)?.as_i64(),
            None => None,
        };

        if !min.is_finite() || !max.is_finite() || min >= max {
            return Ok(PineValue::Na);
        }

        let initial_state = seed.map_or_else(
            || default_random_seed(call_site_id),
            |seed| mix_random_seed(seed as u64),
        );
        let state = self
            .random_state
            .entry(call_site_id)
            .or_insert(initial_state);
        *state = next_random_state(*state);
        let unit = random_unit_interval(*state);
        Ok(finite_float_or_na(min + (max - min) * unit))
    }

    fn eval_math_sum(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let window = self.update_rolling_window(call_site_id, source, length);
        if !window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(window.sum))
    }

    fn eval_math_floor(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.floor())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    fn eval_math_ceil(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.ceil())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    fn eval_math_trunc(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.trunc())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
    }

    fn eval_math_unary_float(
        &mut self,
        args: &[HirCallArg],
        op: impl FnOnce(f64) -> f64,
    ) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(op(value)))
    }

    fn eval_math_sign(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(value) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(PineValue::Float(if value > 0.0 {
            1.0
        } else if value < 0.0 {
            -1.0
        } else {
            0.0
        }))
    }

    fn eval_math_pow(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(base) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(exponent) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(base.powf(exponent)))
    }

    fn eval_math_hypot(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(left) = self.eval_expr(&args[0].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(right) = self.eval_expr(&args[1].value)?.as_f64() else {
            return Ok(PineValue::Na);
        };
        Ok(finite_float_or_na(left.hypot(right)))
    }

    fn eval_math_avg(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let mut total = 0.0;
        let mut count = 0.0;

        for arg in args {
            let Some(value) = self.eval_expr(&arg.value)?.as_f64() else {
                return Ok(PineValue::Na);
            };
            total += value;
            count += 1.0;
        }

        if count == 0.0 {
            return Ok(PineValue::Na);
        }
        Ok(finite_float_or_na(total / count))
    }

    fn eval_math_extreme(
        &mut self,
        args: &[HirCallArg],
        mode: MathExtreme,
    ) -> Result<PineValue, RuntimeError> {
        let mut current = 0.0;
        let mut has_value = false;
        let mut has_float = false;

        for arg in args {
            match self.eval_expr(&arg.value)? {
                PineValue::Int(value) => {
                    let value = value as f64;
                    current = if has_value {
                        math_extreme(current, value, mode)
                    } else {
                        value
                    };
                    has_value = true;
                }
                PineValue::Float(value) => {
                    current = if has_value {
                        math_extreme(current, value, mode)
                    } else {
                        value
                    };
                    has_value = true;
                    has_float = true;
                }
                PineValue::Na => return Ok(PineValue::Na),
                _ => return Ok(PineValue::Na),
            }
        }

        if !has_value {
            return Ok(PineValue::Na);
        }
        if has_float {
            Ok(PineValue::Float(current))
        } else {
            Ok(PineValue::Int(current as i64))
        }
    }

    fn true_range(&self, handle_na: bool) -> PineValue {
        let Some(high) = self.current_builtin_f64("high") else {
            return PineValue::Na;
        };
        let Some(low) = self.current_builtin_f64("low") else {
            return PineValue::Na;
        };
        let high_low = high - low;
        let previous_close = self.previous_close();

        let Some(previous_close) = previous_close else {
            return if handle_na {
                PineValue::Float(high_low)
            } else {
                PineValue::Na
            };
        };

        PineValue::Float(
            high_low
                .max((high - previous_close).abs())
                .max((low - previous_close).abs()),
        )
    }

    fn current_builtin_f64(&self, name: &str) -> Option<f64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        self.current_symbols.get(&symbol.id)?.as_f64()
    }

    fn builtin_f64_at(&self, name: &str, offset: usize) -> Option<f64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        let series_id = symbol.series_id?;
        self.series_store.read(series_id, offset).as_f64()
    }

    fn previous_builtin_f64(&self, name: &str) -> Option<f64> {
        self.builtin_f64_at(name, 1)
    }

    fn current_builtin_i64(&self, name: &str) -> Option<i64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)?;
        self.current_symbols.get(&symbol.id)?.as_i64()
    }

    fn previous_close(&self) -> Option<f64> {
        self.previous_builtin_f64("close")
    }

    fn eval_ema(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Na);
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let alpha = 2.0 / (length as f64 + 1.0);
        let value = match self
            .call_state
            .get(&call_site_id)
            .and_then(PineValue::as_f64)
        {
            Some(previous) => PineValue::Float(alpha * source + (1.0 - alpha) * previous),
            None => PineValue::Float(source),
        };
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    fn eval_dema(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((source, length)) = self.eval_ema_source_and_length(args)? else {
            return Ok(PineValue::Na);
        };
        let (previous_ema1, previous_ema2, _) = ema_chain_state(self.call_state.get(&call_site_id));
        let ema1 = ema_next(previous_ema1, source, length);
        let ema2 = ema_next(previous_ema2, ema1, length);
        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![PineValue::Float(ema1), PineValue::Float(ema2)]),
        );
        Ok(finite_float_or_na(2.0 * ema1 - ema2))
    }

    fn eval_tema(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some((source, length)) = self.eval_ema_source_and_length(args)? else {
            return Ok(PineValue::Na);
        };
        let (previous_ema1, previous_ema2, previous_ema3) =
            ema_chain_state(self.call_state.get(&call_site_id));
        let ema1 = ema_next(previous_ema1, source, length);
        let ema2 = ema_next(previous_ema2, ema1, length);
        let ema3 = ema_next(previous_ema3, ema2, length);
        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(ema1),
                PineValue::Float(ema2),
                PineValue::Float(ema3),
            ]),
        );
        Ok(finite_float_or_na(3.0 * ema1 - 3.0 * ema2 + ema3))
    }

    fn eval_ema_source_and_length(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<Option<(f64, i64)>, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let Some(source) = source.as_f64() else {
            return Ok(None);
        };
        if length <= 0 {
            return Ok(None);
        }
        Ok(Some((source, length)))
    }

    fn eval_rma(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Na);
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let value = rma_next(
            self.call_state
                .get(&call_site_id)
                .and_then(PineValue::as_f64),
            source,
            length,
        );
        let value = PineValue::Float(value);
        self.call_state.insert(call_site_id, value.clone());
        Ok(value)
    }

    fn eval_rsi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Na);
        };
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(mut state) = self.rsi_state.get(&call_site_id).copied() else {
            self.rsi_state.insert(
                call_site_id,
                RsiState {
                    previous_source: source,
                    average_gain: None,
                    average_loss: None,
                },
            );
            return Ok(PineValue::Na);
        };

        let change = source - state.previous_source;
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);
        let average_gain = rma_next(state.average_gain, gain, length);
        let average_loss = rma_next(state.average_loss, loss, length);
        state.previous_source = source;
        state.average_gain = Some(average_gain);
        state.average_loss = Some(average_loss);
        self.rsi_state.insert(call_site_id, state);

        Ok(PineValue::Float(rsi_from_averages(
            average_gain,
            average_loss,
        )))
    }

    fn eval_macd(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let fast_length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let slow_length = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        let signal_length = self.eval_expr(&args[3].value)?.as_i64().unwrap_or(0);
        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Tuple(vec![
                PineValue::Na,
                PineValue::Na,
                PineValue::Na,
            ]));
        };
        if fast_length <= 0 || slow_length <= 0 || signal_length <= 0 {
            return Ok(PineValue::Tuple(vec![
                PineValue::Na,
                PineValue::Na,
                PineValue::Na,
            ]));
        }

        let mut state = self
            .macd_state
            .get(&call_site_id)
            .copied()
            .unwrap_or(MacdState {
                fast_ema: None,
                slow_ema: None,
                signal_ema: None,
            });
        let fast_ema = ema_next(state.fast_ema, source, fast_length);
        let slow_ema = ema_next(state.slow_ema, source, slow_length);
        let macd = fast_ema - slow_ema;
        let signal = ema_next(state.signal_ema, macd, signal_length);
        let hist = macd - signal;
        state.fast_ema = Some(fast_ema);
        state.slow_ema = Some(slow_ema);
        state.signal_ema = Some(signal);
        self.macd_state.insert(call_site_id, state);

        Ok(PineValue::Tuple(vec![
            PineValue::Float(macd),
            PineValue::Float(signal),
            PineValue::Float(hist),
        ]))
    }

    fn eval_tsi(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let short_length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let long_length = self.eval_expr(&args[2].value)?.as_i64().unwrap_or(0);
        if short_length <= 0 || long_length <= 0 {
            return Ok(PineValue::Na);
        }

        let Some(source) = source.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(series_id) = args[0].value.series_id else {
            return Ok(PineValue::Na);
        };
        let Some(previous_source) = self.series_store.read(series_id, 1).as_f64() else {
            return Ok(PineValue::Na);
        };

        let momentum = source - previous_source;
        let previous = tsi_state(self.call_state.get(&call_site_id));
        let short_momentum = ema_next(previous.map(|state| state.0), momentum, short_length);
        let long_momentum = ema_next(previous.map(|state| state.1), short_momentum, long_length);
        let short_abs_momentum =
            ema_next(previous.map(|state| state.2), momentum.abs(), short_length);
        let long_abs_momentum = ema_next(
            previous.map(|state| state.3),
            short_abs_momentum,
            long_length,
        );

        self.call_state.insert(
            call_site_id,
            PineValue::Tuple(vec![
                PineValue::Float(short_momentum),
                PineValue::Float(long_momentum),
                PineValue::Float(short_abs_momentum),
                PineValue::Float(long_abs_momentum),
            ]),
        );

        if long_abs_momentum == 0.0 {
            return Ok(PineValue::Na);
        }
        Ok(finite_float_or_na(long_momentum / long_abs_momentum))
    }

    fn eval_cmo(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let source = self.eval_expr(&args[0].value)?;
        let length = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        if length <= 0 {
            return Ok(PineValue::Na);
        }

        let length = length as usize;
        let (positive_change, negative_change) = match (source.as_f64(), args[0].value.series_id) {
            (Some(source), Some(series_id)) => {
                match self.series_store.read(series_id, 1).as_f64() {
                    Some(previous) => {
                        let change = source - previous;
                        (Some(change.max(0.0)), Some((-change).max(0.0)))
                    }
                    None => (None, None),
                }
            }
            _ => (None, None),
        };

        self.update_cmo_windows(call_site_id, positive_change, negative_change, length);

        let positive_window = self
            .rolling_windows
            .get(&RollingWindowKey::CmoPositive(call_site_id));
        let negative_window = self
            .rolling_windows
            .get(&RollingWindowKey::CmoNegative(call_site_id));
        let (Some(positive_window), Some(negative_window)) = (positive_window, negative_window)
        else {
            return Ok(PineValue::Na);
        };
        if !positive_window.is_ready(length) || !negative_window.is_ready(length) {
            return Ok(PineValue::Na);
        }

        let positive_sum = positive_window.sum;
        let negative_sum = negative_window.sum;
        let denominator = positive_sum + negative_sum;
        if denominator == 0.0 {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(
            100.0 * (positive_sum - negative_sum) / denominator,
        ))
    }

    fn finalize_series_outputs(&mut self) {
        finalize_series_values(&mut self.plots, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_chars, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_shapes, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_arrows, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_bars, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_candles, self.bars);
        finalize_series_values(&mut self.bg_colors, self.bars);
        finalize_series_values(&mut self.bar_colors, self.bars);
    }

    fn push_hline(&mut self, id: u32, price: PineValue) {
        if self.hlines.iter().all(|hline| hline.id != id) {
            self.hlines.push(HLineOutput { id, price });
        }
    }

    fn push_fill(&mut self, id: u32, first: PineValue, second: PineValue) {
        if self.fills.iter().any(|fill| fill.id == id) {
            return;
        }

        let Some(first_id) = output_id(first) else {
            return;
        };
        let Some(second_id) = output_id(second) else {
            return;
        };
        self.fills.push(FillOutput {
            id,
            first_id,
            second_id,
        });
    }
}

impl<'a> RealtimeRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self {
            confirmed: HistoricalRuntime::new(program),
            forming: None,
        }
    }

    pub fn update(&mut self, update: BarUpdate) -> Result<RuntimeResult, RuntimeError> {
        match update.kind {
            BarUpdateKind::Historical => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_kind(update.bar, update.kind)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Confirmed => {
                let is_new_bar = self.forming.is_none();
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, is_new_bar)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                let is_new_bar = self.forming.is_none();
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, is_new_bar)?;
                let result = runtime.result();
                self.forming = Some(runtime);
                Ok(result)
            }
        }
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        self.forming.as_ref().unwrap_or(&self.confirmed).result()
    }

    #[must_use]
    pub fn confirmed_result(&self) -> RuntimeResult {
        self.confirmed.result()
    }

    #[must_use]
    pub fn profile(&self) -> RuntimeProfile {
        self.forming.as_ref().unwrap_or(&self.confirmed).profile()
    }

    #[must_use]
    pub fn confirmed_profile(&self) -> RuntimeProfile {
        self.confirmed.profile()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrossMode {
    Any,
    Over,
    Under,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowExtreme {
    Highest,
    Lowest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RisingFallingMode {
    Rising,
    Falling,
}

impl RollingWindowState {
    fn push(&mut self, value: Option<f64>, length: usize) {
        while self.values.len() >= length {
            self.pop_front();
        }
        if let Some(value) = value {
            self.sum += value;
            self.sum_squares += value * value;
            self.values.push_back(Some(value));
        } else {
            self.na_count += 1;
            self.values.push_back(None);
        }
    }

    fn pop_front(&mut self) {
        if let Some(value) = self.values.pop_front() {
            if let Some(value) = value {
                self.sum -= value;
                self.sum_squares -= value * value;
            } else {
                self.na_count = self.na_count.saturating_sub(1);
            }
        }
    }

    fn is_ready(&self, length: usize) -> bool {
        self.values.len() == length && self.na_count == 0
    }

    fn mean(&self, length: usize) -> f64 {
        self.sum / length as f64
    }

    fn variance(&self, length: usize, biased: bool) -> f64 {
        if !biased && length < 2 {
            return f64::NAN;
        }
        let mean = self.mean(length);
        let squared_diff_sum = self
            .values
            .iter()
            .flatten()
            .map(|value| {
                let diff = *value - mean;
                diff * diff
            })
            .sum::<f64>();
        let denominator = if biased { length } else { length - 1 };
        (squared_diff_sum / denominator as f64).max(0.0)
    }

    fn extreme(&self, mode: WindowExtreme) -> Option<f64> {
        self.values
            .iter()
            .flatten()
            .copied()
            .reduce(|current, value| match mode {
                WindowExtreme::Highest => current.max(value),
                WindowExtreme::Lowest => current.min(value),
            })
    }

    fn range(&self) -> Option<f64> {
        let highest = self.extreme(WindowExtreme::Highest)?;
        let lowest = self.extreme(WindowExtreme::Lowest)?;
        Some(highest - lowest)
    }

    fn extreme_offset(&self, mode: WindowExtreme) -> Option<usize> {
        self.values
            .iter()
            .flatten()
            .copied()
            .enumerate()
            .reduce(|current, value| {
                let better = match mode {
                    WindowExtreme::Highest => value.1 >= current.1,
                    WindowExtreme::Lowest => value.1 <= current.1,
                };
                if better { value } else { current }
            })
            .map(|(index, _)| self.values.len().saturating_sub(1 + index))
    }

    fn mean_absolute_deviation(&self, length: usize) -> f64 {
        let mean = self.mean(length);
        self.values
            .iter()
            .flatten()
            .map(|value| (*value - mean).abs())
            .sum::<f64>()
            / length as f64
    }

    fn center_of_gravity(&self, length: usize) -> f64 {
        let numerator = self
            .values
            .iter()
            .flatten()
            .enumerate()
            .map(|(index, value)| *value * (length - index) as f64)
            .sum::<f64>();
        -numerator / self.sum
    }

    fn weighted_mean(&self, length: usize) -> f64 {
        let weighted_sum = self
            .values
            .iter()
            .flatten()
            .enumerate()
            .map(|(index, value)| *value * (index + 1) as f64)
            .sum::<f64>();
        let denominator = length * (length + 1) / 2;
        weighted_sum / denominator as f64
    }

    fn trend(&self, current: f64, mode: RisingFallingMode) -> bool {
        self.values.iter().flatten().all(|value| match mode {
            RisingFallingMode::Rising => current > *value,
            RisingFallingMode::Falling => current < *value,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathExtreme {
    Max,
    Min,
}

fn rma_next(previous: Option<f64>, source: f64, length: i64) -> f64 {
    match previous {
        Some(previous) => (previous * (length - 1) as f64 + source) / length as f64,
        None => source,
    }
}

fn two_na_tuple() -> PineValue {
    PineValue::Tuple(vec![PineValue::Na, PineValue::Na])
}

fn three_na_tuple() -> PineValue {
    PineValue::Tuple(vec![PineValue::Na, PineValue::Na, PineValue::Na])
}

fn vwap_result_na(has_bands: bool) -> PineValue {
    if has_bands {
        three_na_tuple()
    } else {
        PineValue::Na
    }
}

fn vwap_arg<'a>(args: &'a [HirCallArg], positional: usize, name: &str) -> Option<&'a HirExpr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .map(|arg| &arg.value)
        .or_else(|| {
            args.get(positional)
                .filter(|arg| arg.name.is_none())
                .map(|arg| &arg.value)
        })
}

fn pivot_point_arg<'a>(
    args: &'a [HirCallArg],
    positional: usize,
    name: &str,
) -> Option<&'a HirExpr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .map(|arg| &arg.value)
        .or_else(|| {
            args.get(positional)
                .filter(|arg| arg.name.is_none())
                .map(|arg| &arg.value)
        })
}

fn pivot_na_levels() -> Vec<PineValue> {
    vec![PineValue::Na; 11]
}

fn pivot_level_values(levels: [Option<f64>; 11]) -> Vec<PineValue> {
    levels
        .into_iter()
        .map(|value| value.map_or(PineValue::Na, finite_float_or_na))
        .collect()
}

fn pivot_point_levels(
    type_name: &str,
    period: PivotPointPeriod,
    current_open: f64,
) -> Vec<PineValue> {
    let high = period.high;
    let low = period.low;
    let close = period.close;
    let range = high - low;
    match type_name {
        "Traditional" => {
            let p = (high + low + close) / 3.0;
            pivot_level_values([
                Some(p),
                Some(2.0 * p - low),
                Some(2.0 * p - high),
                Some(p + range),
                Some(p - range),
                Some(2.0 * p + high - 2.0 * low),
                Some(2.0 * p - (2.0 * high - low)),
                Some(3.0 * p + high - 3.0 * low),
                Some(3.0 * p - (3.0 * high - low)),
                Some(4.0 * p + high - 4.0 * low),
                Some(4.0 * p - (4.0 * high - low)),
            ])
        }
        "Fibonacci" => {
            let p = (high + low + close) / 3.0;
            pivot_level_values([
                Some(p),
                Some(p + 0.382 * range),
                Some(p - 0.382 * range),
                Some(p + 0.618 * range),
                Some(p - 0.618 * range),
                Some(p + range),
                Some(p - range),
                None,
                None,
                None,
                None,
            ])
        }
        "Woodie" => {
            let p = (high + low + 2.0 * current_open) / 4.0;
            let r3 = high + 2.0 * (p - low);
            let s3 = low - 2.0 * (high - p);
            pivot_level_values([
                Some(p),
                Some(2.0 * p - low),
                Some(2.0 * p - high),
                Some(p + range),
                Some(p - range),
                Some(r3),
                Some(s3),
                Some(r3 + range),
                Some(s3 - range),
                None,
                None,
            ])
        }
        "Classic" => {
            let p = (high + low + close) / 3.0;
            pivot_level_values([
                Some(p),
                Some(2.0 * p - low),
                Some(2.0 * p - high),
                Some(p + range),
                Some(p - range),
                Some(p + 2.0 * range),
                Some(p - 2.0 * range),
                Some(p + 3.0 * range),
                Some(p - 3.0 * range),
                None,
                None,
            ])
        }
        "DM" => {
            let x = if period.open == close {
                high + low + 2.0 * close
            } else if close > period.open {
                2.0 * high + low + close
            } else {
                2.0 * low + high + close
            };
            pivot_level_values([
                Some(x / 4.0),
                Some(x / 2.0 - low),
                Some(x / 2.0 - high),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ])
        }
        "Camarilla" => {
            let r5 = if low == 0.0 {
                None
            } else {
                Some((high / low) * close)
            };
            let s5 = r5.map(|r5| close - (r5 - close));
            pivot_level_values([
                Some((high + low + close) / 3.0),
                Some(close + 1.1 * range / 12.0),
                Some(close - 1.1 * range / 12.0),
                Some(close + 1.1 * range / 6.0),
                Some(close - 1.1 * range / 6.0),
                Some(close + 1.1 * range / 4.0),
                Some(close - 1.1 * range / 4.0),
                Some(close + 1.1 * range / 2.0),
                Some(close - 1.1 * range / 2.0),
                r5,
                s5,
            ])
        }
        _ => pivot_na_levels(),
    }
}

fn supertrend_state(value: Option<&PineValue>) -> Option<(f64, f64, f64, f64)> {
    let Some(PineValue::Tuple(values)) = value else {
        return None;
    };
    let [atr, upper, lower, supertrend] = values.as_slice() else {
        return None;
    };
    Some((
        atr.as_f64()?,
        upper.as_f64()?,
        lower.as_f64()?,
        supertrend.as_f64()?,
    ))
}

fn dmi_state(value: Option<&PineValue>) -> Option<(f64, f64, f64, f64)> {
    let Some(PineValue::Tuple(values)) = value else {
        return None;
    };
    let [smoothed_tr, smoothed_plus_dm, smoothed_minus_dm, adx] = values.as_slice() else {
        return None;
    };
    Some((
        smoothed_tr.as_f64()?,
        smoothed_plus_dm.as_f64()?,
        smoothed_minus_dm.as_f64()?,
        adx.as_f64()?,
    ))
}

fn kc_state(value: Option<&PineValue>) -> Option<(f64, f64)> {
    let Some(PineValue::Tuple(values)) = value else {
        return None;
    };
    let [basis, range_ema] = values.as_slice() else {
        return None;
    };
    Some((basis.as_f64()?, range_ema.as_f64()?))
}

fn sar_state(value: Option<&PineValue>) -> Option<(f64, f64, f64, bool)> {
    let Some(PineValue::Tuple(values)) = value else {
        return None;
    };
    let [result, max_min, acceleration, is_below] = values.as_slice() else {
        return None;
    };
    let PineValue::Bool(is_below) = is_below else {
        return None;
    };
    Some((
        result.as_f64()?,
        max_min.as_f64()?,
        acceleration.as_f64()?,
        *is_below,
    ))
}

fn tsi_state(value: Option<&PineValue>) -> Option<(f64, f64, f64, f64)> {
    let Some(PineValue::Tuple(values)) = value else {
        return None;
    };
    let [
        short_momentum,
        long_momentum,
        short_abs_momentum,
        long_abs_momentum,
    ] = values.as_slice()
    else {
        return None;
    };
    Some((
        short_momentum.as_f64()?,
        long_momentum.as_f64()?,
        short_abs_momentum.as_f64()?,
        long_abs_momentum.as_f64()?,
    ))
}

fn ema_next(previous: Option<f64>, source: f64, length: i64) -> f64 {
    let alpha = 2.0 / (length as f64 + 1.0);
    match previous {
        Some(previous) => alpha * source + (1.0 - alpha) * previous,
        None => source,
    }
}

fn ema_chain_state(value: Option<&PineValue>) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(PineValue::Tuple(values)) = value else {
        return (None, None, None);
    };
    (
        values.first().and_then(PineValue::as_f64),
        values.get(1).and_then(PineValue::as_f64),
        values.get(2).and_then(PineValue::as_f64),
    )
}

fn rsi_from_averages(average_gain: f64, average_loss: f64) -> f64 {
    if average_loss == 0.0 {
        100.0
    } else if average_gain == 0.0 {
        0.0
    } else {
        100.0 - (100.0 / (1.0 + average_gain / average_loss))
    }
}

fn output_id(value: PineValue) -> Option<u32> {
    match value {
        PineValue::Plot(id) | PineValue::HLine(id) => Some(id),
        _ => None,
    }
}

fn call_arg_expr<'a>(args: &'a [HirCallArg], index: usize, name: &str) -> Option<&'a HirExpr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .or_else(|| args.get(index))
        .map(|arg| &arg.value)
}

fn push_series_value<T: SeriesOutput>(
    outputs: &mut Vec<T>,
    current_bar: usize,
    id: u32,
    value: PineValue,
) {
    if let Some(output) = outputs.iter_mut().find(|output| output.id() == id) {
        let values = output.values_mut();
        while values.len() < current_bar {
            values.push(PineValue::Na);
        }
        if values.len() == current_bar {
            values.push(value);
        } else if let Some(current) = values.last_mut() {
            *current = value;
        }
    } else {
        let mut values = vec![PineValue::Na; current_bar];
        values.push(value);
        outputs.push(T::new(id, values));
    }
}

trait BarAlignedOutput {
    type Point;

    fn id(&self) -> u32;
    fn new_padded(id: u32, current_bar: usize) -> Self;
    fn len(&self) -> usize;
    fn pad_to(&mut self, current_bar: usize);
    fn push_point(&mut self, point: Self::Point);
    fn update_point(&mut self, point: Self::Point);
    fn push_na_point(&mut self);
}

fn push_bar_aligned_output<T: BarAlignedOutput>(
    outputs: &mut Vec<T>,
    current_bar: usize,
    id: u32,
    point: T::Point,
) {
    if let Some(output) = outputs.iter_mut().find(|output| output.id() == id) {
        output.pad_to(current_bar);
        if output.len() == current_bar {
            output.push_point(point);
        } else {
            output.update_point(point);
        }
    } else {
        let mut output = T::new_padded(id, current_bar);
        output.push_point(point);
        outputs.push(output);
    }
}

fn finalize_bar_aligned_outputs<T: BarAlignedOutput>(outputs: &mut [T], current_bar: usize) {
    for output in outputs {
        output.pad_to(current_bar);
        if output.len() == current_bar {
            output.push_na_point();
        }
    }
}

struct PlotCharPoint {
    value: PineValue,
    char_value: PineValue,
    color: PineValue,
}

impl BarAlignedOutput for PlotCharSeries {
    type Point = PlotCharPoint;

    fn id(&self) -> u32 {
        self.id
    }

    fn new_padded(id: u32, current_bar: usize) -> Self {
        Self {
            id,
            values: vec![PineValue::Na; current_bar],
            chars: vec![PineValue::Na; current_bar],
            colors: vec![PineValue::Na; current_bar],
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn pad_to(&mut self, current_bar: usize) {
        while self.values.len() < current_bar {
            self.push_na_point();
        }
    }

    fn push_point(&mut self, point: Self::Point) {
        self.values.push(point.value);
        self.chars.push(point.char_value);
        self.colors.push(point.color);
    }

    fn update_point(&mut self, point: Self::Point) {
        if let Some(current) = self.values.last_mut() {
            *current = point.value;
        }
        if let Some(current) = self.chars.last_mut() {
            *current = point.char_value;
        }
        if let Some(current) = self.colors.last_mut() {
            *current = point.color;
        }
    }

    fn push_na_point(&mut self) {
        self.values.push(PineValue::Na);
        self.chars.push(PineValue::Na);
        self.colors.push(PineValue::Na);
    }
}

struct PlotShapePoint {
    value: PineValue,
    style: PineValue,
    location: PineValue,
    color: PineValue,
    text: PineValue,
    text_color: PineValue,
    size: PineValue,
}

impl BarAlignedOutput for PlotShapeSeries {
    type Point = PlotShapePoint;

    fn id(&self) -> u32 {
        self.id
    }

    fn new_padded(id: u32, current_bar: usize) -> Self {
        Self {
            id,
            values: vec![PineValue::Na; current_bar],
            styles: vec![PineValue::Na; current_bar],
            locations: vec![PineValue::Na; current_bar],
            colors: vec![PineValue::Na; current_bar],
            texts: vec![PineValue::Na; current_bar],
            text_colors: vec![PineValue::Na; current_bar],
            sizes: vec![PineValue::Na; current_bar],
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn pad_to(&mut self, current_bar: usize) {
        while self.values.len() < current_bar {
            self.push_na_point();
        }
    }

    fn push_point(&mut self, point: Self::Point) {
        self.values.push(point.value);
        self.styles.push(point.style);
        self.locations.push(point.location);
        self.colors.push(point.color);
        self.texts.push(point.text);
        self.text_colors.push(point.text_color);
        self.sizes.push(point.size);
    }

    fn update_point(&mut self, point: Self::Point) {
        if let Some(current) = self.values.last_mut() {
            *current = point.value;
        }
        if let Some(current) = self.styles.last_mut() {
            *current = point.style;
        }
        if let Some(current) = self.locations.last_mut() {
            *current = point.location;
        }
        if let Some(current) = self.colors.last_mut() {
            *current = point.color;
        }
        if let Some(current) = self.texts.last_mut() {
            *current = point.text;
        }
        if let Some(current) = self.text_colors.last_mut() {
            *current = point.text_color;
        }
        if let Some(current) = self.sizes.last_mut() {
            *current = point.size;
        }
    }

    fn push_na_point(&mut self) {
        self.values.push(PineValue::Na);
        self.styles.push(PineValue::Na);
        self.locations.push(PineValue::Na);
        self.colors.push(PineValue::Na);
        self.texts.push(PineValue::Na);
        self.text_colors.push(PineValue::Na);
        self.sizes.push(PineValue::Na);
    }
}

struct PlotArrowPoint {
    value: PineValue,
    color_up: PineValue,
    color_down: PineValue,
    min_height: PineValue,
    max_height: PineValue,
}

impl BarAlignedOutput for PlotArrowSeries {
    type Point = PlotArrowPoint;

    fn id(&self) -> u32 {
        self.id
    }

    fn new_padded(id: u32, current_bar: usize) -> Self {
        Self {
            id,
            values: vec![PineValue::Na; current_bar],
            color_ups: vec![PineValue::Na; current_bar],
            color_downs: vec![PineValue::Na; current_bar],
            min_heights: vec![PineValue::Na; current_bar],
            max_heights: vec![PineValue::Na; current_bar],
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn pad_to(&mut self, current_bar: usize) {
        while self.values.len() < current_bar {
            self.push_na_point();
        }
    }

    fn push_point(&mut self, point: Self::Point) {
        self.values.push(point.value);
        self.color_ups.push(point.color_up);
        self.color_downs.push(point.color_down);
        self.min_heights.push(point.min_height);
        self.max_heights.push(point.max_height);
    }

    fn update_point(&mut self, point: Self::Point) {
        if let Some(current) = self.values.last_mut() {
            *current = point.value;
        }
        if let Some(current) = self.color_ups.last_mut() {
            *current = point.color_up;
        }
        if let Some(current) = self.color_downs.last_mut() {
            *current = point.color_down;
        }
        if let Some(current) = self.min_heights.last_mut() {
            *current = point.min_height;
        }
        if let Some(current) = self.max_heights.last_mut() {
            *current = point.max_height;
        }
    }

    fn push_na_point(&mut self) {
        self.values.push(PineValue::Na);
        self.color_ups.push(PineValue::Na);
        self.color_downs.push(PineValue::Na);
        self.min_heights.push(PineValue::Na);
        self.max_heights.push(PineValue::Na);
    }
}

struct PlotBarPoint {
    open: PineValue,
    high: PineValue,
    low: PineValue,
    close: PineValue,
    color: PineValue,
}

impl BarAlignedOutput for PlotBarSeries {
    type Point = PlotBarPoint;

    fn id(&self) -> u32 {
        self.id
    }

    fn new_padded(id: u32, current_bar: usize) -> Self {
        Self {
            id,
            opens: vec![PineValue::Na; current_bar],
            highs: vec![PineValue::Na; current_bar],
            lows: vec![PineValue::Na; current_bar],
            closes: vec![PineValue::Na; current_bar],
            colors: vec![PineValue::Na; current_bar],
        }
    }

    fn len(&self) -> usize {
        self.opens.len()
    }

    fn pad_to(&mut self, current_bar: usize) {
        while self.opens.len() < current_bar {
            self.push_na_point();
        }
    }

    fn push_point(&mut self, point: Self::Point) {
        self.opens.push(point.open);
        self.highs.push(point.high);
        self.lows.push(point.low);
        self.closes.push(point.close);
        self.colors.push(point.color);
    }

    fn update_point(&mut self, point: Self::Point) {
        if let Some(current) = self.opens.last_mut() {
            *current = point.open;
        }
        if let Some(current) = self.highs.last_mut() {
            *current = point.high;
        }
        if let Some(current) = self.lows.last_mut() {
            *current = point.low;
        }
        if let Some(current) = self.closes.last_mut() {
            *current = point.close;
        }
        if let Some(current) = self.colors.last_mut() {
            *current = point.color;
        }
    }

    fn push_na_point(&mut self) {
        self.opens.push(PineValue::Na);
        self.highs.push(PineValue::Na);
        self.lows.push(PineValue::Na);
        self.closes.push(PineValue::Na);
        self.colors.push(PineValue::Na);
    }
}

struct PlotCandlePoint {
    open: PineValue,
    high: PineValue,
    low: PineValue,
    close: PineValue,
    color: PineValue,
    wick_color: PineValue,
    border_color: PineValue,
}

impl BarAlignedOutput for PlotCandleSeries {
    type Point = PlotCandlePoint;

    fn id(&self) -> u32 {
        self.id
    }

    fn new_padded(id: u32, current_bar: usize) -> Self {
        Self {
            id,
            opens: vec![PineValue::Na; current_bar],
            highs: vec![PineValue::Na; current_bar],
            lows: vec![PineValue::Na; current_bar],
            closes: vec![PineValue::Na; current_bar],
            colors: vec![PineValue::Na; current_bar],
            wick_colors: vec![PineValue::Na; current_bar],
            border_colors: vec![PineValue::Na; current_bar],
        }
    }

    fn len(&self) -> usize {
        self.opens.len()
    }

    fn pad_to(&mut self, current_bar: usize) {
        while self.opens.len() < current_bar {
            self.push_na_point();
        }
    }

    fn push_point(&mut self, point: Self::Point) {
        self.opens.push(point.open);
        self.highs.push(point.high);
        self.lows.push(point.low);
        self.closes.push(point.close);
        self.colors.push(point.color);
        self.wick_colors.push(point.wick_color);
        self.border_colors.push(point.border_color);
    }

    fn update_point(&mut self, point: Self::Point) {
        if let Some(current) = self.opens.last_mut() {
            *current = point.open;
        }
        if let Some(current) = self.highs.last_mut() {
            *current = point.high;
        }
        if let Some(current) = self.lows.last_mut() {
            *current = point.low;
        }
        if let Some(current) = self.closes.last_mut() {
            *current = point.close;
        }
        if let Some(current) = self.colors.last_mut() {
            *current = point.color;
        }
        if let Some(current) = self.wick_colors.last_mut() {
            *current = point.wick_color;
        }
        if let Some(current) = self.border_colors.last_mut() {
            *current = point.border_color;
        }
    }

    fn push_na_point(&mut self) {
        self.opens.push(PineValue::Na);
        self.highs.push(PineValue::Na);
        self.lows.push(PineValue::Na);
        self.closes.push(PineValue::Na);
        self.colors.push(PineValue::Na);
        self.wick_colors.push(PineValue::Na);
        self.border_colors.push(PineValue::Na);
    }
}

fn finalize_series_values<T: SeriesOutput>(outputs: &mut [T], current_bar: usize) {
    for output in outputs {
        let values = output.values_mut();
        while values.len() < current_bar {
            values.push(PineValue::Na);
        }
        if values.len() == current_bar {
            values.push(PineValue::Na);
        }
    }
}

fn eval_static_builtin_value(name: &str) -> PineValue {
    if let Some(color) = pine_builtins::named_color(name) {
        return PineValue::Color(color);
    }
    if let Some(value) = pine_builtins::named_float_constant(name) {
        return PineValue::Float(value);
    }
    if let Some(value) = pine_builtins::named_int_constant(name) {
        return PineValue::Int(value);
    }
    pine_builtins::named_string_constant(name)
        .map(|constant| PineValue::String(constant.to_owned()))
        .unwrap_or(PineValue::Void)
}

fn eval_literal(literal: &HirLiteral) -> PineValue {
    match literal {
        HirLiteral::Int(value) => PineValue::Int(*value),
        HirLiteral::Float(value) => PineValue::Float(*value),
        HirLiteral::Bool(value) => PineValue::Bool(*value),
        HirLiteral::String(value) => PineValue::String(value.clone()),
        HirLiteral::ColorHex(value) => PineValue::Color(parse_color_hex(value)),
    }
}

fn parse_color_hex(value: &str) -> u32 {
    u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0)
}

fn apply_transparency(color: u32, transp: i64) -> u32 {
    let rgb = if color > 0xFF_FFFF { color >> 8 } else { color } & 0xFF_FFFF;
    let transp = transp.clamp(0, 100) as u32;
    let alpha = ((100 - transp) * 255 + 50) / 100;
    (rgb << 8) | alpha
}

fn color_channel(value: f64) -> u32 {
    value.round().clamp(0.0, 255.0) as u32
}

fn color_rgba(color: u32) -> (u32, u32, u32, u32) {
    let (rgb, alpha) = if color > 0xFF_FFFF {
        (color >> 8, color & 0xFF)
    } else {
        (color, 0xFF)
    };
    ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF, alpha)
}

fn interpolate_color(bottom_color: u32, top_color: u32, ratio: f64) -> u32 {
    let (bottom_red, bottom_green, bottom_blue, bottom_alpha) = color_rgba(bottom_color);
    let (top_red, top_green, top_blue, top_alpha) = color_rgba(top_color);
    let interpolate = |bottom: u32, top: u32| -> u32 {
        (bottom as f64 + (top as f64 - bottom as f64) * ratio)
            .round()
            .clamp(0.0, 255.0) as u32
    };
    let red = interpolate(bottom_red, top_red);
    let green = interpolate(bottom_green, top_green);
    let blue = interpolate(bottom_blue, top_blue);
    let alpha = interpolate(bottom_alpha, top_alpha);
    (red << 24) | (green << 16) | (blue << 8) | alpha
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorComponent {
    Red,
    Green,
    Blue,
    Transparency,
}

fn color_component(color: u32, component: ColorComponent) -> f64 {
    let (red, green, blue, alpha) = color_rgba(color);

    match component {
        ColorComponent::Red => red as f64,
        ColorComponent::Green => green as f64,
        ColorComponent::Blue => blue as f64,
        ColorComponent::Transparency => (100.0 - (alpha as f64 * 100.0 / 255.0)).round(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeComponent {
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

fn dayofweek_value(datetime: DateTime<Utc>) -> i64 {
    i64::from(datetime.weekday().num_days_from_sunday()) + 1
}

fn replace_nth_non_overlapping(
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

fn replace_zero_width_occurrence(source: &str, replacement: &str, occurrence: usize) -> String {
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

fn replace_all_zero_width_boundaries(source: &str, replacement: &str) -> String {
    let mut result =
        String::with_capacity(source.len() + replacement.len() * (source.chars().count() + 1));
    result.push_str(replacement);
    for ch in source.chars() {
        result.push(ch);
        result.push_str(replacement);
    }
    result
}

fn is_pine_numeric_string(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let mut saw_digit = false;
    let mut saw_decimal = false;
    for ch in unsigned.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
        } else if ch == '.' && !saw_decimal {
            saw_decimal = true;
        } else {
            return false;
        }
    }
    saw_digit
}

fn stringify_array(values: &[PineValue], format: &str) -> String {
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

fn stringify_array_element(value: &PineValue, format: &str) -> String {
    match value {
        PineValue::Int(value) => format_number(*value as f64, format),
        PineValue::Float(value) => format_number(*value, format),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => value.clone(),
        PineValue::Na => "NaN".to_owned(),
        _ => "NaN".to_owned(),
    }
}

fn stringify_array_join_element(value: &PineValue) -> String {
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

fn format_string_placeholders(
    format_string: &str,
    values: &[PineValue],
    runtime: &HistoricalRuntime<'_>,
) -> Result<String, RuntimeError> {
    let mut result = String::new();
    let mut chars = format_string.char_indices().peekable();
    while let Some((byte_index, ch)) = chars.next() {
        match ch {
            '{' => {
                let start = byte_index + ch.len_utf8();
                let Some((end, _)) = chars.find(|(_, next)| *next == '}') else {
                    return Err(RuntimeError {
                        message: "str.format has unmatched `{`".to_owned(),
                    });
                };
                let placeholder = &format_string[start..end];
                if let Some(formatted) = format_placeholder(placeholder, values, runtime) {
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

fn format_placeholder(
    placeholder: &str,
    values: &[PineValue],
    runtime: &HistoricalRuntime<'_>,
) -> Option<String> {
    let mut parts = placeholder.splitn(3, ',').map(str::trim);
    let index = parts.next()?.parse::<usize>().ok()?;
    let value = values.get(index)?;
    let Some(modifier) = parts.next() else {
        return Some(runtime.stringify_value(value, "#,###.###"));
    };

    if modifier != "number" {
        return Some(runtime.stringify_value(value, "#,###.###"));
    }

    let format = match parts.next().map(str::trim) {
        Some("integer") => "#",
        Some("percent") => "#.##%",
        Some("currency") => "#,###.00",
        Some(format) if !format.is_empty() => format,
        _ => "#,###.###",
    };
    Some(runtime.stringify_value(value, format))
}

fn is_supported_utc_timezone(timezone: &str) -> bool {
    matches!(
        timezone,
        "UTC" | "Etc/UTC" | "GMT" | "Z" | "+0000" | "+00:00"
    )
}

fn timeframe_from_seconds(seconds: i64) -> Option<String> {
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

fn timeframe_bucket(timestamp_ms: i64, seconds: i64) -> Option<i64> {
    let duration_ms = seconds.checked_mul(1000)?;
    if duration_ms <= 0 {
        return None;
    }
    Some(timestamp_ms.div_euclid(duration_ms))
}

fn timeframe_seconds(timeframe: &str) -> Option<i64> {
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

fn timestamp_unsigned_parts(
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

fn utc_datetime_from_millis(timestamp: i64) -> Result<DateTime<Utc>, RuntimeError> {
    Utc.timestamp_millis_opt(timestamp)
        .single()
        .ok_or_else(|| RuntimeError {
            message: format!("timestamp is out of range: {timestamp}"),
        })
}

fn format_utc_datetime(datetime: DateTime<Utc>, format: &str) -> String {
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

fn consume_same_chars(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, ch: char) -> usize {
    let mut count = 0;
    while chars.peek().copied() == Some(ch) {
        chars.next();
        count += 1;
    }
    count
}

fn push_padded_or_plain(result: &mut String, value: u32, width: usize) {
    if width >= 2 {
        result.push_str(&format!("{value:0width$}"));
    } else {
        result.push_str(&value.to_string());
    }
}

fn format_month(month: u32, width: usize) -> String {
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

fn format_millis(millis: u32, width: usize) -> String {
    let value = format!("{millis:03}");
    value[..width.min(3)].to_owned()
}

fn format_number(value: f64, format: &str) -> String {
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

fn group_integer_digits(value: &str) -> String {
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
enum StringCase {
    Upper,
    Lower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringMatch {
    Contains,
    StartsWith,
    EndsWith,
}

fn math_extreme(left: f64, right: f64, mode: MathExtreme) -> f64 {
    match mode {
        MathExtreme::Max => left.max(right),
        MathExtreme::Min => left.min(right),
    }
}

fn eval_unary(op: HirUnaryOp, value: PineValue) -> PineValue {
    if value.is_na() {
        return PineValue::Na;
    }

    match op {
        HirUnaryOp::Plus => value,
        HirUnaryOp::Minus => match value {
            PineValue::Int(value) => PineValue::Int(-value),
            PineValue::Float(value) => PineValue::Float(-value),
            _ => PineValue::Na,
        },
        HirUnaryOp::Not => match value {
            PineValue::Bool(value) => PineValue::Bool(!value),
            _ => PineValue::Na,
        },
    }
}

fn eval_binary(op: HirBinaryOp, left: PineValue, right: PineValue) -> PineValue {
    if left.is_na() || right.is_na() {
        return PineValue::Na;
    }

    match op {
        HirBinaryOp::Add => numeric_binary(left, right, |left, right| left + right),
        HirBinaryOp::Sub => numeric_binary(left, right, |left, right| left - right),
        HirBinaryOp::Mul => numeric_binary(left, right, |left, right| left * right),
        HirBinaryOp::Div => numeric_binary(left, right, |left, right| left / right),
        HirBinaryOp::Mod => numeric_binary(left, right, |left, right| left % right),
        HirBinaryOp::Eq => PineValue::Bool(values_equal(&left, &right)),
        HirBinaryOp::NotEq => PineValue::Bool(!values_equal(&left, &right)),
        HirBinaryOp::Gt => compare_binary(left, right, |left, right| left > right),
        HirBinaryOp::Gte => compare_binary(left, right, |left, right| left >= right),
        HirBinaryOp::Lt => compare_binary(left, right, |left, right| left < right),
        HirBinaryOp::Lte => compare_binary(left, right, |left, right| left <= right),
        HirBinaryOp::And => match (left, right) {
            (PineValue::Bool(left), PineValue::Bool(right)) => PineValue::Bool(left && right),
            _ => PineValue::Na,
        },
        HirBinaryOp::Or => match (left, right) {
            (PineValue::Bool(left), PineValue::Bool(right)) => PineValue::Bool(left || right),
            _ => PineValue::Na,
        },
    }
}

fn numeric_binary(
    left: PineValue,
    right: PineValue,
    op: impl FnOnce(f64, f64) -> f64,
) -> PineValue {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => PineValue::Float(op(left, right)),
        _ => PineValue::Na,
    }
}

fn compare_binary(
    left: PineValue,
    right: PineValue,
    op: impl FnOnce(f64, f64) -> bool,
) -> PineValue {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => PineValue::Bool(op(left, right)),
        _ => PineValue::Na,
    }
}

fn values_equal(left: &PineValue, right: &PineValue) -> bool {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => (left - right).abs() < f64::EPSILON,
        _ => left == right,
    }
}

fn array_truthy_value(value: &PineValue) -> bool {
    match value {
        PineValue::Bool(value) => *value,
        PineValue::Int(value) => *value != 0,
        PineValue::Float(value) => *value != 0.0,
        _ => false,
    }
}

fn normalize_array_index(index: i64, len: usize) -> Option<usize> {
    let len = i64::try_from(len).ok()?;
    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };
    if (0..len).contains(&index) {
        Some(index as usize)
    } else {
        None
    }
}

fn normalize_array_insert_index(index: i64, len: usize) -> Option<usize> {
    let len = i64::try_from(len).ok()?;
    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };
    if (0..=len).contains(&index) {
        Some(index as usize)
    } else {
        None
    }
}

fn array_numeric_result(kind: ArrayElementKind, value: f64) -> PineValue {
    match kind {
        ArrayElementKind::Int => PineValue::Int(value as i64),
        ArrayElementKind::Float => finite_float_or_na(value),
        _ => PineValue::Na,
    }
}

fn compare_array_numeric_values(left: &PineValue, right: &PineValue) -> Ordering {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_array_sort_values(
    kind: ArrayElementKind,
    left: &PineValue,
    right: &PineValue,
    descending: bool,
) -> Ordering {
    let left_is_special = is_array_sort_special(kind, left);
    let right_is_special = is_array_sort_special(kind, right);
    match (left_is_special, right_is_special) {
        (true, true) => return Ordering::Equal,
        (true, false) => {
            return if descending {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        (false, true) => {
            return if descending {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
        (false, false) => {}
    }

    let ordering = match kind {
        ArrayElementKind::Float | ArrayElementKind::Int => {
            compare_array_numeric_values(left, right)
        }
        ArrayElementKind::String => match (left, right) {
            (PineValue::String(left), PineValue::String(right)) => left.cmp(right),
            _ => Ordering::Equal,
        },
        _ => Ordering::Equal,
    };
    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

fn is_array_sort_special(kind: ArrayElementKind, value: &PineValue) -> bool {
    matches!(value, PineValue::Na)
        || matches!(value, PineValue::Float(value) if !value.is_finite())
        || matches!(
            (kind, value),
            (ArrayElementKind::String, PineValue::String(value)) if value.is_empty()
        )
}

fn array_numeric_lower_bound(values: &[PineValue], target: &PineValue) -> usize {
    let mut left = 0;
    let mut right = values.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if compare_array_numeric_values(&values[mid], target).is_lt() {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

fn array_numeric_upper_bound(values: &[PineValue], target: &PineValue) -> usize {
    let mut left = 0;
    let mut right = values.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if compare_array_numeric_values(&values[mid], target).is_le() {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

fn finite_float_or_na(value: f64) -> PineValue {
    if value.is_finite() {
        PineValue::Float(value)
    } else {
        PineValue::Na
    }
}

fn default_random_seed(call_site_id: CallSiteId) -> u64 {
    mix_random_seed(0x9e37_79b9_7f4a_7c15_u64 ^ u64::from(call_site_id.0))
}

fn mix_random_seed(seed: u64) -> u64 {
    let mut state = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

fn next_random_state(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

fn random_unit_interval(state: u64) -> f64 {
    ((state >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
}

#[cfg(test)]
mod tests;
