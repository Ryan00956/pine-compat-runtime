use pine_ir::PineType;

use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

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

const TWO_SERIES_FLOATS: &[PineType] = &[SERIES_FLOAT, SERIES_FLOAT];

const THREE_SERIES_FLOATS: &[PineType] = &[SERIES_FLOAT, SERIES_FLOAT, SERIES_FLOAT];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
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
