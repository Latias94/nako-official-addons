use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{av, title};

const MIN_METADATA_YEAR: i32 = 1;
const MAX_METADATA_YEAR: i32 = 9999;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataQuery {
    pub title: String,
    pub year: Option<i32>,
    pub language: String,
    pub external_ids: Vec<QueryExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct QueryExternalId {
    pub provider: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct ProviderFieldPolicy {
    preferences: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProviderFieldQualityDescriptor {
    pub official_av: u16,
    pub community_av: u16,
    pub artwork_av: u16,
    pub trailer_av: u16,
}

impl ProviderFieldQualityDescriptor {
    #[must_use]
    pub const fn new(
        official_av: u16,
        community_av: u16,
        artwork_av: u16,
        trailer_av: u16,
    ) -> Self {
        Self {
            official_av,
            community_av,
            artwork_av,
            trailer_av,
        }
    }

    #[must_use]
    pub const fn none() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl ProviderFieldPolicy {
    #[must_use]
    pub fn from_provider_field_quality_descriptors(
        descriptors: impl IntoIterator<Item = (&'static str, ProviderFieldQualityDescriptor)>,
    ) -> Self {
        const OFFICIAL_AV_FIELDS: &[&str] = &[
            "title",
            "original_title",
            "sort_title",
            "overview",
            "release_date",
            "runtime_minutes",
            "tagline",
            "genres",
            "tags",
            "directors",
            "series",
            "studio",
            "publisher",
            "maker",
            "label",
        ];
        const COMMUNITY_AV_FIELDS: &[&str] = &["actors", "all_actors", "wanted_count"];
        const ARTWORK_AV_FIELDS: &[&str] = &[
            "thumb_url",
            "extrafanart_urls",
            "poster",
            "backdrop",
            "artwork",
        ];
        const TRAILER_AV_FIELDS: &[&str] = &["trailer_url"];

        let descriptors = descriptors
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, (&'static str, ProviderFieldQualityDescriptor))>>();
        let official = providers_by_quality(&descriptors, |quality| quality.official_av);
        let community = providers_by_quality(&descriptors, |quality| quality.community_av);
        let artwork = providers_by_quality(&descriptors, |quality| quality.artwork_av);
        let trailer = providers_by_quality(&descriptors, |quality| quality.trailer_av);

        let mut preferences = BTreeMap::new();
        insert_policy_fields(&mut preferences, OFFICIAL_AV_FIELDS, &official);
        insert_policy_fields(&mut preferences, COMMUNITY_AV_FIELDS, &community);
        insert_policy_fields(&mut preferences, ARTWORK_AV_FIELDS, &artwork);
        insert_policy_fields(&mut preferences, TRAILER_AV_FIELDS, &trailer);

        Self { preferences }
    }

    #[must_use]
    pub fn providers_for(&self, field: &str) -> &[String] {
        self.preferences
            .get(&normalize_policy_field(field))
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.preferences.is_empty()
    }

    #[must_use]
    pub fn from_payload_or_default(
        payload: &serde_json::Value,
        default_policy: &ProviderFieldPolicy,
    ) -> Self {
        let Some(policy) = payload.get("provider_field_policy") else {
            return default_policy.clone();
        };
        let Some(values) = policy.as_object() else {
            return default_policy.clone();
        };

        let mut preferences = BTreeMap::<String, Vec<String>>::new();
        for (field, value) in values {
            let field = normalize_policy_field(field);
            if field.is_empty() {
                continue;
            }
            let providers = policy_providers_from_value(value);
            if !providers.is_empty() {
                preferences.insert(field, providers);
            }
        }

        Self { preferences }
    }
}

fn providers_by_quality(
    descriptors: &[(usize, (&'static str, ProviderFieldQualityDescriptor))],
    score: impl Fn(ProviderFieldQualityDescriptor) -> u16,
) -> Vec<String> {
    let mut providers = descriptors
        .iter()
        .filter_map(|(index, (provider, quality))| {
            let score = score(*quality);
            (score > 0).then_some((*index, *provider, score))
        })
        .collect::<Vec<_>>();
    providers.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
    providers
        .into_iter()
        .map(|(_, provider, _)| provider.to_owned())
        .collect()
}

fn insert_policy_fields(
    preferences: &mut BTreeMap<String, Vec<String>>,
    fields: &[&str],
    providers: &[String],
) {
    if providers.is_empty() {
        return;
    }
    for field in fields {
        preferences.insert((*field).to_owned(), providers.to_vec());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryExternalIdAlias {
    pub top_level_field: &'static str,
    pub provider: &'static str,
    pub reject_non_positive_numeric: bool,
}

impl QueryExternalIdAlias {
    #[must_use]
    pub const fn new(
        top_level_field: &'static str,
        provider: &'static str,
        reject_non_positive_numeric: bool,
    ) -> Self {
        Self {
            top_level_field,
            provider,
            reject_non_positive_numeric,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalIdValueKind {
    Numeric,
    Opaque,
    Url,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderExternalIdCapability {
    pub provider: &'static str,
    pub value_kind: ExternalIdValueKind,
    pub accepts_direct_lookup: bool,
    pub emits: bool,
    pub top_level_fields: &'static [&'static str],
    pub reject_non_positive_numeric: bool,
}

impl ProviderExternalIdCapability {
    #[must_use]
    pub const fn new(
        provider: &'static str,
        value_kind: ExternalIdValueKind,
        accepts_direct_lookup: bool,
        emits: bool,
        top_level_fields: &'static [&'static str],
        reject_non_positive_numeric: bool,
    ) -> Self {
        Self {
            provider,
            value_kind,
            accepts_direct_lookup,
            emits,
            top_level_fields,
            reject_non_positive_numeric,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExternalIdInputDescriptor<'a> {
    provider: &'a str,
    top_level_field: Option<&'a str>,
    reject_non_positive_numeric: bool,
}

impl<'a> ExternalIdInputDescriptor<'a> {
    fn from_alias(alias: &'a QueryExternalIdAlias) -> Self {
        Self {
            provider: alias.provider,
            top_level_field: Some(alias.top_level_field),
            reject_non_positive_numeric: alias.reject_non_positive_numeric,
        }
    }

    fn from_capability(
        capability: &'a ProviderExternalIdCapability,
        top_level_field: Option<&'a str>,
    ) -> Self {
        Self {
            provider: capability.provider,
            top_level_field,
            reject_non_positive_numeric: capability.reject_non_positive_numeric,
        }
    }

    fn rejects_value_for(self, provider: &str, value: &str) -> bool {
        self.provider.eq_ignore_ascii_case(provider) && self.rejects_value(value)
    }

    fn rejects_value(self, value: &str) -> bool {
        if !self.reject_non_positive_numeric {
            return false;
        }

        value.parse::<i128>().is_ok_and(|value| value <= 0)
    }
}

impl MetadataQuery {
    #[must_use]
    pub fn from_payload(payload: &serde_json::Value, default_language: &str) -> Self {
        Self::from_payload_with_external_id_aliases(payload, default_language, &[])
    }

    #[must_use]
    pub fn from_payload_with_external_id_aliases(
        payload: &serde_json::Value,
        default_language: &str,
        external_id_aliases: &[QueryExternalIdAlias],
    ) -> Self {
        let av_facts = av::facts_from_payload(payload);
        let title = av::title_for_query(title_from_payload(payload), av_facts.as_ref());
        let year = year_from_payload(payload);
        let language = language_from_payload(payload, default_language);
        let external_id_descriptors =
            external_id_input_descriptors_from_aliases(external_id_aliases);
        let mut external_ids = external_ids_from_payload(payload, &external_id_descriptors);
        av::push_av_external_id(&mut external_ids, av_facts.as_ref());

        Self {
            title,
            year,
            language,
            external_ids,
        }
    }

    #[must_use]
    pub fn from_payload_with_external_id_capabilities(
        payload: &serde_json::Value,
        default_language: &str,
        external_id_capabilities: &[ProviderExternalIdCapability],
    ) -> Self {
        let av_facts = av::facts_from_payload(payload);
        let title = av::title_for_query(title_from_payload(payload), av_facts.as_ref());
        let year = year_from_payload(payload);
        let language = language_from_payload(payload, default_language);
        let external_id_descriptors =
            external_id_input_descriptors_from_capabilities(external_id_capabilities);
        let mut external_ids = external_ids_from_payload(payload, &external_id_descriptors);
        av::push_av_external_id(&mut external_ids, av_facts.as_ref());

        Self {
            title,
            year,
            language,
            external_ids,
        }
    }

    #[must_use]
    pub fn search_title_variants(&self) -> Vec<String> {
        title::search_title_variants(&self.title)
    }
}

fn policy_providers_from_value(value: &serde_json::Value) -> Vec<String> {
    if let Some(value) = value.as_str() {
        return normalize_policy_provider(value).into_iter().collect();
    }

    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .filter_map(normalize_policy_provider)
        .fold(Vec::new(), |mut providers, provider| {
            if !providers.contains(&provider) {
                providers.push(provider);
            }
            providers
        })
}

fn normalize_policy_field(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_policy_provider(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    (!value.is_empty()).then_some(value)
}

fn normalize_query_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn title_from_payload(payload: &serde_json::Value) -> Option<String> {
    first_non_empty_payload_str(payload, &["title", "name", "original_title", "sort_title"])
        .map(normalize_query_title)
}

fn language_from_payload(payload: &serde_json::Value, default_language: &str) -> String {
    payload
        .get("language")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default_language)
        .to_owned()
}

fn year_from_payload(payload: &serde_json::Value) -> Option<i32> {
    ["year", "release_year", "original_year"]
        .iter()
        .find_map(|key| payload.get(*key).and_then(year_from_value))
        .or_else(|| {
            ["release_date", "date", "air_date"]
                .iter()
                .find_map(|key| payload.get(*key).and_then(year_from_date_value))
        })
}

fn year_from_value(value: &serde_json::Value) -> Option<i32> {
    if let Some(year) = value.as_i64() {
        return i32::try_from(year).ok().and_then(valid_metadata_year);
    }

    let year = value.as_str()?.trim();
    if year.is_empty() {
        return None;
    }

    year.parse::<i32>().ok().and_then(valid_metadata_year)
}

fn year_from_date_value(value: &serde_json::Value) -> Option<i32> {
    let date = value.as_str()?.trim();
    let year = date.get(0..4)?;
    if date
        .as_bytes()
        .get(4)
        .is_some_and(|value| value.is_ascii_digit())
    {
        return None;
    }
    if !year.chars().all(|value| value.is_ascii_digit()) {
        return None;
    }

    year.parse::<i32>().ok().and_then(valid_metadata_year)
}

fn valid_metadata_year(year: i32) -> Option<i32> {
    (MIN_METADATA_YEAR..=MAX_METADATA_YEAR)
        .contains(&year)
        .then_some(year)
}

fn first_non_empty_payload_str<'a>(
    payload: &'a serde_json::Value,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        payload
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn external_id_input_descriptors_from_aliases(
    aliases: &[QueryExternalIdAlias],
) -> Vec<ExternalIdInputDescriptor<'_>> {
    aliases
        .iter()
        .map(ExternalIdInputDescriptor::from_alias)
        .collect()
}

fn external_id_input_descriptors_from_capabilities(
    capabilities: &[ProviderExternalIdCapability],
) -> Vec<ExternalIdInputDescriptor<'_>> {
    let mut descriptors = Vec::new();
    for capability in capabilities {
        if capability.top_level_fields.is_empty() {
            descriptors.push(ExternalIdInputDescriptor::from_capability(capability, None));
            continue;
        }

        descriptors.extend(capability.top_level_fields.iter().map(|top_level_field| {
            ExternalIdInputDescriptor::from_capability(capability, Some(*top_level_field))
        }));
    }
    descriptors
}

fn external_ids_from_payload(
    payload: &serde_json::Value,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) -> Vec<QueryExternalId> {
    let mut external_ids = explicit_external_ids_from_payload(payload, descriptors);
    push_top_level_external_ids(&mut external_ids, payload, descriptors);
    external_ids
}

fn explicit_external_ids_from_payload(
    payload: &serde_json::Value,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) -> Vec<QueryExternalId> {
    if let Some(values) = payload
        .get("external_ids")
        .and_then(serde_json::Value::as_object)
    {
        let mut external_ids = Vec::new();
        for (provider, value) in values {
            push_external_ids_from_object_value(&mut external_ids, provider, value, descriptors);
        }
        return external_ids;
    }

    payload
        .get("external_ids")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            query_external_id(
                value.get("provider")?.as_str()?,
                external_id_array_object_value(value)?,
                descriptors,
            )
        })
        .collect()
}

fn push_top_level_external_ids(
    external_ids: &mut Vec<QueryExternalId>,
    payload: &serde_json::Value,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) {
    for descriptor in descriptors {
        let Some(top_level_field) = descriptor.top_level_field else {
            continue;
        };
        if let Some(value) = payload
            .get(top_level_field)
            .and_then(external_id_scalar_value)
            && let Some(external_id) = query_external_id(descriptor.provider, &value, descriptors)
        {
            external_ids.push(external_id);
        }
    }
}

fn external_id_array_object_value(value: &serde_json::Value) -> Option<&str> {
    value
        .get("value")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("external_id"))
        .and_then(serde_json::Value::as_str)
}

fn external_id_scalar_value(value: &serde_json::Value) -> Option<String> {
    if let Some(value) = value.as_str() {
        return Some(value.to_owned());
    }

    value.as_i64().map(|value| value.to_string())
}

fn query_external_id(
    provider: &str,
    value: &str,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) -> Option<QueryExternalId> {
    let provider = provider.trim();
    let value = value.trim();
    if provider.is_empty() || value.is_empty() {
        return None;
    }
    if rejects_external_id_value(provider, value, descriptors) {
        return None;
    }

    Some(QueryExternalId {
        provider: provider.to_owned(),
        value: value.to_owned(),
    })
}

fn rejects_external_id_value(
    provider: &str,
    value: &str,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) -> bool {
    descriptors
        .iter()
        .any(|descriptor| descriptor.rejects_value_for(provider, value))
}

fn push_external_ids_from_object_value(
    external_ids: &mut Vec<QueryExternalId>,
    provider: &str,
    value: &serde_json::Value,
    descriptors: &[ExternalIdInputDescriptor<'_>],
) {
    if let Some(value) = external_id_scalar_value(value) {
        if let Some(external_id) = query_external_id(provider, &value, descriptors) {
            external_ids.push(external_id);
        }
        return;
    }

    if let Some(values) = value.as_array() {
        external_ids.extend(values.iter().filter_map(|value| {
            let parsed_value = external_id_scalar_value(value)?;
            query_external_id(provider, &parsed_value, descriptors)
        }));
    }
}
