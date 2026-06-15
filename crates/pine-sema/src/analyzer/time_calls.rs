use crate::prelude::*;

impl Analyzer {
    pub(crate) fn validate_timestamp_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        let first_positional_is_timezone = args.first().is_some_and(|arg| arg.name.is_none())
            && arg_types
                .first()
                .copied()
                .flatten()
                .is_some_and(|arg_type| arg_type.kind == ValueKind::String);
        let has_timezone_arg = first_positional_is_timezone
            || args
                .iter()
                .any(|arg| arg.name.as_deref() == Some("timezone"));
        let max_args = if has_timezone_arg { 7 } else { 6 };
        if args.len() > max_args {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{}` expects at most {max_args} argument(s), got {}",
                    signature.name,
                    args.len()
                ),
                args[max_args].span,
            ));
        }

        let mut has_year = false;
        let mut has_month = false;
        let mut has_day = false;
        for (index, arg) in args.iter().enumerate() {
            let Some((param_name, accepts)) = self.resolve_timestamp_arg(
                signature.name,
                args,
                arg_types,
                index,
                first_positional_is_timezone,
            ) else {
                continue;
            };
            match param_name {
                "year" => has_year = true,
                "month" => has_month = true,
                "day" => has_day = true,
                _ => {}
            }
            self.validate_timestamp_arg_type(
                signature.name,
                param_name,
                accepts,
                index,
                arg,
                arg_types,
            );
        }

        if !has_year || !has_month || !has_day {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                "`timestamp` expects year, month, and day arguments",
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
        }
    }

    fn validate_timestamp_arg_type(
        &mut self,
        function_name: &str,
        param_name: &str,
        accepts: Accepts,
        index: usize,
        arg: &CallArg,
        arg_types: &[Option<PineType>],
    ) {
        let Some(arg_type) = arg_types.get(index).copied().flatten() else {
            return;
        };
        if !accepts_type(accepts, arg_type) {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{}` argument `{}` does not accept {:?} {:?}",
                    function_name, param_name, arg_type.qualifier, arg_type.kind
                ),
                arg.span,
            ));
        }
    }

    fn resolve_timestamp_arg(
        &mut self,
        function_name: &str,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
        index: usize,
        first_positional_is_timezone: bool,
    ) -> Option<(&'static str, Accepts)> {
        let arg = args.get(index)?;
        if let Some(name) = &arg.name {
            return match name.as_str() {
                "timezone" => Some(("timezone", Accepts::StringCompatible)),
                "year" => Some(("year", Accepts::IntCompatible)),
                "month" => Some(("month", Accepts::IntCompatible)),
                "day" => Some(("day", Accepts::IntCompatible)),
                "hour" => Some(("hour", Accepts::IntCompatible)),
                "minute" => Some(("minute", Accepts::IntCompatible)),
                "second" => Some(("second", Accepts::IntCompatible)),
                _ => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        format!("`{function_name}` has no argument named `{name}`"),
                        arg.span,
                    ));
                    None
                }
            };
        }

        if index == 0 && first_positional_is_timezone {
            return Some(("timezone", Accepts::StringCompatible));
        }
        if index == 0
            && arg_types
                .first()
                .copied()
                .flatten()
                .is_some_and(|arg_type| !accepts_type(Accepts::IntCompatible, arg_type))
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                "`timestamp` first positional argument must be a year int or timezone string",
                arg.span,
            ));
            return None;
        }

        let calendar_index = if first_positional_is_timezone {
            index.checked_sub(1)?
        } else {
            index
        };
        Some((
            match calendar_index {
                0 => "year",
                1 => "month",
                2 => "day",
                3 => "hour",
                4 => "minute",
                5 => "second",
                _ => return None,
            },
            Accepts::IntCompatible,
        ))
    }
}
