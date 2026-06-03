use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

use super::types::{SERIES_FLOAT, SERIES_INT, SERIES_STRING, VOID};

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
    BuiltinParam {
        name: "limit",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
    BuiltinParam {
        name: "stop",
        accepts: Accepts::SeriesOrSimpleNumeric,
        optional: true,
    },
];

const STRATEGY_CLOSE_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "id",
    accepts: Accepts::SimpleString,
    optional: false,
}];

const STRATEGY_CLOSE_ALL_PARAMS: &[BuiltinParam] = &[];

const STRATEGY_CANCEL_PARAMS: &[BuiltinParam] = &[BuiltinParam {
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

const STRATEGY_TRADE_FIELD_PARAMS: &[BuiltinParam] = &[BuiltinParam {
    name: "trade_num",
    accepts: Accepts::SeriesOrSimpleNumeric,
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
    BuiltinSignature {
        name: "strategy.close_all",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_CLOSE_ALL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.cancel",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_CANCEL_PARAMS,
        returns: ReturnSpec::Fixed(VOID),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.cancel_all",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_CLOSE_ALL_PARAMS,
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
    BuiltinSignature {
        name: "strategy.closedtrades.entry_price",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.entry_id",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.exit_price",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.exit_id",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.entry_bar_index",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.exit_bar_index",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.entry_time",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.exit_time",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.commission",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.size",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.profit",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.closedtrades.max_runup",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.entry_price",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.entry_id",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_STRING),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.entry_bar_index",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.entry_time",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_INT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.size",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.profit",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.commission",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.max_runup",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "strategy.opentrades.max_drawdown",
        phase: BuiltinPhase::Phase1Core,
        params: STRATEGY_TRADE_FIELD_PARAMS,
        returns: ReturnSpec::Fixed(SERIES_FLOAT),
        variadic: false,
    },
];
