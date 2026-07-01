use pine_builtins::Accepts;
use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};

mod matrices;

pub(crate) use matrices::{
    accepts_matrix_element_arg, accepts_matrix_element_array_arg, is_matrix_kind,
    is_numeric_matrix_kind, matrix_array_return_type, matrix_element_return_type,
    matrix_method_builtin_name, matrix_mult_return_type,
};

pub(crate) const UNKNOWN: PineType = PineType::new(Qualifier::Series, ValueKind::Na);
pub(crate) fn const_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::Literal(Literal::Int(value)) => Some(*value),
        ExprKind::QualifiedName(parts) => pine_builtins::named_int_constant(&parts.join(".")),
        ExprKind::Unary {
            op: UnaryOp::Plus,
            expr,
        } => const_int_value(expr),
        ExprKind::Unary {
            op: UnaryOp::Minus,
            expr,
        } => const_int_value(expr).and_then(i64::checked_neg),
        ExprKind::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => const_int_value(left)?.checked_add(const_int_value(right)?),
        ExprKind::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => const_int_value(left)?.checked_sub(const_int_value(right)?),
        _ => None,
    }
}
pub(crate) fn const_numeric_value(expr: &Expr) -> Option<f64> {
    match &expr.kind {
        ExprKind::Literal(Literal::Int(value)) => Some(*value as f64),
        ExprKind::Literal(Literal::Float(value)) => Some(*value),
        ExprKind::Unary {
            op: UnaryOp::Plus,
            expr,
        } => const_numeric_value(expr),
        ExprKind::Unary {
            op: UnaryOp::Minus,
            expr,
        } => const_numeric_value(expr).map(|value| -value),
        _ => None,
    }
}
pub(crate) fn const_string_value(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Literal(Literal::String(value)) => Some(value.clone()),
        ExprKind::QualifiedName(parts) => Some(parts.join(".")),
        _ => None,
    }
}
pub(crate) fn literal_type(literal: &Literal) -> PineType {
    match literal {
        Literal::Int(_) => PineType::new(Qualifier::Const, ValueKind::Int),
        Literal::Float(_) => PineType::new(Qualifier::Const, ValueKind::Float),
        Literal::Bool(_) => PineType::new(Qualifier::Const, ValueKind::Bool),
        Literal::String(_) => PineType::new(Qualifier::Const, ValueKind::String),
        Literal::ColorHex(_) => PineType::new(Qualifier::Const, ValueKind::Color),
    }
}
pub(crate) fn accepts_type(accepts: Accepts, arg_type: PineType) -> bool {
    match accepts {
        Accepts::Any => true,
        Accepts::Exact(expected) => can_assign(expected, arg_type),
        Accepts::Kind(kind) => arg_type.kind == kind,
        Accepts::Numeric => is_numeric(arg_type.kind),
        Accepts::SeriesFloat => {
            arg_type.qualifier == Qualifier::Series && arg_type.kind == ValueKind::Float
        }
        Accepts::SeriesNumeric => {
            arg_type.qualifier == Qualifier::Series && is_numeric(arg_type.kind)
        }
        Accepts::SeriesNumericOrBool => {
            arg_type.qualifier == Qualifier::Series
                && (is_numeric(arg_type.kind) || arg_type.kind == ValueKind::Bool)
        }
        Accepts::SeriesOrSimpleNumeric => {
            accepts_kind_at_most(arg_type, Qualifier::Series, is_numeric)
        }
        Accepts::SeriesOrSimpleNumericOrBool => {
            accepts_kind_at_most(arg_type, Qualifier::Series, |kind| {
                is_numeric(kind) || kind == ValueKind::Bool
            })
        }
        Accepts::SimpleInt => {
            accepts_kind_at_most(arg_type, Qualifier::Simple, |kind| kind == ValueKind::Int)
        }
        Accepts::SimpleIntCompatible => accepts_kind_at_most(arg_type, Qualifier::Simple, |kind| {
            matches!(kind, ValueKind::Int | ValueKind::Na)
        }),
        Accepts::SimpleString => accepts_kind_at_most(arg_type, Qualifier::Simple, |kind| {
            matches!(kind, ValueKind::String | ValueKind::Na)
        }),
        Accepts::SimpleNumeric => accepts_kind_at_most(arg_type, Qualifier::Simple, is_numeric),
        Accepts::SimpleBool => {
            accepts_kind_at_most(arg_type, Qualifier::Simple, |kind| kind == ValueKind::Bool)
        }
        Accepts::ConstNumeric => {
            arg_type.qualifier == Qualifier::Const && is_numeric(arg_type.kind)
        }
        Accepts::ConstString => {
            arg_type.qualifier == Qualifier::Const && arg_type.kind == ValueKind::String
        }
        Accepts::ConstBool => {
            arg_type.qualifier == Qualifier::Const && arg_type.kind == ValueKind::Bool
        }
        Accepts::AtMostInputNumeric => accepts_kind_at_most(arg_type, Qualifier::Input, is_numeric),
        Accepts::ColorCompatible => {
            matches!(arg_type.kind, ValueKind::Color | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::StringCompatible => {
            matches!(arg_type.kind, ValueKind::String | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::StringConvertible => {
            matches!(
                arg_type.kind,
                ValueKind::Int
                    | ValueKind::Float
                    | ValueKind::Bool
                    | ValueKind::String
                    | ValueKind::FloatArray
                    | ValueKind::IntArray
                    | ValueKind::BoolArray
                    | ValueKind::StringArray
                    | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::StringOrIntCompatible => {
            matches!(
                arg_type.kind,
                ValueKind::String | ValueKind::Int | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::CastScalar => {
            matches!(
                arg_type.kind,
                ValueKind::Int | ValueKind::Float | ValueKind::Bool | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::StringCastScalar => {
            matches!(
                arg_type.kind,
                ValueKind::Int
                    | ValueKind::Float
                    | ValueKind::Bool
                    | ValueKind::String
                    | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::ValueWhenSource => {
            matches!(
                arg_type.kind,
                ValueKind::Int
                    | ValueKind::Float
                    | ValueKind::Bool
                    | ValueKind::Color
                    | ValueKind::Na
            ) && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::NumericOrColorCompatible => {
            (is_numeric(arg_type.kind) || matches!(arg_type.kind, ValueKind::Color | ValueKind::Na))
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::NumericCompatible => {
            (is_numeric(arg_type.kind) || arg_type.kind == ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::IntCompatible => {
            matches!(arg_type.kind, ValueKind::Int | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::BoolCompatible => {
            matches!(arg_type.kind, ValueKind::Bool | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::LabelCompatible => {
            matches!(arg_type.kind, ValueKind::Label | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::LineCompatible => {
            matches!(arg_type.kind, ValueKind::Line | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::LineFillCompatible => {
            matches!(arg_type.kind, ValueKind::LineFill | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::PolylineCompatible => {
            matches!(arg_type.kind, ValueKind::Polyline | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::BoxCompatible => {
            matches!(arg_type.kind, ValueKind::Box | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::TableCompatible => {
            matches!(arg_type.kind, ValueKind::Table | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::ChartPointCompatible => {
            matches!(arg_type.kind, ValueKind::ChartPoint | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::PlotOrHLine => matches!(arg_type.kind, ValueKind::Plot | ValueKind::HLine),
        Accepts::Array => is_array_kind(arg_type.kind),
        Accepts::FloatMatrix => arg_type.kind == ValueKind::FloatMatrix,
        Accepts::NumericMatrix => matrices::is_numeric_matrix_kind(arg_type.kind),
        Accepts::Matrix => is_matrix_kind(arg_type.kind),
        Accepts::Map => arg_type.kind == ValueKind::Map,
        Accepts::MatrixOrNumericCompatibleWithMatrixCounterpart(_) => {
            is_matrix_kind(arg_type.kind) || accepts_type(Accepts::NumericCompatible, arg_type)
        }
        Accepts::MatrixOrNumericOrNumericArrayCompatibleWithMatrixCounterpart(_) => {
            is_matrix_kind(arg_type.kind)
                || accepts_type(Accepts::NumericCompatible, arg_type)
                || accepts_type(Accepts::NumericArray, arg_type)
        }
        Accepts::MatrixElementCompatible(_) => false,
        Accepts::MatrixElementArray(_) => false,
        Accepts::Tuple => arg_type.kind == ValueKind::Tuple,
        Accepts::ScalarArray => is_scalar_array_kind(arg_type.kind),
        Accepts::NumericArray => is_numeric_array_kind(arg_type.kind),
        Accepts::NumericOrBoolArray => matches!(
            arg_type.kind.array_element_kind(),
            Some(ValueKind::Float | ValueKind::Int | ValueKind::Bool)
        ),
        Accepts::NumericOrStringArray => matches!(
            arg_type.kind.array_element_kind(),
            Some(ValueKind::Float | ValueKind::Int | ValueKind::String)
        ),
        Accepts::InputDefval => {
            arg_type.qualifier == Qualifier::Const
                && matches!(
                    arg_type.kind,
                    ValueKind::Int
                        | ValueKind::Float
                        | ValueKind::Bool
                        | ValueKind::String
                        | ValueKind::Color
                )
        }
    }
}
fn accepts_kind_at_most(
    arg_type: PineType,
    max_qualifier: Qualifier,
    accepts_kind: impl Fn(ValueKind) -> bool,
) -> bool {
    qualifier_at_most(arg_type.qualifier, max_qualifier) && accepts_kind(arg_type.kind)
}
pub(crate) fn can_assign(target: PineType, value: PineType) -> bool {
    if target.kind == value.kind {
        return qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series;
    }
    if value.kind == ValueKind::Na && can_assign_na_to_kind(target.kind) {
        return qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series;
    }

    target.kind == ValueKind::Float
        && value.kind == ValueKind::Int
        && (qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series)
}

fn can_assign_na_to_kind(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::Int
            | ValueKind::Float
            | ValueKind::Bool
            | ValueKind::String
            | ValueKind::Color
            | ValueKind::Label
            | ValueKind::Line
            | ValueKind::LineFill
            | ValueKind::Polyline
            | ValueKind::Box
            | ValueKind::Table
            | ValueKind::ChartPoint
            | ValueKind::UserType
            | ValueKind::IntArray
            | ValueKind::FloatArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
            | ValueKind::LabelArray
            | ValueKind::LineArray
            | ValueKind::LineFillArray
            | ValueKind::BoxArray
            | ValueKind::TableArray
            | ValueKind::ChartPointArray
            | ValueKind::UserTypeArray
            | ValueKind::FloatMatrix
            | ValueKind::IntMatrix
            | ValueKind::BoolMatrix
            | ValueKind::StringMatrix
            | ValueKind::ColorMatrix
            | ValueKind::Map
    )
}
pub(crate) fn qualifier_at_most(actual: Qualifier, max: Qualifier) -> bool {
    qualifier_rank(actual) <= qualifier_rank(max)
}
pub(crate) fn strongest_qualifier(left: Qualifier, right: Qualifier) -> Qualifier {
    if qualifier_rank(left) >= qualifier_rank(right) {
        left
    } else {
        right
    }
}
pub(crate) fn qualifier_rank(qualifier: Qualifier) -> u8 {
    match qualifier {
        Qualifier::Const => 0,
        Qualifier::Input => 1,
        Qualifier::Simple => 2,
        Qualifier::Series => 3,
    }
}
pub(crate) fn is_numeric(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::Int | ValueKind::Float)
}
pub(crate) fn numeric_result_kind(op: BinaryOp, left: ValueKind, right: ValueKind) -> ValueKind {
    if op == BinaryOp::Div || left == ValueKind::Float || right == ValueKind::Float {
        ValueKind::Float
    } else {
        ValueKind::Int
    }
}
pub(crate) fn common_kind(left: ValueKind, right: ValueKind) -> Option<ValueKind> {
    if left == right {
        Some(left)
    } else if is_numeric(left) && is_numeric(right) {
        Some(ValueKind::Float)
    } else if left == ValueKind::Na {
        Some(right)
    } else if right == ValueKind::Na {
        Some(left)
    } else {
        None
    }
}
pub(crate) fn merge_result_types(current: Option<PineType>, next: PineType) -> Option<PineType> {
    match current {
        Some(current) => Some(PineType::new(
            strongest_qualifier(current.qualifier, next.qualifier),
            common_kind(current.kind, next.kind)?,
        )),
        None => Some(next),
    }
}
pub(crate) fn promoted_numeric_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut result: Option<PineType> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        if !is_numeric(arg_type.kind) && arg_type.kind != ValueKind::Na {
            return None;
        }
        result = Some(match result {
            Some(current) => PineType::new(
                strongest_qualifier(current.qualifier, arg_type.qualifier),
                common_kind(current.kind, arg_type.kind)?,
            ),
            None => arg_type,
        });
    }
    result
}
pub(crate) fn float_return_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Float)
}
pub(crate) fn int_return_for_arg(arg_type: PineType) -> PineType {
    PineType::new(arg_type.qualifier, ValueKind::Int)
}
pub(crate) fn series_return_for_arg(arg_type: PineType) -> Option<PineType> {
    match arg_type.kind {
        ValueKind::Int | ValueKind::Float | ValueKind::Bool | ValueKind::Color | ValueKind::Na => {
            Some(PineType::new(Qualifier::Series, arg_type.kind))
        }
        _ => None,
    }
}
pub(crate) fn promoted_float_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        if !is_numeric(arg_type.kind) && arg_type.kind != ValueKind::Na {
            return None;
        }
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Float))
}
pub(crate) fn promoted_color_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Color))
}
pub(crate) fn promoted_bool_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, ValueKind::Bool))
}
pub(crate) fn promoted_int_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    promoted_kind_type(arg_types, ValueKind::Int)
}
pub(crate) fn promoted_string_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    promoted_kind_type(arg_types, ValueKind::String)
}
pub(crate) fn promoted_kind_type(
    arg_types: &[Option<PineType>],
    kind: ValueKind,
) -> Option<PineType> {
    let mut qualifier: Option<Qualifier> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        qualifier = Some(match qualifier {
            Some(current) => strongest_qualifier(current, arg_type.qualifier),
            None => arg_type.qualifier,
        });
    }
    qualifier.map(|qualifier| PineType::new(qualifier, kind))
}
pub(crate) fn round_return_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let number_type = arg_types.first().copied().flatten()?;
    if arg_types.len() > 1 {
        promoted_kind_type(arg_types, ValueKind::Float)
    } else {
        Some(PineType::new(number_type.qualifier, ValueKind::Int))
    }
}
pub(crate) fn array_element_return_type(
    arg_types: &[Option<PineType>],
    index: usize,
) -> Option<PineType> {
    let array_type = arg_types.get(index).copied().flatten()?;
    let kind = array_type.kind.array_element_kind()?;
    Some(PineType::new(Qualifier::Series, kind))
}
pub(crate) fn array_numeric_return_type(
    arg_types: &[Option<PineType>],
    index: usize,
) -> Option<PineType> {
    let array_type = arg_types.get(index).copied().flatten()?;
    let kind = match array_type.kind.array_element_kind()? {
        ValueKind::Float => ValueKind::Float,
        ValueKind::Int => ValueKind::Int,
        _ => return None,
    };
    Some(PineType::new(Qualifier::Series, kind))
}
pub(crate) fn array_from_return_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut inferred_element_kind: Option<ValueKind> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        let next_element_kind = match arg_type.kind {
            ValueKind::Na => continue,
            kind if kind.array_kind_from_element_kind().is_some() => kind,
            _ => return None,
        };
        inferred_element_kind = Some(match (inferred_element_kind, next_element_kind) {
            (None, kind) => kind,
            (Some(ValueKind::Int), ValueKind::Float)
            | (Some(ValueKind::Float), ValueKind::Int)
            | (Some(ValueKind::Float), ValueKind::Float)
            | (Some(ValueKind::Int), ValueKind::Int) => {
                if matches!(next_element_kind, ValueKind::Float)
                    || matches!(inferred_element_kind, Some(ValueKind::Float))
                {
                    ValueKind::Float
                } else {
                    ValueKind::Int
                }
            }
            (Some(current), kind) if current == kind => current,
            _ => return None,
        });
    }
    let array_kind = inferred_element_kind?.array_kind_from_element_kind()?;
    Some(PineType::new(Qualifier::Simple, array_kind))
}

pub(crate) fn array_kind_from_element_type_name(element_type: &str) -> Option<ValueKind> {
    let element_kind = match element_type {
        "int" => ValueKind::Int,
        "float" => ValueKind::Float,
        "bool" => ValueKind::Bool,
        "string" => ValueKind::String,
        "color" => ValueKind::Color,
        "label" => ValueKind::Label,
        "line" => ValueKind::Line,
        "linefill" => ValueKind::LineFill,
        "polyline" => ValueKind::Polyline,
        "box" => ValueKind::Box,
        "table" => ValueKind::Table,
        "chart.point" => ValueKind::ChartPoint,
        _ => return None,
    };
    element_kind.array_kind_from_element_kind()
}

pub(crate) fn is_array_kind(kind: ValueKind) -> bool {
    kind.array_element_kind().is_some()
}
pub(crate) fn is_collection_kind(kind: ValueKind) -> bool {
    is_array_kind(kind) || is_matrix_kind(kind) || kind == ValueKind::Map
}
pub(crate) fn is_numeric_array_kind(kind: ValueKind) -> bool {
    matches!(
        kind.array_element_kind(),
        Some(ValueKind::Float | ValueKind::Int)
    )
}

pub(crate) fn is_scalar_array_kind(kind: ValueKind) -> bool {
    matches!(
        kind.array_element_kind(),
        Some(
            ValueKind::Float
                | ValueKind::Int
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pine_type(qualifier: Qualifier, kind: ValueKind) -> PineType {
        PineType::new(qualifier, kind)
    }

    #[test]
    fn at_most_input_numeric_accepts_only_const_or_input_numbers() {
        assert!(accepts_type(
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Const, ValueKind::Int)
        ));
        assert!(accepts_type(
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Input, ValueKind::Float)
        ));
        assert!(!accepts_type(
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Simple, ValueKind::Float)
        ));
        assert!(!accepts_type(
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Series, ValueKind::Float)
        ));
        assert!(!accepts_type(
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Input, ValueKind::String)
        ));
    }

    #[test]
    fn simple_acceptors_accept_weaker_qualifiers_and_reject_series() {
        for qualifier in [Qualifier::Const, Qualifier::Input, Qualifier::Simple] {
            assert!(accepts_type(
                Accepts::SimpleInt,
                pine_type(qualifier, ValueKind::Int)
            ));
            assert!(accepts_type(
                Accepts::SimpleNumeric,
                pine_type(qualifier, ValueKind::Float)
            ));
            assert!(accepts_type(
                Accepts::SimpleBool,
                pine_type(qualifier, ValueKind::Bool)
            ));
            assert!(accepts_type(
                Accepts::SimpleString,
                pine_type(qualifier, ValueKind::String)
            ));
        }

        assert!(!accepts_type(
            Accepts::SimpleInt,
            pine_type(Qualifier::Series, ValueKind::Int)
        ));
        assert!(!accepts_type(
            Accepts::SimpleNumeric,
            pine_type(Qualifier::Series, ValueKind::Float)
        ));
        assert!(!accepts_type(
            Accepts::SimpleBool,
            pine_type(Qualifier::Series, ValueKind::Bool)
        ));
        assert!(!accepts_type(
            Accepts::SimpleString,
            pine_type(Qualifier::Series, ValueKind::String)
        ));
    }

    #[test]
    fn simple_int_compatible_accepts_weaker_int_or_na_and_rejects_series() {
        for qualifier in [Qualifier::Const, Qualifier::Input, Qualifier::Simple] {
            assert!(accepts_type(
                Accepts::SimpleIntCompatible,
                pine_type(qualifier, ValueKind::Int)
            ));
            assert!(accepts_type(
                Accepts::SimpleIntCompatible,
                pine_type(qualifier, ValueKind::Na)
            ));
        }

        assert!(!accepts_type(
            Accepts::SimpleIntCompatible,
            pine_type(Qualifier::Series, ValueKind::Int)
        ));
        assert!(!accepts_type(
            Accepts::SimpleIntCompatible,
            pine_type(Qualifier::Series, ValueKind::Na)
        ));
        assert!(!accepts_type(
            Accepts::SimpleIntCompatible,
            pine_type(Qualifier::Input, ValueKind::Float)
        ));
    }

    #[test]
    fn series_or_simple_acceptors_accept_all_numeric_qualifiers() {
        for qualifier in [
            Qualifier::Const,
            Qualifier::Input,
            Qualifier::Simple,
            Qualifier::Series,
        ] {
            assert!(accepts_type(
                Accepts::SeriesOrSimpleNumeric,
                pine_type(qualifier, ValueKind::Int)
            ));
            assert!(accepts_type(
                Accepts::SeriesOrSimpleNumeric,
                pine_type(qualifier, ValueKind::Float)
            ));
            assert!(accepts_type(
                Accepts::SeriesOrSimpleNumericOrBool,
                pine_type(qualifier, ValueKind::Bool)
            ));
        }

        assert!(!accepts_type(
            Accepts::SeriesOrSimpleNumeric,
            pine_type(Qualifier::Series, ValueKind::Bool)
        ));
        assert!(!accepts_type(
            Accepts::SeriesOrSimpleNumericOrBool,
            pine_type(Qualifier::Series, ValueKind::String)
        ));
    }

    #[test]
    fn treats_user_type_array_as_array_but_not_numeric_or_scalar_array() {
        assert!(is_array_kind(ValueKind::UserTypeArray));
        assert!(!is_numeric_array_kind(ValueKind::UserTypeArray));
        assert!(!is_scalar_array_kind(ValueKind::UserTypeArray));
        assert_eq!(
            array_element_return_type(
                &[Some(PineType::new(
                    Qualifier::Simple,
                    ValueKind::UserTypeArray,
                ))],
                0,
            ),
            Some(PineType::new(Qualifier::Series, ValueKind::UserType))
        );
    }

    #[test]
    fn does_not_infer_user_type_array_from_user_type_values() {
        assert_eq!(
            array_from_return_type(&[Some(PineType::new(Qualifier::Series, ValueKind::UserType,))]),
            None
        );
    }
}
