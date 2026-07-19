use pine_syntax::{CallArg, Span};

use super::PineDialect;

pub(crate) const LEGACY_CALL_BINDING_DEFERRED_REASON: &str = "this legacy call requires version-specific overload and argument binding that is not implemented yet";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyRegisteredCallGuard {
    pub(crate) feature: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) span: Span,
}

pub(crate) fn registered_call_guard(
    dialect: PineDialect,
    name: &str,
    callee_span: Span,
    args: &[CallArg],
) -> Option<LegacyRegisteredCallGuard> {
    if !dialect.is_legacy() || name != "time" {
        return None;
    }

    let session_arg = args.iter().enumerate().find(|(index, arg)| {
        arg.name.as_deref() == Some("session") || (arg.name.is_none() && *index == 1)
    })?;
    Some(LegacyRegisteredCallGuard {
        feature: "time.session",
        reason: "legacy session strings use version-specific default weekdays that are deferred to the legacy expression semantics phase",
        span: session_arg.1.span.merge(callee_span),
    })
}
