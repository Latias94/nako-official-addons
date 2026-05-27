# Official Addons v0.1.0-alpha.2 Release Readiness

Status: Complete
Last updated: 2026-05-28

## Problem

The post-alpha.1 official addon refactors moved Addon Task envelopes and
protected-write runtime helpers into public Nako SDK crates. Notification
bridge and Chromecast renderer work also added more official addon packages
that depend on the public Addon Protocol and official catalog crates. Local
path builds passed before the public crate line was complete. On 2026-05-27,
`nako-addon-protocol`, `nako-addon-client`, `nako`, and
`nako-official-addon-catalog` `0.1.0-alpha.2` were visible on crates.io, and
all official addon package dry-runs passed against registry-shaped
dependencies. `nako-notification-bridge`, `nako-metadata-scraper`, and
`nako-chromecast-renderer` `0.1.0-alpha.2` were then published to crates.io.
On 2026-05-28 the local Docker daemon was reachable, and the hosted official
addon E2E smoke passed against `ghcr.io/latias94/nako-server:0.1.0-alpha.2`
with the workspace metadata scraper sidecar.

## Target State

- `nako-addon-protocol`, `nako-addon-client`, and
  `nako-official-addon-catalog` crate package versions can be published as
  `0.1.0-alpha.2`.
- `nako-metadata-scraper`, `nako-notification-bridge`, and
  `nako-chromecast-renderer` package/addon versions advance to
  `0.1.0-alpha.2`.
- Addon Protocol runtime compatibility remains `0.1.0-alpha.1` unless the
  actual wire compatibility version is intentionally changed.
- `cargo publish --dry-run` proves all official addon packages can verify
  against registry-shaped SDK/catalog dependencies.
- The official E2E smoke preflight fails early when Docker daemon is not
  reachable instead of reporting a misleading green preflight, and the full
  hosted smoke passes when Docker is available.

## Scope

In scope:

- Rust crate package version metadata for public SDK crates.
- Official addon package/addon release version metadata.
- Docs and examples that display the addon package/image version.
- Package dry-run and focused verification gates.
- Recording the Docker daemon preflight behavior and successful live smoke.

Out of scope:

- Changing `ADDON_PROTOCOL_VERSION`.
- Pushing images.
- Tagging git releases.
- Splitting official addons into multiple installable sidecars.
- Reworking provider architecture beyond the already completed refactors.

## Architecture Direction

ADR 0033 separates Addon Version, Addon Protocol Version, and Rust crate
package version. This lane follows that decision:

- package versions move forward when Rust crates gain new API;
- addon version moves forward when the sidecar implementation changes;
- protocol version changes only when runtime compatibility semantics change.

## Closeout Expectation

This lane closes when package dry-run, focused tests, formatting, diff checks,
crate publication, and hosted Docker/server smoke evidence are recorded.
