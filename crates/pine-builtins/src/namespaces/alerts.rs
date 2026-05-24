use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

const ALERTCONDITION_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "condition",
        accepts: Accepts::BoolCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "title",
        accepts: Accepts::ConstString,
        optional: false,
    },
    BuiltinParam {
        name: "message",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

pub const SIGNATURES: &[BuiltinSignature] = &[BuiltinSignature {
    name: "alertcondition",
    phase: BuiltinPhase::Phase1Core,
    params: ALERTCONDITION_PARAMS,
    returns: ReturnSpec::Fixed(VOID),
    variadic: false,
}];
