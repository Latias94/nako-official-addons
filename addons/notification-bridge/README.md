# Nako Notification Bridge Addon

This is the operator-facing packaging folder for the official notification
bridge sidecar.

The current slice is ACK-only by default and can send through exactly one
explicitly configured provider:

- it declares `library.scanned`;
- it accepts `POST /events/library-scanned`;
- it returns a redaction-safe ACK with payload keys;
- it reads sidecar-owned provider configuration from operator-provided
  environment variables;
- it reports only redaction-safe provider status in health and diagnostics;
- it sends a fixed redaction-safe JSON summary to the configured HTTP webhook;
- it can send a fixed Discord-compatible webhook payload when
  `discord_webhook` is explicitly configured;
- it does not send Telegram, Home Assistant, email, or other platform-specific
  provider calls.

## Run

```bash
cargo run -p nako-notification-bridge
```

Default endpoint: `http://127.0.0.1:9110/manifest.json`.

Optional HTTP webhook configuration contract:

```bash
NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED=true
NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL=https://automation.example/nako
NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET=<operator-secret>
NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME=X-Nako-Notification-Secret
NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_TIMEOUT_MS=10000
```

Optional Discord webhook configuration contract:

```bash
NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED=true
NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_TIMEOUT_MS=10000
```

Optional safe summary template:

```bash
NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY='Nako {{event_kind}} event for {{subject_kind}} {{subject_id}}'
```

Optional provider attempt history capacity:

```bash
NAKO_NOTIFICATION_BRIDGE_PROVIDER_ATTEMPT_HISTORY_CAPACITY=20
```

Do not put raw webhook URLs or shared secrets in Nako core configuration. Use
the deployment operator's secret reference mechanism to inject them into the
sidecar process. Health and diagnostics expose only configured/valid/secret
presence booleans, provider send path count, safe provider status, and aggregate
configuration status. Configure at most one provider send path at a time; the
sidecar fails closed when multiple provider send paths are enabled. Templates
can use only whitelisted event facts and payload keys; raw event payload values
are not available. Provider attempt history is bounded and in-memory only; it
records actual provider send outcomes and failures for safe operator
diagnostics, not ACK-only disabled-provider records or provider retry.

The outbound payload includes event identifiers and sorted payload keys only;
it does not include raw event payload values.

## Smoke

```powershell
pwsh -File addons/notification-bridge/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9110
```

The default smoke keeps provider send paths disabled and verifies the ACK output
reports provider status `disabled`. Provider sends are covered by
fixture-backed Rust tests so local smoke does not require live webhook secrets.

Optional live provider smoke is local-only and skipped by default:

```powershell
$env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE = '1'
$env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_SIDECAR_BASE_URL = 'http://127.0.0.1:9110'
$env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_EXPECTED_PROVIDER_ID = 'http_webhook'
pwsh -File addons/notification-bridge/smoke.live.ps1
```

Only run live smoke against a sidecar that is already configured with exactly
one real provider send path. Do not run it in default CI and do not commit live
provider URLs or secrets.
