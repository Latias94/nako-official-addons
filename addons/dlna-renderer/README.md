# Nako DLNA Renderer

This is the operator-facing packaging folder for the official DLNA renderer
adapter sidecar.

The foundation sidecar is plan-only:

- it declares `renderer_adapter` at `/renderer-adapter`;
- it supports manual target discovery;
- it validates command envelopes and returns safe `plan_only` results;
- it does not perform SSDP discovery or UPnP control.

Run locally:

```powershell
cargo run -p nako-dlna-renderer
pwsh -File addons/dlna-renderer/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9150
```
