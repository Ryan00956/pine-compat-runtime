//! Historical runtime scaffolding.

use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirLiteral,
    HirProgram, HirStmt, HirStmtKind, HirUnaryOp, SeriesId, SymbolId, VarSlotId,
};
use regex::Regex;

const MAX_WHILE_ITERATIONS: usize = 100_000;
const MAX_ARRAY_ELEMENTS: usize = 100_000;
const MAX_STRING_CHARS: usize = 40_960;
const MAX_SERIES_HISTORY_VALUES: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum PineValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
    Plot(u32),
    HLine(u32),
    Array(u32),
    Tuple(Vec<PineValue>),
    Na,
    Void,
}

impl PineValue {
    #[must_use]
    pub fn is_na(&self) -> bool {
        matches!(self, Self::Na)
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarUpdateKind {
    Historical,
    Forming,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarUpdate {
    pub bar: Bar,
    pub kind: BarUpdateKind,
}

impl BarUpdate {
    #[must_use]
    pub const fn historical(bar: Bar) -> Self {
        Self {
            bar,
            kind: BarUpdateKind::Historical,
        }
    }

    #[must_use]
    pub const fn forming(bar: Bar) -> Self {
        Self {
            bar,
            kind: BarUpdateKind::Forming,
        }
    }

    #[must_use]
    pub const fn confirmed(bar: Bar) -> Self {
        Self {
            bar,
            kind: BarUpdateKind::Confirmed,
        }
    }

    #[must_use]
    pub const fn commits_series(self) -> bool {
        matches!(
            self.kind,
            BarUpdateKind::Historical | BarUpdateKind::Confirmed
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SeriesStore {
    current_bar: usize,
    buffers: HashMap<SeriesId, Vec<PineValue>>,
}

impl SeriesStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_current_bar(&mut self, current_bar: usize) {
        self.current_bar = current_bar;
    }

    #[must_use]
    pub fn current_bar(&self) -> usize {
        self.current_bar
    }

    pub fn commit(&mut self, series_id: SeriesId, value: PineValue, max_depth: Option<usize>) {
        if matches!(max_depth, Some(0)) {
            self.buffers.remove(&series_id);
            return;
        }

        let buffer = self.buffers.entry(series_id).or_default();
        buffer.push(value);
        if let Some(max_depth) = max_depth {
            trim_series_buffer(buffer, max_depth);
        }
    }

    #[must_use]
    pub fn values_len(&self) -> usize {
        self.buffers.values().map(Vec::len).sum()
    }

    #[must_use]
    pub fn max_depth(&self) -> usize {
        self.buffers.values().map(Vec::len).max().unwrap_or(0)
    }

    #[must_use]
    pub fn len(&self, series_id: SeriesId) -> usize {
        self.buffers.get(&series_id).map(Vec::len).unwrap_or(0)
    }

    #[must_use]
    pub fn read(&self, series_id: SeriesId, offset: usize) -> PineValue {
        if offset == 0 {
            return PineValue::Na;
        }

        let Some(buffer) = self.buffers.get(&series_id) else {
            return PineValue::Na;
        };
        if offset > buffer.len() {
            return PineValue::Na;
        }

        buffer[buffer.len() - offset].clone()
    }
}

fn trim_series_buffer(buffer: &mut Vec<PineValue>, max_depth: usize) {
    if buffer.len() > max_depth {
        let excess = buffer.len() - max_depth;
        buffer.drain(0..excess);
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRetentionMode {
    StaticTrimmed,
    DynamicFull,
    MaxBarsBack,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
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
    current_bar_update_kind: BarUpdateKind,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SeriesRetention {
    static_depths: Option<HashMap<SeriesId, usize>>,
    max_bars_back: Option<usize>,
}

impl SeriesRetention {
    fn from_program(program: &HirProgram) -> Self {
        if program.history.has_dynamic_offsets {
            return Self {
                static_depths: None,
                max_bars_back: program.max_bars_back.map(|value| value as usize),
            };
        }

        Self {
            static_depths: Some(
                program
                    .series_history
                    .iter()
                    .map(|requirement| {
                        (
                            requirement.series_id,
                            requirement.max_constant_offset as usize,
                        )
                    })
                    .collect(),
            ),
            max_bars_back: program.max_bars_back.map(|value| value as usize),
        }
    }

    fn max_depth_for(&self, series_id: SeriesId) -> Option<usize> {
        match (&self.static_depths, self.max_bars_back) {
            (Some(depths), Some(max_bars_back)) => Some(
                depths
                    .get(&series_id)
                    .copied()
                    .unwrap_or(0)
                    .min(max_bars_back),
            ),
            (Some(depths), None) => Some(depths.get(&series_id).copied().unwrap_or(0)),
            (None, Some(max_bars_back)) => Some(max_bars_back),
            (None, None) => None,
        }
    }

    fn mode(&self) -> HistoryRetentionMode {
        if self.max_bars_back.is_some() {
            HistoryRetentionMode::MaxBarsBack
        } else if self.static_depths.is_some() {
            HistoryRetentionMode::StaticTrimmed
        } else {
            HistoryRetentionMode::DynamicFull
        }
    }
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
    volume_sum: f64,
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
            current_bar_update_kind: BarUpdateKind::Historical,
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
        for bar in bars {
            self.append_bar(*bar)?;
        }

        Ok(())
    }

    pub fn append_bar(&mut self, bar: Bar) -> Result<(), RuntimeError> {
        self.append_bar_with_kind(bar, BarUpdateKind::Historical)
    }

    fn append_bar_with_kind(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
    ) -> Result<(), RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = update_kind;
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
        self.bars += 1;
        self.current_bar_update_kind = BarUpdateKind::Historical;
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
            ("year", PineValue::Int(datetime.year() as i64)),
            ("month", PineValue::Int(datetime.month() as i64)),
            ("dayofmonth", PineValue::Int(datetime.day() as i64)),
            ("hour", PineValue::Int(datetime.hour() as i64)),
            ("minute", PineValue::Int(datetime.minute() as i64)),
            ("second", PineValue::Int(datetime.second() as i64)),
            ("hl2", PineValue::Float((bar.high + bar.low) / 2.0)),
            (
                "hlc3",
                PineValue::Float((bar.high + bar.low + bar.close) / 3.0),
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
            "dayofmonth" => self.eval_time_component(args, TimeComponent::DayOfMonth),
            "hour" => self.eval_time_component(args, TimeComponent::Hour),
            "minute" => self.eval_time_component(args, TimeComponent::Minute),
            "second" => self.eval_time_component(args, TimeComponent::Second),
            "timestamp" => self.eval_timestamp(args),
            "math.abs" => self.eval_math_abs(args),
            "math.max" => self.eval_math_extreme(args, MathExtreme::Max),
            "math.min" => self.eval_math_extreme(args, MathExtreme::Min),
            "math.avg" => self.eval_math_avg(args),
            "math.floor" => self.eval_math_floor(args),
            "math.ceil" => self.eval_math_ceil(args),
            "math.sqrt" => self.eval_math_unary_float(args, f64::sqrt),
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
            "math.round" => self.eval_math_round(args),
            "math.sum" => self.eval_math_sum(call_site_id, args),
            "ta.sma" => self.eval_sma(call_site_id, args),
            "ta.ema" => self.eval_ema(call_site_id, args),
            "ta.rma" => self.eval_rma(call_site_id, args),
            "ta.rsi" => self.eval_rsi(call_site_id, args),
            "ta.macd" => self.eval_macd(call_site_id, args),
            "ta.tsi" => self.eval_tsi(call_site_id, args),
            "ta.cmo" => self.eval_cmo(call_site_id, args),
            "ta.bb" => self.eval_bb(call_site_id, args),
            "ta.bbw" => self.eval_bbw(call_site_id, args),
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
        let source = self.eval_expr(&args[0].value)?;
        let (Some(source), Some(volume)) = (source.as_f64(), self.current_builtin_f64("volume"))
        else {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(PineValue::Na);
        };
        let weighted = source * volume;
        if !source.is_finite() || !volume.is_finite() || !weighted.is_finite() {
            self.vwap_call_state.remove(&call_site_id);
            return Ok(PineValue::Na);
        }

        let state = self.vwap_call_state.entry(call_site_id).or_default();
        state.weighted_sum += weighted;
        state.volume_sum += volume;
        if state.volume_sum == 0.0
            || !state.weighted_sum.is_finite()
            || !state.volume_sum.is_finite()
        {
            return Ok(PineValue::Na);
        }

        Ok(finite_float_or_na(state.weighted_sum / state.volume_sum))
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
                if previous_supertrend == previous_upper =>
            {
                if close > upper {
                    -1.0
                } else {
                    1.0
                }
            }
            Some(_) => {
                if close < lower {
                    1.0
                } else {
                    -1.0
                }
            }
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
            BarUpdateKind::Historical | BarUpdateKind::Confirmed => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_kind(update.bar, update.kind)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_kind(update.bar, update.kind)?;
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
    DayOfMonth,
    Hour,
    Minute,
    Second,
}

impl TimeComponent {
    fn function_name(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::DayOfMonth => "dayofmonth",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
        }
    }

    fn value(self, datetime: DateTime<Utc>) -> i64 {
        match self {
            Self::Year => datetime.year() as i64,
            Self::Month => datetime.month() as i64,
            Self::DayOfMonth => datetime.day() as i64,
            Self::Hour => datetime.hour() as i64,
            Self::Minute => datetime.minute() as i64,
            Self::Second => datetime.second() as i64,
        }
    }
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

#[cfg(test)]
mod tests {
    use pine_sema::analyze_source;
    use pine_syntax::SourceFile;

    use super::*;

    #[test]
    fn runs_sma_plot_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("SMA")
ma = ta.sma(close, 3)
plot(ma)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(
            result.plots[0].values,
            vec![
                PineValue::Na,
                PineValue::Na,
                PineValue::Float(2.0),
                PineValue::Float(3.0),
            ]
        );
    }

    #[test]
    fn preserves_var_state_across_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("var")
var x = 0
x := x + 1
plot(close + x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(
            result.plots[0].values,
            vec![
                PineValue::Float(2.0),
                PineValue::Float(4.0),
                PineValue::Float(6.0),
            ]
        );
    }

    #[test]
    fn runs_ema_plot_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("EMA")
ma = ta.ema(close, 3)
plot(ma)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(
            result.plots[0].values,
            vec![
                PineValue::Float(1.0),
                PineValue::Float(1.5),
                PineValue::Float(2.25),
                PineValue::Float(3.125),
            ]
        );
    }

    #[test]
    fn runs_rma_plot_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("RMA")
ma = ta.rma(close, 3)
plot(ma)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[
                1.0,
                1.3333333333333333,
                1.8888888888888888,
                2.5925925925925926,
            ],
        );
    }

    #[test]
    fn runs_rsi_plot_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("RSI")
r = ta.rsi(close, 3)
plot(r)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(2.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[1..],
            &[100.0, 100.0, 66.66666666666666, 83.33333333333333],
        );
    }

    #[test]
    fn collects_hline_and_fill_once() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("fill")
p = plot(close)
h = hline(2)
fill(p, h)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.hlines.len(), 1);
        assert_eq!(result.fills.len(), 1);
        assert_eq!(result.hlines[0].price, PineValue::Int(2));
        assert_eq!(result.fills[0].first_id, result.plots[0].id);
        assert_eq!(result.fills[0].second_id, result.hlines[0].id);
    }

    #[test]
    fn runs_output_metadata_parameters() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("output metadata")
p = plot(close, title="Close", color=color.green, linewidth=2, style=plot.style_line, trackprice=false, histbase=0, offset=1, join=false, editable=true, show_last=10, display=display.pane, format=format.price, precision=2, force_overlay=false)
h = hline(2, title="Two", color=color.gray, linestyle=hline.style_dotted, linewidth=1, editable=true, display=display.price_scale)
fill(p, h, color=color.new(color.green, 80), title="Fill", editable=false, show_last=5, fillgaps=true, display=display.status_line)
bgcolor(color.new(color.blue, 90), title="Background", offset=0, editable=false, show_last=3, display=display.data_window)
barcolor(close > open ? color.green : color.red, title="Bars", offset=0, editable=true, show_last=3, display=display.none)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
        assert_eq!(result.hlines.len(), 1);
        assert_eq!(result.hlines[0].price, PineValue::Int(2));
        assert_eq!(result.fills.len(), 1);
        assert_eq!(result.fills[0].first_id, result.plots[0].id);
        assert_eq!(result.fills[0].second_id, result.hlines[0].id);
        assert_eq!(result.bg_colors.len(), 1);
        assert_eq!(result.bg_colors[0].values.len(), 3);
        assert_eq!(result.bar_colors.len(), 1);
        assert_eq!(result.bar_colors[0].values.len(), 3);
    }

    #[test]
    fn runs_input_string_condition() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("input string")
mode = input.string("Close", "Mode")
plot(mode == "Close" ? close : open)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_additional_input_variants() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("more inputs")
threshold = input.price(2.5, "Price")
start = input.time(2, "Start")
symbol = input.symbol("AAPL", "Symbol")
timeframe = input.timeframe("D", "Timeframe")
session = input.session("0930-1600", "Session")
notes = input.text_area("Plan", "Notes")
enabled = time >= start and symbol == "AAPL" and timeframe == "D" and session == "0930-1600" and notes == "Plan"
plot(enabled ? math.max(close, threshold) : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            Bar {
                time: 1,
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 1.0,
            },
            Bar {
                time: 2,
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 2.0,
                volume: 1.0,
            },
            Bar {
                time: 3,
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 3.0,
                volume: 1.0,
            },
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[0.0, 2.5, 3.0]);
    }

    #[test]
    fn runs_generic_input_variants() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("generic input")
length = input(2, "Length")
scale = input(1.5, "Scale")
enabled = input(true, "Enabled")
mode = input("SMA", "Mode")
shade = input(color.orange, "Shade")
plot(enabled and mode == "SMA" ? ta.sma(close, length) * scale : open, color=color.new(shade, 10))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.25, 3.75]);
    }

    #[test]
    fn runs_input_metadata_parameters() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("input metadata")
length = input.int(2, "Length", minval=1, maxval=20, step=1, options=[1, 2, 3], tooltip="Bars", inline="row", group="Settings", confirm=true, display=display.all)
scale = input.float(1.5, "Scale", minval=0.5, maxval=5.0, step=0.25, options=[1.0, 1.5], display=display.none)
enabled = input.bool(true, "Enabled", tooltip="Toggle", inline="row", group="Settings", confirm=false)
mode = input.string("SMA", "Mode", options=["SMA", "EMA"], tooltip="Mode")
shade = input.color(color.orange, "Shade", group="Style")
src = input.source(close, "Source", tooltip="Price", inline="src", group="Settings", confirm=true, display=display.all)
plot(enabled and mode == "SMA" ? math.max(src, length) * scale : close, color=shade)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 3.0, 4.5]);
    }

    #[test]
    fn collects_bgcolor_and_barcolor_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("colors")
if close > 1
    bgcolor(color.green)
barcolor(close > 2 ? color.red : na)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.bg_colors.len(), 1);
        assert_eq!(
            result.bg_colors[0].values,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
        assert_eq!(result.bar_colors.len(), 1);
        assert_eq!(
            result.bar_colors[0].values,
            vec![PineValue::Na, PineValue::Na, PineValue::Color(0xFF0000)]
        );
    }

    #[test]
    fn collects_plotchar_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("plotchar")
if close > 1
    plotchar(close > 2, title="Marker", char="x", color=color.green, location=location.abovebar, offset=1, text="Up", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plot_chars.len(), 1);
        assert_eq!(
            result.plot_chars[0].values,
            vec![PineValue::Na, PineValue::Bool(false), PineValue::Bool(true)]
        );
        assert_eq!(
            result.plot_chars[0].chars,
            vec![
                PineValue::Na,
                PineValue::String("x".to_owned()),
                PineValue::String("x".to_owned())
            ]
        );
        assert_eq!(
            result.plot_chars[0].colors,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
    }

    #[test]
    fn collects_plotshape_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("plotshape")
if close > 1
    plotshape(close > 2, title="Buy", style=shape.triangleup, location=location.belowbar, color=color.green, offset=1, text="Buy", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all, force_overlay=false)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plot_shapes.len(), 1);
        assert_eq!(
            result.plot_shapes[0].values,
            vec![PineValue::Na, PineValue::Bool(false), PineValue::Bool(true)]
        );
        assert_eq!(
            result.plot_shapes[0].styles,
            vec![
                PineValue::Na,
                PineValue::String("shape.triangleup".to_owned()),
                PineValue::String("shape.triangleup".to_owned())
            ]
        );
        assert_eq!(
            result.plot_shapes[0].locations,
            vec![
                PineValue::Na,
                PineValue::String("location.belowbar".to_owned()),
                PineValue::String("location.belowbar".to_owned())
            ]
        );
        assert_eq!(
            result.plot_shapes[0].colors,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
        assert_eq!(
            result.plot_shapes[0].texts,
            vec![
                PineValue::Na,
                PineValue::String("Buy".to_owned()),
                PineValue::String("Buy".to_owned())
            ]
        );
        assert_eq!(
            result.plot_shapes[0].text_colors,
            vec![
                PineValue::Na,
                PineValue::Color(0xFFFFFF),
                PineValue::Color(0xFFFFFF)
            ]
        );
        assert_eq!(
            result.plot_shapes[0].sizes,
            vec![
                PineValue::Na,
                PineValue::String("size.small".to_owned()),
                PineValue::String("size.small".to_owned())
            ]
        );
    }

    #[test]
    fn collects_plotarrow_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("plotarrow")
if close > 1
    plotarrow(close - 2, title="Momentum", colorup=color.green, colordown=color.red, offset=1, minheight=5, maxheight=20, editable=true, show_last=5, display=display.all, force_overlay=false)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plot_arrows.len(), 1);
        assert_eq!(
            result.plot_arrows[0].values,
            vec![PineValue::Na, PineValue::Float(0.0), PineValue::Float(1.0)]
        );
        assert_eq!(
            result.plot_arrows[0].color_ups,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
        assert_eq!(
            result.plot_arrows[0].color_downs,
            vec![
                PineValue::Na,
                PineValue::Color(0xFF0000),
                PineValue::Color(0xFF0000)
            ]
        );
        assert_eq!(
            result.plot_arrows[0].min_heights,
            vec![PineValue::Na, PineValue::Int(5), PineValue::Int(5)]
        );
        assert_eq!(
            result.plot_arrows[0].max_heights,
            vec![PineValue::Na, PineValue::Int(20), PineValue::Int(20)]
        );
    }

    #[test]
    fn collects_plotbar_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("plotbar")
if close > 1
    plotbar(open, high, low, close, title="Bars", color=color.green, editable=true, show_last=5, display=display.all)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 0.0, 1.0),
            bar_ohlc(2.0, 4.0, 1.0, 3.0),
            bar_ohlc(4.0, 6.0, 3.0, 5.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plot_bars.len(), 1);
        assert_eq!(
            result.plot_bars[0].opens,
            vec![PineValue::Na, PineValue::Float(2.0), PineValue::Float(4.0)]
        );
        assert_eq!(
            result.plot_bars[0].highs,
            vec![PineValue::Na, PineValue::Float(4.0), PineValue::Float(6.0)]
        );
        assert_eq!(
            result.plot_bars[0].lows,
            vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(3.0)]
        );
        assert_eq!(
            result.plot_bars[0].closes,
            vec![PineValue::Na, PineValue::Float(3.0), PineValue::Float(5.0)]
        );
        assert_eq!(
            result.plot_bars[0].colors,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
    }

    #[test]
    fn collects_plotcandle_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("plotcandle")
if close > 1
    plotcandle(open, high, low, close, title="Candles", color=color.green, wickcolor=color.white, editable=true, show_last=5, bordercolor=color.red, display=display.all)
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 0.0, 1.0),
            bar_ohlc(2.0, 4.0, 1.0, 3.0),
            bar_ohlc(4.0, 6.0, 3.0, 5.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plot_candles.len(), 1);
        assert_eq!(
            result.plot_candles[0].opens,
            vec![PineValue::Na, PineValue::Float(2.0), PineValue::Float(4.0)]
        );
        assert_eq!(
            result.plot_candles[0].highs,
            vec![PineValue::Na, PineValue::Float(4.0), PineValue::Float(6.0)]
        );
        assert_eq!(
            result.plot_candles[0].lows,
            vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(3.0)]
        );
        assert_eq!(
            result.plot_candles[0].closes,
            vec![PineValue::Na, PineValue::Float(3.0), PineValue::Float(5.0)]
        );
        assert_eq!(
            result.plot_candles[0].colors,
            vec![
                PineValue::Na,
                PineValue::Color(0x008000),
                PineValue::Color(0x008000)
            ]
        );
        assert_eq!(
            result.plot_candles[0].wick_colors,
            vec![
                PineValue::Na,
                PineValue::Color(0xFFFFFF),
                PineValue::Color(0xFFFFFF)
            ]
        );
        assert_eq!(
            result.plot_candles[0].border_colors,
            vec![
                PineValue::Na,
                PineValue::Color(0xFF0000),
                PineValue::Color(0xFF0000)
            ]
        );
    }

    #[test]
    fn runs_macd_tuple_assignment() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("MACD")
[macd, signal, hist] = ta.macd(close, 2, 3, 2)
plot(macd)
plot(signal)
plot(hist)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(
            &result.plots[0].values,
            &[0.0, 0.16666666666666674, 0.30555555555555536],
        );
        assert_values_close(
            &result.plots[1].values,
            &[0.0, 0.11111111111111116, 0.24074074074074063],
        );
        assert_values_close(
            &result.plots[2].values,
            &[0.0, 0.05555555555555558, 0.06481481481481474],
        );
    }

    #[test]
    fn runs_bollinger_bands_tuple_assignment() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("BB")
[basis, upper, lower] = ta.bb(close, 3, 2)
plot(basis)
plot(upper)
plot(lower)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0, 3.0]);
        assert_values_close(
            &result.plots[1].values[2..],
            &[3.632993161855452, 4.6329931618554525],
        );
        assert_values_close(
            &result.plots[2].values[2..],
            &[0.36700683814454793, 1.367006838144548],
        );
    }

    #[test]
    fn runs_bollinger_band_width_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("BB Width")
width = ta.bbw(close, 3, 2)
zero_basis = ta.bbw(close - close, 3, 2)
invalid = ta.bbw(close, 0, 2)
plot(width)
plot(na(zero_basis) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[1.632993161855452, 1.4966629547095767],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_cum_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("cum")
value = ta.cum(close)
index_sum = ta.cum(bar_index)
reset_after_na = ta.cum(bar_index == 2 ? na : close)
plot(value)
plot(index_sum)
plot(reset_after_na)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 3.0, 6.0, 10.0, 15.0]);
        assert_values_close(&result.plots[1].values, &[0.0, 1.0, 3.0, 6.0, 10.0]);
        assert_values_close(&result.plots[2].values[..2], &[1.0, 3.0]);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
        assert_values_close(&result.plots[2].values[3..], &[4.0, 9.0]);
    }

    #[test]
    fn runs_obv_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("obv")
plot(ta.obv)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_volume(1.0, 10.0),
            bar_volume(3.0, 20.0),
            bar_volume(3.0, 30.0),
            bar_volume(2.0, 40.0),
            bar_volume(5.0, 50.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[20.0, 20.0, -20.0, 30.0]);
    }

    #[test]
    fn runs_accdist_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("accdist")
plot(ta.accdist)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
            bar_ohlcv(10.0, 20.0, 10.0, 10.0, 50.0),
            bar_ohlcv(10.0, 10.0, 10.0, 10.0, 30.0),
            bar_ohlcv(20.0, 30.0, 10.0, 25.0, 20.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values[..2], &[40.0, -10.0]);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_values_close(&result.plots[0].values[3..], &[10.0]);
    }

    #[test]
    fn runs_iii_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("iii")
plot(ta.iii)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
            bar_ohlcv(12.0, 20.0, 10.0, 5.0, 2.0),
            bar_ohlcv(10.0, 10.0, 10.0, 10.0, 10.0),
            bar_ohlcv(10.0, 20.0, 10.0, 15.0, 0.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values[..2], &[0.004, -1.0]);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_eq!(result.plots[0].values[3], PineValue::Na);
    }

    #[test]
    fn runs_vwap_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("vwap")
plot(ta.vwap)
plot(ta.vwap(close))
plot(ta.vwap(bar_index == 2 ? na : close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(9.0, 12.0, 6.0, 9.0, 10.0),
            bar_ohlcv(18.0, 24.0, 12.0, 18.0, 30.0),
            bar_ohlcv(25.0, 30.0, 15.0, 15.0, 0.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[9.0, 15.75, 15.75]);
        assert_values_close(&result.plots[1].values, &[9.0, 15.75, 15.75]);
        assert_values_close(&result.plots[2].values[..2], &[9.0, 15.75]);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
    }

    #[test]
    fn runs_nvi_pvi_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("volume index")
plot(ta.nvi)
plot(ta.pvi)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_volume(10.0, 100.0),
            bar_volume(12.0, 90.0),
            bar_volume(6.0, 120.0),
            bar_volume(0.0, 80.0),
            bar_volume(5.0, 60.0),
            bar_volume(10.0, 50.0),
            bar_volume(15.0, 70.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[1.0, 1.2, 1.2, 1.2, 1.2, 2.4, 2.4],
        );
        assert_values_close(
            &result.plots[1].values,
            &[1.0, 1.0, 0.5, 0.5, 0.5, 0.5, 0.75],
        );
    }

    #[test]
    fn runs_pvt_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("pvt")
plot(ta.pvt)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_volume(10.0, 100.0),
            bar_volume(12.0, 50.0),
            bar_volume(6.0, 30.0),
            bar_volume(6.0, 20.0),
            bar_volume(9.0, 10.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[10.0, -5.0, -5.0, 0.0]);
    }

    #[test]
    fn runs_wad_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("wad")
plot(ta.wad)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 10.0, 10.0, 10.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(10.0, 12.0, 8.0, 9.0),
            bar_ohlc(8.0, 10.0, 7.0, 9.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, -1.0, -1.0, 1.0]);
    }

    #[test]
    fn runs_wvad_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("wvad")
plot(ta.wvad)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
            bar_ohlcv(10.0, 10.0, 10.0, 10.0, 50.0),
            bar_ohlcv(20.0, 25.0, 15.0, 15.0, 40.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values[..1], &[20.0]);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[-20.0]);
    }

    #[test]
    fn runs_true_range_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("TR")
tr = ta.tr()
plot(tr)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(9.0, 10.0, 8.0, 9.0),
            bar_ohlc(11.0, 12.0, 11.0, 11.0),
            bar_ohlc(7.0, 8.0, 6.0, 7.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
    }

    #[test]
    fn true_range_can_return_na_on_first_bar() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("TR")
tr = ta.tr(false)
plot(tr)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(9.0, 10.0, 8.0, 9.0),
            bar_ohlc(11.0, 12.0, 11.0, 11.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.0]);
    }

    #[test]
    fn runs_true_range_variable_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("TR variable")
plot(ta.tr)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        assert_eq!(hir.history.max_constant_offset, 1);

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 1.5),
            bar_ohlc(2.0, 5.0, 2.0, 4.0),
            bar_ohlc(3.0, 4.0, 1.0, 2.0),
        ];
        let result = run_historical(&hir, &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.5, 3.0]);
    }

    #[test]
    fn runs_atr_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("ATR")
atr = ta.atr(3)
plot(atr)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(9.0, 10.0, 8.0, 9.0),
            bar_ohlc(11.0, 12.0, 11.0, 11.0),
            bar_ohlc(7.0, 8.0, 6.0, 7.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[2.0, 2.3333333333333335, 3.2222222222222223],
        );
    }

    #[test]
    fn runs_supertrend_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("Supertrend")
[line, direction] = ta.supertrend(2, 3)
[bad_line, bad_direction] = ta.supertrend(2, 0)
plot(line)
plot(direction)
plot(na(bad_line) and na(bad_direction) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 11.0, 9.0, 10.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(12.0, 16.0, 12.0, 15.0),
            bar_ohlc(15.0, 17.0, 14.0, 16.0),
            bar_ohlc(16.0, 14.0, 8.0, 9.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[
                14.0,
                14.0,
                14.0,
                8.666666666666668,
                9.944444444444445,
                20.037037037037038,
            ],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, -1.0, -1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_dmi_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("DMI")
[plus, minus, adx] = ta.dmi(3, 2)
[bad_plus, bad_minus, bad_adx] = ta.dmi(3, 0)
plot(plus)
plot(minus)
plot(adx)
plot(na(bad_plus) and na(bad_minus) and na(bad_adx) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 11.0, 9.0, 10.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(12.0, 16.0, 12.0, 15.0),
            bar_ohlc(15.0, 17.0, 14.0, 16.0),
            bar_ohlc(16.0, 14.0, 8.0, 9.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[
                0.0,
                16.666666666666664,
                27.777777777777775,
                51.38888888888888,
                44.88888888888889,
                18.397085610200364,
            ],
        );
        assert_values_close(
            &result.plots[1].values,
            &[0.0, 0.0, 0.0, 0.0, 0.0, 44.26229508196722],
        );
        assert_values_close(
            &result.plots[2].values,
            &[0.0, 50.0, 75.0, 87.5, 93.75, 67.51453488372093],
        );
        assert_values_close(&result.plots[3].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_change_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("change")
c1 = ta.change(close)
c2 = ta.change(close, 2)
index_change = ta.change(bar_index)
flag_change = ta.change(close > open)
plot(c1)
plot(c2)
plot(index_change)
plot(na(flag_change) ? 0 : flag_change ? 1 : -1)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(6.0), bar(10.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 3.0, 4.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[5.0, 7.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_values_close(&result.plots[2].values[1..], &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[0.0, -1.0, -1.0, -1.0]);
    }

    #[test]
    fn runs_mom_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("mom")
value = ta.mom(close, 2)
index_value = ta.mom(bar_index, 2)
plot(value)
plot(index_value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(6.0), bar(10.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[5.0, 7.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[2.0, 2.0]);
    }

    #[test]
    fn runs_roc_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("roc")
value = ta.roc(close, 2)
zero = ta.roc(open, 2)
index_value = ta.roc(bar_index, 2)
plot(value)
plot(zero)
plot(index_value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(0.0, 1.0, 0.0, 10.0),
            bar_ohlc(1.0, 1.0, 1.0, 15.0),
            bar_ohlc(2.0, 2.0, 2.0, 20.0),
            bar_ohlc(3.0, 3.0, 3.0, 30.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[100.0, 100.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
        assert_values_close(&result.plots[1].values[3..], &[200.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
        assert_values_close(&result.plots[2].values[3..], &[200.0]);
    }

    #[test]
    fn runs_rising_falling_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("trend")
up = ta.rising(close, 2)
down = ta.falling(close, 2)
index_up = ta.rising(bar_index, 2)
index_down = ta.falling(bar_index, 2)
plot(up ? 1 : 0)
plot(down ? 1 : 0)
plot(index_up ? 1 : 0)
plot(index_down ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar(1.0),
            bar(2.0),
            bar(3.0),
            bar(2.0),
            bar(1.0),
            bar(2.0),
            bar(4.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(
            &result.plots[0].values,
            &[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        );
        assert_values_close(
            &result.plots[1].values,
            &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        );
        assert_values_close(
            &result.plots[2].values,
            &[0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        );
        assert_values_close(&result.plots[3].values, &[0.0; 7]);
    }

    #[test]
    fn runs_barssince_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("barssince")
value = ta.barssince(close > 2)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(2.0), bar(4.0), bar(1.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn runs_valuewhen_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("valuewhen")
last_close = ta.valuewhen(close > 2, close, 0)
previous_index = ta.valuewhen(close > 2, bar_index, 1)
last_flag = ta.valuewhen(close > 2, close > open, 0)
plot(last_close)
plot(previous_index)
plot(na(last_flag) ? 0 : last_flag ? 1 : -1)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 1.0, 1.0, 1.0),
            bar_ohlc(2.0, 3.0, 2.0, 3.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(5.0, 5.0, 4.0, 4.0),
            bar_ohlc(1.0, 1.0, 1.0, 1.0),
            bar_ohlc(4.0, 5.0, 4.0, 5.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.0, 3.0, 4.0, 4.0, 5.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
        assert_values_close(&result.plots[1].values[3..], &[1.0, 1.0, 3.0]);
        assert_values_close(&result.plots[2].values, &[0.0, 1.0, 1.0, -1.0, -1.0, 1.0]);
    }

    #[test]
    fn runs_cross_functions_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("cross")
baseline = 2.0
crossed = ta.cross(close, baseline)
over = ta.crossover(close, baseline)
under = ta.crossunder(close, baseline)
plot(crossed ? 1 : 0)
plot(over ? 1 : 0)
plot(under ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(1.0), bar(2.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[0.0, 1.0, 1.0, 0.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[0.0, 1.0, 0.0, 0.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn runs_highest_lowest_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("extremes")
hi = ta.highest(close, 3)
lo = ta.lowest(close, 3)
plot(hi)
plot(lo)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[3.0, 5.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
    }

    #[test]
    fn runs_all_time_extremes_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("all-time extremes")
hi = ta.max(close)
lo = ta.min(open)
held = ta.max(bar_index == 2 ? na : low)
plot(hi)
plot(lo)
plot(held)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            Bar {
                time: 1,
                open: 3.0,
                high: 5.0,
                low: 5.0,
                close: 1.0,
                volume: 100.0,
            },
            Bar {
                time: 2,
                open: 2.0,
                high: 4.0,
                low: 4.0,
                close: 3.0,
                volume: 100.0,
            },
            Bar {
                time: 3,
                open: 4.0,
                high: 6.0,
                low: 1.0,
                close: 2.0,
                volume: 100.0,
            },
            Bar {
                time: 4,
                open: 1.0,
                high: 7.0,
                low: 6.0,
                close: 5.0,
                volume: 100.0,
            },
            Bar {
                time: 5,
                open: 5.0,
                high: 6.0,
                low: 3.0,
                close: 4.0,
                volume: 100.0,
            },
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 3.0, 3.0, 5.0, 5.0]);
        assert_values_close(&result.plots[1].values, &[3.0, 2.0, 2.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[5.0, 5.0, 5.0, 6.0, 6.0]);
    }

    #[test]
    fn runs_highestbars_lowestbars_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("extreme bars")
hi = ta.highestbars(close, 3)
lo = ta.lowestbars(close, 3)
plot(hi)
plot(lo)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0), bar(5.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[1.0, 0.0, 0.0, 1.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[2.0, 1.0, 2.0, 0.0]);
    }

    #[test]
    fn runs_single_argument_extremes_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("single argument extremes")
hi = ta.highest(2)
lo = ta.lowest(2)
hi_offset = ta.highestbars(2)
lo_offset = ta.lowestbars(length=2)
plot(hi)
plot(lo)
plot(hi_offset)
plot(lo_offset)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 5.0, 1.0, 1.0),
            bar_ohlc(1.0, 3.0, 0.0, 1.0),
            bar_ohlc(1.0, 4.0, 2.0, 1.0),
            bar_ohlc(1.0, 4.0, -1.0, 1.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[5.0, 4.0, 4.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_values_close(&result.plots[1].values[1..], &[0.0, 0.0, -1.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_values_close(&result.plots[2].values[1..], &[1.0, 0.0, 0.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_values_close(&result.plots[3].values[1..], &[0.0, 1.0, 0.0]);
    }

    #[test]
    fn runs_stdev_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("stdev")
biased = ta.stdev(close, 3)
sample = ta.stdev(close, 3, false)
plot(biased)
plot(sample)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[0.816496580927726, 1.247219128924647],
        );
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.0, 1.5275252316519468]);
    }

    #[test]
    fn runs_variance_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("variance")
biased = ta.variance(close, 3)
sample = ta.variance(close, 3, false)
plot(biased)
plot(sample)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[0.6666666666666666, 1.5555555555555556],
        );
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.0, 2.3333333333333335]);
    }

    #[test]
    fn runs_correlation_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("correlation")
same = ta.correlation(close, close, 3)
inverse = ta.correlation(close, -close, 3)
flat = ta.correlation(close, open, 3)
simple = ta.correlation(close, 10, 3)
with_na = ta.correlation(close, bar_index == 3 ? na : high, 3)
plot(same)
plot(inverse)
plot(flat)
plot(simple)
plot(with_na)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(10.0, 1.0, 1.0, 1.0, 1.0),
            bar_ohlcv(10.0, 2.0, 2.0, 2.0, 1.0),
            bar_ohlcv(10.0, 3.0, 3.0, 3.0, 1.0),
            bar_ohlcv(10.0, 5.0, 5.0, 5.0, 1.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[1.0, 1.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[-1.0, -1.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
        assert_eq!(result.plots[2].values[3], PineValue::Na);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_eq!(result.plots[3].values[2], PineValue::Na);
        assert_eq!(result.plots[3].values[3], PineValue::Na);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..3], &[1.0]);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
    }

    #[test]
    fn runs_covariance_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("covariance")
same = ta.covariance(close, close, 3)
inverse = ta.covariance(close, -close, 3)
flat = ta.covariance(close, open, 3)
simple = ta.covariance(close, 10, 3)
with_na = ta.covariance(close, bar_index == 3 ? na : high, 3)
invalid = ta.covariance(close, high, 0)
plot(same)
plot(inverse)
plot(flat)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(10.0, 1.0, 1.0, 1.0, 1.0),
            bar_ohlcv(10.0, 2.0, 2.0, 2.0, 1.0),
            bar_ohlcv(10.0, 3.0, 3.0, 3.0, 1.0),
            bar_ohlcv(10.0, 5.0, 5.0, 5.0, 1.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0 / 3.0, 14.0 / 9.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[-2.0 / 3.0, -14.0 / 9.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(&result.plots[2].values[2..], &[0.0, 0.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..], &[0.0, 0.0]);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..3], &[2.0 / 3.0]);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
        assert_eq!(result.plots[5].values[0], PineValue::Na);
        assert_eq!(result.plots[5].values[1], PineValue::Na);
        assert_eq!(result.plots[5].values[2], PineValue::Na);
        assert_eq!(result.plots[5].values[3], PineValue::Na);
    }

    #[test]
    fn runs_median_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("median")
odd = ta.median(close, 3)
even = ta.median(close, 4)
simple = ta.median(3, 3)
with_na = ta.median(bar_index == 3 ? na : close, 3)
invalid = ta.median(close, 0)
plot(odd)
plot(even)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
        assert_values_close(&result.plots[1].values[3..], &[3.5]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(&result.plots[2].values[2..], &[3.0, 3.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..3], &[2.0]);
        assert_eq!(result.plots[3].values[3], PineValue::Na);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_eq!(result.plots[4].values[2], PineValue::Na);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
    }

    #[test]
    fn runs_mode_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("mode")
repeated = ta.mode(close, 3)
unique = ta.mode(close + bar_index, 3)
tie = ta.mode(close, 4)
simple = ta.mode(3, 3)
with_na = ta.mode(bar_index == 3 ? na : close, 3)
invalid = ta.mode(close, 0)
plot(repeated)
plot(unique)
plot(tie)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(1.0), bar(2.0), bar(2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[1.0, 2.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
        assert_values_close(&result.plots[2].values[3..], &[1.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..], &[3.0, 3.0]);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..3], &[1.0]);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
        assert_eq!(result.plots[5].values[0], PineValue::Na);
        assert_eq!(result.plots[5].values[1], PineValue::Na);
        assert_eq!(result.plots[5].values[2], PineValue::Na);
        assert_eq!(result.plots[5].values[3], PineValue::Na);
    }

    #[test]
    fn runs_percentile_nearest_rank_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("percentile")
middle = ta.percentile_nearest_rank(close, 3, 50)
lowest = ta.percentile_nearest_rank(close, 3, 0)
highest = ta.percentile_nearest_rank(close, 3, 100)
simple = ta.percentile_nearest_rank(3, 3, 50)
with_na = ta.percentile_nearest_rank(bar_index == 3 ? na : close, 3, 50)
invalid = ta.percentile_nearest_rank(close, 3, 150)
plot(middle)
plot(lowest)
plot(highest)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(&result.plots[2].values[2..], &[5.0, 8.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..], &[3.0, 3.0]);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..3], &[2.0]);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
        assert_eq!(result.plots[5].values[0], PineValue::Na);
        assert_eq!(result.plots[5].values[1], PineValue::Na);
        assert_eq!(result.plots[5].values[2], PineValue::Na);
        assert_eq!(result.plots[5].values[3], PineValue::Na);
    }

    #[test]
    fn runs_percentile_linear_interpolation_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("linear percentile")
middle = ta.percentile_linear_interpolation(close, 3, 50)
quarter = ta.percentile_linear_interpolation(close, 3, 25)
lowest = ta.percentile_linear_interpolation(close, 3, 0)
highest = ta.percentile_linear_interpolation(close, 3, 100)
simple = ta.percentile_linear_interpolation(3, 3, 50)
with_na = ta.percentile_linear_interpolation(bar_index == 3 ? na : close, 3, 50)
invalid = ta.percentile_linear_interpolation(close, 3, -1)
plot(middle)
plot(quarter)
plot(lowest)
plot(highest)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[1.5, 3.5]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(&result.plots[2].values[2..], &[1.0, 2.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..], &[5.0, 8.0]);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..], &[3.0, 3.0]);
        assert_eq!(result.plots[5].values[0], PineValue::Na);
        assert_eq!(result.plots[5].values[1], PineValue::Na);
        assert_values_close(&result.plots[5].values[2..3], &[2.0]);
        assert_eq!(result.plots[5].values[3], PineValue::Na);
        assert_eq!(result.plots[6].values[0], PineValue::Na);
        assert_eq!(result.plots[6].values[1], PineValue::Na);
        assert_eq!(result.plots[6].values[2], PineValue::Na);
        assert_eq!(result.plots[6].values[3], PineValue::Na);
    }

    #[test]
    fn runs_percentrank_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("percentrank")
rank = ta.percentrank(close, 3)
low_rank = ta.percentrank(bar_index == 3 ? 1 : close, 3)
simple = ta.percentrank(3, 3)
with_na = ta.percentrank(bar_index == 3 ? na : close, 3)
invalid = ta.percentrank(close, 0)
plot(rank)
plot(low_rank)
plot(simple)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[200.0 / 3.0, 100.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..], &[200.0 / 3.0, 100.0 / 3.0]);
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(&result.plots[2].values[2..], &[100.0, 100.0]);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_values_close(&result.plots[3].values[2..3], &[200.0 / 3.0]);
        assert_eq!(result.plots[3].values[3], PineValue::Na);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_eq!(result.plots[4].values[2], PineValue::Na);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
    }

    #[test]
    fn runs_range_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("range")
value = ta.range(close, 3)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[2.0, 3.0]);
    }

    #[test]
    fn runs_dev_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("dev")
value = ta.dev(close, 3)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[1.1111111111111112, 1.7777777777777777],
        );
    }

    #[test]
    fn runs_vwma_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("vwma")
value = ta.vwma(close, 3)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_volume(1.0, 10.0),
            bar_volume(3.0, 20.0),
            bar_volume(5.0, 30.0),
            bar_volume(7.0, 40.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[3.6666666666666665, 5.444444444444445],
        );
    }

    #[test]
    fn runs_mfi_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("mfi")
value = ta.mfi(close, 3)
flat = ta.mfi(close * 0 + 1, 2)
invalid = ta.mfi(close, 0)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_volume(10.0, 100.0),
            bar_volume(11.0, 200.0),
            bar_volume(12.0, 300.0),
            bar_volume(10.0, 400.0),
            bar_volume(13.0, 500.0),
            bar_volume(12.0, 600.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[
                100.0,
                59.183673469387756,
                71.63120567375887,
                36.72316384180791,
            ],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_tsi_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("tsi")
value = ta.tsi(close, 2, 3)
flat = ta.tsi(close * 0 + 1, 2, 3)
invalid = ta.tsi(close, 0, 3)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar(10.0),
            bar(11.0),
            bar(12.0),
            bar(10.0),
            bar(13.0),
            bar(12.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[1..],
            &[
                1.0,
                1.0,
                4.163336342344337e-17,
                0.42857142857142866,
                0.2085561497326204,
            ],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_cmo_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("cmo")
value = ta.cmo(close, 3)
flat = ta.cmo(close * 0 + 1, 2)
invalid = ta.cmo(close, 0)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar(10.0),
            bar(11.0),
            bar(12.0),
            bar(10.0),
            bar(13.0),
            bar(12.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[3..],
            &[0.0, 33.333333333333336, 0.0],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_wma_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("wma")
value = ta.wma(close, 3)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[2.8333333333333335, 5.166666666666667],
        );
    }

    #[test]
    fn runs_hma_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("hma")
value = ta.hma(close, 4)
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0), bar(11.0), bar(16.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_eq!(result.plots[0].values[3], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[4..],
            &[10.38888888888889, 15.38888888888889],
        );
    }

    #[test]
    fn runs_swma_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("swma")
value = ta.swma(close)
with_na = ta.swma(bar_index == 4 ? na : close)
plot(value)
plot(with_na)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0), bar(16.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_values_close(&result.plots[0].values[3..], &[3.5, 7.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
        assert_values_close(&result.plots[1].values[3..4], &[3.5]);
        assert_eq!(result.plots[1].values[4], PineValue::Na);
    }

    #[test]
    fn runs_alma_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("alma")
value = ta.alma(close, 4, 0.85, 6)
floored = ta.alma(close, 4, 0.85, 6, true)
with_na = ta.alma(bar_index == 4 ? na : close, 4, 0.85, 6)
invalid = ta.alma(close, 4, 0.85, 0)
plot(value)
plot(floored)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0), bar(16.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_eq!(result.plots[0].values[2], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[3..],
            &[5.935295490253145, 11.87059098050629],
        );
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
        assert_values_close(
            &result.plots[1].values[3..],
            &[4.370978545474149, 8.741957090948299],
        );
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_eq!(result.plots[2].values[2], PineValue::Na);
        assert_values_close(&result.plots[2].values[3..4], &[5.935295490253145]);
        assert_eq!(result.plots[2].values[4], PineValue::Na);
        assert_eq!(result.plots[3].values[0], PineValue::Na);
        assert_eq!(result.plots[3].values[1], PineValue::Na);
        assert_eq!(result.plots[3].values[2], PineValue::Na);
        assert_eq!(result.plots[3].values[3], PineValue::Na);
        assert_eq!(result.plots[3].values[4], PineValue::Na);
    }

    #[test]
    fn runs_linreg_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("linreg")
current = ta.linreg(close, 3, 0)
previous = ta.linreg(close, 3, 1)
projected = ta.linreg(close, 3, -1)
single = ta.linreg(close, 1, 0)
with_na = ta.linreg(bar_index == 3 ? na : close, 3, 0)
invalid = ta.linreg(close, 0, 0)
plot(current)
plot(previous)
plot(projected)
plot(single)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[3.8333333333333335, 7.666666666666667],
        );
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[1].values[2..],
            &[2.3333333333333335, 4.666666666666667],
        );
        assert_eq!(result.plots[2].values[0], PineValue::Na);
        assert_eq!(result.plots[2].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[2].values[2..],
            &[5.333333333333334, 10.666666666666668],
        );
        assert_values_close(&result.plots[3].values, &[1.0, 2.0, 4.0, 8.0]);
        assert_eq!(result.plots[4].values[0], PineValue::Na);
        assert_eq!(result.plots[4].values[1], PineValue::Na);
        assert_values_close(&result.plots[4].values[2..3], &[3.8333333333333335]);
        assert_eq!(result.plots[4].values[3], PineValue::Na);
        assert_eq!(result.plots[5].values[0], PineValue::Na);
        assert_eq!(result.plots[5].values[1], PineValue::Na);
        assert_eq!(result.plots[5].values[2], PineValue::Na);
        assert_eq!(result.plots[5].values[3], PineValue::Na);
    }

    #[test]
    fn runs_stoch_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("stoch")
k = ta.stoch(close, high, low, 3)
flat = ta.stoch(close, 1 + close * 0, 1 + close * 0, 2)
invalid = ta.stoch(close, high, low, 0)
plot(k)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 11.0, 9.0, 10.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(12.0, 16.0, 12.0, 15.0),
            bar_ohlc(15.0, 17.0, 14.0, 16.0),
            bar_ohlc(16.0, 14.0, 8.0, 9.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[
                75.0,
                83.33333333333333,
                83.33333333333333,
                11.11111111111111,
            ],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_wpr_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("wpr")
value = ta.wpr(3)
invalid = ta.wpr(0)
plot(value)
plot(na(invalid) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 11.0, 9.0, 10.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(12.0, 16.0, 12.0, 15.0),
            bar_ohlc(15.0, 17.0, 14.0, 16.0),
            bar_ohlc(16.0, 14.0, 8.0, 9.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[2..],
            &[
                -25.0,
                -16.666666666666668,
                -16.666666666666668,
                -88.88888888888889,
            ],
        );
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

        let source = SourceFile::new(
            "test.pine",
            r#"indicator("flat wpr")
plot(na(ta.wpr(2)) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 1.0, 1.0, 1.0), bar_ohlc(1.0, 1.0, 1.0, 1.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_sar_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("sar")
sar = ta.sar(0.02, 0.02, 0.2)
plot(sar)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(10.0, 11.0, 9.0, 10.0),
            bar_ohlc(10.0, 12.0, 10.0, 11.0),
            bar_ohlc(11.0, 13.0, 11.0, 12.0),
            bar_ohlc(12.0, 16.0, 12.0, 15.0),
            bar_ohlc(15.0, 17.0, 14.0, 16.0),
            bar_ohlc(16.0, 14.0, 8.0, 9.0),
            bar_ohlc(9.0, 10.0, 6.0, 7.0),
            bar_ohlc(7.0, 8.0, 4.0, 5.0),
            bar_ohlc(5.0, 7.0, 3.0, 6.0),
            bar_ohlc(6.0, 12.0, 5.0, 11.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(
            &result.plots[0].values[1..],
            &[
                9.0, 9.0, 9.16, 9.5704, 17.0, 17.0, 16.56, 15.8064, 14.781888,
            ],
        );
    }

    #[test]
    fn runs_color_new_and_named_colors() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("colors")
c = color.new(color.red, 50)
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
gradient = color.from_gradient(close, 1, 3, color.red, color.green)
missing_gradient = color.from_gradient(na, 1, 3, color.red, color.green)
hex = #ff990080
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
hex_channels = color.r(hex) + color.g(hex) + color.b(hex) + color.t(hex)
gradient_channels = color.r(gradient) + color.g(gradient) + color.b(gradient) + color.t(gradient)
bgcolor(custom)
plot(na(c) ? 0 : 1)
plot(opaque == color.new(color.blue, 0) ? 1 : 0)
plot(channels)
plot(hex_channels)
plot(gradient_channels)
plot(na(missing_gradient) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[458.0, 458.0]);
        assert_values_close(&result.plots[3].values, &[458.0, 458.0]);
        assert_values_close(&result.plots[4].values, &[255.0, 192.0]);
        assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
        assert_eq!(apply_transparency(0xFF0000, 50), 0xFF000080);
        assert_eq!(
            result.bg_colors[0].values,
            vec![PineValue::Color(0xFF990080), PineValue::Color(0xFF990080)]
        );
    }

    #[test]
    fn runs_string_helpers() {
        let source = SourceFile::new(
            "test.pine",
            r##"indicator("strings")
mode = input.string("sma", "Mode")
upper = str.upper(mode)
lower = str.lower(upper)
length = str.length(upper)
missing = str.length(na)
matched = str.contains(upper, "M") and str.startswith(upper, "S") and str.endswith(upper, "A")
empty_match = str.contains(upper, "") and str.startswith(upper, "") and str.endswith(upper, "")
missing_match = str.contains(na, "S")
mid = str.pos(upper, "M")
missing_pos = str.pos(upper, "Z")
empty_pos = str.pos(upper, "")
na_pos = str.pos(upper, na)
slice = str.substring(upper, mid, mid + 1)
tail = str.substring(upper, mid)
wide = str.substring(upper, 1, 99)
na_begin = str.substring(upper, na, 1)
trimmed = str.trim(" \tSMA\n")
repeated = str.repeat("ab", 2, "-")
empty_repeat = str.repeat("ab", 0)
missing_repeat = str.repeat("ab", na)
replace_first = str.replace("hello", "l", "1")
replace_second = str.replace("hello", "l", "1", 1)
replace_missing = str.replace("hello", "z", "1", 0)
replace_all = str.replace_all("hello", "l", "1")
replace_boundary = str.replace("ab", "", ".", 1)
replace_all_boundaries = str.replace_all("ab", "", ".")
missing_replace = str.replace(na, "x", "y")
number = str.tonumber("1234.50")
signed_number = str.tonumber("-.5")
invalid_number = str.tonumber("$1,234.50")
exponent_number = str.tonumber("1e3")
missing_number = str.tonumber(na)
text_int = str.tostring(42)
text_float = str.tostring(1.25)
text_round0 = str.tostring(1.25, "#")
text_round1 = str.tostring(1.25, "#.#")
text_zeros = str.tostring(1.25, "#.0000")
text_percent = str.tostring(0.1234, format.percent)
text_price = str.tostring(1.234567891, format.price)
text_volume = str.tostring(1234.567, format.volume)
text_bool = str.tostring(true)
text_string = str.tostring("ok")
text_na = str.tostring(na)
values = array.new_float(3)
array.set(values, 0, 1.2)
array.set(values, 1, 2.6)
text_array = str.tostring(values, "#")
formatted = str.format("A={0}, B={1}, A2={0}", text_int, text_float)
formatted_missing = str.format("Missing {2}", text_int)
formatted_number = str.format("Rounded {0,number,#.00} Percent {1,number,percent}", 1.2, 0.0345)
formatted_array = str.format("Values {0}", values)
match_prefix = str.match("NASDAQ:AAPL", "^(?:BATS|NASDAQ|NYSE|AMEX):")
match_suffix = str.match("NASDAQ:AAPL", "AAPL$")
match_missing = str.match("NASDAQ:AAPL", "^NYSE:")
missing_match_regex = str.match(na, ".+")
split_words = str.split("A,B,,C", ",")
split_chars = str.split("xy", "")
split_missing = str.split(na, ",")
formatted_time_default = str.format_time(1609459200000)
formatted_time_date = str.format_time(1609459200000, "yyyy-MM-dd")
formatted_time_text = str.format_time(1609459200000, "HH:mm:ss 'on' MMM dd, yyyy", "UTC")
missing_format_time = str.format_time(na)
plot(upper == "SMA" and lower == "sma" ? length : 0)
plot(na(missing) ? 1 : 0)
plot(matched and empty_match ? 1 : 0)
plot(na(missing_match) ? 1 : 0)
plot(mid + empty_pos + na_pos)
plot(na(missing_pos) ? 1 : 0)
plot(slice == "M" and tail == "MA" and wide == "MA" and na_begin == "S" ? 1 : 0)
plot(trimmed == upper and repeated == "ab-ab" and empty_repeat == "" ? 1 : 0)
plot(na(missing_repeat) ? 1 : 0)
plot(replace_first == "he1lo" and replace_second == "hel1o" and replace_missing == "hello" ? 1 : 0)
plot(replace_all == "he11o" and replace_boundary == "a.b" and replace_all_boundaries == ".a.b." ? 1 : 0)
plot(na(missing_replace) ? 1 : 0)
plot(number == 1234.5 and signed_number == -0.5 ? 1 : 0)
plot(na(invalid_number) and na(exponent_number) and na(missing_number) ? 1 : 0)
plot(text_int == "42" and text_float == "1.25" and text_round0 == "1" and text_round1 == "1.3" ? 1 : 0)
plot(text_zeros == "1.2500" and text_percent == "12.34%" ? 1 : 0)
plot(text_price == "1.23456789" and text_volume == "1234.57" ? 1 : 0)
plot(text_bool == "true" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20 Percent 3.45%" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(na(missing_match_regex) ? 1 : 0)
plot(split_words.size() == 4 and split_words.get(0) == "A" and split_words.get(2) == "" and split_words.get(3) == "C" ? 1 : 0)
plot(split_chars.size() == 2 and split_chars.get(0) == "x" and split_chars.get(1) == "y" and na(split_missing) ? 1 : 0)
plot(formatted_time_default == "2021-01-01T00:00:00+0000" and formatted_time_date == "2021-01-01" ? 1 : 0)
plot(formatted_time_text == "00:00:00 on Jan 01, 2021" and na(missing_format_time) ? 1 : 0)
"##,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[9].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[15].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[16].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[17].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[18].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[19].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[20].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[21].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[22].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[23].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[24].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[25].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[26].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[27].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_utc_time_component_variables() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("time components")
plot(year)
plot(month)
plot(dayofmonth)
plot(hour)
plot(minute)
plot(second)
ts = 1612235045000
made_ts = timestamp(2021, 2, 2, 3, 4, 5)
date_ts = timestamp(2021, 1, 1)
plot(year(ts))
plot(month(ts, "UTC"))
plot(dayofmonth(ts))
plot(hour(ts))
plot(minute(ts))
plot(second(ts))
plot(na(year(na)) ? 1 : 0)
plot(made_ts == ts and date_ts == 1609459200000 ? 1 : 0)
plot(na(timestamp(na, 1, 1)) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            Bar {
                time: 1_609_459_200_000,
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 100.0,
            },
            Bar {
                time: 1_612_235_045_000,
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 2.0,
                volume: 100.0,
            },
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[2021.0, 2021.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 2.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 2.0]);
        assert_values_close(&result.plots[3].values, &[0.0, 3.0]);
        assert_values_close(&result.plots[4].values, &[0.0, 4.0]);
        assert_values_close(&result.plots[5].values, &[0.0, 5.0]);
        assert_values_close(&result.plots[6].values, &[2021.0, 2021.0]);
        assert_values_close(&result.plots[7].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[8].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[9].values, &[3.0, 3.0]);
        assert_values_close(&result.plots[10].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[11].values, &[5.0, 5.0]);
        assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_global_price_and_derived_series() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("global series")
plot(open)
plot(high)
plot(low)
plot(close)
plot(volume)
plot(time)
plot(hl2)
plot(hlc3)
plot(ohlc4)
plot(bar_index)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            Bar {
                time: 1000,
                open: 1.0,
                high: 5.0,
                low: -1.0,
                close: 3.0,
                volume: 10.0,
            },
            Bar {
                time: 2000,
                open: 2.0,
                high: 8.0,
                low: 0.0,
                close: 4.0,
                volume: 20.0,
            },
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[5.0, 8.0]);
        assert_values_close(&result.plots[2].values, &[-1.0, 0.0]);
        assert_values_close(&result.plots[3].values, &[3.0, 4.0]);
        assert_values_close(&result.plots[4].values, &[10.0, 20.0]);
        assert_values_close(&result.plots[5].values, &[1000.0, 2000.0]);
        assert_values_close(&result.plots[6].values, &[2.0, 4.0]);
        assert_values_close(&result.plots[7].values, &[7.0 / 3.0, 4.0]);
        assert_values_close(&result.plots[8].values, &[2.0, 3.5]);
        assert_values_close(&result.plots[9].values, &[0.0, 1.0]);
    }

    #[test]
    fn rejects_unsupported_calendar_function_timezone() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad calendar timezone")
plot(hour(time, "America/New_York"))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected calendar timezone error");

        assert!(
            error
                .message
                .contains("hour unsupported timezone `America/New_York`"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_unbalanced_str_format_placeholders() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad format")
plot(str.length(str.format("Value {0", close)))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected str.format placeholder error");

        assert!(
            error.message.contains("str.format has unmatched `{`"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_invalid_str_match_regex() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad match")
plot(str.length(str.match("abc", "(")))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected str.match regex error");

        assert!(
            error.message.contains("str.match invalid regex"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_unsupported_str_format_time_timezone() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad time")
plot(str.length(str.format_time(1609459200000, "yyyy-MM-dd", "America/New_York")))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected str.format_time timezone error");

        assert!(
            error
                .message
                .contains("str.format_time unsupported timezone `America/New_York`"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_invalid_timestamp_date() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad timestamp")
plot(timestamp(2021, 2, 30))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected invalid timestamp error");

        assert!(
            error
                .message
                .contains("timestamp invalid UTC datetime: 2021-02-30"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_invalid_substring_range() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad substring")
plot(str.length(str.substring("SMA", 2, 1)))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected substring range error");

        assert!(
            error
                .message
                .contains("str.substring end_pos 1 is less than begin_pos 2"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_invalid_string_repeat_counts() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bad repeat")
plot(str.length(str.repeat("x", -1)))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected negative repeat error");

        assert!(
            error
                .message
                .contains("str.repeat count cannot be negative: -1"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_oversized_string_repeat_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("huge repeat")
plot(str.length(str.repeat("x", 40961)))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected oversized repeat error");

        assert!(
            error
                .message
                .contains("str.repeat result cannot exceed 40960 characters"),
            "{}",
            error.message
        );
    }

    #[test]
    fn runs_selected_math_functions() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
avg_value = math.avg(open, close, high, low)
floor_value = math.floor(close / 2)
ceil_value = math.ceil(close / 2 - 0.25)
const_value = math.floor(2) + math.ceil(1)
sqrt_value = math.sqrt(close)
log_value = math.log(close)
log10_value = math.log10(close)
exp_value = math.exp(close)
acos_value = math.acos(close - 2)
asin_value = math.asin(close - 2)
atan_value = math.atan(close)
sign_value = math.sign(close - 2)
degrees_value = math.todegrees(close)
radians_value = math.toradians(close)
constants = math.pi + math.e + math.phi + math.rphi
sin_value = math.sin(close)
cos_value = math.cos(close)
tan_value = math.tan(close)
pow_value = math.pow(close, 2)
rounded_precision = math.round(close / 3, 2)
plot(x)
plot(y)
plot(avg_value)
plot(floor_value + ceil_value)
plot(const_value)
plot(sqrt_value)
plot(log_value)
plot(log10_value)
plot(exp_value)
plot(acos_value)
plot(asin_value)
plot(atan_value)
plot(sign_value)
plot(degrees_value)
plot(radians_value)
plot(constants)
plot(sin_value)
plot(cos_value)
plot(tan_value)
plot(pow_value)
plot(rounded_precision)
plot(math.sqrt(-1))
plot(math.log(0))
plot(math.log10(0))
plot(math.exp(1000))
plot(math.acos(2))
plot(math.asin(2))
plot(math.pow(-1, 0.5))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_values_close(&result.plots[0].values, &[2.0, 1.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[2.0, 1.0, 2.0, 2.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 2.0, 3.0, 4.0]);
        assert_values_close(&result.plots[3].values, &[1.0, 2.0, 3.0, 4.0]);
        assert_values_close(&result.plots[4].values, &[3.0, 3.0, 3.0, 3.0]);
        assert_values_close(
            &result.plots[5].values,
            &[1.0, 2.0_f64.sqrt(), 3.0_f64.sqrt(), 2.0],
        );
        assert_values_close(
            &result.plots[6].values,
            &[0.0, 2.0_f64.ln(), 3.0_f64.ln(), 4.0_f64.ln()],
        );
        assert_values_close(
            &result.plots[7].values,
            &[0.0, 2.0_f64.log10(), 3.0_f64.log10(), 4.0_f64.log10()],
        );
        assert_values_close(
            &result.plots[8].values,
            &[1.0_f64.exp(), 2.0_f64.exp(), 3.0_f64.exp(), 4.0_f64.exp()],
        );
        assert_values_close(
            &result.plots[9].values[..3],
            &[(-1.0_f64).acos(), 0.0_f64.acos(), 1.0_f64.acos()],
        );
        assert_eq!(result.plots[9].values[3], PineValue::Na);
        assert_values_close(
            &result.plots[10].values[..3],
            &[(-1.0_f64).asin(), 0.0_f64.asin(), 1.0_f64.asin()],
        );
        assert_eq!(result.plots[10].values[3], PineValue::Na);
        assert_values_close(
            &result.plots[11].values,
            &[
                1.0_f64.atan(),
                2.0_f64.atan(),
                3.0_f64.atan(),
                4.0_f64.atan(),
            ],
        );
        assert_values_close(&result.plots[12].values, &[-1.0, 0.0, 1.0, 1.0]);
        assert_values_close(
            &result.plots[13].values,
            &[
                1.0_f64.to_degrees(),
                2.0_f64.to_degrees(),
                3.0_f64.to_degrees(),
                4.0_f64.to_degrees(),
            ],
        );
        assert_values_close(
            &result.plots[14].values,
            &[
                1.0_f64.to_radians(),
                2.0_f64.to_radians(),
                3.0_f64.to_radians(),
                4.0_f64.to_radians(),
            ],
        );
        assert_values_close(
            &result.plots[15].values,
            &[std::f64::consts::PI
                + std::f64::consts::E
                + 1.618_033_988_749_895
                + 0.618_033_988_749_894_8; 4],
        );
        assert_values_close(
            &result.plots[16].values,
            &[1.0_f64.sin(), 2.0_f64.sin(), 3.0_f64.sin(), 4.0_f64.sin()],
        );
        assert_values_close(
            &result.plots[17].values,
            &[1.0_f64.cos(), 2.0_f64.cos(), 3.0_f64.cos(), 4.0_f64.cos()],
        );
        assert_values_close(
            &result.plots[18].values,
            &[1.0_f64.tan(), 2.0_f64.tan(), 3.0_f64.tan(), 4.0_f64.tan()],
        );
        assert_values_close(&result.plots[19].values, &[1.0, 4.0, 9.0, 16.0]);
        assert_values_close(&result.plots[20].values, &[0.33, 0.67, 1.0, 1.33]);
        assert_eq!(result.plots[21].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[22].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[23].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[24].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[25].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[26].values, vec![PineValue::Na; 4]);
        assert_eq!(result.plots[27].values, vec![PineValue::Na; 4]);
    }

    #[test]
    fn runs_math_sum_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("math sum")
value = math.sum(close, 3)
with_na = math.sum(bar_index == 3 ? na : close, 3)
invalid = math.sum(close, 0)
plot(value)
plot(with_na)
plot(invalid)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[7.0, 14.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values[1], PineValue::Na);
        assert_values_close(&result.plots[1].values[2..3], &[7.0]);
        assert_eq!(result.plots[1].values[3], PineValue::Na);
        assert_eq!(result.plots[2].values, vec![PineValue::Na; 4]);
    }

    #[test]
    fn profiles_runtime_storage() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("profile")
ma = ta.sma(close, 2)
plot(ma)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let profiled =
            run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(profiled.profile.bars, 3);
        assert_eq!(profiled.profile.series_buffers, 0);
        assert_eq!(profiled.profile.series_values, 0);
        assert!(profiled.profile.series_capacity >= profiled.profile.series_values);
        assert_eq!(profiled.profile.max_series_depth, 0);
        assert_eq!(
            profiled.profile.history_retention_mode,
            HistoryRetentionMode::StaticTrimmed
        );
        assert_eq!(profiled.profile.history_max_constant_offset, 0);
        assert_eq!(profiled.profile.history_max_bars_back, None);
        assert!(!profiled.profile.history_has_dynamic_offsets);
        assert_eq!(profiled.profile.rolling_window_slots, 1);
        assert_eq!(profiled.profile.rolling_window_values, 2);
        assert!(
            profiled.profile.rolling_window_value_capacity
                >= profiled.profile.rolling_window_values
        );
        assert_eq!(profiled.profile.plots, 1);
        assert_eq!(profiled.profile.plot_values, 3);
        assert!(profiled.profile.plot_capacity >= profiled.profile.plot_values);
        assert_eq!(profiled.profile.plot_shapes, 0);
        assert_eq!(profiled.profile.plot_arrows, 0);
        assert_eq!(profiled.profile.plot_bars, 0);
        assert_eq!(profiled.profile.plot_candles, 0);
        assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
        assert_values_close(&profiled.result.plots[0].values[1..], &[1.5, 2.5]);
    }

    #[test]
    fn trims_constant_history_to_required_depth() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("static history")
plot(close[2])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let profiled =
            run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(profiled.result.plots.len(), 1);
        assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
        assert_eq!(profiled.result.plots[0].values[1], PineValue::Na);
        assert_values_close(&profiled.result.plots[0].values[2..], &[1.0, 2.0]);
        assert_eq!(profiled.profile.max_series_depth, 2);
        assert_eq!(profiled.profile.series_values, 2);
        assert_eq!(
            profiled.profile.history_retention_mode,
            HistoryRetentionMode::StaticTrimmed
        );
        assert_eq!(profiled.profile.history_max_constant_offset, 2);
        assert_eq!(profiled.profile.history_max_bars_back, None);
        assert!(!profiled.profile.history_has_dynamic_offsets);
    }

    #[test]
    fn keeps_full_history_when_dynamic_offsets_exist() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("dynamic history retention")
length = input.int(1, "Length")
plot(close[length])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let profiled =
            run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(profiled.result.plots.len(), 1);
        assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
        assert_values_close(&profiled.result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
        assert_eq!(profiled.profile.max_series_depth, 4);
        assert!(profiled.profile.series_values >= 4);
        assert_eq!(
            profiled.profile.history_retention_mode,
            HistoryRetentionMode::DynamicFull
        );
        assert_eq!(profiled.profile.history_max_constant_offset, 0);
        assert_eq!(profiled.profile.history_max_bars_back, None);
        assert!(profiled.profile.history_has_dynamic_offsets);
    }

    #[test]
    fn max_bars_back_bounds_dynamic_history_retention() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("dynamic history retention", max_bars_back=2)
offset = bar_index == 0 ? 0 : 3
plot(close[offset])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let profiled =
            run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(profiled.result.plots.len(), 1);
        assert_eq!(profiled.result.plots[0].values[0], PineValue::Float(1.0));
        assert_eq!(profiled.result.plots[0].values[1..], vec![PineValue::Na; 3]);
        assert_eq!(profiled.profile.max_series_depth, 2);
        assert_eq!(
            profiled.profile.history_retention_mode,
            HistoryRetentionMode::MaxBarsBack
        );
        assert_eq!(profiled.profile.history_max_bars_back, Some(2));
        assert!(profiled.profile.history_has_dynamic_offsets);
    }

    #[test]
    fn append_bar_matches_full_historical_run() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("incremental")
ma = ta.sma(close, 3)
e = ta.ema(close, 2)
plot(ma)
plot(e)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];

        let full = run_historical(&hir, &bars).expect("full result");
        let mut runtime = HistoricalRuntime::new(&hir);
        for (index, bar) in bars.iter().copied().enumerate() {
            runtime.append_bar(bar).expect("append result");
            assert_eq!(runtime.profile().bars, index + 1);
        }
        let incremental = runtime.result();

        assert_eq!(incremental, full);
    }

    #[test]
    fn bar_update_model_marks_committing_updates() {
        let bar = bar(1.0);

        assert!(BarUpdate::historical(bar).commits_series());
        assert!(BarUpdate::confirmed(bar).commits_series());
        assert!(!BarUpdate::forming(bar).commits_series());
    }

    #[test]
    fn runs_barstate_isfirst_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("barstate")
plot(barstate.isfirst ? 1 : 0)
plot(barstate.isconfirmed ? 1 : 0)
plot(barstate.ishistory ? 1 : 0)
plot(barstate.isrealtime ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
            .expect("runtime result");

        assert_values_close(&result.plots[0].values, &[1.0, 0.0, 0.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn barstate_realtime_flags_track_update_kind() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("barstate realtime")
plot(barstate.isconfirmed ? close : 0)
plot(barstate.ishistory ? close : 0)
plot(barstate.isrealtime ? close : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        let mut runtime = RealtimeRuntime::new(&hir);

        let confirmed = runtime
            .update(BarUpdate::historical(bar(1.0)))
            .expect("historical update");
        assert_values_close(&confirmed.plots[0].values, &[1.0]);
        assert_values_close(&confirmed.plots[1].values, &[1.0]);
        assert_values_close(&confirmed.plots[2].values, &[0.0]);

        let forming = runtime
            .update(BarUpdate::forming(bar(2.0)))
            .expect("forming update");
        assert_values_close(&forming.plots[0].values, &[1.0, 0.0]);
        assert_values_close(&forming.plots[1].values, &[1.0, 0.0]);
        assert_values_close(&forming.plots[2].values, &[0.0, 2.0]);

        let confirmed = runtime
            .update(BarUpdate::confirmed(bar(3.0)))
            .expect("confirmed update");
        assert_values_close(&confirmed.plots[0].values, &[1.0, 3.0]);
        assert_values_close(&confirmed.plots[1].values, &[1.0, 0.0]);
        assert_values_close(&confirmed.plots[2].values, &[0.0, 3.0]);
    }

    #[test]
    fn realtime_forming_updates_roll_back_previous_forming_output() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("realtime")
plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        let mut runtime = RealtimeRuntime::new(&hir);

        let first = runtime
            .update(BarUpdate::historical(bar(1.0)))
            .expect("historical update");
        assert_values_close(&first.plots[0].values, &[1.0]);

        let forming = runtime
            .update(BarUpdate::forming(bar(2.0)))
            .expect("forming update");
        assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);
        assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

        let rolled_back = runtime
            .update(BarUpdate::forming(bar(3.0)))
            .expect("second forming update");
        assert_values_close(&rolled_back.plots[0].values, &[1.0, 3.0]);
        assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

        let confirmed = runtime
            .update(BarUpdate::confirmed(bar(4.0)))
            .expect("confirmed update");
        assert_values_close(&confirmed.plots[0].values, &[1.0, 4.0]);
        assert_eq!(runtime.profile().bars, 2);
        assert_eq!(runtime.confirmed_profile().bars, 2);
    }

    #[test]
    fn realtime_rollback_restores_var_state() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("realtime var")
var x = 0
x := x + 1
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.expect("HIR");
        let mut runtime = RealtimeRuntime::new(&hir);

        runtime
            .update(BarUpdate::historical(bar(1.0)))
            .expect("historical update");

        let forming = runtime
            .update(BarUpdate::forming(bar(2.0)))
            .expect("forming update");
        assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);

        let rolled_back = runtime
            .update(BarUpdate::forming(bar(3.0)))
            .expect("second forming update");
        assert_values_close(&rolled_back.plots[0].values, &[1.0, 2.0]);

        let confirmed = runtime
            .update(BarUpdate::confirmed(bar(4.0)))
            .expect("confirmed update");
        assert_values_close(&confirmed.plots[0].values, &[1.0, 2.0]);
    }

    #[test]
    fn runs_if_else_reassignment_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("if")
x = close
if close > open
    x := close
else
    x := open
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 5.0, 4.0, 5.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
    }

    #[test]
    fn runs_if_reassignment_with_var_state() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("if var")
var x = 0
if close > open
    x := x + 1
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 5.0, 4.0, 5.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0, 2.0]);
    }

    #[test]
    fn runs_block_local_var_initializes_when_first_reached() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("local var")
if close > open
    var seen = 10
    seen := seen + 1
    plot(seen)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[11.0, 12.0]);
    }

    #[test]
    fn runs_for_body_var_persists_across_iterations_and_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for var")
out = 0
for i = 0 to 2
    var count = 0
    count := count + 1
    out := count
plot(out)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 6.0, 9.0]);
    }

    #[test]
    fn runs_udf_local_var_independently_per_callsite() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf var")
counter() =>
    var value = 0
    value := value + 1
    value
plot(counter() + counter())
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn advances_conditional_sma_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional sma")
ma = close
if close > open
    ma := ta.sma(close, 2)
plot(ma)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
            bar_ohlc(5.0, 8.0, 5.0, 8.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 4.0, 7.0]);
    }

    #[test]
    fn advances_conditional_ema_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional ema")
e = close
if close > open
    e := ta.ema(close, 2)
plot(e)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
            bar_ohlc(5.0, 8.0, 5.0, 8.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(
            &result.plots[0].values,
            &[2.0, 2.0, 4.666666666666667, 6.888888888888889],
        );
    }

    #[test]
    fn pads_conditional_plot_with_na_when_branch_is_skipped() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional plot")
if close > open
    plot(close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(
            result.plots[0].values,
            vec![PineValue::Float(2.0), PineValue::Na, PineValue::Float(6.0)]
        );
    }

    #[test]
    fn runs_else_if_branches() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("else if")
x = close
if close > 6
    x := 10.0
else if close > 3
    x := 5.0
else
    x := 1.0
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(2.0), bar(4.0), bar(8.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 5.0, 10.0]);
    }

    #[test]
    fn runs_nested_if_branches() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("nested if")
x = close
if close > open
    if high > close
        x := high
    else
        x := close
else
    x := open
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 3.0, 6.0]);
    }

    #[test]
    fn runs_block_local_declaration_in_if() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("block local")
if close > open
    spread = high - low
    plot(spread)
else
    spread = open - close
    plot(spread)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(4.0, 5.0, 3.0, 2.0),
            bar_ohlc(2.0, 8.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values[..1], &[2.0]);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[4.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_values_close(&result.plots[1].values[1..2], &[2.0]);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
    }

    #[test]
    fn runs_block_local_tuple_declaration_in_if() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("block local tuple")
if close > open
    [hi, lo] = [high, low]
    plot(hi - lo)
else
    [hi, lo] = [open, close]
    plot(hi - lo)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(4.0, 5.0, 3.0, 2.0),
            bar_ohlc(2.0, 8.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values[..1], &[2.0]);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[4.0]);
        assert_eq!(result.plots[1].values[0], PineValue::Na);
        assert_values_close(&result.plots[1].values[1..2], &[2.0]);
        assert_eq!(result.plots[1].values[2], PineValue::Na);
    }

    #[test]
    fn runs_block_local_tuple_declaration_shadowing_outer_symbols() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("tuple shadow")
x = close
y = close
if close > open
    [x, y] = [high, low]
    plot(x - y)
plot(x)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(4.0, 5.0, 3.0, 2.0),
            bar_ohlc(2.0, 8.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values[..1], &[2.0]);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[4.0]);
        assert_values_close(&result.plots[1].values, &[2.0, 2.0, 6.0]);
    }

    #[test]
    fn advances_conditional_tuple_builtin_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional bb")
if close > open
    [basis, upper, lower] = ta.bb(close, 2, 2)
    plot(basis)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
            bar_ohlc(5.0, 8.0, 5.0, 8.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[4.0, 7.0]);
    }

    #[test]
    fn advances_conditional_rsi_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional rsi")
r = close
if close > open
    r := ta.rsi(close, 2)
plot(r)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
    }

    #[test]
    fn advances_conditional_atr_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional atr")
a = close
if close > open
    a := ta.atr(2)
plot(a)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 2.5]);
    }

    #[test]
    fn advances_conditional_dmi_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional dmi")
score = close
if close > open
    [plus, minus, adx] = ta.dmi(3, 2)
    score := plus + minus + adx
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
            bar_ohlc(5.0, 8.0, 5.0, 8.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(
            &result.plots[0].values,
            &[0.0, 2.0, 100.0, 132.14285714285714],
        );
    }

    #[test]
    fn advances_conditional_stoch_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional stoch")
score = close
if close > open
    score := ta.stoch(close, high, low, 2)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 10.0, 0.0, 5.0),
            bar_ohlc(3.0, 100.0, 100.0, 2.0),
            bar_ohlc(4.0, 20.0, 10.0, 15.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 75.0]);
    }

    #[test]
    fn advances_conditional_wpr_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional wpr")
score = close
if close > open
    score := ta.wpr(2)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 10.0, 0.0, 5.0),
            bar_ohlc(3.0, 100.0, 100.0, 2.0),
            bar_ohlc(4.0, 20.0, 10.0, 15.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, -25.0]);
    }

    #[test]
    fn advances_conditional_sar_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional sar")
score = close
if close > open
    score := ta.sar(0.02, 0.02, 0.2)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 10.0, 1.0, 5.0),
            bar_ohlc(3.0, 4.0, 1.0, 2.0),
            bar_ohlc(4.0, 20.0, 10.0, 15.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 1.0]);
    }

    #[test]
    fn advances_conditional_mfi_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional mfi")
score = close
if close > open
    score := ta.mfi(close, 2)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlcv(1.0, 10.0, 1.0, 5.0, 10.0),
            bar_ohlcv(3.0, 4.0, 1.0, 2.0, 10.0),
            bar_ohlcv(4.0, 20.0, 10.0, 15.0, 10.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
    }

    #[test]
    fn advances_conditional_tsi_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional tsi")
score = close
if close > open
    score := ta.tsi(close, 2, 3)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 10.0, 1.0, 5.0),
            bar_ohlc(3.0, 4.0, 1.0, 2.0),
            bar_ohlc(4.0, 20.0, 10.0, 15.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 1.0]);
    }

    #[test]
    fn advances_conditional_cmo_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional cmo")
score = close
if close > open
    score := ta.cmo(close, 1)
plot(score)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 10.0, 1.0, 5.0),
            bar_ohlc(3.0, 4.0, 1.0, 2.0),
            bar_ohlc(4.0, 20.0, 10.0, 15.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
    }

    #[test]
    fn advances_conditional_macd_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional macd")
if close > open
    [macd, signal, hist] = ta.macd(close, 2, 3, 2)
    plot(macd)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(4.0, 6.0, 4.0, 6.0),
            bar_ohlc(5.0, 8.0, 5.0, 8.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(
            &[
                result.plots[0].values[0].clone(),
                result.plots[0].values[2].clone(),
                result.plots[0].values[3].clone(),
            ],
            &[0.0, 0.666666666666667, 0.8888888888888893],
        );
    }

    #[test]
    fn runs_expression_body_function() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf")
double(x) => x * 2
plot(double(close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn runs_function_with_ta_call() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf sma")
smooth(src, len) => ta.sma(src, len)
plot(smooth(close, 2))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[1.5, 2.5, 3.5]);
    }

    #[test]
    fn runs_function_body_with_global_reference() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf global")
bias = 1.5
add_bias(x) => x + bias
plot(add_bias(close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.5, 3.5, 4.5]);
    }

    #[test]
    fn runs_block_body_function() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf block")
spread(hi, lo) =>
    value = hi - lo
    value * 2
plot(spread(high, low))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(2.0, 6.0, 3.0, 5.0),
            bar_ohlc(5.0, 9.0, 4.0, 7.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[4.0, 6.0, 10.0]);
    }

    #[test]
    fn runs_block_body_function_with_ta_call() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf block ta")
smooth2(src, len) =>
    ma = ta.sma(src, len)
    ma * 2
plot(smooth2(close, 2))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.0, 5.0, 7.0]);
    }

    #[test]
    fn runs_if_reassignment_inside_block_body_function() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf if")
select_value(x, y) =>
    result = y
    if x > y
        result := x
    result
plot(select_value(high, close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(4.0, 4.0, 2.0, 5.0),
            bar_ohlc(2.0, 8.0, 4.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 5.0, 8.0]);
    }

    #[test]
    fn runs_for_loop_reassignment() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for")
sum = 0
for i = 0 to 4 by 2
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_descending_for_loop_reassignment() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for desc")
sum = 0
for i = 4 to 0 by 2
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_for_loop_step_that_overshoots_end() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for overshoot")
sum = 0
for i = 0 to 5 by 2
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_for_loop_signed_step_by_range_direction() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for signed step")
sum = 0
for i = 0 to 4 by -2
    sum := sum + i
down = 0
for j = 4 to 0 by -2
    down := down + j
plot(close + sum + down)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[13.0, 14.0, 15.0]);
    }

    #[test]
    fn runs_for_loop_with_series_na_bounds() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for na bounds")
limit = close > 1 ? 3 : na
sum = close > 0 ? 0.0 : 0.0
for i = 0 to limit by 2
    sum := sum + i
value = for j = limit to 0 by 2
    j
plot(close + sum + nz(value))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 5.0, 6.0]);
    }

    #[test]
    fn runs_for_loop_break_and_continue() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for control")
sum = 0
for i = 0 to 5
    if i == 2
        continue
    if i == 4
        break
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[5.0, 6.0, 7.0]);
    }

    #[test]
    fn runs_nested_for_loop_control_on_nearest_loop() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("nested for control")
sum = 0
for outer = 0 to 1
    for inner = 0 to 3
        if inner == 1
            continue
        if inner == 3
            break
        sum := sum + outer + inner
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_for_loop_inside_block_body_function() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf for")
repeat3(x) =>
    result = x * 0
    for i = 0 to 2
        result := result + x
    result
plot(repeat3(close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 6.0, 9.0]);
    }

    #[test]
    fn runs_udf_local_declaration_shadowing_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf shadow")
bump(x) =>
    x = x + 1
    x
plot(bump(close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn runs_udf_loop_counter_shadowing_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf counter shadow")
mix(x) =>
    total = 0
    for x = 0 to 2
        total := total + x
    total + x
plot(mix(close))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[4.0, 5.0, 6.0]);
    }

    #[test]
    fn runs_for_expression_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for expression")
value = for i = 0 to 5
    if i == 2
        continue
    if i == 4
        break
    i * 2
plot(close + value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_tuple_for_expression_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("tuple for expression")
[x, y] = for i = 0 to 3
    if i == 1
        continue
    if i == 3
        break
    [i, i * 2]
plot(close + x + y)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn runs_for_expression_that_reaches_no_result_as_na() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for no result")
value = for i = 0 to 2
    if i >= 0
        continue
    i
plot(nz(value) + close)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_while_loop_reassignment() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while")
i = 0
sum = 0
while i < 5
    i := i + 1
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[16.0, 17.0, 18.0]);
    }

    #[test]
    fn runs_while_loop_break_and_continue() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while control")
i = 0
sum = 0
while i < 6
    i := i + 1
    if i > 1 and i < 3
        continue
    if i > 4
        break
    sum := sum + i
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[9.0, 10.0, 11.0]);
    }

    #[test]
    fn runs_while_loop_with_na_condition() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while na condition")
i = 0
sum = close > 0 ? 0.0 : 0.0
while close > 1 ? i < 3 : na
    sum := sum + i
    i := i + 1
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 5.0, 6.0]);
    }

    #[test]
    fn runs_nested_while_loop_control_on_nearest_loop() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("nested while control")
outer = 0
sum = 0
while outer < 2
    inner = 0
    while inner < 4
        inner := inner + 1
        if inner == 2
            continue
        if inner == 4
            break
        sum := sum + outer + inner
    outer := outer + 1
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[11.0, 12.0, 13.0]);
    }

    #[test]
    fn runs_while_body_var_persists_across_iterations_and_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while local var")
i = 0
total = 0
while i < 2
    var seen = 0
    seen := seen + 1
    total := seen
    i := i + 1
plot(total)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn runs_loops_inside_if_branches() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("loops in if")
sum = close > 0 ? 0.0 : 0.0
if close > 1
    for i = 0 to 2
        sum := sum + i
else
    j = 0
    while j < 2
        sum := sum + open
        j := j + 1
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 0.0, 1.0),
            bar_ohlc(2.0, 3.0, 1.0, 2.0),
            bar_ohlc(3.0, 4.0, 2.0, 3.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[3.0, 5.0, 6.0]);
    }

    #[test]
    fn runs_switch_inside_for_loop() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("switch in for")
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 2
    value = switch i
        0 => close
        1 => high
        => low
    sum := sum + value
plot(sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 0.0, 2.0),
            bar_ohlc(2.0, 5.0, 1.0, 4.0),
            bar_ohlc(3.0, 7.0, 2.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[5.0, 10.0, 15.0]);
    }

    #[test]
    fn advances_stateful_calls_inside_for_loop_body() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("for stateful")
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 1
    sum := sum + nz(ta.sma(close, 2))
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 5.5, 8.5]);
    }

    #[test]
    fn runs_while_loop_inside_block_body_function() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf while")
repeat_until(src, limit) =>
    i = 0
    total = src * 0.0
    while i < limit
        total := total + src
        i := i + 1
    total
plot(repeat_until(close, 2))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn advances_stateful_calls_inside_while_loop_body() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while stateful")
i = 0
sum = close > 0 ? 0.0 : 0.0
while i < 2
    sum := sum + nz(ta.sma(close, 2))
    i := i + 1
plot(close + sum)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 5.5, 8.5]);
    }

    #[test]
    fn rejects_while_loop_that_exceeds_iteration_guard() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("while guard")
while true
    close
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected while guard error");

        assert!(
            error
                .message
                .contains("while loop exceeded maximum iteration count"),
            "{}",
            error.message
        );
    }

    #[test]
    fn runs_float_array_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array ops")
values = array.new_float(2, close)
array.push(values, high)
array.set(values, 0, low)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_float_array_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array methods")
values = array.new_float(2, close)
values.push(high)
values.set(0, low)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_int_array_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("int array ops")
values = array.new_int(2, bar_index)
array.push(values, 10)
array.set(values, 0, 3)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_int_array_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("int array methods")
values = array.new_int(2, bar_index)
values.push(10)
values.set(0, 3)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_bool_array_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bool array ops")
values = array.new_bool(2, close > open)
array.push(values, true)
array.set(values, 0, false)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot((first or last) ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_bool_array_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bool array methods")
values = array.new_bool(2, close > open)
values.push(true)
values.set(0, false)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot((first or last) ? values.size() : 0)
plot(na(missing) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_string_array_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("string array ops")
values = array.new_string(2, "seed")
array.push(values, "tail")
array.set(values, 0, "head")
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
text = str.tostring(values)
plot(first == "head" and last == "tail" ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(text == "[head, seed]" ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_string_array_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("string array methods")
values = array.new_string(2, "seed")
values.push("tail")
values.set(0, "head")
first = values.get(0)
last = values.pop()
missing = values.get(10)
text = str.format("Values {0}", values)
plot(first == "head" and last == "tail" ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(text == "Values [head, seed]" ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_color_array_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("color array ops")
values = array.new_color(2, color.red)
array.push(values, color.green)
array.set(values, 0, color.blue)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first == color.blue and last == color.green ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
    }

    #[test]
    fn runs_color_array_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("color array methods")
values = array.new_color(2, color.red)
values.push(color.green)
values.set(0, color.blue)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first == color.blue and last == color.green ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
    }

    #[test]
    fn runs_array_helper_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array helpers")
values = array.new_int()
array.unshift(values, 2)
array.unshift(values, 1)
first = array.first(values)
last = array.last(values)
shifted = array.shift(values)
empty = array.new_string()
plot(first + last + shifted + array.size(values))
plot(na(array.first(empty)) and na(array.last(empty)) and na(array.shift(empty)) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[5.0, 5.0, 5.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_array_helper_method_calls() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array helper methods")
values = array.new_string()
values.unshift("tail")
values.unshift("head")
first = values.first()
last = values.last()
shifted = values.shift()
colors = array.new_color()
colors.unshift(color.green)
colors.unshift(color.red)
color_first = colors.first()
color_last = colors.last()
color_shifted = colors.shift()
plot(first == "head" and last == "tail" and shifted == "head" ? values.size() : 0)
plot(color_first == color.red and color_last == color.green and color_shifted == color.red ? colors.size() : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 2);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_array_insert_remove_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array insert remove")
ints = array.new_int()
ints.push(1)
ints.push(3)
array.insert(ints, 1, 2)
removed = ints.remove(0)
plot(removed)
plot(ints.get(0) * 10 + ints.get(1))

words = array.new_string()
words.push("a")
words.push("c")
words.insert(1, "b")
word_removed = array.remove(words, 2)
plot(word_removed == "c" and words.join("|") == "a|b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.insert(1, color.green)
color_removed = colors.remove(0)
plot(color_removed == color.red and colors.get(0) == color.green ? 1 : 0)

flags = array.new_bool()
flags.insert(0, true)
plot(flags.remove(0) ? flags.size() : 99)

plot(na(array.remove(flags, 0)) ? 1 : 0)
array.insert(flags, 3, false)
plot(flags.size())

negative = array.from(10, 20, 30)
plot(negative.get(-1) + negative.get(-3))
negative.set(-2, 99)
plot(negative.get(1))
negative.insert(-1, 25)
plot(negative.get(2) * 100 + negative.get(-1))
negative_head = negative.remove(-4)
negative_tail = negative.remove(-1)
plot(negative_head + negative_tail + negative.size())
plot(na(negative.get(-3)) and na(negative.remove(-3)) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 12);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[4].values, &[0.0, 0.0]);
        assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[6].values, &[0.0, 0.0]);
        assert_values_close(&result.plots[7].values, &[40.0, 40.0]);
        assert_values_close(&result.plots[8].values, &[99.0, 99.0]);
        assert_values_close(&result.plots[9].values, &[2530.0, 2530.0]);
        assert_values_close(&result.plots[10].values, &[42.0, 42.0]);
        assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_array_fill_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array fill")
ints = array.new_int(4, 1)
array.fill(ints, 9, 1, 3)
plot(ints.get(0) * 1000 + ints.get(1) * 100 + ints.get(2) * 10 + ints.get(3))
ints.fill(2)
plot(ints.get(0) + ints.get(3))

floats = array.new_float(3, close)
floats.fill(high, 0, 2)
plot(floats.get(0) + floats.get(1) + floats.get(2))

words = array.new_string(3, "a")
words.fill("b", 1, 3)
plot(words.join("|") == "a|b|b" ? 1 : 0)

colors = array.new_color(2, color.red)
colors.fill(color.green)
plot(colors.get(0) == color.green and colors.get(1) == color.green ? 1 : 0)

flags = array.new_bool(2, false)
array.fill(flags, true, 0, 1)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

array.fill(flags, false, -1, 1)
array.fill(flags, false, 0, 3)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 7);
        assert_values_close(&result.plots[0].values, &[1991.0, 1991.0]);
        assert_values_close(&result.plots[1].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[2].values, &[10.0, 15.0]);
        assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_array_from_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array from")
ints = array.from(1, 2, 3)
plot(ints.size())
plot(ints.sum())
ints.push(4)
plot(ints.last())

floats = array.from(1, close, na)
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)

flags = array.from(true, false)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

words = array.from("a", "b")
plot(words.join("|") == "a|b" ? 1 : 0)

colors = array.from(color.red, color.green)
plot(colors.get(0) == color.red and colors.get(1) == color.green ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 8);
        assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
        assert_values_close(&result.plots[1].values, &[6.0, 6.0]);
        assert_values_close(&result.plots[2].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[3].values, &[3.0, 4.0]);
        assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_array_reference_and_copy_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array references")
source = array.new_int()
alias = source
copy = array.copy(source)
method_copy = source.copy()
array.push(alias, 1)
array.push(copy, 2)
method_copy.push(3)
plot(array.size(source))
plot(array.get(source, 0))
plot(array.size(copy))
plot(array.get(copy, 0))
plot(method_copy.size())
plot(method_copy.get(0))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 6);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[5].values, &[3.0, 3.0, 3.0]);
    }

    #[test]
    fn runs_array_search_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array search")
numbers = array.new_int()
array.push(numbers, 2)
array.push(numbers, 3)
array.push(numbers, 2)
plot(array.includes(numbers, 2) ? 1 : 0)
plot(array.indexof(numbers, 2))
plot(array.lastindexof(numbers, 2))
plot(numbers.indexof(9))
array.sort(numbers)
plot(array.binary_search(numbers, 2))
plot(numbers.binary_search(9))
plot(array.binary_search_leftmost(numbers, 4))
plot(array.binary_search_rightmost(numbers, 4))
plot(numbers.binary_search_leftmost(2))
plot(numbers.binary_search_rightmost(2))

truth_flags = array.from(true, true)
plot(array.every(truth_flags) and truth_flags.some() ? 1 : 0)
truth_flags.push(false)
plot(array.every(truth_flags) ? 99 : (array.some(truth_flags) ? 1 : 0))
truth_numbers = array.from(1, -2, 3)
plot(truth_numbers.every() and array.some(truth_numbers) ? 1 : 0)
truth_numbers.push(0)
plot(array.every(truth_numbers) ? 99 : 1)
truth_floats = array.new_float()
truth_floats.push(na)
truth_floats.push(0)
truth_floats.push(close)
plot(array.every(truth_floats) ? 99 : (truth_floats.some() ? 1 : 0))
empty_truth = array.new_bool()
plot(array.every(empty_truth) and not empty_truth.some() ? 1 : 0)
na_truth = array.new_int(2)
plot(array.every(na_truth) ? 99 : (array.some(na_truth) ? 98 : 1))

words = array.new_string()
words.push("a")
words.push("b")
words.push("a")
plot(words.includes("b") ? words.lastindexof("a") : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.includes(color.green) ? colors.indexof(color.green) : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 19);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[0.0, 0.0, 0.0]);
        assert_values_close(&result.plots[2].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[3].values, &[-1.0, -1.0, -1.0]);
        assert_values_close(&result.plots[4].values, &[0.0, 0.0, 0.0]);
        assert_values_close(&result.plots[5].values, &[-1.0, -1.0, -1.0]);
        assert_values_close(&result.plots[6].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[7].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[8].values, &[0.0, 0.0, 0.0]);
        assert_values_close(&result.plots[9].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[10].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[11].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[12].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[13].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[14].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[15].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[16].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[17].values, &[2.0, 2.0, 2.0]);
        assert_values_close(&result.plots[18].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn runs_numeric_array_statistics() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array statistics")
ints = array.new_int()
array.push(ints, 2)
array.push(ints, 5)
array.push(ints, 1)
plot(array.min(ints))
plot(array.max(ints))
plot(array.sum(ints))
plot(array.avg(ints))
plot(array.range(ints))
plot(array.median(ints))
plot(array.percentile_nearest_rank(ints, 50))
plot(ints.percentile_linear_interpolation(75))
plot(array.percentrank(ints, 1))
plot(array.variance(ints, false))
mode_ints = array.from(1, 3, 3, 2, 2)
plot(mode_ints.mode())

floats = array.new_float()
floats.push(close)
floats.push(high)
floats.push(na)
plot(floats.min())
plot(floats.max())
plot(floats.sum())
plot(floats.avg())
plot(floats.range())
plot(floats.median())
plot(floats.percentile_nearest_rank(50))
plot(array.percentile_linear_interpolation(floats, 50))
plot(floats.percentrank(1))
plot(array.variance(floats))
plot(floats.stdev(false))

signs = array.from(-2, 0, 3)
absolutes = signs.abs()
plot(absolutes.get(0) + absolutes.get(1) + absolutes.get(2))
plot(signs.get(0))
float_signs = array.new_float()
float_signs.push(-close)
float_signs.push(na)
float_abs = array.abs(float_signs)
plot(float_abs.get(0))
plot(na(float_abs.get(1)) ? 1 : 0)

standard_values = array.from(2, 4, 4, 4, 5, 5, 7, 9)
standardized = standard_values.standardize()
plot(standardized.get(0))
plot(standardized.get(7))
plot(standard_values.get(0))
standard_with_na = array.from(close, na, high)
standardized_with_na = array.standardize(standard_with_na)
plot(standardized_with_na.size())
plot(na(standardized_with_na.get(1)) ? 1 : 0)

covariance_x = array.from(1, 2, 3)
covariance_y = array.from(1, 5, 7)
plot(array.covariance(covariance_x, covariance_y))
plot(covariance_x.covariance(covariance_y, false))
covariance_with_na_x = array.from(close, na, high)
covariance_with_na_y = array.from(open, close, na)
plot(array.covariance(covariance_with_na_x, covariance_with_na_y))
plot(na(covariance_with_na_x.covariance(covariance_with_na_y, false)) ? 1 : 0)
mismatched_covariance = array.from(1, 2)
plot(na(array.covariance(covariance_x, mismatched_covariance)) ? 1 : 0)

empty = array.new_float()
only_na = array.new_int(2)
empty_standardized = array.standardize(empty)
only_na_standardized = only_na.standardize()
plot(na(array.min(empty)) and na(array.max(only_na)) and na(array.sum(empty)) and na(array.avg(only_na)) and na(array.range(empty)) and na(array.mode(ints)) and na(array.percentile_nearest_rank(empty, 50)) and na(array.percentile_linear_interpolation(ints, 150)) and na(array.percentrank(empty, 0)) and empty_standardized.size() == 0 and only_na_standardized.size() == 0 and na(array.covariance(empty, empty)) and na(array.variance(empty)) and na(only_na.stdev()) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 37);
        assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[1].values, &[5.0, 5.0]);
        assert_values_close(&result.plots[2].values, &[8.0, 8.0]);
        assert_values_close(&result.plots[3].values, &[8.0 / 3.0, 8.0 / 3.0]);
        assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[5].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[6].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[7].values, &[3.5, 3.5]);
        assert_values_close(&result.plots[8].values, &[100.0, 100.0]);
        assert_values_close(&result.plots[9].values, &[13.0 / 3.0, 13.0 / 3.0]);
        assert_values_close(&result.plots[10].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[11].values, &[2.0, 3.0]);
        assert_values_close(&result.plots[12].values, &[4.0, 6.0]);
        assert_values_close(&result.plots[13].values, &[6.0, 9.0]);
        assert_values_close(&result.plots[14].values, &[3.0, 4.5]);
        assert_values_close(&result.plots[15].values, &[2.0, 3.0]);
        assert_values_close(&result.plots[16].values, &[3.0, 4.5]);
        assert_values_close(&result.plots[17].values, &[2.0, 3.0]);
        assert_values_close(&result.plots[18].values, &[3.0, 4.5]);
        assert_values_close(&result.plots[19].values, &[100.0, 100.0]);
        assert_values_close(&result.plots[20].values, &[1.0, 2.25]);
        assert_values_close(&result.plots[21].values, &[2.0_f64.sqrt(), 4.5_f64.sqrt()]);
        assert_values_close(&result.plots[22].values, &[5.0, 5.0]);
        assert_values_close(&result.plots[23].values, &[-2.0, -2.0]);
        assert_values_close(&result.plots[24].values, &[2.0, 3.0]);
        assert_values_close(&result.plots[25].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[26].values, &[-1.5, -1.5]);
        assert_values_close(&result.plots[27].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[28].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[29].values, &[3.0, 3.0]);
        assert_values_close(&result.plots[30].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[31].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[32].values, &[3.0, 3.0]);
        assert_values_close(&result.plots[33].values, &[0.0, 0.0]);
        assert_values_close(&result.plots[34].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[35].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[36].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_array_ordering_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array ordering")
ints = array.new_int()
array.push(ints, 3)
array.push(ints, 1)
array.push(ints, 2)
array.sort(ints)
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
desc_ints = array.from(1, 3, 2)
desc_ints.sort(order.descending)
plot(desc_ints.get(0) * 100 + desc_ints.get(1) * 10 + desc_ints.get(2))
desc_float_special = array.new_float()
desc_float_special.push(na)
desc_float_special.push(close)
desc_float_special.push(high)
desc_float_special.sort(order.descending)
plot(na(desc_float_special.get(0)) and desc_float_special.get(1) == high and desc_float_special.get(2) == close ? 1 : 0)
ints.reverse()
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
unsorted_ints = array.from(30, 10, 20)
sorted_int_indices = unsorted_ints.sort_indices()
plot(sorted_int_indices.get(0) * 100 + sorted_int_indices.get(1) * 10 + sorted_int_indices.get(2))
desc_sorted_int_indices = unsorted_ints.sort_indices(order.descending)
plot(desc_sorted_int_indices.get(0) * 100 + desc_sorted_int_indices.get(1) * 10 + desc_sorted_int_indices.get(2))
plot(unsorted_ints.get(0) * 100 + unsorted_ints.get(1) * 10 + unsorted_ints.get(2))

floats = array.new_float()
floats.push(na)
floats.push(high)
floats.push(close)
floats.sort()
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)
float_indices_source = array.new_float()
float_indices_source.push(na)
float_indices_source.push(high)
float_indices_source.push(close)
float_indices = array.sort_indices(float_indices_source)
plot(float_indices.get(0) * 100 + float_indices.get(1) * 10 + float_indices.get(2))

words = array.new_string()
words.push("b")
words.push("a")
words.push("c")
words.push("")
array.sort(words)
plot(words.get(0) == "a" and words.get(1) == "b" and words.get(2) == "c" and words.get(3) == "" ? 1 : 0)
words.sort(order.descending)
plot(words.get(0) == "" and words.get(1) == "c" and words.get(2) == "b" and words.get(3) == "a" ? 1 : 0)
word_indices = words.sort_indices(order.ascending)
plot(word_indices.get(0) == 3 and word_indices.get(1) == 2 and word_indices.get(2) == 1 and word_indices.get(3) == 0 ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors.reverse()
plot(colors.get(0) == color.green and colors.get(1) == color.red ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 14);
        assert_values_close(&result.plots[0].values, &[123.0, 123.0]);
        assert_values_close(&result.plots[1].values, &[321.0, 321.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[3].values, &[321.0, 321.0]);
        assert_values_close(&result.plots[4].values, &[120.0, 120.0]);
        assert_values_close(&result.plots[5].values, &[21.0, 21.0]);
        assert_values_close(&result.plots[6].values, &[3120.0, 3120.0]);
        assert_values_close(&result.plots[7].values, &[6.0, 9.0]);
        assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[9].values, &[210.0, 210.0]);
        assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
    }

    #[test]
    fn runs_array_join_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array join")
ints = array.new_int()
ints.push(1)
ints.push(2)
plot(array.join(ints, "|") == "1|2" ? 1 : 0)

floats = array.new_float()
floats.push(1.25)
floats.push(2.5)
plot(floats.join() == "1.25,2.5" ? 1 : 0)

flags = array.new_bool()
flags.push(false)
flags.push(true)
plot(array.join(flags, "/") == "false/true" ? 1 : 0)

words = array.new_string()
words.push("a")
words.push("b")
plot(words.join("-") == "a-b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.join("|") == "16711680|32768" ? 1 : 0)

empty = array.new_string()
plot(array.join(empty, "|") == "" ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 6);
        for plot in &result.plots {
            assert_values_close(&plot.values, &[1.0, 1.0]);
        }
    }

    #[test]
    fn rejects_oversized_array_join_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array join limit")
values = array.new_string(410)
array.set(values, 0, str.repeat("x", 100))
plot(str.length(array.join(values, str.repeat("y", 100))))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected array.join limit error");

        assert!(
            error
                .message
                .contains("array.join result cannot exceed 40960 characters"),
            "{}",
            error.message
        );
    }

    #[test]
    fn runs_array_slice_concat_operations() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array slice concat")
ints = array.new_int()
ints.push(1)
ints.push(2)
ints.push(3)
part = array.slice(ints, 1, 3)
part.set(0, 20)
plot(part.size())
plot(part.get(0) + part.get(1))
plot(ints.get(1))

more = array.new_int()
more.push(4)
returned = array.concat(ints, more)
plot(array.size(ints))
plot(array.size(returned))
plot(returned.get(3))

words = array.new_string()
words.push("a")
words.push("b")
words.push("c")
tail = words.slice(1, 3)
extra = array.new_string()
extra.push("d")
words.concat(extra)
plot(tail.join("|") == "b|c" and words.join("|") == "a|b|c|d" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors_tail = colors.slice(1, 2)
colors.concat(colors_tail)
plot(colors.get(2) == color.green ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 8);
        assert_values_close(&result.plots[0].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
        assert_values_close(&result.plots[2].values, &[2.0, 2.0]);
        assert_values_close(&result.plots[3].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[5].values, &[4.0, 4.0]);
        assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
        assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    }

    #[test]
    fn handles_invalid_array_slice_bounds() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array slice bounds")
values = array.new_int()
values.push(1)
plot(na(array.slice(values, -1, 1)) ? 1 : 0)
plot(na(values.slice(1, 3)) ? 1 : 0)
plot(na(array.slice(values, 1, 0)) ? 1 : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let result =
            run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[1.0]);
        assert_values_close(&result.plots[1].values, &[1.0]);
        assert_values_close(&result.plots[2].values, &[1.0]);
    }

    #[test]
    fn rejects_oversized_array_concat_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array concat limit")
left = array.new_int(100000, 1)
right = array.new_int(1, 2)
array.concat(left, right)
plot(array.size(left))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected array.concat limit error");

        assert!(
            error
                .message
                .contains("array.concat cannot exceed 100000 elements"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_oversized_array_insert_result() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array insert limit")
values = array.new_int(100000, 1)
array.insert(values, 0, 2)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected array.insert limit error");

        assert!(
            error
                .message
                .contains("array.insert cannot exceed 100000 elements"),
            "{}",
            error.message
        );
    }

    #[test]
    fn var_float_array_persists_across_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("var array")
var values = array.new_float()
fresh = array.new_float()
array.push(values, close)
array.push(fresh, close)
plot(array.size(values))
plot(array.size(fresh))
plot(array.get(values, 0))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
        assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
        assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    }

    #[test]
    fn handles_float_array_edge_cases() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array edges")
values = array.new_float()
missing = array.get(values, 0)
popped = array.pop(values)
array.set(values, 10, close)
plot(na(missing) ? 1 : 0)
plot(na(popped) ? 1 : 0)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let result =
            run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

        assert_eq!(result.plots.len(), 3);
        assert_values_close(&result.plots[0].values, &[1.0]);
        assert_values_close(&result.plots[1].values, &[1.0]);
        assert_values_close(&result.plots[2].values, &[0.0]);
    }

    #[test]
    fn rejects_negative_float_array_size() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array negative size")
values = array.new_float(-1)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected negative array size error");

        assert!(
            error
                .message
                .contains("array.new_float size cannot be negative"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_oversized_float_array_creation() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array oversized")
values = array.new_float(100001)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected oversized array error");

        assert!(
            error
                .message
                .contains("array.new_float size cannot exceed 100000 elements"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_float_array_push_past_limit() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array push limit")
values = array.new_float(100000)
array.push(values, close)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected array push limit error");

        assert!(
            error
                .message
                .contains("array.push cannot exceed 100000 elements"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_float_array_unshift_past_limit() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array unshift limit")
values = array.new_float(100000)
array.unshift(values, close)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("expected array unshift limit error");

        assert!(
            error
                .message
                .contains("array.unshift cannot exceed 100000 elements"),
            "{}",
            error.message
        );
    }

    #[test]
    fn profiles_float_array_storage() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array profile")
var values = array.new_float()
array.push(values, close)
plot(array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let profiled = run_historical_profiled(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)])
            .expect("profiled runtime result");

        assert_eq!(profiled.profile.array_slots, 1);
        assert_eq!(profiled.profile.array_values, 2);
        assert!(profiled.profile.array_value_capacity >= 2);
    }

    #[test]
    fn runs_readonly_float_array_udf_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("array udf")
first(values) => array.get(values, 0)
var values = array.new_float()
array.push(values, close)
plot(first(values) + array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn runs_readonly_int_array_udf_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("int array udf")
first(values) => array.get(values, 0)
var values = array.new_int()
array.push(values, bar_index)
plot(first(values) + array.size(values))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_readonly_bool_array_udf_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("bool array udf")
first(values) => array.get(values, 0)
var values = array.new_bool()
array.push(values, bar_index == 0)
plot(first(values) ? array.size(values) : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_readonly_string_array_udf_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("string array udf")
first(values) => array.get(values, 0)
var values = array.new_string()
array.push(values, "seed")
plot(first(values) == "seed" ? array.size(values) : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_readonly_color_array_udf_parameter() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("color array udf")
first(values) => array.get(values, 0)
var values = array.new_color()
array.push(values, color.red)
plot(first(values) == color.red ? array.size(values) : 0)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn runs_condition_switch_expression() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("condition switch")
value = switch
    close > open => high
    close < open => low
    => close
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 5.0, 0.0, 2.0),
            bar_ohlc(3.0, 6.0, 1.0, 2.0),
            bar_ohlc(2.0, 7.0, 4.0, 2.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[5.0, 1.0, 2.0]);
    }

    #[test]
    fn runs_selector_switch_expression() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("selector switch")
direction = close > open ? 1 : close < open ? -1 : 0
value = switch direction
    1 => high
    -1 => low
    => close
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 5.0, 0.0, 2.0),
            bar_ohlc(3.0, 6.0, 1.0, 2.0),
            bar_ohlc(2.0, 7.0, 4.0, 2.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[5.0, 1.0, 2.0]);
    }

    #[test]
    fn switch_returns_na_when_no_arm_matches() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("switch no match")
value = switch
    close > open => high
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar_ohlc(2.0, 5.0, 1.0, 2.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values, vec![PineValue::Na]);
    }

    #[test]
    fn stores_expression_history_before_reading_previous_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("expression history")
plot((close + open)[1])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 2.0, 1.0, 2.0),
            bar_ohlc(3.0, 4.0, 3.0, 4.0),
            bar_ohlc(5.0, 6.0, 5.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.0, 7.0]);
    }

    #[test]
    fn runs_input_history_offset() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("input history")
length = input.int(2, "Length")
plot(close[length])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[0].values[1], PineValue::Na);
        assert_values_close(&result.plots[0].values[2..], &[1.0, 2.0]);
    }

    #[test]
    fn runs_simple_history_offset() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("simple history")
var values = array.new_int()
array.push(values, 1)
offset = math.min(array.size(values), 1)
plot(close[offset])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0]);
    }

    #[test]
    fn runs_series_history_offset() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("series history")
offset = bar_index == 0 ? 0 : 1
plot(close[offset])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let profiled =
            run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(profiled.result.plots.len(), 1);
        assert_values_close(&profiled.result.plots[0].values, &[1.0, 1.0, 2.0, 3.0]);
        assert_eq!(profiled.profile.max_series_depth, 4);
    }

    #[test]
    fn series_history_offset_out_of_range_returns_na() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("series history out of range")
plot(close[bar_index + 1])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values, vec![PineValue::Na; 3]);
    }

    #[test]
    fn rejects_negative_dynamic_history_offset_at_runtime() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("negative dynamic history")
values = array.new_int()
offset = array.indexof(values, 1)
plot(close[offset])
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
            .expect_err("runtime should reject negative dynamic history offset");
        assert!(error.message.contains("non-negative"), "{}", error.message);
    }

    #[test]
    fn advances_switch_sma_only_when_arm_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("switch conditional sma")
value = switch
    close > open => ta.sma(close, 2)
    => close
plot(value)
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(0.0, 1.0, 0.0, 1.0),
            bar_ohlc(3.0, 3.0, 2.0, 2.0),
            bar_ohlc(3.0, 4.0, 3.0, 4.0),
            bar_ohlc(5.0, 6.0, 5.0, 6.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[2.0, 2.5, 5.0]);
    }

    #[test]
    fn runs_stateful_call_as_function_argument_once() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf arg")
duplicate(x) => x + x
plot(duplicate(ta.sma(close, 2)))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_values_close(&result.plots[0].values[1..], &[3.0, 5.0, 7.0]);
    }

    #[test]
    fn runs_function_with_named_arguments() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("udf named args")
spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))
"#,
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        let bars = vec![
            bar_ohlc(1.0, 3.0, 1.0, 2.0),
            bar_ohlc(2.0, 6.0, 3.0, 5.0),
            bar_ohlc(5.0, 9.0, 4.0, 7.0),
        ];
        let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

        assert_eq!(result.plots.len(), 1);
        assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
    }

    fn bar(close: f64) -> Bar {
        bar_ohlc(close, close, close, close)
    }

    fn bar_volume(close: f64, volume: f64) -> Bar {
        Bar {
            time: 0,
            open: close,
            high: close,
            low: close,
            close,
            volume,
        }
    }

    fn bar_ohlc(open: f64, high: f64, low: f64, close: f64) -> Bar {
        bar_ohlcv(open, high, low, close, 1.0)
    }

    fn bar_ohlcv(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Bar {
        Bar {
            time: 0,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    fn assert_values_close(actual: &[PineValue], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            let actual = actual
                .as_f64()
                .unwrap_or_else(|| panic!("expected numeric value, got {actual:?}"));
            assert!(
                (actual - expected).abs() < 1e-10,
                "expected {expected}, got {actual}"
            );
        }
    }
}
