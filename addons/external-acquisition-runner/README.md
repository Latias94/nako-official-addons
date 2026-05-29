# Nako External Acquisition Runner

Fixture/no-op official sidecar for the External Acquisition Runner contract.

The sidecar implements:

- `GET /manifest.json`
- `POST /health`
- `POST /tasks/external-acquisition-action`
- `GET /ui/diagnostics`

It accepts only host-owned opaque target references from the task payload. It
does not accept raw URLs or passwords, and it does not call external download
runners.

The fixture has an optional host materialization client. It is disabled by
default for local smoke tests. Enable it only when Nako provides the runtime
endpoint and addon token:

```powershell
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_MATERIALIZATION_ENABLED = 'true'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_BASE_URL = 'http://127.0.0.1:3000'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_ADDON_TOKEN = '<addon runtime token>'
```

Transmission profile configuration is opt-in. The checked-in manifest declares
the optional `transmission_password` secret field; do not put raw RPC passwords
in task payloads, logs, or diagnostics.

```powershell
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_ENABLED = 'true'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_PROFILE_ID = 'transmission'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_RPC_URL = 'http://127.0.0.1:9091/transmission/rpc'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_USERNAME = '<optional rpc user>'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_PASSWORD = '<optional rpc password>'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_TIMEOUT_MS = '10000'
```

Run locally:

```powershell
cargo run -p nako-external-acquisition-runner
```

Smoke test:

```powershell
pwsh -File addons/external-acquisition-runner/smoke.local.ps1
```
