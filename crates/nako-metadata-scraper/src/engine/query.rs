use serde::Deserialize;

use super::title;

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
        let title = title_from_payload(payload);
        let year = year_from_payload(payload);
        let language = language_from_payload(payload, default_language);
        let external_id_descriptors =
            external_id_input_descriptors_from_aliases(external_id_aliases);
        let external_ids = external_ids_from_payload(payload, &external_id_descriptors);

        Self {
            title: if title.is_empty() {
                "Unknown Title".to_owned()
            } else {
                title
            },
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
        let title = title_from_payload(payload);
        let year = year_from_payload(payload);
        let language = language_from_payload(payload, default_language);
        let external_id_descriptors =
            external_id_input_descriptors_from_capabilities(external_id_capabilities);
        let external_ids = external_ids_from_payload(payload, &external_id_descriptors);

        Self {
            title: if title.is_empty() {
                "Unknown Title".to_owned()
            } else {
                title
            },
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

fn normalize_query_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn title_from_payload(payload: &serde_json::Value) -> String {
    first_non_empty_payload_str(payload, &["title", "name", "original_title", "sort_title"])
        .map(normalize_query_title)
        .unwrap_or_else(|| "Unknown Title".to_owned())
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
