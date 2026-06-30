use crate::prelude::*;

impl Analyzer {
    pub(crate) fn return_type(
        &self,
        signature: &BuiltinSignature,
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
        match signature.returns {
            ReturnSpec::Fixed(pine_type) => Some(pine_type),
            ReturnSpec::Tuple(_) => Some(pine_builtins::tuple_return_type()),
            ReturnSpec::SameAsArg(index) => arg_types.get(index).copied().flatten(),
            ReturnSpec::BoolFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(pine_builtins::fallback_bool_for_arg),
            ReturnSpec::ColorFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(pine_builtins::color_return_for_arg),
            ReturnSpec::PromotedColor => promoted_color_type(arg_types),
            ReturnSpec::PromotedBool => promoted_bool_type(arg_types),
            ReturnSpec::PromotedInt => promoted_int_type(arg_types),
            ReturnSpec::PromotedString => promoted_string_type(arg_types),
            ReturnSpec::FloatFromStringArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::PromotedNumeric => promoted_numeric_type(arg_types),
            ReturnSpec::ArrayElement(index) => array_element_return_type(arg_types, index),
            ReturnSpec::ArrayNumeric(index) => array_numeric_return_type(arg_types, index),
            ReturnSpec::ArrayFromArgs => array_from_return_type(arg_types),
            ReturnSpec::IntFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(int_return_for_arg),
            ReturnSpec::FloatFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::SeriesFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(series_return_for_arg),
            ReturnSpec::ChangeFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(pine_builtins::change_return_for_arg),
            ReturnSpec::PromotedFloat => promoted_float_type(arg_types),
            ReturnSpec::Round => round_return_type(arg_types),
            ReturnSpec::InputFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(pine_builtins::input_return_for_arg),
        }
    }
}
