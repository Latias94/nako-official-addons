use crate::domain::{ResourceLink, ResourceLinkType};

#[must_use]
pub fn resource_link(url: impl Into<String>, source: impl Into<String>) -> Option<ResourceLink> {
    let url = url.into();
    let link_type = classify_resource_url(&url);
    resource_link_with_type(url, link_type, source)
}

#[must_use]
pub fn resource_link_with_type(
    url: impl Into<String>,
    link_type: ResourceLinkType,
    source: impl Into<String>,
) -> Option<ResourceLink> {
    let url = url.into();
    let normalized_url = normalize_resource_url(&url)?;

    Some(ResourceLink {
        url: url.trim().to_owned(),
        normalized_url,
        link_type,
        source: source.into(),
        password: None,
        note: None,
    })
}

#[must_use]
pub fn classify_resource_url(raw_url: &str) -> ResourceLinkType {
    let value = raw_url.trim().to_ascii_lowercase();
    if value.starts_with("magnet:?") {
        return ResourceLinkType::Magnet;
    }
    if value.starts_with("ed2k://") {
        return ResourceLinkType::Ed2k;
    }

    let Some(host) = host_from_url(&value) else {
        return ResourceLinkType::Other;
    };

    if host_matches(&host, "aliyundrive.com") || host_matches(&host, "alipan.com") {
        ResourceLinkType::Aliyun
    } else if host_matches(&host, "pan.baidu.com") || host_matches(&host, "yun.baidu.com") {
        ResourceLinkType::Baidu
    } else if host_matches(&host, "pan.quark.cn") || host_matches(&host, "drive.quark.cn") {
        ResourceLinkType::Quark
    } else if host_matches(&host, "cloud.189.cn") {
        ResourceLinkType::Tianyi
    } else if host_matches(&host, "drive.uc.cn") || host_matches(&host, "uc.cn") {
        ResourceLinkType::Uc
    } else if host_matches(&host, "caiyun.139.com") || host_matches(&host, "yun.139.com") {
        ResourceLinkType::Mobile
    } else if host_matches(&host, "115.com") || host_matches(&host, "115cdn.com") {
        ResourceLinkType::OneOneFive
    } else if host_matches(&host, "mypikpak.com") || host_matches(&host, "pikpakdrive.com") {
        ResourceLinkType::Pikpak
    } else if host_matches(&host, "pan.xunlei.com") || host_matches(&host, "xunlei.com") {
        ResourceLinkType::Xunlei
    } else if host_matches(&host, "123pan.com") || host_matches(&host, "123684.com") {
        ResourceLinkType::OneTwoThree
    } else {
        ResourceLinkType::Web
    }
}

#[must_use]
pub fn normalize_resource_url(raw_url: &str) -> Option<String> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed).trim();
    if without_fragment.is_empty() {
        return None;
    }

    let lower = without_fragment.to_ascii_lowercase();
    if lower.starts_with("magnet:?") || lower.starts_with("ed2k://") {
        return Some(lower);
    }

    let Some(scheme_end) = without_fragment.find("://") else {
        return Some(without_fragment.to_owned());
    };
    let scheme = without_fragment[..scheme_end].to_ascii_lowercase();
    let rest = &without_fragment[scheme_end + 3..];
    let authority_end = rest.find(['/', '?']).unwrap_or(rest.len());
    let authority = rest[..authority_end].to_ascii_lowercase();
    let suffix = &rest[authority_end..];

    Some(format!("{scheme}://{authority}{suffix}"))
}

fn host_from_url(value: &str) -> Option<String> {
    let scheme_end = value.find("://")?;
    let rest = &value[scheme_end + 3..];
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_port.split(':').next().unwrap_or(host_port).trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_owned())
    }
}

fn host_matches(host: &str, suffix: &str) -> bool {
    host == suffix || host.ends_with(&format!(".{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_resource_link_types() {
        let cases = [
            (
                "https://www.aliyundrive.com/s/demo",
                ResourceLinkType::Aliyun,
            ),
            ("https://pan.baidu.com/s/1abc", ResourceLinkType::Baidu),
            ("https://pan.quark.cn/s/demo", ResourceLinkType::Quark),
            ("https://cloud.189.cn/t/demo", ResourceLinkType::Tianyi),
            ("https://drive.uc.cn/s/demo", ResourceLinkType::Uc),
            ("https://caiyun.139.com/m/i/demo", ResourceLinkType::Mobile),
            ("https://115.com/s/demo", ResourceLinkType::OneOneFive),
            ("https://mypikpak.com/s/demo", ResourceLinkType::Pikpak),
            ("https://pan.xunlei.com/s/demo", ResourceLinkType::Xunlei),
            (
                "https://www.123pan.com/s/demo",
                ResourceLinkType::OneTwoThree,
            ),
            ("magnet:?xt=urn:btih:abcdef", ResourceLinkType::Magnet),
            ("ed2k://|file|demo.mkv|1|abc|/", ResourceLinkType::Ed2k),
            ("https://example.com/file", ResourceLinkType::Web),
        ];

        for (url, expected) in cases {
            assert_eq!(classify_resource_url(url), expected, "{url}");
        }
    }

    #[test]
    fn normalizes_host_scheme_and_fragment_for_deduplication() {
        assert_eq!(
            normalize_resource_url(" HTTPS://PAN.QUARK.CN/s/Demo#section ").as_deref(),
            Some("https://pan.quark.cn/s/Demo")
        );
        assert_eq!(
            normalize_resource_url("magnet:?xt=URN:BTIH:ABCDEF").as_deref(),
            Some("magnet:?xt=urn:btih:abcdef")
        );
        assert_eq!(normalize_resource_url("   "), None);
    }

    #[test]
    fn resource_link_classifies_and_normalizes_urls() {
        let link = resource_link(" HTTPS://PAN.QUARK.CN/s/Demo#section ", "fixture").unwrap();

        assert_eq!(link.url, "HTTPS://PAN.QUARK.CN/s/Demo#section");
        assert_eq!(link.normalized_url, "https://pan.quark.cn/s/Demo");
        assert_eq!(link.link_type, ResourceLinkType::Quark);
        assert_eq!(link.source, "fixture");
    }

    #[test]
    fn resource_link_with_type_keeps_explicit_provider_type() {
        let link = resource_link_with_type(
            "https://example.com/file",
            ResourceLinkType::Magnet,
            "provider",
        )
        .unwrap();

        assert_eq!(link.link_type, ResourceLinkType::Magnet);
        assert_eq!(link.normalized_url, "https://example.com/file");
    }
}
