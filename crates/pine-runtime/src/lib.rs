//! Historical runtime scaffolding.

use pine_ir::{CallSiteId, HirExpr, SeriesId, SymbolId, VarSlotId};

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
pub use output::drawings::{LabelOutput, LabelSnapshot};
pub use output::json::{public_runtime_profiled_result_json, public_runtime_result_json};
pub use output::model::{
    ColorSeries, FillOutput, HLineOutput, PUBLIC_OUTPUT_SCHEMA_VERSION, PlotArrowSeries,
    PlotBarSeries, PlotCandleSeries, PlotCharSeries, PlotSeries, PlotShapeSeries,
    RuntimeDiagnostic, RuntimeResult,
};
pub use profile::{RuntimeProfile, RuntimeProfiledResult};
pub use retention::HistoryRetentionMode;
pub use runtime::historical::{HistoricalRuntime, run_historical, run_historical_profiled};
pub use runtime::realtime::RealtimeRuntime;
pub use series::SeriesStore;
pub use value::PineValue;

use algorithms::numeric::finite_float_or_na;
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
use runtime::statements::StmtControl;

#[cfg(test)]
use builtins::colors::apply_transparency;
#[cfg(test)]
use builtins::ta::{PivotPointPeriod, pivot_na_levels, pivot_point_levels};

const MAX_WHILE_ITERATIONS: usize = 100_000;
const MAX_ARRAY_ELEMENTS: usize = 100_000;
const MAX_STRING_CHARS: usize = 40_960;
const MAX_SERIES_HISTORY_VALUES: usize = 1_000_000;
const MAX_LABELS: usize = 500;
const DEFAULT_CHART_TIMEFRAME: &str = "1";

#[cfg(test)]
mod tests;
