use std::collections::BTreeMap;

use super::{ResourceLink, ResourceLinkType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceLinkCheckRequest {
    pub link: ResourceLink,
    pub refresh: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceLinkCheckResponse {
    pub link_type: ResourceLinkType,
    pub status: ResourceLinkCheckStatus,
    pub checked_at_ms: u64,
    pub requires_password: bool,
    pub retryable: bool,
    pub retry_after_ms: Option<u64>,
    pub safe_message: Option<String>,
    pub safe_facts: BTreeMap<String, String>,
}

impl ResourceLinkCheckResponse {
    #[must_use]
    pub fn new(
        link_type: ResourceLinkType,
        status: ResourceLinkCheckStatus,
        checked_at_ms: u64,
    ) -> Self {
        Self {
            link_type,
            status,
            checked_at_ms,
            requires_password: false,
            retryable: false,
            retry_after_ms: None,
            safe_message: None,
            safe_facts: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_requires_password(mut self, requires_password: bool) -> Self {
        self.requires_password = requires_password;
        self
    }

    #[must_use]
    pub fn with_safe_message(mut self, safe_message: impl Into<String>) -> Self {
        let safe_message = safe_message.into();
        let safe_message = safe_message.trim();
        if !safe_message.is_empty() {
            self.safe_message = Some(safe_message.to_owned());
        }
        self
    }

    #[must_use]
    pub fn with_safe_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();
        let key = key.trim();
        let value = value.trim();
        if !key.is_empty() && !value.is_empty() {
            self.safe_facts.insert(key.to_owned(), value.to_owned());
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceLinkCheckStatus {
    Reachable,
    Unavailable,
    PasswordNeeded,
    Unsupported,
    RateLimited,
    Error,
    Unknown,
}

impl ResourceLinkCheckStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::Unavailable => "unavailable",
            Self::PasswordNeeded => "password_needed",
            Self::Unsupported => "unsupported",
            Self::RateLimited => "rate_limited",
            Self::Error => "error",
            Self::Unknown => "unknown",
        }
    }
}
