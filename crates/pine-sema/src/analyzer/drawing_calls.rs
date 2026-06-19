use crate::prelude::*;

const LINE_XLOCS: &[&str] = &["xloc.bar_index", "xloc.bar_time"];
const LINE_EXTENDS: &[&str] = &["extend.none", "extend.right", "extend.left", "extend.both"];
const LINE_STYLES: &[&str] = &[
    "line.style_solid",
    "line.style_dotted",
    "line.style_dashed",
    "line.style_arrow_left",
    "line.style_arrow_right",
    "line.style_arrow_both",
];

#[derive(Clone, Copy)]
struct LineNewParam {
    name: &'static str,
    accepts: Accepts,
    optional: bool,
}

const LINE_NEW_SCALAR_PARAMS: &[LineNewParam] = &[
    LineNewParam {
        name: "x1",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    LineNewParam {
        name: "y1",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    LineNewParam {
        name: "x2",
        accepts: Accepts::IntCompatible,
        optional: false,
    },
    LineNewParam {
        name: "y2",
        accepts: Accepts::NumericCompatible,
        optional: false,
    },
    LineNewParam {
        name: "xloc",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "extend",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    LineNewParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "width",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
    LineNewParam {
        name: "force_overlay",
        accepts: Accepts::ConstBool,
        optional: true,
    },
];

const LINE_NEW_POINT_PARAMS: &[LineNewParam] = &[
    LineNewParam {
        name: "first_point",
        accepts: Accepts::ChartPointCompatible,
        optional: false,
    },
    LineNewParam {
        name: "second_point",
        accepts: Accepts::ChartPointCompatible,
        optional: false,
    },
    LineNewParam {
        name: "xloc",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "extend",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "color",
        accepts: Accepts::ColorCompatible,
        optional: true,
    },
    LineNewParam {
        name: "style",
        accepts: Accepts::ConstString,
        optional: true,
    },
    LineNewParam {
        name: "width",
        accepts: Accepts::IntCompatible,
        optional: true,
    },
    LineNewParam {
        name: "force_overlay",
        accepts: Accepts::ConstBool,
        optional: true,
    },
];

impl Analyzer {
    pub(crate) fn validate_line_new_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> bool {
        if signature.name != "line.new" {
            return false;
        }
        let params = if line_new_uses_point_overload(args, arg_types) {
            LINE_NEW_POINT_PARAMS
        } else {
            LINE_NEW_SCALAR_PARAMS
        };
        self.validate_line_new_overload(args, arg_types, params);
        true
    }

    fn validate_line_new_overload(
        &mut self,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
        params: &[LineNewParam],
    ) {
        let required_count = params.iter().filter(|param| !param.optional).count();
        if args.len() < required_count {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`line.new` expects at least {required_count} argument(s), got {}",
                    args.len()
                ),
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
            return;
        }
        if args.len() > params.len() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`line.new` expects at most {} argument(s), got {}",
                    params.len(),
                    args.len()
                ),
                args[params.len()].span,
            ));
        }
        for (index, arg) in args.iter().enumerate() {
            let Some(param) = line_new_param(params, index, arg, &mut self.diagnostics) else {
                continue;
            };
            let Some(arg_type) = arg_types.get(index).copied().flatten() else {
                continue;
            };
            if !accepts_type(param.accepts, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`line.new` argument `{}` does not accept {:?} {:?}",
                        param.name, arg_type.qualifier, arg_type.kind
                    ),
                    arg.span,
                ));
            }
        }
        self.validate_line_new_string_arg(args, params, "xloc", LINE_XLOCS);
        self.validate_line_new_string_arg(args, params, "extend", LINE_EXTENDS);
        self.validate_line_new_string_arg(args, params, "style", LINE_STYLES);
    }

    fn validate_line_new_string_arg(
        &mut self,
        args: &[CallArg],
        params: &[LineNewParam],
        name: &str,
        supported: &[&str],
    ) {
        for (index, arg) in args.iter().enumerate() {
            let Some(param) = line_new_param(params, index, arg, &mut Vec::new()) else {
                continue;
            };
            if param.name != name {
                continue;
            }
            let supported_value = const_string_value(&arg.value)
                .as_deref()
                .is_some_and(|value| supported.contains(&value));
            if !supported_value {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    format!(
                        "`line.new` argument `{name}` only supports {}",
                        supported.join(", ")
                    ),
                    arg.span,
                ));
            }
        }
    }
}

fn line_new_uses_point_overload(args: &[CallArg], arg_types: &[Option<PineType>]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.name.as_deref(), Some("first_point" | "second_point")))
        || arg_types
            .first()
            .copied()
            .flatten()
            .is_some_and(|arg_type| arg_type.kind == ValueKind::ChartPoint)
}

fn line_new_param<'a>(
    params: &'a [LineNewParam],
    index: usize,
    arg: &CallArg,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<&'a LineNewParam> {
    if let Some(name) = &arg.name {
        let param = params.iter().find(|param| param.name == name);
        if param.is_none() {
            diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                format!("`line.new` has no argument named `{name}`"),
                arg.span,
            ));
        }
        param
    } else {
        params.get(index)
    }
}
