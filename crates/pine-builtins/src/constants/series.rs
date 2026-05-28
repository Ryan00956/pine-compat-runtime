use pine_ir::{PineType, Qualifier, ValueKind};

use crate::namespaces::types::{SERIES_BOOL, SIMPLE_BOOL, SIMPLE_INT, SIMPLE_STRING};

const BUILTIN_SERIES_VALUES: &[(&str, PineType)] = &[
    (
        "strategy.position_size",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "strategy.position_avg_price",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "strategy.openprofit",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "strategy.netprofit",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "strategy.equity",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "barstate.isfirst",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.islast",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.isnew",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.isconfirmed",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.ishistory",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    (
        "barstate.isrealtime",
        PineType::new(Qualifier::Series, ValueKind::Bool),
    ),
    ("session.ismarket", SERIES_BOOL),
    ("session.ispremarket", SERIES_BOOL),
    ("session.ispostmarket", SERIES_BOOL),
    ("timeframe.period", SIMPLE_STRING),
    ("timeframe.isseconds", SIMPLE_BOOL),
    ("timeframe.isminutes", SIMPLE_BOOL),
    ("timeframe.isintraday", SIMPLE_BOOL),
    ("timeframe.isdaily", SIMPLE_BOOL),
    ("timeframe.isweekly", SIMPLE_BOOL),
    ("timeframe.ismonthly", SIMPLE_BOOL),
    ("timeframe.isdwm", SIMPLE_BOOL),
    ("timeframe.multiplier", SIMPLE_INT),
    (
        "ta.accdist",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    ("ta.iii", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.nvi", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.obv", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.pvi", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.pvt", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.tr", PineType::new(Qualifier::Series, ValueKind::Float)),
    ("ta.wad", PineType::new(Qualifier::Series, ValueKind::Float)),
    (
        "ta.vwap",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
    (
        "ta.wvad",
        PineType::new(Qualifier::Series, ValueKind::Float),
    ),
];

#[must_use]
pub fn builtin_series_value_type(name: &str) -> Option<PineType> {
    BUILTIN_SERIES_VALUES
        .iter()
        .find(|(value_name, _)| *value_name == name)
        .map(|(_, pine_type)| *pine_type)
}
