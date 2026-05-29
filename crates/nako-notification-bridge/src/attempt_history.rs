use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use nako_addon_protocol::AddonEventRequest;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct ProviderAttemptHistory {
    inner: Arc<Mutex<VecDeque<ProviderAttemptRecord>>>,
    capacity: usize,
}

impl ProviderAttemptHistory {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn record(&self, record: ProviderAttemptRecord) {
        if self.capacity == 0 {
            return;
        }

        let mut records = self.inner.lock().unwrap();
        while records.len() >= self.capacity {
            records.pop_front();
        }
        records.push_back(record);
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<ProviderAttemptRecord> {
        self.inner.lock().unwrap().iter().cloned().collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAttemptRecord {
    pub provider_id: &'static str,
    pub event_id: String,
    pub event_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub addon_attempt: u32,
    pub provider_status: &'static str,
    pub retryable: bool,
    pub provider_http_status: Option<u16>,
    pub recorded_at_unix_ms: u128,
}

impl ProviderAttemptRecord {
    #[must_use]
    pub fn new(
        provider_id: &'static str,
        request: &AddonEventRequest,
        provider_status: &'static str,
        retryable: bool,
        provider_http_status: Option<u16>,
    ) -> Self {
        Self {
            provider_id,
            event_id: request.event_id.clone(),
            event_kind: request.event_kind.clone(),
            subject_kind: request.subject_kind.clone(),
            subject_id: request.subject_id.clone(),
            addon_attempt: request.attempt,
            provider_status,
            retryable,
            provider_http_status,
            recorded_at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_millis()),
        }
    }
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::ADDON_PROTOCOL_VERSION;

    use super::*;

    fn request(event_id: &str) -> AddonEventRequest {
        AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: "nako.official.notification-bridge".to_owned(),
            subscription_id: "library-scanned-notification".to_owned(),
            event_id: event_id.to_owned(),
            event_kind: "library.scanned".to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo",
                "source_id": "source-1"
            }),
        }
    }

    #[test]
    fn provider_attempt_history_is_bounded_and_redaction_safe() {
        let history = ProviderAttemptHistory::new(2);
        history.record(ProviderAttemptRecord::new(
            "http_webhook",
            &request("event-1"),
            "disabled",
            false,
            None,
        ));
        history.record(ProviderAttemptRecord::new(
            "discord_webhook",
            &request("event-2"),
            "sent",
            false,
            Some(202),
        ));
        history.record(ProviderAttemptRecord::new(
            "discord_webhook",
            &request("event-3"),
            "retryable_failure",
            true,
            Some(429),
        ));

        let snapshot = history.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].event_id, "event-2");
        assert_eq!(snapshot[1].event_id, "event-3");

        let text = serde_json::to_string(&snapshot).unwrap();
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("source-1"));
    }
}
