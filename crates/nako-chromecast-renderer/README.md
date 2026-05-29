# Nako Chromecast Renderer

Official Nako Addon Sidecar for Chromecast renderer adapters.

The sidecar exposes the Nako `renderer_adapter` resource at
`/renderer-adapter`. Nako owns playback policy, media authorization, and
cast-safe transport tickets; this sidecar only handles Chromecast-specific
readiness, target discovery, and command translation.

Default mode is safe for local development:

- manual devices can be configured deterministically;
- live LAN discovery is disabled unless explicitly enabled;
- live Chromecast control is disabled unless explicitly enabled;
- diagnostics never echo media URLs, bearer tokens, ticket values, or LAN host
  addresses.

## Development

```bash
cargo run -p nako-chromecast-renderer
cargo nextest run -p nako-chromecast-renderer --no-fail-fast
```

Default listen address: `127.0.0.1:9120`.

Useful environment variables:

- `NAKO_CHROMECAST_RENDERER_LISTEN_ADDR`
- `NAKO_CHROMECAST_RENDERER_BASE_URL`
- `NAKO_CHROMECAST_RENDERER_RECEIVER_APP_ID`
- `NAKO_CHROMECAST_RENDERER_MANUAL_DEVICES_JSON`
- `NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED`
- `NAKO_CHROMECAST_RENDERER_LIVE_CONTROL_ENABLED`
- `NAKO_CHROMECAST_RENDERER_DISCOVERY_TIMEOUT_MS`
- `NAKO_CHROMECAST_RENDERER_COMMAND_TIMEOUT_MS`
