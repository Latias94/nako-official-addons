# Nako Chromecast Renderer

Official Chromecast renderer adapter sidecar for Nako.

This addon is intentionally narrow: Nako generates cast-safe media transports
and renderer command envelopes, while this sidecar handles Chromecast readiness,
discovery, diagnostics, and command translation.

## Safe Defaults

- Listens on `127.0.0.1:9120`.
- Live LAN discovery is disabled by default.
- Live Chromecast control is disabled by default.
- Manual device addresses are accepted through environment configuration but
  never echoed by health, diagnostics, or resource responses.

## Run Locally

```powershell
cargo run -p nako-chromecast-renderer
pwsh -File addons/chromecast-renderer/smoke.local.ps1
```

Manual device example:

```powershell
$env:NAKO_CHROMECAST_RENDERER_MANUAL_DEVICES_JSON = '[{"stable_device_id":"living-room","display_name":"Living Room TV","host":"192.168.1.50","port":8009}]'
cargo run -p nako-chromecast-renderer
```

Optional live discovery:

```powershell
$env:NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED = '1'
pwsh -File addons/chromecast-renderer/smoke.local.ps1 -RunDiscovery
```

Optional live control should only be used on a trusted LAN with a real
Chromecast-capable receiver:

```powershell
$env:NAKO_CHROMECAST_RENDERER_LIVE_CONTROL_ENABLED = '1'
```
