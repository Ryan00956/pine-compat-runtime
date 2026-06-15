use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

const NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "prefix",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "ticker",
        accepts: Accepts::SimpleString,
        optional: false,
    },
];

const SYMBOL_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "symbol",
    accepts: Accepts::SimpleString,
    optional: false,
}];

const TICKERID_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "tickerid",
    accepts: Accepts::SimpleString,
    optional: false,
}];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "ticker.new",
        phase: BuiltinPhase::Phase1Core,
        params: NEW_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "ticker.modify",
        phase: BuiltinPhase::Phase1Core,
        params: TICKERID_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "ticker.standard",
        phase: BuiltinPhase::Phase1Core,
        params: SYMBOL_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_STRING),
        variadic: false,
    },
];
