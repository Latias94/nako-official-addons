use std::fmt;

use nako_addon_protocol::AddonEventRequest;

pub const DEFAULT_SUMMARY_TEMPLATE: &str =
    "Nako {{event_kind}} event for {{subject_kind}} {{subject_id}}";

const ALLOWED_TOKENS: &[&str] = &[
    "event_id",
    "event_kind",
    "subject_kind",
    "subject_id",
    "occurred_at",
    "attempt",
    "payload_keys",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateStatus {
    Valid,
    Invalid,
}

impl TemplateStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateError {
    UnclosedToken,
    EmptyToken,
    UnknownToken,
}

impl TemplateError {
    #[must_use]
    pub const fn safe_code(self) -> &'static str {
        match self {
            Self::UnclosedToken => "unclosed_template_token",
            Self::EmptyToken => "empty_template_token",
            Self::UnknownToken => "unknown_template_token",
        }
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_code())
    }
}

impl std::error::Error for TemplateError {}

pub struct TemplateContext<'a> {
    pub request: &'a AddonEventRequest,
    pub payload_keys: &'a [String],
}

pub fn validate_template(template: &str) -> Result<(), TemplateError> {
    render_segments(template, token_value_exists)?;
    Ok(())
}

#[must_use]
pub fn template_status(template: &str) -> TemplateStatus {
    if validate_template(template).is_ok() {
        TemplateStatus::Valid
    } else {
        TemplateStatus::Invalid
    }
}

pub fn render_template(
    template: &str,
    context: &TemplateContext<'_>,
) -> Result<String, TemplateError> {
    let mut output = String::new();

    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let token_rest = &rest[start + 2..];
        let Some(end) = token_rest.find("}}") else {
            return Err(TemplateError::UnclosedToken);
        };
        let token = token_rest[..end].trim();
        if token.is_empty() {
            return Err(TemplateError::EmptyToken);
        }
        output.push_str(&token_value(token, context)?);
        rest = &token_rest[end + 2..];
    }

    if rest.contains("}}") {
        return Err(TemplateError::UnclosedToken);
    }

    output.push_str(rest);
    Ok(output)
}

fn render_segments(
    template: &str,
    mut on_token: impl FnMut(&str) -> Result<(), TemplateError>,
) -> Result<(), TemplateError> {
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let token_rest = &rest[start + 2..];
        let Some(end) = token_rest.find("}}") else {
            return Err(TemplateError::UnclosedToken);
        };
        let token = token_rest[..end].trim();
        if token.is_empty() {
            return Err(TemplateError::EmptyToken);
        }
        on_token(token)?;
        rest = &token_rest[end + 2..];
    }

    if rest.contains("}}") {
        return Err(TemplateError::UnclosedToken);
    }

    Ok(())
}

fn token_value_exists(token: &str) -> Result<(), TemplateError> {
    if ALLOWED_TOKENS.contains(&token) {
        Ok(())
    } else {
        Err(TemplateError::UnknownToken)
    }
}

fn token_value(token: &str, context: &TemplateContext<'_>) -> Result<String, TemplateError> {
    let value = match token {
        "event_id" => context.request.event_id.clone(),
        "event_kind" => context.request.event_kind.clone(),
        "subject_kind" => context.request.subject_kind.clone(),
        "subject_id" => context.request.subject_id.clone(),
        "occurred_at" => context.request.occurred_at.clone(),
        "attempt" => context.request.attempt.to_string(),
        "payload_keys" => {
            if context.payload_keys.is_empty() {
                "none".to_owned()
            } else {
                context.payload_keys.join(", ")
            }
        }
        _ => return Err(TemplateError::UnknownToken),
    };
    Ok(value)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::ADDON_PROTOCOL_VERSION;

    use super::*;

    fn request() -> AddonEventRequest {
        AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: "nako.official.notification-bridge".to_owned(),
            subscription_id: "library-scanned-notification".to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: "library.scanned".to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 2,
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo",
                "source_id": "source-1"
            }),
        }
    }

    #[test]
    fn renders_allowed_tokens_without_raw_payload_values() {
        let request = request();
        let payload_keys = vec!["secret".to_owned(), "source_id".to_owned()];
        let context = TemplateContext {
            request: &request,
            payload_keys: &payload_keys,
        };

        let rendered = render_template(
            "{{event_kind}} {{subject_kind}} {{subject_id}} keys={{payload_keys}} attempt={{attempt}}",
            &context,
        )
        .unwrap();

        assert_eq!(
            rendered,
            "library.scanned library library-1 keys=secret, source_id attempt=2"
        );
        assert!(!rendered.contains("nako_at_should_not_echo"));
        assert!(!rendered.contains("source-1"));
    }

    #[test]
    fn rejects_unknown_or_malformed_tokens() {
        assert_eq!(
            validate_template("{{payload.source_id}}").unwrap_err(),
            TemplateError::UnknownToken
        );
        assert_eq!(
            validate_template("{{event_kind").unwrap_err(),
            TemplateError::UnclosedToken
        );
        assert_eq!(
            validate_template("{{ }}").unwrap_err(),
            TemplateError::EmptyToken
        );
    }
}
