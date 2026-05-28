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
];
