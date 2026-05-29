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

Run locally:

```powershell
cargo run -p nako-external-acquisition-runner
```

Smoke test:

```powershell
pwsh -File addons/external-acquisition-runner/smoke.local.ps1
```
