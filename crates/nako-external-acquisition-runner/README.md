# Nako External Acquisition Runner

Official fixture/no-op External Acquisition Runner sidecar for Nako.

This package implements the `external-acquisition-action` Addon Task contract.
It intentionally does not call qBittorrent, Transmission, aria2, ed2k handlers,
HTTP downloaders, or any other external runner. It exists to prove manifest,
health, idempotency, cancellation, status, progress, and redaction behavior
before real adapters are added.

Run locally:

```powershell
cargo run -p nako-external-acquisition-runner
```

Default endpoint: `http://127.0.0.1:9160/manifest.json`.
