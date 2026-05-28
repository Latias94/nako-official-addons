use async_trait::async_trait;

use crate::domain::{ResourceSearchQuery, ResourceSearchResult};

#[async_trait]
pub trait ResourceSearchProvider: Send + Sync {
    fn id(&self) -> &'static str;

    fn priority(&self) -> u16 {
        100
    }

    async fn search(&self, query: &ResourceSearchQuery) -> anyhow::Result<ProviderSearchBatch>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSearchBatch {
    pub provider_id: String,
    pub results: Vec<ResourceSearchResult>,
    pub warnings: Vec<ProviderSearchWarning>,
    pub finality: ProviderSearchFinality,
}

impl ProviderSearchBatch {
    #[must_use]
    pub fn complete(provider_id: impl Into<String>, results: Vec<ResourceSearchResult>) -> Self {
        Self {
            provider_id: provider_id.into(),
            results,
            warnings: Vec::new(),
            finality: ProviderSearchFinality::Complete,
        }
    }

    #[must_use]
    pub fn partial(provider_id: impl Into<String>, results: Vec<ResourceSearchResult>) -> Self {
        Self {
            provider_id: provider_id.into(),
            results,
            warnings: Vec::new(),
            finality: ProviderSearchFinality::Partial,
        }
    }

    #[must_use]
    pub fn with_warning(mut self, warning: ProviderSearchWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub(crate) fn safe_message(&self) -> Option<String> {
        let mut messages = self
            .warnings
            .iter()
            .map(|warning| warning.safe_message.as_str())
            .collect::<Vec<_>>();
        if self.finality == ProviderSearchFinality::Partial {
            messages.push("partial_results");
        }

        if messages.is_empty() {
            None
        } else {
            Some(messages.join(";"))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSearchWarning {
    safe_message: String,
}

impl ProviderSearchWarning {
    #[must_use]
    pub fn safe(message: impl Into<String>) -> Option<Self> {
        let message = message.into();
        let message = message.trim();
        if message.is_empty() {
            None
        } else {
            Some(Self {
                safe_message: message.to_owned(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderSearchFinality {
    Complete,
    Partial,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_batch_safe_message_reports_warnings_and_partial_finality() {
        let batch = ProviderSearchBatch::partial("fixture", Vec::new()).with_warning(
            ProviderSearchWarning::safe("cache_stale").expect("warning is not empty"),
        );

        assert_eq!(
            batch.safe_message().as_deref(),
            Some("cache_stale;partial_results")
        );
    }
}
