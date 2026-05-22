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
    SeriesNumeric,
    SeriesNumericOrBool,
    SeriesOrSimpleNumeric,
    SeriesOrSimpleNumericOrBool,
    SimpleInt,
    SimpleString,
    SimpleNumeric,
    SimpleBool,
    ConstNumeric,
    ConstString,
    ConstBool,
    ConstOrInputFloat,
    ColorCompatible,
    StringCompatible,
    StringConvertible,
    CastScalar,
    StringCastScalar,
    ValueWhenSource,
    NumericOrColorCompatible,
    NumericCompatible,
    IntCompatible,
    BoolCompatible,
    PlotOrHLine,
    Array,
    Tuple,
    NumericArray,
    NumericOrBoolArray,
    NumericOrStringArray,
    InputDefval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnSpec {
    Fixed(PineType),
    Tuple(&'static [PineType]),
    SameAsArg(usize),
    BoolFromArg(usize),
    ColorFromArg(usize),
    PromotedColor,
    PromotedBool,
    PromotedInt,
    PromotedString,
    FloatFromStringArg(usize),
    PromotedNumeric,
    ArrayElement(usize),
    ArrayNumeric(usize),
    ArrayFromArgs,
    IntFromArg(usize),
    FloatFromArg(usize),
    SeriesFromArg(usize),
    ChangeFromArg(usize),
    PromotedFloat,
    Round,
    InputFromArg(usize),
}

const INPUT_INT: PineType = PineType::new(Qualifier::Input, ValueKind::Int);
const INPUT_FLOAT: PineType = PineType::new(Qualifier::Input, ValueKind::Float);
const INPUT_BOOL: PineType = PineType::new(Qualifier::Input, ValueKind::Bool);
const INPUT_COLOR: PineType = PineType::new(Qualifier::Input, ValueKind::Color);
const INPUT_STRING: PineType = PineType::new(Qualifier::Input, ValueKind::String);
const SERIES_FLOAT: PineType = PineType::new(Qualifier::Series, ValueKind::Float);
const SERIES_INT: PineType = PineType::new(Qualifier::Series, ValueKind::Int);
const SERIES_BOOL: PineType = PineType::new(Qualifier::Series, ValueKind::Bool);
const SERIES_STRING: PineType = PineType::new(Qualifier::Series, ValueKind::String);
const SERIES_FLOAT_TUPLE: PineType = PineType::new(Qualifier::Series, ValueKind::Tuple);
const PLOT: PineType = PineType::new(Qualifier::Const, ValueKind::Plot);
const HLINE: PineType = PineType::new(Qualifier::Const, ValueKind::HLine);
const VOID: PineType = PineType::new(Qualifier::Const, ValueKind::Void);
const SIMPLE_INT: PineType = PineType::new(Qualifier::Simple, ValueKind::Int);
const SIMPLE_BOOL: PineType = PineType::new(Qualifier::Simple, ValueKind::Bool);
const SIMPLE_STRING: PineType = PineType::new(Qualifier::Simple, ValueKind::String);
const SIMPLE_FLOAT_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::FloatArray);
const SIMPLE_INT_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::IntArray);
const SIMPLE_BOOL_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::BoolArray);
const SIMPLE_STRING_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::StringArray);
const SIMPLE_COLOR_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::ColorArray);

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
    BuiltinParam {
        name: "max_bars_back",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
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
    BuiltinParam {
        name: "options",
        accepts: Accepts::Tuple,
        optional: true,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "minval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
        optional: true,
    },
    BuiltinParam {
        name: "maxval",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
        optional: true,
    },
    BuiltinParam {
        name: "step",
        accepts: Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
        optional: true,
    },
    BuiltinParam {
        name: "options",
        accepts: Accepts::Tuple,
        optional: true,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "minval",
        accepts: Accepts::ConstNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "maxval",
        accepts: Accepts::ConstNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "step",
        accepts: Accepts::ConstNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "options",
        accepts: Accepts::Tuple,
        optional: true,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "options",
        accepts: Accepts::Tuple,
        optional: true,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const INPUT_TEXT_AREA_PARAMS: &[BuiltinParam] = &[
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
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "inline",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "group",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "confirm",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::StringCompatible,
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
    BuiltinParam {
        name: "linewidth",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "trackprice",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "histbase",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "offset",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "join",
        accepts: Accepts::ConstBool,
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
        name: "format",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "precision",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "force_overlay",
        accepts: Accepts::ConstBool,
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
    BuiltinParam {
        name: "offset",
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
    BuiltinParam {
        name: "location",
        accepts: Accepts::ConstString,
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
    BuiltinParam {
        name: "linestyle",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "linewidth",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "editable",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
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
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
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
        name: "fillgaps",
        accepts: Accepts::ConstBool,
        optional: true,
    },
    BuiltinParam {
        name: "display",
        accepts: Accepts::ConstString,
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
        optional: true,
    },
];

const COLOR_RGB_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "red",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "green",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "blue",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "transp",
        accepts: Accepts::Numeric,
        optional: true,
    },
];

const COLOR_COMPONENT_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "color",
    accepts: Accepts::ColorCompatible,
    optional: false,
}];

const COLOR_FROM_GRADIENT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "value",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "bottom_value",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "top_value",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "bottom_color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "top_color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const MATH_NUMBER_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "number",
    accepts: Accepts::Numeric,
    optional: false,
}];

const TYPE_CAST_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "x",
    accepts: Accepts::CastScalar,
    optional: false,
}];

const STRING_CAST_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "x",
    accepts: Accepts::StringCastScalar,
    optional: false,
}];

const COLOR_CAST_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "x",
    accepts: Accepts::ColorCompatible,
    optional: false,
}];

const STR_TEXT_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "string",
    accepts: Accepts::StringCompatible,
    optional: false,
}];

const STR_SOURCE_SUBSTRING_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "str",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const STR_SOURCE_REGEX_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "regex",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const STR_SPLIT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "separator",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const STR_SUBSTRING_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "begin_pos",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "end_pos",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
];

const STR_REPEAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "repeat",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "separator",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const STR_REPLACE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "target",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "replacement",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "occurrence",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
];

const STR_REPLACE_ALL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "target",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "replacement",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const STR_TOSTRING_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "value",
        accepts: Accepts::StringConvertible,
        optional: false,
    },
    BuiltinParam {
        name: "format",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const STR_FORMAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "formatString",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "arg",
        accepts: Accepts::StringConvertible,
        optional: true,
    },
];

const STR_FORMAT_TIME_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "time",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "format",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "timezone",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const TIME_COMPONENT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "time",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "timezone",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const TIMEFRAME_IN_SECONDS_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "timeframe",
    accepts: Accepts::SimpleString,
    optional: true,
}];

const TIMEFRAME_FROM_SECONDS_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "seconds",
    accepts: Accepts::SimpleInt,
    optional: false,
}];

const TIMEFRAME_CHANGE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "timeframe",
    accepts: Accepts::SimpleString,
    optional: false,
}];

const TIMESTAMP_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "year",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "month",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "day",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "hour",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "minute",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "second",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
];

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

const MATH_AVG_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "number",
    accepts: Accepts::Numeric,
    optional: false,
}];

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

const MATH_HYPOT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "number1",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "number2",
        accepts: Accepts::Numeric,
        optional: false,
    },
];

const MATH_ROUND_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "number",
        accepts: Accepts::Numeric,
        optional: false,
    },
    BuiltinParam {
        name: "precision",
        accepts: Accepts::Kind(ValueKind::Int),
        optional: true,
    },
];

const MATH_RANDOM_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "min",
        accepts: Accepts::Numeric,
        optional: true,
    },
    BuiltinParam {
        name: "max",
        accepts: Accepts::Numeric,
        optional: true,
    },
    BuiltinParam {
        name: "seed",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
];

const MATH_SUM_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
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

const FIXNAN_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "source",
    accepts: Accepts::NumericOrColorCompatible,
    optional: false,
}];

const TA_SOURCE_LENGTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_SOURCE_LENGTH_OFFSET_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "offset",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_PIVOT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "leftbars",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "rightbars",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_PIVOT_POINT_LEVELS_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "type",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "anchor",
        accepts: Accepts::BoolCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "developing",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const TA_ALMA_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "series",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "offset",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "sigma",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "floor",
        accepts: Accepts::SimpleBool,
        optional: true,
    },
];

const TA_SOURCE_ONLY_SERIES_FLOAT_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "source",
    accepts: Accepts::SeriesNumeric,
    optional: false,
}];

const TA_SOURCE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "source",
    accepts: Accepts::SeriesOrSimpleNumeric,
    optional: false,
}];

const TA_VWAP_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "anchor",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "stdev_mult",
        accepts: Accepts::SimpleNumeric,
        optional: true,
    },
];

const TA_SOURCE_LENGTH_BIASED_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "biased",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const TA_SOURCE_OR_SIMPLE_LENGTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_SOURCE_LENGTH_PERCENTAGE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "percentage",
        accepts: Accepts::ConstOrInputFloat,
        optional: false,
    },
];

const TA_CONDITION_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "condition",
    accepts: Accepts::BoolCompatible,
    optional: false,
}];

const TA_VALUEWHEN_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "condition",
        accepts: Accepts::BoolCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "source",
        accepts: Accepts::ValueWhenSource,
        optional: false,
    },
    BuiltinParam {
        name: "occurrence",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_SOURCE_OPTIONAL_LENGTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumericOrBool,
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

const TA_TWO_SOURCE_LENGTH_PARAMS: &[BuiltinParam] = &[
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
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_STOCH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "high",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "low",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_LENGTH_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "length",
    accepts: Accepts::SimpleInt,
    optional: false,
}];

const TA_AO_PARAMS: &[BuiltinParam] = &[];

const TA_BOP_PARAMS: &[BuiltinParam] = &[];

const TA_TR_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "handle_na",
    accepts: Accepts::ConstBool,
    optional: true,
}];

const TA_MACD_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
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

const TA_TSI_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "short_length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "long_length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_BB_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
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

const TA_KC_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "source",
        accepts: Accepts::SeriesNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "length",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "mult",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "useTrueRange",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const TA_SUPERTREND_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "factor",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "atrPeriod",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_DMI_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "diLength",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "adxSmoothing",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const TA_SAR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "start",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "inc",
        accepts: Accepts::SimpleNumeric,
        optional: false,
    },
    BuiltinParam {
        name: "max",
        accepts: Accepts::SimpleNumeric,
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

const ARRAY_NEW_INT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "size",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
];

const ARRAY_NEW_BOOL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "size",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const ARRAY_NEW_STRING_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "size",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const ARRAY_NEW_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "size",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

const ARRAY_FROM_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "value",
    accepts: Accepts::Any,
    optional: false,
}];

const ARRAY_SIZE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::Array,
    optional: false,
}];

const ARRAY_NUMERIC_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::NumericArray,
    optional: false,
}];

const ARRAY_TRUTHY_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::NumericOrBoolArray,
    optional: false,
}];

const ARRAY_SORT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::NumericOrStringArray,
        optional: false,
    },
    BuiltinParam {
        name: "order",
        accepts: Accepts::ConstString,
        optional: true,
    },
];

const ARRAY_VARIANCE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "biased",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const ARRAY_COVARIANCE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id1",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "id2",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "biased",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const ARRAY_PERCENTILE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "percentage",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: false,
    },
];

const ARRAY_PERCENTRANK_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "index",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const ARRAY_JOIN_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "separator",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

const ARRAY_SLICE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "index_from",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "index_to",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const ARRAY_CONCAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "id2",
        accepts: Accepts::Array,
        optional: false,
    },
];

const ARRAY_VALUE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Any,
        optional: false,
    },
];

const ARRAY_NUMERIC_VALUE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::NumericArray,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Any,
        optional: false,
    },
];

const ARRAY_FILL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Any,
        optional: false,
    },
    BuiltinParam {
        name: "index_from",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
    BuiltinParam {
        name: "index_to",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
];

const ARRAY_INDEX_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::Array,
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
        accepts: Accepts::Array,
        optional: false,
    },
    BuiltinParam {
        name: "index",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::Any,
        optional: false,
    },
];

const TWO_SERIES_FLOATS: &[PineType] = &[SERIES_FLOAT, SERIES_FLOAT];
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
        name: "input.session",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_STRING_PARAMS,
        returns: ReturnSpec::Fixed(INPUT_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "input.text_area",
        phase: BuiltinPhase::Phase1Core,
        params: INPUT_TEXT_AREA_PARAMS,
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
        name: "color.rgb",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_RGB_PARAMS,
        returns: ReturnSpec::PromotedColor,
        variadic: false,
    },
    BuiltinSignature {
        name: "color.r",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_COMPONENT_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "color.g",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_COMPONENT_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "color.b",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_COMPONENT_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "color.t",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_COMPONENT_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "color.from_gradient",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_FROM_GRADIENT_PARAMS,
        returns: ReturnSpec::PromotedColor,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.length",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TEXT_PARAMS,
        returns: ReturnSpec::IntFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.upper",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TEXT_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.lower",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TEXT_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.contains",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SOURCE_SUBSTRING_PARAMS,
        returns: ReturnSpec::PromotedBool,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.startswith",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SOURCE_SUBSTRING_PARAMS,
        returns: ReturnSpec::PromotedBool,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.endswith",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SOURCE_SUBSTRING_PARAMS,
        returns: ReturnSpec::PromotedBool,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.pos",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SOURCE_SUBSTRING_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.substring",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SUBSTRING_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.trim",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TEXT_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.repeat",
        phase: BuiltinPhase::Phase1Core,
        params: STR_REPEAT_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.replace",
        phase: BuiltinPhase::Phase1Core,
        params: STR_REPLACE_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.replace_all",
        phase: BuiltinPhase::Phase1Core,
        params: STR_REPLACE_ALL_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.tonumber",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TEXT_PARAMS,
        returns: ReturnSpec::FloatFromStringArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.tostring",
        phase: BuiltinPhase::Phase1Core,
        params: STR_TOSTRING_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.format",
        phase: BuiltinPhase::Phase1Core,
        params: STR_FORMAT_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: true,
    },
    BuiltinSignature {
        name: "str.match",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SOURCE_REGEX_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "str.split",
        phase: BuiltinPhase::Phase1Core,
        params: STR_SPLIT_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "str.format_time",
        phase: BuiltinPhase::Phase1Core,
        params: STR_FORMAT_TIME_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "year",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "month",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "dayofmonth",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "dayofweek",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "weekofyear",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "hour",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "minute",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "second",
        phase: BuiltinPhase::Phase1Core,
        params: TIME_COMPONENT_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "timestamp",
        phase: BuiltinPhase::Phase1Core,
        params: TIMESTAMP_PARAMS,
        returns: ReturnSpec::PromotedInt,
        variadic: false,
    },
    BuiltinSignature {
        name: "timeframe.in_seconds",
        phase: BuiltinPhase::Phase1Core,
        params: TIMEFRAME_IN_SECONDS_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "timeframe.from_seconds",
        phase: BuiltinPhase::Phase1Core,
        params: TIMEFRAME_FROM_SECONDS_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "timeframe.change",
        phase: BuiltinPhase::Phase1Core,
        params: TIMEFRAME_CHANGE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "int",
        phase: BuiltinPhase::Phase1Core,
        params: TYPE_CAST_PARAMS,
        returns: ReturnSpec::IntFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "float",
        phase: BuiltinPhase::Phase1Core,
        params: TYPE_CAST_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "bool",
        phase: BuiltinPhase::Phase1Core,
        params: TYPE_CAST_PARAMS,
        returns: ReturnSpec::BoolFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "string",
        phase: BuiltinPhase::Phase1Core,
        params: STRING_CAST_PARAMS,
        returns: ReturnSpec::PromotedString,
        variadic: false,
    },
    BuiltinSignature {
        name: "color",
        phase: BuiltinPhase::Phase1Core,
        params: COLOR_CAST_PARAMS,
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
        name: "math.avg",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_AVG_PARAMS,
        returns: ReturnSpec::PromotedFloat,
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
        name: "math.trunc",
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
        name: "math.cbrt",
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
        name: "math.acos",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.asin",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.atan",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.sign",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.todegrees",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.toradians",
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
        name: "math.hypot",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_HYPOT_PARAMS,
        returns: ReturnSpec::PromotedFloat,
        variadic: false,
    },
    BuiltinSignature {
        name: "math.round",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_ROUND_PARAMS,
        returns: ReturnSpec::Round,
        variadic: false,
    },
    BuiltinSignature {
        name: "math.round_to_mintick",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_NUMBER_PARAMS,
        returns: ReturnSpec::FloatFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.random",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_RANDOM_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "math.sum",
        phase: BuiltinPhase::Phase1Core,
        params: MATH_SUM_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
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
        name: "fixnan",
        phase: BuiltinPhase::Phase1Core,
        params: FIXNAN_PARAMS,
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
        name: "array.new_int",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NEW_INT_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.new_bool",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NEW_BOOL_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_BOOL_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.new_string",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NEW_STRING_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.new_color",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NEW_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_COLOR_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.from",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_FROM_PARAMS,
        returns: ReturnSpec::ArrayFromArgs,
        variadic: true,
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
        returns: ReturnSpec::ArrayElement(0),
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
        name: "array.insert",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SET_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.pop",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::ArrayElement(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.remove",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_INDEX_PARAMS,
        returns: ReturnSpec::ArrayElement(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.shift",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::ArrayElement(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.unshift",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.fill",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_FILL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.first",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::ArrayElement(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.last",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::ArrayElement(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.copy",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.slice",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SLICE_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.concat",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_CONCAT_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.includes",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.every",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_TRUTHY_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.some",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_TRUTHY_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.indexof",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.lastindexof",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.binary_search",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.binary_search_leftmost",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.binary_search_rightmost",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_VALUE_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.abs",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::SameAsArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.min",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.max",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.sum",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.avg",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.range",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.median",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.mode",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.percentile_nearest_rank",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_PERCENTILE_PARAMS,
        returns: ReturnSpec::ArrayNumeric(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.percentile_linear_interpolation",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_PERCENTILE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.percentrank",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_PERCENTRANK_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.covariance",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_COVARIANCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.standardize",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_NUMERIC_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.variance",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VARIANCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.stdev",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_VARIANCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.sort",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SORT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.sort_indices",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SORT_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.reverse",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "array.join",
        phase: BuiltinPhase::Phase1Core,
        params: ARRAY_JOIN_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_STRING),
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
        name: "ta.dema",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.tema",
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
        name: "ta.tsi",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TSI_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.cmo",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.cci",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.cog",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.ao",
        phase: BuiltinPhase::Phase1Core,
        params: TA_AO_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.bop",
        phase: BuiltinPhase::Phase1Core,
        params: TA_BOP_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
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
        name: "ta.bbw",
        phase: BuiltinPhase::Phase1Core,
        params: TA_BB_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.kc",
        phase: BuiltinPhase::Phase1Core,
        params: TA_KC_PARAMS,
        returns: ReturnSpec::Tuple(THREE_SERIES_FLOATS),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.kcw",
        phase: BuiltinPhase::Phase1Core,
        params: TA_KC_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.pivothigh",
        phase: BuiltinPhase::Phase1Core,
        params: TA_PIVOT_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.pivotlow",
        phase: BuiltinPhase::Phase1Core,
        params: TA_PIVOT_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.pivot_point_levels",
        phase: BuiltinPhase::Phase1Core,
        params: TA_PIVOT_POINT_LEVELS_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.cum",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.max",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.min",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.stdev",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_BIASED_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.variance",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_BIASED_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.range",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.dev",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.vwap",
        phase: BuiltinPhase::Phase1Core,
        params: TA_VWAP_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.vwma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.mfi",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.wma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.hma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.swma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_ONLY_SERIES_FLOAT_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.alma",
        phase: BuiltinPhase::Phase1Core,
        params: TA_ALMA_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.linreg",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_OFFSET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.stoch",
        phase: BuiltinPhase::Phase1Core,
        params: TA_STOCH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.wpr",
        phase: BuiltinPhase::Phase1Core,
        params: TA_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.correlation",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TWO_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.covariance",
        phase: BuiltinPhase::Phase1Core,
        params: TA_TWO_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.median",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_OR_SIMPLE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.mode",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_OR_SIMPLE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.percentile_nearest_rank",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PERCENTAGE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.percentile_linear_interpolation",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PERCENTAGE_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.percentrank",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_OR_SIMPLE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
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
        name: "ta.supertrend",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SUPERTREND_PARAMS,
        returns: ReturnSpec::Tuple(TWO_SERIES_FLOATS),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.dmi",
        phase: BuiltinPhase::Phase1Core,
        params: TA_DMI_PARAMS,
        returns: ReturnSpec::Tuple(THREE_SERIES_FLOATS),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.sar",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SAR_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.change",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_OPTIONAL_LENGTH_PARAMS,
        returns: ReturnSpec::ChangeFromArg(0),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.mom",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.roc",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.rising",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.falling",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOOL),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.barssince",
        phase: BuiltinPhase::Phase1Core,
        params: TA_CONDITION_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.valuewhen",
        phase: BuiltinPhase::Phase1Core,
        params: TA_VALUEWHEN_PARAMS,
        returns: ReturnSpec::SeriesFromArg(1),
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
    BuiltinSignature {
        name: "ta.highestbars",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "ta.lowestbars",
        phase: BuiltinPhase::Phase1Core,
        params: TA_SOURCE_LENGTH_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct NamedFloatConstant {
    name: &'static str,
    value: f64,
}

const NAMED_FLOAT_CONSTANTS: &[NamedFloatConstant] = &[
    NamedFloatConstant {
        name: "math.e",
        value: std::f64::consts::E,
    },
    NamedFloatConstant {
        name: "math.pi",
        value: std::f64::consts::PI,
    },
    NamedFloatConstant {
        name: "math.phi",
        value: 1.618_033_988_749_895,
    },
    NamedFloatConstant {
        name: "math.rphi",
        value: 0.618_033_988_749_894_8,
    },
    NamedFloatConstant {
        name: "syminfo.mintick",
        value: 0.01,
    },
];

#[must_use]
pub fn named_float_constant(name: &str) -> Option<f64> {
    NAMED_FLOAT_CONSTANTS
        .iter()
        .find(|constant| constant.name == name)
        .map(|constant| constant.value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamedIntConstant {
    name: &'static str,
    value: i64,
}

const NAMED_INT_CONSTANTS: &[NamedIntConstant] = &[
    NamedIntConstant {
        name: "dayofweek.sunday",
        value: 1,
    },
    NamedIntConstant {
        name: "dayofweek.monday",
        value: 2,
    },
    NamedIntConstant {
        name: "dayofweek.tuesday",
        value: 3,
    },
    NamedIntConstant {
        name: "dayofweek.wednesday",
        value: 4,
    },
    NamedIntConstant {
        name: "dayofweek.thursday",
        value: 5,
    },
    NamedIntConstant {
        name: "dayofweek.friday",
        value: 6,
    },
    NamedIntConstant {
        name: "dayofweek.saturday",
        value: 7,
    },
];

#[must_use]
pub fn named_int_constant(name: &str) -> Option<i64> {
    NAMED_INT_CONSTANTS
        .iter()
        .find(|constant| constant.name == name)
        .map(|constant| constant.value)
}

const BUILTIN_SERIES_VALUES: &[(&str, PineType)] = &[
    (
        "barstate.isfirst",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.isconfirmed",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.ishistory",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.isrealtime",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    ("timeframe.period", SIMPLE_STRING),
    ("timeframe.isseconds", SIMPLE_BOOL),
    ("timeframe.isminutes", SIMPLE_BOOL),
    ("timeframe.isintraday", SIMPLE_BOOL),
    ("timeframe.isdaily", SIMPLE_BOOL),
    ("timeframe.isweekly", SIMPLE_BOOL),
    ("timeframe.ismonthly", SIMPLE_BOOL),
    ("timeframe.isdwm", SIMPLE_BOOL),
    ("timeframe.multiplier", SIMPLE_INT),
    (
        "ta.accdist",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    ("ta.iii", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.nvi", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.obv", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.pvi", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.pvt", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.tr", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.wad", PineType::new(Qualifier::Series, ValueKind::Float)),
    (
        "ta.vwap",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "ta.wvad",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
];

#[must_use]
pub fn builtin_series_value_type(name: &str) -> Option<PineType> {
    BUILTIN_SERIES_VALUES
        .iter()
        .find(|(value_name, _)| *value_name == name)
        .map(|(_, pine_type)| *pine_type)
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
    "plot.style_line",
    "plot.style_stepline",
    "plot.style_stepline_diamond",
    "plot.style_histogram",
    "plot.style_cross",
    "plot.style_area",
    "plot.style_columns",
    "plot.style_circles",
    "plot.style_linebr",
    "plot.style_areabr",
    "hline.style_solid",
    "hline.style_dotted",
    "hline.style_dashed",
    "size.auto",
    "size.tiny",
    "size.small",
    "size.normal",
    "size.large",
    "size.huge",
    "display.all",
    "display.none",
    "display.pane",
    "display.price_scale",
    "display.status_line",
    "display.data_window",
    "format.mintick",
    "format.price",
    "format.percent",
    "format.volume",
    "order.ascending",
    "order.descending",
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
pub fn change_return_for_arg(arg_type: PineType) -> Option<PineType> {
    match arg_type.kind {
        ValueKind::Bool => Some(PineType::new(Qualifier::Series, ValueKind::Bool)),
        ValueKind::Int | ValueKind::Float => {
            Some(PineType::new(Qualifier::Series, ValueKind::Float))
        }
        _ => None,
    }
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
