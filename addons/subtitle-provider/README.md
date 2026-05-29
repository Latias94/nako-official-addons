# Nako Subtitle Provider

This is the operator-facing packaging folder for the official subtitle provider
sidecar.

The foundation sidecar is read-only:

- it declares `subtitle` at `/subtitle`;
- it requires only `subtitle_read`;
- it returns deterministic fixture subtitle candidates;
- it does not write subtitle files or import subtitles into Nako.

Run locally:

```powershell
cargo run -p nako-subtitle-provider
pwsh -File addons/subtitle-provider/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9140
```
