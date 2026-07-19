use std::collections::HashMap;

use pine_syntax::Span;

use crate::source_graph::SourceContextId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LegacyUseKey {
    source_context_id: SourceContextId,
    span_start: usize,
    span_end: usize,
}

#[derive(Debug, Default)]
pub(crate) struct LegacyLoweringPlan {
    calls: HashMap<LegacyUseKey, &'static str>,
    call_arg_names: HashMap<LegacyUseKey, Vec<Option<&'static str>>>,
    values: HashMap<LegacyUseKey, &'static str>,
}

impl LegacyLoweringPlan {
    pub(crate) fn new() -> Self {
        Self {
            calls: HashMap::new(),
            call_arg_names: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub(crate) fn record_call_arg_names(
        &mut self,
        source_context_id: SourceContextId,
        span: Span,
        names: Vec<Option<&'static str>>,
    ) {
        self.call_arg_names.insert(
            LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            },
            names,
        );
    }

    pub(crate) fn record_call(
        &mut self,
        source_context_id: SourceContextId,
        span: Span,
        canonical_name: &'static str,
    ) {
        self.calls.insert(
            LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            },
            canonical_name,
        );
    }

    pub(crate) fn record_value(
        &mut self,
        source_context_id: SourceContextId,
        span: Span,
        canonical_name: &'static str,
    ) {
        self.values.insert(
            LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            },
            canonical_name,
        );
    }

    pub(crate) fn call_name(
        &self,
        source_context_id: SourceContextId,
        span: Span,
    ) -> Option<&'static str> {
        self.calls
            .get(&LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            })
            .copied()
    }

    pub(crate) fn value_name(
        &self,
        source_context_id: SourceContextId,
        span: Span,
    ) -> Option<&'static str> {
        self.values
            .get(&LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            })
            .copied()
    }

    pub(crate) fn call_arg_names(
        &self,
        source_context_id: SourceContextId,
        span: Span,
    ) -> Option<&[Option<&'static str>]> {
        self.call_arg_names
            .get(&LegacyUseKey {
                source_context_id,
                span_start: span.start,
                span_end: span.end,
            })
            .map(Vec::as_slice)
    }
}
