use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunningAlertEventSelection {
    IndicatorAlertCalls,
    StrategyOrderFills,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunningAlertRealtimePolicy {
    RealtimeOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningAlertConfig {
    pub script_snapshot_id: String,
    pub symbol: String,
    pub timeframe: String,
    pub event_selection: RunningAlertEventSelection,
    pub message_template: String,
    pub realtime_policy: RunningAlertRealtimePolicy,
}

impl RunningAlertConfig {
    pub fn new_strategy_order_fills(
        script_snapshot_id: impl Into<String>,
        symbol: impl Into<String>,
        timeframe: impl Into<String>,
        message_template: impl Into<String>,
    ) -> Self {
        Self {
            script_snapshot_id: script_snapshot_id.into(),
            symbol: symbol.into(),
            timeframe: timeframe.into(),
            event_selection: RunningAlertEventSelection::StrategyOrderFills,
            message_template: message_template.into(),
            realtime_policy: RunningAlertRealtimePolicy::RealtimeOnly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_order_fill_constructor_sets_realtime_only_boundary() {
        let config = RunningAlertConfig::new_strategy_order_fills(
            "snapshot-1",
            "NASDAQ:AAPL",
            "60",
            "Order: {{strategy.order.alert_message}}",
        );

        assert_eq!(config.script_snapshot_id, "snapshot-1");
        assert_eq!(config.symbol, "NASDAQ:AAPL");
        assert_eq!(config.timeframe, "60");
        assert_eq!(
            config.event_selection,
            RunningAlertEventSelection::StrategyOrderFills
        );
        assert_eq!(
            config.message_template,
            "Order: {{strategy.order.alert_message}}"
        );
        assert_eq!(
            config.realtime_policy,
            RunningAlertRealtimePolicy::RealtimeOnly
        );
    }

    #[test]
    fn serializes_stable_host_owned_shape() {
        let config = RunningAlertConfig::new_strategy_order_fills(
            "snapshot-1",
            "NASDAQ:AAPL",
            "60",
            "Order: {{strategy.order.alert_message}}",
        );

        let json = serde_json::to_value(&config).expect("config should serialize");

        assert_eq!(
            json,
            serde_json::json!({
                "scriptSnapshotId": "snapshot-1",
                "symbol": "NASDAQ:AAPL",
                "timeframe": "60",
                "eventSelection": "strategyOrderFills",
                "messageTemplate": "Order: {{strategy.order.alert_message}}",
                "realtimePolicy": "realtimeOnly"
            })
        );
    }

    #[test]
    fn deserializes_all_documented_event_selection_variants() {
        let selections = [
            (
                "indicatorAlertCalls",
                RunningAlertEventSelection::IndicatorAlertCalls,
            ),
            (
                "strategyOrderFills",
                RunningAlertEventSelection::StrategyOrderFills,
            ),
            ("both", RunningAlertEventSelection::Both),
        ];

        for (selection_json, expected) in selections {
            let config: RunningAlertConfig = serde_json::from_value(serde_json::json!({
                "scriptSnapshotId": "snapshot-1",
                "symbol": "NASDAQ:AAPL",
                "timeframe": "60",
                "eventSelection": selection_json,
                "messageTemplate": "Order: {{strategy.order.alert_message}}",
                "realtimePolicy": "realtimeOnly"
            }))
            .expect("config should deserialize");

            assert_eq!(config.event_selection, expected);
        }
    }
}
