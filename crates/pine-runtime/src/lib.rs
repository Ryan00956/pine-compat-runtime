//! Historical runtime scaffolding.

use std::collections::{HashMap, VecDeque};

use pine_ir::{CallSiteId, HirExpr, HirProgram, SeriesId, SymbolId, VarSlotId};

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
use builtins::arrays::{ArrayElementKind, ArrayPercentileMode};
use builtins::ta::{MacdState, PivotPointState, RsiState, VwapState};
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

#[cfg(test)]
mod tests;
