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

const LABEL_SET_X_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "x",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const LABEL_SET_Y_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const LABEL_SET_XY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
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
];

const LABEL_SET_TEXT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const LABEL_SET_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const LABEL_SET_TEXTCOLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "textcolor",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const LABEL_SET_STYLE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const LABEL_SET_SIZE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "size",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const LABEL_SET_TOOLTIP_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "tooltip",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const LABEL_DELETE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::LabelCompatible,
    optional: false,
}];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "label.new",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_NEW_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_LABEL),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_x",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_X_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_y",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_Y_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_xy",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_XY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_text",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_TEXT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_color",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_textcolor",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_TEXTCOLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_style",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_STYLE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_size",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.set_tooltip",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_TOOLTIP_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.delete",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_DELETE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
];
