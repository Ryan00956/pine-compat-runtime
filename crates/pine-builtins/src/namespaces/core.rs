use pine_ir::{PineType, Qualifier, ValueKind};

use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

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

pub(crate) const SCRIPT_SIGNATURES: &[BuiltinSignature] = &[
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
];

pub(crate) const CAST_SIGNATURES: &[BuiltinSignature] = &[
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
];

pub(crate) const VALUE_SIGNATURES: &[BuiltinSignature] = &[
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
];
