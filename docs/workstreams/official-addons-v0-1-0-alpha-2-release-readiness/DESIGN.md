# Official Addons v0.1.0-alpha.2 Release Readiness

Status: Blocked on publish approval
Last updated: 2026-05-25

## Problem

The post-alpha.1 official addon refactors moved Addon Task envelopes and
protected-write runtime helpers into public Nako SDK crates. Notification
bridge work also added a second official addon package that depends on the
public Addon Protocol crate. Local path builds pass, but
`cargo publish --dry-run` verifies against the already published
`0.1.0-alpha.1` crates and fails because those crates do not contain the new
SDK surface.

Live Docker/server smoke also remains a release proof, but the current local
environment has the Docker CLI installed without a reachable Docker daemon.

## Target State

- `nako-addon-protocol`, `nako-addon-client`, and
  `nako-official-addon-catalog` crate package versions can be published as
  `0.1.0-alpha.2`.
- `nako-metadata-scraper` and `nako-notification-bridge` package/addon
  versions advance to `0.1.0-alpha.2`.
- Addon Protocol runtime compatibility remains `0.1.0-alpha.1` unless the
  actual wire compatibility version is intentionally changed.
- `cargo publish --dry-run` proves both official addon packages can verify
  against registry-shaped SDK dependencies.
- The official E2E smoke preflight fails early when Docker daemon is not
  reachable instead of reporting a misleading green preflight.

## Scope

In scope:

- Rust crate package version metadata for public SDK crates.
- Official addon package/addon release version metadata.
- Docs and examples that display the addon package/image version.
- Package dry-run and focused verification gates.
- Recording the Docker daemon live-smoke blocker.

Out of scope:

- Changing `ADDON_PROTOCOL_VERSION`.
- Publishing crates or pushing images.
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

This lane can close when package dry-run, focused tests, formatting, and diff
checks pass, and when any live Docker/server smoke gap is recorded with the
exact environment blocker.
