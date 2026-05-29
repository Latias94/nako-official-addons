use async_trait::async_trait;

use crate::domain::{
    ResourceLinkCheckRequest, ResourceLinkCheckResponse, ResourceLinkCheckStatus, ResourceLinkType,
};

#[async_trait]
pub trait ResourceLinkCheckProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn check(
        &self,
        request: &ResourceLinkCheckRequest,
    ) -> anyhow::Result<ResourceLinkCheckResponse>;
}

#[derive(Clone, Debug, Default)]
pub struct ConservativeResourceLinkCheckProvider;

#[async_trait]
impl ResourceLinkCheckProvider for ConservativeResourceLinkCheckProvider {
    fn id(&self) -> &'static str {
        "conservative"
    }

    async fn check(
        &self,
        request: &ResourceLinkCheckRequest,
    ) -> anyhow::Result<ResourceLinkCheckResponse> {
        Ok(conservative_check(request, current_time_ms()))
    }
}

fn conservative_check(
    request: &ResourceLinkCheckRequest,
    checked_at_ms: u64,
) -> ResourceLinkCheckResponse {
    let link = &request.link;
    let source_family = source_family(&link.source);
    if source_family == "fixture" && cloud_drive_link_type(link.link_type) {
        return ResourceLinkCheckResponse::new(
            link.link_type,
            ResourceLinkCheckStatus::Reachable,
            checked_at_ms,
        )
        .with_requires_password(link.password.is_some())
        .with_safe_message("fixture_link_reachable")
        .with_safe_fact("checker_provider", "conservative")
        .with_safe_fact("link_family", "fixture")
        .with_safe_fact("live_network", "false");
    }

    match link.link_type {
        ResourceLinkType::Magnet | ResourceLinkType::Ed2k => ResourceLinkCheckResponse::new(
            link.link_type,
            ResourceLinkCheckStatus::Unsupported,
            checked_at_ms,
        )
        .with_safe_message("peer_to_peer_link_check_not_supported")
        .with_safe_fact("checker_provider", "conservative")
        .with_safe_fact("link_family", "peer_to_peer")
        .with_safe_fact("live_network", "false"),
        ResourceLinkType::Aliyun
        | ResourceLinkType::Baidu
        | ResourceLinkType::Quark
        | ResourceLinkType::Tianyi
        | ResourceLinkType::Uc
        | ResourceLinkType::Mobile
        | ResourceLinkType::OneOneFive
        | ResourceLinkType::Pikpak
        | ResourceLinkType::Xunlei
        | ResourceLinkType::OneTwoThree => ResourceLinkCheckResponse::new(
            link.link_type,
            ResourceLinkCheckStatus::Unknown,
            checked_at_ms,
        )
        .with_requires_password(link.password.is_some())
        .with_safe_message("site_specific_checker_not_configured")
        .with_safe_fact("checker_provider", "conservative")
        .with_safe_fact("link_family", "cloud_drive")
        .with_safe_fact("live_network", "false"),
        ResourceLinkType::Web => ResourceLinkCheckResponse::new(
            link.link_type,
            ResourceLinkCheckStatus::Unknown,
            checked_at_ms,
        )
        .with_safe_message("generic_web_link_check_not_enabled")
        .with_safe_fact("checker_provider", "conservative")
        .with_safe_fact("link_family", "web")
        .with_safe_fact("live_network", "false"),
        ResourceLinkType::Other => ResourceLinkCheckResponse::new(
            link.link_type,
            ResourceLinkCheckStatus::Unsupported,
            checked_at_ms,
        )
        .with_safe_message("unsupported_link_type")
        .with_safe_fact("checker_provider", "conservative")
        .with_safe_fact("link_family", "other")
        .with_safe_fact("live_network", "false"),
    }
}

const fn cloud_drive_link_type(link_type: ResourceLinkType) -> bool {
    matches!(
        link_type,
        ResourceLinkType::Aliyun
            | ResourceLinkType::Baidu
            | ResourceLinkType::Quark
            | ResourceLinkType::Tianyi
            | ResourceLinkType::Uc
            | ResourceLinkType::Mobile
            | ResourceLinkType::OneOneFive
            | ResourceLinkType::Pikpak
            | ResourceLinkType::Xunlei
            | ResourceLinkType::OneTwoThree
    )
}

fn source_family(source: &str) -> &str {
    source.split(':').next().unwrap_or(source).trim()
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::domain::ResourceLink;

    use super::*;

    #[test]
    fn conservative_checker_marks_fixture_cloud_links_reachable_without_live_network() {
        let response = conservative_check(
            &ResourceLinkCheckRequest {
                link: resource_link(ResourceLinkType::Quark, "fixture"),
                refresh: false,
            },
            1_779_814_400_000,
        );

        assert_eq!(response.status, ResourceLinkCheckStatus::Reachable);
        assert_eq!(response.checked_at_ms, 1_779_814_400_000);
        assert_eq!(
            response
                .safe_facts
                .get("checker_provider")
                .map(String::as_str),
            Some("conservative")
        );
        assert_eq!(
            response.safe_facts.get("live_network").map(String::as_str),
            Some("false")
        );
    }

    #[test]
    fn conservative_checker_keeps_cloud_links_unknown_without_site_provider() {
        let response = conservative_check(
            &ResourceLinkCheckRequest {
                link: resource_link(ResourceLinkType::Quark, "pansou:movies")
                    .with_password("secret-code"),
                refresh: true,
            },
            1,
        );

        assert_eq!(response.status, ResourceLinkCheckStatus::Unknown);
        assert!(response.requires_password);
        assert_eq!(
            response.safe_message.as_deref(),
            Some("site_specific_checker_not_configured")
        );
    }

    #[test]
    fn conservative_checker_rejects_peer_to_peer_checks() {
        let response = conservative_check(
            &ResourceLinkCheckRequest {
                link: resource_link(ResourceLinkType::Magnet, "pansou:bt"),
                refresh: false,
            },
            1,
        );

        assert_eq!(response.status, ResourceLinkCheckStatus::Unsupported);
        assert_eq!(
            response.safe_facts.get("link_family").map(String::as_str),
            Some("peer_to_peer")
        );
    }

    fn resource_link(link_type: ResourceLinkType, source: &str) -> ResourceLink {
        ResourceLink {
            url: "https://pan.quark.cn/s/demo".to_owned(),
            normalized_url: "https://pan.quark.cn/s/demo".to_owned(),
            link_type,
            source: source.to_owned(),
            password: None,
            note: None,
        }
    }
}
