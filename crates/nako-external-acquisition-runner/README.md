# Nako External Acquisition Runner

Official External Acquisition Runner sidecar for Nako.

This package implements the `external-acquisition-action` Addon Task contract.
Fixture mode remains the default local profile. Transmission profile
configuration is opt-in and must consume host materialization before any raw
acquisition material reaches the adapter.

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

Optional Transmission profile configuration:

```powershell
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_ENABLED = 'true'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_PROFILE_ID = 'transmission'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_RPC_URL = 'http://127.0.0.1:9091/transmission/rpc'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_USERNAME = '<optional rpc user>'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_PASSWORD = '<optional rpc password>'
$env:NAKO_EXTERNAL_ACQUISITION_RUNNER_TRANSMISSION_TIMEOUT_MS = '10000'
```

Do not put raw Transmission credentials, magnet links, URLs, or passwords in
task payloads. They must not appear in diagnostics or task output.
