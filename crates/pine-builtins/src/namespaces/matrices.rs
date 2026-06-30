use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::*;

const MATRIX_NEW_FLOAT_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "rows",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "columns",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "initial_value",
        accepts: Accepts::NumericCompatible,
        optional: true,
    },
];

const MATRIX_ID_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::FloatMatrix,
    optional: false,
}];

const MATRIX_GET_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATRIX_SET_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const MATRIX_FILL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "value",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
];

const MATRIX_RESHAPE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "rows",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "columns",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATRIX_ADD_ROW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "array_id",
        accepts: Accepts::Exact(SIMPLE_FLOAT_ARRAY),
        optional: false,
    },
];

const MATRIX_ADD_COL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
    BuiltinParam {
        name: "array_id",
        accepts: Accepts::Exact(SIMPLE_FLOAT_ARRAY),
        optional: false,
    },
];

const MATRIX_REMOVE_ROW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATRIX_REMOVE_COL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATRIX_ROW_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "row",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

const MATRIX_COL_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "id",
        accepts: Accepts::FloatMatrix,
        optional: false,
    },
    BuiltinParam {
        name: "column",
        accepts: Accepts::SimpleInt,
        optional: false,
    },
];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "matrix.new<float>",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_NEW_FLOAT_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_MATRIX),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.get",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_GET_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.set",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_SET_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.fill",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_FILL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.copy",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ID_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_MATRIX),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.reshape",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_RESHAPE_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.add_row",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ADD_ROW_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.add_col",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ADD_COL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.remove_row",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_REMOVE_ROW_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.remove_col",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_REMOVE_COL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.rows",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ID_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.columns",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ID_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.sum",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ID_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.avg",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ID_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.row",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_ROW_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_ARRAY),
        variadic: false,
    },
    BuiltinSignature {
        name: "matrix.col",
        phase: BuiltinPhase::Phase1Core,
        params: MATRIX_COL_PARAMS,
        returns: ReturnSpec::Fixed(SIMPLE_FLOAT_ARRAY),
        variadic: false,
    },
];
