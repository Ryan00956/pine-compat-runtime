use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::running_alerts::{
    RunningAlertConfig, RunningAlertEvaluationError, render_strategy_order_fill_running_alert,
};
use super::strategy::StrategyOrderFillAlertOutput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeliveryEventKind {
    IndicatorAlertCall,
    StrategyOrderFill,
}

impl DeliveryEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndicatorAlertCall => "indicatorAlertCall",
            Self::StrategyOrderFill => "strategyOrderFill",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryCandidate {
    pub running_alert_id: String,
    pub script_snapshot_id: String,
    pub event_kind: DeliveryEventKind,
    pub bar_index: usize,
    pub time: i64,
    pub event_id: String,
    pub rendered_message: String,
}

impl DeliveryCandidate {
    pub fn new(
        running_alert_id: impl Into<String>,
        script_snapshot_id: impl Into<String>,
        event_kind: DeliveryEventKind,
        bar_index: usize,
        time: i64,
        event_id: impl Into<String>,
        rendered_message: impl Into<String>,
    ) -> Self {
        Self {
            running_alert_id: running_alert_id.into(),
            script_snapshot_id: script_snapshot_id.into(),
            event_kind,
            bar_index,
            time,
            event_id: event_id.into(),
            rendered_message: rendered_message.into(),
        }
    }

    pub fn dedupe_key(&self) -> DeliveryDedupeKey {
        DeliveryDedupeKey {
            running_alert_id: self.running_alert_id.clone(),
            script_snapshot_id: self.script_snapshot_id.clone(),
            event_kind: self.event_kind,
            bar_index: self.bar_index,
            time: self.time,
            event_id: self.event_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDedupeKey {
    pub running_alert_id: String,
    pub script_snapshot_id: String,
    pub event_kind: DeliveryEventKind,
    pub bar_index: usize,
    pub time: i64,
    pub event_id: String,
}

impl From<&DeliveryCandidate> for DeliveryDedupeKey {
    fn from(candidate: &DeliveryCandidate) -> Self {
        candidate.dedupe_key()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDeliveryIdentity {
    pub adapter_id: String,
    pub dedupe_key: DeliveryDedupeKey,
}

impl ExternalDeliveryIdentity {
    pub fn new(adapter_id: impl Into<String>, dedupe_key: DeliveryDedupeKey) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            dedupe_key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeliveryAttemptStatus {
    Pending,
    InFlight,
    Delivered,
    TransientFailure,
    PermanentFailure,
}

impl DeliveryAttemptStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::PermanentFailure)
    }

    pub fn is_retryable_failure(&self) -> bool {
        matches!(self, Self::TransientFailure)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryAttemptRecord {
    pub dedupe_key: DeliveryDedupeKey,
    pub adapter_id: String,
    pub attempt_number: u32,
    pub status: DeliveryAttemptStatus,
    pub scheduled_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub next_retry_at: Option<i64>,
    pub failure_code: Option<String>,
}

impl DeliveryAttemptRecord {
    pub fn new(
        dedupe_key: DeliveryDedupeKey,
        adapter_id: impl Into<String>,
        attempt_number: u32,
        scheduled_at: i64,
    ) -> Self {
        Self {
            dedupe_key,
            adapter_id: adapter_id.into(),
            attempt_number,
            status: DeliveryAttemptStatus::Pending,
            scheduled_at,
            started_at: None,
            completed_at: None,
            next_retry_at: None,
            failure_code: None,
        }
    }

    pub fn external_identity(&self) -> ExternalDeliveryIdentity {
        ExternalDeliveryIdentity::new(self.adapter_id.clone(), self.dedupe_key.clone())
    }

    pub fn start(mut self, started_at: i64) -> Self {
        self.status = DeliveryAttemptStatus::InFlight;
        self.started_at = Some(started_at);
        self
    }

    pub fn complete(mut self, result: &ExternalDeliveryResult) -> Self {
        self.status = result.status.into();
        self.completed_at = Some(result.completed_at);
        self.failure_code = result.failure_code.clone();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalDeliveryStatus {
    Delivered,
    TransientFailure,
    PermanentFailure,
}

impl ExternalDeliveryStatus {
    pub fn is_retryable_failure(&self) -> bool {
        matches!(self, Self::TransientFailure)
    }
}

impl From<ExternalDeliveryStatus> for DeliveryAttemptStatus {
    fn from(status: ExternalDeliveryStatus) -> Self {
        match status {
            ExternalDeliveryStatus::Delivered => Self::Delivered,
            ExternalDeliveryStatus::TransientFailure => Self::TransientFailure,
            ExternalDeliveryStatus::PermanentFailure => Self::PermanentFailure,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDeliveryResult {
    pub status: ExternalDeliveryStatus,
    pub provider_status_code: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub completed_at: i64,
}

impl ExternalDeliveryResult {
    pub fn delivered(completed_at: i64) -> Self {
        Self {
            status: ExternalDeliveryStatus::Delivered,
            provider_status_code: None,
            failure_code: None,
            failure_message: None,
            completed_at,
        }
    }

    pub fn transient_failure(
        completed_at: i64,
        failure_code: impl Into<String>,
        failure_message: impl Into<String>,
    ) -> Self {
        Self {
            status: ExternalDeliveryStatus::TransientFailure,
            provider_status_code: None,
            failure_code: Some(failure_code.into()),
            failure_message: Some(failure_message.into()),
            completed_at,
        }
    }

    pub fn permanent_failure(
        completed_at: i64,
        failure_code: impl Into<String>,
        failure_message: impl Into<String>,
    ) -> Self {
        Self {
            status: ExternalDeliveryStatus::PermanentFailure,
            provider_status_code: None,
            failure_code: Some(failure_code.into()),
            failure_message: Some(failure_message.into()),
            completed_at,
        }
    }

    pub fn with_provider_status_code(mut self, status_code: impl Into<String>) -> Self {
        self.provider_status_code = Some(status_code.into());
        self
    }
}

pub fn strategy_order_fill_delivery_candidate(
    running_alert_id: impl Into<String>,
    config: &RunningAlertConfig,
    alert: &StrategyOrderFillAlertOutput,
) -> Result<DeliveryCandidate, RunningAlertEvaluationError> {
    let rendered_message = render_strategy_order_fill_running_alert(config, alert)?;
    Ok(DeliveryCandidate::new(
        running_alert_id,
        config.script_snapshot_id.clone(),
        DeliveryEventKind::StrategyOrderFill,
        alert.bar_index,
        alert.time,
        alert.id.clone(),
        rendered_message,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Delivered,
    Duplicate,
}

pub trait DeliverySink {
    fn deliver(&mut self, candidate: DeliveryCandidate) -> DeliveryOutcome;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryDeliverySink {
    delivered_keys: BTreeSet<DeliveryDedupeKey>,
    delivered: Vec<DeliveryCandidate>,
}

impl InMemoryDeliverySink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delivered(&self) -> &[DeliveryCandidate] {
        &self.delivered
    }

    pub fn delivered_keys(&self) -> &BTreeSet<DeliveryDedupeKey> {
        &self.delivered_keys
    }

    pub fn has_delivered(&self, key: &DeliveryDedupeKey) -> bool {
        self.delivered_keys.contains(key)
    }
}

impl DeliverySink for InMemoryDeliverySink {
    fn deliver(&mut self, candidate: DeliveryCandidate) -> DeliveryOutcome {
        let key = candidate.dedupe_key();
        if !self.delivered_keys.insert(key) {
            return DeliveryOutcome::Duplicate;
        }
        self.delivered.push(candidate);
        DeliveryOutcome::Delivered
    }
}

#[cfg(test)]
mod tests {
    use super::super::running_alerts::{RunningAlertConfig, RunningAlertEventSelection};
    use super::*;

    fn candidate(message: &str) -> DeliveryCandidate {
        DeliveryCandidate::new(
            "alert-1",
            "snapshot-1",
            DeliveryEventKind::StrategyOrderFill,
            2,
            300,
            "XL",
            message,
        )
    }

    fn alert(message: &str) -> StrategyOrderFillAlertOutput {
        StrategyOrderFillAlertOutput {
            id: "XL".to_owned(),
            bar_index: 2,
            time: 300,
            direction: "strategy.exit".to_owned(),
            qty: 1.0,
            price: 99.0,
            entry_id: Some("L".to_owned()),
            exit_id: Some("XL".to_owned()),
            message: message.to_owned(),
        }
    }

    #[test]
    fn delivery_candidate_builds_stable_dedupe_key_without_message() {
        let key = candidate("first").dedupe_key();

        assert_eq!(key.running_alert_id, "alert-1");
        assert_eq!(key.script_snapshot_id, "snapshot-1");
        assert_eq!(key.event_kind, DeliveryEventKind::StrategyOrderFill);
        assert_eq!(key.bar_index, 2);
        assert_eq!(key.time, 300);
        assert_eq!(key.event_id, "XL");
    }

    #[test]
    fn delivery_candidate_serializes_host_owned_shape() {
        let value = serde_json::to_value(candidate("Rendered")).expect("candidate serializes");

        assert_eq!(
            value,
            serde_json::json!({
                "runningAlertId": "alert-1",
                "scriptSnapshotId": "snapshot-1",
                "eventKind": "strategyOrderFill",
                "barIndex": 2,
                "time": 300,
                "eventId": "XL",
                "renderedMessage": "Rendered",
            })
        );
    }

    #[test]
    fn in_memory_sink_collects_candidates_once_per_dedupe_key() {
        let mut sink = InMemoryDeliverySink::new();
        let first = candidate("first");
        let duplicate = candidate("second");
        let key = first.dedupe_key();

        assert_eq!(sink.deliver(first), DeliveryOutcome::Delivered);
        assert_eq!(sink.deliver(duplicate), DeliveryOutcome::Duplicate);

        assert!(sink.has_delivered(&key));
        assert_eq!(sink.delivered_keys().len(), 1);
        assert_eq!(sink.delivered().len(), 1);
        assert_eq!(sink.delivered()[0].rendered_message, "first");
    }

    #[test]
    fn in_memory_sink_allows_same_event_for_different_running_alerts() {
        let mut sink = InMemoryDeliverySink::new();
        let mut second = candidate("second alert");
        second.running_alert_id = "alert-2".to_owned();

        assert_eq!(
            sink.deliver(candidate("first alert")),
            DeliveryOutcome::Delivered
        );
        assert_eq!(sink.deliver(second), DeliveryOutcome::Delivered);

        assert_eq!(sink.delivered_keys().len(), 2);
        assert_eq!(sink.delivered().len(), 2);
    }

    #[test]
    fn external_delivery_identity_combines_adapter_and_dedupe_key() {
        let key = candidate("message").dedupe_key();
        let identity = ExternalDeliveryIdentity::new("webhook-main", key.clone());

        assert_eq!(identity.adapter_id, "webhook-main");
        assert_eq!(identity.dedupe_key, key);
    }

    #[test]
    fn delivery_attempt_record_serializes_host_owned_shape() {
        let record =
            DeliveryAttemptRecord::new(candidate("message").dedupe_key(), "webhook-main", 1, 1_000)
                .start(1_010)
                .complete(
                    &ExternalDeliveryResult::transient_failure(
                        1_020,
                        "timeout",
                        "adapter timed out",
                    )
                    .with_provider_status_code("504"),
                );

        let value = serde_json::to_value(record).expect("attempt record serializes");

        assert_eq!(
            value,
            serde_json::json!({
                "dedupeKey": {
                    "runningAlertId": "alert-1",
                    "scriptSnapshotId": "snapshot-1",
                    "eventKind": "strategyOrderFill",
                    "barIndex": 2,
                    "time": 300,
                    "eventId": "XL",
                },
                "adapterId": "webhook-main",
                "attemptNumber": 1,
                "status": "transientFailure",
                "scheduledAt": 1000,
                "startedAt": 1010,
                "completedAt": 1020,
                "nextRetryAt": null,
                "failureCode": "timeout",
            })
        );
    }

    #[test]
    fn delivery_attempt_record_keeps_external_identity_stable_across_attempts() {
        let key = candidate("message").dedupe_key();
        let first = DeliveryAttemptRecord::new(key.clone(), "webhook-main", 1, 1_000);
        let second = DeliveryAttemptRecord::new(key, "webhook-main", 2, 2_000);

        assert_ne!(first.attempt_number, second.attempt_number);
        assert_eq!(first.external_identity(), second.external_identity());
    }

    #[test]
    fn external_delivery_result_classifies_retryable_status() {
        let delivered = ExternalDeliveryResult::delivered(1_000);
        let transient = ExternalDeliveryResult::transient_failure(1_001, "timeout", "timeout");
        let permanent =
            ExternalDeliveryResult::permanent_failure(1_002, "badRequest", "bad request");

        assert!(!delivered.status.is_retryable_failure());
        assert!(transient.status.is_retryable_failure());
        assert!(!permanent.status.is_retryable_failure());
        assert!(DeliveryAttemptStatus::from(delivered.status).is_terminal());
        assert!(DeliveryAttemptStatus::from(transient.status).is_retryable_failure());
        assert!(DeliveryAttemptStatus::from(permanent.status).is_terminal());
    }

    #[test]
    fn strategy_order_fill_builder_renders_delivery_candidate() {
        let config = RunningAlertConfig::new_strategy_order_fills(
            "snapshot-1",
            "NYSE:IBM",
            "1",
            "Running: {{strategy.order.alert_message}}",
        );

        let candidate =
            strategy_order_fill_delivery_candidate("alert-1", &config, &alert("loss alert"))
                .expect("delivery candidate");

        assert_eq!(candidate.running_alert_id, "alert-1");
        assert_eq!(candidate.script_snapshot_id, "snapshot-1");
        assert_eq!(candidate.event_kind, DeliveryEventKind::StrategyOrderFill);
        assert_eq!(candidate.bar_index, 2);
        assert_eq!(candidate.time, 300);
        assert_eq!(candidate.event_id, "XL");
        assert_eq!(candidate.rendered_message, "Running: loss alert");
    }

    #[test]
    fn strategy_order_fill_builder_uses_candidate_dedupe_key() {
        let config = RunningAlertConfig::new_strategy_order_fills(
            "snapshot-1",
            "NYSE:IBM",
            "1",
            "{{strategy.order.alert_message}}",
        );
        let candidate =
            strategy_order_fill_delivery_candidate("alert-1", &config, &alert("loss alert"))
                .expect("delivery candidate");

        assert_eq!(
            candidate.dedupe_key(),
            DeliveryDedupeKey {
                running_alert_id: "alert-1".to_owned(),
                script_snapshot_id: "snapshot-1".to_owned(),
                event_kind: DeliveryEventKind::StrategyOrderFill,
                bar_index: 2,
                time: 300,
                event_id: "XL".to_owned(),
            }
        );
    }

    #[test]
    fn strategy_order_fill_builder_keeps_both_selection_design_only() {
        let mut config = RunningAlertConfig::new_strategy_order_fills(
            "snapshot-1",
            "NYSE:IBM",
            "1",
            "{{strategy.order.alert_message}}",
        );
        config.event_selection = RunningAlertEventSelection::Both;

        let error =
            strategy_order_fill_delivery_candidate("alert-1", &config, &alert("loss alert"))
                .expect_err("both should not build strategy-only candidate");

        assert_eq!(
            error,
            RunningAlertEvaluationError::UnsupportedEventSelection {
                selection: RunningAlertEventSelection::Both,
            }
        );
    }
}
