# Nako DLNA Renderer

Official Nako DLNA renderer adapter Addon Sidecar.

The foundation release is plan-only. It declares one `renderer_adapter` resource
at `/renderer-adapter`, supports manual target discovery, validates
renderer command envelopes, and returns safe `plan_only` command results. It
does not perform SSDP discovery, UPnP SOAP actions, or live device control.

## Run

```powershell
cargo run -p nako-dlna-renderer
pwsh -File addons/dlna-renderer/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9150
```

## Configuration

- `NAKO_DLNA_RENDERER_LISTEN_ADDR`: bind address. Defaults to
  `127.0.0.1:9150`.
- `NAKO_DLNA_RENDERER_BASE_URL`: advertised base URL. Defaults to
  `http://127.0.0.1:9150`.
- `NAKO_DLNA_RENDERER_MANUAL_DEVICES_JSON`: JSON array of manual targets with
  `stable_device_id`, `display_name`, `host`, optional `port`, and optional
  `model`.

Live discovery and live control are follow-on work.
