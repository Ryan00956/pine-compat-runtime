use pine_ir::{HirCallArg, HirExpr};

use crate::*;

pub(crate) fn output_id(value: PineValue) -> Option<u32> {
    match value {
        PineValue::Plot(id) | PineValue::HLine(id) => Some(id),
        _ => None,
    }
}

pub(crate) fn call_arg_expr<'a>(
    args: &'a [HirCallArg],
    index: usize,
    name: &str,
) -> Option<&'a HirExpr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .or_else(|| args.get(index))
        .map(|arg| &arg.value)
}
