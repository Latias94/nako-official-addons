# Nako Notification Bridge Addon

This is the operator-facing packaging folder for the official notification
bridge sidecar.

The current slice is ACK-only by default and can send to the first selected
provider target, `http_webhook`, when explicitly configured:

- it declares `library.scanned`;
- it accepts `POST /events/library-scanned`;
- it returns a redaction-safe ACK with payload keys;
- it reads sidecar-owned HTTP webhook configuration from operator-provided
  environment variables;
- it reports only redaction-safe provider status in health and diagnostics;
- it sends a fixed redaction-safe JSON summary to the configured HTTP webhook;
- it does not send Telegram, Discord, Home Assistant, email, or other
  platform-specific provider calls.

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

Do not put raw webhook URLs or shared secrets in Nako core configuration. Use
the deployment operator's secret reference mechanism to inject them into the
sidecar process. Health and diagnostics expose only configured/valid/secret
presence booleans plus a safe provider status.

The outbound payload includes event identifiers and sorted payload keys only;
it does not include raw event payload values.

## Smoke

```powershell
pwsh -File addons/notification-bridge/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9110
```

The default smoke keeps `http_webhook` disabled and verifies the ACK output
reports provider status `disabled`. Provider sends are covered by
fixture-backed Rust tests so local smoke does not require live webhook secrets.
