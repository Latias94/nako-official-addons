use std::{collections::HashSet, fmt, time::Duration};

use nako_addon_protocol::{
    AddonRendererAdapterCommand, AddonRendererAdapterCommandEnvelope,
    AddonRendererAdapterCommandResult, AddonRendererAdapterCommandState,
    AddonRendererAdapterControlCapabilities, AddonRendererAdapterMediaCapabilities,
    AddonRendererAdapterNetworkScope, AddonRendererAdapterProtocol, AddonRendererAdapterReadiness,
    AddonRendererAdapterReadinessStatus, AddonRendererAdapterTarget,
    AddonRendererAdapterTransportMode, AddonRendererAdapterTransportUrl,
    AddonRendererAdapterTransportUrlKind,
};
use oxicast::{CastApp, CastClient, DeviceInfo, MediaInfo, StreamType};
use thiserror::Error;

use crate::{Config, ManualChromecastDevice, manifest::DEFAULT_RECEIVER_APP_ID};

#[derive(Clone, Eq, PartialEq)]
pub struct ChromecastCommandPlan {
    pub stable_device_id: String,
    pub receiver_app_id: String,
    pub command: AddonRendererAdapterCommand,
    pub transport_mode: AddonRendererAdapterTransportMode,
    pub media_url: Option<String>,
    pub content_type: Option<String>,
    pub supports_range_requests: Option<bool>,
    pub start_time_seconds: Option<u64>,
    pub volume_percent: Option<u8>,
}

impl fmt::Debug for ChromecastCommandPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChromecastCommandPlan")
            .field("stable_device_id", &self.stable_device_id)
            .field(
                "receiver_app_id_configured",
                &!self.receiver_app_id.is_empty(),
            )
            .field("command", &self.command)
            .field("transport_mode", &self.transport_mode)
            .field("media_url_configured", &self.media_url.is_some())
            .field("content_type", &self.content_type)
            .field("supports_range_requests", &self.supports_range_requests)
            .field("start_time_seconds", &self.start_time_seconds)
            .field("volume_percent", &self.volume_percent)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum ChromecastAdapterError {
    #[error("unsupported renderer adapter protocol")]
    UnsupportedProtocol,
    #[error("receiver app id is not configured")]
    ReceiverAppIdMissing,
    #[error("target is unknown")]
    UnknownTarget,
    #[error("play command requires a transport url")]
    MissingTransportUrl,
    #[error("play command requires a non-empty content type")]
    MissingContentType,
    #[error("transport url is not cast safe")]
    UnsafeTransportUrl,
    #[error("seek command requires a position")]
    MissingSeekPosition,
    #[error("set_volume command requires a volume percent")]
    MissingVolume,
    #[error("cast control failed")]
    CastControlFailed,
}

impl ChromecastAdapterError {
    #[must_use]
    pub const fn safe_code(&self) -> &'static str {
        match self {
            Self::UnsupportedProtocol => "unsupported_protocol",
            Self::ReceiverAppIdMissing => "receiver_app_id_missing",
            Self::UnknownTarget => "unknown_target",
            Self::MissingTransportUrl => "missing_transport_url",
            Self::MissingContentType => "missing_content_type",
            Self::UnsafeTransportUrl => "unsafe_transport_url",
            Self::MissingSeekPosition => "missing_seek_position",
            Self::MissingVolume => "missing_volume",
            Self::CastControlFailed => "cast_control_failed",
        }
    }
}

#[must_use]
pub fn readiness(config: &Config) -> AddonRendererAdapterReadiness {
    if !config.receiver_app_id_configured() {
        return AddonRendererAdapterReadiness {
            protocol: AddonRendererAdapterProtocol::Chromecast,
            status: AddonRendererAdapterReadinessStatus::ConfigurationRequired,
            reason_code: "receiver_app_id_missing".to_owned(),
            safe_message: Some("Chromecast receiver app id is required.".to_owned()),
        };
    }

    if !config.manual_devices_json_valid {
        return AddonRendererAdapterReadiness {
            protocol: AddonRendererAdapterProtocol::Chromecast,
            status: AddonRendererAdapterReadinessStatus::Degraded,
            reason_code: "manual_devices_json_invalid".to_owned(),
            safe_message: Some("Manual Chromecast device configuration is invalid.".to_owned()),
        };
    }

    if config.has_discovery_source() {
        return AddonRendererAdapterReadiness {
            protocol: AddonRendererAdapterProtocol::Chromecast,
            status: AddonRendererAdapterReadinessStatus::Ready,
            reason_code: "target_source_configured".to_owned(),
            safe_message: None,
        };
    }

    AddonRendererAdapterReadiness {
        protocol: AddonRendererAdapterProtocol::Chromecast,
        status: AddonRendererAdapterReadinessStatus::Degraded,
        reason_code: "no_target_source_configured".to_owned(),
        safe_message: Some(
            "Configure manual Chromecast devices or enable live LAN discovery.".to_owned(),
        ),
    }
}

pub async fn discover_targets(
    config: &Config,
    timeout_ms: Option<u64>,
) -> Vec<AddonRendererAdapterTarget> {
    let mut targets = config
        .manual_devices
        .iter()
        .map(target_from_manual_device)
        .collect::<Vec<_>>();

    if config.live_discovery_enabled {
        let timeout = Duration::from_millis(
            timeout_ms
                .unwrap_or(config.discovery_timeout_ms)
                .clamp(250, 30_000),
        );
        match oxicast::discovery::discover_devices(timeout).await {
            Ok(devices) => {
                targets.extend(devices.iter().map(target_from_discovered_device));
            }
            Err(error) => {
                tracing::warn!(safe_error = %error, "Chromecast live discovery failed");
            }
        }
    }

    dedupe_targets(targets)
}

pub async fn dispatch_command(
    config: &Config,
    envelope: &AddonRendererAdapterCommandEnvelope,
) -> AddonRendererAdapterCommandResult {
    match build_command_plan(config, envelope) {
        Ok(plan) => {
            if !config.live_control_enabled {
                return accepted_result(envelope, "plan_only");
            }

            match execute_live_command(config, &plan).await {
                Ok(()) => accepted_result(envelope, "sent_to_chromecast"),
                Err(error) => AddonRendererAdapterCommandResult {
                    stable_device_id: envelope.stable_device_id.clone(),
                    command: envelope.command,
                    state: AddonRendererAdapterCommandState::Failed,
                    safe_reason_code: Some(error.safe_code().to_owned()),
                },
            }
        }
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
) -> Result<ChromecastCommandPlan, ChromecastAdapterError> {
    if envelope.target_kind != AddonRendererAdapterProtocol::Chromecast {
        return Err(ChromecastAdapterError::UnsupportedProtocol);
    }
    if !config.receiver_app_id_configured() {
        return Err(ChromecastAdapterError::ReceiverAppIdMissing);
    }
    if !known_target(config, &envelope.stable_device_id) && !config.live_discovery_enabled {
        return Err(ChromecastAdapterError::UnknownTarget);
    }

    validate_transport_urls(&envelope.transport.urls)?;
    let selected_url = if envelope.command == AddonRendererAdapterCommand::Play {
        Some(select_play_url(
            &envelope.transport.urls,
            envelope.transport.mode,
        )?)
    } else {
        None
    };

    if envelope.command == AddonRendererAdapterCommand::Seek && envelope.position_ms.is_none() {
        return Err(ChromecastAdapterError::MissingSeekPosition);
    }
    if envelope.command == AddonRendererAdapterCommand::SetVolume
        && envelope.volume_percent.is_none()
    {
        return Err(ChromecastAdapterError::MissingVolume);
    }

    Ok(ChromecastCommandPlan {
        stable_device_id: envelope.stable_device_id.clone(),
        receiver_app_id: config.receiver_app_id.clone(),
        command: envelope.command,
        transport_mode: envelope.transport.mode,
        media_url: selected_url.map(|url| url.url.clone()),
        content_type: selected_url.map(|url| url.content_type.clone()),
        supports_range_requests: selected_url.map(|url| url.supports_range_requests),
        start_time_seconds: envelope.position_ms.map(|value| value / 1_000),
        volume_percent: envelope.volume_percent,
    })
}

fn select_play_url(
    urls: &[AddonRendererAdapterTransportUrl],
    mode: AddonRendererAdapterTransportMode,
) -> Result<&AddonRendererAdapterTransportUrl, ChromecastAdapterError> {
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
        .ok_or(ChromecastAdapterError::MissingTransportUrl)?;

    if selected.content_type.trim().is_empty() {
        return Err(ChromecastAdapterError::MissingContentType);
    }

    Ok(selected)
}

fn validate_transport_urls(
    urls: &[AddonRendererAdapterTransportUrl],
) -> Result<(), ChromecastAdapterError> {
    for url in urls {
        if !is_cast_safe_url(&url.url) {
            return Err(ChromecastAdapterError::UnsafeTransportUrl);
        }
    }

    Ok(())
}

fn is_cast_safe_url(value: &str) -> bool {
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

fn target_from_manual_device(device: &ManualChromecastDevice) -> AddonRendererAdapterTarget {
    AddonRendererAdapterTarget {
        stable_device_id: device.stable_device_id.clone(),
        target_kind: AddonRendererAdapterProtocol::Chromecast,
        display_name: device.display_name.clone(),
        network_scope: AddonRendererAdapterNetworkScope::Local,
        media_capabilities: chromecast_media_capabilities(),
        control_capabilities: chromecast_control_capabilities(),
        discovered_at_ms: None,
    }
}

fn target_from_discovered_device(device: &DeviceInfo) -> AddonRendererAdapterTarget {
    AddonRendererAdapterTarget {
        stable_device_id: discovered_device_stable_id(device),
        target_kind: AddonRendererAdapterProtocol::Chromecast,
        display_name: device.name.clone(),
        network_scope: AddonRendererAdapterNetworkScope::Local,
        media_capabilities: chromecast_media_capabilities(),
        control_capabilities: chromecast_control_capabilities(),
        discovered_at_ms: None,
    }
}

fn chromecast_media_capabilities() -> AddonRendererAdapterMediaCapabilities {
    AddonRendererAdapterMediaCapabilities {
        direct_play: true,
        containers: vec!["mp4".to_owned(), "webm".to_owned(), "hls".to_owned()],
        video_codecs: vec!["h264".to_owned(), "vp8".to_owned(), "vp9".to_owned()],
        audio_codecs: vec![
            "aac".to_owned(),
            "mp3".to_owned(),
            "opus".to_owned(),
            "vorbis".to_owned(),
        ],
    }
}

fn chromecast_control_capabilities() -> AddonRendererAdapterControlCapabilities {
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

async fn execute_live_command(
    config: &Config,
    plan: &ChromecastCommandPlan,
) -> Result<(), ChromecastAdapterError> {
    let endpoint = resolve_target_endpoint(config, &plan.stable_device_id).await?;
    let client = CastClient::connect(&endpoint.host, endpoint.port)
        .await
        .map_err(|_| ChromecastAdapterError::CastControlFailed)?;

    let result = match plan.command {
        AddonRendererAdapterCommand::Play => execute_play(config, &client, plan).await,
        AddonRendererAdapterCommand::Pause => client
            .pause()
            .await
            .map(|_| ())
            .map_err(|_| ChromecastAdapterError::CastControlFailed),
        AddonRendererAdapterCommand::Resume => client
            .play()
            .await
            .map(|_| ())
            .map_err(|_| ChromecastAdapterError::CastControlFailed),
        AddonRendererAdapterCommand::Seek => {
            let position = plan
                .start_time_seconds
                .ok_or(ChromecastAdapterError::MissingSeekPosition)?;
            client
                .seek(position as f64)
                .await
                .map(|_| ())
                .map_err(|_| ChromecastAdapterError::CastControlFailed)
        }
        AddonRendererAdapterCommand::Stop => client
            .stop_media()
            .await
            .map(|_| ())
            .map_err(|_| ChromecastAdapterError::CastControlFailed),
        AddonRendererAdapterCommand::SetVolume => {
            let volume = plan
                .volume_percent
                .ok_or(ChromecastAdapterError::MissingVolume)?;
            client
                .set_volume((volume as f32 / 100.0).clamp(0.0, 1.0))
                .await
                .map(|_| ())
                .map_err(|_| ChromecastAdapterError::CastControlFailed)
        }
    };

    let _ = client.disconnect().await;
    result
}

async fn execute_play(
    config: &Config,
    client: &CastClient,
    plan: &ChromecastCommandPlan,
) -> Result<(), ChromecastAdapterError> {
    let media_url = plan
        .media_url
        .as_deref()
        .ok_or(ChromecastAdapterError::MissingTransportUrl)?;
    let content_type = plan
        .content_type
        .as_deref()
        .ok_or(ChromecastAdapterError::MissingContentType)?;
    let start_time_seconds = plan.start_time_seconds.unwrap_or(0) as f64;

    client
        .launch_app(&cast_app_for_receiver(&config.receiver_app_id))
        .await
        .map_err(|_| ChromecastAdapterError::CastControlFailed)?;

    let media = MediaInfo::new(media_url, content_type).stream_type(StreamType::Buffered);
    client
        .load_media(&media, true, start_time_seconds, None)
        .await
        .map_err(|_| ChromecastAdapterError::CastControlFailed)?;

    Ok(())
}

async fn resolve_target_endpoint(
    config: &Config,
    stable_device_id: &str,
) -> Result<ChromecastEndpoint, ChromecastAdapterError> {
    if let Some(device) = config
        .manual_devices
        .iter()
        .find(|device| device.stable_device_id == stable_device_id)
    {
        return Ok(ChromecastEndpoint {
            host: device.host.clone(),
            port: device.port,
        });
    }

    if config.live_discovery_enabled {
        let timeout = Duration::from_millis(config.discovery_timeout_ms.clamp(250, 30_000));
        let devices = oxicast::discovery::discover_devices(timeout)
            .await
            .map_err(|_| ChromecastAdapterError::UnknownTarget)?;
        if let Some(device) = devices
            .into_iter()
            .find(|device| discovered_device_stable_id(device) == stable_device_id)
        {
            return Ok(ChromecastEndpoint {
                host: device.ip.to_string(),
                port: device.port,
            });
        }
    }

    Err(ChromecastAdapterError::UnknownTarget)
}

fn cast_app_for_receiver(receiver_app_id: &str) -> CastApp {
    if receiver_app_id == DEFAULT_RECEIVER_APP_ID {
        CastApp::DefaultMediaReceiver
    } else {
        CastApp::Custom(receiver_app_id.to_owned())
    }
}

fn discovered_device_stable_id(device: &DeviceInfo) -> String {
    if let Some(uuid) = device
        .uuid
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return format!("chromecast:{uuid}");
    }

    format!("chromecast:{}:{}", stable_slug(&device.name), device.port)
}

fn stable_slug(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();

    if slug.is_empty() {
        "device".to_owned()
    } else {
        slug
    }
}

fn accepted_result(
    envelope: &AddonRendererAdapterCommandEnvelope,
    reason_code: &str,
) -> AddonRendererAdapterCommandResult {
    AddonRendererAdapterCommandResult {
        stable_device_id: envelope.stable_device_id.clone(),
        command: envelope.command,
        state: AddonRendererAdapterCommandState::Accepted,
        safe_reason_code: Some(reason_code.to_owned()),
    }
}

struct ChromecastEndpoint {
    host: String,
    port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_addon_protocol::{
        AddonRendererAdapterTransport, AddonRendererAdapterTransportUrlKind,
    };

    #[test]
    fn readiness_degrades_without_target_source() {
        let readiness = readiness(&Config::default());

        assert_eq!(
            readiness.status,
            AddonRendererAdapterReadinessStatus::Degraded
        );
        assert_eq!(readiness.reason_code, "no_target_source_configured");
    }

    #[test]
    fn manual_devices_are_published_as_local_targets_without_host() {
        let config = config_with_manual_device();

        let target = target_from_manual_device(&config.manual_devices[0]);
        assert_eq!(target.stable_device_id, "living-room");
        assert_eq!(target.target_kind, AddonRendererAdapterProtocol::Chromecast);
        assert_eq!(
            target.network_scope,
            AddonRendererAdapterNetworkScope::Local
        );
        assert!(target.media_capabilities.direct_play);
        assert!(target.control_capabilities.set_volume);

        let target_json = serde_json::to_string(&target).unwrap();
        assert!(!target_json.contains("192.168.1.50"));
    }

    #[test]
    fn command_plan_accepts_cast_safe_play_transport() {
        let config = config_with_manual_device();
        let envelope = play_envelope("https://nako.local/cast/media-ticket");

        let plan = build_command_plan(&config, &envelope).unwrap();

        assert_eq!(plan.stable_device_id, "living-room");
        assert_eq!(plan.command, AddonRendererAdapterCommand::Play);
        assert_eq!(
            plan.transport_mode,
            AddonRendererAdapterTransportMode::Direct
        );
        assert_eq!(
            plan.media_url.as_deref(),
            Some("https://nako.local/cast/media-ticket")
        );
        assert_eq!(plan.content_type.as_deref(), Some("video/mp4"));
        assert_eq!(plan.start_time_seconds, Some(12));

        let debug = format!("{plan:?}");
        assert!(!debug.contains("media-ticket"));
    }

    #[test]
    fn hls_plan_prefers_playlist_url() {
        let config = config_with_manual_device();
        let mut envelope = play_envelope("https://nako.local/cast/fallback.mp4");
        envelope.transport.mode = AddonRendererAdapterTransportMode::Hls;
        envelope
            .transport
            .urls
            .push(AddonRendererAdapterTransportUrl {
                kind: AddonRendererAdapterTransportUrlKind::Playlist,
                url: "https://nako.local/cast/master.m3u8".to_owned(),
                content_type: "application/vnd.apple.mpegurl".to_owned(),
                supports_range_requests: false,
            });

        let plan = build_command_plan(&config, &envelope).unwrap();

        assert_eq!(
            plan.media_url.as_deref(),
            Some("https://nako.local/cast/master.m3u8")
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
            let envelope = play_envelope(unsafe_url);

            let error = build_command_plan(&config, &envelope).unwrap_err();

            assert_eq!(error.safe_code(), "unsafe_transport_url");
        }
    }

    #[test]
    fn non_play_commands_validate_required_control_facts() {
        let config = config_with_manual_device();
        let mut seek = play_envelope("https://nako.local/cast/media-ticket");
        seek.command = AddonRendererAdapterCommand::Seek;
        seek.position_ms = None;
        assert_eq!(
            build_command_plan(&config, &seek).unwrap_err().safe_code(),
            "missing_seek_position"
        );

        let mut volume = play_envelope("https://nako.local/cast/media-ticket");
        volume.command = AddonRendererAdapterCommand::SetVolume;
        volume.volume_percent = None;
        assert_eq!(
            build_command_plan(&config, &volume)
                .unwrap_err()
                .safe_code(),
            "missing_volume"
        );
    }

    #[test]
    fn command_plan_rejects_wrong_protocol_and_unknown_target() {
        let config = config_with_manual_device();
        let mut wrong_protocol = play_envelope("https://nako.local/cast/media-ticket");
        wrong_protocol.target_kind = AddonRendererAdapterProtocol::DlnaRenderer;
        assert_eq!(
            build_command_plan(&config, &wrong_protocol)
                .unwrap_err()
                .safe_code(),
            "unsupported_protocol"
        );

        let mut unknown = play_envelope("https://nako.local/cast/media-ticket");
        unknown.stable_device_id = "unknown".to_owned();
        assert_eq!(
            build_command_plan(&config, &unknown)
                .unwrap_err()
                .safe_code(),
            "unknown_target"
        );
    }

    #[tokio::test]
    async fn dispatch_accepts_valid_plan_without_live_control() {
        let config = config_with_manual_device();
        let envelope = play_envelope("https://nako.local/cast/media-ticket");

        let result = dispatch_command(&config, &envelope).await;

        assert_eq!(result.state, AddonRendererAdapterCommandState::Accepted);
        assert_eq!(result.safe_reason_code.as_deref(), Some("plan_only"));
    }

    fn config_with_manual_device() -> Config {
        Config {
            manual_devices: vec![ManualChromecastDevice {
                stable_device_id: "living-room".to_owned(),
                display_name: "Living Room".to_owned(),
                host: "192.168.1.50".to_owned(),
                port: 8009,
                model: Some("Chromecast".to_owned()),
            }],
            ..Config::default()
        }
    }

    fn play_envelope(url: &str) -> AddonRendererAdapterCommandEnvelope {
        AddonRendererAdapterCommandEnvelope {
            adapter_id: "nako.official.chromecast-renderer".to_owned(),
            stable_device_id: "living-room".to_owned(),
            target_kind: AddonRendererAdapterProtocol::Chromecast,
            renderer_session_id: "renderer-session-1".to_owned(),
            playback_session_id: "playback-session-1".to_owned(),
            source_id: "source-1".to_owned(),
            command: AddonRendererAdapterCommand::Play,
            position_ms: Some(12_345),
            volume_percent: Some(50),
            transport: AddonRendererAdapterTransport {
                mode: AddonRendererAdapterTransportMode::Direct,
                expires_at: "2026-05-27T12:00:00.000Z".to_owned(),
                urls: vec![AddonRendererAdapterTransportUrl {
                    kind: AddonRendererAdapterTransportUrlKind::Stream,
                    url: url.to_owned(),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                }],
            },
        }
    }
}
