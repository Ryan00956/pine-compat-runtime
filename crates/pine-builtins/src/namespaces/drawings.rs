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
    BuiltinParam {
        name: "xloc",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "yloc",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "textcolor",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "size",
        accepts: Accepts::ConstString,
        optional: true,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::StringCompatible,
        optional: true,
    },
];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[BuiltinSignature {
    name: "label.new",
    phase: BuiltinPhase::Phase1Core,
    params: LABEL_NEW_PARAMS,
    returns: ReturnSpec::Fixed(SERIES_LABEL),
    variadic: false,
}];
