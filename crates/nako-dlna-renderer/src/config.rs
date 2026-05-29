use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub manual_devices: Vec<ManualDlnaDevice>,
    pub manual_devices_json_valid: bool,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9150";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9150";

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let manual_devices_json =
            lookup("NAKO_DLNA_RENDERER_MANUAL_DEVICES_JSON").and_then(non_empty_trimmed);
        let (manual_devices, manual_devices_json_valid) = parse_manual_devices(manual_devices_json);

        Self {
            listen_addr: lookup("NAKO_DLNA_RENDERER_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_DLNA_RENDERER_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            manual_devices,
            manual_devices_json_valid,
        }
    }

    #[must_use]
    pub fn has_discovery_source(&self) -> bool {
        !self.manual_devices.is_empty()
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
            .field("manual_device_count", &self.manual_devices.len())
            .field("manual_devices_json_valid", &self.manual_devices_json_valid)
            .field("plan_only", &true)
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManualDlnaDevice {
    pub stable_device_id: String,
    pub display_name: String,
    pub host: String,
    #[serde(default = "default_dlna_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ManualDlnaDevice {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.stable_device_id.trim().is_empty()
            && !self.display_name.trim().is_empty()
            && !self.host.trim().is_empty()
            && self.port > 0
    }
}

impl fmt::Debug for ManualDlnaDevice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManualDlnaDevice")
            .field("stable_device_id", &self.stable_device_id)
            .field("display_name", &self.display_name)
            .field("host_configured", &!self.host.trim().is_empty())
            .field("port", &self.port)
            .field("model_configured", &self.model.is_some())
            .finish()
    }
}

const fn default_dlna_port() -> u16 {
    8200
}

fn parse_manual_devices(value: Option<String>) -> (Vec<ManualDlnaDevice>, bool) {
    let Some(value) = value else {
        return (Vec::new(), true);
    };

    match serde_json::from_str::<Vec<ManualDlnaDevice>>(&value) {
        Ok(devices) => {
            let valid_devices = devices
                .into_iter()
                .filter(ManualDlnaDevice::is_valid)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_plan_only_without_targets() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert!(config.manual_devices.is_empty());
        assert!(!config.has_discovery_source());
        assert!(config.manual_devices_json_valid);
    }

    #[test]
    fn config_reads_manual_devices_without_leaking_hosts_in_debug() {
        let manual_devices = serde_json::json!([{
            "stable_device_id": "living-room",
            "display_name": "Living Room DLNA",
            "host": "192.168.1.20",
            "port": 8200,
            "model": "Generic"
        }]);
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_DLNA_RENDERER_LISTEN_ADDR" => Some(" 0.0.0.0:9150 ".to_owned()),
            "NAKO_DLNA_RENDERER_BASE_URL" => Some(" http://dlna.local ".to_owned()),
            "NAKO_DLNA_RENDERER_MANUAL_DEVICES_JSON" => Some(manual_devices.to_string()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9150");
        assert_eq!(config.base_url, "http://dlna.local");
        assert_eq!(config.manual_device_count(), 1);
        assert!(config.manual_devices_json_valid);
        let debug = format!("{config:?}");
        assert!(!debug.contains("192.168.1.20"));
    }
}
