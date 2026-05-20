//! Historical runtime scaffolding.

use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt,
    HirStmtKind, HirUnaryOp, SeriesId, SymbolId, VarSlotId,
};
use regex::Regex;

const MAX_WHILE_ITERATIONS: usize = 100_000;
const MAX_ARRAY_ELEMENTS: usize = 100_000;
const MAX_STRING_CHARS: usize = 40_960;

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

    pub fn commit(&mut self, series_id: SeriesId, value: PineValue) {
        self.buffers.entry(series_id).or_default().push(value);
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
    series_store: SeriesStore,
    current_symbols: HashMap<SymbolId, PineValue>,
    current_series: HashMap<SeriesId, PineValue>,
    var_store: HashMap<VarSlotId, PineValue>,
    array_store: HashMap<u32, Vec<PineValue>>,
    next_array_id: u32,
    call_state: HashMap<CallSiteId, PineValue>,
    rolling_windows: HashMap<CallSiteId, RollingWindowState>,
    rsi_state: HashMap<CallSiteId, RsiState>,
    macd_state: HashMap<CallSiteId, MacdState>,
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

#[derive(Debug, Default, Clone, PartialEq)]
struct RollingWindowState {
    values: VecDeque<Option<f64>>,
    sum: f64,
    sum_squares: f64,
    na_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StmtControl {
    None,
    Break,
    Continue,
}

impl<'a> HistoricalRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self {
            program,
            bars: 0,
            series_store: SeriesStore::new(),
            current_symbols: HashMap::new(),
            current_series: HashMap::new(),
            var_store: HashMap::new(),
            array_store: HashMap::new(),
            next_array_id: 0,
            call_state: HashMap::new(),
            rolling_windows: HashMap::new(),
            rsi_state: HashMap::new(),
            macd_state: HashMap::new(),
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
        let bar_index = self.bars;
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
        self.commit_current_series();
        self.bars += 1;
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
        let array_values = self.array_store.values().map(Vec::len).sum::<usize>();
        let array_value_capacity = self.array_store.values().map(Vec::capacity).sum::<usize>();

        RuntimeProfile {
            bars: self.bars,
            series_buffers,
            series_values,
            series_capacity,
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
        let builtins = [
            ("open", PineValue::Float(bar.open)),
            ("high", PineValue::Float(bar.high)),
            ("low", PineValue::Float(bar.low)),
            ("close", PineValue::Float(bar.close)),
            ("volume", PineValue::Float(bar.volume)),
            ("time", PineValue::Int(bar.time)),
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

    fn commit_current_series(&mut self) {
        for raw_series_id in 0..self.program.next_series_id {
            let series_id = SeriesId(raw_series_id);
            let value = self
                .current_series
                .remove(&series_id)
                .unwrap_or(PineValue::Na);
            self.series_store.commit(series_id, value);
        }
    }

    fn eval_expr(&mut self, expr: &HirExpr) -> Result<PineValue, RuntimeError> {
        let value = match &expr.kind {
            HirExprKind::Literal(literal) => eval_literal(literal),
            HirExprKind::Symbol(symbol) => self
                .current_symbols
                .get(symbol)
                .cloned()
                .unwrap_or(PineValue::Na),
            HirExprKind::Builtin(name) => eval_builtin_value(name),
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
            HirExprKind::History { expr, offset } => {
                if *offset == 0 {
                    self.eval_expr(expr)?
                } else if let Some(series_id) = expr.series_id {
                    self.series_store.read(series_id, *offset as usize)
                } else {
                    PineValue::Na
                }
            }
        };

        if let Some(series_id) = expr.series_id {
            self.current_series.insert(series_id, value.clone());
        }

        Ok(value)
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
            | "input.timeframe" | "input.source" => self.eval_expr(&args[0].value),
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
            "str.format_time" => self.eval_str_format_time(args),
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
            "ta.sma" => self.eval_sma(call_site_id, args),
            "ta.ema" => self.eval_ema(call_site_id, args),
            "ta.rma" => self.eval_rma(call_site_id, args),
            "ta.rsi" => self.eval_rsi(call_site_id, args),
            "ta.macd" => self.eval_macd(call_site_id, args),
            "ta.bb" => self.eval_bb(call_site_id, args),
            "ta.tr" => self.eval_tr(args),
            "ta.atr" => self.eval_atr(call_site_id, args),
            "ta.change" => self.eval_change(args),
            "ta.cross" => self.eval_cross(args, CrossMode::Any),
            "ta.crossover" => self.eval_cross(args, CrossMode::Over),
            "ta.crossunder" => self.eval_cross(args, CrossMode::Under),
            "ta.highest" => self.eval_window_extreme(call_site_id, args, WindowExtreme::Highest),
            "ta.lowest" => self.eval_window_extreme(call_site_id, args, WindowExtreme::Lowest),
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
            "array.size" => self.eval_array_size(args),
            "array.push" => self.eval_array_push(args),
            "array.get" => self.eval_array_get(args),
            "array.set" => self.eval_array_set(args),
            "array.pop" => self.eval_array_pop(args),
            "array.clear" => self.eval_array_clear(args),
            _ => Err(RuntimeError {
                message: format!("unsupported runtime call `{callee}`"),
            }),
        }
    }

    fn eval_array_new_float(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let size = if let Some(size_arg) = args.first() {
            let Some(size) = self.eval_expr(&size_arg.value)?.as_i64() else {
                return Ok(PineValue::Na);
            };
            if size < 0 {
                return Err(RuntimeError {
                    message: "array.new_float size cannot be negative".to_owned(),
                });
            }
            let size = size as usize;
            if size > MAX_ARRAY_ELEMENTS {
                return Err(RuntimeError {
                    message: format!(
                        "array.new_float size cannot exceed {MAX_ARRAY_ELEMENTS} elements"
                    ),
                });
            }
            size
        } else {
            0
        };

        let initial_value = if let Some(value_arg) = args.get(1) {
            match self.eval_expr(&value_arg.value)? {
                PineValue::Int(value) => PineValue::Float(value as f64),
                PineValue::Float(value) => PineValue::Float(value),
                PineValue::Na => PineValue::Na,
                _ => PineValue::Na,
            }
        } else {
            PineValue::Na
        };

        let id = self.next_array_id;
        self.next_array_id += 1;
        self.array_store.insert(id, vec![initial_value; size]);
        Ok(PineValue::Array(id))
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
        let value = self.eval_float_array_value(&args[1].value)?;
        let PineValue::Array(id) = id else {
            return Ok(PineValue::Void);
        };
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
        if index < 0 {
            return Ok(PineValue::Na);
        }
        Ok(self
            .array_store
            .get(&id)
            .and_then(|values| values.get(index as usize))
            .cloned()
            .unwrap_or(PineValue::Na))
    }

    fn eval_array_set(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let index = self.eval_expr(&args[1].value)?.as_i64();
        let value = self.eval_float_array_value(&args[2].value)?;
        let (PineValue::Array(id), Some(index)) = (id, index) else {
            return Ok(PineValue::Void);
        };
        if index < 0 {
            return Ok(PineValue::Void);
        }
        if let Some(slot) = self
            .array_store
            .get_mut(&id)
            .and_then(|values| values.get_mut(index as usize))
        {
            *slot = value;
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

    fn eval_float_array_value(&mut self, expr: &HirExpr) -> Result<PineValue, RuntimeError> {
        Ok(match self.eval_expr(expr)? {
            PineValue::Int(value) => PineValue::Float(value as f64),
            PineValue::Float(value) => PineValue::Float(value),
            PineValue::Na => PineValue::Na,
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
        let variance = window.variance(length);
        let dev = mult * variance.sqrt();

        Ok(PineValue::Tuple(vec![
            PineValue::Float(basis),
            PineValue::Float(basis + dev),
            PineValue::Float(basis - dev),
        ]))
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

        let Some(current) = current.as_f64() else {
            return Ok(PineValue::Na);
        };
        let Some(series_id) = args[0].value.series_id else {
            return Ok(PineValue::Na);
        };
        let previous = self.series_store.read(series_id, length as usize);
        let Some(previous) = previous.as_f64() else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Float(current - previous))
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

    fn eval_window_extreme(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        mode: WindowExtreme,
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

        let value = window.extreme(mode).unwrap_or(f64::NAN);
        Ok(PineValue::Float(value))
    }

    fn update_rolling_window(
        &mut self,
        call_site_id: CallSiteId,
        source: PineValue,
        length: usize,
    ) -> &RollingWindowState {
        let source = source.as_f64();
        let window = self.rolling_windows.entry(call_site_id).or_default();
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
        let Some(datetime) = Utc.timestamp_millis_opt(timestamp).single() else {
            return Err(RuntimeError {
                message: format!("str.format_time timestamp is out of range: {timestamp}"),
            });
        };

        let result = format_utc_datetime(datetime, &format);
        self.string_value_or_error(result, "str.format_time")
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

    fn previous_close(&self) -> Option<f64> {
        let symbol = self
            .program
            .symbols
            .iter()
            .find(|symbol| symbol.name == "close")?;
        let series_id = symbol.series_id?;
        self.series_store.read(series_id, 1).as_f64()
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
                runtime.append_bar(update.bar)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar(update.bar)?;
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

    fn variance(&self, length: usize) -> f64 {
        let mean = self.mean(length);
        (self.sum_squares / length as f64 - mean * mean).max(0.0)
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

fn eval_builtin_value(name: &str) -> PineValue {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorComponent {
    Red,
    Green,
    Blue,
    Transparency,
}

fn color_component(color: u32, component: ColorComponent) -> f64 {
    let (rgb, alpha) = if color > 0xFF_FFFF {
        (color >> 8, color & 0xFF)
    } else {
        (color, 0xFF)
    };

    match component {
        ColorComponent::Red => ((rgb >> 16) & 0xFF) as f64,
        ColorComponent::Green => ((rgb >> 8) & 0xFF) as f64,
        ColorComponent::Blue => (rgb & 0xFF) as f64,
        ColorComponent::Transparency => (100.0 - (alpha as f64 * 100.0 / 255.0)).round(),
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
        "" | "format.mintick" => "#.########",
        "format.percent" => "#.##%",
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
enabled = time >= start and symbol == "AAPL" and timeframe == "D"
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
    plotchar(close > 2, char="x", color=color.green)
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
    plotshape(close > 2, style=shape.triangleup, location=location.belowbar, color=color.green, text="Buy", textcolor=color.white, size=size.small)
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
    plotarrow(close - 2, colorup=color.green, colordown=color.red, minheight=5, maxheight=20)
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
    plotbar(open, high, low, close, color=color.green)
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
    plotcandle(open, high, low, close, color=color.green, wickcolor=color.white, bordercolor=color.red)
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
    fn runs_change_over_historical_bars() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("change")
c1 = ta.change(close)
c2 = ta.change(close, 2)
plot(c1)
plot(c2)
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
    fn runs_color_new_and_named_colors() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("colors")
c = color.new(color.red, 50)
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
bgcolor(custom)
plot(na(c) ? 0 : 1)
plot(opaque == color.new(color.blue, 0) ? 1 : 0)
plot(channels)
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
plot(text_bool == "true" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20 Percent 3.45%" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(na(missing_match_regex) ? 1 : 0)
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
        assert!(profiled.profile.series_buffers >= 10);
        assert!(profiled.profile.series_values >= 30);
        assert!(profiled.profile.series_capacity >= profiled.profile.series_values);
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

    fn bar_ohlc(open: f64, high: f64, low: f64, close: f64) -> Bar {
        Bar {
            time: 0,
            open,
            high,
            low,
            close,
            volume: 1.0,
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
