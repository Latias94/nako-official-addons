use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataQuery {
    pub title: String,
    pub year: Option<i32>,
    pub language: String,
}

impl MetadataQuery {
    #[must_use]
    pub fn from_payload(payload: &serde_json::Value, default_language: &str) -> Self {
        let title = payload
            .get("title")
            .and_then(serde_json::Value::as_str)
            .or_else(|| payload.get("name").and_then(serde_json::Value::as_str))
            .unwrap_or("Unknown Title")
            .trim()
            .to_owned();
        let year = payload
            .get("year")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok());
        let language = payload
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_language)
            .to_owned();

        Self {
            title: if title.is_empty() {
                "Unknown Title".to_owned()
            } else {
                title
            },
            year,
            language,
        }
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct MetadataCandidate {
    pub provider: String,
    pub provider_id: String,
    pub confidence_milli: u16,
    pub patch: nako_addon_protocol::AddonMetadataPatch,
    pub evidence: CandidateEvidence,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct CandidateEvidence {
    pub matched_title: bool,
    pub matched_year: bool,
    pub note: String,
}
