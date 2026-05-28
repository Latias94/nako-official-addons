use std::collections::HashSet;

use nako_addon_protocol::{
    AddonRendererAdapterCommand, AddonRendererAdapterCommandEnvelope,
    AddonRendererAdapterCommandResult, AddonRendererAdapterCommandState,
    AddonRendererAdapterControlCapabilities, AddonRendererAdapterMediaCapabilities,
    AddonRendererAdapterNetworkScope, AddonRendererAdapterProtocol, AddonRendererAdapterReadiness,
    AddonRendererAdapterReadinessStatus, AddonRendererAdapterTarget,
    AddonRendererAdapterTransportMode, AddonRendererAdapterTransportUrl,
    AddonRendererAdapterTransportUrlKind,
};

use crate::{Config, ManualDlnaDevice};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlnaCommandPlan {
    pub stable_device_id: String,
    pub command: AddonRendererAdapterCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DlnaAdapterError {
    UnsupportedProtocol,
    UnknownTarget,
    MissingTransportUrl,
    MissingContentType,
    UnsafeTransportUrl,
    MissingSeekPosition,
    MissingVolume,
}

impl DlnaAdapterError {
    const fn safe_code(self) -> &'static str {
        match self {
            Self::UnsupportedProtocol => "unsupported_renderer_protocol",
            Self::UnknownTarget => "unknown_target",
            Self::MissingTransportUrl => "missing_transport_url",
            Self::MissingContentType => "missing_content_type",
            Self::UnsafeTransportUrl => "unsafe_transport_url",
            Self::MissingSeekPosition => "missing_seek_position",
            Self::MissingVolume => "missing_volume",
        }
    }
}

#[must_use]
pub fn readiness(config: &Config) -> AddonRendererAdapterReadiness {
    if !config.manual_devices_json_valid {
        return AddonRendererAdapterReadiness {
            protocol: AddonRendererAdapterProtocol::DlnaRenderer,
            status: AddonRendererAdapterReadinessStatus::Degraded,
            reason_code: "manual_devices_json_invalid".to_owned(),
            safe_message: Some("Manual DLNA device configuration is invalid.".to_owned()),
        };
    }

    if config.has_discovery_source() {
        return AddonRendererAdapterReadiness {
            protocol: AddonRendererAdapterProtocol::DlnaRenderer,
            status: AddonRendererAdapterReadinessStatus::Ready,
            reason_code: "manual_targets_configured_plan_only".to_owned(),
            safe_message: Some(
                "DLNA renderer is plan-only; live control is not enabled.".to_owned(),
            ),
        };
    }

    AddonRendererAdapterReadiness {
        protocol: AddonRendererAdapterProtocol::DlnaRenderer,
        status: AddonRendererAdapterReadinessStatus::Degraded,
        reason_code: "no_manual_targets_configured".to_owned(),
        safe_message: Some("Configure manual DLNA renderer targets.".to_owned()),
    }
}

#[must_use]
pub fn discover_targets(config: &Config) -> Vec<AddonRendererAdapterTarget> {
    let targets = config
        .manual_devices
        .iter()
        .map(target_from_manual_device)
        .collect::<Vec<_>>();
    dedupe_targets(targets)
}

#[must_use]
pub fn dispatch_command(
    config: &Config,
    envelope: &AddonRendererAdapterCommandEnvelope,
) -> AddonRendererAdapterCommandResult {
    match build_command_plan(config, envelope) {
        Ok(plan) => AddonRendererAdapterCommandResult {
            stable_device_id: plan.stable_device_id,
            command: plan.command,
            state: AddonRendererAdapterCommandState::Accepted,
            safe_reason_code: Some("plan_only".to_owned()),
        },
        Err(error) => AddonRendererAdapterCommandResult {
            stable_device_id: envelope.stable_device_id.clone(),
            command: envelope.command,
            state: AddonRendererAdapterCommandState::Rejected,
            safe_reason_code: Some(error.safe_code().to_owned()),
        },
    }
}

pub fn build_command_plan(
    config: &Config,
    envelope: &AddonRendererAdapterCommandEnvelope,
) -> Result<DlnaCommandPlan, DlnaAdapterError> {
    if envelope.target_kind != AddonRendererAdapterProtocol::DlnaRenderer {
        return Err(DlnaAdapterError::UnsupportedProtocol);
    }
    if !known_target(config, &envelope.stable_device_id) {
        return Err(DlnaAdapterError::UnknownTarget);
    }

    validate_transport_urls(&envelope.transport.urls)?;
    if envelope.command == AddonRendererAdapterCommand::Play {
        select_play_url(&envelope.transport.urls, envelope.transport.mode)?;
    }

    if envelope.command == AddonRendererAdapterCommand::Seek && envelope.position_ms.is_none() {
        return Err(DlnaAdapterError::MissingSeekPosition);
    }
    if envelope.command == AddonRendererAdapterCommand::SetVolume
        && envelope.volume_percent.is_none()
    {
        return Err(DlnaAdapterError::MissingVolume);
    }

    Ok(DlnaCommandPlan {
        stable_device_id: envelope.stable_device_id.clone(),
        command: envelope.command,
    })
}

fn select_play_url(
    urls: &[AddonRendererAdapterTransportUrl],
    mode: AddonRendererAdapterTransportMode,
) -> Result<&AddonRendererAdapterTransportUrl, DlnaAdapterError> {
    let preferred_kind = match mode {
        AddonRendererAdapterTransportMode::Hls => AddonRendererAdapterTransportUrlKind::Playlist,
        AddonRendererAdapterTransportMode::Direct | AddonRendererAdapterTransportMode::Remux => {
            AddonRendererAdapterTransportUrlKind::Stream
        }
    };

    let selected = urls
        .iter()
        .find(|url| url.kind == preferred_kind)
        .or_else(|| urls.first())
        .ok_or(DlnaAdapterError::MissingTransportUrl)?;

    if selected.content_type.trim().is_empty() {
        return Err(DlnaAdapterError::MissingContentType);
    }

    Ok(selected)
}

fn validate_transport_urls(
    urls: &[AddonRendererAdapterTransportUrl],
) -> Result<(), DlnaAdapterError> {
    for url in urls {
        if !is_dlna_safe_url(&url.url) {
            return Err(DlnaAdapterError::UnsafeTransportUrl);
        }
    }

    Ok(())
}

fn is_dlna_safe_url(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
    else {
        return false;
    };

    if rest.is_empty() || rest.starts_with('/') {
        return false;
    }

    let authority = rest.split('/').next().unwrap_or_default();
    if authority.contains('@') {
        return false;
    }

    !lower.contains("bearer")
        && !lower.contains("nako_at_")
        && !lower.contains("source_locator")
        && !lower.contains("file://")
        && !lower.contains("local://")
}

fn target_from_manual_device(device: &ManualDlnaDevice) -> AddonRendererAdapterTarget {
    AddonRendererAdapterTarget {
        stable_device_id: device.stable_device_id.clone(),
        target_kind: AddonRendererAdapterProtocol::DlnaRenderer,
        display_name: device.display_name.clone(),
        network_scope: AddonRendererAdapterNetworkScope::Local,
        media_capabilities: dlna_media_capabilities(),
        control_capabilities: dlna_control_capabilities(),
        discovered_at_ms: None,
    }
}

fn dlna_media_capabilities() -> AddonRendererAdapterMediaCapabilities {
    AddonRendererAdapterMediaCapabilities {
        direct_play: true,
        containers: vec![
            "mp4".to_owned(),
            "mpegts".to_owned(),
            "mp3".to_owned(),
            "jpeg".to_owned(),
        ],
        video_codecs: vec!["h264".to_owned(), "mpeg2video".to_owned()],
        audio_codecs: vec!["aac".to_owned(), "mp3".to_owned(), "ac3".to_owned()],
    }
}

fn dlna_control_capabilities() -> AddonRendererAdapterControlCapabilities {
    AddonRendererAdapterControlCapabilities {
        set_volume: true,
        ..AddonRendererAdapterControlCapabilities::basic_playback()
    }
}

fn dedupe_targets(targets: Vec<AddonRendererAdapterTarget>) -> Vec<AddonRendererAdapterTarget> {
    let mut seen = HashSet::new();
    targets
        .into_iter()
        .filter(|target| seen.insert(target.stable_device_id.clone()))
        .collect()
}

fn known_target(config: &Config, stable_device_id: &str) -> bool {
    config
        .manual_devices
        .iter()
        .any(|device| device.stable_device_id == stable_device_id)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{
        AddonRendererAdapterTransport, AddonRendererAdapterTransportMode,
        AddonRendererAdapterTransportUrl, AddonRendererAdapterTransportUrlKind,
    };

    use super::*;

    #[test]
    fn readiness_requires_manual_targets_but_stays_plan_only() {
        let empty = Config::default();
        assert_eq!(
            readiness(&empty).status,
            AddonRendererAdapterReadinessStatus::Degraded
        );

        let configured = config_with_manual_device();
        let readiness = readiness(&configured);
        assert_eq!(readiness.status, AddonRendererAdapterReadinessStatus::Ready);
        assert_eq!(readiness.reason_code, "manual_targets_configured_plan_only");
    }

    #[test]
    fn manual_discovery_returns_dlna_target_without_host_details() {
        let targets = discover_targets(&config_with_manual_device());

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].stable_device_id, "living-room");
        assert_eq!(
            targets[0].target_kind,
            AddonRendererAdapterProtocol::DlnaRenderer
        );
        assert_eq!(targets[0].display_name, "Living Room DLNA");
    }

    #[test]
    fn dispatch_command_accepts_known_target_as_plan_only() {
        let result = dispatch_command(&config_with_manual_device(), &play_envelope());

        assert_eq!(result.state, AddonRendererAdapterCommandState::Accepted);
        assert_eq!(result.safe_reason_code.as_deref(), Some("plan_only"));
    }

    #[test]
    fn dispatch_command_rejects_wrong_protocol() {
        let mut envelope = play_envelope();
        envelope.target_kind = AddonRendererAdapterProtocol::Chromecast;

        let result = dispatch_command(&config_with_manual_device(), &envelope);

        assert_eq!(result.state, AddonRendererAdapterCommandState::Rejected);
        assert_eq!(
            result.safe_reason_code.as_deref(),
            Some("unsupported_renderer_protocol")
        );
    }

    #[test]
    fn command_plan_rejects_missing_play_transport_and_content_type() {
        let config = config_with_manual_device();
        let mut missing_url = play_envelope();
        missing_url.transport.urls.clear();
        assert_eq!(
            build_command_plan(&config, &missing_url).unwrap_err(),
            DlnaAdapterError::MissingTransportUrl
        );

        let mut missing_type = play_envelope();
        missing_type.transport.urls[0].content_type.clear();
        assert_eq!(
            build_command_plan(&config, &missing_type).unwrap_err(),
            DlnaAdapterError::MissingContentType
        );
    }

    #[test]
    fn command_plan_rejects_forbidden_transport_facts() {
        let config = config_with_manual_device();
        for unsafe_url in [
            "file:///media/movie.mp4",
            "local://source/movie",
            "https://nako.local/cast?Authorization=Bearer%20secret",
            "https://nako.local/cast?nako_at_secret=1",
            "https://nako.local/cast?source_locator=/media/movie.mp4",
            "https://user:pass@nako.local/cast/movie.mp4",
        ] {
            let mut envelope = play_envelope();
            envelope.transport.urls[0].url = unsafe_url.to_owned();

            assert_eq!(
                build_command_plan(&config, &envelope).unwrap_err(),
                DlnaAdapterError::UnsafeTransportUrl
            );
        }
    }

    fn config_with_manual_device() -> Config {
        Config {
            manual_devices: vec![ManualDlnaDevice {
                stable_device_id: "living-room".to_owned(),
                display_name: "Living Room DLNA".to_owned(),
                host: "192.168.1.20".to_owned(),
                port: 8200,
                model: Some("Generic".to_owned()),
            }],
            ..Config::default()
        }
    }

    fn play_envelope() -> AddonRendererAdapterCommandEnvelope {
        AddonRendererAdapterCommandEnvelope {
            adapter_id: "dlna".to_owned(),
            stable_device_id: "living-room".to_owned(),
            target_kind: AddonRendererAdapterProtocol::DlnaRenderer,
            renderer_session_id: "renderer-session".to_owned(),
            playback_session_id: "playback-session".to_owned(),
            source_id: "source".to_owned(),
            command: AddonRendererAdapterCommand::Play,
            position_ms: None,
            volume_percent: None,
            transport: AddonRendererAdapterTransport {
                mode: AddonRendererAdapterTransportMode::Direct,
                expires_at: "2026-05-28T00:00:00Z".to_owned(),
                urls: vec![AddonRendererAdapterTransportUrl {
                    kind: AddonRendererAdapterTransportUrlKind::Stream,
                    url: "http://127.0.0.1:3000/playback/stream".to_owned(),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                }],
            },
        }
    }
}
