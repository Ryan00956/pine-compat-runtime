use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

const LABEL_NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "x",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[BuiltinSignature {
    name: "label.new",
    phase: BuiltinPhase::Phase1Core,
    params: LABEL_NEW_PARAMS,
    returns: ReturnSpec::Fixed(SERIES_LABEL),
    variadic: false,
}];
