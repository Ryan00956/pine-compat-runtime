struct NamedIntConstant {
    name: &'static str,
    value: i64,
}

const NAMED_INT_CONSTANTS: &[NamedIntConstant] = &[
    NamedIntConstant {
        name: "dayofweek.sunday",
        value: 1,
    },
    NamedIntConstant {
        name: "dayofweek.monday",
        value: 2,
    },
    NamedIntConstant {
        name: "dayofweek.tuesday",
        value: 3,
    },
    NamedIntConstant {
        name: "dayofweek.wednesday",
        value: 4,
    },
    NamedIntConstant {
        name: "dayofweek.thursday",
        value: 5,
    },
    NamedIntConstant {
        name: "dayofweek.friday",
        value: 6,
    },
    NamedIntConstant {
        name: "dayofweek.saturday",
        value: 7,
    },
    NamedIntConstant {
        name: "syminfo.minmove",
        value: 1,
    },
    NamedIntConstant {
        name: "syminfo.pricescale",
        value: 100,
    },
];

#[must_use]
pub fn named_int_constant(name: &str) -> Option<i64> {
    NAMED_INT_CONSTANTS
        .iter()
        .find(|constant| constant.name == name)
        .map(|constant| constant.value)
}
