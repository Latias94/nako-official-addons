use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePolicy {
    Official,
    ExternalService,
    ThirdParty,
}

impl SourcePolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::ExternalService => "external_service",
            Self::ThirdParty => "third_party",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_policy_names_are_stable() {
        assert_eq!(SourcePolicy::Official.as_str(), "official");
        assert_eq!(SourcePolicy::ExternalService.as_str(), "external_service");
        assert_eq!(SourcePolicy::ThirdParty.as_str(), "third_party");
    }
}
