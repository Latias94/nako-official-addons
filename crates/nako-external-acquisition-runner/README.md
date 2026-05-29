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

Optional host materialization:

```powershell
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_MATERIALIZATION_ENABLED = 'true'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_BASE_URL = 'http://127.0.0.1:3000'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_ADDON_TOKEN = '<addon runtime token>'
```

When materialization is disabled, the fixture uses a local no-op materializer so
the sidecar can still run smoke tests without a Nako host. When enabled, both
the Nako base URL and addon token are required; missing runtime credentials make
enqueue reject with a redaction-safe error.
