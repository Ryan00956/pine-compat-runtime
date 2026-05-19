//! Built-in registry scaffolding.

use pine_ir::{PineType, Qualifier, ValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinSignature {
    pub name: &'static str,
    pub phase: BuiltinPhase,
    pub params: &'static [BuiltinParam],
    pub returns: ReturnSpec,
    pub variadic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinPhase {
    Phase1Core,
    Later,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinParam {
    pub name: &'static str,
    pub accepts: Accepts,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accepts {
    Any,
    Exact(PineType),
    Kind(ValueKind),
    Numeric,
    SeriesFloat,
    SeriesOrSimpleNumeric,
    SeriesOrSimpleNumericOrBool,
    SimpleInt,
    ConstString,
    ConstBool,
    ConstOrInputFloat,
    ColorCompatible,
    PlotOrHLine,
    FloatArray,
    InputDefval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnSpec {
    Fixed(PineType),
    Tuple(&'static [PineType]),
    SameAsArg(usize),
    BoolFromArg(usize),
    ColorFromArg(usize),
    PromotedNumeric,
    FloatFromArg(usize),
    PromotedFloat,
    InputFromArg(usize),
}

const INPUT_INT: PineType = PineType::new(Qualifier::Input, ValueKind::Int);
const INPUT_FLOAT: PineType = PineType::new(Qualifier::Input, ValueKind::Float);
const INPUT_BOOL: PineType = PineType::new(Qualifier::Input, ValueKind::Bool);
const INPUT_COLOR: PineType = PineType::new(Qualifier::Input, ValueKind::Color);
const INPUT_STRING: PineType = PineType::new(Qualifier::Input, ValueKind::String);
const SERIES_FLOAT: PineType = PineType::new(Qualifier::Series, ValueKind::Float);
const SERIES_BOOL: PineType = PineType::new(Qualifier::Series, ValueKind::Bool);
const SERIES_FLOAT_TUPLE: PineType = PineType::new(Qualifier::Series, ValueKind::Tuple);
const PLOT: PineType = PineType::new(Qualifier::Const, ValueKind::Plot);
const HLINE: PineType = PineType::new(Qualifier::Const, ValueKind::HLine);
const VOID: PineType = PineType::new(Qualifier::Const, ValueKind::Void);
const SIMPLE_INT: PineType = PineType::new(Qualifier::Simple, ValueKind::Int);
const SIMPLE_FLOAT_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::FloatArray);

const INDICATOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: false,
    },
    BuiltinParam {
        name: "shorttitle",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "overlay",
        accepts: Accepts::ConstBool,
        optional: true,
    },
];

const INPUT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::InputDefval,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_INT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_FLOAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Float)),
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_BOOL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Bool)),
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Color)),
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_STRING_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::String)),
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const INPUT_SOURCE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "defval",
        accepts: Accepts::SeriesFloat,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const PLOT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "series",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

const COLOR_OUTPUT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const PLOTCHAR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "series",
        accepts: Accepts::SeriesOrSimpleNumericOrBool,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "char",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

const PLOTSHAPE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "series",
        accepts: Accepts::SeriesOrSimpleNumericOrBool,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "location",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "offset",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "text",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "textcolor",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "editable",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "size",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "show_last",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "force_overlay",
        accepts: Accepts::ConstBool,
        optional: true,
    },
];

const PLOTARROW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "series",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "colorup",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "colordown",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "offset",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "minheight",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "maxheight",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "editable",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "show_last",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "force_overlay",
        accepts: Accepts::ConstBool,
        optional: true,
    },
];

const PLOTBAR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "open",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "high",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "low",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "close",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "editable",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "show_last",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const PLOTCANDLE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "open",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "high",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "low",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "close",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "wickcolor",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "editable",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "show_last",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "bordercolor",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const HLINE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "price",
        accepts: Accepts::ConstOrInputFloat,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

const FILL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "plot1",
        accepts: Accepts::PlotOrHLine,
        optional: false,
    },
    BuiltinParam {
        name: "plot2",
        accepts: Accepts::PlotOrHLine,
        optional: false,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

const COLOR_NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "transp",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATH_NUMBER_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "number",
    accepts: Accepts::Numeric,
    optional: false,
}];

const MATH_MIN_MAX_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "a",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "b",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const MATH_POW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "base",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "exponent",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const NA_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "x",
    accepts: Accepts::Any,
    optional: false,
}];

const NZ_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "x",
        accepts: Accepts::Any,
        optional: false,
    },
    BuiltinParam {
        name: "replacement",
        accepts: Accepts::Any,
        optional: true,
    },
];

const TA_SOURCE_LENGTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesFloat,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_SOURCE_OPTIONAL_LENGTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesFloat,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
];

const TA_TWO_SOURCE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source1",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "source2",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
];

const TA_LENGTH_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "length",
    accepts: Accepts::SimpleInt,
    optional: false,
}];

const TA_TR_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "handle_na",
    accepts: Accepts::ConstBool,
    optional: true,
}];

const TA_MACD_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesFloat,
        optional: false,
    },
    BuiltinParam {
        name: "fastlen",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "slowlen",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "siglen",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_BB_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesFloat,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "mult",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const ARRAY_NEW_FLOAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "size",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::Numeric,
        optional: true,
    },
];

const ARRAY_SIZE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::FloatArray,
    optional: false,
}];

const ARRAY_VALUE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatArray,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const ARRAY_INDEX_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatArray,
        optional: false,
    },
    BuiltinParam {
        name: "index",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const ARRAY_SET_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatArray,
        optional: false,
    },
    BuiltinParam {
        name: "index",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const THREE_SERIES_FLOATS: &[PineType] = &[SERIES_FLOAT, SERIES_FLOAT, SERIES_FLOAT];

pub const PHASE_1_BUILTINS: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "indicator",
        phase: BuiltinPhase::Phase1Core,
        params: INDICATOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "input",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_PARAMS,
        returns: ReturnSpec::InputFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.int",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_INT_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.float",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_FLOAT_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.bool",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_BOOL_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.source",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.color",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_COLOR),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.string",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_STRING_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.price",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_FLOAT_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.time",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_INT_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.symbol",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_STRING_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.timeframe",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_STRING_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "plot",
        phase: BuiltinPhase::Phase1Core,
        params: PLOT_PARAMS,
        returns: ReturnSpec::Fixed(PLOT),
        variadic: false,
    },
    BuiltinSignature {
        name: "bgcolor",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_OUTPUT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "barcolor",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_OUTPUT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "plotchar",
        phase: BuiltinPhase::Phase1Core,
        params: PLOTCHAR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "plotshape",
        phase: BuiltinPhase::Phase1Core,
        params: PLOTSHAPE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "plotarrow",
        phase: BuiltinPhase::Phase1Core,
        params: PLOTARROW_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "plotbar",
        phase: BuiltinPhase::Phase1Core,
        params: PLOTBAR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "plotcandle",
        phase: BuiltinPhase::Phase1Core,
        params: PLOTCANDLE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "hline",
        phase: BuiltinPhase::Phase1Core,
        params: HLINE_PARAMS,
        returns: ReturnSpec::Fixed(HLINE),
        variadic: false,
    },
    BuiltinSignature {
        name: "fill",
        phase: BuiltinPhase::Phase1Core,
        params: FILL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "color.new",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_NEW_PARAMS,
        returns: ReturnSpec::ColorFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.abs",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.max",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_MIN_MAX_PARAMS,
        returns: ReturnSpec::PromotedNumeric,
        variadic: true,
    },
    BuiltinSignature {
        name: "math.min",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_MIN_MAX_PARAMS,
        returns: ReturnSpec::PromotedNumeric,
        variadic: true,
    },
    BuiltinSignature {
        name: "math.floor",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.ceil",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.sqrt",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.log",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.log10",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.exp",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.sin",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.cos",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.tan",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.pow",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_POW_PARAMS,
        returns: ReturnSpec::PromotedFloat,
        variadic: false,
    },
    BuiltinSignature {
        name: "math.round",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "na",
        phase: BuiltinPhase::Phase1Core,
        params: NA_PARAMS,
        returns: ReturnSpec::BoolFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "nz",
        phase: BuiltinPhase::Phase1Core,
        params: NZ_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.new_float",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NEW_FLOAT_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.size",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.push",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.get",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_INDEX_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.set",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SET_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.pop",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.clear",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.sma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.ema",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.rma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.rsi",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.macd",
        phase: BuiltinPhase::Phase1Core,
        params: TA_MACD_PARAMS,
        returns: ReturnSpec::Tuple(THREE_SERIES_FLOATS),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.bb",
        phase: BuiltinPhase::Phase1Core,
        params: TA_BB_PARAMS,
        returns: ReturnSpec::Tuple(THREE_SERIES_FLOATS),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.tr",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TR_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.atr",
        phase: BuiltinPhase::Phase1Core,
        params: TA_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.change",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_OPTIONAL_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.cross",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TWO_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.crossover",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TWO_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.crossunder",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TWO_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.highest",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.lowest",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
];

#[must_use]
pub fn is_phase_1_builtin(name: &str) -> bool {
    PHASE_1_BUILTINS
        .iter()
        .any(|signature| signature.name == name)
}

#[must_use]
pub fn get_phase_1_builtin(name: &str) -> Option<&'static BuiltinSignature> {
    PHASE_1_BUILTINS
        .iter()
        .find(|signature| signature.name == name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedColor {
    pub name: &'static str,
    pub rgb: u32,
}

pub const NAMED_COLORS: &[NamedColor] = &[
    NamedColor {
        name: "color.black",
        rgb: 0x000000,
    },
    NamedColor {
        name: "color.silver",
        rgb: 0xC0C0C0,
    },
    NamedColor {
        name: "color.gray",
        rgb: 0x808080,
    },
    NamedColor {
        name: "color.white",
        rgb: 0xFFFFFF,
    },
    NamedColor {
        name: "color.maroon",
        rgb: 0x800000,
    },
    NamedColor {
        name: "color.red",
        rgb: 0xFF0000,
    },
    NamedColor {
        name: "color.purple",
        rgb: 0x800080,
    },
    NamedColor {
        name: "color.fuchsia",
        rgb: 0xFF00FF,
    },
    NamedColor {
        name: "color.green",
        rgb: 0x008000,
    },
    NamedColor {
        name: "color.lime",
        rgb: 0x00FF00,
    },
    NamedColor {
        name: "color.olive",
        rgb: 0x808000,
    },
    NamedColor {
        name: "color.yellow",
        rgb: 0xFFFF00,
    },
    NamedColor {
        name: "color.navy",
        rgb: 0x000080,
    },
    NamedColor {
        name: "color.blue",
        rgb: 0x0000FF,
    },
    NamedColor {
        name: "color.teal",
        rgb: 0x008080,
    },
    NamedColor {
        name: "color.aqua",
        rgb: 0x00FFFF,
    },
    NamedColor {
        name: "color.orange",
        rgb: 0xFF9900,
    },
];

#[must_use]
pub fn named_color(name: &str) -> Option<u32> {
    NAMED_COLORS
        .iter()
        .find(|color| color.name == name)
        .map(|color| color.rgb)
}

const NAMED_STRING_CONSTANTS: &[&str] = &[
    "shape.xcross",
    "shape.cross",
    "shape.circle",
    "shape.triangleup",
    "shape.triangledown",
    "shape.flag",
    "shape.arrowup",
    "shape.arrowdown",
    "shape.square",
    "shape.diamond",
    "shape.labelup",
    "shape.labeldown",
    "location.abovebar",
    "location.belowbar",
    "location.top",
    "location.bottom",
    "location.absolute",
    "size.auto",
    "size.tiny",
    "size.small",
    "size.normal",
    "size.large",
    "size.huge",
    "display.all",
    "display.none",
];

#[must_use]
pub fn named_string_constant(name: &str) -> Option<&'static str> {
    NAMED_STRING_CONSTANTS
        .iter()
        .copied()
        .find(|constant| *constant == name)
}

#[must_use]
pub fn fallback_bool_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Bool)
}

#[must_use]
pub fn color_return_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Color)
}

#[must_use]
pub fn input_return_for_arg(arg_type: PineType) -> Option<PineType> {
    if arg_type.qualifier != Qualifier::Const {
        return None;
    }
    match arg_type.kind {
        ValueKind::Int
        | ValueKind::Float
        | ValueKind::Bool
        | ValueKind::String
        | ValueKind::Color => Some(PineType::new(Qualifier::Input, arg_type.kind)),
        _ => None,
    }
}

#[must_use]
pub const fn tuple_return_type() -> PineType {
    SERIES_FLOAT_TUPLE
}
