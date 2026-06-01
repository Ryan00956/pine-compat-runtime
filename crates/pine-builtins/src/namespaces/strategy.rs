use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::VOID;

const STRATEGY_ENTRY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "direction",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "qty",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
];

const STRATEGY_CLOSE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::SimpleString,
    optional: false,
}];

const STRATEGY_EXIT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "from_entry",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "stop",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "limit",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "profit",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "loss",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "trail_price",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "trail_points",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "trail_offset",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "qty",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "qty_percent",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "strategy.entry",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_ENTRY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.close",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_CLOSE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.exit",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_EXIT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
];
