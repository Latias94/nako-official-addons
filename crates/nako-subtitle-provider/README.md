# Nako Subtitle Provider

Official Nako subtitle provider Addon Sidecar.

The foundation release is read-only and fixture-backed. It declares one
`subtitle` resource at `/subtitle` with `subtitle_read` scope, returns
deterministic inline subtitle candidates for local smoke, and does not write
subtitle files, import subtitles into media sources, or call live subtitle
providers.

## Run

```powershell
cargo run -p nako-subtitle-provider
pwsh -File addons/subtitle-provider/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9140
```

## Configuration

- `NAKO_SUBTITLE_PROVIDER_LISTEN_ADDR`: bind address. Defaults to
  `127.0.0.1:9140`.
- `NAKO_SUBTITLE_PROVIDER_BASE_URL`: advertised base URL. Defaults to
  `http://127.0.0.1:9140`.
- `NAKO_SUBTITLE_PROVIDER_FIXTURE_PROVIDER_ENABLED`: enables deterministic
  fixture candidates. Defaults to `true`.
- `NAKO_SUBTITLE_PROVIDER_DEFAULT_LANGUAGE`: fallback subtitle language.
  Defaults to `en`.
- `NAKO_SUBTITLE_PROVIDER_DEFAULT_LIMIT`: default candidate limit. Defaults to
  `10`.
- `NAKO_SUBTITLE_PROVIDER_MAX_LIMIT`: maximum candidate limit. Defaults to `50`.

Live provider adapters are follow-on work and should stay disabled by default.
