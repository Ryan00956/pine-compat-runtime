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

const LABEL_SET_XLOC_PARAMS: &[BuiltinParam] = &[
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
        name: "xloc",
        accepts: Accepts::ConstString,
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

const LABEL_SET_YLOC_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LabelCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "yloc",
        accepts: Accepts::ConstString,
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

const LABEL_COPY_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::LabelCompatible,
    optional: false,
}];

const LABEL_GET_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::LabelCompatible,
    optional: false,
}];

const LINE_NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "x1",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y1",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "x2",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y2",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const LINE_SET_X_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "x",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const LINE_SET_Y_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const LINE_SET_XY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
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

const LINE_SET_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const LINE_SET_WIDTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "width",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const LINE_SET_STYLE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const LINE_SET_EXTEND_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::LineCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "extend",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const LINE_DELETE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::LineCompatible,
    optional: false,
}];

const LINE_COPY_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::LineCompatible,
    optional: false,
}];

const BOX_NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "left",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "top",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "right",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "bottom",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const BOX_SET_X_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "x",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const BOX_SET_Y_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "y",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const BOX_SET_XY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
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

const BOX_SET_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const BOX_SET_BORDER_WIDTH_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "width",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const BOX_SET_BORDER_STYLE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const BOX_SET_EXTEND_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "extend",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const BOX_SET_TEXT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
];

const BOX_SET_TEXT_COLOR_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text_color",
        accepts: Accepts::ColorCompatible,
        optional: false,
    },
];

const BOX_SET_TEXT_SIZE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text_size",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const BOX_SET_TEXT_HALIGN_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::BoxCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text_halign",
        accepts: Accepts::ConstString,
        optional: false,
    },
];

const BOX_DELETE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::BoxCompatible,
    optional: false,
}];

const BOX_COPY_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::BoxCompatible,
    optional: false,
}];

const BOX_GET_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::BoxCompatible,
    optional: false,
}];

const TABLE_NEW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "position",
        accepts: Accepts::ConstString,
        optional: false,
    },
    BuiltinParam {
        name: "columns",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "rows",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
];

const TABLE_CELL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::TableCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "text",
        accepts: Accepts::StringCompatible,
        optional: false,
    },
    BuiltinParam {
        name: "bgcolor",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "text_color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
];

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
        name: "label.set_xloc",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_XLOC_PARAMS,
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
        name: "label.set_yloc",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_SET_YLOC_PARAMS,
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
    BuiltinSignature {
        name: "label.copy",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_COPY_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_LABEL),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.get_x",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.get_y",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "label.get_text",
        phase: BuiltinPhase::Phase1Core,
        params: LABEL_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.new",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_NEW_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_LINE),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_x1",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_X_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_y1",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_Y_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_xy1",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_XY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_x2",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_X_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_y2",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_Y_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_xy2",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_XY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_color",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_width",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_WIDTH_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_style",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_STYLE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.set_extend",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_SET_EXTEND_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.delete",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_DELETE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "line.copy",
        phase: BuiltinPhase::Phase1Core,
        params: LINE_COPY_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_LINE),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.new",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_NEW_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOX),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_left",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_X_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_top",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_Y_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_right",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_X_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_bottom",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_Y_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_lefttop",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_XY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_rightbottom",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_XY_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_bgcolor",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_border_color",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_border_width",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_BORDER_WIDTH_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_border_style",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_BORDER_STYLE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_extend",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_EXTEND_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_text",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_TEXT_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_text_color",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_TEXT_COLOR_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_text_size",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_TEXT_SIZE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.set_text_halign",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_SET_TEXT_HALIGN_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.delete",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_DELETE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.copy",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_COPY_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_BOX),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.get_top",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.get_bottom",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.get_left",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "box.get_right",
        phase: BuiltinPhase::Phase1Core,
        params: BOX_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "table.new",
        phase: BuiltinPhase::Phase1Core,
        params: TABLE_NEW_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_TABLE),
        variadic: false,
    },
    BuiltinSignature {
        name: "table.cell",
        phase: BuiltinPhase::Phase1Core,
        params: TABLE_CELL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
];
