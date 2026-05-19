//! Historical runtime scaffolding.

use std::collections::{HashMap, VecDeque};

use pine_ir::{
    CallSiteId, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt,
    HirStmtKind, HirUnaryOp, SeriesId, SymbolId, VarSlotId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PineValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
    Plot(u32),
    HLine(u32),
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
    call_state: HashMap<CallSiteId, PineValue>,
    rolling_windows: HashMap<CallSiteId, RollingWindowState>,
    rsi_state: HashMap<CallSiteId, RsiState>,
    macd_state: HashMap<CallSiteId, MacdState>,
    plots: Vec<PlotSeries>,
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
            call_state: HashMap::new(),
            rolling_windows: HashMap::new(),
            rsi_state: HashMap::new(),
            macd_state: HashMap::new(),
            plots: Vec::new(),
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
            self.eval_stmt(statement)?;
        }

        self.finalize_plot_values();
        self.commit_current_series();
        self.bars += 1;
        Ok(())
    }

    fn eval_stmt(&mut self, statement: &HirStmt) -> Result<(), RuntimeError> {
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
                    _ => return Ok(()),
                };
                for statement in branch {
                    self.eval_stmt(statement)?;
                }
            }
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
                    return Ok(());
                };
                for (symbol, value) in symbols.iter().zip(values) {
                    self.set_symbol_value(*symbol, value);
                }
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        RuntimeResult {
            plots: self.plots.clone(),
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
            HirExprKind::Tuple(items) => PineValue::Tuple(
                items
                    .iter()
                    .map(|item| self.eval_expr(item))
                    .collect::<Result<_, _>>()?,
            ),
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

    fn eval_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        match callee {
            "indicator" => Ok(PineValue::Void),
            "input.int" | "input.float" | "input.bool" | "input.color" | "input.source" => {
                self.eval_expr(&args[0].value)
            }
            "plot" => {
                let value = self.eval_expr(&args[0].value)?;
                self.push_plot_value(call_site_id.0, value);
                Ok(PineValue::Plot(call_site_id.0))
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
            "math.abs" => self.eval_math_abs(args),
            "math.max" => self.eval_math_extreme(args, MathExtreme::Max),
            "math.min" => self.eval_math_extreme(args, MathExtreme::Min),
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
            _ => Err(RuntimeError {
                message: format!("unsupported runtime call `{callee}`"),
            }),
        }
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
        let transp = self.eval_expr(&args[1].value)?.as_i64().unwrap_or(0);
        let PineValue::Color(color) = color else {
            return Ok(PineValue::Na);
        };

        Ok(PineValue::Color(apply_transparency(color, transp)))
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
        match self.eval_expr(&args[0].value)? {
            PineValue::Int(value) => Ok(PineValue::Int(value)),
            PineValue::Float(value) => Ok(PineValue::Float(value.round())),
            PineValue::Na => Ok(PineValue::Na),
            _ => Ok(PineValue::Na),
        }
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

    fn push_plot_value(&mut self, id: u32, value: PineValue) {
        if let Some(plot) = self.plots.iter_mut().find(|plot| plot.id == id) {
            while plot.values.len() < self.bars {
                plot.values.push(PineValue::Na);
            }
            if plot.values.len() == self.bars {
                plot.values.push(value);
            } else if let Some(current) = plot.values.last_mut() {
                *current = value;
            }
        } else {
            let mut values = vec![PineValue::Na; self.bars];
            values.push(value);
            self.plots.push(PlotSeries { id, values });
        }
    }

    fn finalize_plot_values(&mut self) {
        for plot in &mut self.plots {
            while plot.values.len() < self.bars {
                plot.values.push(PineValue::Na);
            }
            if plot.values.len() == self.bars {
                plot.values.push(PineValue::Na);
            }
        }
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

fn eval_builtin_value(name: &str) -> PineValue {
    pine_builtins::named_color(name).map_or(PineValue::Void, PineValue::Color)
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
        HirBinaryOp::Eq => PineValue::Bool(left == right),
        HirBinaryOp::NotEq => PineValue::Bool(left != right),
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
plot(na(c) ? 0 : 1)
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
        assert_eq!(apply_transparency(0xFF0000, 50), 0xFF000080);
    }

    #[test]
    fn runs_selected_math_functions() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
plot(x)
plot(y)
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
    fn advances_conditional_tuple_builtin_only_when_branch_executes() {
        let source = SourceFile::new(
            "test.pine",
            r#"indicator("conditional bb")
[basis, upper, lower] = [close, close, close]
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
        assert_values_close(&result.plots[0].values[1..], &[2.0, 4.0, 7.0]);
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
[macd, signal, hist] = [close, close, close]
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
        assert_values_close(
            &result.plots[0].values,
            &[0.0, 2.0, 0.666666666666667, 0.8888888888888893],
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
