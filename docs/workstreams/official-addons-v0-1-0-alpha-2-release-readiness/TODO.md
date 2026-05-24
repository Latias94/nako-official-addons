# Official Addons v0.1.0-alpha.2 Release Readiness - TODO

Status: Blocked on publish approval
Last updated: 2026-05-24

Task IDs use the `OAR2` prefix.

## M0 - Scope

- [x] OAR2-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addons-v0-1-0-alpha-2-release-readiness]
  Goal: Open the alpha.2 release-readiness lane from package dry-run and live
  smoke findings.
  Validation: Workstream docs agree.
  Evidence: Workstream docs.
  Result: DONE 2026-05-24.
  Handoff: Continue with OAR2-020 version boundary repair.

## M1 - Version Boundary

- [x] OAR2-020 [owner=codex] [deps=OAR2-010] [scope=Cargo.toml,../nako/crates/nako-addon-client,../nako/crates/nako-addon-protocol,docs]
  Goal: Advance Rust crate package versions and addon package version to
  `0.1.0-alpha.2` while keeping Addon Protocol runtime compatibility at
  `0.1.0-alpha.1`.
  Validation: `cargo metadata --format-version 1 --no-deps`; focused manifest
  tests.
  Review: Do not change `ADDON_PROTOCOL_VERSION` unless the wire protocol
  compatibility decision changes.
  Evidence: Metadata output and manifest drift tests.
  Result: DONE 2026-05-24.
  Evidence: Cargo metadata reports `nako-metadata-scraper`,
  `nako-addon-client`, and `nako-addon-protocol` package versions
  `0.1.0-alpha.2`; manifest `protocol_version` remains
  `0.1.0-alpha.1`; focused and full metadata scraper tests passed.
  Handoff: Continue with OAR2-030 package verification.

## M2 - Package Verification

- [ ] OAR2-030 [owner=codex] [deps=OAR2-020] [scope=crates/nako-metadata-scraper,../nako/crates/nako-addon-client,../nako/crates/nako-addon-protocol]
  Goal: Prove metadata scraper packaging verifies against registry-shaped SDK
  dependencies.
  Validation: `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty`.
  Review: If SDK crates are not yet published, record the exact blocker and
  verify the SDK crate dry-runs separately.
  Evidence: Package dry-run output.
  Result: BLOCKED 2026-05-24 on publish order approval.
  Evidence: `nako-addon-protocol 0.1.0-alpha.2` dry-run passed.
  `nako-addon-client 0.1.0-alpha.2` dry-run is blocked until
  `nako-addon-protocol 0.1.0-alpha.2` exists on crates.io. The metadata
  scraper dry-run is blocked until `nako-addon-client 0.1.0-alpha.2` exists on
  crates.io.
  Handoff: Publish order is protocol, then client, then metadata scraper after
  user approval.

## M3 - Smoke And Closeout

- [x] OAR2-040 [owner=codex] [deps=OAR2-020] [scope=scripts,docs/workstreams]
  Goal: Prove smoke harness preflight behavior and record live smoke status.
  Validation: PowerShell parser checks; E2E preflight; live E2E smoke if Docker
  daemon is reachable.
  Review: Docker daemon absence is an environment blocker, not a code pass.
  Evidence: EVIDENCE_AND_GATES.md.
  Result: DONE 2026-05-24.
  Evidence: PowerShell parser check passed. E2E preflight now fails early with
  a Docker daemon diagnostic because Docker is not reachable on
  `//./pipe/docker_engine`.
  Handoff: Re-run live E2E smoke after Docker daemon is available.
