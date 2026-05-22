use pine_ir::{PineType, ValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinSignature {
    pub name: &'static str,
    pub phase: BuiltinPhase,
    pub params: &'static [BuiltinParam],
    pub returns: ReturnSpec,
    pub variadic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinPhase {
    Phase1Core,
    Later,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinParam {
    pub name: &'static str,
    pub accepts: Accepts,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accepts {
    Any,
    Exact(PineType),
    Kind(ValueKind),
    Numeric,
    SeriesFloat,
    SeriesNumeric,
    SeriesNumericOrBool,
    SeriesOrSimpleNumeric,
    SeriesOrSimpleNumericOrBool,
    SimpleInt,
    SimpleString,
    SimpleNumeric,
    SimpleBool,
    ConstNumeric,
    ConstString,
    ConstBool,
    ConstOrInputFloat,
    ColorCompatible,
    StringCompatible,
    StringConvertible,
    CastScalar,
    StringCastScalar,
    ValueWhenSource,
    NumericOrColorCompatible,
    NumericCompatible,
    IntCompatible,
    BoolCompatible,
    LabelCompatible,
    LineCompatible,
    BoxCompatible,
    PlotOrHLine,
    Array,
    Tuple,
    NumericArray,
    NumericOrBoolArray,
    NumericOrStringArray,
    InputDefval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnSpec {
    Fixed(PineType),
    Tuple(&'static [PineType]),
    SameAsArg(usize),
    BoolFromArg(usize),
    ColorFromArg(usize),
    PromotedColor,
    PromotedBool,
    PromotedInt,
    PromotedString,
    FloatFromStringArg(usize),
    PromotedNumeric,
    ArrayElement(usize),
    ArrayNumeric(usize),
    ArrayFromArgs,
    IntFromArg(usize),
    FloatFromArg(usize),
    SeriesFromArg(usize),
    ChangeFromArg(usize),
    PromotedFloat,
    Round,
    InputFromArg(usize),
}
