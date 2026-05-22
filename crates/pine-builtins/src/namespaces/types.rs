use pine_ir::{PineType, Qualifier, ValueKind};

pub(crate) const INPUT_INT: PineType = PineType::new(Qualifier::Input, ValueKind::Int);
pub(crate) const INPUT_FLOAT: PineType = PineType::new(Qualifier::Input, ValueKind::Float);
pub(crate) const INPUT_BOOL: PineType = PineType::new(Qualifier::Input, ValueKind::Bool);
pub(crate) const INPUT_COLOR: PineType = PineType::new(Qualifier::Input, ValueKind::Color);
pub(crate) const INPUT_STRING: PineType = PineType::new(Qualifier::Input, ValueKind::String);
pub(crate) const SERIES_FLOAT: PineType = PineType::new(Qualifier::Series, ValueKind::Float);
pub(crate) const SERIES_INT: PineType = PineType::new(Qualifier::Series, ValueKind::Int);
pub(crate) const SERIES_BOOL: PineType = PineType::new(Qualifier::Series, ValueKind::Bool);
pub(crate) const SERIES_STRING: PineType = PineType::new(Qualifier::Series, ValueKind::String);
pub(crate) const SERIES_FLOAT_TUPLE: PineType = PineType::new(Qualifier::Series, ValueKind::Tuple);
pub(crate) const PLOT: PineType = PineType::new(Qualifier::Const, ValueKind::Plot);
pub(crate) const HLINE: PineType = PineType::new(Qualifier::Const, ValueKind::HLine);
pub(crate) const VOID: PineType = PineType::new(Qualifier::Const, ValueKind::Void);
pub(crate) const SIMPLE_INT: PineType = PineType::new(Qualifier::Simple, ValueKind::Int);
pub(crate) const SIMPLE_BOOL: PineType = PineType::new(Qualifier::Simple, ValueKind::Bool);
pub(crate) const SIMPLE_STRING: PineType = PineType::new(Qualifier::Simple, ValueKind::String);
pub(crate) const SIMPLE_FLOAT_ARRAY: PineType =
    PineType::new(Qualifier::Simple, ValueKind::FloatArray);
pub(crate) const SIMPLE_INT_ARRAY: PineType = PineType::new(Qualifier::Simple, ValueKind::IntArray);
pub(crate) const SIMPLE_BOOL_ARRAY: PineType =
    PineType::new(Qualifier::Simple, ValueKind::BoolArray);
pub(crate) const SIMPLE_STRING_ARRAY: PineType =
    PineType::new(Qualifier::Simple, ValueKind::StringArray);
pub(crate) const SIMPLE_COLOR_ARRAY: PineType =
    PineType::new(Qualifier::Simple, ValueKind::ColorArray);
