use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLinkType {
    Aliyun,
    Baidu,
    Quark,
    Tianyi,
    Uc,
    Mobile,
    #[serde(rename = "115")]
    OneOneFive,
    Pikpak,
    Xunlei,
    #[serde(rename = "123")]
    OneTwoThree,
    Magnet,
    Ed2k,
    Web,
    Other,
}

impl ResourceLinkType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aliyun => "aliyun",
            Self::Baidu => "baidu",
            Self::Quark => "quark",
            Self::Tianyi => "tianyi",
            Self::Uc => "uc",
            Self::Mobile => "mobile",
            Self::OneOneFive => "115",
            Self::Pikpak => "pikpak",
            Self::Xunlei => "xunlei",
            Self::OneTwoThree => "123",
            Self::Magnet => "magnet",
            Self::Ed2k => "ed2k",
            Self::Web => "web",
            Self::Other => "other",
        }
    }
}

impl FromStr for ResourceLinkType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "aliyun" | "ali" => Ok(Self::Aliyun),
            "baidu" => Ok(Self::Baidu),
            "quark" => Ok(Self::Quark),
            "tianyi" | "189" => Ok(Self::Tianyi),
            "uc" => Ok(Self::Uc),
            "mobile" | "139" => Ok(Self::Mobile),
            "115" => Ok(Self::OneOneFive),
            "pikpak" => Ok(Self::Pikpak),
            "xunlei" => Ok(Self::Xunlei),
            "123" | "123pan" => Ok(Self::OneTwoThree),
            "magnet" => Ok(Self::Magnet),
            "ed2k" => Ok(Self::Ed2k),
            "web" => Ok(Self::Web),
            "other" | "others" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceLink {
    pub url: String,
    pub normalized_url: String,
    pub link_type: ResourceLinkType,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ResourceLink {
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        let password = password.into();
        if !password.trim().is_empty() {
            self.password = Some(password.trim().to_owned());
        }
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        let note = note.into();
        if !note.trim().is_empty() {
            self.note = Some(note.trim().to_owned());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_link_types_keep_wire_names() {
        assert_eq!(
            serde_json::to_value(ResourceLinkType::OneOneFive).unwrap(),
            serde_json::json!("115")
        );
        assert_eq!(
            serde_json::to_value(ResourceLinkType::OneTwoThree).unwrap(),
            serde_json::json!("123")
        );
        assert_eq!(
            serde_json::from_value::<ResourceLinkType>(serde_json::json!("115")).unwrap(),
            ResourceLinkType::OneOneFive
        );
    }
}
