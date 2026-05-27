use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub receiver_app_id: String,
    pub discovery_timeout_ms: u64,
    pub command_timeout_ms: u64,
    pub live_discovery_enabled: bool,
    pub live_control_enabled: bool,
    pub manual_devices: Vec<ManualChromecastDevice>,
    pub manual_devices_json_valid: bool,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9120";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9120";
    pub const DEFAULT_DISCOVERY_TIMEOUT_MS: u64 = 3_000;
    pub const DEFAULT_COMMAND_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let manual_devices_json =
            lookup("NAKO_CHROMECAST_RENDERER_MANUAL_DEVICES_JSON").and_then(non_empty_trimmed);
        let (manual_devices, manual_devices_json_valid) = parse_manual_devices(manual_devices_json);

        Self {
            listen_addr: lookup("NAKO_CHROMECAST_RENDERER_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_CHROMECAST_RENDERER_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            receiver_app_id: lookup("NAKO_CHROMECAST_RENDERER_RECEIVER_APP_ID")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| {
                    nako_official_addon_catalog::chromecast_renderer::DEFAULT_RECEIVER_APP_ID
                        .to_owned()
                }),
            discovery_timeout_ms: lookup("NAKO_CHROMECAST_RENDERER_DISCOVERY_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .map(|value| value.clamp(250, 30_000))
                .unwrap_or(Self::DEFAULT_DISCOVERY_TIMEOUT_MS),
            command_timeout_ms: lookup("NAKO_CHROMECAST_RENDERER_COMMAND_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .map(|value| value.clamp(500, 60_000))
                .unwrap_or(Self::DEFAULT_COMMAND_TIMEOUT_MS),
            live_discovery_enabled: lookup("NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            live_control_enabled: lookup("NAKO_CHROMECAST_RENDERER_LIVE_CONTROL_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            manual_devices,
            manual_devices_json_valid,
        }
    }

    #[must_use]
    pub fn receiver_app_id_configured(&self) -> bool {
        !self.receiver_app_id.trim().is_empty()
    }

    #[must_use]
    pub fn has_discovery_source(&self) -> bool {
        self.live_discovery_enabled || !self.manual_devices.is_empty()
    }

    #[must_use]
    pub fn manual_device_count(&self) -> usize {
        self.manual_devices.len()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            receiver_app_id:
                nako_official_addon_catalog::chromecast_renderer::DEFAULT_RECEIVER_APP_ID.to_owned(),
            discovery_timeout_ms: Self::DEFAULT_DISCOVERY_TIMEOUT_MS,
            command_timeout_ms: Self::DEFAULT_COMMAND_TIMEOUT_MS,
            live_discovery_enabled: false,
            live_control_enabled: false,
            manual_devices: Vec::new(),
            manual_devices_json_valid: true,
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("listen_addr", &self.listen_addr)
            .field("base_url", &self.base_url)
            .field(
                "receiver_app_id_configured",
                &self.receiver_app_id_configured(),
            )
            .field("discovery_timeout_ms", &self.discovery_timeout_ms)
            .field("command_timeout_ms", &self.command_timeout_ms)
            .field("live_discovery_enabled", &self.live_discovery_enabled)
            .field("live_control_enabled", &self.live_control_enabled)
            .field("manual_device_count", &self.manual_devices.len())
            .field("manual_devices_json_valid", &self.manual_devices_json_valid)
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManualChromecastDevice {
    pub stable_device_id: String,
    pub display_name: String,
    pub host: String,
    #[serde(default = "default_cast_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ManualChromecastDevice {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.stable_device_id.trim().is_empty()
            && !self.display_name.trim().is_empty()
            && !self.host.trim().is_empty()
            && self.port > 0
    }

    #[must_use]
    pub fn safe_model(&self) -> Option<String> {
        self.model
            .as_deref()
            .and_then(|value| non_empty_trimmed(value.to_owned()))
    }
}

impl fmt::Debug for ManualChromecastDevice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManualChromecastDevice")
            .field("stable_device_id", &self.stable_device_id)
            .field("display_name", &self.display_name)
            .field("host_configured", &!self.host.trim().is_empty())
            .field("port", &self.port)
            .field("model_configured", &self.model.is_some())
            .finish()
    }
}

const fn default_cast_port() -> u16 {
    8009
}

fn parse_manual_devices(value: Option<String>) -> (Vec<ManualChromecastDevice>, bool) {
    let Some(value) = value else {
        return (Vec::new(), true);
    };

    match serde_json::from_str::<Vec<ManualChromecastDevice>>(&value) {
        Ok(devices) => {
            let valid_devices = devices
                .into_iter()
                .filter(ManualChromecastDevice::is_valid)
                .collect();
            (valid_devices, true)
        }
        Err(_) => (Vec::new(), false),
    }
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_plan_only_and_redaction_safe() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert_eq!(
            config.receiver_app_id,
            nako_official_addon_catalog::chromecast_renderer::DEFAULT_RECEIVER_APP_ID
        );
        assert!(!config.live_discovery_enabled);
        assert!(!config.live_control_enabled);
        assert!(config.manual_devices.is_empty());

        let debug = format!("{config:?}");
        assert!(!debug.contains("CC1AD845"));
    }

    #[test]
    fn config_reads_manual_devices_without_leaking_host_in_debug() {
        let config = Config::from_env_lookup(|name| {
            match name {
            "NAKO_CHROMECAST_RENDERER_LISTEN_ADDR" => Some(" 0.0.0.0:9120 ".to_owned()),
            "NAKO_CHROMECAST_RENDERER_BASE_URL" => {
                Some(" http://chromecast-renderer.local ".to_owned())
            }
            "NAKO_CHROMECAST_RENDERER_RECEIVER_APP_ID" => Some(" custom-app ".to_owned()),
            "NAKO_CHROMECAST_RENDERER_DISCOVERY_TIMEOUT_MS" => Some("500".to_owned()),
            "NAKO_CHROMECAST_RENDERER_COMMAND_TIMEOUT_MS" => Some("1500".to_owned()),
            "NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED" => Some("yes".to_owned()),
            "NAKO_CHROMECAST_RENDERER_LIVE_CONTROL_ENABLED" => Some("true".to_owned()),
            "NAKO_CHROMECAST_RENDERER_MANUAL_DEVICES_JSON" => Some(
                r#"[{"stable_device_id":"living-room","display_name":"Living Room","host":"192.168.1.50","model":"Chromecast"}]"#
                    .to_owned(),
            ),
            _ => None,
        }
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9120");
        assert_eq!(config.base_url, "http://chromecast-renderer.local");
        assert_eq!(config.receiver_app_id, "custom-app");
        assert_eq!(config.discovery_timeout_ms, 500);
        assert_eq!(config.command_timeout_ms, 1500);
        assert!(config.live_discovery_enabled);
        assert!(config.live_control_enabled);
        assert!(config.manual_devices_json_valid);
        assert_eq!(config.manual_devices.len(), 1);
        assert_eq!(config.manual_devices[0].port, 8009);

        let debug = format!("{config:?} {:?}", config.manual_devices[0]);
        assert!(!debug.contains("192.168.1.50"));
        assert!(!debug.contains("custom-app"));
    }

    #[test]
    fn invalid_manual_devices_json_is_reported_safely() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_CHROMECAST_RENDERER_MANUAL_DEVICES_JSON" => Some("{not-json".to_owned()),
            _ => None,
        });

        assert!(!config.manual_devices_json_valid);
        assert!(config.manual_devices.is_empty());
    }
}
