# Evidence And Gates

## Required Gates

- `cargo nextest run -p nako-chromecast-renderer --no-fail-fast`
- `cargo fmt -p nako-chromecast-renderer -- --check`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-chromecast-renderer addons/chromecast-renderer docs/workstreams/official-chromecast-renderer-adapter`

## Optional Manual Gates

These are not CI gates because they require a Chromecast-capable receiver on the
same trusted LAN.

- `cargo run -p nako-chromecast-renderer`
- `pwsh -File addons/chromecast-renderer/smoke.local.ps1`
- `NAKO_CHROMECAST_RENDERER_LIVE_DISCOVERY_ENABLED=1 pwsh -File addons/chromecast-renderer/smoke.local.ps1 -RunDiscovery`

## Evidence Log

### 2026-05-27 - OCRA-010 through OCRA-050

- `cargo nextest run -p nako-chromecast-renderer --no-fail-fast`
  - Result: passed.
  - Coverage: 20 tests across config, manifest, Chromecast command-plan mapping,
    renderer adapter route envelope handling, redaction, and plan-only dispatch.
- `cargo fmt -p nako-chromecast-renderer -- --check`
  - Result: passed.
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-chromecast-renderer addons/chromecast-renderer docs/workstreams/official-chromecast-renderer-adapter`
  - Result: passed with the existing Windows line-ending warning for
    `Cargo.lock`.

Not run:

- Live Chromecast discovery/control smoke. It requires a Chromecast-capable
  receiver on the same trusted LAN and is intentionally optional for this lane.
