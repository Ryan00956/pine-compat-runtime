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
    BuiltinParam {
        name: "session",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "adjustment",
        accepts: Accepts::SimpleString,
        optional: true,
    },
];

const SYMBOL_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "symbol",
    accepts: Accepts::SimpleString,
    optional: false,
}];

const MODIFY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "tickerid",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "session",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "adjustment",
        accepts: Accepts::SimpleString,
        optional: true,
    },
];

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
        params: MODIFY_PARAMS,
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
