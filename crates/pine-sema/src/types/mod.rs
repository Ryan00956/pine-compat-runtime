use pine_builtins::Accepts;
use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};

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
            qualifier_at_most(arg_type.qualifier, Qualifier::Series) && is_numeric(arg_type.kind)
        }
        Accepts::SeriesOrSimpleNumericOrBool => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Series)
                && (is_numeric(arg_type.kind) || arg_type.kind == ValueKind::Bool)
        }
        Accepts::SimpleInt => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && arg_type.kind == ValueKind::Int
        }
        Accepts::SimpleIntCompatible => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && matches!(arg_type.kind, ValueKind::Int | ValueKind::Na)
        }
        Accepts::SimpleString => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && matches!(arg_type.kind, ValueKind::String | ValueKind::Na)
        }
        Accepts::SimpleNumeric => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple) && is_numeric(arg_type.kind)
        }
        Accepts::SimpleBool => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Simple)
                && arg_type.kind == ValueKind::Bool
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
        Accepts::ConstOrInputFloat => {
            qualifier_at_most(arg_type.qualifier, Qualifier::Input) && is_numeric(arg_type.kind)
        }
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
        Accepts::BoxCompatible => {
            matches!(arg_type.kind, ValueKind::Box | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::TableCompatible => {
            matches!(arg_type.kind, ValueKind::Table | ValueKind::Na)
                && qualifier_at_most(arg_type.qualifier, Qualifier::Series)
        }
        Accepts::PlotOrHLine => matches!(arg_type.kind, ValueKind::Plot | ValueKind::HLine),
        Accepts::Array => is_array_kind(arg_type.kind),
        Accepts::Tuple => arg_type.kind == ValueKind::Tuple,
        Accepts::ScalarArray => is_scalar_array_kind(arg_type.kind),
        Accepts::NumericArray => is_numeric_array_kind(arg_type.kind),
        Accepts::NumericOrBoolArray => {
            is_numeric_array_kind(arg_type.kind) || arg_type.kind == ValueKind::BoolArray
        }
        Accepts::NumericOrStringArray => {
            is_numeric_array_kind(arg_type.kind) || arg_type.kind == ValueKind::StringArray
        }
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
pub(crate) fn can_assign(target: PineType, value: PineType) -> bool {
    if target.kind == value.kind {
        return qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series;
    }

    target.kind == ValueKind::Float
        && value.kind == ValueKind::Int
        && (qualifier_at_most(value.qualifier, target.qualifier)
            || target.qualifier == Qualifier::Series)
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
    let kind = match array_type.kind {
        ValueKind::FloatArray => ValueKind::Float,
        ValueKind::IntArray => ValueKind::Int,
        ValueKind::BoolArray => ValueKind::Bool,
        ValueKind::StringArray => ValueKind::String,
        ValueKind::ColorArray => ValueKind::Color,
        ValueKind::LabelArray => ValueKind::Label,
        ValueKind::LineArray => ValueKind::Line,
        ValueKind::BoxArray => ValueKind::Box,
        ValueKind::TableArray => ValueKind::Table,
        _ => return None,
    };
    Some(PineType::new(Qualifier::Series, kind))
}
pub(crate) fn array_numeric_return_type(
    arg_types: &[Option<PineType>],
    index: usize,
) -> Option<PineType> {
    let array_type = arg_types.get(index).copied().flatten()?;
    let kind = match array_type.kind {
        ValueKind::FloatArray => ValueKind::Float,
        ValueKind::IntArray => ValueKind::Int,
        _ => return None,
    };
    Some(PineType::new(Qualifier::Series, kind))
}
pub(crate) fn array_from_return_type(arg_types: &[Option<PineType>]) -> Option<PineType> {
    let mut inferred_kind: Option<ValueKind> = None;
    for arg_type in arg_types {
        let arg_type = (*arg_type)?;
        let next_kind = match arg_type.kind {
            ValueKind::Na => continue,
            ValueKind::Int => ValueKind::IntArray,
            ValueKind::Float => ValueKind::FloatArray,
            ValueKind::Bool => ValueKind::BoolArray,
            ValueKind::String => ValueKind::StringArray,
            ValueKind::Color => ValueKind::ColorArray,
            ValueKind::Label => ValueKind::LabelArray,
            ValueKind::Line => ValueKind::LineArray,
            ValueKind::Box => ValueKind::BoxArray,
            ValueKind::Table => ValueKind::TableArray,
            _ => return None,
        };
        inferred_kind = Some(match (inferred_kind, next_kind) {
            (None, kind) => kind,
            (Some(ValueKind::IntArray), ValueKind::FloatArray)
            | (Some(ValueKind::FloatArray), ValueKind::IntArray)
            | (Some(ValueKind::FloatArray), ValueKind::FloatArray)
            | (Some(ValueKind::IntArray), ValueKind::IntArray) => {
                if matches!(next_kind, ValueKind::FloatArray)
                    || matches!(inferred_kind, Some(ValueKind::FloatArray))
                {
                    ValueKind::FloatArray
                } else {
                    ValueKind::IntArray
                }
            }
            (Some(current), kind) if current == kind => current,
            _ => return None,
        });
    }
    inferred_kind.map(|kind| PineType::new(Qualifier::Simple, kind))
}
pub(crate) fn is_array_kind(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::FloatArray
            | ValueKind::IntArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
            | ValueKind::LabelArray
            | ValueKind::LineArray
            | ValueKind::BoxArray
            | ValueKind::TableArray
    )
}
pub(crate) fn is_numeric_array_kind(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::FloatArray | ValueKind::IntArray)
}

pub(crate) fn is_scalar_array_kind(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::FloatArray
            | ValueKind::IntArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
    )
}
