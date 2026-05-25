# Nako Notification Bridge Addon

Official notification bridge Addon Sidecar for Nako.

Current release target: `v0.1.0-alpha.2`.

By default this sidecar stays ACK-only for event delivery. When `http_webhook`
is explicitly configured, it sends a fixed redaction-safe JSON summary to the
configured outbound HTTP webhook after receiving a `library.scanned` Addon
Event.

## Run Locally

```bash
cargo run -p nako-notification-bridge
```

Endpoints:

- `GET /manifest.json`
- `POST /health`
- `POST /events/library-scanned`
- `GET /ui/diagnostics`

## Local Smoke

Start the sidecar first, then run:

```powershell
pwsh -File addons/notification-bridge/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9110
```

The smoke fetches the manifest, calls health, and posts a safe
`library.scanned` envelope to `/events/library-scanned`. The response includes
payload keys but not payload values. The default smoke keeps `http_webhook`
disabled; provider send behavior is covered by fixture-backed Rust tests.

## HTTP Webhook Configuration

`http_webhook` is the first selected provider target. These settings are read
by the sidecar and surfaced only as redaction-safe diagnostics:

| Environment variable | Default | Purpose |
| --- | --- | --- |
| `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED` | `false` | Enables the `http_webhook` provider send path when the URL is valid. |
| `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL` | unset | Target webhook URL. Treat as secret-adjacent and inject through operator secret/config tooling. |
| `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET` | unset | Optional shared secret for the outbound header. Never logged or returned by diagnostics. |
| `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME` | `X-Nako-Notification-Secret` | Optional header name override for the shared secret. Diagnostics report only whether it is customized. |
| `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_TIMEOUT_MS` | `10000` | Provider HTTP timeout. Invalid or non-positive values fall back to the default. |

Nako core must not store the webhook URL or shared secret. In container or
orchestrated deployments, bind those values into the sidecar process through
the operator's secret reference mechanism, such as Docker Compose environment
substitution or Kubernetes Secret-backed environment variables.

The sidecar health response and `/ui/diagnostics` report only booleans and a
safe status (`disabled`, `missing_target_url`, `invalid_target_url`, or
`configured`). They do not echo the URL, secret header name, shared secret, or
event payload values.

The webhook payload is fixed JSON:

- schema id;
- event id, event kind, subject kind/id, occurrence time, and attempt number;
- sorted event payload keys only.

Provider HTTP `408`, `429`, and `5xx` responses are mapped to a safe retryable
sidecar failure so Nako's existing Addon Event retry can run. Other provider
HTTP failures are reported as non-retryable safe failures.

## Boundary

Nako core owns event facts, scheduling, Addon grants, delivery attempts, replay,
filters, and redaction-safe delivery attempts. This sidecar owns notification
message formatting, provider configuration, provider calls, and provider retry
decisions. This sidecar can call the configured HTTP webhook provider; it does
not call Telegram, Discord, Home Assistant, email, or any platform-specific
provider.
