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

Run locally:

```powershell
cargo run -p nako-external-acquisition-runner
```

Smoke test:

```powershell
pwsh -File addons/external-acquisition-runner/smoke.local.ps1
```
