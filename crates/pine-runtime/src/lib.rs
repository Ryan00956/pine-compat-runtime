//! Historical runtime scaffolding.

use std::collections::{HashMap, VecDeque};

use pine_ir::{CallSiteId, HirCallArg, HirExpr, HirProgram, SeriesId, SymbolId, VarSlotId};

mod algorithms;
mod bar;
mod builtins;
mod error;
mod output;
mod profile;
mod retention;
mod runtime;
mod series;
mod value;

pub use bar::{Bar, BarUpdate, BarUpdateKind};
pub use error::RuntimeError;
pub use output::json::{public_runtime_profiled_result_json, public_runtime_result_json};
pub use output::model::{
    ColorSeries, FillOutput, HLineOutput, PUBLIC_OUTPUT_SCHEMA_VERSION, PlotArrowSeries,
    PlotBarSeries, PlotCandleSeries, PlotCharSeries, PlotSeries, PlotShapeSeries,
    RuntimeDiagnostic, RuntimeResult,
};
pub use profile::{RuntimeProfile, RuntimeProfiledResult};
pub use retention::HistoryRetentionMode;
pub use runtime::historical::{run_historical, run_historical_profiled};
pub use runtime::realtime::RealtimeRuntime;
pub use series::SeriesStore;
pub use value::PineValue;

use algorithms::numeric::finite_float_or_na;
use algorithms::random::{default_random_seed, next_random_state, random_unit_interval};
use algorithms::rolling_window::{
    RisingFallingMode, RollingWindowKey, RollingWindowState, WindowExtreme,
};
use builtins::args::output_id;
use builtins::arrays::{
    ArrayBinarySearchMode, ArrayElementKind, ArrayNumericMode, ArrayPercentileMode, ArrayTruthMode,
    ArrayVarianceMode,
};
use builtins::colors::ColorComponent;
use builtins::math::MathExtreme;
use builtins::strings::{StringCase, StringMatch};
use builtins::ta::{CrossMode, MacdState, PivotPointState, RsiState, VwapState};
use builtins::time::TimeComponent;
use output::align::finalize_bar_aligned_outputs;
use output::collect::finalize_series_values;
use retention::SeriesRetention;
use runtime::expressions::values_equal;

#[cfg(test)]
use builtins::colors::apply_transparency;
#[cfg(test)]
use builtins::ta::{PivotPointPeriod, pivot_na_levels, pivot_point_levels};

const MAX_WHILE_ITERATIONS: usize = 100_000;
const MAX_ARRAY_ELEMENTS: usize = 100_000;
const MAX_STRING_CHARS: usize = 40_960;
const MAX_SERIES_HISTORY_VALUES: usize = 1_000_000;
const DEFAULT_CHART_TIMEFRAME: &str = "1";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StmtControl {
    None,
    Break,
    Continue,
}

impl<'a> HistoricalRuntime<'a> {
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
                self.eval_input(args)
            }
            "plot" => self.eval_plot(call_site_id, args),
            "plotchar" => self.eval_plotchar(call_site_id, args),
            "plotshape" => self.eval_plotshape(call_site_id, args),
            "plotarrow" => self.eval_plotarrow(call_site_id, args),
            "plotbar" => self.eval_plotbar(call_site_id, args),
            "plotcandle" => self.eval_plotcandle(call_site_id, args),
            "bgcolor" => self.eval_bgcolor(call_site_id, args),
            "barcolor" => self.eval_barcolor(call_site_id, args),
            "hline" => self.eval_hline(call_site_id, args),
            "fill" => self.eval_fill(call_site_id, args),
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
            "na" => self.eval_na(args),
            "nz" => self.eval_nz(args),
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
}

#[cfg(test)]
mod tests;
