use serde::{Deserialize, Serialize};

use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

use super::{BANGUMI_PROVIDER_ID, mapper::BangumiSubjectCandidate};

#[derive(Debug, Serialize)]
pub(super) struct BangumiSubjectSearchRequest {
    pub(super) keyword: String,
    pub(super) sort: &'static str,
    pub(super) filter: BangumiSubjectSearchFilter,
}

#[derive(Debug, Serialize)]
pub(super) struct BangumiSubjectSearchFilter {
    #[serde(rename = "type")]
    pub(super) subject_type: Vec<u8>,
    pub(super) nsfw: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) air_date: Option<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BangumiSubjectSearchResponse {
    #[serde(default)]
    pub(super) data: Vec<BangumiSubject>,
}

impl BangumiSubjectSearchResponse {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let items = value
            .get("data")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse Bangumi subject search response: missing field `data`"
                )
            })?
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse Bangumi subject search response: `data` must be an array"
                )
            })?;
        let mut skipped_count = 0usize;
        let data = items
            .iter()
            .filter_map(|item| match serde_json::from_value::<BangumiSubject>(item.clone()) {
                Ok(subject) if subject.id > 0 => Some(subject),
                Ok(_) => {
                    skipped_count += 1;
                    tracing::warn!(
                        provider = BANGUMI_PROVIDER_ID,
                        "skipping Bangumi search subject item with zero id"
                    );
                    None
                }
                Err(error) => {
                    skipped_count += 1;
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "skipping malformed Bangumi search subject item");
                    None
                }
            })
            .collect();
        if !items.is_empty() && skipped_count == items.len() {
            anyhow::bail!("all Bangumi search subject items were malformed");
        }
        Ok(Self { data })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct BangumiSubject {
    pub(super) id: u64,
    #[serde(rename = "type")]
    pub(super) subject_type: Option<u8>,
    pub(super) name: Option<String>,
    pub(super) name_cn: Option<String>,
    #[serde(alias = "air_date")]
    pub(super) date: Option<String>,
    #[serde(alias = "short_summary")]
    pub(super) summary: Option<String>,
    pub(super) platform: Option<String>,
    pub(super) images: Option<BangumiImages>,
    pub(super) nsfw: Option<bool>,
    pub(super) locked: Option<bool>,
    pub(super) series: Option<bool>,
    pub(super) volumes: Option<u32>,
    pub(super) eps: Option<u32>,
    pub(super) total_episodes: Option<u32>,
    pub(super) air_weekday: Option<u8>,
    pub(super) rank: Option<u32>,
    pub(super) score: Option<f64>,
    pub(super) rating: Option<BangumiRating>,
    pub(super) collection_total: Option<u32>,
    pub(super) collection: Option<BangumiSubjectCollection>,
    #[serde(default)]
    pub(super) infobox: Vec<BangumiInfoboxItem>,
    #[serde(default)]
    pub(super) meta_tags: Vec<String>,
    #[serde(default)]
    pub(super) tags: Vec<BangumiTag>,
}

impl BangumiSubject {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse Bangumi subject response: {error}"))
    }

    pub(super) fn into_degraded_candidate(
        self,
        query: &MetadataQuery,
    ) -> ProviderMetadataCandidate {
        BangumiSubjectCandidate {
            search: self.clone(),
            detail: self,
            degraded: true,
        }
        .into_candidate(query)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct BangumiImages {
    pub(super) large: Option<String>,
    pub(super) common: Option<String>,
    pub(super) medium: Option<String>,
    pub(super) small: Option<String>,
    pub(super) grid: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct BangumiRating {
    pub(super) rank: Option<u32>,
    pub(super) total: Option<u32>,
    pub(super) score: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct BangumiSubjectCollection {
    pub(super) wish: Option<u32>,
    pub(super) collect: Option<u32>,
    pub(super) doing: Option<u32>,
    pub(super) on_hold: Option<u32>,
    pub(super) dropped: Option<u32>,
}

impl BangumiSubjectCollection {
    pub(super) fn total(&self) -> Option<u32> {
        let total = self
            .wish
            .unwrap_or_default()
            .saturating_add(self.collect.unwrap_or_default())
            .saturating_add(self.doing.unwrap_or_default())
            .saturating_add(self.on_hold.unwrap_or_default())
            .saturating_add(self.dropped.unwrap_or_default());
        (total > 0).then_some(total)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct BangumiTag {
    pub(super) name: Option<String>,
    pub(super) count: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct BangumiInfoboxItem {
    pub(super) key: Option<String>,
    #[serde(default)]
    pub(super) value: serde_json::Value,
}
