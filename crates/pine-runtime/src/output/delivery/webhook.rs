use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::DeliveryCandidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebhookBodyMode {
    RenderedMessage,
    JsonEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookAdapterConfig {
    pub adapter_id: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub secret_header_refs: BTreeMap<String, String>,
    pub body_mode: WebhookBodyMode,
    pub timeout_ms: u32,
}

impl WebhookAdapterConfig {
    pub fn new(
        adapter_id: impl Into<String>,
        url: impl Into<String>,
        body_mode: WebhookBodyMode,
        timeout_ms: u32,
    ) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            url: url.into(),
            headers: BTreeMap::new(),
            secret_header_refs: BTreeMap::new(),
            body_mode,
            timeout_ms,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_secret_header_ref(
        mut self,
        name: impl Into<String>,
        secret_ref: impl Into<String>,
    ) -> Self {
        self.secret_header_refs
            .insert(name.into(), secret_ref.into());
        self
    }

    pub fn validate(&self) -> Result<(), WebhookAdapterConfigError> {
        if self.adapter_id.trim().is_empty() {
            return Err(WebhookAdapterConfigError::EmptyAdapterId);
        }
        validate_webhook_url(&self.url)?;
        if self.timeout_ms == 0 || self.timeout_ms > MAX_WEBHOOK_TIMEOUT_MS {
            return Err(WebhookAdapterConfigError::InvalidTimeout {
                timeout_ms: self.timeout_ms,
            });
        }
        validate_webhook_headers(self)
    }
}

const MAX_WEBHOOK_TIMEOUT_MS: u32 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookAdapterConfigError {
    EmptyAdapterId,
    EmptyUrl,
    UnsupportedUrlScheme,
    MissingUrlHost,
    UrlContainsCredentials,
    InvalidUrlPort { port: String },
    UnsupportedUrlPort { port: u16 },
    InvalidTimeout { timeout_ms: u32 },
    EmptyHeaderName,
    DuplicateHeaderName { header_name: String },
    StaticHeaderMayContainSecret { header_name: String },
    EmptySecretReference { header_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub content_type: String,
    pub body: String,
}

impl WebhookPayload {
    pub fn new(content_type: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            content_type: content_type.into(),
            body: body.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookPayloadError {
    InvalidConfig(WebhookAdapterConfigError),
    JsonSerializationFailed(String),
}

impl From<WebhookAdapterConfigError> for WebhookPayloadError {
    fn from(error: WebhookAdapterConfigError) -> Self {
        Self::InvalidConfig(error)
    }
}

pub fn render_webhook_payload(
    config: &WebhookAdapterConfig,
    candidate: &DeliveryCandidate,
) -> Result<WebhookPayload, WebhookPayloadError> {
    config.validate()?;
    match config.body_mode {
        WebhookBodyMode::RenderedMessage => Ok(WebhookPayload::new(
            "text/plain; charset=utf-8",
            candidate.rendered_message.clone(),
        )),
        WebhookBodyMode::JsonEnvelope => render_webhook_json_envelope(config, candidate),
    }
}

fn render_webhook_json_envelope(
    config: &WebhookAdapterConfig,
    candidate: &DeliveryCandidate,
) -> Result<WebhookPayload, WebhookPayloadError> {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "adapterId": config.adapter_id,
        "bodyMode": config.body_mode,
        "candidate": candidate,
    });
    let body = serde_json::to_string(&value)
        .map_err(|error| WebhookPayloadError::JsonSerializationFailed(error.to_string()))?;
    Ok(WebhookPayload::new("application/json", body))
}

fn validate_webhook_url(url: &str) -> Result<(), WebhookAdapterConfigError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(WebhookAdapterConfigError::EmptyUrl);
    }
    let Some((scheme, rest)) = trimmed.split_once("://") else {
        return Err(WebhookAdapterConfigError::UnsupportedUrlScheme);
    };
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(WebhookAdapterConfigError::UnsupportedUrlScheme);
    }
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .expect("split always yields first segment");
    if authority.is_empty() {
        return Err(WebhookAdapterConfigError::MissingUrlHost);
    }
    if authority.contains('@') {
        return Err(WebhookAdapterConfigError::UrlContainsCredentials);
    }
    let host_port = authority.rsplit_once(':');
    if let Some((host, port_text)) = host_port {
        if host.is_empty() {
            return Err(WebhookAdapterConfigError::MissingUrlHost);
        }
        let Ok(port) = port_text.parse::<u16>() else {
            return Err(WebhookAdapterConfigError::InvalidUrlPort {
                port: port_text.to_owned(),
            });
        };
        if port != 80 && port != 443 {
            return Err(WebhookAdapterConfigError::UnsupportedUrlPort { port });
        }
    }
    Ok(())
}

fn validate_webhook_headers(
    config: &WebhookAdapterConfig,
) -> Result<(), WebhookAdapterConfigError> {
    let mut normalized_names = BTreeSet::new();
    for (name, value) in &config.headers {
        let normalized = normalize_header_name(name)?;
        if !normalized_names.insert(normalized.clone()) {
            return Err(WebhookAdapterConfigError::DuplicateHeaderName {
                header_name: name.clone(),
            });
        }
        if header_name_looks_secret(&normalized) || header_value_looks_secret(value) {
            return Err(WebhookAdapterConfigError::StaticHeaderMayContainSecret {
                header_name: name.clone(),
            });
        }
    }
    for (name, secret_ref) in &config.secret_header_refs {
        let normalized = normalize_header_name(name)?;
        if !normalized_names.insert(normalized) {
            return Err(WebhookAdapterConfigError::DuplicateHeaderName {
                header_name: name.clone(),
            });
        }
        if secret_ref.trim().is_empty() {
            return Err(WebhookAdapterConfigError::EmptySecretReference {
                header_name: name.clone(),
            });
        }
    }
    Ok(())
}

fn normalize_header_name(name: &str) -> Result<String, WebhookAdapterConfigError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(WebhookAdapterConfigError::EmptyHeaderName);
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn header_name_looks_secret(normalized_name: &str) -> bool {
    normalized_name == "authorization"
        || normalized_name.contains("api-key")
        || normalized_name.contains("token")
        || normalized_name.contains("secret")
        || normalized_name.contains("password")
}

fn header_value_looks_secret(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("bearer ") || normalized.contains("password=")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::delivery::DeliveryEventKind;

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

    #[test]
    fn webhook_adapter_config_accepts_host_owned_safe_shape() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/tradingview",
            WebhookBodyMode::JsonEnvelope,
            5_000,
        )
        .with_header("Content-Type", "application/json")
        .with_secret_header_ref("X-Webhook-Signature", "secret://alerts/webhook-signature");

        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn webhook_adapter_config_serializes_camel_case_shape() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            5_000,
        )
        .with_header("Content-Type", "text/plain")
        .with_secret_header_ref("X-Token", "secret://token");

        let value = serde_json::to_value(config).expect("webhook config serializes");

        assert_eq!(
            value,
            serde_json::json!({
                "adapterId": "webhook-main",
                "url": "https://example.com/hook",
                "headers": {
                    "Content-Type": "text/plain",
                },
                "secretHeaderRefs": {
                    "X-Token": "secret://token",
                },
                "bodyMode": "renderedMessage",
                "timeoutMs": 5000,
            })
        );
    }

    #[test]
    fn webhook_adapter_config_rejects_unsafe_urls() {
        let cases = [
            ("", WebhookAdapterConfigError::EmptyUrl),
            ("/relative", WebhookAdapterConfigError::UnsupportedUrlScheme),
            (
                "file:///tmp/hook",
                WebhookAdapterConfigError::UnsupportedUrlScheme,
            ),
            ("https:///hook", WebhookAdapterConfigError::MissingUrlHost),
            (
                "https://user:pass@example.com/hook",
                WebhookAdapterConfigError::UrlContainsCredentials,
            ),
            (
                "https://example.com:abc/hook",
                WebhookAdapterConfigError::InvalidUrlPort {
                    port: "abc".to_owned(),
                },
            ),
            (
                "https://example.com:8080/hook",
                WebhookAdapterConfigError::UnsupportedUrlPort { port: 8080 },
            ),
        ];

        for (url, expected) in cases {
            let config =
                WebhookAdapterConfig::new("webhook-main", url, WebhookBodyMode::RenderedMessage, 1);
            assert_eq!(config.validate(), Err(expected));
        }
    }

    #[test]
    fn webhook_adapter_config_rejects_invalid_timeout() {
        let zero = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            0,
        );
        let too_large = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            30_001,
        );

        assert_eq!(
            zero.validate(),
            Err(WebhookAdapterConfigError::InvalidTimeout { timeout_ms: 0 })
        );
        assert_eq!(
            too_large.validate(),
            Err(WebhookAdapterConfigError::InvalidTimeout { timeout_ms: 30_001 })
        );
    }

    #[test]
    fn webhook_adapter_config_rejects_duplicate_header_names() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            1_000,
        )
        .with_header("X-Trace", "public")
        .with_secret_header_ref("x-trace", "secret://trace");

        assert_eq!(
            config.validate(),
            Err(WebhookAdapterConfigError::DuplicateHeaderName {
                header_name: "x-trace".to_owned(),
            })
        );
    }

    #[test]
    fn webhook_adapter_config_rejects_static_secret_headers() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            1_000,
        )
        .with_header("Authorization", "Bearer abc123");

        assert_eq!(
            config.validate(),
            Err(WebhookAdapterConfigError::StaticHeaderMayContainSecret {
                header_name: "Authorization".to_owned(),
            })
        );
    }

    #[test]
    fn webhook_adapter_config_rejects_empty_secret_refs() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            1_000,
        )
        .with_secret_header_ref("X-Signature", " ");

        assert_eq!(
            config.validate(),
            Err(WebhookAdapterConfigError::EmptySecretReference {
                header_name: "X-Signature".to_owned(),
            })
        );
    }

    #[test]
    fn webhook_payload_renders_message_body_without_json_wrapping() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::RenderedMessage,
            1_000,
        );

        let payload =
            render_webhook_payload(&config, &candidate("Price crossed")).expect("payload renders");

        assert_eq!(payload.content_type, "text/plain; charset=utf-8");
        assert_eq!(payload.body, "Price crossed");
    }

    #[test]
    fn webhook_payload_renders_host_versioned_json_envelope() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::JsonEnvelope,
            1_000,
        )
        .with_header("Content-Type", "application/json")
        .with_secret_header_ref("X-Signature", "secret://signature");

        let payload =
            render_webhook_payload(&config, &candidate("Price crossed")).expect("payload renders");
        let value: serde_json::Value =
            serde_json::from_str(&payload.body).expect("payload body is json");

        assert_eq!(payload.content_type, "application/json");
        assert_eq!(
            value,
            serde_json::json!({
                "schemaVersion": 1,
                "adapterId": "webhook-main",
                "bodyMode": "jsonEnvelope",
                "candidate": {
                    "runningAlertId": "alert-1",
                    "scriptSnapshotId": "snapshot-1",
                    "eventKind": "strategyOrderFill",
                    "barIndex": 2,
                    "time": 300,
                    "eventId": "XL",
                    "renderedMessage": "Price crossed",
                },
            })
        );
    }

    #[test]
    fn webhook_payload_does_not_include_url_headers_or_secret_refs() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "https://example.com/hook",
            WebhookBodyMode::JsonEnvelope,
            1_000,
        )
        .with_header("Content-Type", "application/json")
        .with_secret_header_ref("X-Signature", "secret://signature");

        let payload =
            render_webhook_payload(&config, &candidate("Price crossed")).expect("payload renders");

        assert!(!payload.body.contains("example.com"));
        assert!(!payload.body.contains("headers"));
        assert!(!payload.body.contains("secret"));
        assert!(!payload.body.contains("X-Signature"));
    }

    #[test]
    fn webhook_payload_reuses_config_validation() {
        let config = WebhookAdapterConfig::new(
            "webhook-main",
            "file:///tmp/hook",
            WebhookBodyMode::RenderedMessage,
            1_000,
        );

        assert_eq!(
            render_webhook_payload(&config, &candidate("message")),
            Err(WebhookPayloadError::InvalidConfig(
                WebhookAdapterConfigError::UnsupportedUrlScheme
            ))
        );
    }
}
