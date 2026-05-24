use crate::prelude::*;

fn has_alert_placeholder(value: &str) -> bool {
    value.contains("{{") && value.contains("}}")
}

impl Analyzer {
    pub(crate) fn validate_alert_args(&mut self, signature: &BuiltinSignature, args: &[CallArg]) {
        if !matches!(signature.name, "alert" | "alertcondition") {
            return;
        }

        for (index, arg) in args.iter().enumerate() {
            let Some(param_name) = self
                .resolve_param(signature, index, arg)
                .map(|param| param.name)
            else {
                continue;
            };

            if signature.name == "alert" && param_name == "freq" {
                self.unsupported(
                    "alert_frequency",
                    "alert frequency modes are not supported in the current alert subset",
                    arg.span,
                );
            }

            if matches!(param_name, "message" | "title")
                && const_string_value(&arg.value)
                    .as_deref()
                    .is_some_and(has_alert_placeholder)
            {
                self.unsupported(
                    "alert_placeholders",
                    "alert placeholder interpolation is not supported in the current alert subset",
                    arg.span,
                );
            }
        }
    }
}
